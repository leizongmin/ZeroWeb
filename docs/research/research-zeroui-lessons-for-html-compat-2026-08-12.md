# ZeroUI 对 HTML 兼容线的可迁移机制：身份、输入事务与可测试运行时

> 日期：2026-08-12
> 模式：源码深潜 + 多模块交叉验证
> 目标：评估 `../ZeroUI` 的自绘 UI 经验对 ZeroWeb HTML 行为兼容线 M0-M4 的增量价值

## 来源分级总表

| 分级 | 本文使用方式 | 代表来源 |
|---|---|---|
| 一手事实 | ZeroUI/ZeroWeb 源码、测试、经验文档及本次 scoped 测试 | [1]-[23] |
| 前期调研 | 现有 HTML 兼容调研与 Spec/RFC | [21][22] |
| 假设 | 尚需 ZeroWeb driving test 验证的迁移判断 | 各章显式标注 |
| 推理 | 从 ZeroUI 机制映射到 Web processing model | 各章显式标注 |
| 作者综合 | 迁移分级、优先级和 M0-M4 映射 | 各表格显式标注 |

## 30 秒速览

- ZeroUI 最值得借鉴的不是 Widget 外观，而是 `stable identity + retained state + centralized event routing + typed invalidation + headless query` 的组合。
- ZeroWeb 已经迁入分级失效、事件事务合帧、retained 表单状态和单一焦点/IME 路由，不应重复建设。
- 尚未迁入且价值最高的是：按下目标捕获、稳定节点引用与 generation、逐步交互 Scenario、自动化只读查询面、完整事件闭环诊断。
- `WidgetId + ComponentType + creation_epoch` 为 WebDriver opaque element reference 提供了直接架构先例，但 ZeroWeb 必须使用 DOM node identity，不能用 selector 冒充身份。
- ZeroUI 的 pointer capture 可防止 press/release 间轻微移动或重建导致状态丢失；ZeroWeb 应记录 `pressed_target`，但 click 生成仍须服从 Web 规范和拖动阈值。
- ZeroUI 的 `Scenario` Builder、按步骤报错、`assert_focused`、虚拟时间和 headless query 很适合 M0 自有测试；当前 `Scenario` 本身缺少直接端到端单测，因此只借鉴 API 形态。
- ZeroUI TextEditCore 不能直接复用：它使用 UTF-8 字节偏移，没有 `beforeinput`、取消回滚和 HTML activation semantics。
- ZeroUI checkbox/radio 的状态由应用 reducer 控制，也不能替代 HTML checkedness、dirty flag、radio group 和 reset 语义。

建议完整循环：

`平台事件 → 稳定目标 → prepare/dispatch/rollback/commit → typed effects → 单次失效合并 → 可查询快照 → 场景断言`

## 执行摘要

### 核心裁决

| 机制 | ZeroUI 证据 | ZeroWeb 状态 | 裁决 |
|---|---|---|---|
| 分级失效与事件事务 | `InvalidationFlags` [6] | `FrameInvalidation/FrameTransaction` 已实现 [19] | 已迁入，不重复 |
| retained 文本/IME 状态 | TextInput/TextEditCore [9][10] | `FormControlStateStore` 已实现 [20] | 已迁入模型，保留 Web 语义 |
| press target 捕获 | `pointer_capture` + 回归测试 [3][5] | 仅有 pointer target，缺 pressed identity [20] | M1 增补 |
| 稳定 identity + epoch | reconcile + epoch 测试 [4][5] | Spec 仍有 element identity TBD [21] | M1/M2/M4 增补 |
| 完整事件闭环 | dispatch → action → reducer → rebuild [18] | 三宿主仍在收敛 [21] | M2 强化 |
| Scenario + query | testing/headless [12][13] | 目前为散点 test helper [21] | M0 增补 |
| 确定性时钟 | Scheduler + advance_time [7][12] | HTML 测试主要用墙钟轮询 | M3/M4 可选增补 |
| automation command bridge | command + reply owner loop [15] | renderer 已有 IPC，但 automation 未闭合 [21] | M4 借鉴协议纪律 |
| ZeroUI TextEditCore 代码 | UTF-8 byte offsets [9] | DOM 需要 UTF-16 offsets [20][21] | 禁止直接复用 |
| Widget reducer 控件状态 | Radio app-owned state [11] | HTML 需要 UA checkedness/reset [21] | 禁止照搬 |

