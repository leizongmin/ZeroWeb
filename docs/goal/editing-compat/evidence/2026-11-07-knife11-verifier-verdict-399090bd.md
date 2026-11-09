# knife11 sep 刀体终判核验（核验方 399090bd，2026-11-07 13:35）

对 census 轮终判批 `batch-knife11-verdict.log`（12:51:39 起飞 head=98b1e5296，13:22:10 ALL-DONE）的独立核验结论。判读域字节绑定成立，红线全过，F2P 键级账 23/24 与预测集差异已归因。

## 1. 字节绑定（build-anchored）

- 批内锚 `ad212b5f8ed9422bb835190abb43487d`（四 shim 文件清单含 part01e）；本会话 13:23 现树复算 **同值**——判读域 = 现工作区字节，无漂移。
- 双面制四 marker 全中：paren 旧形 0/新形 3（part01 ①/②b + part01f）；no-paren 旧形 1（part06 qV 值面保留）/新形 1（part01e retag）。
- 批构建走缓存 0.18s 无重编（认领轮 12:45 预核第 1 节同值）。

## 2. 红线（judge-dps.py vs knife10c 基线）

| 项 | 值 | 判定 |
|---|---|---|
| rows | 10092 → 10092 | 恒一 ✓ |
| P2F | 0 | ✓ |
| post-only | Pass 0 / Fail 0 | ✓ |
| base-only(lost) | 0 | ✓ |
| F2P | 23 | 键级账见 §3 |
| raw 面 | P 19854→19900（+46），F 330→284（−46） | −46 = 23 键 × 2 页（multitest + multitest-ab 同数据文件，judge 去重后半）✓ |
| UNEXPECTED 桶 | 无行 | qS-after 与 styled 组均未动，无预测外位移 ✓ |

**KNIFE11-REDLINE OK**（批内 + 本会话独立复核一致）。

## 3. F2P 键级账（vs 认领轮 12:45 预核预测集）

预测集（landing-division §2）：恰 24 键 ip-compare F2P（foo[]bar 24 = rich 3-cmd 12 + B 簇 2-cmd 12）；styled 22 不出现；qS-after 翻绿则记 UNEXPECTED。

实际 F2P = 23，逐项对账：

- **23 键全部为 foo[]bar rich 链 compare innerHTML**：11 命令 × 2 形（2-cmd `[styled,ip]` + 3-cmd `[styled,ip,inserttext a]`）= 22，加 createlink 2-cmd × 1。
- **styled 22 不出现（仍 F）✓** ——与预测一致。
- **qS-after 未翻 ✓** ——无 UNEXPECTED 记账项。
- **唯一偏差：预测 24 中 `[["createlink","http://www.google.com/"],["insertparagraph",""],["inserttext","a"]]` 未翻**，基线 F / 现仍 F（本会话双 log 逐行核对）。归因：该键 typed 段落点为 **typed-fate 域**（createlink 壳内 typed "a" 落点），非 separator 块面——与认领轮探针定谳的三层病理（typed 文本落点 / caret 壳外落点 / faithful 相 typed 灭）同族，属认领轮 v2 刀处方 (b) caret 入壳下行的治理范围。预测集将此键计入 sep 可翻面偏乐观，账面修正为 **sep 可翻 23 + typed-fate 残 1 + styled 22 + qS 2**；不构成红线或归因异议。

## 4. 同窗待落地项（现工作区，未提交）

- 刀体五文件：part01.js/part01e.js/part01f.js/part06.js（sep 翻转，census 轮落地权）+ `js_dom_bridge_tests/part06.rs`（r3254 双断言块 div→p 期望预改，svg 核验方预警、认领轮落树——落地分工 landing-division §3：与刀体同提交或紧邻提交，避免 shim=p/期望=div 破头窗）。
- 本会话不改 shim 字节（认领轮冻结声明同守）；刀体提交前本笔记判读绑定现树字节，提交后字节不变则账目自动随附。

## 5. 核验方结论

**VERDICT: OK**——红线全过、字节绑定成立、键级账 23 + 1 偏差归因成立、无 UNEXPECTED。sep 刀体可落地；typed-fate 残账（createlink 3-cmd 1 键 + styled 22 + qS 2）移交认领轮 v2 刀治理。
