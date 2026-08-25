# R243 Evidence — cDP 重评转正（R235 负结果的翻转面已消失，净 +26）

**日期**: 2026-08-25
**切片**: M4——R243(a) cDP 40 重评（R235 负结果复测）
**改动面**: `part03.js`（detached-doc 内部 docEl/headEl/body 附 contains/cDP own-prop）+ `part23.rs`（r243 单测）
**commit**: `75c87456c`

## 一、重评结果：负结果翻正

R235 首测（当时）：fixed 6 / new 34（净 **-28**）回退。R243 复测
（R236–R242 十一个 surround/extract 全序分支落地后）：
**fixed 34 / new 8（净 +26）**——当时 position 断言翻转面随 host
行为对齐已消失。方法论印证：**负结果会过期**——host 行为面大改
后旧实验必须复测。

## 二、修复

`_makeDetachedDocument` 内部 docEl/headEl/body 附加
contains/cDP own-property（委托 `_zwNodeContains`/
`_zwCompareDocumentPosition`，与 _zwMEl 同源 R79）——sim
（isAncestorContainer/getPosition）深入 foreignDoc/iframe doc
合成树不再 TypeError。

## 三、验证链（vs R242 基线）

| 项 | R242 | R243 | Δ |
|---|---|---|---|
| Range-surroundContents | 1768P/72F | **1794P/46F** | **+26 净**（fixed 34/new 8——17,x/30,x cDP 簇主解，8 个 17,x position subtests 残余 R244） |
| Range-extractContents | 132P | **140P** | +8 |
| Range-cloneContents | 155P | **162P** | +7 |
| Range-insertNode | 1841P/0F | 1841P/0F | 0（100% 保持） |
| Range-deleteContents | 67P | 67P | 0 |
| nodes 失败集 | 57 | 57 | **逐条一致**（零扰动，set-diff exit 0） |
| **ranges 全量**（除 probe） | 40080 行 | 40080 行 | set-diff **118 Fail→Pass / 8 Pass→Fail**（净 +110——extract/clone 连带解锁） |

- **native 同值**：ZW_NATIVE_DOM=1 surround 1768→1794P（+26 一致）。
- **engine 单测**：**2389 全绿**（新增 r243 单测）。
- fmt/clippy 干净。

## 四、R244 靶点（46F 重聚类）

| 簇 | 计数 | 行 | 备注 |
|---|---|---|---|
| HRE | 14 | 25/26,x 各 6 + 18/19,x 各 1 | 25/26,x `[document,0,…]` Document 容器 |
| differing | 13 | 17,x 9 + 13/14,x 4 | 17,x position 残余（cDP 解锁后的新前沿——expected "[object Object]" 形态） |
| startOffset | 11 | 16,x | harness-iframe index 算术 |
| other | 6 | 17,x 3 + 30,x 2 + 28,x 1 | 残余 |
| assert_unreached | 2 | 18/19,x | 残余 |

- **首选**：25/26,x Document 容器 HRE 12F（`[document,0,document,1/2]`
  ——surround 对 doc 级容器的 insertNode 位序校验）。
- 次选：17,x position 9F（cDP 解锁后暴露的 host/sim 深分歧）。
