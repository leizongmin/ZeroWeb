# ihe5 rawtext 4 键归因分析与决定性探针设计（核验轮转分析，2026-11-16）

来源：`.acceptance/dps/ihe4-iht-a.log`（批 ihe4，锚 `4f5e7a75d`，bin `7fa4cc0e9eac20d852b62c077cd5f92e`，iht 腿 1556P/9F）。rawtext 族 4 键（census B 族，ihc1 档案号 14-17）：

- `inserthtml "<b>abc</b>"` × `<xmp>f[o]o</xmp>`（swcss 双态）
- `inserthtml "<b>abc</b>"` × `<script>f[o]o</script>bar`（swcss 双态）

exp `<xmp>f<b>abc</b>o</xmp>`（markup 保留）→ got `<xmp>f&lt;b&gt;abc&lt;/b&gt;o</xmp>`（转义文本）。

## 已排除面（本轮静态归因）

1. **写链正确**：单元钉 `test_inserthtml_xmp_markup_payload_r3254_m3`（part112_ihtml_shell_sel_probe.rs:232）活跃且通过（ihe4 批 lib 4074P/0F 含此钉），同时断言 ①SetInnerHtml mutation 串 `<xmp>f<b>abc</b>o</xmp>` ②Rust 镜像 apply_mutations_to_html 含字面 markup。拼接臂（part06.js :3617 SPLICE 臂）在钉环境产出正确重建串。
2. **块臂非疑点**：`<b>` 载荷 rootTag 不入 BLOCK_T/SPLIT_T（:4193）恒旁路。
3. **R3216 序列化器不可能产出 got**：crates/dom/src/serializer.rs `is_raw_text_element` 已含 xmp/script（无条件 raw text，literal 直出）。无论解析态（rawtext 单文本子 or 普通解析 f+`<b>`元素+o），该序列化器输出均为 exp 形。
4. **转义形的数据层要求**：got 串内含字面 `&lt;b&gt;` 序列。R3216 序列化器下唯一来源 = 文本节点 data 本身含 `&lt;`——即「写链写入转义串且 rawtext 解析不解码实体」或「读链某序列化器对 xmp 文本做 escape」。前者已被钉测排除（mutation 断言字面）；故 **corpus 读链必经另一个无 rawtext 意识的序列化器**。
5. **corpus 对照组**：inserthtml.js :188（xmp+`abc` 文本载荷）、:203（script+`abc`）不在红集＝通过。文本无 `<>/&` 特殊字符，文本案通过不构成对读链的判别——分叉恰在载荷含 markup 字符时。

## 嫌疑面（未定谳）

- `js_dom_bridge/computed_style_cache.rs:512` `DomMutation::SetInnerHtml` 分支——style cache apply 侧自己的子树处理/重序列化。
- shim JS 侧序列化域（part03 `_tree.innerHTML` / `_zwMSerialize` 域——R369 注释证实其存在且独立于 Rust 链）。
- 查询视图文档烘焙链中某中间串化步骤。

## 决定性探针（刀前第一步行）

在钉环境（part112 同构：V8Sandbox + generate_js_dom_shim + register_dom_callbacks）复刻 corpus 读链——execCommand 后**经 shim `host.innerHTML` getter**（走 `__zw_get_inner_html` 回调，非 Rust 镜像 apply_mutations_to_html）读宿主：

- 复现转义形 → 钉死读链层；再二分 getter 链（`__zw_get_inner_html` → with_query_view_doc → query_inner_html_from_html_doc → doc.inner_html）与嫌疑面定位落点，按落点定刀界（修读链序列化器 rawtext 意识 vs 修拼接臂 payload 形语义）。
- 若 getter 读字面 → 分叉在 runner 环境（wpt-runner 的 harness 读形/引擎孪生视图），转而做 runner 单案插桩复跑（ZW_SCRIPT_TIMEOUT_MS 预算内单案可跑）。

## 边界

- 本档为读侧分析（零码面改动），刀界与实施归后续轮（writer 若认领实施则让位，本档按 ihc1 普查先例供采纳）。
- E 空壳 ×2 / F CE-false 邻接 ×2 / CE-false pre ×1 维持攒批搭车（v1.120 既定）。
