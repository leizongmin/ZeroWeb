# M4 R13 切片 — classList 去重 + contains 空白不抛

**日期**: 2026-08-14
**里程碑**: M4（WPT dom 上游基线按聚类驱动修复）
**前置**: R12（element.prefix getter + getElementsByTagNameNS）
**commit**: 见 `git log`（feat(js-dom): classList token dedupe + contains whitespace no-throw）

## 背景

Element-classlist.html 第二大失败块（280 失败）。聚类：classList.length 去重（215 assert_equals）+ contains 空白边界（~60）。

## 改动（2 处，part03 `_classListProxy`）

### 1. DOMTokenList token 有序去重

cur() 加 `Object.create(null)` seen 表去重。spec：有序去重（首位置保留）+ ASCII 空白分隔。`"a a a"` → `["a"]`。

### 2. contains() 空串/含 ASCII 空白 → false（不抛）

spec `dom-domtokenlist-contains`：空/含空白 token → false（区别于 add/remove/toggle/replace 抛）。原 contains 调 check 抛，改直接判返 false。

### 3. 单测（part07）

`test_classlist_dedupe_and_contains_whitespace_r13`。

## 基线（dom/nodes，178 用例 / 4502 subtest）

| 路径 | R12 | R13 | Δ |
|------|----|----|---|
| polyfill | 41.71% | 46.60% | +4.89pp |
| native | 41.45% | 46.33% | +4.88pp |

classlist 用例 1140P/280F → 1360P/60F（+220）。双路径对等差 0.27pp。迄今最大单切片提升。

## 验证

engine v8 2084 / quickjs 1408 单测；fmt + clippy（v8 + quickjs）零警告。

## 下一步

- classlist 剩 60F（add/toggle/replace 边缘 + toString/value + forEach）。
- createEvent（183）/ createDocumentType（81）/ createElementNS 大小写。
