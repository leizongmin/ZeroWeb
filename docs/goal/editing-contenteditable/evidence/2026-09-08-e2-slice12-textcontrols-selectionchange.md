# E2 切片 12 — selection/textcontrols selectionchange 全语义面（2026-09-08）

**用例来源**：上游 WPT `web-platform-tests/wpt` @ `315976933870b34d6ea30e3f6643403edae678ba`
`selection/textcontrols/` 首批 3 案，经 `fetch-selection-subset.sh`（`TEXTCONTROLS_CASES`）
拉取到 `wpt-data/selection/textcontrols/`（gitignored）。runner `SELECTION_TEST_SUBDIRS`
追加 `selection/textcontrols` 目录。

**导入清单**：
- `selectionchange.html` — 60 subtests（4 collector × 15 语义：selectionStart/End
  setter、setSelectionRange、select、setRangeText 的派发/不派发/去重面）
- `selectionchange-bubble.html` — 4 subtests（text control selectionchange bubbles=true）
- `onselectionchange-content-attribute.html` — 2 subtests（内容属性 handler 编译触发）

**排除项**（有据）：`focus.html`（pointer 拖选 + focus 取消扩展——依赖布局命中测试，
runner 无布局 rect，同 anchor-removal Actions 拖选归因 renderer S3）；`click-input-after-
iframe-focus.html` / `initial-selection-*`（focus 交互/渲染面）；`selectionchange-on-
shadow-dom.html`（shadow DOM 面，js-dom 域）；`selectionchange.html`(textcontrols 之
外的 caret/bidi/contenteditable 子目录) 归后续切片。

## 本轮结果

```
selection 全套件：2993P / 6F（基线 2928P/5F，净 +65P——新增 66 subtest 全 Pass）
三连跑稳定；textcontrols 三案 25 连跑 0 Fail/Timeout
keyboard 套件 18P/7F/2T 零回归
```

## 实现语义（shim）

spec 依据：selection-api「Firing selectionchange event」（element target bubbles=true /
document target bubbles=false）+「Scheduling selectionchange event」（has-scheduled
按 target 去重）+ HTML「To set the selection range」（extent/direction 实际变更才
queue select/派发任务）。

1. **part06 排程器**：pending 条目改为 `{node, bubble}`（同任务多 target 各持自己的
   bubbles flag）；去重按 `entry.node === target`（spec has-scheduled 语义）。
2. **part03 setSelectionRange**：同值早退——`(start,end,direction)` 全等不排程。
3. **part03 setRangeText**：末步「Set the selection range with selection start and
   selection end」→ `(start,end)` 变更才排程（preserve 不变形态 0 事件）。
4. **part04 select()**：spec「Set the selection range with 0 and infinity」→ 已全选时
   不排程（select() twice = 1 事件）。
5. **part04 selectionStart/End/Direction setter**：前值/后值比对，变更才排程。
6. 全部 text control 调用点 `bubbles=true`；document 调用点（`_zwSync`）不变。

## 修复的 runner 基建缺陷（本轮重大发现）

**timer stub 单 turn 存活缺陷**：`V8Sandbox::execute` 每次执行都把 `register_callback`
注册的原生回调重新 `global.set` 到全局对象（v8_runtime.rs execute 内 for-callbacks
循环）——runner `timer_stub` 对 `__zw_setTimeout` 的 JS 层赋值只存活一个脚本 turn；
下一个 `execute_script`（probe 泵）起，shim `setTimeout` 静默回落 host 真线程路径
（`drain_next_async_callback_if_pending` 每次 execute 只 resolve 一个、到达序随机）。
后果：页面加载 turn 之后排的所有 timer 派发顺序随机——本次 WPT textcontrols 断言族
~30%+ flake 的根因，也是 master.md 既有记录「onselectionchange-on-document timer 时序
flake」的真实根因。

**修复**：stub 换独立名 `__zw_test_setTimeout`（不在 host 注册表 → 永不被 rebind）；
shim `setTimeout`/`setInterval` 经 `_zwHostSetTimeout()` 优先探测该名，生产路径（无
stub）回落 `__zw_setTimeout` 原语义零回归。全部 timer 从此走 stub 确定性队列（push 序
FIFO + due 分割保序）。

**验证**：最小复现探针（4 collector × selectionStart 语义）修复前 ~27-50% 失败率 →
修复后 textcontrols 三案 25 连跑 0 失败；make test 19,005 全绿（含 runner 既有
send_keys/timer 相关测试）零回归。

## 残余记录

- `onselectionchange-on-document.html` 第 3 subtest（'task to fire selectionchange
  event gets queued each time'）在 FILTER 单案跑下稳定报 `IndexSizeError: The given
  offset is out of bounds` unhandled rejection（`setPosition(container, 2)` 于
  innerHTML pending 未 apply 时读 fusion childNodes 越界）；全量套件跑下通过
  （三连跑稳定）。基线（stash 验证）同型 flake 亦存在——既有问题，非本轮引入，
  待 innerHTML fusion 视图与 selection 端点校验协调切片。

### 同日追加：flake 精确归因（最小上下文复现）

复现形态：subtest 1 + subtest 2（各做一次 `container.innerHTML = ...` + setPosition）
前置后再跑 subtest 3 原样——第 4 次 `setPosition(container, 2)`（spin 后）抛
IndexSizeError，**抛点实测 `container.childNodes.length === 0`**。

机制定位：同 sel 连续多次 innerHTML 赋值（s2 一次 + s3 一次）后，spin await 边界处
`#container` 的融合 childNodes 视图塌缩为 0（R304 挂槽的解析 wrapper 从 overlay 桶
消失 + `_zwChildBaseCache['#container']` 被 setter 置空 `[]` 后未再失效）——第二次
innerHTML 的 queue-side invalidate 与 `_zwFragmentAdded` 预注册的桶条目存在时序耦合。
单案最小形态（单 subtest、单次 innerHTML）700+ 连跑不复现；全量套件（fresh WebView
per case、无 stacked innerHTML）三连跑稳定通过。

定性与切分：js-dom 共享面（fusion 视图 R51c/R304/R380 族）的既有深缺陷，非本轮
selectionchange 变更引入；修复需专门切片（stacked same-sel innerHTML 的桶生命周期
重整），记入 js-dom/编辑协调点，不阻塞本轮资产化与 goal 收口判定。
- textcontrols `selectionchange.html` 断言依赖事件在**单 spin** 内到达；stub 队列
  FIFO 下已稳定（25 连跑）。

## 门禁

- `make test`：19,005 passed / 0 failed（guard 包裹）
- `cargo clippy --workspace --all-targets -- -D warnings`：零警告
- `cargo fmt --all -- --check`：无 diff
- 单测：`test_selectionchange_textcontrol_full_semantics_r3254_e2_slice12`（六组断言）
