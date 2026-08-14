# M4 切片 R44 — NamedNodeMap supported property names（ownKeys + named getter）

**日期**: 2026-08-14
**里程碑**: M4（WPT dom 上游基线 + 按聚类驱动修复）/ DC-3
**证据**: [../evidence/2026-08-14-r44-namednodemap-own-enumeration.json](../evidence/2026-08-14-r44-namednodemap-own-enumeration.json)

## 切片内容

WPT `namednodemap-supported-property-names`（0P/3F）驱动：`Object.getOwnPropertyNames(el.attributes)` 期望 `[indices..., names...]`，当前返 `[]`——`_attributesProxy` 是 `Proxy({}, …)` 无 `ownKeys` trap。

### 修复（part03 `_attributesProxy` handler）

1. **ownKeys trap**：返 `[数值索引 "0","1",…, 属性名 id/class/…]` 文档序（spec `dom-namednodemap-supported-property-names`）
2. **getOwnPropertyDescriptor trap**：indexed/named 返 enumerable data-property descriptor、length 非枚举——Proxy invariant 要求 ownKeys 列出键均有 descriptor
3. **get trap named 分支**：`attrs.<name>` 命中当前属性名时返 Attr 节点（与枚举一致；解锁 dom/nodes 的 `el.attributes.<name>` 访问模式，nodes +4）

### 未修（诊断归档）

- **querySelector-mixed-case**：Test 1 `[viewBox]` 已过、Test 2 `[viewbox]` 得 0。根因**不在** selector 匹配（zero-dom `has_attribute` 经 `attr_name_effective` 已对 HTML ns 属性名小写、SVG 精确——语义正确）——用例建 **detached 树**（createElement/appendChild 不挂载）后在 root 上 querySelectorAll，detached root 无 selector、元素作用域 host 查询返空。与 dom/traversal detached 遍历同根因（M1 L2 handle 树）
- **M8 canvas path-objects 继续延后**：canvas 流当日 22:15 仍在 commit（part05.js 热碰撞面）

## 结果

| 项 | 前 | 后 |
|----|-----|-----|
| namednodemap-supported-property-names | 0P/3F | **3P/0F（100%）** |
| dom/collections polyfill | 21P（43.75%）| **24P/24F = 50.00%** |
| dom/collections native | 21P | **24P（对等差 0pp）** |
| dom/nodes polyfill | 2503P | **2507P（+4，named getter 解锁）** |

零回归：events 189P / traversal 9P / ranges 39P。

## 验证门禁

- 单测 `test_namednodemap_own_enumeration_r44`（4 断言组）
- engine v8 2125 / quickjs 1415 全绿；quickjs 矩阵 14 crate 全绿
- clippy 双矩阵零警告，fmt 无 diff