### 推荐修改

1. M0 增加测试专用 `HtmlScenario` 和 `PageQuery`，支持 selector 动作、逐步断言、阶段诊断。
2. M1 先建立最小 page-scoped node reference，再在 `PageInteractionState` 增加 `pressed_target`，把 release/cancel 回投给按下时的稳定目标。
3. M2 把 focus owner、retained form state 和 action target 全部迁到该 node reference。
4. M2 明确完整闭环：dispatch → default action transaction → DOM/apply → invalidation → publish，任何 adapter 不得只执行其中一段。
5. M3 为重复输入/焦点/reset/IME 增加短序列 stress tests；不把内存混沌测试设为 M0 阻塞项。
6. M4 automation request/response 使用 request id、单 owner、bounded queue、显式 shutdown 和 query snapshot；不引入 ZeroUI WebSocket 栈。

> **来源说明（执行摘要）**
>
> - **一手事实** [3]-[20][23]：ZeroUI 机制、ZeroWeb 当前状态和本次测试结果。
> - **前期调研** [21][22]：M0-M4 边界。
> - **推理**：迁移优先级基于“是否已迁入、是否符合 Web 规范、是否降低三宿主分叉”。
> - **作者综合**：核心裁决表与推荐修改。

## 1. 任务规划

### 1.1 5W1H

| 维度 | 当前理解 | 调研处理 |
|---|---|---|
| What | 找出 ZeroUI 对 HTML 行为线可借鉴的架构与关键细节 | 聚焦事件、状态、identity、调度、测试 |
| Why | 避免 ZeroWeb 再次踩自绘 UI 已解决的交互与运行时问题 | 只迁移机制，不迁移 Widget |
| Where | ZeroWeb M0-M4，重点 `page-runtime`、renderer、WebDriver、测试 | 显式排除 CSS/外观 |
| When | 以 2026-08-12 两仓当前 main 为准 | 不分析历史版本演进 |
| Who | HTML 兼容线实施者与测试维护者 | 输出可直接回补 Spec 的决策 |
| How | ZeroUI 类型定义 + 使用方 + 回归测试/learning 三证交叉 | 关键结论至少 2 个源码证据 |

### 1.2 术语映射

| ZeroUI 术语 | ZeroWeb 对应概念 | 注意 |
|---|---|---|
| `WidgetId` | DOM node reference / automation element ref | selector 不是 identity |
| `creation_epoch` | document generation / node generation | navigation epoch 仍需独立 |
| `UiEvent` | browser/renderer platform-neutral input | HTML 还需 DOM event semantics |
| `EventResult` | event handled/action effects | 不等价于 `preventDefault` |
| `InvalidationFlags` | `FrameInvalidation` | ZeroWeb 已扩展 style/publish/hit-test |
| `WidgetHost` | page interaction coordinator | 不能替代 DOM/JS runtime |
| `Scenario` | HTML 自有交互场景 DSL | 应优先按 selector/node ref，不按坐标 |
| `HeadlessRenderer` query | WebDriver/test harness live page query | 应查询 DOM 状态，不查询 Widget tree |

### 1.3 子任务

1. 识别 ZeroUI 已验证的交互闭环。
2. 核对 ZeroWeb 已迁入机制，排除重复工作。
3. 评估 stable identity、pointer capture、测试 DSL 和 command bridge。
4. 识别不符合 Web 规范的 ZeroUI 具体实现。
5. 给出 M0-M4 的增量修改与测试建议。

