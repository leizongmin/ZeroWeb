# Compression 兼容 — CompressionStream / DecompressionStream

**版本**: v1.0
**日期**: 2026-10-08
**状态**: Active（已立项——实现型小套件，M2 为实现轮）
**执行模式**: WPT 驱动（上游 compression corpus 为验收标尺）+ 实现
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——数据面）

> **▶ 拆分动机（2026-10-08 预留编号立项，用户「候选 goal 也立项吧」）**：streams
> 底座由 net-api M4 修齐（CountQueuing/pipeTo/品牌检查等），CompressionStream/
> DecompressionStream 是其上的薄封装 API，corpus 小、收益直接。
>
> **▶ 基线事实（立项时点，M1 盘点复核）**：API 存在性未知——若缺失基线即 0%，
> 属实现型套件（非纯语义修齐）；实现候选 = 复用仓内既有 flate 系依赖（盘点确认，
> 不新引重依赖）。

## Mission

以 WPT compression corpus（gzip/deflate-raw 格式、分块输入/flush 语义、无效
流错误）为验收标准，实现并对齐 CompressionStream/DecompressionStream。排除：
HTTP Content-Encoding 传输解压（zero-net 既有域）、文件压缩格式（zip 等，未立项）。

## Done Criteria

- **DC-1** 导入 + 基线 + csv planned 行回填
- **DC-2** gzip/deflate-raw 全格式对齐（roundtrip/分块/错误语义），分级通过率
  不再下行两个连续轮次
- **DC-3** 回归测试账本纪律；make test 全绿、make reftest 零回归、product-smoke
  逐字节恒值；**新依赖引入须在 evidence/ 记录选型理由（准则：优先复用既有依赖）**

## 里程碑

M1 导入与基线 → M2 实现与语义对齐 → M3 收口归档。

## 依赖约束（run-rules §9）

与 net-api（已归档）streams 底座为消费关系；Cargo 依赖变更走 workspace 统一
评审（perf-gate config_hash 敏感）。
