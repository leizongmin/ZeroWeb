# Drag and Drop 兼容 — DataTransfer / DragEvent / 拖放交互（门控 goal）

**版本**: v1.3
**日期**: 2026-10-18
**状态**: Active·**M3 交互管线推进中**（RFC v1.0 已确认——M1+M2 自主面收口
387/387；M3-s1~s4 已提交，s5 交互语义收口完成，corpus 437/446（98.0%））
**执行模式**: WPT 驱动（上游 dnd corpus 为验收标尺）+ 分层实现
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——交互面）

> **▶ 当前状态（2026-10-18 M3-s5）**：交互管线 RFC 确认后 s1~s5 连续推进，
> corpus **213/387（55.0%）→ 437/446（98.0%）**。s1 拖拽源命中标志全链透传 →
> s2 zero-webview 拖放会话状态机 → s3 浏览器仲裁与 DragInput IPC → s4 runner
> 真管道注入（语料 11→44 案）→ s5 交互语义收口（UA 拖拽手势屏障 + 边界事件
> relatedTarget + dragstart 发起 move 初始 target 更新，24+1 案翻绿）。管线
> 证据与剩余失败簇见 [M3 管线 corpus evidence](dnd-compat/evidence/2026-10-18-m3-pipeline-corpus.md)
> 与 [M3-s5 evidence](dnd-compat/evidence/2026-10-18-m3-s5-semantics.md)；
> RFC 见 [M3 管线 RFC v1.0](dnd-compat/m3-drag-input-pipeline-spec-rfc.md)。
> **剩余失败（9 案，语料外延已甄别）**：input 文本选择手势 3 案（选择/
> display 变更交互域）、declarative shadow DOM 重排 1 案、注入 trace 超时
> 2 案（input_image）、pointer-suppression 完整面 1 案、js-file File 面 2 案
> ——均为独立域，不阻塞管线语义收口判定。
> **下一步**：M4 收口归档（DC-4 门禁 + 剩余簇挂账评估）；若继续自主推进，
> 优先 input 文本选择手势簇（3 案同域）。

> **▶ 拆分动机（2026-10-08 html5test 缺口立项，用户「立项」）**：web-api-batch2
> （编号 92，已收口）明确挂账「DnD excluded pending host drag-input pipeline」；
> html5test 拖放节有分值。本 goal 把挂账正式化：纯 JS 面先行自主，交互管线门控。
>
> **▶ 基线事实（立项时点，M1 盘点复核）**：DataTransfer/DragEvent 构造器存在性
> 未盘点；draggable 属性/dragstart-dragover-drop 事件序依赖宿主拖拽状态机
> （mousedown 后拖动判据、drop target 命中）——host drag-input 管道缺失即挂账面。
> M1 盘点后事实上修：DataTransfer/DragEvent/DataTransferItemList/Item 构造器面
> 已在 web-api-batch2（R2937/R2948）落地（js_dom_shim/part05.js），draggable
> 反射 R2848 已落——M2 语义收敛而非从零建面。

## Mission

分层推进：①（自主）DataTransfer/DragEvent/构造器与属性面——纯 JS API，corpus
中可静态执行子集；②（门控）宿主拖拽状态机与事件序——mousedown→dragstart→
dragenter/over/leave→drop→dragend 管线，触 host-runtime 输入管道。

**关键约束**：
1. 门控切片启动前须用户点名（先例：navigation-compat M3 iframe / 99-webgl M2+）
2. 自主切片不预建交互管线占位代码（避免假面）
3. drop effect/effectAllowed 语义与 dataStore 类型映射按 corpus 口径

### 排除（明确不在范围内）

- 文件系统拖入（OS 文件拖放）— host 管道门控面的一部分，同门槛
- 触摸拖拽 — touch-events（编号 22）域协调

---

## Done Criteria

- **DC-1（自主）** 导入 + corpus census + 可执行子集甄别 + csv planned 行回填
- **DC-2（自主）** DataTransfer/DragEvent 构造器与属性面收敛出数字
- **DC-3（门控，须点名）** 交互管线与事件序收口；管线方案（输入注入架构）先出
  RFC 征询
- **DC-4** 回归测试账本纪律；make test 全绿、make reftest 零回归、product-smoke
  逐字节恒值

## 里程碑

M1 导入与甄别（自主）→ M2 构造器/DataTransfer 面（自主）→ M3 交互管线 RFC +
实现（**用户门控**）→ M4 收口归档。

## 依赖约束（run-rules §9）

host 输入管道改动与 keyboard/uievents 遗产（已归档）同域协调；与 22-touch-events
的触摸拖拽面互指不重叠。
