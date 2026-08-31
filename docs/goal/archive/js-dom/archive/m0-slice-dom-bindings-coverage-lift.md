# M0 Slice R31 — dom_bindings coverage 提升 + DOMException 真 bug 修复

**日期**: 2026-08-14
**里程碑**: M0 → M4（DC-4 持续提升覆盖率，不退化）
**切片**: R31
**前置**: R30（dom_bindings coverage 口径落地，基线源码 93.14%）

## 切片选择（决策记录）

R30 建覆盖率基线后，3 最低文件为提升候选：dom_exception 71.4% / css_style_declaration 86.7% / custom_elements 89.0%。本切片纯补单测覆盖（test-only，dom_bindings 为本目标自有工作面，零碰撞），直接服务 DC-4「dom_bindings 覆盖率持续提升、不退化」。写测试时发现一个**真 bug**（DOMException 构造 name 缺省），同步修复。

## 真 bug 修复（DOMException 构造 name 缺省）

**现象**：`new DOMException('boom')`（无 name 参）实例 `name === "undefined"` 而非 spec `"Error"`。

**根因**：`dom_exception.rs` `native_dom_exception_constructor_invoke` 用 `super::string_arg(scope, &args, 1)` 读 name 参。`string_arg` 实现 = `args.get(idx).to_string(...)`——缺省参 `args.get(1)` 返 `undefined`，`.to_string()` → `"undefined"`（非空），故 `if n.is_empty() { "Error" }` 分支不进，name=undefined。

**影响**：native 路径 `new DOMException(msg)`（不传 name）name 错误，WPT `assert_throws_dom` 按 name 判定时该构造路径不合规（default-on 生产路径缺陷）。

**修复**：`native_dom_exception_constructor_invoke` 改先判 `args.get(idx).is_undefined()`：
- message 缺省（undefined）→ `""`（spec message 缺省空串）
- name 缺省（undefined）→ `"Error"`（spec name 缺省 "Error"）；非空用原值；显式空串仍 "Error"（error-names-table 无空名）

`throw_dom_exception`（总传非空 message+name）不受影响。

## 补的单测（9 个，tests.rs）

### DOMException（覆盖 dom_exception.rs：构造/code 表/toString/legacy 常量/constructor）
- `native_dom_exception_constructor_defaults_r31`：message 透传 + name 缺省 "Error" + code=0（Error 无 legacy code）+ message 缺省空串
- `native_dom_exception_name_to_code_table_r31`：21 条 name→code 全表（含此前未触发的 13/14/15/18/19/20/21/23/24/25 + 默认 _ => 0）
- `native_dom_exception_to_string_r31`：`"name: message"` / message 空→仅 name / 缺省 name Error（覆盖此前几乎无测试的 `native_dom_exception_to_string_invoke`）
- `native_dom_exception_legacy_constants_r31`：`DOMException.SYNTAX_ERR` 等构造器常量（含 INVALID_MODIFICATION_ERR/NAMESPACE_ERR 等此前未触发的 register_const）
- `native_dom_exception_constructor_identity_r31`：`instance.constructor === DOMException`（prototype.constructor 链）

### custom element lifecycle（覆盖 custom_elements.rs：connect/disconnect/attr-change 派发路径）
- `native_custom_element_connect_disconnect_lifecycle_r31`：body.appendChild→connect / body.removeChild→disconnect（`__zw_native_ce_notify_connect` 记录器，覆盖 notify_connect_after_insert + notify_disconnect_after_remove + dispatch_connect 主体）
- `native_custom_element_attribute_change_dispatch_r31`：setAttribute/removeAttribute→attr-change 派发（`__zw_native_ce_notify_attr_change`，old/new/null 全路径，覆盖 notify_attribute_change 真派发）
- `native_custom_element_attr_change_fast_path_skip_r31`：非 custom tag（div）setAttribute fast-path 跳过（`!tag.contains('-')` return）

### CSSStyleDeclaration（覆盖 css_style_declaration.rs：getPropertyPriority/item 边界/named-deleter）
- `native_style_property_priority_r31`：getPropertyPriority important/非 important/未设 + setProperty important upsert（覆盖 get_property_priority + set_property important 分支）
- `native_style_item_boundary_and_named_deleter_r31`：item() 越界/负 index 空串 + `delete el.style.color` named-deleter（覆盖 native_style_named_deleter + item 边界）

## 验证

- **单测全绿**：engine v8 2107 passed（+10 vs R29 2097，含 9 新 R31 + 真 bug 修复回归），quickjs 1410 passed
- **fmt + clippy 双矩阵**：`cargo fmt --all -- --check` clean；zero-engine v8 + quickjs clippy 零警告
- **coverage 提升**（`scripts/check-dom-bindings-coverage.sh`）：

  | 文件 | R30 基线 | R31 | Δ |
  |---|---|---|---|
  | dom_exception.rs | 71.4% | **94.4%** | +23.0pp |
  | css_style_declaration.rs | 86.7% | **92.7%** | +6.0pp |
  | custom_elements.rs | 89.0% | 89.0% | +0（lifecycle 派发路径补测，但剩 19 行防御/OOM 边缘不实际触发） |
  | **dom_bindings 源码总** | **93.14%** | **94.27%** | **+1.13pp** |
  | dom_bindings 全部 | 95.15% | 95.94% | +0.79pp |

- **基线 JSON**：`evidence/2026-08-14-r31-dom-bindings-coverage.json`

## 决策记录

- **custom_elements 剩 19 行为何不硬补**：经 lcov 逐行核对，剩余未覆盖行均为防御/OOM/不实际触发分支：
  - `dispatch_connect`：`v8::String::new` 返 None（OOM，line 98）/ notify 未注册（101）/ get_or_create_native_element None（111，需未注册 NodeId）/ element_tag None（114）/ pairs 空（119，custom 元素总配对成功）
  - `notify_connect_after_insert` disconnect 分支（47/58/59/64）：`!parent_connected && was`——已连接元素重新 appendChild 到 detached parent（不实际）
  - `collect_custom_subtree`（145）：`node.get(id) None` stale NodeId
  - `element_tag`/`element_tag_inner`（189/270）：非元素 tag 分支（text/comment 节点 setAttribute，但 Text 无属性）
  - `notify_attribute_change`（221/230/233/236/239/242/257）：`Some(x) else return` 防御返回，happy-path 跳过，仅失败时 hit

  这些是 defensive guards（spec 允许保留），强行构造触发场景（如注入 OOM）违背「不为不可能发生的场景编写错误处理」与测试有效性原则。custom_elements 89.0% 已接近 90%，记低 ROI 候选（⑨）不硬补。

- **DOMException bug 为何是真 bug 非仅 coverage**：`new DOMException(msg)` 是 JS 侧构造 DOMException 的标准用法（polyfill part01b 同款 `throw new DOMException(msg, name)`）。native 路径此构造 name 错误会破坏 default-on 后的 assert_throws_dom 合规。修复是 production 正确性净正，非为刷 coverage 而改生产代码。

## 残留（转 R32+）

- **Event-dispatch 系列**（深结构，~33 个 0-pass 主力）：最高 ROI 但需 document/window listener 独立存储 + Document/Text 基础设施
- custom_elements coverage 剩 19 防御行（低 ROI ⑨）
- 双路径差 6.56pp 收口 / native namespaceURI getter / iframe.contentDocument / querySelector-mixed-case（见 master.md 剩余聚类）
