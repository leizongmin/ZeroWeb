# constructor residual 收敛——Event 接口 eventPhase 常量 native 补装（2026-10-19，收口后切片 2）

TBD-3 收敛后移交面最后一红（constructor.html `Event.NONE`）推进至全绿。
实施期修正记账归因的**修复面定位**：残留归因「dom/events 域」正确，但缺装面
不在 JS shim——页面 realm 的 `globalThis.Event` 经 `register_ctor`
（`dom_bindings/event.rs`）被原生构造器覆盖（R384 原生绑定唯一生产路径），
shim part05.js R23 已装的常量（接口对象静态 + 原型，js-dom M4）在生产页面
不生效。真缺口 = native `Event` FunctionTemplate 从未挂 eventPhase 常量。

## 1. 根因链

红案断言（pointerlock/constructor.html:21）：`assert_equals(ev.eventPhase,
Event.NONE)`——期望值 `Event.NONE` 读 undefined（红案报文「expected
(undefined) undefined but got (number) 0」：eventPhase 实例值 0 正确，常量缺）。
bridge 测试 part07.rs R23 在 shim 环境全绿（`generate_js_dom_shim` 直装），
runner 页面经 `run_testharness_html` → `WebView` 原生管线，两 realm 的
`Event` 非同一对象——shim 常量先装、native 注册后覆盖，静态丢失。

## 2. 修复面（2 文件）

1. **`crates/engine/src/dom_bindings/event.rs`**：`attach_event_constants`
   ——`NONE=0`/`CAPTURING_PHASE=1`/`AT_TARGET=2`/`BUBBLING_PHASE=3` 挂接口
   对象（`Event.NONE` 静态）+ `Event.prototype`（实例/createEvent 产物/子类
   实例经原型链可达）。spec：DOM `Event`
   https://dom.spec.whatwg.org/#interface-event + WebIDL 常量暴露
   https://webidl.spec.whatwg.org/#es-constants（数据属性，V8 `set` 缺省
   writable/enumerable/configurable 与之一致）。常量只挂定义接口（Event），
   子类接口对象无自有常量；派发期数值与常量定义一致（`event_target` 覆写
   1/2/3）。注册时机 realm 一次，无逐事件成本。
2. **`crates/engine/src/dom_bindings/tests.rs`**：钉
   `native_event_phase_constants_interface_and_prototype`（三断言面：WPT
   断言形态镜像 `new Event().eventPhase===Event.NONE` / 接口对象+prototype+
   实例三面 `0,1,2,3` / 派发期 `eventPhase===Event.AT_TARGET`）。

JS shim（part05.js R23，enumerable:false shim 侧 R10 for-in 约定）不动——
两 realm 各自成立；enumerable 差异为域内既有约定，本切片不扩散。

## 3. 实跑验证

- `make testharness-pointerlock`：**22/22 子测试 Pass（100%），文件 18/18
  全绿**——constructor 两案翻绿，通道首次零红。
- dom 通道 Event 面（`make testharness-dom FILTER=Event`，stash 前后同过滤
  A/B）：**251 → 230 fail，零新增红，21 subtest 翻绿**——Event-constants ×4
  （同源 WPT 面）/Event-dispatch-bubbles-true|false ×10/dispatch-handlers-
  changed/dispatch-omitted-capture/dispatch-order-at-target/dispatch-target-
  moved/dispatch-target-removed/EventListenerOptions-capture ×2（均以
  `eventPhase === Event.*` 常量断言或过滤）。timestamp 族逐条对照为 5µs 量化
  边界既有抖动（前后两轮 Mouse/Keyboard 恒红，Wheel/Gamepad 各翻一边），非
  本切片影响。
- uievents wheel 族（`make testharness-uievents FILTER=wheel`）**3/3 Pass**
  零回归。
- `cargo fmt --check` 清；`make test` 全腿绿（含 clippy `-D warnings`）；
  `make reftest` 712/712 零回归。

## 4. 账本校正（文件口径 off-by-one）

Commit 4 起文件口径少记 1：「wheel×2」为同一文件
（pointerlock_wheel_event_outside_target.html）的两个 subtest，被按两文件
计入红案文件数。实跑 runner 输出复核（本轮两轮 18 文件逐条清点 + 上轮 JSON
复账一致）：Commit 4 实为 15/18、Commit 5 实为 16/18、TBD-3 实为 17/18。
历史记录（本文件同级 Commit 4/5 证据、RFC v1.4–v1.6 changelog、CSV 行
178/179）保留原样，以本节为准；方向保守未影响任何决策。

## 5. 残留挂账（收敛后）

- **goal 通道零红**：constructor/shadow/wheel 三族挂账全部关闭。
- 桌面锁定 wheel 转发——renderer 无 DOM wheel 派发底座，接入时语义面已就绪
  （v1.4 挂账不变）。
- TBD-4 OS 光标隐藏/显示——平台窗口面远期池。
- 桌面端 GUI 手验（真窗口锁光标观感）——WSL2 环境受限，挂账桌面域。
- 实例域锁的 `__zw_pointer_lock_state_changed` 镜像 payload `selector:null`
  ——host 侧转发按 selector 命中收不到锁定输入流；WPT 面不触达。
- 附带发现（域外转交）：`tests/wpt-runner/src/testharness.rs` 每用例写
  `/tmp/zw-hb.txt` 调试残留（commit 216a6c7d8，media 流域），不属本 goal，
  未改动。
