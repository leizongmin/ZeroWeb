# M2-S192 input.list IDL getter——6F 翻绿 + hidden/text 同根红利 3F 共 9F，shim 1 臂零 rust

## 入场定刀位

s191 收口后全树失败集中度重勘定：customizable/filterable select 簇
177F 仍居首（GB-20261031 拍板在途不动）；非门禁最大簇 =
the-input-element/input-list.html 6F（`.list` IDL 解析面）。

## 勘根

6F 同根：`input.list` IDL getter 整体缺位——`list` 不在
`_REFLECTED_STRING_FLAT` 平面反射表、无任何 IDL 臂，属性读落 expando
恒 undefined（语料 `assert_equals(input.list, datalistEl)` 元素身份
断言与 `assert_equals(input.list, null)` 全 Fail）。spec HTML
§4.10.5.3.8 the list attribute：current suggestions source element =
list 属性值 id 在**文档序**首个命中元素且其为 datalist 才返回；
缺省/空/无命中/首命中非 datalist → null。语料 6 案全形：
同 id datalist vs p 文档序竞争（先 p 后 datalist → null；先 datalist
后 p → 返回 datalist）。

红利面三案同根（hidden.html 1F + text.html 2F）：`list attribute
must return null for hidden/text/search`——此前 `.list` 返 undefined
≠ null 断言 Fail，getter 落地后 null 正确。

## 施工（shim 1 臂零 rust）

part03 INPUT get trap 新增 `prop === 'list'` 臂（valueAsDate 块后）：
list 属性 lw 读（缺省/空 → null）→ `globalThis.document.getElementById`
文档序首命中 → `.tagName` DATALIST 门 → 返回元素 proxy 或 null。
getter 身份复用 getElementById 既有 proxy 缓存（语料 assert_equals
身份比对直接过）。跨 iframe 文档同树约束语料无面，FIXME 不建模。

## 验证

- **定向 input-list.html 6/6 全 Pass**（6F→P 精确）。
- **forms 全树：4409P/291F/12T → 4418P/282F/12T**（ΔP=+9/ΔF=−9 精确
  闭合）；逐文件 diff 恰 3 文件（input-list 6F→P + hidden 1F→P +
  text 2F→P 同根红利），其余 bit-exact。
- **邻接 bit-exact**：interactive-elements 439P/44F；engine lib
  **3642P/1F**（1F = ua_display_li 渲染流在册同键；总数 3641→3642
  为并行流新增用例，非本刀）。
- **shim 拼接序 node --check OK**；cargo fmt --all --check 过
  （clippy 按 .rs 零触碰跳过）。

## 残留与边界

- `.list` IDL setter 未建模（readonly，WebIDL 非严格静默 no-op；现
  落 expando 写、getter 恒走解析臂不可被 expando 污染；语料无面）。
- 跨文档（iframe）list 解析同树约束未建模（语料无面，FIXME）。
- datalist suggestions 本体（option 枚举/autocomplete 交互面）不在
  本刀范围（customizable select 拍板簇邻域）。
- 残余非门禁面：checkable-active-onblur-with-click 6F、fieldset
  disabled-001 5F、radio-focus-navigation 6F（时序域）、email/url/
  tel IDN 12F（分立小簇）。

## 下一步候选

非门禁簇次席勘根序：fieldset disabled-001 5F（disabling 传播语义面）
→ email/url/tel IDN 12F（校验面，疑 idna 管线）→ checkable/radio
focus 时序域（负载敏感面嫌疑，s171 先例）；customizable select 177F
挂起待拍板（GB-20261031 征询在途）。
