---
date: 2026-10-23
modules: wpt-runner, engine
---

# 真 Chrome 仲裁：WPT 语料在本地 Chromium 的基线跑法

## 问题

WPT 语料的期望断言与引擎行为冲突时（html-semantics s25–s27 三轮反复归因 corner-cases `focusin calls showModal` 变体），缺真浏览器基准只能靠 spec 字面序推演——而 spec 字面序与 Chrome 实测行为可能不一致（本例：上游期望 closed，真 Chrome 端态 open 且测试同样失败）。没有基准，无法区分「引擎 bug」与「上游期望在真浏览器也失败」。

## 方案

`tests/wpt-runner/wpt-data/` 可直接作为静态站点跑进本地 Chromium（playwright 铁装 `~/.cache/ms-playwright/chromium-1243`，项目 `tests/playwright-matrix` 带 playwright-core 1.63）：

1. `python3 -m http.server --directory tests/wpt-runner/wpt-data`——语料 `/resources/testharness.js` 同根可达。注意 `testdriver*.js` 不在语料树（runner 自带 stub），需 `addInitScript` 自定义 `window.test_driver`。
2. test_driver 桥必须走 **CDP 真实输入**（`page.keyboard.press` / `page.mouse`），不能派发合成 DOM 事件——Chrome 的 light dismiss、ESC close watcher、mousedown 焦点默认动作都在浏览器输入层，合成事件不触发。
3. 桥接坑：对象字面量 shorthand method 不可 `new`（`Actions: function(){}`）；Actions 实例需带 `ButtonType`/`PointerType`（popover-utils `actions.ButtonType.LEFT`）；WDT 特殊键 U+E00C→`Escape` 需映射；`add_completion_callback` 轮询注册收结果。

仲裁页面可直接用 runner 账本的同一语料文件，subtest 级对比「引擎 vs Chrome」逐条定性：同 F = 上游失败（Chrome parity 非 gap），Chrome P 引擎 F = 真分歧。

## 案例

`docs/goal/html-semantics-compat/evidence/2026-10-23-m2s28-modal-focus-gate.md`——dialog-closedby-corner-cases 30 案：真 Chrome 29/30，引擎修复后同 29/30。
