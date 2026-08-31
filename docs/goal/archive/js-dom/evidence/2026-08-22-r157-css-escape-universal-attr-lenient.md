# R157 Evidence — CSS 转义反解 + universal `*` + Selectors-API 宽容 attr 形态（M4 nodes）

**日期**: 2026-08-22
**Commit**: `53efe41b0`（rebase 后；原 `23b839c68`）
**切片**: M4 — R156 后 Element-matches 剩 77F 的 attr/引擎簇收口 + R157(a) native 虚警排除

## R157(a)：native iframe contentDocument「缺口」= 陈旧二进制虚警

R156 记录的「native 路径 Element-matches 整页 error（iframe doc body 空）」经本轮
探针复测为**陈旧 release 二进制**所致——`make testharness-dom` 会重建，但直接
`./target/test-guard -- ./target/release/zero-wpt-runner` 用的是上次构建的旧
binary。重建后：

| 路径 | Element-matches | 全量 dom WPT |
|---|---|---|
| polyfill | 657P/18F | **8534P/1329F/18T** |
| native（ZW_NATIVE_DOM=1） | 657P/18F | **8534P/1329F/18T** |

**双路径完全对等（逐计数一致）**——R156 的 tokenizer + 激活链在 native 同样
生效；不存在 native iframe 缺口。教训已记：native 路径 WPT 验证前必须先经
`make` 入口重建（或确认 binary 新于源码）。

## R157(b)：host 引擎三件修复

### 1. CSS 规范转义反解（`unescape_css_string`）

CSS Syntax §4.3.2「consume an escaped code point」：`\` + 1–6 位十六进制 +
可选单个空白终结 = 码点（`\e9`→é、`\0000e9 `→é）；`\` + 非十六进制 = 字面
字符。应用于属性值/属性名；`unescape_css_ident`（id/class）升级同源——wire 侧
`escape_css_ident` 只转义非 `[a-zA-Z0-9_-]` 字符（均非 ASCII 十六进制数字），
hex 升级向后兼容（R3254 wire 对不破）。

WPT 簇：`[data-attr-value="\e9"]` / `[data-attr-value_foo="\e9"]` escaped value。

### 2. Universal selector `*`

`SimpleSelector::matches` 的 tag 比对旧版把 `"*"` 当字面 tag——`eq_ignore_ascii_case("*")`
恒 false → `querySelectorAll("*")` 返空、`matches("*')` 恒 false。修 = `tag != "*"`
跳过比对（spec selectors-4 §6.2）。WPT Universal selector 12F 簇根源（单一 if）。

### 3. Selectors-API 宽容 attr 形态（词法 + parse 双层）

- **截断属性段自动补全**：`#a [align="center"`（无 `]`）——WPT validSelectors
  expect 命中（浏览器自动补 `]`）。parse 端 `find(']')` miss 落 `r.len()`；
  词法端尾部未闭合 `[` 放行（多余 `]` / `(` 不配对仍拒）。既有单测
  `test_parse_unclosed_bracket` 按 WPT 语义更新（None → Some，严格 CSS 判定
  留在词法层）。
- **substring 族运算符空白宽容**：`[class*= banana ]`（unquoted 值两端空白）
  WPT expect 命中；Exact `=` 保持严格（`[class= space unquoted ]` 在
  invalidSelectors）。
- **`*|` any-ns 属性名前缀剥离**：`[*|TiTlE]` → name=TiTlE（本引擎属性无 ns 域）；
  `ns|` 前缀词法层仍拒（Undeclared namespace 簇）。
- **词法校验 bracket/escape 感知**：`|`（`|=` 运算符）/`.`/`#`（unquoted 值字符）
  在 `[...]` 内不参与 ns/class 结构判定；转义对（`\[`, `\]`, `\.`）整对跳过
  （`.test\.foo\[5\]bar` 的 `\[` 不是 attr 括号）。

## A/B 双路径

全量 dom WPT：polyfill 与 native **逐计数一致**（8534P/1329F/18T）；Element-matches
双路径 657P/18F 一致。R156+R157 两轮累计：全量 6290P→8534P（**+2244P**）。

## Element-matches 剩 18F（R158 候选）

- `:empty` 8F（引擎伪类——注释子节点/空白文本判定）
- `:lang` 2F（继承链）、`:root`/`:target` 2F、`:nth-child` 2F（matches 路径位置上下文）
- `[TiTlE]` 变体 3F（matches 路径 case-insensitive——文档级已修，matches 面待查）
- Type selector html 1F

## 验证

- `cargo test -p zero-dom`：845 全绿（新增 4 件回归：escaped values + 空白终结符 /
  universal star / unclosed auto-close + star-pipe + case-insensitive / 词法误杀表）
- `cargo test -p zero-engine`：2304 全绿
- `make test`：全绿；`cargo fmt --check` / clippy `-D warnings`：干净
- 全量 dom WPT 双路径：8534P/1329F/18T 完全对等
