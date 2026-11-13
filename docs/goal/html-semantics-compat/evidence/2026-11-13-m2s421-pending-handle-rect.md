# M2 s421 收口账——pending 视图 handle rect（裸 handle 身份 pending 窗口真几何）+ embed/object 无内容态零 rect 门（embed-allowed-schemas 5F 清账）

- 日期: 2026-11-13；基线: HEAD 8605e2734（s420 收口），驱动: s420 挂账「A2 簇
  （load 同 turn 派发先于 apply——快照无 rect）」承前。
- 驱动案: embedded-content/the-embed-element/embed-allowed-schemas.sub.window.js
  5F——about:/blob:/data:/https:/http: 五案全 `assert_true: expected true got
  false`（has_width = `embed.offsetWidth > 0`，读点在 onload resolve 的
  continuation，与 load 派发同 turn）。

## 勘根（probe4/probe5 DIAG 时间线双面）

1. **A2 时序面（主根因）**: embed load 事件经 `_defer`（part01 queueMicrotask,
   budget 256）在 **宿主 apply+重渲染之前** 同 turn 派发——onload handler 里的
   `offsetWidth` 读的是 layout_rect_snapshot，而动态插入的 embed 尚未进任何
   快照（probe5：有 id embed onload 读 w0h0，同文件静态段 st0 读 w784h0——
   快照只在 apply 后才有该元素 rect）。`__zw_getBoundingClientRect`
   （callbacks.rs 注册，in-process runner 路径）只读快照，pending 窗口内恒
   空串 → 零 rect。
2. **A1 身份面（伴生）**: 无 id 的 JS createElement 产物身份 = 裸 handle
   `__n{n}`，不是合法 selector → `find_by_selector` 永不命中（probe4/probe5：
   no-id embed 全时间线点恒 0，含 raf2）。s395 已判明宿主侧全局 handle→selector
   解析是 **s284 雷区**（clone-img 真 rect 泄漏进 attr 期望面，regress
   naturalWidth 族 30 行）；宿主臂扩面必须绕开跨批 handle 解析。

## 修复（四件套）

1. **宿主 pending 臂**（callbacks.rs `__zw_getBoundingClientRect`）: sel 前缀
   `__n`（裸 handle 身份）+ mutation 队列 pending（`style_version > applied`）
   → `pending_view_handle_rect`；drained 态返空串（= 旧行为，跨批 handle 刻意
   不解析——s284 雷区不点亮）。spec 依据 CSSOM View：geometry 读先 update the
   style and layout steps（强制布局语义），快照 stale 不是跳过布局的理由。
2. **handle 视图槽**（computed_style_cache.rs）: `pending_view_handle_rect` 复用
   HANDLE_VIEW_CACHE 槽（parse 基座 html + `apply_dom_mutations_full` 重放
   `[applied..count)` pending 区间物化 handle 链）→ 新算布局
   （`LayoutEngine::new(vw, vh).compute` + HitTestCache + snapshot 填充）→
   HANDLE_RECT_CACHE 键含 **视口**（auto 几何随视口走，rect_bridge
   RECT_LAYOUT_CACHE 同理）；`set_pending_layout_viewport` 由 webview shim 安装
   点同步 config 尺寸（默认 (0,0) → 回落 1280×800 M3 约定）。
