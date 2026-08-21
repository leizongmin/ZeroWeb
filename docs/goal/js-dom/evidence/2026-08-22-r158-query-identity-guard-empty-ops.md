# R158 Evidence — 查询 identity 缓存 + 非法选择器守卫 + 空值运算符（M4 nodes）

**日期**: 2026-08-22
**Commit**: `42143fd1a`（rebase 后；原 `ed78a66a0`）
**切片**: M4 — ParentNode-querySelector-All 激活（941P→1811P；全量 8534P→9400P 双路径一致）

## 根因与修复（四件）

### 1. Element identity 跨查询不保（最大簇 ~600F）

WPT `runFinderTest` 断言 `assert_equals(found, foundall[0])` —— 同一元素的
querySelector 与 querySelectorAll[0] 必须**同对象**。shim 三条查询面每次
`new _zwParseEl` 全新对象 → 全 fail。三处 per-root wrapper 缓存：

- **detached doc 工厂**（`_zwWrapCached`）：键 tag+\x1f+id+\x1f+outer；树变更经
  `_zwQWrapGen++`/`clear()`（bodyHtml 三写点 hook：innerHTML setter / handle
  append / chtml 更新）
- **Element.prototype 查询**（`_zwMWrapCached`）：键同款，Map 挂 root 对象
  （`_zwQWrapMap` 槽），上限 512 防爆
- **`_zwParseEl.prototype` + fragment**：同款（元素自身/fragment 对象槽）

### 2. 非法选择器 SyntaxError / 无参 TypeError（~120F）

`_zwQueryGuard` 全查询 API 边界（六入口：_zwParseEl / detached doc body+doc /
Element.prototype / fragment / part06 主 document / part04 proxy 的 sel+handle
双分支）：
- 非法选择器 → SyntaxError DOMException（`__zw_selector_valid` 探针，R156 建）
- 真无参 → TypeError（WebIDL 必参；`querySelector(undefined)` 是 1 参 undefined
  → 按浏览器语义查字面 "undefined" type selector 不抛）

### 3. `split_top_level_selector_list` 转义感知（engine r118 回归修复）

`#\.\,\:\!` 的 `\,` 是字面逗号——旧版在转义逗号处切列表 → 残段 `\:\!` parse
None → 整链误判非法（guard 新暴露的既有缺陷）。转义对 `\x` 整对跳过。

### 4. 空值运算符 + VT ident（host 语义两件）

- `[class^=""]` / `[class$=""]` / `[class*=""]` 空值**恒不匹配**（WPT expect 0；
  `starts_with("")`/`contains("")` 旧恒真）
- U+000B（VT）在 `.`/`#` ident-start 判定按 R124 语义算字面类名字符
  （`querySelector('.'+VT)` 返 null 不抛——engine r124 回归验证）

## A/B 双路径

全量 dom WPT（两次捕获运行 + native 各一次，逐计数一致）：

| 路径 | 结果 |
|---|---|
| polyfill ×2 | 9400P/464F/18T（双跑稳定） |
| native（ZW_NATIVE_DOM=1） | **9400P/464F/18T** |

vs R157（8534P/1329F）：**+866P/-865F**。R156 以来三轮累计：6290P→9400P
（**+3110P**）。

## 验证

- `cargo test -p zero-dom`：847 全绿（+3 回归：bare bracket 拒 / escaped id
  全形态 / VT ident）
- `cargo test -p zero-engine`：2306 全绿（r118 转义 + r124 VT 两件既有回归
  守住——guard 新暴露的 split 缺陷当轮修）
- `make test`：全绿；fmt/clippy 干净
- 方法论注记：全量 WPT 计数须经文件捕获（`> /tmp/run.txt`）读数——裸管道
  awk 计数曾出现 6381P 的截断误读，双跑文件对比证实 9400P 稳定

## ParentNode-querySelector-All 剩 163F（R159 候选）

分散单例簇：伪元素（`:before`/`::first-line` one/two-colon 期望 0 匹配）、
`:root`/`:empty`/`:not(*|*)`、ns 选择器三形态、`[*|TiTlE]` 第 3 命中缺
（setAttributeNS 的 title 在 mutable tree 不入 host 快照）、`no parameter`
TypeError 的剩余入口、NodeList instance / tree order 断言。
