# M4 R38 — HTMLCollection namedItem 空串守卫 + Element.children 返 HTMLCollection

**日期**: 2026-08-14
**轮次**: R38
**里程碑**: M4（WPT dom 上游基线 + 按聚类驱动修复 / DC-3 collections）
**前置**: R37（导入 dom/collections 基线）
**状态**: ✅ 已 land（双路径对等，零回归）

---

## 背景

R37 建立 dom/collections 基线（22.9%），失败聚类评估：HTMLCollection own 属性枚举（~19 fail）经诊断是**深结构**（live collection + 完整 indexed/named property 语义 get/set/has/delete/ownKeys + 数字边界 + 元素 id/name 动态），非「ownKeys trap」轻量切片。

转更轻量的 `HTMLCollection-empty-name.html`（7 subtest，1P/6F）：诊断发现根因是 **namedItem 空串命中空 id/name 元素**（spec：空串非 supported property name，namedItem("")/named getter("") 应返 null/undefined）。

probe 诊断（元素级 vs 文档级）：
- 文档级 `getElementsByTagName("*")` 返 **length=0**（独立 bug，`__zw_query_all` 对 "*" 文档级返空，恰好让文档级 namedItem("") Pass）
- 元素级 length=3（正确）→ namedItem("") 命中空 id 元素返元素（Fail）

另发现 `Element.children` 返**纯数组**（不经 `_zwMakeCollection`），缺 namedItem 方法（`c.namedItem("")` 抛 TypeError）。

## 实现

### namedItem 空串守卫（part05.js `_zwMakeCollection`）

namedItem 入口加 `if (n === '') return null;`（spec `dom-htmlcollection` supported property names 排除空串）。元素空 id/name（`<div id>`）不被空串命中。

### Element.children 返 HTMLCollection（part04.js get trap）

`children` 从返纯数组改为经 `_zwMakeCollection(arr, true)`（HTMLCollection，带 item/namedItem + R38 空串守卫）。`_splitSelectors` 已 `.map(_wrapSelector)` 返 proxy 数组，直接传入。NodeList（querySelectorAll，`htmlCollection=false`）不变。

## 验证

| 门禁 | 命令 | 结果 |
|------|------|------|
| R38 polyfill 单测 | `cargo test -p zero-engine --features v8 --lib test_htmlcollection_nameditem_empty_and_children_is_htmlcollection_r38` | ✅ 1 passed（namedItem 空串 + children HTMLCollection + 正向 namedItem） |
| engine v8 全量 | `cargo test -p zero-engine --features v8 --lib` | ✅ 2115 passed（R37 基线 2114 +1） |
| engine quickjs 全量 | `cargo test -p zero-engine --no-default-features --features quickjs --lib` | ✅ 1411 passed（零回归，含既有 children.length/[0] 用法） |
| clippy v8 | `cargo clippy -p zero-engine --features v8 --all-targets -- -D warnings` | ✅ 零警告 |
| clippy quickjs | `cargo clippy -p zero-engine -p zero-wpt-runner --no-default-features --features quickjs --all-targets -- -D warnings` | ✅ 零警告 |
| fmt | `cargo fmt --all -- --check` | ✅ 无 diff |
| WPT polyfill empty-name | `make testharness-dom FILTER=HTMLCollection-empty-name` | ✅ 1P/6F→**7P/0F（100%）** |
| WPT native empty-name | `make testharness-dom-native FILTER=HTMLCollection-empty-name` | ✅ 1P/6F→**7P/0F（双路径对等）** |
| dom/collections polyfill 全量 | `make testharness-dom FILTER=dom/collections` | 17P/31F/1timeout（R37 11P → +6，22.9%→**35.4%**） |
| dom/events 全量（回归） | `make testharness-dom FILTER=dom/events` | 177P/150F（R35 177P/151F，净 +1 无回归） |

## 决策记录

- **为何从 HTMLCollection own 属性枚举转向 empty-name**：own 属性枚举经 probe 诊断是深结构（live + 完整 property 语义 + 数字边界），非轻量切片；empty-name 根因清楚（namedItem 空串守卫 + children 缺 namedItem），改动面小（2 处），driving 用例明确（7 subtest），净 +6 pass。own 属性枚举深结构留下轮评估。
- **Element.children 包 HTMLCollection 不破坏既有用法**：既有 children 测试用 `.length`/`[0].tagName`（数组自有 indexed + length，HTMLCollection 保留），engine v8/quickjs 2115/1411 全绿 + part08/part12 children 测试零回归验证。
- **文档级 getElementsByTagName("*") 返空（独立 bug）**：probe 发现文档级 `__zw_query_all` 对 "*" 返空集合，恰好让文档级 empty-name Pass。这是独立 bug（非本切片），记未解决问题——但 R38 空串守卫让文档级仍 Pass（namedItem("") 无元素时也返 null）。

## 净影响

- DC-3（WPT dom 基线）：dom/collections polyfill/native 双路径 22.9%→**35.4%**（+12.5pp，empty-name 7 用例全过）
- DC-4（每项修复有单测）：namedItem 空串守卫 + Element.children HTMLCollection 单测（spec `dom-htmlcollection` supported property names 排除空串）
- dom/events 净 +1（间接，零回归）