> **来源说明（第 1 章）**
>
> - **一手事实** [1]-[22]：两仓架构与当前 Spec。
> - **作者综合**：术语映射和子任务。

## 2. 已经迁入 ZeroWeb 的机制

### 2.1 分级失效与事务合帧

ZeroUI 使用 `NEEDS_LAYOUT/PAINT/SEMANTICS/COMPOSITE`，并规定 layout 隐含 paint [6]。ZeroWeb 已有更适合页面管线的 `NEEDS_STYLE/LAYOUT/PAINT/COMPOSITE/PUBLISH/HIT_TEST`，`FrameTransaction` 还支持嵌套事件事务和导航时丢弃旧工作 [19]。

裁决：不再从 ZeroUI 迁移失效类型。后续只补 action effects 到 `FrameInvalidation` 的确定映射和相关测试。

### 2.2 Retained 表单状态与 IME

ZeroUI 将 `TextInputState`、caret、selection 和 composition 保存在 widget 实例中 [9][10]。ZeroWeb 已把 value、UTF-16 selection、composition、dirty flag 和 revision 放入 `FormControlStateStore` [20]。

裁决：保留 ZeroWeb 模型。可借鉴“状态与绘制分离、preedit 不提交、失焦清临时状态”的原则，但不复用结构体。

### 2.3 单一焦点路由

ZeroUI 的 `WidgetHost` 统一处理 Tab、focused-only key/IME、Lost/Gained 事件和 focus scope [3]。ZeroWeb 已有 `PageInteractionState`、renderer focus owner 和 TabManager 回执同步 [20][21]。

裁决：不新增第二套焦点管理器。M1/M2 应强化现有 owner 的稳定 identity 和事件顺序。

### 2.4 渲染缓存与性能门禁

ZeroUI 的形状、glyph、图片和 GPU 资源缓存经验已被既有 `zeroui-gui-smoothness-migration-spec-rfc.md` 迁入 ZeroWeb [22]。本 HTML 行为线明确不处理 render-foundation。

裁决：不在本轮重复提出 GPU/cache 工作。

> **来源说明（第 2 章）**
>
> - **一手事实** [3][6][9][10][19][20]：两仓当前实现。
> - **前期调研** [21][22]：既有迁移范围。
> - **推理**：已存在等价或更完整实现的机制不再立项。

## 3. 高价值增量机制

### 3.1 Press target capture

ZeroUI 在 primary press 时记录 `pointer_capture`，release/cancel 不重新 hit-test，而是回投给按下时的 widget [3]。回归测试证明，即使 release 坐标移动到另一个控件，事件仍送回原 pressed widget [5]。

ZeroUI 的历史故障还表明，press/release 之间发生树重建会丢失 `pressed` 状态，导致按钮不触发 [16]。这与 ZeroWeb 目标页测试中的“按下和释放间允许小幅移动”属于同类风险。

建议 ZeroWeb 在 `PageInteractionState` 增加：

```rust
struct PressedTarget {
    node: PageNodeRef,
    button: PointerButton,
    press_position: (f32, f32),
}
```

约束：

- release/cancel 先解析 `pressed_target`，不以当前 hover target 替换。
- 导航、节点移除或 generation 失配时取消事务。
- capture 只保证 paired event 的目标稳定；是否合成 click 仍由 Web 规范和现有拖动阈值决定。

### 3.2 Stable node identity + generation

ZeroUI 的 reconcile 仅在 `WidgetId + ComponentType` 一致时复用实例，并用 `creation_epoch` 验证跨插入、删除和位置变化的身份稳定 [4][5]。匿名父容器导致整棵有状态子树重建的真实故障说明，只有叶子有 ID 仍不够，祖先 identity 链也必须稳定 [16]。

对 ZeroWeb 的直接启示：

- selector 是查询表达式，不是节点身份。
- WebDriver element ref、pressed target、focus owner 和 retained form state 最终应引用同一 page-scoped node identity。
- `navigation_epoch` 解决跨文档陈旧；`document_generation`/node generation 解决同文档 replacement。
- DOM 重新解析导致 identity 无法保留时，宁可返回 stale，也不能重新用 selector 命中新节点。

