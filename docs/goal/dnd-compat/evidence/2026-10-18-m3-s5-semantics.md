# M3-s5 交互语义收账 — UA 手势屏障 + relatedTarget，436 → 437 Pass（2026-10-18）

## 运行方式

- 通道不变：`make testharness-dnd`（test-guard 包裹，语料 pin
  315976933870b34d6ea30e3f6643403edae678ba）。
- 原始账本：[run1（终版）](2026-10-18-m3-s5-semantics-run1.txt)。
- 上轮基线：s4 账本 446 subtest 412P/29F/5T
  （[M3-s4](2026-10-18-m3-pipeline-corpus.md)）。

## 总账：446 subtest —— 437 Pass / 4 Fail / 5 Timeout（98.0%）

| 维度 | 数值 |
|---|---|
| s4 基线 | 412P / 29F / 5T |
| s5 净变化 | **+25 Pass**（29 fail → 4 fail，+5T 中 0 回归） |
| 手势屏障簇 | interactiveelements 24 fail 全闭（s4 聚类 1） |
| relatedTarget 簇 | events/dragenter-dragleave-related-target 翻绿（s4 聚类 2） |

## 剩余红面（9 案，与 s4 聚类 3/4 一致，均独立域）

1. **input 文本选择手势 3 案**（platform/mousedown-drag-select-input-display-change）：
   按压拖动改变 display 后选择/收点交互——文本选择引擎交互域，非拖放管线。
2. **declarative shadow DOM 重排 1 案**（platform/mousedown-move-inner-when-dragging-2）：
   mousedown handler 移动容器后 dragstart 派发链，依赖 shadow DOM 重排面。
3. **Timeout 5**：input_image 两案（注入 trace）、pointer-suppression（抑制完整面）、
   js-file 两案（DataTransfer File 面，M3 范围外）。

## 实施注记（三项语义，随本切片提交）

- **UA 拖拽手势屏障**（engine `drag_source::drag_gesture_barrier`）：文本控件
  （text 类 input/textarea/select）与 contenteditable true 状态元素按压-拖动被
  文本选择/控件交互消费，不得起拖且阻断向可拖祖先上溯（即使显式
  `draggable=true`）——spec processing model 只定义 draggable 链，交互例外为
  Chromium `Node::CanStartDrag` 同构的 UA 手势语义，语料验收
  `interactiveelements/*` 24 案。屏障标志随 hit-test meta/IPC 快照全链透传
  （`HitTestNodeMeta.drag_barrier` → `IpcHitTestNodeMeta.drag_barrier`）。
- **边界事件 relatedTarget**（webview `drag_input.rs`）：dragleave.related =
  将进入目标、dragenter.related = 上一 update 相位边界目标（`boundary_prev`）；
  首个真实目标 related=null（来源不计入链，Chrome/corpus 口径）。`DragEventPlan`
  携带 `related_target` 全程注入 DomEventDetail。
- **dragstart 发起 move 初始 target 更新**：发起按压后首个 move 若已离源，accept
  批次内一次完成 source→target 转换（leave@source(rel=target) → enter@target →
  over@target）；悬空目标（accept 时 target=None）补 leave@source 收尾。探针
  逐事件核对与 Chrome 序列一致（dragstart→drag→dragleave→dragenter→dragover→
  drop→dragend，relatedTarget 逐点匹配）。

## DC-4 门禁（2026-10-18）

- `make reftest`：**全绿零回归**——Layout 508/508（100.0%）、Text 204/204（100.0%）。
- `make test`（run2 全账本）：除最后腿外全腿绿——workspace 主腿、browser-shell
  341P、renderer bin/lib、browser bin（xvfb）/tests、quickjs workspace/bin 腿、
  clippy（与首腿并行无错）均通过。唯一红：最后腿 quickjs renderer lib 268/269，
  失败 1 例为既有失败（见下），本切片改动 crate 覆盖腿全绿。
- `make product-smoke`：**全绿**——form-input-perf-gate PASS；欢迎页 vs chromium
  oracle diff 15.48%（≤20%）；全部 struct-check PASS（card:4/shortcut:6/
  link-tile:4/article:1 等）。hit_test/IPC wire 加字段后渲染输出恒值面无回归。
- `make testharness-dnd`：437P/4F/5T（见 [run1]）。

## 观察项（与本切片无关的既有失败，clean-tree 复现）

- `zero-renderer page_scripts::tests::run_page_scripts_reports_uncaught_error_r2940`
  （quickjs 腿，page_scripts.rs:2624）：`window.onerror` 应收到抛错消息，实得
  `at <eval> (<input>:1:6)`——quickjs 栈消息头丢失方向的老问题（T64
  985a3303f 修过相似面），t5 b3f39e2dd（execute 错误栈捕获，早于 s4 入库）后
  存在。stash 本切片改动后 clean tree 同样失败（0.43s 确定性复现）——非 dnd
  引入。quickjs 错误报告域，待专项。

- `zero-browser-shell tests::bookmarks::test_bookmarks_multiple_folders`（bookmarks.rs:190，
  `list_in_folder(f1)` 得 3 期望 2）在 make test 全量并行腿偶发失败：本会话 3 次
  （均发生在有后台构建/测试并行的时段），随后安静环境复测不复现（串行 0/10、
  bookmarks 模块内并行 0/8、全量并行 0/8）。本切片零触碰 browser-shell；串行恒绿 +
  同一二进制败/过翻转，指向并行调度相关的跨测试干扰，机制未定位（测试体全用本地
  Vec + 全局 `COUNTER: AtomicU64` 原子唯一，算法层无解释路径）。待专项排查，
  不在本切片处理。
