# R384 — M5 V8 `native_dom` default-on land + A/B 守门（flip 抓出的 AbortSignal realm 修复）

**日期**: 2026-08-31（工作跨 08-30 深夜窗口）
**执行序**: zero-web ⚡ 块（2026-08-19 批复）步骤② flip + 步骤③ A/B——R383 步骤① 三门已闭合
**改动面**: `crates/webview/src/webview.rs`（flip + 代际守卫）+ `crates/engine/src/dom_bindings/{mod,factories,dom_exception}.rs`
+ `crates/engine/src/js_dom_shim/{part02,part03}.js` + `crates/webview/src/tests/coverage.rs`（断言分域 + 新 realm 断言面）

---

## 1. flip（步骤②）

`WebViewConfig::default()` 的 `native_dom` 在 **v8 分支默认 `true`**（cfg 双分支与 M7 对偶；
quickjs-only 分支 R382 已 `true`）。**双页面引擎路径 default-on 至此全部落地**（DC-1 第一项）。

随 flip 落地的四件 M5 flip 暴露面修复（全部 clean-HEAD A/B 归因为 flip 后才可达的路径，
非 native 行为缺陷）：

### 1a. 跨 WebView 线程局部代际守卫（webview.rs + dom_bindings/mod.rs）

M5 flip 后同线程多 WebView 全部 native：V8 模板/身份缓存线程局部（gc.rs）持有的 `Global`
Handle **绑定创建它的 Isolate**——WebView B install 时若缓存还是 WebView A 的 Isolate 产物，
`Local::new(scope, old_global)` 即 "Handle hosted by disposed Isolate" panic（webview
cache_storage/indexed_db_owner 双 WebView 共存测试实测）。

修复：`dom_bindings::state_generation()`（AtomicU64，`reset_native_state` 时递增）+
WebView 侧 `native_state_gen: Option<u64>`（`#[cfg(feature = "v8")]`——QuickJS 绑定无
Isolate 绑定模板缓存，安装即全量重建，字段在 quickjs-only 下不消费）。install 前代际不符 →
先 reset 再装；相符 → 复用（同 WebView 跨 execute 的对象 identity 语义保留，R3107/R3117 断言面）。

### 1b. shim HTMLElement 无条件重申 + native S5b upgrade 委托（part03.js + mod.rs + factories.rs）

- `_zwBuiltNodeChain` 改判「三者是否已由本 shim 先前自建」（以
  `HTMLElement.prototype.__zwShimCtorBridge` 标记为据）——flip 后 native HTMLElement 在
  execute_script 前注册全局，旧判 `!HTMLElement` 恒 false → shim 原型链不建 → 页面 CE/lit 链断。
- shim HTMLElement **无条件重申**全局：native factory（`__zw_native_create_element`）的 CE
  upgrade 经 `Reflect.construct(用户 ctor)` 时 ctor 链的 super() 命中 shim ctor（页面类 extends
  shim HTMLElement）——新增 `__zw_native_upgrade_ffi()` / `__zw_native_element_for_ffi(ffi)`
  两个探针/工厂全局，shim ctor 读在途即取 native 元素对象**返回**（spec derived-ctor：base 返回
  对象成为整条 ctor 链的 this）——native slot 语义与 shim 返回值语义合流。
- factories.rs `try_upgrade_custom_element`：桥返回产物按 `ctor.prototype` 重挂原型——
  `instanceof MyEl` 成立（R3270 断言面），native 模板 accessor 经 internal 拦截面可达不依赖
  JS 原型链（nodeType=1 仍过）。

### 1c. DOMException 原型链接 Error.prototype（dom_exception.rs）

spec WebIDL：DOMException inherits Error（`instanceof Error` 语义）。flip 后
`globalThis.DOMException` = native 构造器，其原型链若不含 Error.prototype，shim reject 路径
`new (globalThis.DOMException || Error)` 产物 `instanceof Error` 恒 false（webview
navigator_registration 断言 "true|true" 的右半失败根因）。与 QuickJS 侧
`DOMException.prototype = Object.create(Error.prototype)` 双引擎对齐。

### 1d. AbortSignal abort 默认 reason 的 realm 修复（part02.js，flip A/B 唯一行为差异）

**现象**：M5 flip 全量 A/B 中，`dom/abort/reason-constructor.html` 在 native-forced 跑
Fail（polyfill-forced 跑 Pass）——`AbortSignal.reason.constructor`（iframe 域）不再 ===
`iframeWin.DOMException`。

