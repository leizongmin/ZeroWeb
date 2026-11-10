---
date: 2026-11-11
modules: engine, style-system, css-parser
---

# inline style 写面有三条实现臂——值校验/归一类改动必须三处同形

## 问题

WPT `acos-asin-atan-atan2-invalid.html` 63 个「非法值不得落库」子测试全部失败：
`el.style.transform = 'rotate(asin())'` 仍被存储。但值校验（T54 白名单 +
宿主回调裁决）已在 shim `_zwMStyleProxy.writeDecl` 挂好，单测也绿。

## 根因

`el.style.x = v` 在本仓有**三条互不相通的写臂**，只补其一等于没补：

1. **native CSD**（`css_style_declaration.rs` `native_style_named_setter` /
   `native_style_set_property_invoke`）——engine 侧 V8 拦截器实现；
2. **shim `_zwMStyleProxy.writeDecl`**（part03.js ~22.7k 行）——纯 shim 世界
   （无 handle 物化）元素的 JS 代理写臂；
3. **shim sel/handle 代理 `setProp`**（part03.js ~30.2k 行）——engine
   **物化元素的实际写臂**。corpus 页面的 createElement/解析元素全部物化带
   handle，值落到 `__zw_set_style_handle` 宿主回调——corpus 行为几乎全由此臂
   决定，而它最容易漏（不改 attr、`getAttribute('style')` 返回 null，表面看
   不到写痕迹）。

判别「写进了哪条臂」的探针：对目标元素读 `String(d.style.setProperty)`——
native 臂是 `[native code]`，shim 代理臂是 JS 函数源码；再看
`getAttribute('style')` 是否有值（sel/handle 臂写宿主 decl store，attr 恒 null）。

第二个坑叠加：宿主回调返 **JS 字符串**（shim 通用约定，比较须
`=== 'false'`，先例 part01c.js `=== 'false'`）——写成 `=== false`（boolean）
则校验恒放行，静默失效。

第三个坑：用 `apply_property_value`（apply 链）作文法 oracle 会把引擎未实现
的 tentative 函数（progress/calc-mix/random/sibling-index）一并拒掉——误拒
store+serialize 路径本已正常的合法值（css-values serialize 页 -12 回归）。
oracle 须先做「实现面函数集」粗扫：值内出现集外 `fn(` 调用即放行 store 原文。

## 解决

值校验/归一类改动在三条臂同形落点（T54 最终形态）：

- native 两函数：`validate_decl_value` 拒绝 → `return kYes` / `return`；
- shim 两代理：宿主回调裁决，`=== 'false'` 拒绝；
- oracle 收窄：`value_has_unimplemented_function` 粗扫放行 tentative 面。

效果：css-values corpus invalid 族 9 页翻正（acos 族 63/63），
serialize 页 0 回归（values Pass 1,540 → 1,770）。

## 避免方式

改 inline style 写行为前先 `grep -n "setProp\|writeDecl\|native_style_named_setter"`
盘点全部写臂；新钩子三处同形 + 每臂一条 engine 测试（createElement 物化元素
驱动 sel/handle 臂，非物化驱动 writeDecl 臂，二者不可互替）。宿主回调返回值
比较一律字符串形态。
