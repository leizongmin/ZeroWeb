# Drag and Drop 兼容 — DataTransfer / DragEvent / 拖放交互（门控 goal）

**版本**: v1.0
**日期**: 2026-10-08
**状态**: Active·**门控**（M1 盘点 + M2 构造器/DataTransfer 面自主；拖放交互管线
须用户点名批准——99-webgl 门控模式）
**执行模式**: WPT 驱动（上游 dnd corpus 为验收标尺）+ 分层实现
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——交互面）

> **▶ 拆分动机（2026-10-08 html5test 缺口立项，用户「立项」）**：web-api-batch2
> （编号 92，已收口）明确挂账「DnD excluded pending host drag-input pipeline」；
> html5test 拖放节有分值。本 goal 把挂账正式化：纯 JS 面先行自主，交互管线门控。
>
> **▶ 基线事实（立项时点，M1 盘点复核）**：DataTransfer/DragEvent 构造器存在性
> 未盘点；draggable 属性/dragstart-dragover-drop 事件序依赖宿主拖拽状态机
> （mousedown 后拖动判据、drop target 命中）——host drag-input 管道缺失即挂账面。

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
