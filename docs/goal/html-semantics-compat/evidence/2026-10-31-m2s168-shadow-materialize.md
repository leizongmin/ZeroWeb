# M2-S168 Shadow DOM 物化与查询封装门（commit 1/2）

**日期**: 2026-10-31
**刀位**: s167 候选②（`the-link-element/link-rel-attribute.html` shadow 案，目录余留唯一 F）。本刀拆两提交：commit 1 = attach_shadow 物化 + 查询封装门 + 声明式 Shadow DOM 序列化回环（本文）；commit 2 = shadow 域级联下发（ACTIVE_SHADOW_CSS_ENTRIES 槽，commit 1 落地后续接）。

## 勘根定谳

探针（zz-s168-shadow-probe.html，验后删）三态统一归因：

- p9 cloneNode 变体 inline-style 反射：expected `rgb(1, 2, 3)` got `''`
- p10 importNode 变体 inline-style 反射：expected `rgb(4, 5, 6)` got `rgb(0, 0, 0)`
- p11 两变体 `.green` 级联：expected `rgb(0, 128, 0)` got `rgb(0, 0, 0)`

机制：shim 侧 shadow 容器是 fragment handle（`_shadowRoots`/`_shadowHandles`/`_fragmentHandles` 三表），engine 侧 shadow 内容 append 走 fragment flatten wire（part04.js `__zw_append_fragment_children_handle`）落 **detached fragment**——`styles` map 无条目 → serialize 默认黑（p10/p11）；replay Err 截断 → nodes miss → `''`（p9）；级联走查（collect_stylesheets → compute_styles_recursive）永不访问 shadow 子树。layout-engine tree.rs 的 shadow+slotting 基建（`doc.shadow_root(dom_id)` → collect_shadow_children）休眠中，只等文档树里有 shadow 内容。

## 施工（10 文件，+411/−16）

**engine（4 文件）**

- `js_dom_bridge.rs`：mutation 双变体 `AttachShadow { host_selector, root_handle, mode }` / `AttachShadowOnHandle { host_handle, root_handle, mode }`；翻译臂（unanchored sel host 前置翻译）+ 执行臂（`find_by_selector` → `doc.attach_shadow` → `handles.insert(root_handle, root)`——**root_handle 注册 ShadowRoot NodeId 后，既有 fragment flatten / append 零改动自然落树**）；unanchored handle host lenient warn 跳过（R125 口径）。
- `js_dom_bridge/callbacks.rs`：`__zw_attach_shadow` / `__zw_attach_shadow_handle` 注册。
- `js_dom_shim/part05.js`：`_attachShadow` 尾部（`_shadowRoots[key]` 赋值后 return 前）wire 上报（sel / handle 双形态，try/catch 不阻断页面路径）。
- `js_dom_bridge/computed_style_cache.rs`：gCS by handle `nodes` miss 臂加 `query_selector_including_shadow` 回落——gCS 是对象身份通道，页面持元素对象访问 shadow 内容合法，不受 light DOM 查询封装约束。

**dom（4 文件）**

- `document/mod.rs`：`is_shadow_root_node` 谓词 + 九处遍历下降点 shadow 门（collect_by_tag_names / collect_by_tag_name / collect_by_tag_name_ns / collect_by_class_name / find_first_matching_opt / find_first_matching_chain_opt / find_first_matching_chains_opt / collect_matching_opt / collect_matching_chains_opt——light DOM 查询封装化）；`get_element_by_id` shadow_roots 非空时父链 ShadowRoot enclosure 校验（id_map 命中 shadow 内元素返 None，空表快门零开销）。
- `document/shadow.rs`：`shadow_root_ids()`（slotmap 插入序）；`query_selector_including_shadow()`（light 先查，miss 后逐 shadow root 为根——封装门不剪根自身；嵌套 shadow 不在面 FIXME）；`materialize_declarative_shadow_roots()`（`<template shadowrootmode=open|closed>` → 宿主 attach_shadow → contents 移入 → 摘 template；属性缺省/非法保留普通 template，spec 语义）。
- `serializer.rs`：`ShadowRoot` arm 输出 `<template shadowrootmode="open|closed">` 声明式形态（drain 清队列重序列化回环不丢 shadow 内容）；`inner_html` 主循环 shadow 子跳过（封装；full-doc 走声明式分支不经此处）。
- `parser.rs`：`parse_html_with_builder` 解析后 `materialize_declarative_shadow_roots()`（页面直写声明式形态同回环）。

