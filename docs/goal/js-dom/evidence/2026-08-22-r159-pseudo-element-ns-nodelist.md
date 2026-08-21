# R159 Evidence — 伪元素 valid-no-match + ns type selector + NodeList instance（M4 nodes）

**日期**: 2026-08-22
**Commit**: `fc1611bd9`（rebase 后；原 `1bc3606f6`）
**切片**: M4 — ParentNode-querySelector-All 剩余分散簇（1811P→1907P；全量 9400P/464F→9503P/360F）

## 四件修复

### 1. 伪元素「合法但零匹配」（~64F 簇）

spec：DOM querySelector **不匹配伪元素**（伪元素不是元素）——WPT 期望
`#x:before` / `#x::before` / `::slotted(foo)` 全部**合法查询且返空**（旧版
词法拒 `::` → SyntaxError 抛出，整簇 fail）：

- `PseudoClass::PseudoElement` 新变体（`matches_full` 恒 false）
- parse_pseudo 收录 before/after/first-line/first-letter/slotted/selection/
  marker/placeholder/backdrop
- `::` 双冒号语法：name 解析前剥前导第二冒号
- `::slotted(foo`（未闭合括号）：词法层仅**伪参上下文**宽容（最后一个 `:`
  之后有 `(`；顶层裸 `(`/`)` 仍拒——WPT invalidSelectors），parse 层
  `find(')')` miss 时 args 取到串尾

### 2. ns type selector（~24F 簇，部分收）

- `*|div`（AnyNs）：任意 ns 的同名 localName 命中——`NsKind` 新字段 +
  tag 解析剥前缀，HTML 元素（HTMLNS）命中 ✓
- `|div`（EmptyNs）：仅 namespace 为空串的元素命中（HTML 解析产物是
  HTMLNS → 不命中 ✓ spec 语义）
- `ns|div`（有名前缀）仍拒（无 @namespace 声明表——WPT Undeclared ns）

### 3. html/body 保真（iframe 查询树）

detHtml 包装层恢复原始 `<html id="html" lang="en"><body id="body">` 属性
（doc 槽 `_r159HtmlAttrs`/`__r159BodyAttrs` 从 iframe markup 提取）——
`querySelectorAll('html')` / `:root` / `[id=html]` 命中且 id 正确（旧版
html/body 属性全丢）。

### 4. NodeList instance（4F）

querySelectorAll 产物数组打 `__zwQSA` 标记；`NodeList[Symbol.hasInstance]`
接受该标记（WPT "returns NodeList instance"；live childNodes 的
`__zwLiveNL` 语义不变）。

## 已知限制（R160 候选，深结构）

`#any-namespace *|div` 的 4 命中在 **Document 上下文** fail：probe 实证
`setupSpecialElements` 的 `parent.appendChild(doc.createElement(...))` 落在
**per-element mutable tree**（`_zwParseEl._ensureMutTree()`），与 doc 级查询树
（detached factory 的 `_tree`）**互不合并**——`doc.getElementById(
"any-namespace")` 都查不到（anyKids:-1）。这是 polyfill 树碎片化的结构性
问题（L2 三方合一的正解域），单点修补风险大，记 R160 评估。

## A/B 双路径

| 路径 | 全量 dom WPT |
|---|---|
| polyfill | **9503P/360F/18T** |
| native（ZW_NATIVE_DOM=1） | 9494P/368F/19T（差 1P/1T——边缘 Timeout 漂移，逐簇一致） |

vs R158（9400P/464F）：**+103P/-104F**。R156 以来四轮累计：6290P→9503P
（**+3213P**）。

## 验证

- `cargo test -p zero-dom`：848 全绿（+1 回归 zz_r159_pseudo_element_and_ns：
  伪元素合法零匹配 + `*|div`/`|div`/`ns|div` 三形态 + slotted 未闭合）
- `cargo test -p zero-engine`：2306 全绿；`make test` 全绿；fmt/clippy 干净
