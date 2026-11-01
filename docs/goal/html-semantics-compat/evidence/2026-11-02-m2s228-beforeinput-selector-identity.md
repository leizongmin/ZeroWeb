# M2 s228 — beforeinput.tentative 16F 勘根定性：testdriver selector 跨族 identity 断链（两层根因，施工计划落账，零代码收口）

日期：2026-11-02。切片号 s228（承 s227）。

## 入场定刀位

s227 下一步候选首选「forms misc 残面 16F 勘根」。s228 新拉 forms 基线
（4492P/225F/10T）定位：16F = `forms/beforeinput.tentative.html` 恰为控制
文档挂账承袭的「beforeinput.tentative」，且是最大非门禁单文件族。

案面：16 个表单控件（7 type × 2 input 双胞胎 + textarea 双胞胎）经
`container.innerHTML` 构建，promise_test 逐个 `test_driver.send_keys` 后断言
事件序 `["beforeinput","input"]`。16 案全 Fail，错误同一：
`Unhandled rejection: "Error: testdriver target has no stable selector"`——
失败在任何 beforeinput 语义执行之前，属 testdriver 驱动通道。

## 勘根（探针六轮 + 一次施工尝试）

- **p1**：复现 reject；innerHTML 产物 `__zwHandle`/`__zwSelector` 双
  undefined；`qsa(input)=2`、`qsa-nth=0`、`qsa-attr=0`、`:scope > input`=0。
- **p2 反转**：探针 dump 全在 eval 期——innerHTML 变更 pending 未 apply，
  raw `__zw_query_all` 读老快照全空；`qsa(input)=2` 是 shim qSA 的 R161/R3902b
  pending 归并假象。**eval 期与 dequeue 期两个时态必须分开观测。**
- **p3 apply 后矩阵**：一切查询形态通——`#container > input:nth-of-type(1)`
  命中 1、`:scope > input`=2、`input[type="text"]`=2、宿主唯一选择器
  `#container > input:nth-child(N)`。zero_dom 引擎对 nth-of-type/attr/
  child 组合全支持，宿主快照含属性。**查询引擎无缺。**
- **p4 dequeue 期轨迹（层1 定谳）**：包 `__zw_td_selector` +
  `document.querySelectorAll` 录 dequeue 窗——每轮
  `qsa(input)=2 → qsa(input[type="text"])=2 → qsa(#container > input:nth-of-type(3))=0
  → td_sel=null` 重试 18 轮后 reject。selectorFor（testharness.rs:12063）nth
  臂 `sibs[si] === element` **跨 wrapper 族恒败**：qSA 返回宿主 handle proxy
  族，测试捕获的是 eval 期 parse view 族（多族稳定包装，learning
  2026-10-14 实证「跨族 === 恒败」）→ idx 越过 sibs.length 产
  `:nth-of-type(N+1)` 零命中 → null。
- **R5386 施工尝试（已回退）**：nth 臂加身份无关回退计数（从元素自身
  previousElementSibling 数同型前驱 + 越界守卫；仅 `===` 漏配臂触发，改动
  面单调）。选择器解析成功（`nth-of-type(1/2)` 正确产出、keydown 派发到达），
  **揭出层 2**：
  - inline attr 双胞胎（#1）：await 走过（inline 属性是节点数据，派发触达），
    但 `events` Map 按 `event.target`（宿主 wrapper）键 ≠ 测试捕获的 parse
    view → `events.has(element)` false；
  - 裸双胞胎（#2）：JS 赋值 handler 挂在 parse view 上，派发解析到宿主
    wrapper → 永不触发 → `afterOnInput` 永挂 → pending:15 文件级 Timeout。
  - 失败形态 16F（快败）→ 1F + 1T（挂起）：**零通过 + 运行时回归**，不落地，
    回退后复验 16F/0T 基线还原。
- **p5 层2/3 对照**：id'd 动态元素与 tag 唯一 textarea（`getElementById`
  捕获、selectorFor id/tag 臂直接命中）send_keys **全链通**——
  `beforeinput@SAME,input@SAME` + value 落字。identity key 一致时 listener
  注册、dispatch target、`===` 全自洽（dispatch sel `textarea` ≠ 注册 sel
  `#dyn2` 仍 `@SAME`——shim 有跨 sel-key 的节点级统一）。
  **唯一断点 = innerHTML 产物没有稳定 identity。**
