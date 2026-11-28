# s10k85 · 刀72：webkit-animation 族电池语境 4T 残相收敛（gCS StyleFlushBarrier 分段 apply ＋ 空关键帧注册 ＋ insertRule cssText）

日期：2026-11-28。任务面：docs/goal/wptfyi-dom.md（WPT 95% 冲刺 · 06 DOM 与 Shadow DOM）。

## 刀体（6 文件 +184/−2，含 Pin A/Pin B 回归钉测）

| 文件 | 变化 |
| --- | --- |
| `crates/engine/src/animation.rs` | 空关键帧列表照常注册（`points.is_empty()` 早退改插入 registry）＋ Pin A 钉测 `test_knife72_empty_keyframes_register_and_start` |
| `crates/engine/src/js_dom_shim/part06.js` | CSSOM `@keyframes` 规则工厂构造期初始化 `cssText`（不触发 flushFn）——缺此则 flushToOwner 文本写回静默丢规则 |
| `crates/engine/src/js_dom_shim/part01.js` | `getComputedStyle` 属性读前调用 `__zw_style_flush_barrier`（host 未注册 lenient 跳过） |
| `crates/engine/src/js_dom_bridge.rs` | `DomMutation::StyleFlushBarrier` 变体＋apply 端 no-op 臂（不落 DOM；视图重放等直达调用方安全跳过） |
| `crates/engine/src/js_dom_bridge/callbacks.rs` | `__zw_style_flush_barrier` 注册：队列尾非 barrier 才插（last-is-barrier 守卫） |
| `crates/webview/src/webview.rs` | `apply_pending_shared_mutations` 分段 apply 路径：barrier 切段→逐段 `apply_mutations_subset`→多界快照按「节点最早出现界」折叠 seed 基线；barrier 批不入两步切分（partition 会挪出原位）＋ Pin B 钉测 `test_style_flush_barrier_multi_boundary_per_node_baseline` |

## 三重根因

1. **空关键帧不注册**（animation.rs）：`@keyframes anim {}` 空帧列表早退不进 registry → `start_animation_with_underlying` 恒 false → animationstart/animationend 永不派发。css-animations-1 §keyframes：空 `<keyframe-block>` 列表是合法规则。WPT dom/events webkit-animation-*-event 四页 `insertRule('@keyframes anim {}')` 动态注入形 promise_test 全部永挂＝页级 Timeout。
2. **insertRule @keyframes cssText=undefined 被静默丢弃**（part06.js）：规则工厂不初始化 `cssText`，`flushToOwner` 的 `rulesCache.map(r=>r.cssText).join` 把 undefined 拼进写回文本 → 引擎重解析零 @keyframes → 动画钟 registry miss。probe8（静态 `<style>` vs `insertRule` 对照）实证仅动态注入形挂。
3. **结构臂 apply 把应用后样式写进过渡检测基线**：WPT 形「建元素＋gCS flush＋改类」同轮入队，单步/两步 apply 均把终态样式写进 `cached_styles` → 动画钟 tick old==new → transition 永不启动（css-transitions §starting-of-transitions：before-change style 应取上个 style change event 时的计算值；gCS 强制样式更新＝style change event 边界，cssom-1）。

## 分段 apply 设计（根因 #3 修复）

- shim 属性读入队 `StyleFlushBarrier` 标记；泵 drain（`apply_pending_shared_mutations`）遇 barrier 批切段按队列序逐段落 DOM，段间快照 `cached_styles`。
- **多界快照按「节点最早出现界」折叠**成 seed 基线：一次 drain 可含多个测试各自的 gCS 边界，任何单一 vintage 快照都顾此失彼——取尾界快照会让早界变更节点 old==new（过渡永不启动）。每节点正确 before-change vintage＝含该节点的最早界快照值；尾段新建节点不入任何界快照＝无 before-change（css-transitions：新建元素不起过渡）。
- 游标闭合：各段 `apply_mutations_subset` 内部按段长推进＋barrier 数补推进＝恒等 tail.len()。
- barrier 守卫迭代：首版「自上界 barrier 起新增条数 >0 才插」在 drain 清空重长后 `len==prev` 撞相等时静默吞掉整个窗（run4 T6 缺界实证）；终版 last-is-barrier（队尾非 barrier 即插，drain 后空队列照插＝空段早退＋边界快照取当前 cached_styles，正是该窗后续变更的正确 vintage）。

## 诊断定谳链（K72DBG 仪器化，五跑）

- run4（diag 页全复刻＋eprintln）：transition CREATED（dur=0.03 正常解析）但元素在相邻两 tick 间从 cached_styles 消失；tick4b 的 `baseline_present` 列恒 false（take 在 tick 开头消费，列无信号价值）。
- run6（真实页＋tail 组成＋remove 解析打点）：tail 32 条＝`C(n0) Ap(n0) B ?×7 C(n1) Ap(n1) B ?×7 C(n2..5)...`——elem1 样式变异落在界 1/界 2 之间（seg1），elem2 的落在 seg2；seed 取尾界快照（12 节点）→ elem1 old==green==new 不启动（Listener of transitionend 挂）、elem2 old=initial（在 seg1 快照）启动（Legacy listener of transitionend 过）＝5P＋页 T 逐测试吻合。
- 定谳机制与修复后复验一致：首界快照给 elem1（initial black）、次界给 elem2 → 双过渡启动。

## 单元与 solo 复验（修复后，/tmp/k72-v/）

