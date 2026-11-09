---
date: 2026-11-08
modules: tests/wpt-runner, .acceptance/dps
---

# 门禁批计数跨域混算——multitest raw 行数(×2 双窗) ≠ judge 键域(去重)，完账检查与残数预期必须同域取数

## 问题描述

F2 link-absorption 刀 v2 门禁批（2026-11-08）两处假 ABORT 源于同一根因：

1. mt 完账检查 `grep -c '^Pass \|^Fail '` 得 **20184**，脚本期望写死
   `10092`（judge 报告的 rows 数）→ 完好的孤儿 runner 日志被误判不完整，
   触发无谓的 mt 整批补跑（~30 min 浪费）。
2. 残键普查预期 `EXP_RES=$((244 - NF2P))`，其中 NF2P 取自 judge 的
   **assertion 去重域**（38），而 244 与残键计数都是 census 的
   **entry 行域**（每键 ×2 比较模式行）→ 必然假 ABORT（应为 244−76=168）。

## 根因分析

multitest 语料每个 (page, assertion) 键在日志中输出**两行**——两个比较模式
（`compare innerHTML` 与 `checks for modifications to non-editable content`）。
三个工具计数域不同：

- `grep -c '^Pass '`：raw 行域（20184 = 2×10092）；
- `judge-dps.py`：assertion 去重键域（10092，F 计数按键）；
- `census/f2ga-calibrate.py`：entry 行域（每键 2 行，244/78/76 均为行域）。

脚本作者把 judge 报告的 rows 当成 raw 行数用，又把 judge 的键域 F2P 与
census 行域基数互减——**两域不可互减**。

## 解决方案

- 门禁脚本内每个计数注明计数域（`raw 行域` / `judge 键域` / `census entry 域`），
  期望值一律用**与检查时同一工具**预先实测基线得出（如完账检查先对基线日志
  `grep -c` 取真值 20184），不抄另一个工具的输出。
- 跨域换算只有唯一合法形态：`census 行域数 = 244 − calibrate 行域 F2P`
  （calibrate 与 census 同域）；judge 域数字只用于 judge 域断言
  （`rows X->X`、`F2P N`、`P2F 0`）。
- 判读工具输出带域标注（judge 输出 `rows`、calibrate 输出 `rows <raw>-><raw>`），
  写门时以工具自身输出的域为准绳。