建议 M1 先建立 contract，M2/M4 扩大消费面：

```rust
struct PageNodeRef {
    navigation_epoch: u64,
    document_generation: u64,
    node: PageNodeHandle,
}

struct PageNodeHandle { /* opaque; representation decided by M1 spike */ }
```

具体 handle 来源仍需 M2 spike 验证，但 identity contract 应先固定。

### 3.3 完整事件闭环

ZeroUI 的窗口驱动闭环是：

```text
platform event
  -> host.dispatch_event
  -> emitted action
  -> app.dispatch reducer
  -> reconcile retained tree
  -> invalidation
  -> frame
```

移动端曾只完成命中/emit，未执行 reducer/rebuild/overlay sync，表现为“日志显示命中但 UI 不变化” [18]。这证明仅验证事件抵达不能证明交互完成。

对 ZeroWeb 的映射：

```text
platform/automation input
  -> DOM event dispatch
  -> default-action transaction
  -> script + UA mutations
  -> live DOM/apply
  -> invalidation
  -> publish
  -> browser query/snapshot
```

建议每个 adapter 只调用共享 coordinator，不允许只派事件或只改状态。测试必须断言最终 live DOM/输出/导航，而不只断言 `default_allowed` 或 snapshot sequence。

### 3.4 Scenario + query 测试层

ZeroUI 的 `Scenario` 用 typed steps 描述 click/type/key/resize/advance-time 和断言，并在失败中携带 step number [12]；`WidgetQuery` 提供 focused/visible/rect 查询 [13]。Headless server 则通过 command + reply 把同一 renderer owner 暴露给远程客户端 [15]。

这与 M0/M4 高度匹配。建议 ZeroWeb 增加测试专用：

```rust
HtmlScenario::new(&mut harness)
    .click("#name")
    .type_text("abc")
    .assert_value("#name", "abc")
    .press_key("Backspace")
    .assert_focused("#name")
    .run();
```

失败结果至少包含：

- step index 和 step description。
- selector/node ref。
- expected/actual。
- current URL/navigation epoch。
- last snapshot sequence。

限制：本次 `zeroui-testing` 测试通过，但源码中没有直接覆盖 `Scenario::run` 的端到端单测。因此应借鉴形态，并为 ZeroWeb 新 helper 自身补正常与失败测试。

### 3.5 自动化 command/reply 所有权

ZeroUI headless bridge 让 `!Send` renderer 固定驻留在专用 owner 线程，外部通过 bounded command queue 和 oneshot reply 操作，Drop/Shutdown 都显式收尾 [15]。

ZeroWeb 已使用独立 renderer 进程，无需复制线程桥或 WebSocket。但 M4 automation IPC 应保留同样的协议纪律：

- 每个 request 有唯一 ID。
- 一个 owner 串行修改 live page。
- query 与 mutation 都有显式 response。
- bounded pending map/queue。
- timeout、peer close、shutdown 返回确定错误。
- 绝不由 HTTP/WebDriver 线程直接持有页面内部可变对象。

### 3.6 确定性时间与短序列压力测试

ZeroUI Scheduler 用整数 tick，Scenario 能显式 `advance_time` [7][12]；chaos 测试可重复执行 click/type/delete/resize 并检查稳定性 [14]。

建议：

- M0 不引入通用虚拟事件循环。
- M3 对 focus/IME/reset 重复序列增加 20-100 轮 deterministic stress test，检查状态、revision 和内存不无界增长。
- M4 testharness timeout 继续用墙钟保护，但页面 timer 相关用例应通过可注入 test clock 推进。

> **来源说明（第 3 章）**
>
> - **一手事实** [3]-[7][12]-[18][20]：pointer capture、identity、测试和事件闭环。
> - **假设**：`PageNodeRef` 的 node_handle 来源需在 M2 spike 验证。
> - **推理**：建议保留机制语义，不复制 ZeroUI 的 Widget/线程/WebSocket 实现。
> - **作者综合**：ZeroWeb 类型草图与 M0-M4 映射。

