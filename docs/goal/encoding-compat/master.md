# 编码兼容 — 运行时控制面板（master.md）

**入口文档**: [../encoding-compat.md](../encoding-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-01（M1 基线落账）

## 当前状态

**M1 完成（2026-10-01）**：encoding corpus 212 文件拉取 + runner 通道
（`make testharness-encoding`）+ 基线落账——21 案 window 可执行子集
**268/4403 = 6.1% Pass**（top 5.4% / streams 41.2% / legacy-mb any.js 2.0%），
[wpt-suites.csv](../../compat/trends/wpt-suites.csv) 已转数据行。
证据：[evidence/2026-10-01-m1-baseline.md](evidence/2026-10-01-m1-baseline.md)。

与 html-syntax-compat 划界不变：文档级编码嗅探（bom-handling/eof-*/utf-32*/sniffing/
big5·iso-2022-jp-encoder 等 `<meta charset>`/BOM document 面）运行面排除，挂账定稿在 M4。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | encoding/ corpus 导入 + 基线 | ✅ M1（2026-10-01） |
| P2 | labels 标签匹配全表（数据化） | ⏳ M2——基线簇：constructor 不校验（api-invalid-label 0/3421、replacement 不拒绝、`.encoding` 恒 utf-8） |
| P3 | TextDecoder legacy 编码解码（单字节 28 编码/GBK/GB18030/Big5/Shift_JIS/EUC-x/ISO-2022-JP/replacement/UTF-16） | ⏳ M2——基线簇：单字节 0/168、gbk 0/82、gb18030 7/275、iso-2022-jp 1/34、replacement 直通 |
| P4 | TextEncoder + BOM/fatal/ignore 模式 + 编码往返 | ⏳ M3——基线簇：encodeInto 36/111、lone surrogate 直编、ignoreBOM 0/12、fatal 不 error |
| P5 | 文档级编码嗅探挂账定稿（html-syntax-compat 划界） | ⏳ M4 |

## 已完成切片

- **M1**（2026-10-01）：fetch 脚本定稿（显式 DIRS + GOAL_PULL_ANY_JS + sab.js 补拉）、
  runner `testharness-encoding` 子命令（`run_any_js_corpus_subdirs` 通用化 + skip 双保险）、
  Makefile 双 target、基线 JSON+md 落 evidence/、CSV 数据行。

## 下一步计划

1. **M2-S1**：labels 全表数据化（encoding 标准 tables 落数据资产）+ TextDecoder
   label 校验门（未知 label TypeError / replacement 拒绝 / `.encoding` 恒等）
2. **M2-S2**：单字节 28 编码 index 表 + 解码器（single-byte-decoder TextDecoder 半脸翻转）
3. **M2-S3**：CJK 双字节（gbk/gb18030/big5/shift_jis/euc-jp/euc-kr）+ iso-2022-jp
   状态机 + replacement 三编码 + UTF-16 legacy
4. **M3**：BOM/fatal/ignore 模式 + TextEncoder/encodeInto 语义 + streams 属性面
5. **M4**：收口（DC-4 门禁 + 嗅探挂账定稿）

**待用户决策清单**：（空）
