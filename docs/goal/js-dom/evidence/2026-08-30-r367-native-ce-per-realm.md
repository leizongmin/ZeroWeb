# R367 — CE registry 专项 slice 3：native CE hooks per-realm 查表（ownerId 域判据贯通）

**日期**: 2026-08-30
**切片**: CE registry 专项第三片（Rust `factories`/`custom_elements` CE hooks 与 polyfill
per-realm registry 的查表对齐——default-on 前对齐项）
**改动面**: `dom_bindings/factories.rs` + `custom_elements.rs` + `mod.rs`（owner id 解析
与第三参传递）+ `part03.js`（`__zw_native_ce_entry` 域判据 + 三 hook 改造）+ 双测试文件

## 1. 改动

1. **Rust 侧 owner id 传递**——`try_upgrade_custom_element`（createElement upgrade）、
   `dispatch_connect`（connect/disconnect lifecycle）、`notify_attribute_change` 三处把
   元素 node document root（`Document::owner_document`，ffi u64 十进制串）作为新参数传给
   polyfill hook（lookup 第 2 参 / notify_connect 第 4 参 ownerIds 数组 / notify_attr_change
   第 6 参）。
2. **`__zw_native_doc_root_id()` 全局探针**（mod.rs）——当前 live Document root 的 ffi
   字符串，polyfill 侧域判据的「本文档 root」锚点。
3. **part03 `__zw_native_ce_entry(tag, ownerId)` 归一入口**——ownerId 缺省 / 等于本文档
   root → 主实例三变量；distinct root → 该 doc `_zwCERegistry` 槽（R364 per-realm 实例 +
   R365 槽接线）的 get 记录；**探针缺席（shim-only 沙箱）→ 主实例回退**（native 生产路径
   探针恒在，域路由只在 default-on 后生效，shim-only 回退 = 主文档语义零回归）。三个
   native CE hook（lookup/notify_connect/notify_attr_change）全部改经此归一入口。

## 2. 过程勘误（单测当场抓回）

**detached 节点的 owner_document 陷阱**——`Document::owner_document` 沿 parent 链上行，
对已摘除节点（removeChild 后 parent=None，节点仍在 arena）返回**节点自身** → 产出
bogus root id ≠ doc root → polyfill 域判据误判 FOREIGN → 主实例 miss →
**disconnectedCallback 丢失**（webview `test_native_custom_element_e2e_lifecycle_r3270`
从 "cd" 退化为 "c"，三个 r3270 e2e 当场红）。修复：owner 解析在 detached 形态
（owner_document == Some(id)）回落 live Document root——本架构 live Document 单源、无跨
文档 native 元素，FOREIGN 臂仅为 per-realm 查表预留的形态位。**教训**：给「root 比对」
类判据引入新语义时，必须枚举节点的全部生命周期态（connected / detached-but-in-arena /
removed）——detached 节点的「沿链到自身」是 owner_document 的隐式契约外行为。

## 3. 验证（landing 门）

| 门 | 结果 |
|----|------|
| 全量 dom sweep（polyfill，TIME_LIMIT=2400） | **55491P/13F/15T——Fail 文件集合 11=11 恒等零回归**（+2P 净正、Timeout -2 为并发噪声轮转族收敛） |
| native 路径 spot check | node-realm 族 15P、Event-dispatch 族 221P、目标件维持（polyfill/native 一致） |
| engine 单测 | v8 **2495**（+3：`native_ce_hooks_per_realm_lookup_r367`[ownerId 传播形态 + 主路径不回归] / `native_ce_entry_fallback_without_domain_probe_r367`[探针缺席回退] / part24 `test_native_ce_entry_domain_routing_r367`[shim 侧域判据四断言]）/ quickjs 1472 全绿 |
| webview | r3270 CE e2e 三个恢复全绿（勘误验证）；`navigator_controller_tracks` 1F 为 SW 流既存 flake（clean HEAD 同败，run-rules §10；R342 已记档） |
| integration | 781P 全绿 |
| clippy / fmt | v8 + quickjs 双矩阵 `-D warnings` 零警告 / 无 diff |
| make test | 唯一失败 XOpenDisplayFailed 环境既存项 |

## 4. CE registry 专项收口状态

- **R364** per-realm registry 实例（define 冲突分离）✅
- **R365** 创建路由（工厂 innerHTML setter node-document 查表升级）✅
- **R366** ShadowRoot.prototype innerHTML（shadow 域路由 + 工厂委托 + realm 转发）✅
- **R367** native CE hooks per-realm 查表（Rust↔JS ownerId 贯通）✅

已知 Fail 文件集合余 **11**（全部深结构/基建域定性，见 R366 evidence §5）：
frame-removal 1F（creation realm 记录——`document.getElementById` 访问路径不依赖 JS 对象域，
需解析子独立 creation-realm 印记；本轮探针实证主文档查询面对 adopted 工厂子树整体不可见
[q:null/gebi:null/mainP:0，R220 identity 域已知形态的 document 级面]）、Node-isConnected
iframe 专项、MutationObserver-document 3F（parse-time）、cross-realm 1F（工厂 body 可观察
id）、remove-and-adopt-thcrash（window.open 无 popup 通道）、querySelector-mixed-case /
remove-next-sibling（R220 identity 双源域）、events 2F（onerror 跨 realm / Chromium 专有
pseudo）、ranges dataChange/replaceData 2F（文件级 Timeout 尾批）。

## 5. 后续

- **frame-removal 1F / document 级查询可见性**（工厂子树 adopted 后的主文档 GEBI/QSA
  可见性——R220 identity 域的 document 级缺口，与 querySelector-mixed-case 同簇）；
- **M5/M7 default-on**（待用户点名，改 Mission 级单向门）；
- M4 基线持续维护。
