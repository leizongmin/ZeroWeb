# M2-S211：button 激活门双缺口——command/commandfor 缺省 type + fieldset 传染提交门

- **日期**：2026-11-01
- **目标面**：`button-click-submits-with-commandfor.html` 3F（implicit
  type + command/commandfor 属性点击不应提交）+
  `button-click-submits.html` 1F（disabled fieldset 直下 button
  `.click()` 不应提交）
- **状态**：收口——两语料 16/16 Pass；forms 全域 A/B 净 F→P=4 恰 4 靶
  键，P→F=0，键漂移 0，其他位移 0

## 勘根

- **缺口一（缺省 type，3F）**：spec §4.10.5.7 #the-button-element 的
  missing value default——invoker commands 集成后 Submit Button 缺省仅
  在 **command 与 commandfor 属性皆缺**时成立，任一存在 → Button
  state（点击无提交行为）；显式 type=submit 不受影响（语料后 3 测
  Pass 钉面）。现状两个激活门（.click() API 门 part04:5838 区 +
  R106 dispatchEvent click 默认动作门 part04:5710 区）BUTTON 空
  type 恒判 submit——`_r106ty === ''`/`_subTy === ''` 并入 submit
  分支。R4400 invoker command 事件派发面（part73 钉）不经 type 门
  不受扰。
- **缺口二（fieldset 传染，1F）**：两个激活门的 disabled 检查只查
  **自身** disabled 属性——disabled fieldset 直下 button `.click()`
  照常提交。s210 的 `_zwFeDisabled` 判据（#concept-fe-disabled 首
  legend 豁免）已在 validity 通道落地，激活门未接；语料对照面
  legend 内 button 应提交（该测 Pass——自身 disabled 门与豁免面对，
  缺的就是传染）。
- **勘根深挖（lw 通道 miss）**：首版帮手用 `__zw_has_attr`（纯快照
  通道）读 command/commandfor——探针实测
  `__zw_has_attr('#b1','command')` 恒 '0' 而 `__zw_has_attr_lw` 返
  '1'：`setAttribute` 写 **latest-wins 层不落快照**，纯快照通道恒
  miss。R4400 祖先链（part01:1752）/ s136 type 读早已 lw 口径，帮手
  初版漏接——修后 R5378 六腿全绿。同教训适用于 disabled 读（帮手 A
  一并 lw 融合）。
- **不能伤的键**：R313 disabled 门（click 臂开头自身 disabled 派发
  面——不在本轮改动路径）；显式 type=submit/reset 语义（帮手透传显
  式值）；INPUT 提交/image 面无 command 缺省语义（帮手 tag 门限
  BUTTON）；R57 validity 面（s210 收口，`_validityState` 的 disabled
  判定本轮共享化替换为 `_zwFeDisabled` 调用——R5328 六腿复跑钉回
  归）；R3048 reset 按钮（`type='reset'` 帮手透传，asReset 路径不
  变）。

## 施工

（shim 2 文件 + rust 测试 1 文件；语料零改动）

- **part03.js 共享帮手 ×2**（`_validityState` 前，IIFE 顶层）：
  `_zwFeDisabled(sel, handle)`——自身 disabled（lw 融合读）+ disabled
  fieldset 祖先走查（首 legend 元素子直子豁免 + 豁免后继续上行，
  guard 64）；`_zwButtonEffType(sel, handle, tag)`——显式 type lw 融
  合读优先，BUTTON 空 type 且 command/commandfor 任一存在（lw 融合
  读）→ 'button'。
- **part04.js 两激活门换帮手**：type 读换 `_zwButtonEffType`（s136
  lw 融合读同源保留在帮手内）；disabled 判定换 `_zwFeDisabled`（补
  传染面）。
- **part03.js `_validityState` 共享化**：s210 的内联 fieldset 走查
  块替换为 `_zwFeDisabled` 调用（同语义；R5328 复跑钉回归）；willValidate
  臂（s193 disabled-001 钉面）不动，共享化挂账承袭。
- **part09.rs R5378 新桥测试**
  （test_r5378_button_effective_type_command_default_fieldset_submit_gate）：
  六腿——implicit+command 不提交 / implicit+commandfor 不提交 / 显式
  type=submit 仍提交 / disabled fieldset 直下不提交 / legend 内豁免
  仍提交 / 属性全清缺省回归提交。

## 验证

- 拼接 shim（15 part include 序）node --check SYNTAX_OK。
- 桥测试 R5378 单测 **1 passed**（首版 lw miss 经探针
  `__zw_has_attr` vs `__zw_has_attr_lw` 对比实测归因，修后六腿全
  绿）；R5328 复跑 **1 passed**（共享化替换无回归）。
- 靶面：`FILTER=button-click-submits` → **16/16 Pass**（两文件，4F
  清零）。
- forms 全域 A/B（两侧各 4711 (key,state) 对）：**净 F→P=4 恰 4 靶
  键**；另 1 键 F→P 落 onblur flake 族（active-onblur Buttons
  spacebar，s210 摆走的键回摆——族内闭环）；P→F=0；键漂移 0；其他
  位移 0。
- zero-engine lib：**3671P+0F**（R5378 计入）。
- fmt 本刀零 diff；guarded-clippy 过（0 警告）。
- push 前 rebase 带入并行 clone 同文件上游提交 e73eb4cbd（part03 `_zwMEl`
  attr 写入点索引失效，editing 面）——合并树复验：shim 15 part 按
  include_str 序拼接 node --check SYNTAX_OK，zero-engine lib 复跑
  **3671P+0F**，两组改动组合无损。

## 挂账

- `__zw_has_attr` 纯快照通道对 setAttribute 写面恒 miss 的坑是通用
  的（本轮探针实测）；shim 内 sel 面属性存在性读应统一 lw 融合——
  存量点位审计（`__zw_has_attr(` 非 lw 调用面 grep 审计）挂账，不在
  本刀扩大。
- fieldset 走查三处同构（`_zwFeDisabled` 帮手 + part04 willValidate
  臂内联）——willValidate 臂共享化仍挂（disabled-001 钉面，独立小
  刀评估）。
- part03 `_zwMElCheckValidity` FORM 分支短路形态（s210 挂账承袭，
  本地 _zwMEl 树面无语料未动）。
- onblur flake 域持续采样（本轮 +1 回摆样本，s210 摆动闭环）。
