---
date: 2026-10-22
modules: engine
---

# shim void 方法返回值泄漏 → WPT cleanup abort → 整文件 Timeout 吞案

## 问题

`dialog-closedby.html`、`dialog-closedby-corner-cases.html`、
`dialog-popover-closedby-simple.html` 等文件自 s22 起同一失败形态：首案
Pass/Fail 后整文件 Timeout，余 27-29 案被吞。轮次挂账曾归因「detach 窗口
身份链」（错误层）。

## 根因

两层机制叠加，第二层才是断链点：

1. **shim 侧**：`close()` IDL 包装透传内部 helper 的 `wasOpen` boolean
   （part04.js `return _zwDialogClose(...)`）。spec `close()` 是 void
   （返 undefined）。
2. **testharness 侧**（语料 `resources/testharness.js`，正确行为）：
   - `is_valid_cleanup_result`：cleanup 回调必须返 `undefined`（promise_test
     可返 thenable）；返回 boolean → `bad_value_count` → `tests.abort()`。
   - `Test.prototype.cleanup` 的 `on_error`：cleanup 回调**抛错**同样
     `tests.abort()`（源码注释原话 "Abort tests immediately"）。
   - `tests.abort()` 后完成回调永不被调 → runner 探到
     "completion callback was not called" 合成 file Timeout，已完成的
     partial_results 照常上报、余案全部消失。

即 `t.add_cleanup(() => dialog.close())` 返 `false` 一个值，就吞掉整文件。

## 修复

part04.js 三处重复 `close` 包装去掉 `return`（void 化）；part14.rs R3290
契约两处钉定旧错误行为的断言更正；part60.rs 新增
`test_dialog_void_return_r4500`（dialog 四方法全生命周期 + remove/focus/
blur/select/reset/showPopover/hidePopover 族审计 + close 事件仍达）。

## 诊断路径与避免

- file Timeout 的**形态学**是关键信号：partial results 只有首案 + 余案全
  无 → 疑 cleanup abort，而非单案 hang。写探针页时每步独立 promise_test
  （partial flush 定位断点）+ cleanup 空操作对照（DA/DB 文件区分 body
  close 与 cleanup close），两轮即锁定。
- spec void 方法在 shim 一律不得透传内部 helper 的状态返回值；新契约审计
  面落在 part60（void 返回值族），后续新增 void 方法照此补断言。
- 上游 testharness 的 abort 语义（cleanup 返非 undefined / 抛错都杀全文件）
  意味着 shim 任何「顺手返回状态」的 void 方法都是全文件放大器；归因时先
  排除该层再往产品语义走。