## 4. 不应照搬的部分

### 4.1 TextInputState 的索引模型

ZeroUI `TextInputState` 明确使用 UTF-8 byte offset，便于直接操作 Rust `String` [9]。DOM `selectionStart/selectionEnd` 使用 UTF-16 code unit；ZeroWeb 当前状态也按 UTF-16 保存 [20]。

裁决：不能复用 `TextInputState` 或其 API。可复用测试思想，包括选区替换、CJK、代理对和 no-op 边界。

### 4.2 TextEditCore 的事件模型

ZeroUI TextEditCore 收到 key/IME 后直接改状态并 emit action [10]。它没有 DOM capture/bubble、`keydown.preventDefault`、cancelable `beforeinput`、不可取消 `input`、activation rollback。

裁决：不能把 TextEditCore 放入 page-runtime。ZeroWeb 必须保留 Spec 中的 prepare/dispatch/rollback/commit 事务。

### 4.3 Checkbox/Radio 的 reducer ownership

ZeroUI Radio 明确把组互斥交给应用 reducer，widget 只 emit value [11]。HTML radio group、checkedness、dirty checkedness、form reset 和 input/change 都是 UA processing model。

裁决：不能用 ZeroUI Radio/Checkbox 状态机替代 HTML 默认动作。只能借鉴 disabled gating、pressed pairing 和“控件只产生 typed intent”的分层。

### 4.4 Widget tree/semantics 不是 DOM

ZeroUI 的 semantics tree 适合 a11y 和 Widget 自动化，但它没有 DOM tree、IDL、事件传播和 form owner 的完整语义 [1][3]。

裁决：WebDriver 必须查询 live DOM node，不应以绘制树、hit-test selector 或 a11y semantics 冒充 DOM identity。

### 4.5 Headless WebSocket 栈

ZeroUI 使用 WebSocket JSON-RPC 让跨语言客户端控制 headless renderer [15]。ZeroWeb 已有 W3C WebDriver HTTP 与 browser-renderer IPC。

裁决：不引入 WebSocket/tokio-tungstenite。只借鉴 command/reply、错误码、超时和 owner 隔离。

> **来源说明（第 4 章）**
>
> - **一手事实** [1][3][9]-[11][15][20][21]：具体实现边界。
> - **推理**：Web 平台语义比通用 UI Widget 语义更强，不能以相似交互表象替代。

## 5. M0-M4 映射与证据 Gate

### 5.1 实施映射

| 里程碑 | 借鉴机制 | 具体增量 | 自有测试 |
|---|---|---|---|
| M0 | Scenario + query [12][13] | `HtmlScenario`、step diagnostics、live state assertions | helper 正常/失败测试 + 完整表单场景 |
| M1 | identity + press capture [3]-[5] | `PageNodeRef` contract、`pressed_target`、paired release/cancel | identity + 移动/重排/移除/导航 |
| M2 | identity migration + closed loop [4][5][18] | identity-backed focus/form state、完整 coordinator | 三宿主 identity/conformance |
| M3 | deterministic sequence [7][14] | focus/IME/reset 短序列 stress；same-source text boundary contract [17] | 状态、revision、内存、caret/IME rect |
| M4 | command/reply owner [15] | automation request id、bounded pending、query snapshot、shutdown | roundtrip/timeout/stale/peer-close |

### 5.2 证据矩阵

