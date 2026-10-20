# M2 片 d — 生命周期时序（the-end 4/4 + DCL-defer 1/1）

**日期**: 2026-10-02 | **对应缺口**: P2 生命周期片（the-end 4 + DCL-defer 1 = +5）

## 根因与修法（runner 生命周期尾 + shim 派发语义）

runner 尾部生命周期派发（testharness.rs 生命周期尾脚本）旧为**同步裸 Event**：
DCL 无 bubbles、load/pageshow 无 target=document 语义、pageshow 靠 shim 首监听钩子
自派发（早于 load，序断言必失败）、DCL 同步派发（defer 脚本已入队 timer 任务
未跑，DCL-defer 时序必失败）。

- **时序**：DCL→load→pageshow 收进**单 timer 任务**严格序——DCL 异步入队使 defer
  脚本的已入队任务先跑（spec the-end 步骤：DCL 任务后于 defer 脚本任务入队）。
- **语义**：DCL `bubbles:true`；load/pageshow 经 window 通道派发但 target=document
  （spec the-end：Window load/pageshow 的 target 是 Document）——shim
  `_dispatchWithBubble` 增 `_zwTargetOverride` 通道（头部写、逐站 adjusted-target
  写、doc/win 站写、R114 post-dispatch 写**四处**全部门控——探针逐点实证，漏一处
  即被吞）；pageshow `bubbles+cancelable:true`（whatwg#6794）+ 原型接
  PageTransitionEvent + `persisted:false`。
- **原型标记**：`Event.prototype[Symbol.toStringTag]='Event'` +
  `_defineEventSubclass` 子类原型统一 toStringTag（assert_class_string 面，浏览器
  一致行为）。
- **钩子让位**：runner 预热 execute 置 `__zwPageShowRunnerOwned`——shim
  首监听 pageshow 自派发钩子在 runner 上下文让位（防早发/双发）；非 runner 上下文
  钩子语义不变。

## 结果（全通道复跑逐案对账，分母恒 113 案）

| 案 | 修复前 | 修复后 |
|---|---|---|
| the-end | 0/4 | **4/4** |
| DOMContentLoaded-defer | 0/1 | **1/1** |
| **全通道** | 46185/61304 = 75.34% | **46190/61304 = 75.35%** |

回归：**0**。

## 门禁

make test 68 suites 全绿 + fmt/clippy 干净；探针案（zzz-probe/zzz-probe2，已删）
逐点实证 override 链（impl 路由/分支命中/target 保持/原型标记）。

## 记账

- **bench-gate 挂账**：本轮两窗被兄弟流 clone（ZeroWeb-2）的 zero_webview 长测
  进程（170-421% CPU 持续 6 分钟+）污染——paint_simple/first_paint_wall 系指标
  发丝到 36% 级超限与机器态相关，代码改动（JS 派发分支）无热路径面。等让后复评。
- make test 栈加固（32MiB）延续生效。
