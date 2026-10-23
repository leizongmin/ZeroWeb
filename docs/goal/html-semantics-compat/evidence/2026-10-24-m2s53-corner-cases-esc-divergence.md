# M2 切片 53（s53）——dialog-closedby-corner-cases「focusin calls showModal, modeless, ESC」案定性：spec 一致行为，测试期望与真实浏览器分歧

日期：2026-10-24。零代码收口：该文件 1F（30 子测试 29 Pass）定性为**测试期望分歧**，非 shim 缺口，1F 保留挂账。

**对 s41 前史的更新**：s41（2026-10-23，同案同机制独立重推，探针序列与本轮一致）以「Chrome-parity 同红」（crbug 417571176，2026-04 时 wpt.fyi 仍红，上游改过一次 assertion 仍未过）关闭挂账。本轮新证据推翻「同红」前提的时效性：2026-10-23 最新 master run（chrome-158.0.8118.0 + firefox-161.0a1，同 revision c645564b0e）该文件双双 30/30 转绿；crbug 417571176 对应的 CL 正是 wpt #58337（2026-04-01，assert_true→assert_equals，仅 click 腿断言形式、无行为变化）。转绿与同 build 直探矛盾，见下文反证段。s41 试错回退结论（R5244 subject 非 focusable 门：目标案 GAIN 1 但 REG 4）继续有效——弯就 shim 迎合该期望有实测回归代价，且与 spec/直探相悖，双重不可取。

## 案情

失败案 `Pressing escape, when focusin calls showModal, modeless`：断言 `respondsToEsc === shouldRespond`，`respondsTo('modeless') = openMethod !== 'modal' = true`，期望 ESC 后 `!dialog.open === true`（dialog 关闭）；我方实况 `open=true`。

测试机制：`<dialog closedby="any">` + body 级 `focusin` 监听（target 不在 dialog 内就 `dialog.showModal()`）。setup 链 `outside.focus()` → showModal（模态，prevFocused=outside）→ `openDialog('modeless')` 先 `close()`：close-the-dialog 焦点恢复 focusin(outside) → 级联 showModal 重开（模态）→ `if (!dialog.open)` 跳过 `show()`。ESC 时 dialog 已模态。

## 证据链（四重一致：open=true 重开）

1. **live spec**（raw.githubusercontent whatwg/html main，/tmp/html-spec.txt 67213 行起 close-the-dialog）：恢复门「focused area 的 DOM anchor 是 subject 的 shadow-including inclusive 后代，或 wasModal 为真 → 对 previously focused element 跑 focusing steps」。ESC 时 focus=dialog（inclusive 后代 ✓）且 wasModal=true → 恢复必跑 → focusin(outside) → 处理器 showModal → 重开。showModal 全部前置守卫（66995 行起：open 属性/fully active/connected/popover 态/beforetoggle）无一命中。无重入门。
2. **Chromium main 源码**（chromium.googlesource.com html_dialog_element.cc）：`Close()` 焦点恢复在 `is_closing_` AutoReset 释放后执行 `previously_focused_element_->Focus(...)`（preventScroll），门 `was_modal || FlatTreeTraversal::Contains(*this, *FocusedElement())`；`showModal()` 无重入/关闭期守卫。重开必发生。
3. **直探 Chromium 151**（/usr/bin/chromium，playwright 真 ESC 键）：modeless 臂 ESC 后 `open=true, focus=dialog`，级联序列与 spec 推演逐步吻合。
4. **直探 Chrome for Testing 158.0.8118.0**（storage.googleapis.com/chrome-for-testing-public/158.0.8118.0/linux64/chrome-headless-shell-linux64.zip，**即 wpt.fyi 绿色 run 的同一二进制**）：modeless 臂同样 `open=true` 重开。

## 反证（唯一指向 closed 的来源）与矛盾

wpt.fyi revision c645564b0e（run 2026-10-23）chrome-158.0.8118.0 与 firefox-161.0a1 summary 均记该文件 `{"s":"O","c":[30,30]}`。summary 压缩格式已验证（125141 条目、5545 条 c[0]<c[1]、s 分布含 F/T/E，c=[passes,total] 无疑）。但同 build 直探（证据 4）即重开，页面内机制推不出 closed：读点无论落在 send_keys 往返的哪个位置，`dialog.open` 均为 true（setup 级联已重开，ESC 级联同步重开）。三臂中仅 `open` 臂（`dialog.open = true` 开启，prevFocused 为 null，恢复步整体跳过）按 spec 必然 closed——两引擎 30/30 与同 build 直探矛盾只能落在 wptrunner 环境层（键盘传输/rAF 调度差异），页面可见语义内无法构造 PASS 世界。

## 我方 runner 实证（探针页 zzz-probe-s53.html，已删）

`ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1 test-guard 包裹 zero-wpt-runner` 单案序列：

```
focusin target=outside openAtFocusin=false      ← outside.focus()
focusin target=dialog openAtFocusin=true        ← showModal [A] dialog focusing steps
  showModal ok open=true
focusin target=outside openAtFocusin=false      ← close() 恢复
focusin target=dialog openAtFocusin=true        ← 级联 showModal [B]
  showModal ok open=true
setup-done open=true focus=dialog               ← 与 Chrome 直探同构
cancel event                                     ← ESC
focusin target=outside openAtFocusin=false      ← ESC-close 恢复
focusin target=dialog openAtFocusin=true        ← 级联 showModal [C]
  showModal ok open=true
close event open=true
final open=true focus=dialog
respondsToEsc=false
```

与 CfT158 直探序列逐步同构。其余 29 子测试（modal 臂 ESC、open 臂 ESC、全部 click 腿、其余 4 个 changeMethod）Pass。

## 裁决

- 我方行为 = spec + Chromium main 源码 + Chromium 151/CfT158 直探。改 shim 迎合该案期望即偏离规范与真实浏览器，且 s41 实测 REG 4，不做。
- 记账：1F 保留，标注 spec-consistent known divergence（s41「Chrome-parity 同红」依据已被 2026-10 双引擎 CI 绿取代，但直探矛盾未解前不作为收口依据）；后续若 wptrunner 环境差查明或我方 runner 引入同款环境层，再复评。
- 同族挂账不变：pointer-capture 2F、bounds-clicking 2F（dialog-closedby-bounds-clicking.html，wpt.fyi Chrome 14/14 全绿，非分歧面）待分诊。

## 验证

- runner 全域 interactive-elements 计数不变（本轮零代码，433P/50F/0T 维持）
- 探针页/本地 HTTP 服务/临时件已清理（git status 除既有 playwright-matrix 未跟踪件外干净）
