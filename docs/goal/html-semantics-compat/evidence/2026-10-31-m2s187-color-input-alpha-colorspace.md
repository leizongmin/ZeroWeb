# M2-S187 color input alpha/colorSpace 反射——IDL 缺位单根 2F 恰靶翻绿，shim 2 零 rust

## 入场定刀位

s186 账本「下一步候选」首位：color-attributes input.alpha/colorSpace 2F
小反射面。

## 勘根

color-attributes.window.js 两案（`createElement('input')` proxy 形）同
单根：`input.alpha` / `input.colorSpace` IDL 双向缺位——复跑确认读返
undefined（`assert_true: expected true got undefined` /
`expected "limited-srgb" but got undefined`）。两名字不在任何反射表面
（`_REFLECTED_STRING_FLAT`/`_reflectedStringAttr`/`_REFLECTED_UINT`/
`_REFLECTED_BOOL` 全查无——零劫持风险），set 落 expando 死槽。

设计取舍：**不入 `_REFLECTED_BOOL` 通用表**——该表全元素生效（div.alpha
会造出非标准假面），且 CE 反射映射 `_zwCeReflectionAttr`（part03:12132）
消费 `_reflectedBoolAttr` 会给 custom elements 的 alpha 属性变更派
attributeChangedCallback（真浏览器无此面）。取 INPUT tag 门专属臂，与
valueAsDate（part03:26505）/ set trap 布尔分支（part05:539）同形。

## 施工（shim 2 零 rust）

- **part03 get trap 双臂**（valueAsDate 臂后、selectionStart 前，同款
  `prop === … && _realTag(sel, handle) === 'INPUT'` 形）：
  - `alpha`：presence 读返 boolean（`__zw_has_attr_handle` / sel 走
    `__zw_has_attr_lw` R136 融合读——同批 setAttribute 后 IDL 读
    latest-wins）。
  - `colorSpace`：enumerated 关键字反射——读属性（lw 融合）ASCII 小写
    归一，`display-p3` 命中返 canonical，否则（含缺失/无效）
    `"limited-srgb"`（missing-value default 与 invalid-value default 同
    值）。
- **part05 set trap 双分支**（`_reflectedBoolAttr` 分支前）：
  - `alpha`（INPUT 门）：truthy → presence 设空 / falsy → 真移除
    （R3039 同款三态：handle 设/移 + `_zwAttrInstanceRemoveKey` 实例
    层同步 + sel 移除）。
  - `colorSpace`（INPUT 门）：setter verbatim 写属性
    （`String(value)`——WebIDL DOMString 转换 null → "null"，
    DISPLAY-P3 存原串；canonical 归一在读侧，enumerated 反射标准形）。

spec：html.spec.whatwg.org/multipage/input.html#color-state-(type=color)
（`attr-input-alpha`/`dom-input-alpha`/`attr-input-colorspace`/
`dom-input-colorspace`——反射不按 type 门（接口面无条件），实现同口径
（INPUT tag 门、无 type=color 门）。

## 验证

- **定向 color-attributes.window.js：2/2 全 Pass**（2F→P 精确）。
- **forms 全树：4363P/337F/12T → 4365P/335F/12T**（ΔP=+2/ΔF=−2 精确
  闭合）；文件粒度状态表 diff 仅 color-attributes.window.js 翻面
 （2F→2P），其余 400+ 文件零变化。
- **interactive-elements 483 键零位移**（439P/44F bit-exact）。
- **engine lib 3641P/1F**（1F = ua_display_li 渲染流在册同键）。
- **shim node --check SYNTAX_OK**；cargo fmt --all --check 过 + guarded
  workspace clippy 零告警。无探针页（勘根经定向复跑 + 表查即闭合）。

## 残留与边界

- `alpha` getter 无 type=color 门（spec 接口面无条件反射，Chrome 同形）
  ——非 color input 的 `.alpha` 读属性存在性，标准形。
- mutTree wrapper（innerHTML 解析元素）无 alpha/colorSpace accessor
  ——语料 proxy 形，无语料面（R154/R5000 模式可按需后补）。
- colorSpace 值集仅两关键字（spec 现行全集）；后续 spec 扩关键字时按
  值表扩展。

## 下一步候选

submission-checks 三兄弟（连接性面）在册；customizable select 177F
挂起待拍板（GB-20261031 飞书征询在途）；fix#19c override builtins 1F
挂待决承袭。
