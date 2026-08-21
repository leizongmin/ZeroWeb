# R156 Evidence — selector tokenizer 符号吞段修复 + Element-matches 激活（M4 nodes）

**日期**: 2026-08-22
**Commit**: `2ba3ccdf1`（rebase 后；原 `f56123b06`）
**切片**: M4 — Element-matches.html 整页 error 收口（R153–R155 连续三轮未动的 0(a) 优先项）

## 根因（host tokenizer，影响面超出 js-dom）

`zero-dom/src/query.rs` 的 `tokenize_combinators`：pending-explicit 符号边界以
「下一段首字节 idx」记录，切分左段 `&s[seg_start..b_idx]` 因此**包含符号本体**——
`#a1+div` 切出 `("#a1+", NextSibling)`，id 标识吞 `+` → 全形态 miss。

实证（Rust 单测诊断）：
- `#a1+div`（无空格 adjacent）→ 0 命中；`#a1 +div`（符号前空白 upgrade 路径）→ 1 ✓
- `#a1~div` / `div~div`（tilde 全形态）→ 0 命中（tilde 从无 upgrade 路径，全灭）
- `#adjacent>div`（无空格 child）→ 0 命中

**影响面**：所有 `querySelector(All)` 消费者（js-dom shim、CSS 匹配走独立
zero_css_parser 路径未受影响——`Combinator` 枚举两处定义）。此前文档级 WPT
用例未覆盖「无空格组合器 + tilde」形态，故长期潜伏。

修复：符号起点 `seg_char_run_start`（向前跳空白）记录 + 切分 monotonic clamp
（`a > > b` 病态形态旧版 slice `[6..3]` panic——test-guard 实证 abort）。

## Element-matches 激活链（7P → 598P/77F）

1. **WIP 续接**（上轮 429 中断遗留）：`_zwParseEl.matches`、deepClone 查询面、
   setAttributeNS 族、fragment querySelector、iframe `.html` kind 正则。
2. **文档上下文匹配**：查询根（detached doc queryOne/queryAll、fragment、
   deepClone）把根源串挂产物 `_zwRootHtml`——sibling/descendant 组合器在整树
   上下文解析（元素自身 outerHTML 内 `#a+#b` 永不命中）。
3. **deepClone 经 `_zwMEl` 工厂重建**：原型链 + setAttribute/mutation 面
   （traverse(clone) 的 `elem.setAttribute` 旧 plain 对象 TypeError）；
   cloneNode(deep) 语义（缺省 false 浅克隆）；ownerDocument 继承
   （`root.ownerDocument.defaultView.DOMException` 断言）。
4. **Element.prototype 查询面**（matches/matchesSelector/webkitMatchesSelector/
   querySelector(All)）：`assert_idl_attribute` 要求**原型链**非 own property；
   模块级 `_zwMOuterHtml/_zwMQueryAll/_zwDeepCloneEl`（闭包内定义 → 原型方法
   ReferenceError，已提出）。
5. **SyntaxError DOMException**：新 host 探针 `__zw_selector_valid`
   （`selector_is_valid` = 词法预检 + 结构 parse 双层）覆盖 WPT
   invalidSelectors 33 形态（裸括号/ns| 前缀/`::`/class ident-start/attr
   名值良构/病态组合器/空段逗号列表/顶层组合器开头）。
6. **无参 TypeError** + Node 常量挂解析元素原型（interfaceCheckMatches 的
   `obj.nodeType === obj.ELEMENT_NODE` 分支）。
7. **R134 保序**：handle-only（detached createElementNS）的 `urn:ns|h` 匹配
   前移到语法校验之前（`ns|type` 对 detached 元素合法——R134 单测
   `test_matches_ns_and_unscopables_r134` 验证不回归）。

## A/B 双路径

| 路径 | Element-matches 子集 | 全量 dom WPT |
|---|---|---|
| polyfill | 598P/77F（原整页 error 7P） | **8371P/1491F/18T** |
| native（ZW_NATIVE_DOM=1） | 整页 error（iframe doc body 空——native 独立缺口，见 R157 候选） | 6289P/264F/19T（= R155 基线，零回归） |

全量 +2081P 主要来自两个此前整页 error 的文件解锁执行：
- `ParentNode-querySelector-All.html`：897P/**1076F**（文档级 querySelectorAll
  深比较——attr-escape/伪类缺口与新解锁的组合器面）
- `Element-webkitMatchesSelector.html`：592P/77F（与 Element-matches 同簇）

## 剩余 77F 聚类（R157 候选）

- Attribute operator 边缘（~30）：`~=` 空白分隔 / `|=` 连字符 / `*=` unquoted /
  `\e9` 转义值（`[data-attr-value="\e9"]`）——zero-dom attr 匹配引擎
- 伪类 in matches（~10）：`:lang` 继承 / `:empty` / `:root` / `:target` / `:not` 组合
- Universal/no-id 定位（~4）：matches 的 identity 判定在无 id 元素上的近似

## 验证

- `cargo test -p zero-dom`：841 全绿（含 zz_r156 三件：no-space 组合器 /
  invalid forms 拒绝 / fuzz no-panic）
- `cargo test -p zero-engine`：2304 全绿（R134 ns|type 保序验证）
- `make test`：全绿（SW `repeated_registration` 时序 flaky 一轮，单跑稳定绿，
  归因 service-workers 流——见 run-rules §10）
- `cargo fmt --check` / `cargo clippy --workspace --all-targets -D warnings`：干净
- 诊断方法学沿用 R155：zz-*.html 内联 WPT 用例 + assert_equals 'MARKER'
  强制失败输出诊断串
