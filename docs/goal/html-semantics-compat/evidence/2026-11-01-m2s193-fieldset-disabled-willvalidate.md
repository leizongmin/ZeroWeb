# M2-S193 disabled fieldset 祖先 willValidate 传导——disabled-001 5F 翻绿，shim 1 臂零 rust

## 入场定刀位

s192 在册次席候选：the-fieldset-element/disabled-001.html 5F（fieldset
disabled 语义面）。

## 勘根

探针钉态（s193probe.html，验后删）：`fs.disabled` IDL 已在（通用臂可
读，hasOwn=false 走 trap 链）——缺口单一 = **willValidate 的 disabled
fieldset 祖先传导缺位**（探针 name=true 应 false；legend 子正确 true
但系无传导的「未排除」假阳）。spec §4.10.5.2.2 barred 条款「descendant
of a fieldset element whose disabled attribute is specified」+ 豁免
「not a descendant of that fieldset element's first legend element
child, if any」。语料 5 案钉态全形：首 legend 子豁免（含嵌 fieldset
形态）；legend 包 p 隔层不豁免；嵌 fieldset 隔层不豁免；第 2+ legend
不豁免；fieldset 本身 disabled 后其内控件 barred（豁免不穿透自身
disabled）；豁免后继续上行（更外层 disabled fieldset 仍可 barred）。
part04:11643 在册「fieldset disabled 传播 FIXME」同面记账。

## 施工（shim 1 臂零 rust）

part04 `willValidate` 臂祖先走查重构（原 datalist 走查扩展）：路径
节点 `_wvNode` + 父链 `_wp` 同步行进——DATALIST 命中原样 false；
FIELDSET 且 `_wp.disabled` → 枚举 `_wp.children` 取首个 LEGEND 直子，
上行路径直子 `_wvNode === _fsLeg` 才豁免（proxy 身份复用既有缓存），
否则 false；豁免继续上行。64 层 guard 保持。

## 验证

- **定向 the-fieldset-element 目录**：disabled-001 **5/5 全 Pass**；
  邻接 disabled-003 2F / HTMLFieldSetElement 1F+3P 与 s192 基线逐位
  一致（既有 F 非本刀位移）；fieldset-willvalidate/validation/
  checkvalidity/setcustomvalidity 单案面全 P 保持。
- **forms 全树：4418P/282F/12T → 4423P/277F/12T**（ΔP=+5/ΔF=−5 精确
  闭合，探针 2F 验后删）；逐文件 diff 恰 disabled-001。
- **邻接 bit-exact**：interactive-elements 439P/44F；engine lib
  **3642P/1F**（在册同键）。
- **shim 拼接序 node --check OK**；cargo fmt --all --check 过
  （clippy 按 .rs 零触碰跳过）。

## 残留与边界

- willValidate 其余三实现位（part03:19855 mutTree wrapper defineProperty
  面、part03 R351 target 预置位）未同步 fieldset 传导——runner 语料
  面全在 proxy get trap 臂；mutTree wrapper 面语料无案，FIXME 记账。
- fieldset disabled 对 checkValidity()/form 提交按钮收集（part04:11643
  form serialization 面 FIXME）、激活行为（click 激活门 R313 disabled
  分支）的传导各有在册记账位，语料再钉再补。
- disabled-003「light/deep-nested should not crash/hang」2F 既有（基线
  同在，非本刀位移，待独立勘根）。

## 下一步候选

email/url/tel IDN 校验簇 12F（email-value-idn 4F + email 3F +
telephone 3F + url 2F，疑 idna/validationMessage 管线）；focus 时序域
（checkable-active-onblur 6F + radio-focus-navigation 6F，负载敏感
嫌疑 s171 先例）；customizable select 177F 挂起待拍板（GB-20261031
征询在途）。
