# M3-s4 语料落账 — runner 真管道注入，11 → 44 案（2026-10-18）

## 运行方式

- 通道：`make testharness-dnd`（test-guard 包裹 `--per-proc-mem 4 --total-mem 8` 墙钟
  1500s，release runner）。语料 pin 不变（315976933870b34d6ea30e3f6643403edae678ba）。
- 注入模式：dnd 通道 Actions 指针链步改走宿主拖放状态机真管道
  （`PointerStepInjection::DragStateMachine` → `WebView::apply_drag_input`，与桌面浏览器
  同一实现）；uievents/pointerevents 通道不动（shim 脚本合成并存，RFC §5.6 已知取舍）。
- 原始账本：[run3（终版）](2026-10-18-m3-pipeline-corpus-run3.txt)、
  [run2（初版注入）](2026-10-18-m3-pipeline-corpus-run2.txt)。

## 总账：446 subtest —— 412 Pass / 29 Fail / 5 Timeout（92.4%）

| 维度 | 数值 |
|---|---|
| 案 | 44（既有 11 + 新增 33） |
| 全绿案 | 22（既有 11 全数全绿 + 新案 11） |
| subtest | 446（412 Pass / 29 Fail / 5 Timeout） |
| 既有 11 案回归 | 387/387 全绿，零回归 |
| 新增 33 案 | 334 subtest：25 Pass 案系数、11 案带红/超时（见聚类） |

## 新增扫描面（DND_TEST_SUBDIRS 扩展 + Actions skip 移除）

`interactiveelements` 24 案 / `platform` 5 案 / `events` 2 案 / 顶层 `js-file-*` 2 案
（skip 规则移除后入账）。

## 剩余红面聚类（实跑落账，待后续切片）

1. **交互内容拖拽起源语义**（interactiveelements 24 fail，`assert_false: dragstart
   should not fire`）：文本控件（input text/readonly/disabled、textarea、select）与
   contenteditable 在边框/文本选区拖拽时不得起拖（UA 交互内容例外）——引擎命中仲裁
   仅按 `draggable` 属性/默认可拖判定，未实现该例外。
2. **dragenter/dragleave relatedTarget**（events/dragenter-dragleave-related-target
   1 fail）：shim 事件面未携带进/出元素引用。
3. **拖拽中 DOM 突变链**（platform 4 fail）：mousedown handler 移动容器/display
   突变后 dragstart 仍应派发的链路。
4. **Timeout 5**：pointer-suppression（事件面抑制断言依赖）、input_image 两案、
   js-file 两案（DataTransfer 文件拖拽面——`items.add(File)` 依赖，M3 范围外）。

## 实施注记（对 s1/s2 的语义修正，随本切片提交）

- **拖拽源派至最近可拖祖先**：语料驱动修正——命中元素不可拖而祖先
  `draggable=true` 时，dragstart 目标须为祖先选择器而非命中元素
  （`ElementHit.drag_source_eligible: bool` → `drag_source: Option<String>`，
  spec drag-and-drop processing model）。
- **鼠标面与状态机并行**：初版注入只喂状态机，29 案 Timeout——测试经
  dragstart `preventDefault` 后等待普通 `mouseup` 收尾（自动化注入语义：取消拖拽 =
  鼠标流恢复）。修正为鼠标/指针事件面（shim 合成）与状态机并行喂入，dragstart
  接受后（`DragApplyReport.session_active`）抑制鼠标面至 dragend，取消/未武装则
  鼠标面正常——run2 29 Timeout → run3 5。