- Pin B（webview 级多界基线）：`cargo test -p zero-webview --lib test_style_flush_barrier_multi_boundary` 1 passed（Run×2＋End×2；修复前形态 Run 仅 1）。
- Pin A（空关键帧）：`test_knife72_empty_keyframes_register_and_start` passed。
- 全量 zero-engine lib transition 套件 passed（test-guard 限内存墙钟）。
- solo FILTER：invoke-legacy **6/6**、probe9 1/1、probe8 2/2、webkit 四页各 13P＝52P/0T（EXIT 全 0）。
- 探针页（11 个 zz-k72-*）跑后全删（census 干净）；K72DBG 仪器化全部移除；fmt 零 diff。

## chain85 电池判读（bin f0719d32f373 腿绑 / 8795dacd5d51 提交域）

字节绑定：dom/cssanim 腿绑 f0719d32f373（发射时字节）；cssanim 的 make phony 依赖触发重编出 8795dacd5d51（＝提交域字节，doc comment 订正后 cargo 0.18s cached 恒等复证；语义面零差，二进制字节差异来自 -C metadata 源文件哈希指纹）；chain85 首射 san 腿断言 ABORT，chain85b 绑 8795dacd5d51 补跑 san/tt/ref。

### dom 腿（主目标达成）

P 57,766 / Fail 10 / Timeout 6 ＝ 失败行 20→16。multiset comm vs 刀71 dom2 名册 20 行：**REMOVED 恰 4 ＝ webkit 四页页级 Timeout 全翻绿（电池语境）**，ADDED 0，Fail 10 行恒等；P +55 逐页闭合＝webkit 4×9（4P→13P）＋Range-mutations 双页 +25/−6（页级 Timeout 行未变，已知截断抖动族）。提交域 bin（8795dacd5d51）solo 补验：webkit 四页各 13P/0F/0T rc=0（/tmp/k72-solo8795-*.log）。

### css-animations 语料（828/458/11 → 844/431/10，P+16/F−27/T−1）

逐行 F/T 源：runner transcript `/tmp/css-css-animations-corpus.log`（`Pass|Fail|Timeout <page> :: <subtest>` 行；语料腿主日志只打 summary 行）。multiset comm（469→441 行）：**REMOVED 32 ／ ADDED 4**。

REMOVED 32 构成：
- 页级 Timeout ×2：`animation-iteration-event-001/002.html`——刀72 前空关键帧不注册致 promise 链永挂；修复后 002 全绿（T→零残）、001 完成态仅余 1 条子测 F（`Got the expected elapsedTime in the correct order`，计入 ADDED）。
- 子测 Fail ×30：`event-dispatch.tentative.html` 15 条＋`parsing/keyframe-selectors.html` 7 条＋`KeyframeEffect-setKeyframes.tentative.html` 2 条＋`Element-getAnimations.tentative.html` 3 条＋`responsive/*` 3 条（15+7+2+3+3=30，与 REMOVED 总数闭合）。

ADDED 4 构成（均「页跑得更深」形，非既有通过项翻红）：
- `event-dispatch.tentative.html` 页级 Timeout ×1：刀71 电池该页完整跑完（30 条子测 Fail、无页级 T；页共 32 子测＝2P/30F）；刀72 电池在 30s 内部超时边界截断（15 Fail 行后挂，页级 T 接管）＝**页预算浮动族新成员**，非刀72 回归——提交域 solo 复核（`CSS_CORPUS_ONLY_SUBSTR=event-dispatch`，/tmp/k72-solo-page 系）：**2P/30F/0T 完整跑完**，与刀71 电池形态恒等；截断成因＝页内 30 条失败子测各烧事件等待、总时长贴 30s 边界＋电池窗负载抖动（battery 窗或含兄弟轮 solo 并行＝墙钟争抢族）。cancel/重启事件序 Fail 刀71 已有（`The order of the cancel events...` 全在刀71 名册），非新败。
- `Element-getAnimations.tentative.html :: getAnimations for CSS Animations that have finished` ×2：该页基线与 post 均为页级 T，修复后在挂死前多报出 2 条同参数化子测 F。
- `animation-iteration-event-001.html :: Got the expected elapsedTime in the correct order` ×1：001 页解除永挂后的残余子测 F。

净对账：F 30−3=27、T 2−1=1，与计数差（458→431、11→10）恒等闭合。语料总行 1297→1285（−12）＝event-dispatch（电池截断 −15F−2P+1T）与 Element-getAnimations（挂死点漂移，battery 24P vs solo 19P）两页可见性漂移，非账面缺案。残账沿既有簇：event-dispatch cancel/重启事件序（刀71 已有 30F 族，Web Animations 状态机事件域，非本轮引入）。

### san 腿（chain85b）

773P／F 集 5 行 md5 `e8ac48d450e8` ＝ 与基线 F 集恒等（零漂移）。

### tt / ref 腿（chain85b，EXPECT_BIN=8795dacd5d51 逐腿断言零漂移）

- tt：Pass 2290=2290、F/T 名册 32 行 multiset 双向 comm 零差＝恒等。
- ref：Layout 521/521＋Text 204/204＝725/725 100% 恒等，仅墙钟 0.39s→0.41s。
- 挂账：tt 语料含 `zz-s10k28-inline-gate-probe.html` 残留探针页（2P；chain84 已有＝刀72 前既存，本轮不动账），下刀清理。

### census

84,745 → **84,816**（Δdom +55 ＋ Δcssanim +16 ＋ san/tt/ref 恒等 0）；距 DC-3 校准线 94,093 差 **9,277**（用户拍板选 B、pin 31597693 不动；原跨周期线 99,947 单列）。
