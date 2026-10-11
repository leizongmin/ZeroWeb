# File API 兼容 — Blob / File / FileReader / objectURL

**版本**: v1.5
**日期**: 2026-10-08（立项） | 2026-10-11（M1 基线 + M2-P1/P3/P4/P5）
**状态**: Active（M2 全簇收口——98.8%；残余 6F 跨域挂账，DC-2 下轮确认后 M3 归档）
**执行模式**: WPT 驱动（上游 FileAPI corpus 为验收标尺）+ 语义/实现修齐
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——文件与二进制面）
**运行时控制面板**: [fileapi-compat/master.md](fileapi-compat/master.md)

> **▶ 拆分动机（2026-10-08 预留编号立项，用户「候选 goal 也立项吧」）**：net-api
> M2 打过 Blob.type MIME parse-serialize 底，但 FileAPI 全语义面（FileReader 异步
> 读、createObjectURL 生命周期、slice/构造器）从未系统度量。
>
> **▶ 基线事实（2026-10-11 M1 实测 + M2 修齐）**：Blob/File/FileReader/FormData/
> createObjectURL 均已在 js_dom_shim 纯 JS 落地；M1 基线 62.3%（甄别与失败簇见
> [evidence/2026-10-11-m1-baseline.md](fileapi-compat/evidence/2026-10-11-m1-baseline.md)），
> M2-P1 构造器 WebIDL +124P → 87.2%（上游语料冲突 chromium 仲裁见
> [evidence/2026-10-11-m2-p1-constructor-webidl.md](fileapi-compat/evidence/2026-10-11-m2-p1-constructor-webidl.md)），
> M2-P3 readAs* 状态机 +32P → 93.6%（簇 53/53，见
> [evidence/2026-10-11-m2-p3-readas-state-machine.md](fileapi-compat/evidence/2026-10-11-m2-p3-readas-state-machine.md)），
> M2-P4 零散面 +21P → 97.8%（残余跨域挂账，见
> [evidence/2026-10-11-m2-p4-scattered.md](fileapi-compat/evidence/2026-10-11-m2-p4-scattered.md)），
> M2-P5 元素代理面 +5P → **98.8%**（String(el) spec class 串/HC value iterator/
> select 索引面，见
> [evidence/2026-10-11-m2-p5-element-proxy.md](fileapi-compat/evidence/2026-10-11-m2-p5-element-proxy.md)）。

## Mission

以 WPT FileAPI corpus 可执行子集为验收标准，收敛 Blob/File/FileReader/objectURL
语义。排除：streams 消费端（net-api 已归档域）、OPFS（storage-opfs 已归档域）、
drag-drop 文件入口（uievents/rendering 面依赖）。

## Done Criteria

- **DC-1** 导入 + 基线 + csv planned 行回填（可执行子集甄别落 evidence/）——✅ 2026-10-11
- **DC-2** Blob/File 构造器与 slice 语义、FileReader 读状态机、objectURL 生命周期
  对齐，分级通过率不再下行两个连续轮次
- **DC-3** 每修复附回归测试（imported-tests.txt 账本纪律）；make test 全绿、
  make reftest 零回归、product-smoke 逐字节恒值

## 里程碑

M1 导入与基线（含可执行性甄别）✅ → M2 语义/实现修齐 → M3 收口归档。

## 依赖约束（run-rules §9）

与 net-api（已归档）共享 streams 消费面为消费基础；与 storage 系（已归档）OPFS
面不重叠；渲染面零交集。
