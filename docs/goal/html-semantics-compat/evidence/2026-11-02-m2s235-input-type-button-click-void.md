# M2 s235 — input-type-button click() void 返回值（dispatchEvent 布尔泄漏）1F→P

日期：2026-11-02。切片号 s235（承 s234）。

## 入场定刀位

s234 下一步候选：input-type-button 1F（s233 仅记失败签名
`expected (undefined) undefined but got (boolean) true`，测试未读）——
本轮一轮勘根闭环。

## 勘根（零仪器）

测试面（input-type-button.html:26）：`assert_equals(buttons[0].click(),
undefined, ...)`——`click()` 应返 undefined（spec HTMLElement.click()
是 void），我方返 `true`。失败签名定路径：`true` 只能来自 proxy 侧
click() API（part04 `prop === 'click'`）尾部的 `return notPrevented;`
——把 `_dispatchWithBubble`（即 dispatchEvent）的布尔结果泄成返回值；
wrapper/plain-node 路径（part03 `_zwMElClick`）只能返 false
（preventDefault 臂）或 undefined，与签名不符。关闭面核对：
- R313 disabled 早退臂本就 `return;`（undefined）——Event-dispatch-on-
  disabled-elements 钉面不受影响。
- shim 内部无调用方消费 click() 返回值（grep `= x.click()` /
  `if(...click())` 零命中——三处调用全语句式）。
- 语料断言面：唯 input-type-button 一处断言 click() 返回值。
- part04:5869 dispatchEvent prop 函数的同名 `return notPrevented;` 是
  spec 正确面（dispatchEvent 返 boolean），不动。

## 施工（+5/-1：5 注释换 1 代码，单文件单点）

`crates/engine/src/js_dom_shim/part04.js` click() API 函数尾删
`return notPrevented;` 落尾返 undefined，加 s235 注释（spec void +
WPT 钉面 + 与 disabled 臂口径归一）。`notPrevented` 变量在函数前段
R57 守卫仍有使用，无孤立变量。wrapper 路径 `_zwMElClick` 的
preventDefault `return false` 是另一代码路径（R155 LABEL 簇 11F 建立其
上、无失败语料面），本轮不动不扩面。

## 验证

- solo：input-type-button 5/5 全 Pass（修复前 4P+1F）——目标 1F→P；
  click-user-gesture 14P/0F（s186 瞬态激活面）。
- 目录级 the-input-element：1655P/7F/1T，7F/1T 逐键对账全在册——
  checkable-active ×2 = s196/197 flake 族 solo 侧（双向摆动在册）；
  color.window.js = 语料缺口挂账；date-readonly-keyboard = keyboard
  车道；email-set-value = 下轮候选；maxlength-number 文件级 Timeout =
  基线同态；radio-focused-and-scrolled = 布局车道；reset.html form
  element pointer = parser form pointer 缓行（用户决策 2026-10-23）。
  零回归签名。
- 域 A/B：三轮归因（本轮扰动键纪律最重的一次）——run1（修复树，紧随
  solo 构建后）：8095 行 vs 基线 8101，目标 F→P 之外 4 个文件级形变
  （focus-dynamic 子测试 P 丢→文件 T、customizable-select 两文件子测
  F→文件 F、dialog-closedby-complex 子 F→文件 T）——三域均属在册交互
  时序域环境截断（s160 先例逐文件同名）；stash 干净树复跑：**与 s234
  基线逐键零差异**（7508P/512F/81T 完全一致，证明当前机器安静窗口稳
  定且基线可复现）；run2（恢复修复树，同安静窗口）：**恰 2 键位移**
  = 目标 input-type-button F→P + checkable following-label P→F（在册
  s196/197 flake 族常规侧——该键四轮 P→F→F→P 双向摆动有档，s233 轮
  幸运侧 F→P，本轮回摆不归因不计数）。run1 噪声未在 run2 复现 =
  负载瞬态定谳（solo 三文件对照佐证：focus-dynamic 80P 全绿、
  customizable-select/dialog 子测试态正常呈现）。行级分布 7508P/
  512F/81T 与基线同数（目标 +1P−1F 与 flake −1P+1F 对冲）——键级
  join 才见真位移，行级同数不等于无变化（对账教训）。官方日志
  s235-ab.log（= run2）。
- 注记：form-requestsubmit 不在 runner 案集（s234 基线 0 键一致——
  本轮初查 solo filter 0/0 系案集缺席非回归，防误读记录）。

## 方法论教训

- **三跑归因协议**（扰动键形变超出在册 flake 族时的最重装备）：修复树
  跑 → stash 干净树跑 → 恢复修复树安静窗口重跑。本轮干净树与基线逐键
  零差异既是「基线可复现」验证也是「机器当前安静」验证；修复树二跑只
  剩在册 flake 键 = 首跑形变全部归负载瞬态。三跑顺序关键：干净树跑放
  中间，让恢复跑继承已证安静的窗口。
- **行级分布同数 ≠ 无变化**：本轮 7508/512/81 与基线逐位同数，但键级
  join 有 2 键对向位移（目标 F→P + flake P→F）——只看分布会漏掉目标
  翻转并漏掉 flake 回摆；键级 (key,state) join 永远是判据，分布数只是
  粗筛。

## 承袭挂账（不动）

- email-set-value 1F（文本控件 selection ↔ window.getSelection 集成
  面，中面）；inserted-or-removed 1F（innerHTML SELECT selectedness，
  先核对 s228 挂账同根）；color.window.js 语料缺口（补资产 flow）。
- keyboard 2F 车道；table/label 簇用户门禁 GB-20261031；parent-label
  布局零盒 2F；checkable following-label flake 族在册。
