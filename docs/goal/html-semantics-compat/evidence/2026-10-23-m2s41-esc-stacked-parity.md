# M2 切片 41 — 挂账定性：ESC 组栈边缘乱序 → Chrome-parity 同红收口（2026-10-23）

## 范围

s35 挂账「ESC 组栈边缘乱序」最后一个失败面：dialog-closedby-corner-cases
「Pressing escape, when focusin calls showModal, modeless」（82 files dialog
集群唯一 ESC 相关 F，s35 账本即 29P/1F）。

## 探针钉定（zz-probe-s41，验证后已删）

复刻 corpus 全文件 + 事件日志，我方引擎该案实际事件序：

```
pre-ae:outside | show() | beforetoggle:open | focusin:dialog | post-open-ae:dialog
| cancel | beforetoggle:closed | handler-showModal | beforetoggle:open
| focusin:dialog | handler-showModal-ok | focusin:outside | close(open=true)
| final-open=true   ← 期望 final-open=false，FAIL
```

链路：show() 聚焦空 modeless dialog（focus delegate 无 → subject fallback）→
ESC close request → close-the-dialog 焦点恢复尾步：prevFocused=outside（show()
记录），anchor 门「焦点锚是 subject shadow-including 后代」命中（焦点=dialog）
→ 对 outside 跑 focusing steps → focusin → 处理器 re-entrant showModal → 重开。

## spec 钉定（html.spec.whatwg.org interactive-elements.html，2026-10 现行文本）

- 「when a dialog element is to be closed」恢复尾步：previously focused
  非空 → 焦点锚 shadow-including 后代 **或 wasModal** → run focusing steps。
  我方 R4600 `_zwDialogRestoreFocusR4600` 门逐字一致（本次正向确认）。
- previously focused element：**show() 与 showModal() 都记录**（"When
  showModal() and show() are called, this element is set to the currently
  focused element before running the dialog focusing steps"）。
- dialog focusing steps：「If control is not focusable, this will do
  nothing」——但 dialog 可聚焦面由 Chrome 定义：dialog-focus-shadow.html
  （chromium 作者 Mason Freed）modeless「show: No autofocus, no
  delegatesFocus, no siblings」臂明确断言 `activeElement === dialog`
  （**show() 无 focus delegate 时聚焦 dialog 自身**）；whatwg/html#8199
  只移除 TAB 序可聚焦（dialog-focusability.html），不移除 focusing steps
  聚焦。

## 试错与回退（R5244 假设门，未合入）

假设「subject fallback 且非 focusable（无 tabindex 且非模态）→ no-op」
落地后：目标 subtest 转绿（GAIN 1），但 **REG 4**——dialog-focus-shadow
「show:」三臂 + dialog-focus-previous-outside「dialog itself is focused」
（全部 Chrome 断言面：show() 聚焦 dialog）。两面矛盾（corner-cases 期望
不重开 ⇔ focus-shadow 期望聚焦）只能二选一，Chrome 选边后者。回退后
全域 **419P/61F/2T 对 s40 逐位 REG 0**，lib 3432P/0F，零代码净变化。

## Chrome-parity 同红证据

- crbug 417571176「the-dialog-element/dialog-closedby-corner-cases.html
  is failing」（2025-05 建，父 bug 376516550 closedby 特性）：上游
  "[dialog] Fix assertion" 已改过一次 respondsTo（现文本「if it was
  originally modal」即改后），**2026-04-01 仍 "still failing according to
  wpt.fyi"**，issue 未关。
- Safari 曾 28/28 过、Chrome 0/28 过（不同阶段），该文件为 Chrome 特异
  缺口；本项目按 Chrome 拉齐（corpus 验收标尺 = Chrome 行为面）。
- 我方引擎行为链与 spec 文本逐字一致 → 该案与 Chrome 同红即正确
  parity；Chrome 侧修复（crbug 关闭）后随 corpus 刷新自然转绿，无需
  预埋偏离。

## 收口定性

- 「ESC 组栈边缘乱序」挂账**关闭**（定性 Chrome-parity 同红，语料钉住，
  crbug:417571176 引用入账）；非「组栈乱序」实现缺陷——R5236 两段式
  与 R4600 恢复门经本轮逐字对账无漂移。
- dialog 集群余面：non-modal close 不出组栈（下一挂账）、
  dialog-focusability TAB 序 1F（sequential focus navigation 面，
  whatwg/html#8199 后半，既有缺口另账）、focus-after-close Complex 1F。
- 数据 [2026-10-23-m2s41-esc-stacked-parity.json](2026-10-23-m2s41-esc-stacked-parity.json)
  （meta + interactive-elements 全域 98 files 482 subtest dump）。

## 门禁

zero-engine lib **3432P/0F**（回退后 = s40 基线）；clippy/fmt 无变化
（零 Rust/JS 净 diff）；零新增依赖。runner release 二进制随 make 依赖
链重建两次（假设门 + 回退）。
