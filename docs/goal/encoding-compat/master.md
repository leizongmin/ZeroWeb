# 编码兼容 — 运行时控制面板（master.md）

**入口文档**: [../encoding-compat.md](../encoding-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-02（M2 labels + legacy 解码落账）

## 当前状态

**M2 完成（2026-10-02）**：labels 标签全表 + TextDecoder legacy 解码（encoding_rs
host native + 有状态 decoder 表）——全语料 36 案 **11876/12075 = 98.4%**
（top 98.8% / legacy-mb any.js 100% / streams 51.8%）。labels 校验门、单字节 28 编码、
GBK/GB18030/iso-2022-jp/replacement 全绿。
证据：[evidence/2026-10-02-m2-labels-legacy-decode.md](evidence/2026-10-02-m2-labels-legacy-decode.md)
（含 net-api 共享面零回归核对：streams 905/1020 恒值、xhr +12 正漂移）。

M1（2026-10-01）：corpus 通道 + 基线 268/4403 = 6.1%（21 案，语料缺角——后经
FORCE=1 补齐至 36 案，见 M2 证据口径注记）。

与 html-syntax-compat 划界不变：文档级编码嗅探运行面排除，挂账定稿在 M4。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | encoding/ corpus 导入 + 基线 | ✅ M1（2026-10-01） |
| P2 | labels 标签匹配全表（数据化） | ✅ M2（2026-10-02——api-invalid-label 3421/3421） |
| P3 | TextDecoder legacy 编码解码（单字节 28/GBK/GB18030/Big5/Shift_JIS/EUC-x/ISO-2022-JP/replacement/UTF-16） | ✅ M2（2026-10-02——legacy-mb 100%、single-byte 336/336、iso-2022-jp 34/34） |
| P4 | TextEncoder + BOM/fatal/ignore 模式 + 编码往返 | ⏳ M3——残差簇：utf-8 JS 快路 spec 状态机（fatal 2/36、mistakes 78/87、eof/copy 0/2）、encodeInto 36/111、lone surrogate、TextDecoderStream options 转发/chunk 门/错误传播、EncoderStream 跨 chunk 缓冲 |
| P5 | 文档级编码嗅探挂账定稿（html-syntax-compat 划界） | ⏳ M4 |

## 已完成切片

- **M1**（2026-10-01）：fetch 脚本定稿、runner `testharness-encoding` 子命令、Makefile
  双 target、基线 JSON+md、CSV 数据行。
- **M2**（2026-10-02）：`text_encoding.rs` host 三 native（encoding_rs）+ 有状态
  decoder 表（handle FIFO 封顶 4096）+ shim 构造门（RangeError 双脸）+ XHR
  replacement 面 + host 单测 10 条 + shim 集成测 1 条；语料补齐 36 案。

## 下一步计划

1. **M3-S1**：utf-8 JS 快路按 spec #utf-8-decoder 状态机重写（BOM 前缀机跨 chunk +
   fatal 线程 + 逐字节 U+FFFD 语义）
2. **M3-S2**：TextEncoder 默认参数 ""、lone surrogate → U+FFFD、encodeInto
   read/written spec 拆分语义
3. **M3-S3**：TextDecoderStream options 转发（fatal/ignoreBOM IDL 反射）+ chunk 类型
   门 + fatal error 传播；TextEncoderStream 跨 chunk 多字节输出缓冲
4. **M4**：收口（DC-4 门禁 + 嗅探挂账定稿）

**待用户决策清单**：（空）
