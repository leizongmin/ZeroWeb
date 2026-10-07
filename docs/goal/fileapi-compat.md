# File API 兼容 — Blob / File / FileReader / objectURL

**版本**: v1.0
**日期**: 2026-10-08
**状态**: Active（已立项——无用户门控项）
**执行模式**: WPT 驱动（上游 FileAPI corpus 为验收标尺）+ 语义/实现修齐
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——文件与二进制面）

> **▶ 拆分动机（2026-10-08 预留编号立项，用户「候选 goal 也立项吧」）**：net-api
> M2 打过 Blob.type MIME parse-serialize 底，但 FileAPI 全语义面（FileReader 异步
> 读、createObjectURL 生命周期、slice/构造器）从未系统度量。
>
> **▶ 基线事实（立项时点，M1 盘点复核）**：Blob 面部分存在（net-api M2 腿）；
> FileReader/createObjectURL/revokeObjectURL 可用性未盘点。

## Mission

以 WPT FileAPI corpus 可执行子集为验收标准，收敛 Blob/File/FileReader/objectURL
语义。排除：streams 消费端（net-api 已归档域）、OPFS（storage-opfs 已归档域）、
drag-drop 文件入口（uievents/rendering 面依赖）。

## Done Criteria

- **DC-1** 导入 + 基线 + csv planned 行回填（可执行子集甄别落 evidence/）
- **DC-2** Blob/File 构造器与 slice 语义、FileReader 读状态机、objectURL 生命周期
  对齐，分级通过率不再下行两个连续轮次
- **DC-3** 每修复附回归测试（imported-tests.txt 账本纪律）；make test 全绿、
  make reftest 零回归、product-smoke 逐字节恒值

## 里程碑

M1 导入与基线（含可执行性甄别）→ M2 语义/实现修齐 → M3 收口归档。

## 依赖约束（run-rules §9）

与 net-api（已归档）共享 streams 消费面为消费基础；与 storage 系（已归档）OPFS
面不重叠；渲染面零交集。
