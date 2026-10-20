# Compression 兼容 — CompressionStream / DecompressionStream

**版本**: v1.3
**日期**: 2026-10-15（v1.0 立项 2026-10-08）
**状态**: ✅ Completed——终态 245/318 = 77.04%（非 brotli 面 245/245 全绿）；73 Fail 全为 brotli mission-defer 挂账，重开条件 = brotli crate 依赖评审立项
**执行模式**: WPT 驱动（上游 compression corpus 为验收标尺）+ 实现
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——数据面）

> **▶ 当前状态（2026-10-15 M3 收口）**：M2 五项修复（构造 TypeError、严格
> BufferSource、流式 update/finish 状态机、手工帧校验 + 错误 TypeError 化、
> 空流 done）后组合态复测逐字节同账 245/318 = 77.04%，DC-2 两连续轮次零下行
> （evidence/[2026-10-15-m3-close.md](compression-compat/evidence/2026-10-15-m3-close.md)）。
> M1 三案 Timeout 清零，子测试分母 309 → 318。零新增依赖（CRC32 =
> `flate2::Crc`，adler32 内联）。组合态 engine lib 3059 绿。
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
- **DC-2** ✅ gzip/deflate-raw 全格式对齐（roundtrip/分块/错误语义），分级通过率
  不再下行两个连续轮次（M2 77.04% → M3 复测 77.04%，非 brotli 面 245/245）
- **DC-3** ✅ 回归测试账本纪律；make test 全绿（M2 轮 71 套件）、make reftest
  零回归（710/710）、product-smoke 恒值（6 面 PASS）；零新增依赖（flate2 复用，
  CRC32 = `flate2::Crc` + adler32 内联，evidence/ 记录）

## 里程碑

M1 导入与基线（✅ 2026-10-15）→ M2 实现与语义对齐（✅ 2026-10-15）→
M3 收口归档（✅ 2026-10-15）。

## 依赖约束（run-rules §9）

与 net-api（已归档）streams 底座为消费关系；Cargo 依赖变更走 workspace 统一
评审（perf-gate config_hash 敏感）。M1 盘点：flate2 为 engine 既有直接依赖
（WOFF zlib 解码先例），M2/M3 未新增依赖。

## 轮次账

- **2026-10-15 M1**：runner 通道（`testharness-compression` 子命令 + skip 规则
  + Makefile target + fetch 脚本补 pako/media 依赖）；基线 53.40%，六簇归因，
  evidence 落档；csv 数据行回填。
- **2026-10-15 M2**：host 流式状态机重写（`js_dom_bridge/compress.rs`——handle
  表 + `Decompress::new(false)` 裸 inflate + 手工帧校验 zlib CMF/FLG/FDICT/
  adler32、gzip 头/CRC32/ISIZE/多成员串联、截断/多余输入检测）+ shim 流式化
  （`part02.js`——enum TypeError、严格 BufferSource、update/finish 逐 chunk 回
  传）+ 六回调（`callbacks.rs`）；77.04%（245/318，非 brotli 全绿，Timeout
  清零）；engine lib 3049 绿 + compress 单测 10 项；csv 数据行回填。brotli 73
  挂账不变。
- **2026-10-15 M3**：组合态 main（含并行流 d141bc389）复测——逐字节同账
  245/318 = 77.04%，非 brotli 面 245/245 稳定，DC-2 达成；组合态 engine lib
  3059 绿。goal 关闭；重开条件 = brotli 立项（evidence/
  [2026-10-15-m3-close.md](compression-compat/evidence/2026-10-15-m3-close.md)）。
