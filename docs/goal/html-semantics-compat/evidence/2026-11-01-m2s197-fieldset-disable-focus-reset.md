# M2-S197：fieldset disabled 置位焦点内含检查——解析节点焦点域双通道钩子

**日期**：2026-11-01
**入场**：s196 在册首席候选 disabled-003 crash/hang 2F（the-fieldset-element/
disabled-003.html——light-nested subtest 1 F「should not crash」+ deep-nested
subtest 2 F「should not hang」）。
**结果**：**2F 翻绿**（light + deep 双案），forms 全树 keyed diff 恰 2 靶键
F→P + flake 域 3 键（`:active` onblur 家族在册负载敏感域，双向摆动），零真
P2F。

## 勘根

失败语义单根三分支：

1. **disable 不失焦（主根）**：fieldset 置 disabled 后，落其后代可聚焦区
   内的活动元素不执行 unfocusing steps——`document.activeElement` 停留被
   禁控件。spec（HTML §4.10.4 #the-fieldset-element）：disabled 移除后代
   focusable area（首 legend 元素子树豁免），活动元素在被除区内则失焦回
   body。
2. **钩点第一陷阱（part05 早分支拦截）**：`disabled` IDL 写先落 part05:495
   早 boolean 分支（`hidden/checked/disabled/selected`），早于
   `_reflectedBoolAttr` 反射表臂——初版钩子挂后者恒不可达，light 案不动。
3. **TAG 门第二陷阱（`_realTag` 常量 DIV）**：innerHTML 物化的 handle-only
   proxy 宿主 tag 表 miss → `_realTag` 回落恒 DIV（s168 在册缺口）——
   初版 `_realTag(sel, handle) === 'FIELDSET'` 门误拦 deep 案；改 proxy
   `.tagName` 读。
4. **深案真根（焦点态双域 + 键缺失）**：deep 案 input 经 innerHTML 物化为
   解析节点 proxy，其 `.focus()` 走 R114 解析节点焦点路径置
   `_zwMElFocused` 并清 `_activeElKey`（R148 所有权互斥，activeElement
   getter R148 优先域直读）。两重后果：
   - 初版 helper 只读 `_activeElKey`→`_proxyCache` 轨——解析节点焦点态下
     读 null 即返（探针 `autoCalls=0` 另证钩子根本未触，见 5）；
   - 解析节点 proxy 无 `__zwSelector`/`__zwHandle`（探针
     `sel=undefined hdl=undefined` 实测）——共享原语 `_zwUnfocusIfFocused`
     的 `_elKey` 重算守卫必 null bail（探针 `manual=still-inp` 实证）。
5. **钩点第三陷阱（解析节点面写通道缺位）**：解析节点（M-El plain
   object）的 `.disabled=` 走 R154 描述符表（part03 `_zwDescBoolReflected`）
   → `_zwMElSetAttribute`——part05 part04 两钩子（selector-proxy/setAttr
   trap 面）均不在该通道。探针包装计数 `autoCalls=0` 定谳。

## 施工（4 文件，shim only，净 +110/−4）

- `part06.js`：`_zwFieldsetDisableFocusReset` helper——焦点内含走查（从
  焦点元素 parent 链上行 ≤128 跳，命中 disabled FIELDSET 即判豁免：该层
  首个 legend 元素子 === 走查直子则豁免续上行，否则 unfocus 终结；与
  s193 读面同口径）。焦点态双域取焦点元素：`_zwMElFocused` 优先、
  `_activeElKey`→`_proxyCache` 次之（R148 所有权互斥同序）。unfocus 序
  分流：proxy 态复用共享原语 `_zwUnfocusIfFocused`（key 重算守卫 +
  `_navMarkFocusChanged` + blur/focusout 落焦点元素 + `__zw_focus_changed('')`
  无 body focus）；解析节点态（key 无法重算）就地展开同序——清
  `_zwMElFocused` + `_navMarkFocusChanged` + blur（不冒泡）/focusout
  （冒泡）派发 + `__zw_focus_changed('')`，事件参数逐字节对齐共享原语。
  https://html.spec.whatwg.org/multipage/#the-fieldset-element
  https://html.spec.whatwg.org/multipage/#unfocusing-steps
- `part05.js`：早 boolean 分支 `if (value)` 臂（`__zw_set_attr` 写后）
  挂钩——TAG 门走 proxy tagName 读（`_realTag` 陷阱规避），FIELDSET 才
  触发 helper（静态面 IDL 通道）。
