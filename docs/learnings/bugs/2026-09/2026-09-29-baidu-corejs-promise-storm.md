# baidu core-js wrapped Promise 微任务自馈环 → V8 堆 OOM abort

## 问题描述

baidu.com 在 ZeroBrowser 加载 ~10-30s 后 renderer 进程必现 int3（signal 5）崩溃，无任何
fatal 文本输出（release 构建静默）。崩溃前 renderer JS 线程 100% CPU 单核自旋，页面
自动化全部超时。此前该站点还有另一形态：同线程 CPU 风暴但进程不死（微任务互递归
RangeError 被站点 catch 重试）。

## 根因分析

两层叠加，均为「全局可观测面」缺口：

1. **`queueMicrotask` 载体再入**（第一层，已先修复）：shim 用
   `Promise.resolve().then(cb)` polyfill `queueMicrotask`，但 `Promise` 是**调用时**全局
   查找。baidu core-js 替换 `globalThis.Promise` 后，其内部调度器经
   `globalThis.queueMicrotask` 再入，形成
   queueMicrotask → 站点 Promise.resolve → 站点 resolve → 站点调度器 → queueMicrotask
   的同步互递归（每圈 ~0x4d8 原生栈直至 StackGuard → RangeError → 站点 catch 重试 →
   CPU 风暴）。修复：shim 初始化时**捕获原生构造器** `_zwNativePromiseCtor`，载体不再
   依赖调用时全局。

2. **`PromiseRejectionEvent` 全局缺失**（第二层，本条主根因）：core-js 的原生 Promise
   完备性检测含 `isCallable(globalThis.PromiseRejectionEvent)`（unhandledrejection 支持
   判据）。缺失 → `CONSTRUCTOR forced` 置真 → core-js **强制 wrapped 模式**替换
   `globalThis.Promise` 为包装实现。该实现的内部状态机制（`newPromiseCapability` 对
   `C === Promise` 返回**裸内部状态对象**作为 promise、`.then` 经原型链继承使状态对象
   伪装 thenable、采纳链 `Qc(t)=t.then` 再入 wrapped then）与本引擎微任务载体叠加形成
   **微任务自馈环**：每圈
   `cs/resolve → Hc/ctor → Sc/state-init → yc(queueMicrotask)` 构造新 promise +
   WeakMap 簿记，队列永不排空，单线程 100% CPU 分配至 V8 堆上限 →
   `V8::FatalProcessOutOfMemory` → `OS::Abort`（int3，静默）。实测 205k+ 圈/~10s。
   Chrome 有原生 `PromiseRejectionEvent` → 检测通过 → 沿用原生 Promise → 无此路径。

定位链路（无 gdb 环境）：V8 `--prof` 日志 tick 栈符号化（code-creation 事件按**最高
时间戳**取活跃函数——JIT 地址复用，取首个会命中 stale 函数）→ 热点函数聚合识别
wrapped Promise 内圈 → 2.4GB core dump `readelf -n` 提取故障线程 RIP/RSP → PT_LOAD
段直接读栈内存 + nm 符号表扫描返回地址（完整调用链
`FatalProcessOutOfMemory ← WeakCollectionSet ← Sc ← Hc ← wrapped then`）→ shim
`_zwRunOrDeferPromiseReaction` 加圈数计数器（host 回调写 /tmp 日志，风暴中仍可用）
采样到肇事闭包 `function(o){e(t,o,n)}`（core-js `is` 偏函数）→ core 堆 mmap 搜索该
源码串提取完整 wrapped 模式源码 → 读出 `Tc` 检测式定位 `PromiseRejectionEvent` 轴心。

## 解决方案

- `js_dom_shim/part01.js`：queueMicrotask 载体捕获原生 Promise 构造器（禁止调用时全局查找）。
- `js_dom_shim/part05.js`：暴露 `PromiseRejectionEvent`（extends Event，`promise`/`reason`
  载荷，spec：PromiseRejectionEvent interface）。全局**可调用性**本身即有兼容效应——
  站点 polyfill 以它判定原生 Promise 完备性。
- 回归测试：`test_queue_microtask_site_promise_replacement_r_baidu_storm`（毒化
  globalThis.Promise 断言载体不再入站点机制）、`test_promise_rejection_event_ctor_r_baidu_storm`。

## 如何避免

- polyfill 全局 API 时，**载体不得依赖调用时全局查找**——shim 初始化即捕获原生引用。
- 暴露 Web API 构造器不能只看「页面用到才补」：全局可调用性常被 polyfill 当作
  **特性检测信号**，缺失会触发站点侧降级路径（本例：wrapped Promise），其与引擎
  差异的组合效应远超构造器本身。
- V8 release 崩溃静默（`g_hard_abort` int3）：先查 dmesg trap 行拿 ip+offset，core
  dump 的 PT_LOAD 段可直接读栈内存做符号化——不依赖 gdb。
- `--prof` 的 code-creation 事件同一地址会被多次覆盖，活跃函数取**最高时间戳**事件。