3. **shim 无内容态门**（part05.js `_zwRectIdent` 尾 + `_zwEmbedObjectNoContent`）:
   embed/object 且 **handle-only 人口**（sel 空 + 无 id——selector/id 身份在门
   前 early-return，静态快照面与 selector 族零触碰）→ `_zwObjectNavCache` entry
   收窄态（缺席/loading/error/fallback）→ rect 身份置 null → 宿主查询打掉、
   本地 definite 兜底原样（= 修复前零语义）。放行态 = 取回成功面：`done`
   （嵌套文档物化）/ `image`（可渲染 essence text/*、image/* 成功解码即 load——
   embed-allowed-schemas data:,Hello 驱动案）。spec 处理模型：无资源可取/
   取回未完成/取回失败/essence 不可渲染 → 元素 represents nothing。
   https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-embed-element-setup-steps

   过程勘误（message 级归因抓出）: 首版门只放行 `done`，solo embed-dir 出现
   embed-change-src-2 断言①回归（`represents nothing expected 0 but got 784`
   ——无 src embed 的 entry 缺席门闭正确，但 data:,Hello 驱动案落 `image` 态
   被误收窄 → ①过②错常态）。state join 对该文件 F→F 不可见，唯 message 级
   diff 可判——改门为白名单收窄态后 ①/② 语义归位。
4. **单测 part111**（js_dom_bridge_tests）: 7 断言面钉——①shim 级 no-src embed
   同 turn offsetWidth 0（门闭，embed-change-src-2 ① oracle）②data: 落地 done/
   image 后 offsetWidth > 0（门开 + pending 视图臂，corpus has_width 根因面）
   ③raw 队列 pending div 全宽 ④视口换档重算（cache 键含视口）⑤inline style
   精确几何 ⑥selector 身份不劫持（快照原值——s284 收窄面）⑦drain 后回落空串
   （旧行为）。raw 队列 handle 用 `__n9{k}` 高位段避开宿主 HANDLE_COUNTER 撞号。

## 验证

- unit part111: **1P**（test-guard cargo test --lib，dev profile）。
- js_dom_bridge 全模块: **1429P/7F**，失败清单对 s420 收口轮
  （/tmp/s420/s420-failnames.txt）**逐名 IDENTICAL 零位移**——img_runtime_
  deferred_settle_r5275 / svg_img_settle_density_r5277 /
  fetch_filtered_response_type_matrix / iframe_service_worker_messages /
  response_body_readable_stream_r2967 / t8k_websocket_error_nonterminal /
  t8k_websocket_event_flow（在册慢性红腿，与本轮无关）。
- solo the-embed-element 目录: **25P/2F**（7F→2F）——5 driving F→P 清账零扰动；
  余 2F 均在册渲染流面：embed-change-src-2 断言② `expected 100 but got 784`
  （intrinsic sizing——基线红 `expected 100 but got 0` 的 ① 侧已修，② 侧语义
  位移记档见口径）+ embed-svg-navigation-resets-size `expected 400 but got 784`。
- sweep ×2 join 对 s420 双基线（\x1f pairs、(key,state) 语义、stable 集）:
  - my1 vs b1: **stable=8449 state-diff=6 new=0 gone=40**——state-diff 恰为
    5 driving F→P + 簇 D detailed.optional 文件键 T→F（在册 T↔F 漂移，红集
    不变）；gone=40 全 sizes-auto.html 子测（双轮该文件均文件级 Timeout，
    超时前上报子测数随轮闪变——s420 收口轮同文件同 Timeout 行，F5 用户门在
    册面，Timeout 沾染键不入集逐案归因）。
  - my2 vs b2: **stable=8449 state-diff=6 new=2 gone=40**——同 5 driving F→P +
    簇 D detailed.optional F→T（漂移反向翻回）；new=2 = 簇 D optional 文件
    2 子测本轮成行 F（族内闪变，红集不变）。
  - my1 vs my2: **stable=8454 state-diff=1 new=2**——全簇 D 族 churn，红集恒定。
- make test 全量: **152 腿 ok** + engine lib 3912P/**7F 在册慢性红腿逐名
  IDENTICAL**（名单同上）+ clippy 腿 1 error（本轮新码
  `let...else` → clippy::question_mark，computed_style_cache.rs:1077——修复为
  `view.as_ref()?` 后 engine clippy 重跑零告警 + fmt 门 OK + engine lib 终码
  重跑 3912P/7F 名单不变）。过程注记：①首启误双跑 make test（s420 旧脚本
  残留调用 + 新脚本并发）——前者 target-disk-guard 测得 target 97GB 超阈值
  全量清空，双 make 竞争重链 target/test-guard 致 lld SIGBUS 双败；单跑重启
  后 fmt 门先抓出本轮新码格式漂移（cargo fmt 修复）再进测试段。教训：后台
  重活启动前先核对无同族残留进程/脚本。

## 口径

- 在册红 s420 收口 213F+14T → 本轮终态 raw **204F+14T**（轮1）/ 205F+15T（轮2，
  簇 D optional 2 子测成行 +1F +1T）——归一簇 D 漂移与 sizes-auto 超时上报
  闪变后稳定红面 **−5F**（embed-allowed-schemas 5 案清账），new 红 0。
- **embed-change-src-2 语义位移记档**: 基线红面 ②`represents its content
  navigable expected 100 but got 0`（无内容门未建——loaded embed 亦恒零）→
  本轮 ①represents nothing 恒 0 归位（Pass 面）+ ②`expected 100 but got 784`
  ——784 为块级替换元素占位全宽（视口 800−body margin），intrinsic sizing
  （SVG width=100 固有尺寸面）为 **渲染流** 结构缺位，挂账池在册
  （embed-svg-navigation-resets-size 同面）。基线红语义「loaded embed 恒零」
  已随本轮消亡，红面计数不变但失败轴从「恒零」转为「无固有尺寸」。
- **A1 drained 态刻意不开**: 跨批 handle（renderer apply 后）的裸 handle 几何
  仍走空串旧行为——宿主侧全局 handle→selector 解析是 s284 雷区
  （callbacks.rs:770 注记在案，s395 判明），本轮 pending 窗口臂刻意收窄于
  `style_version > applied`，drained 面不在修复范围。
- 下一步: 挂账池承 s416/s417/s420/s421（渲染流 6 面 + embed intrinsic sizing
  面 ×2 / sizes-auto F5 用户门 / 簇 D select 立项征询 / checkable parent-label
  2F / video 帧像素表面 FIXME / A1 drained 态与 s284 联合勘根）勘根首位。