| 关键结论 | 来源 1 | 来源 2 | 一致性 | 置信度 | 处理 |
|---|---|---|---|---|---|
| pointer capture 值得迁移 | host 实现 [3] | runtime 回归测试 [5] | 一致 | 高 | M1 |
| stable identity 应替代 selector identity | reconcile [4] | 匿名父状态丢失故障 [16] | 一致 | 高 | M1/M2/M4 |
| Scenario/query 适合 M0 | Scenario [12] | Query/headless [13] | 一致 | 高 | M0 |
| Scenario 尚非充分验证资产 | API 源码 [12] | testing 实测无 Scenario 用例 [23] | 差异可解释 | 高 | 借形态并自测 |
| 事件必须闭合到 frame/query | driver/host [3] | mobile 断链故障 [18] | 一致 | 高 | M2 |
| command/reply 适合 automation | bridge [15] | ZeroWeb M4 需求 [21] | 一致 | 高 | M4 |
| 通用虚拟时钟不应进入 M0 | Scheduler 很轻量 [7] | M0 仅表单同步行为 [21] | 范围差异 | 中高 | M3/M4 |
| TextEditCore 不可直接复用 | UTF-8 byte 模型 [9] | DOM UTF-16 契约 [20][21] | 冲突 | 高 | 禁止 |
| Radio reducer 不可替代 HTML state | app-owned 设计 [11] | HTML action transaction [21] | 冲突 | 高 | 禁止 |
| 分级失效无需再迁移 | ZeroUI flags [6] | ZeroWeb flags/transaction [19] | ZeroWeb 更完整 | 高 | 不立项 |

Gate 结论：六项增量建议均有两份以上源码/测试证据；两项直接代码复用建议因 Web 规范冲突被否决。

### 5.3 优先级

| 优先级 | 项目 | 理由 |
|---|---|---|
| P0 | M0 `HtmlScenario`/`PageQuery` | 立即提升自有测试诊断质量 |
| P0 | M1 `PageNodeRef` + `pressed_target` | 同时保护 paired event、M4 stale ref 和 retained state |
| P1 | M2 完整闭环 trace/assertions | 防“事件到了但状态/帧没到” |
| P1 | M4 bounded request/reply owner | 自动化可靠性和资源收尾 |
| P2 | M3 deterministic stress/test clock | 在基础正确性稳定后投入 |

> **来源说明（第 5 章）**
>
> - **一手事实** [3]-[23]：全部证据矩阵来源。
> - **假设**：PageNodeRef 的具体 node_handle 承载仍待 M2 spike。
> - **作者综合**：M0-M4 映射和优先级。

## 6. 本次验证

使用 ZeroUI 自带 `test-guard` 执行：

```bash
scripts/test-guard.sh --memory-limit-mb 4096 --timeout-sec 300 -- cargo test -p zeroui-runtime --lib
scripts/test-guard.sh --memory-limit-mb 4096 --timeout-sec 300 -- cargo test -p zeroui-testing
scripts/test-guard.sh --memory-limit-mb 4096 --timeout-sec 300 -- cargo test -p zeroui-widgets --lib
```

结果：

| Crate | 结果 | 关键覆盖 |
|---|---|---|
| `zeroui-runtime` | 124 passed | pointer capture、epoch/reconcile、focus、invalidation、scheduler |
| `zeroui-testing` | 47 个可执行测试 passed | query/golden/diff/chaos/fake clock；Scenario 无直接测试 |
| `zeroui-widgets` | 281 passed | TextInput/IME/selection、button cancel、radio/checkbox |

测试运行只读取和编译 ZeroUI；未修改 ZeroUI 工作树。

> **来源说明（第 6 章）**
>
> - **一手事实** [23]：本次 guarded scoped test 输出。
> - **限制**：测试通过证明当前源码行为，不证明其符合 HTML 规范。

## 参考资料