**根因**（WebView 探针实证，clean-HEAD vs flip 对照）：
`_zw_abort_signal` / `throwIfAborted` 用**词法作用域** `new DOMException(...)`——flip 后
`globalThis.DOMException` = 原生构造器，但闭包内裸名仍解析到 shim 私有构造器 → reason 是
shim 构造器实例，`reason.constructor === w.DOMException`（= 全局 native 构造器）恒 false。
R9（createElement/PI 校验路径）+ R382（SW register reject）同族「wrong-global」第三例。

**修复**：两处改 `new (globalThis.DOMException || DOMException)(...)`。修复后该用例
双路径 Pass；新增 webview 单测 `test_abort_signal_reason_realm_identity_r384` 锁定 realm 语义
（`reason.constructor === iframeWin.DOMException` + `instanceof` + name=AbortError）。

## 2. A/B 守门（步骤③）

| 门 | flip 前（R383 步骤① 基线） | flip 后 | 判定 |
|----|---------------------------|---------|------|
| webview 全量（v8） | 658P/0F | **659P/0F**（+1 = 新增 realm 断言面） | ✅ |
| webview 全量（quickjs） | 611P/0F | **612P/0F**（+1 同上，quickjs 矩阵也跑该测试） | ✅ |
| engine（v8 / quickjs） | 2504 / 1484 | **2511 / 1484**（v8 +7 = dom_bindings 新探针面单测） | ✅ |
| integration（v8 / quickjs） | 781 / 763 | **781 / 763**（vue 3/3 + lit 双 feature 全绿） | ✅ |
| testharness-dom 全量 sweep（polyfill-forced 对照跑） | 55807P/12F/16T | **55807P/12F/16T**——Fail 集合与 R383 基线**逐项恒等** | ✅ |
| testharness-dom 全量 sweep（ZW_NATIVE_DOM=1 native-forced） | — | **55808P/12F**——修复后 Fail 集合与 polyfill-forced **逐项恒等**（reason-constructor 双路径转 Pass） | ✅ |
| product-smoke | 23.37%（既存 oracle 慢性项） | **23.37% 逐字节同值**（112153/480000 px 与渲染流 R3830-F 同值）；struct-check 全 PASS（welcome@800/@375/@320 + article@800/@375/@320 + form-input-perf PASS） | ✅（oracle diff 归 rendering-compat） |
| clippy（v8 + quickjs 双矩阵 `-D warnings`） | 干净 | **干净** | ✅ |
| fmt | 无 diff | **无 diff** | ✅ |

- **Fail 12 项 = 既有已知集合恒等**（R380 定档）：MutationObserver-document 3F（parse-time
  架构域）、remove-and-adopt-thcrash（window.open）、click-on-absolute-pseudo（Chromium 专有）、
  Range-mutations dataChange/replaceData 2F（R353 游离树堆积域）、historical 3F（stale spec）、
  window-extends-event-target 2F（EventTarget 继承域）。**flip 零新增**。
- **Timeout 轮转族**（16±3 个文件级 Timeout：handler-count 变体 / Node-parentNode /
  insertBefore-iframe-crash / slot-recalc 等轮转）为 master.md R331/R355 多轮记录的并发噪声族——
  单跑复测全 Pass（handler-count 6/6、crash 3/3、shadow-host 2/2 双路径复验），非 flip 回归。

## 3. 教训

1. **flip A/B 的行为差异集中暴露在「全局名 vs 词法名」分叉面**——R9/R382/R384 三例同根：
   shim 内部抛错点用裸 `new DOMException`，flip 后全局名易主即 wrong-global。剩余裸名点
   （part02 crypto 族 reject 等）走的是「构造器可达性」而非 realm 断言面，未观察到失败；
   后续若 flip 后 A/B 出现 identity 差异优先按此模式排查。
2. **线程局部引擎状态跨 Isolate 复用是 default-on 的隐藏地雷**——default-off 时代每 WebView
   独占一次 install/生命周期，flip 后多 WebView 共存即触发；代际守卫是通用解。
3. product-smoke 的 welcome oracle 23.37% 与渲染流逐字节同值（md5 同 diff 计数）——结构门
   全 PASS 证明 flip 零渲染影响；oracle 慢性项归 rendering-compat 流（ZRG hmtx 族）。

## 4. M5 剩余

- kill-switch 删除单片（R382 勘误的耦合时序：env + 回退死代码 +
  `install_dom_bindings_if_enabled`，双 feature 全量回归守门）→ DC-1 第二项闭合。
- bench-gate flip 后对照跑（net≥0，真空窗法）。
