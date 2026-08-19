# R122 — M4 nodes：Attr identity 绑定表 + 同名多实例覆盖层（attributes.html 全 100%，+38 净）

**日期**: 2026-08-19
**里程碑**: M4（WPT dom 上游基线建立与扩展）
**驱动用例**: `attributes.html`（36F→67P/0F 全 100%）、`attributes-namednodemap.html`（2F→8P/0F 全 100%）
**规范**: https://dom.spec.whatwg.org/#dom-element-setattributenode / #dom-element-setattributens / #dom-attr-value

## 结果摘要

| 路径 | 前（R121） | 后 | 净 |
|------|----|----|----|
| polyfill nodes 全量 | 7631P | 7669P | +38（attributes 36F→P + namednodemap 2F→P + Attr-prefix HTML 4F→P，零新增聚类） |
| events / collections / traversal | 419P / 48P / 1595P | 419P / 48P / 1595P | 零回归 |

## 根因与修复（五层）

1. **Attr identity 无绑定**（`attributes[i] === getAttributeNode('foo')`、
   `setAttributeNode(attr) === attr` 往返恒 false——每次 `_zwMakeAttr` 新对象）。
   `_zwAttrBindings`（elKey → Map(限定名 → Attr 对象)）：`attrObj` 取/建/登记统一入口
   （readNames 名含 `'\x00#k'` 合成后缀作键——同 qname 多实例各有 identity）；
   `setAttributeNode`/`setNamedItem` 经 `_zwSetAttributeNodeCore`（InUse 校验 +
   (ns,local) 替换 + ownerElement 绑定）；`removeAttribute`/`removeAttributeNS`/
   `removeAttributeNode`/`removeNamedItem` 解绑（ownerElement=null——闭合
   "Attribute loses its owner when removed" 后 setAttributeNode 误抛 InUse）。
2. **host 扁平限定名存储无法表达同 local 多 ns**（`setAttributeNS('ab','attr')` +
   `setAttributeNS('kl','attr')` 两实例并存；非 NS `getAttribute` 返第一个 local 匹配）。
   `_zwAttrInstances`（elKey → 有序 [ {qname, ns, prefix, local, value} ]）：JS 侧
   权威多实例视图；host 只写每 local 首实例（渲染 best-effort）。非 NS 读
   **host 优先、实例层 local 兜底**（classList/style/className 直写 host 路径不经过
   实例同步面——首版 instance-first 使 classList.add 后 getAttribute 读回 stale，
   Element-classlist 550F 回归；host-first + 直写点同步双修）。
3. **`attr.value = v` 写回不传播**（own 数据属性拦截赋值）。`Attr.prototype.value/
   nodeValue` accessor（`_r122V` 存储 + 幂等护栏）：setter 经 ownerElement
   setAttribute/setAttributeNS 回写（spec `dom-attr-value`「change an attribute」）。
4. **NamedNodeMap 方法 identity**（`map.item === NamedNodeMap.prototype.item`）：
   per-element Proxy 缓存 `_zwNNMCache`（缓存命中先于原型重赋值——否则每次
   el.attributes 刷新原型为新材料闭包，identity 分叉）；named 'item' 属性不遮蔽方法
   （get trap 分支序：方法先于 named property）。
5. **setAttributeNS validate-and-extract 全分支**（InvalidCharacterError：
   空名/前置冒号/尾冒号/空白与'>'/首字符非 NameStartChar；NamespaceError：prefix
   存在 ns 空、'xml'/'xmlns' 保留绑定、XMLNS ns 形态）+ **小写化收窄到 HTML-ns
   元素**（`_r116AttrName` 检查 `_nsHandles`——非 HTML ns 元素限定名大小写敏感
   保留，WPT "Non-HTML element with upper-case attribute"）。
6. **`el.style = cssText` 落 expando**（style 不在 reflected 列表 → hasAttribute
   ('style') 恒 false）。显式 set trap 分支写 style 内容属性（WPT "Toggling
   element with inline style"）。

## 回归与修正（过程中三处）

- **Element-classlist 550F**：instance-first getAttribute 使直写 host 的 classList
  路径读 stale → host-first + classList/className/style 直写点实例同步。
- **`_r116NsQName is not a function`（Attr-prefix HTML 4F）**：var 函数表达式定义在
  属性族分支之后，分支先 return 时闭包捕获 undefined → 定义上移至分支前。
- **ownKeys 泄漏合成名**（"a\0#2" 出现在 getEnumerableOwnProps）：supportedNames/
  ownKeys/getAttributeNames 统一经 `_zwAttrStripSyn` 剥离。

## 验证

- attributes.html 67P/0F + attributes-namednodemap 8P/0F（polyfill 路径全 100%）
- nodes 全量 7669P（+38 净，剩余 fail 聚类与 R121 相同：PI-attributes 133F +
  removeChild 28F[frames 域] + getElementsByClassName-whitespace 19F 等）
- events 419P / collections 48P / traversal 1595P 零回归
- engine 单测 `test_attr_identity_and_multi_instance_r122`（10 断言组）
- `make test` 全绿 exit 0；fmt 无 diff；clippy `-D warnings` 零警告
- 账本：`tests/wpt-runner/imported-tests.txt`（R122 条目）

## 设计注记

- **绑定键含合成后缀**：同 qname 多实例（ab:attr + kl:attr）的 Attr identity 按
  readNames 合成名（'attr'/'attr\0#2'）分别绑定——裸 qname 键会两实例互抢。
- **instance 层是 NS 语义的权威、host 是渲染权威**：非 NS 读写走 host-first 保
  直写路径一致；NS 读写（getAttributeNS/hasAttributeNS）instance-first（host 无
  ns 视图）。两个权威面按 API 族分野，不在同一路径混用。
- **NamedNodeMap.prototype 方法 identity 的低成本方案**：原型方法 = 最后创建的
  map 实例闭包（per-element 缓存保证 el.attributes 稳定 identity 后，「最后创建」
  即「当前唯一」；named property 冲突由 get trap 分支序保证方法优先）。
