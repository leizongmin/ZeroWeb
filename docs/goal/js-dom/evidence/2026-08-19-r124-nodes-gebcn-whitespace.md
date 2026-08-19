# R124 — M4 nodes：getElementsByClassName-whitespace ASCII 分词（19F→0F 全 100%，+19 净）

**日期**: 2026-08-19
**里程碑**: M4（WPT dom 上游基线建立与扩展）
**驱动用例**: `dom/nodes/getElementsByClassName-whitespace-class-names.html`（7P/19F→26P/0F）
**规范**: https://html.spec.whatwg.org/multipage/infrastructure.html#ascii-whitespace
（HTML「ASCII whitespace」= space / `\t` / `\n` / `\f` / `\r` **五字符**——U+00A0 /
U+2000-200A / U+3000 / U+2028/2029 等 Unicode 空白是 class token 的**字面字符**非分隔符）

## 结果摘要

| 路径 | 前（R123） | 后 | 净 |
|------|----|----|----|
| polyfill nodes 全量 | 7806P | 7825P | +19（whitespace 19F→P，零新增 fail） |
| driving 簇（whitespace） | 7P/19F | 26P/0F（100%） | +19 |
| native nodes 全量 | 6107P/1025F | 6107P/1024F* | whitespace 簇同步全绿（*计数含 1 条漂移，见下） |
| traversal / collections / events | 1595P/9F / 48P / 419P/27F | 同值 | 零回归 |

\* native 全量两轮实测 1025F 与 1024F 各一次（文件级 flake 池既存成员——
create-element-realm-after-adoption 等，R118 已记）；driving 簇在两路径均 26P/0F。

## 根因与修复（三面）

**根因**：Rust `str::split_whitespace` / `str::trim` 与 JS `/\s/`、`/\s+/` 都是
**Unicode 空白集**（U+00A0 / U+2000-200A / U+3000 / U+2028/2029 等），而 spec 的
class 属性分词域是 **ASCII whitespace 五字符**。`<span class="&#x00A0;">` 的 class
是合法单字符类名，`gEBCN(' ')` 须命中——三方（Rust dom crate / host 桥回调 /
JS shim）各自把 Unicode 空白当分隔符，把该形态误切成空 token。

1. **Rust dom crate（`crates/dom`）**：
   - `node.rs` 新 `split_ascii_whitespace`（5 字符分隔）替换 `ElementData` 三处
     `split_whitespace`（new / set_attribute / parser TreeSink 的 class_list 构建）。
   - `query.rs` 新 `pub fn trim_ascii_ws`（5 字符 trim）替换 `parse_selector_chain` /
     `parse_simple_selector` / `document/mod.rs` 四查询入口的 `str::trim`——
     `.\u{000B}` 这类「单个 Unicode 空白字符类名」选择器的类名字符不再被剥。
     `~=` 属性选择器的值分词同源切换（`split_ascii_whitespace`）。
   - 单测 `test_ascii_whitespace_class_tokenization_r124`（4 断言组）。
2. **host 桥（`crates/engine`）**：`js_dom_bridge.rs` 四处 + `selector_match.rs` 四处
   `selector.trim()` → `zero_dom::trim_ascii_ws`（`__zw_matches` / `__zw_closest` /
   子树 query 两族——matches/closest 的 ASCII trim 直接影响 driving 簇的
     `matches('.' + VT)` 断言形态）。
3. **JS shim（part03/04/05）**：新 `_zwSplitClassList`（`/[ \t\n\f\r]+/` 分词 +
   空串滤除，part03 定义、全 shim 共享作用域）接入四消费点——classList `cur()`
   分词 / `classList.contains` 的 token 含空白判定（`/\s/` → `/[ \t\n\f\r]/`，
   U+00A0 token 不再被误拒）/ `getElementsByClassName` 参数分词（part04）/
   part05 handle 元素 `_hClassesOf` + `~=` 求值。

## 验证

- driving 簇双路径 26P/0F（`make testharness-dom FILTER=getElementsByClassName-whitespace`
  + `make testharness-dom-native FILTER=...`）。
- 回归面：dom/nodes 7825P（+19，fail 文件集与 R123 逐文件比对零新增）；
  Element-classlist 1420P 全绿（classList 分词改造零回归）；traversal/collections/events
  与 R123 同值；Element-matches-namespaced 6F 与 ParentNode-querySelector-escapes 4F
  均 clean-HEAD 复跑同败（既存缺口，非本切片回归）。
- `make test` 全绿 exit 0（v8 + quickjs 双矩阵）；`cargo fmt --check` 无 diff；
  clippy 双矩阵零警告。
- 单测：dom `test_ascii_whitespace_class_tokenization_r124` + engine
  `test_ascii_whitespace_class_domain_r124`（9 断言段：gEBCN/classList len·item·contains
  三形态/`~=` 字面与 word 边界/VT 类名选择器 miss）。

## 教训

1. **「空白」在 web spec 里几乎总是 ASCII 五字符**，而 Rust/JS 标准库的空白原语默认
   Unicode 集——class 域、选择器 trim、`~=` 分词三处同坑；修一处时按「域」搜全
   消费点（本次三面八处一次性收口）。
2. 同 execute 的 `setAttribute` 不入 `__zw_matches` 读的快照（mutation 在 execute 结束
   apply）——测试该回调时属性值须进初始 parse HTML（WPT 用例即静态标记形态）。
3. engine 单测对 handle 元素（createElement）调 `matches` 恒 false（分支需 sel）——
   断言 matches 族语义要用文档内元素（getElementById）。
