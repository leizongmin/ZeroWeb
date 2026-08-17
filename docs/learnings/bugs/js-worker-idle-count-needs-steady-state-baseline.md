# JS worker 空闲计数应使用 steady-state 基线

**日期**: 2026-08-17
**相关模块**: `apps/renderer`

## 问题

`async_script_mutation_is_committed_to_webview_frame` 断言 idle drain 后脚本执行总数为 0，
但持久 JS worker 初始化时会完成 shim 启动期 microtask checkpoint，计数稳定为 1。

## 根因

测试要验证的是 renderer 进入 steady state 后，空闲 drain 不反复执行空脚本。绝对计数 0
同时约束了 worker 启动实现，混淆了初始化执行与生产期空闲轮询。

## 解决

测试先 drain 启动期 microtask，再记录 execution baseline；随后执行 16 次 idle drain，
断言计数不增长。定时器路径继续断言仅增加调度脚本和 checkpoint 两次执行。修改后目标测试
连续 3 轮通过。
