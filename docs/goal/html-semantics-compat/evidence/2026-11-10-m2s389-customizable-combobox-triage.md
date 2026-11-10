# M2 切片 389（s389）——customizable-combobox 域分诊：`appearance:base` 特征门簇（簇 D）12F 全红无流内施工入口，立项挂账（零源码改动，域新入账）

日期：2026-11-10（承 s388 下一步候选：customizable-combobox 域分诊——控制文档在册零记录新目录）
域：html/semantics/forms/customizable-combobox（8 html：7 testharness 全红 12F + 1 reftest 对不入 testharness 账）
性质：**分诊轮——s386 方法论复用（标记扫描→门检测→逐案读源），零源码改动**

## 目录画像与门检测

8 个 html：7 个 testharness（6 .tentative + 1 stable `option-attributes`）+ 1 reftest 对（appearance + -ref）。源码标记扫描：**base-select/::picker/selectedcontent 全零命中**，特征门为 **`appearance: base`**（7/7 testharness 文件 style 块 `input, datalist { appearance: base }`）——customizable combobox 特性族（open-ui combobox explainer，jarhar@chromium.org 与 customizable select 同作者同系；crbug.com/453705243 在案 TODO）。

ZW 门缺席实证：`parse_appearance`（css-parser parse_extended_visual.rs:371）接受 none/auto/button/checkbox/listbox/menulist/menulist-button/meter/progress-bar/push-button/radio/searchfield/slider-horizontal/square-button/textarea/textfield——**无 `base`（亦无 base-select/base-combobox）**→声明非法被弃→datalist 恒 classic UA 形态→popover 永不物化。

## 本轮 solo 实测红形（FILTER=customizable-combobox，12F 与 s387 sweep 同形）

| 红形 | 文件×F | 首环断言 |
|---|---|---|
| datalist popover 物化面 | datalist-popover 1 + option-attributes 1 + pointer-behavior 2 + keyboard-behavior 1 + beforefilter 1 + popover-exception 1 = **6 文件 7F** | `datalist.matches(':popover-open')` focus/click 后 false（popover-exception=beforetoggle 事件 0 次）；全链后继断言（键盘选择/指针激活/input 回填文本/beforefilter 事件）均以 popover 开态为前提 |
| `:filtered` 伪类 + base 模式 option 显示面 | filtered-pseudo 2F | 首环 `gCS(option).display=="none"`——R1679 UA 抑制（appearance:base 生效时真实浏览器 option 恒可见行） |
| `:active-option` 伪类面 | keyboard-behavior 3F | `SyntaxError: ':active-option' is not a valid selector`——选择器解析器无此伪类；解析放行亦不翻绿：抽验 "No active option and removal" 子测方向键移动后仍 `assert_true(matches(':active-option'))`，需 active-option 追踪态机 |

## 全有全无门定谳

簇 D 12F 断言链首环即 `appearance: base` 生效面（popover 物化/option 可见/伪类匹配），**不存在特征门下增量施工入口**——解析面单点放行（appearance 值或伪类）均不翻任何子测（`:active-option` 放行后 matches 恒 false，方向键断言仍红；`appearance:base` 值放行而无 UA 规则/态机，popover-open 恒 false）。五子面=①css-parser appearance 值 `base`（与簇 A 子面②**同面同刀**——parse_appearance 一处加值可双簇共享）+②selector parser `:filtered`/`:active-option` 伪类族+③style-system/UA sheet base 模式规则（option 显示分支——与 s388 勘根 listbox 形态分支同构：R1679/s387 抑制需加模式/形态 carve-out）+④engine datalist popover 物化+open/close 态机+beforetoggle/beforefilter 事件+⑤交互键盘 active-option 追踪/指针激活/type-ahead `:filtered` 过滤语义。

## 归属判决与口径

- **簇 D 立项挂账**，与簇 A（customizable select）同特性包不同元素位（select vs input+datalist）：①面共享一刀；③④⑤元素位独立。上游态同簇 A（open-ui explainer + Chromium tentative 在飞，crbug 453705243 TODO 在案）。
- 域口径新入账：**customizable-combobox 12F 全红=簇 D 立项挂账**（7/7 testharness 文件全红；reftest 对不入本账，预期同红未实测注记）。域 corpus 总红由 s387 口径 149F+2T（the-select-element）+ 本域 12F 构成，无位移（本轮零源码改动）。

## 下一步

- **the-input-element 域余红分诊**（s387 sweep 观测 5F：reset.html form 元素指针语义 1F / date-readonly-keyboard 1F / radio-focused-and-scrolled-into-view 1F / checkable-active-onblur-with-click 2F——均非特征门形，逐案勘根定性流内可施工性）。
- A2+D 簇立项征询合并呈报维持待用户决策。
