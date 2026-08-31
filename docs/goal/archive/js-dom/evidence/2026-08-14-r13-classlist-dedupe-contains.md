# R13 — classList 去重 + contains 空白不抛（M4 / DC-3）

**日期**: 2026-08-14
**轮次**: R13
**里程碑**: M4（WPT dom 上游基线按聚类驱动修复）
**commit**: 见 `git log`（feat(js-dom): classList token dedupe + contains whitespace no-throw）

## 背景

R12 后 Element-classlist.html 是第二大失败块（280 失败）。聚类：classList.length 去重（215 assert_equals）+ contains 空白边界（~60）。

## 改动

### 1. DOMTokenList token 有序去重（part03 `_classListProxy` cur）

spec `dom-domtokenlist`：token 集合为**有序去重**（首个出现位置保留，后续重复丢弃）+ ASCII 空白分隔。`"a a a"` → `["a"]`（length 1）；`"\t\n\f\r a\t\n\f\r b\t\n\f\r "` → `["a","b"]`。cur() 原 `split(/\s+/).filter(Boolean)` 无去重，加 `Object.create(null)` seen 表去重。

### 2. contains() 空串/含 ASCII 空白 → false（不抛）（part03）

spec `dom-domtokenlist-contains`：空串或含 ASCII 空白 token → 返 false（**不抛**，区别于 add/remove/toggle/replace 的 check 抛 SyntaxError/InvalidCharacterError）。原 contains 调用 check（抛），与 spec 不符。WPT checkContains(null,["a","","  "],false) + checkContains("a",["a\t",...],false)。

### 3. 单测

`test_classlist_dedupe_and_contains_whitespace_r13`：去重 length/item、contains 空白不抛（空串/双空格/`a\t`）、add no-op、前后空白规范化。

## 基线结果（dom/nodes，178 用例 / 4502 subtest）

| 路径 | R12 | R13 | Δ |
|------|----|----|---|
| polyfill | 41.71% | **46.60%** | +4.89pp |
| native | 41.45% | **46.33%** | +4.88pp |

双路径对等差 0.27pp。**classlist 用例**：1140P/280F → **1360P/60F**（+220 pass，迄今最大单切片提升）。完整 JSON 快照入 evidence。

## 验证

engine v8 2084 / quickjs 1408 单测；fmt + clippy（v8 + quickjs）零警告。

## 下一步

- classlist 剩 60F（add/toggle/replace 边缘 + toString/value + forEach 迭代等，需细查）。
- createEvent（183）/ createDocumentType（81）/ createElementNS 大小写（case.js prefix）。
- iframe.contentDocument（深结构 html-compat 域，待评估）。
