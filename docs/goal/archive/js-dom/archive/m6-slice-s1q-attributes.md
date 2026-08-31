# M6 S1q 复合对象 — attributes NamedNodeMap 面（R69）

**日期**: 2026-08-16
**Commit**: `18843344`
**前置**: R68（observedAttributes + oldValue，`d6dc00d5`）
**证据**: [evidence/2026-08-16-r69-quickjs-s1q-attributes.json](../evidence/2026-08-16-r69-quickjs-s1q-attributes.json)

## 背景

S1q 剩余项「复合对象（attributes/classList/dataset 二级身份缓存）」首片。`el.attributes` 是框架/库（含 Angular 模板编译、lit property/attribute 桥）的高频面。

## 实现

1. **二级身份缓存** `ATTR_MAP_OBJECTS`（owner ffi → Persistent）：`el.attributes === el.attributes`（spec identity），同 NODE_OBJECTS 模式；`reset_quickjs_state` 清理。
2. **方法面**：
   - `length` getter：`attribute_names` live 计数（快照对象、值即时读）
   - `item(i)`：越界/非数字 → null
   - `getNamedItem(name)`：miss → null
   - `setNamedItem(attr)`：从入参对象读 `name`/`value`（兼容 plain 对象与 Attr 形态——镜像 V8 `read_str_prop`），写 owner + 派发 attributeChangedCallback（old 写前捕获）
   - `removeNamedItem(name)`：返被移除 `{name, value}` 条目；miss → null（spec NotFoundError 对齐延 DOMException 构造器域）；派发 new=null
3. **条目形态**：`{name, value}` plain object（Attr 节点 instanceof 面延后——与 V8 侧 _zwMakeAttr 同域问题）。

## 验证

- PoC 断言五组：身份缓存 / 新建元素零属性 / set+读回闭环 / miss 语义 / remove 返条目+再移除 null+元素 getAttribute live 跟随
- engine quickjs **1419** / v8 **2153** 全绿零回归；clippy 双矩阵零警告；fmt 无 diff
- pre-commit-guard PASS

## 过程注记

断言初版用 `#main` 元素——前序测试脚本已累积 6 个属性，`item(0)` 期望 `data-a` 实得 `id`。改用 `__zw_native_create_element('section')` detached 新建元素隔离属性序（断言与执行历史解耦——长 PoC 测试的通用实践）。

## S1q 复合对象剩余

classList（DOMTokenList 面：add/remove/toggle/contains/replace/value + token 校验抛错——R66 DOMException 基建可复用）→ dataset（DOMStringMap：data-* 反射）。
