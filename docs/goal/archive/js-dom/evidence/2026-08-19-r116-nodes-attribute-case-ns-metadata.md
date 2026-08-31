# R116 evidence — nodes：属性族大小写语义 + NS 元数据 + createAttribute

**日期**: 2026-08-19
**里程碑**: M4（WPT dom 上游基线按聚类驱动修复）
**基线**: R115（nodes 7146P/1051F；events 419P/32F；`make test` 18,235P）

## 簇与修复（五件）

1. **非 NS 属性族 HTML 小写 + 空名异常**（`_r116AttrName` helper，part04）：`setAttribute`/`getAttribute`/`hasAttribute`/`removeAttribute`/`toggleAttribute`/`getAttributeNode` 六个 trap 统一——HTML 文档 ASCII 小写（spec dom-element-setattribute：非 NS 族按小写匹配）；空名抛 InvalidCharacterError（productions.js invalid_names=['']）。
2. **NS 读大小写敏感**（getAttributeNS/hasAttributeNS 直查绕过小写 helper，part04）：旧经 proxy.getAttribute 转发会吃到小写——`hasAttributeNS('', 'CHEEseCaKe')` 误命中（spec：NS 查找精确匹配）。
3. **per-attr NS 元数据 registry**（`_attrNSMeta`，part01 声明 + part04 setAttributeNS 登记 + part03 attrObj 消费）：host 属性存储是扁平限定名（无 ns）——Attr 节点的 prefix/localName/namespaceURI 从 setAttributeNS 登记的 `{ns, prefix, local}` 取；get/hasAttributeNS 按 (ns, local) **反查任意 prefix 的存储名**（ns='http://FOO' + local 'def' 命中存储的 'abc:def'——`_nsQualName` 只认已知 prefix 映射的缺口闭合）。
4. **handle 元素 toggleAttribute presence**（part04）：旧 client-side 决策只查 sel（handle 恒 absent）——第二次 toggle 仍返 true；现经 `__zw_has_attr_handle` 判 + want=false 时真移除（`__zw_remove_attr_handle`）。
5. **createAttribute/createAttributeNS**（主文档 part06 + detached doc part03）：空名 InvalidCharacterError；**大小写按文档类型**（HTML contentType → 小写；XML doc 保持原样——与 createElement R115 语义一致）；detached doc（implementation.createDocument 产物）补齐两方法。**Attr 节点补 textContent/data**（= value，spec dom-attr——WPT attr_is 读 textContent）。

## A/B 结果（WPT testharness）

| 项 | R115 基线 | R116 | 净 |
|---|---|---|---|
| case.html | 131P/155F | **285P/1F** | +154 |
| attributes.html | 55P/193F | **214P/36F** | +159 |
| Document-createAttribute | 0P/36F | **36P/0F（100%）** | +36 |
| dom/nodes 全量 | 7146P/1051F | **7366P/831F** | **+220 净** |
| dom/events | 419P/32F | 419P/32F | 0 |
| dom/collections | 48P/1F | 48P/1F | 0 |
| dom/traversal | 1595P/10F | 1595P/10F | 0 |

## 单测（part20.rs +1）

- `test_attribute_case_and_ns_metadata_r116`：HTML 小写（setAttribute('CHEESE') → hasAttribute('cheese') 命中 + hasAttributeNS('', 'CHEESE') 不命中）+ 空名异常（setAttribute/createAttribute）+ NS 读大小写敏感（getAttributeNS('http://FOO','def') 命中 / 'DEF' null）+ Attr NS 元数据（prefix/localName/namespaceURI/textContent）+ createAttribute 文档类型语义（主文档小写 / XML doc 保持）+ handle toggle presence（二连 toggle true→false + hasAttribute 消失）。

## 验证

- `make test` **18,240 passed / 0 failed**（exit 0）
- `cargo fmt --all -- --check` 无 diff；workspace clippy 零警告
- engine js_dom_bridge 602 单测全绿（含 R116 +1）

## 教训

1. **NS 族与非 NS 族的大小写语义相反**：非 NS（setAttribute 等）HTML 文档小写；NS 族（setAttributeNS/hasAttributeNS）精确匹配——共享转发路径（NS 读经非 NS 写 helper）会把两语义搅在一起，NS 读须独立直查。
2. **扁平存储的 NS 语义须 JS 端登记**：host 侧属性按限定名扁平存（无 ns/prefix 结构）——Attr 节点字段与 (ns, local) 反查都要一层 JS-side registry；只靠 ns→已知 prefix 映射（xlink/xml）覆盖不了任意自定义 prefix。
3. **toggle 的 client-side 决策要判 presence**：toggle 返值 = 切换后状态——presence 检查对 handle 元素同样必须走 `__zw_has_attr_handle`（sel-only 判定使 handle 恒「不存在」）。
4. **插入位置验证**：往大对象字面量插方法时先确认锚点属于哪个对象（body vs doc 同名 createRange 方法——我插错过一次，`xml_document.createAttribute is not a function` 暴露）。