- **p6**：对捕获 parse view 直接 `dispatchEvent(keydown)`——`prevented=true`
  且零 handler 触发、value 不落。shim dispatchEvent 不跑 keydown 默认动作链
  （beforeinput/input/value 在宿主驱动键路径 part01:8371）→「runner 侧 stub
  直接派发」路线出局。

## 根因定性（两层叠加）

1. **层1（selectorFor）**：nth 臂 `===` 身份扫描跨 wrapper 族恒败。身份无关
   walk 修法已写码验证（产出正确选择器），随层 2 定谳一并回退。
2. **层2（identity 架构，主根因）**：innerHTML setter 的 t8r 物化
   （`_wcRebuildAsHandle`）仅 handle 容器生效（part04 `if (handle && …)`）；
   **sel 容器**（本例 `#container`，页面静态元素常态）产物只留 `_zwMEl`
   parse view——无 handle、无稳定 identity。`_zwQueryWrapIdentity`
   （part05:1845）只认宿主 sel→handle 表；monolithic `replace_inner_html`
   变更由宿主自解析自建 handle，JS 侧 handle 无从被收编（对照：createElement
   AppendChild 变更**携带 JS handle**被 R100 收编 → 查询反查回同一 proxy →
   全链统一）。beforeinput 面的 event.target 断链与 JS listener 丢失均由此。

## 施工裁定

完整修 = innerHTML 管线改逐子变更 + JS handle 收编（createElement 收编先例
R100）——架构级、最热路径、回归面覆盖全部 innerHTML 使用。本轮不落地；
R5386 单独落地是挂起回归，已回退。**零 shim 零 Rust 净变更收口**（同 s227
勘根定性轮先例）。

## 验证

- 回退后 beforeinput solo：16 Fail / 0 Timeout，与基线快败形态一致
  （s228-beforeinput-revert-confirm.log 对 s228-forms-baseline.log）。
- 探针验后删（zz-s228-probe.html 六版）；零语料残留。
- `git status` 干净——R5386 已 `git checkout` 还原 testharness.rs。

## 方法论教训

- 单文件全 Fail 且 unhandled rejection 同错误 = 驱动通道问题非语义问题，
  勘根先查 testdriver 链（selector 解析 → focus → 派发）再进事件语义。
- 探针必须分 eval 期（pending 未 apply）与 dequeue 期（apply 后）两时态
  观测——p2 的 raw 直调 vs shim qSA pending 归并假象曾致方向反转一轮。
- 「查询形态全通」≠「驱动链通」：wrapper identity 是 selector 解析之外的
  第三维（p3 全绿与 p4 定谳同日并存）。
- 修复验证看端到端失败形态位移：16 快败 → 挂起 Timeout 是回归不是进展，
  半修不可落地。

## 承袭挂账（更新）

- **beforeinput.tentative 16F**：定性完毕，转新候选「innerHTML sel 容器产物
  identity 架构」——三层施工计划：① setter sel 容器臂物化（t8r-for-sel）；
  ② innerHTML 逐子变更替换 replace_inner_html 使宿主收编 JS handle（R100
  收编先例）；③ R5386 selectorFor walk 重挂（层 1 修复须 identity 落位后
  才能端到端）。涉最热路径，立项时按渲染门禁 + 全量 A/B 预算独立切片。
- forms misc 余下散面（email-set-value 1F、input-type-change-value 2F、
  date-readonly-keyboard 1F、radio-keyboard-navigation-order 1F 等）：
  勘根时先对照本账本——若同用 innerHTML 动态元素 + send_keys/click 驱动，
  可能同根，勿重复分型。
- 其余承袭不动：not-broken 2F（renderer 流车道归因）；lazy-loading 族 ~24F；
  img naturalWidth SVG viewBox 族 22F（③A）；用户门禁三项（GB-20261031
  customizable select 167F / form-nameditem override builtins / form owner
  table 缓行族）；error.html 破图 alt 7F；sizes-auto Timeout 68 案；
  svg-img-density-vs-zoom 7F；part06 `_zwFieldsetDisableFocusReset`（s215）；
  part03 `_zwMElCheckValidity` FORM 短路（s210）；onblur flake 族（s209~）；
  非 canvas 面一般渲染门 FIXME(rendering)（s218）；flat tree slot
  FIXME(flat-tree)（s218）；modern API img 树变更面余项（s221）；TreeMutated
  IMG 移除臂余项（s221）；mimesniff 裸空白 value 理论缺口（s222）；
  image-alternate-no-store known limitation（s225）。
