# R147 — strict classic 脚本的全局函数发布（webkit-animation/transition 4 文件整页 error → subtest Pass）

**日期**: 2026-08-21
**里程碑**: M4（WPT dom 上游基线扩展）
**驱动用例**: `dom/events/webkit-animation-{end,iteration,start}-event.html` +
`dom/events/webkit-transition-end-event.html`（各 13 subtest）

## 根因

`script_run_classic_page`（script_gen.rs）经间接 `(0,eval)(...)` 执行 classic 页面
脚本（persistent_context 模式的既有机制，R3258 currentScript + try-catch sentinel）。
**源内 `'use strict'`/"use strict" 指令使 eval 建独立变量环境——顶层 `function`
声明不落 globalThis**。真实浏览器 classic 脚本即便 strict 也创建全局绑定（script
是 global code 非 eval code，strict 只影响赋值/this 语义不改变顶层绑定位置）。

WPT 外链测试库（`dom/events/resources/prefixed-animation-event-tests.js` 顶层
`function runAnimationEventTests` / `createDiv` / `addTestScopedEventHandler` 等）
经 `inline_local_scripts` 内联后执行——声明困在 eval 环境内，用例脚本的
`runAnimationEventTests({...})` 跨 `<script>` 不可见 → "is not defined" 整页 error
（每文件 1 行 error，13 subtest 全灭）。

## 修复（script_gen.rs 单点）

eval 源尾拼接 `;globalThis.NAME=NAME;`（字符串拼接 `'源' + ';globalThis.x=x;'`——
拼接在两侧，不改 `'use strict'` 须为源首语句的语义）：

- **启发式扫描**：仅**行首零缩进**的 `function NAME(`——真顶层声明（WPT 测试库顶层
  函数无缩进）；IIFE 内部缩进函数不误匹配（testharness.js 全 IIFE 包裹源零导出；
  初版 trim_start 版曾使 harness 内部函数名 eval 报 ReferenceError，零缩进锚定修复）。
- strict 局部声明经此导出；non-strict 本已全局，赋值恒等。
- 误报上限：字符串/注释内的伪匹配最坏多发布一个无害 globalThis 赋值。

## 系统性收益

不止 webkit 簇——**所有 strict 声明顶层函数的跨脚本调用**修复（用例自身
`<script>` 内 `'use strict'` + `function helper()` 后续脚本引用的形态全系受益）。

## A/B 结果（polyfill / native 双路径）

| 套件 | 结果 |
|---|---|
| webkit 4 文件 | 每件 4/13 subtest Pass（"not aliases" + dispatchEvent 族）+ 1 Timeout 行（9 个 promise_test 待真动画事件）；旧为整页 error 1 行 |
| dom/events 全量 | polyfill **438P/17F/10T**（vs R146 421P/21F：**+17P/-4F**）；native **438P/17F/10T fail 集逐文件一致** |
| dom/nodes 全量 | 5473P/230F/9T——fail 集零新增（node-appendchild-crash 隔离复跑 Pass 系套件负载 flake；query-target-in-load-event.part flake 消失） |
| traversal / collections | 50P / 0F |
| `make test` | 66 套件全绿 |
| fmt / clippy | 零 diff / 零警告（v8 + quickjs 双矩阵） |

## 单元测试（part21.rs 追加）

`test_classic_script_strict_function_globals_r147`：strict 单/双引号两形态顶层
函数跨脚本可见 + IIFE 内缩进函数不误发布（`__innerRef=function`）+ 返回值计算
（topFnA()+topFnB()=3）+ sentinel 干净四段断言。

## 残留（记入下一步）

webkit 4 文件各 9 个 promise_test 等待**真 CSS 动画事件**（`div.style.animation =
'anim 1ms'` + `animationstart/end` + rAF）——runner 无动画时钟 pump
（`render_html_with_animation` 不进 probe 循环，pipeline 的
`take_pending_animation_events` 无消费者）。runner 架构项，记档不追本轮。
