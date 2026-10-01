# 编码兼容 — 运行时控制面板（master.md）

**入口文档**: [../encoding-compat.md](../encoding-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-02（M3 落账）

## 当前状态

**M3 完成（2026-10-02）**：TextEncoder/BOM/fatal/ignore 模式 + 编码往返 + streams 语义——
全语料 36 案 **12070/12075 = 99.96%**（top 99.7% / streams 93.9% / legacy-mb 100%）。
残差 5 案已分类记账：4 个 detached-AB/SAB transfer 结构面（workers/transfer 域）+
1 个 encoding_rs iso-2022-jp fatal-stream 状态契约（上游错误契约）。
证据：[evidence/2026-10-02-m3-textencoder-modes-streams.md](evidence/2026-10-02-m3-textencoder-modes-streams.md)
（含 net-api 共享面逐域同值零漂移核对）。

M2（2026-10-02）：labels 全表 + legacy 解码（encoding_rs host native + 有状态 decoder 表）
11876/12075 = 98.4%——[evidence/2026-10-02-m2-labels-legacy-decode.md](evidence/2026-10-02-m2-labels-legacy-decode.md)。
M1（2026-10-01）：corpus 通道 + 基线 268/4403 = 6.1%（21 案，语料缺角后 FORCE=1 补齐）。

与 html-syntax-compat 划界不变：文档级编码嗅探运行面排除，挂账定稿在 M4。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | encoding/ corpus 导入 + 基线 | ✅ M1（2026-10-01） |
| P2 | labels 标签匹配全表（数据化） | ✅ M2（api-invalid-label 3421/3421） |
| P3 | TextDecoder legacy 编码解码 | ✅ M2（legacy-mb 100%、single-byte 336/336、iso-2022-jp 34/34） |
| P4 | TextEncoder + BOM/fatal/ignore 模式 + 编码往返 | ✅ M3（99.96%；残差 5 已记账——transfer 结构面 ×4 + encoding_rs 契约 ×1） |
| P5 | 文档级编码嗅探挂账定稿（html-syntax-compat 划界） | ⏳ M4 |

## 已完成切片

- **M1**（2026-10-01）：fetch 脚本定稿、runner `testharness-encoding` 子命令、Makefile
  双 target、基线 JSON+md、CSV 数据行。
- **M2**（2026-10-02）：`text_encoding.rs` host 三 native（encoding_rs）+ 有状态 decoder
  表（handle FIFO 封顶 4096）+ shim 构造门（RangeError 双脸）+ XHR replacement 面 +
  host 单测 + shim 集成测；语料补齐 36 案。
- **M3**（2026-10-02）：utf-8 spec 逐字节状态机（重处理语义）+ BOM 前缀嗅探机 +
  TextEncoder 默认参数/孤立代理/encodeInto spec 语义 + TextDecoderStream options 转发/
  chunk 门/错误传播 + TextEncoderStream 跨 chunk 驻留 + WritableStream.prototype.
  getWriter 委托 + XHR BOM 嗅探序 + encoding_rs 空块 guard（回归测试锁定）。

## 下一步计划

1. **M4 收口**：DC-1~4 全量判定（`make test` + clippy + fmt + `make reftest` 零回归）+
   残差分类定稿 + 文档级嗅探挂账定稿（html-syntax-compat 划界文书化）+ CSV completed 行

**待用户决策清单**：（空）
