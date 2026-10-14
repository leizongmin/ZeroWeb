# Compression 兼容 — CompressionStream / DecompressionStream

**版本**: v1.1
**日期**: 2026-10-15（v1.0 立项 2026-10-08）
**状态**: Active——**M1 完成**（基线 165/309 = 53.40%），M2 实现轮进行中
**执行模式**: WPT 驱动（上游 compression corpus 为验收标尺）+ 实现
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——数据面）

> **▶ 当前状态（2026-10-15 M1 收）**：corpus 19 案导入（18 执行，idlharness 规则
> 排除），基线 165P/141F/3T = 53.40%，失败六簇全归因（evidence/
> [2026-10-15-m1-baseline.md](compression-compat/evidence/2026-10-15-m1-baseline.md)）。
> **M2 五项修复**：①构造器 TypeError（WebIDL enum 转换）；②严格 BufferSource
> chunk 校验；③流式 update/finish 状态机（替 buffer-then-flush——3 案死锁根因）；
> ④解压错误全量 TypeError 化 + 手工帧校验（zlib CMF/FLG/FDICT/adler32、gzip
> CRC32/ISIZE、截断/多余输入检测——flate2 读路径不校验 trailer）；⑤空流 done
> 语义。brotli 70 子测试 mission-defer 挂账（不新引重依赖）。依赖复用：flate2 =
> engine 既有直接依赖，M2 零新增。
>
> **▶ 拆分动机（2026-10-08 预留编号立项，用户「候选 goal 也立项吧」）**：streams
> 底座由 net-api M4 修齐（CountQueuing/pipeTo/品牌检查等），CompressionStream/
> DecompressionStream 是其上的薄封装 API，corpus 小、收益直接。

## Mission

以 WPT compression corpus（gzip/deflate-raw 格式、分块输入/flush 语义、无效
流错误）为验收标准，实现并对齐 CompressionStream/DecompressionStream。排除：
HTTP Content-Encoding 传输解压（zero-net 既有域）、文件压缩格式（zip 等，未立项）。

## Done Criteria

- **DC-1** ✅ 导入 + 基线 + csv planned 行回填（2026-10-15）
- **DC-2** gzip/deflate-raw 全格式对齐（roundtrip/分块/错误语义），分级通过率
  不再下行两个连续轮次
- **DC-3** 回归测试账本纪律；make test 全绿、make reftest 零回归、product-smoke
  逐字节恒值；**新依赖引入须在 evidence/ 记录选型理由（准则：优先复用既有依赖）**

## 里程碑

M1 导入与基线（✅ 2026-10-15）→ M2 实现与语义对齐（进行中）→ M3 收口归档。

## 依赖约束（run-rules §9）

与 net-api（已归档）streams 底座为消费关系；Cargo 依赖变更走 workspace 统一
评审（perf-gate config_hash 敏感）。M1 盘点：flate2 为 engine 既有直接依赖
（WOFF zlib 解码先例），M2 不新增依赖。

## 轮次账

- **2026-10-15 M1**：runner 通道（`testharness-compression` 子命令 + skip 规则
  + Makefile target + fetch 脚本补 pako/media 依赖）；基线 53.40%，六簇归因，
  evidence 落档；csv 数据行回填。M2 五项修复清单见文件头当前状态。

