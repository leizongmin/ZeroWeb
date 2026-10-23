# M2 切片 45 — summary-less details 内非激活后代点击翻转——nearest activation 上溯收口（R5248）（2026-10-23）

## 范围

s37 已知边界「点击 summary-less details 内非激活后代不翻转（同根邻近
缺口，真浏览器会翻转，记待办）」。2026-10-23 重新定性：spec 无 summary
翻转是 UA affordance 非 MUST，corpus 无观测面，优先级维持挂账序——本
切片按挂账序开工，性质同 s42/s43：spec 正确性收口 + 契约钉住，不图
GAIN。

## 探针实证（修前行为分裂）

七案探针（临时 probe_sumless.rs，已删）：静态后代 p/span `.click()` 与
dispatch `new MouseEvent` 均不翻转；details 本体 `.click()` 与 summary
站正常翻转。真浏览器七案全翻。

## 根因（通道分裂）

details 激活面存在两条通道，各自断半边：

1. `.click()` 合成通道——part04 click 默认动作段只处理 receiver 自身三
   站（LABEL 转发/SUMMARY/DETAILS），receiver 是非激活后代（p/span）时
   无 nearest 上溯，默认动作静默丢失；
2. JS dispatch 通道——part03 R112 post act:details 消费块门为
   `event._zwUaDispatch === true`（UA 坐标/宿主计划通道专用，R4400），
   JS `new MouseEvent`（无 UA 印记、无 synthetic 标）被挡。

## 修复（R5248 双刀）

- **part04 click 默认动作段 nearest 上溯分支**：receiver 非
  LABEL/SUMMARY/DETAILS 且 `_zwFindClickActivation` 命中 act:details 时
  `_zwDetailsActivateR4400` 翻转（spec the-details-element activation
  behaviour + concept-event-dispatch legacy-pre-activation「nearest
  ancestor (or self) with activation behavior」——DETAILS 无 summary 子
  时其 activation behavior 覆盖全部非交互后代）。receiver 自身三站让位
  （DETAILS 自身再跑 nearest 会双翻）；nearest 命中交互后代
  （act:button/act:a/act:summary/checkbox）时 type 非 act:details 自然
  豁免。分支在 `notPrevented` 门内（preventDefault 取消语义继承）。
- **part03 R112 post act:details 门放宽**：
  `(_zwUaDispatch === true || _zwSyntheticClick !== true)`——JS dispatch
  通道进同款翻转；`.click()` 合成（`_zwSyntheticClick` 标）保持不进
  （part04 内联段承接），单翻互斥。preventDefault/connected 门由消费块
  入口守卫（`_zwDispatching === 0` / `!(cancelable && _defaultPrevented)`
  / `_zwClickActivationConnected`）继承处理。
- hasSum 门槛让位不变：`_zwDetailsActivateR4400` 对有 summary 的
  details no-op（summary 站优先，R4009 面不回归）。

## 验证

- 探针七案修后全翻且单翻互斥成立（有 summary 时 summary 优先让位）。
- interactive-elements 全域 98 files 482 subtest **420P/60F/2T 对 s44
  逐位 REG 0 / GAIN 0**（R4400 既有面无回归；corpus 无观测面无 GAIN，
  符合切片预判）。
- zero-engine lib **3440P/0F**（3438 + 2 新契约 part77）。
- 契约翻转力实证（临时 patch 断言红后还原）：part04 nearest 分支短路
  → 钉 1(a) 红（s37 边界 click() 半边回归）；part03 门收紧回
  `_zwUaDispatch === true` → 钉 1(c) 红（dispatch 半边回归）。

## 契约（part77）

- `summaryless_details_descendant_activation_r5248`：(a) 静态后代
  p.click() nearest 上溯翻转、(b) 二次点击翻回（toggle + 合成标互斥）、
  (c) 后代 dispatch MouseEvent 翻转（放宽门）、(d) preventDefault 取消
  后入口守卫拦截、(e) details 本体 .click() R4400 既有分支（nearest
  让位，双翻则红）、(f) 动态 createElement details + 后代点击翻转
  （R91 记账链 nearest 可达）。
- `summary_and_interactive_descendant_exemption_r5248`：(a) 有 summary
  时非 summary 后代点击不翻转（hasSum 让位）、(b) summary.click() 翻转
  保持（R4009 面）、(c) 交互后代 button 点击 nearest 命中 act:button
  豁免、details 不翻转（BUTTON 判定 ty 为空/submit/reset）。

## 已知边界

- corpus 无观测面：本轮无全域 GAIN，正确性由契约钉住。
- detached details 后代点击不翻转（R112 消费块 connected 门）——spec
  activation behavior 无 connected 条件，但 corpus 无观测面，维持现状
  不扩面。

## 门禁

zero-engine lib 3440P/0F、clippy --workspace -D warnings（test-guard 包
裹）/fmt clean；零新增依赖。数据
[2026-10-23-m2s45-summaryless-details-descendant-activation.json](2026-10-23-m2s45-summaryless-details-descendant-activation.json)。