**style-system（2 文件）**

- `lib.rs`：`ShadowScopeStylesheets { root, sheets }` + `ShadowScopeView`（per-root (sheets, index) 查找，空表兜底）+ `compute_styles_with_shadow`（并集预扫 registered_properties/font_feature_values/伪元素旗标 chain shadow sheets，cache_safe = doc AND all scopes；per-scope 预建 index；递归 17 参传 view）；子代循环 ShadowRoot 边界换传 per-scope pair；`compute_styles_incremental` shadow 包含退化门（`is_shadow_enclosed` → 全量重算）。
- `font_feature_values.rs`：`collect_font_feature_values` IntoIterator 化（shadow 表 chain 迭代消费，两轮物化引用表）。

commit 1 级联语义：shadow 子树走查取**空表**（UA + host 继承）——blob `.green` 不漏入 shadow 域（p11 预期红）。级联下发是 commit 2。

## 验证

- check / fmt（hunk 全落本次编辑函数）/ guarded-clippy workspace 全绿。
- engine lib `js_dom_bridge` **3611P/11F**，失败名单与 s167 基线同集（fetch 族 9 + ua_display_li + style_sheet_from_handle_r113），零新增。
- shim 13-part 拼接 `node --check` 过。
- 探针（commit 后删）：**p9/p10 `''`/黑 → Pass**（物化 + 身份通道双臂生效）；p11 保持红（级联未下发，commit 2 面）——三态与勘根设计逐位对应。
- 目录 `the-link-element`（含探针 115P/2F；剔探针 **113P/1F 与 s167 账面逐位一致**，余 1F 同为 link-rel-attribute 级联面）：零位移。
- `shadowrootmode` 语料命中面 5 案定向（parser materialize 全 parse 生效的实际作用域）：form-indexed-element-shadow 1F = M1 基线同态；form-submit-dialog-shadow 1F = M1 同态；dialog-focus-previous-outside 3P/3F = s167 stash 二分钉定位集；dialog-popover-closedby-complex 24P/16F 落 s167 钉定漂移带（16F~27F 负载相关，s167 stash 二分已证非代码归因）；chrome-531745321-crash 非 testharness 形态零枚举。零 P2F。

## 过程注记：runner 静默 exit=1 定谳（非回归）

commit 1 验证期 `FILTER=zz-s168-shadow-probe` / `FILTER=the-link-element` 直跑 runner 仅输出 `Using 8 test job(s).` + exit 1 + 零结果行。插桩二轮（DBG-SPECS/DBG-FILTER，验后还原）定谳：**直跑时 `FILTER=x` 整串（含前缀）作为 filter 位置参数进路径子串匹配，恒假 → 空用例集 → exit 1**（`cmd_testharness_html_semantics` 空集不打印直接退）。make 目标会剥 `FILTER=` 前缀只透传值；**直跑 runner 必须传裸值**。非代码回归，勘根插桩已还原（`git diff` 零残留）。

## FIXME 新记

- 嵌套 shadow 级联（`query_selector_including_shadow` 与 per-scope 走查均不入嵌套面；声明式嵌套声明同不重建）。
- `document.outerHTML` 含声明式 `<template shadowrootmode>`（Chrome 不含）——序列化回环需要、上游对照语义差，记账。
- `getHTML()` serializableShadowRoots 分型未建模。

## commit 2 待续

级联下发：ACTIVE_SHADOW_CSS_ENTRIES 槽（host_sel/sel/text 结构化）+ `__zw_set_shadow_css_entries` + 缓存键扩元（GENERATION_CACHE/HANDLE_VIEW_CACHE/cs_cache，两谓词逐字同步）+ engine collector `collect_shadow_scoped_sheets`（host_sel 锚定 → `doc.shadow_root` → texts parse）+ `compute_styles_with_shadow` 填表 + shim `_zwSsRecomputeExt` shadow 循环 + a2–a4 反应链验证（s158 setAttribute 门对 shadow 元素触发面待验）。
