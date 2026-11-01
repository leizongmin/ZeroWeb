# M2-S200：键盘激活 click-only 事件序——uievents keyboard-click-event 4 子测收口

**日期**：2026-11-01
**入场**：s199 挂账勘根——uievents/interface/keyboard-click-event.html
文件级 Timeout（runner 诊断 `pending:4, tests:4, phase:4` 零上报，
s199 双臂一致）。
**结果**：**4/4 Pass——该文件自 M1 基线（2026-10-03）以来首次绿**；
forms 全树 keyed diff 零 P2F / 零键增减。

## 勘根

- **历史定谳**：archive JSON 29 份（uievents-compat 2026-10-03 M1 基线
  → tail28）该文件**全程 Timeout 从未绿**——s199 账本「R3254-K4/尾簇
  32 时代绿面漂移」表述有误（勘误已记 s199 账本）；尾簇 32（2026-10-06
  39ca800f1）只修了行为层（send_keys 自聚焦，探针 4/4 click 兑现），
  余面当时标「EventWatcher/mini-report 交互层」未勘根。
- **探针五轮二分**（探针页验后删）：p1 send_keys promise 兑现 ✓ /
  p2 空格激活 click 兑现（s199 靶面同形）✓ / p3 EventWatcher
  wait_for 兑现 ✓ / p4 Actions 构造无抛 ✓ / p5 真文件同形 + Fail
  消息转储 eventLog → 揭根。
- **根因**：空格激活走 InsertText → webview Activate 合成**全指针/
  鼠标序**（p5 实测 eventLog=[pointerdown, mousedown, pointerup,
  mouseup, click]），而 UI Events 键盘激活语义 = **只发 click**（无
  pointer/mouse 事件；同轮 p5 Enter 臂经尾簇 11 既有 click-only 通道
  实测 [click] 单事件 Pass——同文件双键不对称）。该语料在 async
  executor 内直接 assert_array_equals——不匹配抛错使外层 `new
  Promise` 永不 settle → **hang 非 Fail**（4 子测全 pending 零上报
  形态的机制解释；p1-p4 单层全通故挂点必在组合断言层）。

## 施工（1 文件 rust，净 +21/−4）

- space_buttonish 激活从 `dispatch_action`(InsertText)（webview 全序
  合成）换 `script_dispatch_dom_event` click-only——PointerEvent 实例
  + pointerId -1 + pointerType ''，与 Enter formless buttonish 臂
  同型同参（尾簇 11 先例；R108 pre-click activation 经 PE 原型链接
  通）。s199 preventDefault 门与 keyup 恒派保持；非 buttonish 空格
  路径零改动（探针门内）。

## 验证

- 靶面 keyboard-click-event **4/4 Pass**（fmt 终形复验仍 4/4）；
  s199 靶面 button-activate-keyup-prevented 保持 Pass。
- **stash 隔离 A/B（FILTER=semantics/forms）**：基线 4452P/249F/11T
  → WIP 4452P/249F/11T，keyed diff（(key,state) 对 sort -u）
  **P2F 0 / LOST 0 / GAINED 0**，唯一位移 F→P 1 =
  checkable-active-onblur spacebar（`:active` onblur flake 域在册
  双向摆动，有利向）——forms 树无面依赖旧全序合成，零机制回归实证。
- interactive-elements：基线与 WIP 均 483 行 **bit-exact**
  （439P/44F/0T）。
- **uievents 域面收窄**：语料 grep send_keys 仅 5 文件，其中
  buttonish-空格可达面唯 keyboard-click-event（modifier ×2 目标非
  buttonish、textInput ×2 走 helper 非 buttonish——探针门未触，旧
  InsertText 路径逐字节不变）；全树 A/B 无判别力，以靶面 + 机制
  收窄承载。过程注记：首轮 uievents 全树跑超 Bash 10min 窗口被杀
  （test-guard 1200s 限制内），部分日志弃。
- runner 单元测试 **276P/0F/76I**；fmt + guarded-clippy workspace
  `-D warnings` 过。
- 过程注记：PUA 键字面量 Write 工具不可见坑再踩——uE006 探针行被
  静默写成空串（s196 在册坑），python 字节级改写为 `` 转义
  修复；探针页 5 件验后删（语料零改动）。

## 残留/挂账

- 无新挂账。`:active` onblur flake 域 +1 键样本（F→P 有利向）。
- customizable select 177F 挂起待拍板（GB-20261031 征询在途）。
- 键盘激活 click 的 isTrusted/trusted 印记面：语料无断言，不扩刀
 （click-only 通道经 `__zw_dispatch_event` UA 通道，R312 trusted
  双印既有）。

## 下一步候选

customizable select 177F 挂起待拍板（GB-20261031）→ s196 挂账序
余项（inert-svg-hittest 第 2 子测、`:not(:active)` 否定面；父 label
零矩形属渲染流域非本流）。

## 日志

`s200-probe-round1.log` / `s200-probe-p5.log` /
`s200-uievents-targeted.log` / `s200-uievents-targeted-final.log` /
`s200-forms-button-activate.log` / `s200-forms-stash-baseline.log` /
`s200-forms-wip.log` / `s200-forms-targeted-final.log` /
`s200-interactive-stash-baseline.log` /
`s200-interactive-wip.log` / `s200-runner-unittests.log` /
`s200-clippy.log`（同目录）。