- `part04.js`：setAttribute trap 写后挂钩（s150/s155 同款 TAG 门 + 写后
  触发；静态面 setAttribute 通道）。
- `part03.js`：`_zwMElSetAttribute` 写后共享尾（`_fsTail`）挂钩——
  `disabled` 属性写（值无关，存在性语义）+ localName FIELDSET 门 →
  helper。解析节点面 IDL（R154 描述符 setter → setAttribute）与直接
  setAttribute 双通道同经此口。

disabled 移除不回焦（spec 无此步，removeAttribute 不挂钩）。

## 验证

- shim 全序拼接 `node --check` OK（part01–06 + part01c，三处施工后复验）。
- 探针语料（`zw-fs-probe.html`，验后删）四轮钉面：parent 链走查
  disabled 置位可见（`FIELDSET:true`）、手动 helper 解析节点焦点域
  unfocus 生效（`manual=moved`）、包装计数定谳解析节点面 autoCalls=0
  （钩点缺位）、`_zwMElFocused === inp` + `_activeElKey === undefined`
  双域所有权实测。
- **stash 隔离基线 vs WIP 对照（s195/s196 同法）**：
  - forms 全树：基线 **4448P/253F/11T**（复现 s196 HEAD 数字逐位一致）
    → WIP **4451P/250F/11T**（ΔP=+3/ΔF=−3）。逐测试键 diff（(key,state)
    对 sort -u 去重）**F→P 4 键**：disabled-003 双案（靶键）+
    button/active-onblur + checkable radio onblur（flake 域，见下）；
    **P2F 1 键**：checkable checkbox onblur（flake 域）。
  - flake 域定谳：3 位移键全落 `:active` onblur 家族（s196 在册
    「runner 单命令链 probe-tick 竞速负载敏感」域）；checkbox onblur 键
    同树三轮 F→F→P 摆动实测（`s197-checkable-flake-probe.log`）——
    双向摆动非本刀位移（本刀钩子仅在 fieldset disabled 写后触发，与
    Space/Tab :active 清除零机制关联）。
  - the-fieldset-element 目录级：**21P/1F**——唯一 F
    `HTMLFieldSetElement.html :: type attribute`（A/B 两轮同键 F，既有
    缺口非本刀位移）。
  - interactive-elements：基线与 WIP 均 **439P/44F/0T bit-exact**
    （零键位移）。
  - lib：zero-engine **3642P/1F**（在册 s166 失败同键）+ zero-dom
    **886P/0F**（880 + 6 小 binary），stash 基线逐位一致。
- fmt 过（Rust 面零改动，JS 不受 cargo fmt 影响）+ guarded-clippy
  workspace `-D warnings` 过。

## 残留/挂账

- **HTMLFieldSetElement `type` IDL**：返值非 'fieldset'（1F 既有，独立
  小缺口可作低优先候选）。
- **`:active` onblur 家族 flake 域**：s196 在册，runner 单命令链竞速
  负载敏感（本轮 A/B 双向摆动 +1 键样本）。
- 解析节点焦点域 unfocus 的 focusin/focusout 补充事件面：helper 派
  blur/focusout 对齐 `_zwUnfocusIfFocused` 既有口径（无 body focus、无
  focusin/focusin——共享原语同限），语料无形态不扩面。
- fieldset disabled 惰性重评（legend 免除区随 DOM 变更重算）未建模——
  s193 读面同限，语料无形态。

## 下一步候选

customizable select 177F 挂起待拍板（GB-20261031 征询在途）→
HTMLFieldSetElement `type` 1F 小缺口 → s196 挂账序（父 label 零矩形
layout、inert-svg-hittest 第 2 子测等）。

## 日志

`s197-forms-stash-baseline.log` / `s197-forms-wip.log` /
`s197-fieldset-dir-wip.log` / `s197-interactive-stash-baseline.log` /
`s197-interactive-wip.log` / `s197-lib-stash-baseline.log` /
`s197-lib-wip.log` / `s197-lib-dom-stash-baseline.log` /
`s197-lib-dom-wip.log` / `s197-clippy.log` +
探针族 `s197-probe*.log` / `s197-disabled003-*.log` /
`s197-checkable-flake-probe.log`（同目录）。
