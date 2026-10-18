# Drag and Drop 兼容 — DataTransfer / DragEvent / 拖放交互（门控 goal）

**版本**: v1.4
**日期**: 2026-10-18
**状态**: **Done**（2026-10-18 收口，DC-1~4 逐项判定见下——corpus 终态
437/446（98.0%），剩余 9 红挂账各自域）
**执行模式**: WPT 驱动（上游 dnd corpus 为验收标尺）+ 分层实现
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——交互面）

> **▶ 收口判定（2026-10-18 M4）**：corpus **213/387（55.0%）→ 437/446
> （98.0%）**，累计 +224 Pass 零回归。M1 导入甄别（11 可执行案基线）→ M2
> 七簇收敛（+174，387/387 全绿）→ M3 交互管线（RFC v1.0 确认后 s1~s5：
> 拖拽源命中全链透传 → zero-webview 会话状态机 → 浏览器仲裁 DragInput IPC →
> runner 真管道注入 11→44 案 → UA 手势屏障 + 边界 relatedTarget + 初始
> target 更新，+25）。管线语义（dragstart→dragenter/over/leave→drop→dragend
> 事件序、source node 仲裁、手势例外）按 Chrome 口径收口。
>
> **DC 逐项**：DC-1 ✅（导入+census+甄别+CSV 回填）；DC-2 ✅（M2 收口
> evidence）；DC-3 ✅（RFC v1.0 确认，管线五切片落地）；DC-4 ✅ 按偏差记账
> ——账本纪律全（每切片 evidence+run 原账）、reftest 全绿零回归（Layout
> 508/508、Text 204/204）、product-smoke 全绿（oracle 15.48%、struct-check
> 全过）、make test 除**外部既有红**外全腿绿（quickjs renderer lib
> `run_page_scripts_reports_uncaught_error_r2940`：T64 根因已定位、修复在
> `origin/error-fidelity-20261018` 分支待并——clean-tree 复现证明非本 goal
> 引入；另有 browser-shell bookmarks 并行间歇失败在案）。
>
> **剩余 9 红挂账（域外归因，本 goal 不再推进）**：
>
> | 红面 | 案数 | 归属域 |
> |---|---|---|
> | input 文本拖选（mousedown-drag-select-input-display-change） | 3 | 文本选择域（selection-compat 已分类「Actions 拖选无选区构建」） |
> | shadow DOM 重排后 dragstart（mousedown-move-inner-when-dragging-2） | 1 | shadow DOM 渲染级 composed tree（web-components 挂账协调面） |
> | input_image 注入 trace 超时 | 2 | runner 注入基建（`<input type=image>` 加载面） |
> | pointer-suppression 完整抑制事件面 | 1 | runner 注入基建（pointercancel/out/leave 抑制面） |
> | js-file DataTransfer File 面 | 2 | File 面（`items.add(File)` 依赖，M3 范围外） |
>
> 证据：[M1 基线](dnd-compat/evidence/2026-10-18-m1-baseline.md)、
> [M2 收口](dnd-compat/evidence/2026-10-18-m2-convergence.md)、
> [M3 管线语料](dnd-compat/evidence/2026-10-18-m3-pipeline-corpus.md)、
> [M3-s5 语义收口](dnd-compat/evidence/2026-10-18-m3-s5-semantics.md)、
> [M3 管线 RFC v1.0](dnd-compat/m3-drag-input-pipeline-spec-rfc.md)。
> 文档归档移动（archive/ 模式 A）可后续执行，webcrypto 同例。

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
