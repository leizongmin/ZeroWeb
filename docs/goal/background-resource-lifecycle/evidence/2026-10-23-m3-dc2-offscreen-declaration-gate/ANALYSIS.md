# DC-2 续：offscreen 声明级门控（声明活跃 = 存在可见采样面）

**运行身份**：HEAD 1839c5d58（M1 布局切片）+ 本轮工作区。
**策略锚**：CSS Animations 无 offscreen 声明强制语义；「视口外动画不驱动 compositor 采样」对齐 Chromium 主线程合成剔除同族策略。M3 证据定性 offscreen 动画页 compositor 采样重光栅空转 0.498 cores（[DC-2 呈现链](../2026-10-22-m3-dc2-present-chain/ANALYSIS.md)）——声明照发 → `animations_active=true` → Get 帧采样持续重光栅不可见像素。

## 门控语义

- **门控点**：renderer `send_compositor_animation_state` 内 `advance` 输入侧（`paint_export::visible_declarations` 单口过滤）——按本帧存活作用域（`frame.animation_scopes`）过滤声明集。视口剔除（`cull_invisible_with_scopes`）丢弃区间端点/transform 槽被剔的作用域，无存活作用域的声明即无可见采样面，不进 compositor 状态。
- **空转消除**：全部声明被剔（视口外动画页）→ 发送空集（首帧必发/差分判变语义不变）→ compositor `animations_active` 不置位/回落 → 采样重光栅与 keep-alive 呈现链不对不可见动画空转。
- **恢复自动闭合**：元素回视口下一帧作用域重现 → 声明经 advance 集合差分恢复（generation 递增、compositor 按新代际采样）；无显式恢复动作。稳定集合（含稳定空集）判等零消息——offscreen 页稳态后呈现链零 churn。
- **engine 侧零改动**：缓存/增量臂（`compositor_export_cache`/`make_render_result`）照常携带全量声明 + 本帧作用域，过滤收敛在 renderer 发送单点。engine 侧导出期门控被否决：空缓存声明会断恢复链（作用域重现时无声明可发，painter 永不再发射该作用域），且增量臂多点改动违背单点收口。

## 已知边界（后续切片）

1. **offscreen 期间 gCS 覆盖表缺失**：`query_overrides::publish_declarations` 收过滤后 converted，被剔声明的键自然不进表——隐藏期间页面 JS 对视口外元素 gCS 动画属性回落底层级联（Chromium 对 offscreen 动画采样同样不保证；语义上「无采样面即无采样」自洽，但与「动画仍在跑」的页面预期有偏差）。
2. **offscreen 期间 animationend/cancel 事件丢失**：advance 只见可见集，被剔声明的终结事件不派发（`finished_reported`/`cancelled_reported` 不含它们）；回视口按重新开始记新锚点，不补发历史事件。
3. **动画时间轴重启偏差**：advance 锚点「消失剪除/重现重新开始」语义——offscreen 一段时间后回视口，动画从头播放而非按墙钟继续。在既有渲染近似模型边界内（隐藏页 rAF 泵已停，无帧产出即无采样推进）。

## 证据

1. **纯函数单测**（`animation_export_tests::visible_declarations_filters_by_surviving_scopes`）：匹配保留/不匹配剔除、空 scopes 全剔（offscreen 空集）、同 id 多 scope 不重复。
2. **runtime 级集成测**（真 `send_compositor_animation_state` + IPC drain，flag `ZW_COMPOSITOR_ANIMATIONS=1` + Compositor 模式）：
   - `compositor_state_sends_declarations_with_surviving_scope`：声明+匹配作用域 → 首帧必发且含声明；
   - `compositor_state_sends_empty_set_when_all_scopes_culled`：声明存在但作用域全被剔 → 空集发送（compositor animations_active 不置位的直接前提）；
   - `compositor_state_recovers_declarations_when_scope_returns`：空集代际 → 稳定空集零消息（判等不重发）→ 作用域重现声明恢复、generation 递增。
3. **全量 make test**：76 条 result 全绿（22623 passed / 0 failed）；workspace clippy 零警告。
4. **生产对照腿**（GUI X :20、1920x893 真实窗口、30s、sampler 250ms 进程树采样；本目录 `bg02-anim-compgui/`、`bg02-onscreen-compgui/`）：

| 腿 | 帧 dump | browser CPU | compositor CPU | renderer CPU |
|---|---|---|---|---|
| offscreen（M3 基线） | 512+ 封顶 | 3.301 cores | 0.498 cores | 0.011 cores |
| offscreen（**本轮门控后**） | 512 封顶 | **0.006** | **0.003** | 0.005 |
| onscreen（**本轮对照**） | 513 | 1.876 | 0.361 | 0.059 |
| onscreen（M3 基线） | 512+ 封顶 | 2.708 | 0.427 | 0.013 |

- offscreen 空转消除：compositor 采样重光栅 0.498→0.003 cores（**-99.4%**），browser present-blit 3.301→0.006（**-99.8%**）——`animations_active` 不置位后呈现链不对不可见动画重光栅/全窗位图往返。
- onscreen 合法工作不误停：compositor 0.361 与基线 0.427 同带（重光栅活跃）、browser 1.876 与基线 2.708 同量级（present 链活跃）——同代码同 flag，唯一变量是作用域存活，差分即门控生效的直接证据。
- 帧 dump 两腿同为 512 封顶：帧泵（keep-alive/自举探测 Get）与动画采样面解耦——offscreen 帧为 `fills=1` 空帧（无声明改写），门控消除的是每帧重光栅成本而非帧计数。GUI 腿无 CDP 端点（`probe_ok=0` 为 GUI 腿常态，同 M3 基线轮口径）。

5. **清理纪律**：frame dump `.bin` 已清理留 manifest；`browser.log`/`http.log` 按 `*.log` 忽略约定不入库；观测腿自建实例随腿收尾。

## 测试教训（记录备查）

1. `drain_messages` 是从 buf 头部全量解析的**快照**（Cursor 重放），非增量消费——轮询辅助若跨轮 `extend` 累计会把早期消息重复计入（首版三测连环红的直接原因）。
2. `send_regular` 经 compositor 发布线程异步写出——测试直调发送后须轮询等待（无 flush 原语）；「零消息」断言（稳定集合不重发）用等待窗口 + 快照长度，失败方向为假红不掩盖回归。
3. 集成测进程内 `set_var`/`remove_var` 翻 `ZW_COMPOSITOR_ANIMATIONS`——默认并行 test-threads 下跨测试互踩，须按仓库口径 `--test-threads=1`（Makefile renderer 腿已串行）。
