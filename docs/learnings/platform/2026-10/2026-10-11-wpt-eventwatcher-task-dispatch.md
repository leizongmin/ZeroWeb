---
date: 2026-10-11
modules: engine
---

# shim 异步事件必须 task 承载——microtask 链与 promise_test 完成回调竞速

## 问题描述

FileAPI M2-P3 收口 FileReader 读取状态机时，事件派发首版用 `Promise.resolve().then`
链逐事件间隔（loadstart → progress → load → loadend 各占一个微任务）。
`filereader_result.any.js` 的 4 个 progress 变体仍挂：

```
assert_true: Not expecting event, but got loadend event expected true got false
```

而结构几乎相同的 loadstart 变体全部通过，且无法用纯 FIFO 微任务序解释两变体的
不对称。

## 根因分析

WPT `EventWatcher`（testharness.js 内建）的语义是严格状态机：事件到达时若无挂起的
`wait_for`（`waitingFor === null`）或类型不匹配即断言失败；promise_test 的完成回调
（done-part）也是微任务，与页面事件排在同一条微任务队列里。

microtask 链承载事件时，「最后一个事件」和「测试函数返回触发的 done-part」的先后
取决于测试体里 `await` 续体的排队位置——即取决于测试写法（等的是哪个事件、中间
多几跳），而非实现语义。filereader_result progress 变体的第二读尾部：`wait_for`
在 progress 处 resolve，测试函数随即返回，done-part 与读链的 load/loadend 步同队列
竞速，loadend 抢先即撞 `waitingFor=null`。loadstart 变体恰好多一跳
（progressStep 无监听）使 done-part 先行，属排队巧合而非正确。

真实浏览器中 loadstart/progress/load/loadend 是独立 **task**：promise 续体
（含 done-part）在 task 之间全部 drain 完毕，EventWatcher 状态机必然推进到位。

## 解决方案

事件派发改 `setTimeout(fn, 0)` 逐 task 承载。runner 侧 `__zw_test_setTimeout` stub
（R3254-E2）本就是确定性 FIFO 队列（`__zw_fire_due_timers` per-task 模式一轮一个），
序确定且必然 settle；无 host 环境（engine 桥接测试 / reftest）setTimeout 回落
microtask，仅粒度退化、序不变，桥接测试零改动全绿。改后 32F 簇清零。

**通用规则**：shim 内新增「异步多事件接口」（XHR 进度族、IDB request 族、
MediaElement 族等），事件一律 task 承载，禁止用 promise 链模拟事件间隔——
WPT EventWatcher 形态的用例会与 promise_test 完成回调竞速，且失败形态随测试写法
漂移，极难定位。
