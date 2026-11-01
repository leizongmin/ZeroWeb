# M2-S202：legend/option form IDL 反射——HTMLLegendElement/legend-form/option-form 收口

- **日期**：2026-11-01
- **目标面**：`html/semantics/forms/the-legend-element/`（HTMLLegendElement.html 3F +
  legend-form.html 2F，s181 分型在册「legend.form 5F 单根（小）」）+
  `the-option-element/option-form.html` 1F（同 IDL 域红利）
- **状态**：收口——8F→P 全数落账（forms 全树 keyed diff 净 F→P=8 P→F=0 键漂移 0）

## 勘根

入场：s201 forms-wip2 现势 F 集重聚类（250F，除 customizable-select 特性门面
177F 与 onblur flake 域后 63F）——s181 分型表「legend.form 5F 单根（小）」
系后备序从未刀项（s182 起各轮按色清/编码/集合层/click 域序推进，此刀位悬置）。

面定谳（语料 expected/got 直读）：

- `lgd1.form`（form 直父，无 fieldset）→ got undefined 应 null——IDL 属性缺位；
- `lgd2.form`（fieldset 内）→ 应同 `fs.form`/`fm`——父 fieldset form owner 关联缺位；
- `assert_readonly(lgd1, 'form')` → `property "form" not found`——两断言链：
  ①has 白名单缺列（`'form' in legend` 恒 false，part05 R5249/R5266/media M2a
  同款根因）；②gOPD 链无描述符（assert_readonly 走 getOwnPropertyDescriptor
  上行走查）；
- `o1.form`（detached option）→ got undefined 应 null；select 内 → 应
  `select.form`（option-form.html 双臂）。

机制面：live 元素 proxy 三 trap 分工——get trap `prop === 'form'` 臂
（part03:28007 R2841/R4001/R4005/R4007 既有 form-associated 名单 +
LABEL 特臂）无 LEGEND/OPTION 分支；has 白名单（part05:1265）与 gOPD
（part05:1435）均无 form 列。镜像面 `_zwFormGet`（detached wrapper，
formAssociated 查表）对 legend/option 已返 null（ detachment 臂现势正确，
fieldset/select 祖先臂语料无驱动不扩）。

## 施工

（shim 2 文件，净 +69/−0；rust/语料零改动）

- **part03**：`_zwLegendOptionForm(lsel, lhandle, lpx)` 值算子（`_makeProxy`
  前声明，闭包共享域）——LEGEND 返**父 fieldset** 的 form IDL 值（父非
  fieldset → null；legend 非 form-associated，自身 form content attribute
  不参与）；OPTION 返**最近 select 祖先**的 form IDL 值（无 → null）。
  父/祖先上行走 JS 视图 parentNode（R4005 同款三族统一）；命中元素的
  `.form` 读回 get trap 同名臂（FIELDSET/SELECT 属名单）。get trap
  `prop === 'form'` 臂 LABEL 特臂后插 LEGEND/OPTION 分支。
- **part05**：has 白名单 videoWidth 块后插 tag 门臂（LEGEND/OPTION →
  `'form' in el` true，media M2a 同形）；gOPD 臂 `return undefined` 前插
  get-only accessor 描述符（`get` 复用值算子与 get trap 同源、`set:
  undefined`——assert_readonly readonly 面）。
- spec：legend.form（the-legend-element）/ option.form（#dom-option-form）；
  #11708 tentative 阻断面不在本刀（见挂账）。

## 验证

- 拼接 shim node --check SYNTAX_OK（14 part 按 js_dom_bridge.rs 拼接序）。
- 靶面：the-legend-element 5/5 + option-form.html 1/1 + gated 域
  option-form-ancestor-select F→P 1（bonus）。
- stash A/B keyed diff：forms 4711 键恒定 4452P/248F/11T →
  **4460P/240F/11T，F→P=8（6 靶键 + gated bonus + onblur flake 域
  checkable-active-onblur 顺向摆动）P→F=0 键漂移 0**；interactive 483 键
  对 s201 post-rebase 基线 bit-exact（440P/43F/0T，零位移）。
- zero-engine 全量 3664P+1F（1F = ua_display_li 渲染流域在册键，s201
  stash 基线同败，非本刀）；fmt --check 过；guarded-clippy 过。

## 挂账

- `option-form-ancestor-select.html` 第 2 子测（option 嵌 datalist/optgroup/
  hr 内 option.form 应 null——whatwg/html#11708 tentative 阻断语义）保持 F：
  阻断清单与常规 `select > optgroup > option` 主线结构的语义边界需提案原文
  定谳，属 customizable-select 特性面（GB-20261031 挂起待拍板），不越界。
- 镜像面 `_zwFormGet`（detached wrapper）LEGEND 父 fieldset / OPTION select
  祖先臂未扩（语料无驱动；detachment null 臂现势正确）。
- onblur flake 域持续采样（本轮 +1 顺向样本）。

## 日志

`s202-{targeted-legend-option,forms-stash-baseline,forms-wip,interactive-wip,engine-unittests,clippy}.log`
（evidence/logs/ gitignore，账本仅存档文件名）
