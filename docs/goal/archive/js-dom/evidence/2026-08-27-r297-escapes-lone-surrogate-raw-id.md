# R297 Evidence — querySelector escapes 孤立代理 never-match（JS 原值 id 缓存，escapes 66P/2F→68P/0F 100%）

**日期**: 2026-08-27
**切片**: M4——R297(d) selector 小簇首件（escapes 2F）
**改动面**: `part01.js`（`_zwRawIds` per-key 原值缓存）+ `part03.js`（id getter 原值优先）+ `part04.js`（id setter / setAttribute('id') / removeAttribute('id') 三写点同步）+ `js_dom_bridge_tests/part24.rs`（新测试段，+1 单测）

## 一、根因

WPT `ParentNode-querySelector-escapes` 两失败（`"\ud83dsurrogateFirst" should never
match with "#\\d83d surrogateFirst"` 及 trailSurrogate 变体）：

1. `child.id = "\ud83d…"`（孤立代理）→ proxy set trap → `__zw_set_attr_handle`
   跨 V8→Rust 边界，`to_rust_string_lossy` 把 lone surrogate **换损为 U+FFFD**
   （Rust `String` 须合法 UTF-8）——host 存储值已是 `"\u{FFFD}surrogateFirst"`；
2. 选择器 `#\d83d …` 的转义按 spec（css-syntax §4.3.4 consume an escaped code
   point）对 surrogate 码点解码为 **U+FFFD**（JS `_readCompoundToken` 与 Rust
   `unescape_css_string` 的 `char::from_u32(0xd83d)=None→FFFD` 双侧一致——正确）；
3. JS 客户端匹配（handle 容器 `_handleQueryFirst` → `_hIdOf` 读 `p.id` → 回读
   host 的 lossy 值）→ **U+FFFD === U+FFFD 误命中**。真浏览器：DOM id 保留
   lone surrogate（UTF-16 code unit 语义），U+FFFD 选择器 ≠ \ud83d id → null。

探针实证（part24 前身 probe 单测，sandbox 直跑）：
`handle:WRONG|handleId:"�surrogateFirst"`（id 读回已 lossy）。

## 二、修复：JS 侧 id 原值缓存（`_zwRawIds`）

**取向**：host 侧无损化（WTF-8 编码全 callback 面）是深结构 marshalling 改动，
超出轻量切片；WPT 失败域只在 JS 客户端匹配——JS 侧保真即可解。

- `part01.js`：`_zwRawIds = {}`（per-element-key，同 `_classCache` 模式）——仅当
  值含 lone surrogate（`/[\uD800-\uDFFF]/`）时入表，否则清表走 host 快路径（零开销）；
- `part04.js` 三写点同步：`.id=` setter / `setAttribute('id', v)` /
  `removeAttribute('id')`（delete 回落 host）；
- `part03.js` id getter：`hasOwnProperty(_zwRawIds, key)` 命中返原值；
- `_hIdOf`（part05 客户端匹配）读 `p.id` 自然取原值——\ud83d ≠ U+FFFD → 不命中。

**对照组不回退**：id 本体就是 U+FFFD 时（WPT testMatched 族
`"\u{fffd}surrogateFirst" should match`）不走 raw 表（无 surrogate）→ U+FFFD ===
U+FFFD → 正常命中（单测 `fffdMatch:hit` 断言）。

**已知限制（记档）**：host 侧（doc 树/id_map/Rust matcher）的 id 仍是 lossy 值——
sel-based（in-document）元素的 Rust 侧 `#id` 匹配对 lone-surrogate id 与 U+FFFD
选择器仍会相等（probe `indoc` 形态）。WPT escapes 套件只测 detached 容器形态
（createElement div + appendChild），本切片覆盖全部失败用例；host 侧无损化归
深结构（L2 live Document 域，随 M1 收口一并评估）。

## 三、验证

| 套件 | 基线（stash A/B） | R297 | Δ |
|---|---|---|---|
| ParentNode-querySelector-escapes | 66P/2F | **68P/0F（100%）** | +2P/-2F |
| ParentNode-querySelector 全族（All/All-content/case/escapes/scope/removed…） | 2046P/9F | 2048P/7F | +2/-2（恰 escapes 两例；余 tree-order 4F + scope 2F + removed 1F 为既存） |
| getElementById 族 | 23P/0F | 23P/0F | 持平 |
| **dom 全量 sweep**（nodes/events/collections/traversal/ranges，TIME_LIMIT=2400 双跑） | 54101P/94F/25T | **54106P/92F/22T** | +5P/-2F；set-diff：Fail 恰 -2（escapes 两例）；Timeout 差 5 例双向——`ParentNode-querySelector-All-content` 基线 4 跑 2 Timeout 实证 flaky（环境时序噪声，非本修复引入） |
| engine 单测（v8） | 2434 | **2435**（part24 新段 +1：lead/trail 双向 never-match + U+FFFD 对照命中 + id 读回保真 + setAttribute/removeAttribute 路径同步 + plain 回归） | +1 |
| make test | — | 1F = `window_surface_present_smoke`（XOpenDisplayFailed 环境项，run-rules §10 既档） | 持平 |
| fmt / clippy（-p zero-engine --all-targets -D warnings） | — | 干净 | — |

## 四、教训

1. **Rust String 边界即 lossy 边界**：JS（UTF-16 code unit 语义，可持孤立代理）
   与 Rust（String 须合法 UTF-8）之间的任何字符串跨界都隐含 U+FFFD 换损——
   「id 读回 ≠ id 写入」这类保真断言是边界换损的通用检测器。
2. **两侧同解码不是等价保障**：选择器侧 surrogate→U+FFFD 与存储侧 surrogate→U+FFFD
   是**两个不同语义域的碰巧同值**（前者是 spec 规定的 special replacement，后者是
   marshalling 副作用）——正确语义恰要求它们**不相等**。
3. **对照组必须同测**：never-match 修复必须带 testMatched 对照（U+FFFD 本体 id
   仍须命中），否则「全部返 null」的过宽修复会静默通过。