| 编号 | 来源 | 类型 | 用途 |
|---|---|---|---|
| [1] | [`ZeroUI/AGENTS.md`](../../../ZeroUI/AGENTS.md) | 一手事实 | 分层、retained tree、adapter 边界 |
| [2] | [`ZeroUI core event`](../../../ZeroUI/crates/core/src/event.rs) | 一手事实 | 平台无关 UiEvent |
| [3] | [`ZeroUI WidgetHost`](../../../ZeroUI/crates/runtime/src/host.rs) | 一手事实 | 焦点、capture、路由、失效 |
| [4] | [`ZeroUI reconcile`](../../../ZeroUI/crates/runtime/src/host/reconcile.rs) | 一手事实 | stable identity 与 epoch |
| [5] | [`ZeroUI runtime tests`](../../../ZeroUI/crates/runtime/src/host_tests.rs) | 一手事实 | capture/reconcile/focus 回归 |
| [6] | [`ZeroUI invalidation`](../../../ZeroUI/crates/core/src/invalidation.rs) | 一手事实 | 分级失效 |
| [7] | [`ZeroUI scheduler`](../../../ZeroUI/crates/runtime/src/scheduler.rs) | 一手事实 | 整数 tick 与确定性调度 |
| [8] | [`ZeroUI IME controller`](../../../ZeroUI/crates/runtime/src/ime.rs) | 一手事实 | IME rect change detection |
| [9] | [`ZeroUI TextInputState`](../../../ZeroUI/crates/widgets/src/text_input.rs) | 一手事实 | UTF-8 byte 编辑模型 |
| [10] | [`ZeroUI TextEditCore`](../../../ZeroUI/crates/widgets/src/text_edit_core.rs) | 一手事实 | key/IME 直接状态转换 |
| [11] | [`ZeroUI Radio`](../../../ZeroUI/crates/widgets/src/radio.rs) | 一手事实 | app-owned radio state |
| [12] | [`ZeroUI Scenario`](../../../ZeroUI/crates/testing/src/interaction.rs) | 一手事实 | typed step 与诊断 |
| [13] | [`ZeroUI Query`](../../../ZeroUI/crates/testing/src/query.rs) | 一手事实 | focused/visible/rect 查询 |
| [14] | [`ZeroUI Chaos`](../../../ZeroUI/crates/testing/src/chaos.rs) | 一手事实 | 重复交互与内存稳定 |
| [15] | [`ZeroUI headless bridge`](../../../ZeroUI/crates/headless-server/src/bridge.rs) | 一手事实 | command/reply owner loop |
| [16] | [`匿名容器状态丢失`](../../../ZeroUI/docs/learnings/bugs/anonymous-container-rebuild-loses-widget-state.md) | 一手事实 | identity 故障证据 |
| [17] | [`caret hit-test 与重绘`](../../../ZeroUI/docs/learnings/bugs/code-caret-hit-test-and-repaint.md) | 一手事实 | 同源 metrics 与延迟帧 |
| [18] | [`移动端事件闭环断裂`](../../../ZeroUI/docs/learnings/bugs/mobile-pump-frame-missing-event-dispatch.md) | 一手事实 | dispatch/reducer/frame 闭环 |
| [19] | [`ZeroWeb frame invalidation`](../../crates/page-runtime/src/frame_invalidation.rs) | 一手事实 | 已迁入的增强失效模型 |
| [20] | [`ZeroWeb form control state`](../../crates/page-runtime/src/form_control.rs) | 一手事实 | UTF-16 retained 状态 |
| [21] | [`HTML 行为兼容 Spec/RFC`](../specs/html-behavior-compatibility-spec-rfc.md) | 前期调研 | M0-M4 当前设计 |
| [22] | [`ZeroUI 流畅性迁移 RFC`](../specs/zeroui-gui-smoothness-migration-spec-rfc.md) | 前期调研 | 已完成迁移范围 |
| [23] | 本次 ZeroUI guarded scoped tests | 一手事实 | 124 + 47 + 281 测试结果 |

## 质量审查

- [x] 10+ 多模块一手来源。
- [x] 核心推荐均有至少 2 个源码/测试证据。
- [x] 区分“已迁入、增量可借鉴、禁止照搬”。
- [x] 明确记录 ZeroUI 自身的测试缺口。
- [x] 未把通用 Widget 语义写成 HTML 规范事实。
- [x] 建议可直接回补 M0-M4 Spec/RFC。
