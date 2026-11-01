# M2-S199：Space 键激活 keydown preventDefault 抑制——runner space_buttonish 激活门

**日期**：2026-11-01
**入场**：s196 挂账序「Space 激活行为」分型——按钮面 1F
（button-activate-keyup-prevented.html ::「Button activation submits on
keyup, but not if keydown is defaultPrevented」，失败签名
`assert_unreached: button got incorrectly activated`——第二次按键
（keydown preventDefault 后）仍触发 click）。checkbox Space checkedness
翻转分型无 corpus 驱动（非施工候选）。customizable select 177F 挂起
待拍板（GB-20261031 征询在途）。
**结果**：**1F 翻绿**；forms 全树 keyed diff F→P 恰 1 靶键零真 P2F。

## 勘根

- 靶面两段：先 `send_keys(button, " ")` 断言 click promise 兑现
  （既有绿面）；再挂 document 级 keydown listener `preventDefault()`
  + click `unreached_func`，`send_keys(" ")` 断言**无 click**——当前
  失败在第二段。
- runner send_keys 的 space_buttonish 臂（R3254-K4 切片 2，
  tests/wpt-runner/src/testharness.rs ~11724）：派发 keydown 后
  **返回值丢弃** → 派发 keyup → `dispatch_action`（webview 递归
  Activate 合成 click）**无条件执行**——页面级 keydown
  preventDefault 抑制不了 keyup 激活。
- 语义判据：UI Events——取消键即取消其默认动作；Space 激活
  （click）时序在 keyup（keydown 只派事件），keydown preventDefault
  → keyup 激活不触发，keyup 事件本身照派。同文件 scroll-key 臂
  既有 `!= "prevented"` 门同型先例。
- 三个 Space 负案（prevented-default/untrusted/disabled，checkbox
  onblur 族）修复前已 Pass；s196 `__zwKeyActiveGate` 是 :active 标记
  通道非激活通道——与勘根一致，无面冲突。

## 施工（1 文件 rust，净 +10/−2）

- space_buttonish 臂：keydown 结果捕获（`space_keydown_result`）→
  keyup 恒派 → 激活 `dispatch_action` 加 `!= "prevented"` 门。
  clippy `collapsible_if` 折叠为 let-chain（`if result != "prevented"
  && let Some(error) = ...`）。键事件序 keydown→keyup 不变，非
  buttonish 空格路径零改动。

## 验证

- 靶面双形 Pass：嵌套臂 + 折叠臂终形各复验一次（终形
  s199-targeted-final.log）。
- **stash 隔离 A/B（FILTER=semantics/forms，与 s195-s198 同口径）**：
  - forms 全树：基线 **4451P/250F/11T** → WIP **4452P/249F/11T**。
    keyed diff（(key,state) 对 sort -u）**F→P 2 = 靶键 + flake 域
    button-active-onblur**（有利向）；**P2F 1** = checkable
    -active-onblur checkbox-spacebar（`:active` onblur flake 域在册
    双向摆动样本，checkbox 空格路径本刀未触无机制关联）；
    LOST/GAINED 0。
  - interactive-elements：基线与 WIP 均 **439P/44F/0T bit-exact**
    （零键位移）。
- runner 自身单元测试 **276P/0F/76I**（变更面 = runner crate；lib
  zero-engine/zero-dom 测试不经 runner 通道，A/B 无信息量，本轮以
  runner 单测承载面守卫——与 shim 切片的 lib sweep 口径差异记此）。
- fmt 过 + guarded-clippy workspace `-D warnings` 过（首轮
  collapsible_if 阻断，修为 let-chain 复过）。
- 过程注记：首轮 forms 基线误跑无 FILTER 口径（全 html/semantics
  树 7404P/599F/83T，与 s195-s198 的 FILTER 口径不可比），留档
  `s199-fulltree-stash-baseline.log` 后按协议 FILTER 重跑；无早杀，
  为口径修正非事故重跑。

## 残留/挂账

- **新记**：uievents keyboard-click-event 文件级 Timeout——本刀
  WIP 与 stash 基线**双臂一致**（tip 现状漂移非本刀引入；R3254-K4
  切片 2 / 尾簇 32 时代绿面），勘根挂账待下轮。
- `:active` onblur flake 域 +2 键样本（F2P 1 + P2F 1）。
- checkbox Space checkedness 翻转分型无 corpus 驱动（维持挂账）。
- customizable select 177F 挂起待拍板（GB-20261031 征询在途）。

## 下一步候选

customizable select 177F 挂起待拍板（GB-20261031）→ uievents
keyboard-click-event Timeout 勘根（新挂账，同属 runner 键盘通道）
→ s196 挂账序余项（inert-svg-hittest 第 2 子测、`:not(:active)`
否定面；父 label 零矩形属渲染流域非本流）。

## 日志

`s199-targeted.log` / `s199-targeted-final.log` /
`s199-fulltree-stash-baseline.log`（口径误跑留档）/
`s199-forms-stash-baseline.log` / `s199-forms-wip.log` /
`s199-interactive-stash-baseline.log` / `s199-interactive-wip.log` /
`s199-runner-unittests.log` / `s199-uievents-keyboard-click.log` /
`s199-uievents-keyboard-click-baseline.log` / `s199-clippy.log`
（同目录）。
