# M6 S4q 完整化 — QuickJS DOMException 错误路径（R66）

**日期**: 2026-08-16
**Commit**: `4024e525`
**前置**: R65（S5q 深化：完整 ctor 执行 + attributeChangedCallback，`7a486499`）
**证据**: [evidence/2026-08-16-r66-quickjs-s4q-domexception.json](../evidence/2026-08-16-r66-quickjs-s4q-domexception.json)

## 背景

M6 S0q–S5q 全部有 land 实现（R57–R65 十二切片），但错误路径仍是 PoC 形态：appendChild/insertBefore/removeChild 失败返 null 吞错、createElement 非法 tag 照建、customElements.define 重复定义静默覆盖。master.md「R65 后下一步首选」清单第 ① 项。

## 实现

1. **`throw_dom_exception(ctx, name, message)`**：构造带 `name`/`message`/`stack` 属性的对象经 `Ctx::throw` 抛出——JS 侧 catch 得 `e.name`/`e.message`，与 DOMException 的可观测面等价。DOMException 全局构造器（instanceof 面）延后：V8 侧 R6 的 identity 三重根因教训（prototype.constructor / 幂等注册 / wrong-global）在案，QuickJS 需要先有构造器基建。
2. **`dom_error_name(&DomError) -> (&'static str, String)`**：镜像 V8 `dom_bindings/node.rs` 的 `dom_error_exception` 映射（HierarchyRequestError / NotFoundError / InvalidStateError）。
3. **错误路径接线**：
   - `append_child_method` / `insert_before_method`：`Some(Err(e))` → throw（此前 null）
   - `remove_child_method`：非子节点 → NotFoundError
   - `native_create_element_entry`：非法 tag → InvalidCharacterError（`is_valid_tag_name` 镜像 V8 R3）
   - `ce_define`：非 callable ctor → TypeError；重复定义 → NotSupportedError（spec `dom-customelementregistry-define`）
4. **签名迁移**：四个方法从 `-> Value<'js>` 改 `-> rquickjs::Result<Value<'js>>`（Err 状态装在 ctx，返回给 rquickjs 才生效）。

## 验证

- PoC 断言三条：`appendChild(self)` → catch `e.name === 'HierarchyRequestError'`；`createElement('<bad>')` → `'InvalidCharacterError'`；重复 `customElements.define` → `'NotSupportedError'`
- engine quickjs **1419** / v8 **2153** 全绿零回归；clippy 双矩阵 `-D warnings` 零警告；fmt 无 diff
- pre-commit-guard PASS

## 过程注记

上一轮 429 限流中断本切片 WIP（429 前方法签名迁移 + helper + 测试已写完）。本轮：stash → rebase over 并行 canvas 流 R56h 五提交 → pop 零冲突 → fmt 一处 line-wrap 修复 → 验证 land。

## M6 剩余（深度补齐清单）

capture/bubble 祖先链虚站（镜像 V8 R40）→ observedAttributes 过滤 + oldValue 写前捕获 → S0q 续 weak/finalizer → S1q 复合对象（attributes/classList/dataset 二级身份缓存）→ Event 构造器 → DOMException 构造器 instanceof 面。
