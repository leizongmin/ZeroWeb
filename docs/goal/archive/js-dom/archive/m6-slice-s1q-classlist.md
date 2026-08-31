# M6 S1q 复合对象 — classList DOMTokenList 面（R70）

**日期**: 2026-08-16
**Commit**: `7e4c63c4`
**前置**: R69（attributes NamedNodeMap，`18843344`）
**证据**: [evidence/2026-08-16-r70-quickjs-s1q-classlist.json](../evidence/2026-08-16-r70-quickjs-s1q-classlist.json)

## 背景

S1q 复合对象第二片。classList 是框架（Vue/React className diff、Bootstrap 选择器切换）最高频面之一。复用 R69 二级身份缓存模式 + R66 throw_dom_exception 校验基建。

## 实现

镜像 V8 `dom_bindings/dom_token_list.rs`：

1. **身份缓存** `CLASS_LIST_OBJECTS`（owner ffi → Persistent）：`el.classList === el.classList`。
2. **token 读写模型**：`dtl_current_tokens`（class 属性 split_whitespace）/ `dtl_write_tokens`（join 单空格写回；空列表 → class="" 非删除属性）+ attributeChangedCallback 派发（old 写前捕获）。
3. **token 校验**（`dtl_validate_token`）：空 → SyntaxError、含 ASCII 空白 → InvalidCharacterError（复用 R66 `throw_dom_exception`）；add/remove **全量校验后统一写**（all-or-nothing）。
4. **方法面**：length / item(i) / contains / add / remove / toggle（force + 切换双模式）/ replace（原位 true·oldT 不在 false·同名返 contains·newT 已在 dedupe）/ value getter+setter / toString。
5. **contains 例外**：空/空白 token 返 false **不抛**（spec contains 区别于 add/remove/toggle/replace 的 check 抛）。

## 验证

- PoC 断言六组（自建元素隔离）：身份缓存、add 多参幂等+读写闭环、toggle 双模式（含全移除后 value 空串）、replace 四路+className 反射一致、校验 all-or-nothing（非法 add 后 value 保持）、contains 例外。
- engine quickjs **1419** / v8 **2153** 全绿零回归；clippy 双矩阵零警告（4 处 `manual_contains` 修复）；fmt 无 diff
- pre-commit-guard PASS

## 过程注记

断言期望值三轮修正：toggle 序列末尾 value（全移除后是空串非 'b'）、replace 段数（join 元素计数）、分隔符（`/` vs `,` 混写）——**期望值先在脑内逐段模拟再写**，join 计数最易错。

## S1q 复合对象剩余

dataset（DOMStringMap：data-* 驼峰↔连字符反射 + identity 缓存）→ 完成后 S1q 收口，转 S0q 续 weak/finalizer 或 Event 构造器。
