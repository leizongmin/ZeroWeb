# M4 Slice R18 — createElementNS 大小写敏感 + prefix/localName/namespaceURI getter

**日期**: 2026-08-14
**里程碑**: M4 — WPT dom 上游基线 + 按聚类驱动修复
**切片**: R18
**前置**: R12（element.prefix getter + getElementsByTagNameNS，遗留 abc/Abc 三态仅 ABC 匹配的限制）

## 问题

WPT `dom/nodes/case.html` 测 createElementNS 的 **qualifiedName 大小写敏感性**（abc/Abc/ABC 三态 prefix/localName）。spec `dom-document-createelementns` 明确 createElementNS **不**小写 localName（`"Abc"` → localName `"Abc"`），区别于 createElement 的 HTML 无条件小写。

R12 时 polyfill `createElementNS` 直接调 `__zw_create_element`（走 createElement 路径），且 `_realTag` 强制大写化——导致：
- `createElementNS('ns','abc:l')` 的 prefix 返 `'ABC'`（大写化），仅 ABC 态匹配 case.js，abc/Abc 态 fail
- `createElementNS('ns','Abc')` 的 localName 经 `toLowerCase()` 变 `'abc'`，丢失大小写

## 修复

polyfill `document.createElementNS` 改经**新回调** `__zw_create_element_ns` → host `doc.create_element_ns`（dom crate，已正确处理 prefix/local 大小写敏感 + namespace 经 `QualName` 存储）：

1. **`js_dom_bridge.rs`**：新增 `DomMutation::CreateElementNS { handle, namespace, qualified_name }` + `apply_dom_mutations` 分支（路由到 `doc.create_element_ns`）。
2. **`callbacks.rs`**：注册 `__zw_create_element_ns(namespace, qualifiedName)` 回调（push CreateElementNS mutation，返 handle）。
3. **`part01.js`**：新增 `_nsHandles = {}`（存 `{ qualifiedName, namespace }` 原值，与 `_piHandles` 对称的 handle 标识模式，跨 part01-06 共享作用域）。
4. **`part03.js`**：新增大小写敏感解析 helper `_nsLocal`（冒号后）/`_nsPrefix`（冒号前，无则 null）/`_nsQualified`（原样）。
5. **`part04.js`** get trap：
   - `tagName`/`nodeName`：isNs → `_nsQualified`（大小写敏感 qualifiedName）
   - `localName`：isNs → `_nsLocal`（不 `toLowerCase`）
   - `prefix`：isNs → `_nsPrefix`（大小写敏感，`abc:l` → `'abc'`）
   - 新增 `namespaceURI`：isNs → 读 `_nsHandles[handle].namespace`；普通 createElement 元素恒 `'http://www.w3.org/1999/xhtml'`（spec HTML 元素）
6. **`part06.js`** `createElementNS`：改调 `__zw_create_element_ns`（fallback 旧 `__zw_create_element`），记 `_nsHandles`。

## 验证

- **单测**：更新 `test_prefix_and_get_elements_by_tag_name_ns_r12`（prefix 断言 `'SVG'`→`'svg'`，大小写敏感）+ 新增 `test_create_element_ns_case_sensitive_and_namespace_uri_r18`（abc/Abc/ABC 三态 prefix + 裸名无 prefix + localName 大小写敏感 + namespaceURI SVG/null/HTML）。v8 矩阵两个测试均 pass。
- **clippy 双矩阵**：v8 + quickjs（`--no-default-features --features quickjs`）零警告。
- **fmt**：`cargo fmt --all -- --check` 无 diff。
- **WPT dom/nodes 双路径对照**（完整 JSON 入 evidence）：

  | 路径 | R17 基线 | R18 | Δ |
  |---|---|---|---|
  | Polyfill | 53.20% | **55.11%**（2481P/2021F） | **+1.91pp** |
  | Native | 52.93% | **54.46%**（2452P/2050F） | **+1.53pp** |

  `case.html` createElementNS abc/Abc/ABC 三态全 Pass（R12 仅 ABC）。双路径对等差 0.65pp（polyfill 修复更直接，native namespaceURI getter 部分仍依赖 shim 共享逻辑，可接受范围）。

## 决策记录

- **为何 polyfill document 必须实现 createElementNS（非仅 native）**：沿用 R9 发现——用例侧 `globalThis.document` 始终是 polyfill shim 装的（part06.js），即使 `ZW_NATIVE_DOM=1`。native document template 方法用例访问不到。故 polyfill document.* 须实现所有方法，native dom_bindings 仅作 default-on 后的生产能力。
- **namespaceURI 对 null/空串处理**：上游用例用 `createElementNS(null, ...)`（Document-createElementNS.html / Element-classlist.html）。part06 `(ns == null) ? '' : String(ns)` + `_nsHandles` 存 `(_nsStr || null)` → getter 返 `null`（spec 无命名空间）。
- **`querySelector-mixed-case.html` 既存失败非 R18 回归**：R14 evidence 已记同样失败（`[viewbox] expected 2 but got 0`），属 selector 引擎对 mixed-case attributes 命名空间匹配 gap（selector 域），与 R18 createElementNS getter 修复正交。stash 对照确认改动前后同失败。

## 残留（非本切片）

- `querySelector-mixed-case.html`（selector 引擎 case/namespace 匹配，属选择器域）
- native 路径双路径差略增（0.27→0.65pp）：native namespaceURI getter 经 dom_bindings element.rs 是否独立读 namespace 待 M6/M1 对齐评估
- iframe.contentDocument（createElementNS 子文档工作，深结构 html-compat 域，仍记未解决问题）
