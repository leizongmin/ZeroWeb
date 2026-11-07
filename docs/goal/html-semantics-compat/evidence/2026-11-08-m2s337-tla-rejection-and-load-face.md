# M2-S337：evaluation-order-4 TLA 收口——rejection 上报桥 + async 入口 load 检查点 gen1 位次（承 s336）

日期：2026-11-08。控制文档：`docs/goal/html-semantics-compat.md`（L5 链 s337 段）。
本切片收口 s336 挂账的 `evaluation-order-4`（TLA `await import()`）Timeout 三子面中的
两子面（第三子面「依赖 F4」已由 s336 动态 import 异步求值跳收口）。

## 定谳

### F1：TLA rejection 上报通路（双层包裹吞拒绝）

async 入口模块求值失败的**唯一**错误通路是 async IIFE promise 拒绝（`await` 掷与
体同步 throw 同型归一为 promise rejection）——宿主包装 try/catch（webview s255 面）
接不到 promise 拒绝。旧 `compile_module_script` async 臂在 `build_module_script`
**已发射的完整 async IIFE**（尾 `})()`）外再包一层
`(async function() { <IIFE> })()`：内层 promise 沦为语句求值即弃，外层以
`undefined` resolve——rejection 永不上浮 → `window.onerror` 不触达 →
runner 面 `window.onerror = testDone` 永不执行 → 文件级 Timeout
（corpus 实测序对全对而 reportError 缺席）。

spec：run a module script 求值 rejection → report the exception
（window error 事件，`ev.error` 为原值拒绝理由）。
https://html.spec.whatwg.org/multipage/webappapis.html#run-a-module-script

### F2：async 入口 load 投递位次（体尾/sync 端双面皆错位）

corpus 判据（`evaluation-order-4.html`）：`script-load` 须落
`microtask-4.1` 之后、`step-4.2-1` 之前。两面皆不达：

- **runner 体尾片**（s255 形，`inline_local_scripts` 附加）：位于 async 体
  `await` 之后——拒绝路径体不执行到尾，永不可达；TLA 成功路径又晚于
  microtask-4.1 排空前位。
- **引擎 s107 sync 端臂**（post-execute 派发）：`sandbox.execute` 返回时检查点已
  排空，派发恒迟到（evaluation-order-4 红态：全序对而无 script-load）。

spec：load 属 fetch 完成语义、模块求值属后续 job——load 在依赖体首语句之前。
https://html.spec.whatwg.org/multipage/webappapis.html#fetch-a-single-module-script

### F3：动态 import 单跳同代竞争

s336 单跳 `Promise.resolve().then(() => __zw_load_module(...))` 下，取回体与
（新设的）load 派发**同代排队**——同步块排队序先于 load 尾片，依赖体反先于
load 求值（Chrome 取回跨宏任务，body 落次代）。双跳后 hop1 空代让位 load
派发，依赖体落次代；order-3 序不变（hop1 无日志，依赖体本就后于 mt-3.1）。

## 施工

| 面 | 位置 | 内容 |
|---|---|---|
| F1 | `es_module.rs` `compile_module_script` async 臂 | 去双层包裹：`body.trim_end()` 直附调用尾 `.catch(function (__zwModTlaErr) { try { if (typeof reportError === 'function') reportError(__zwModTlaErr); } catch (_zwTlaRpt) {} })`——非页面宿主无 shim reportError 守卫空转；不设 `__zwModEvalErrObj` 哨兵（异步置位会陈滞滞留并被下一模块元素误报，s255 哨兵语义不适配异步面）；元素 load 不由本桥补发 |
| F3 | `es_module.rs` `__zw_dynamic_import` | 单跳改双跳 `Promise.resolve().then(function () { return Promise.resolve().then(function () { return Promise.resolve(__zw_load_module(...)); }); })`——取回/缓存/失败记账/错误身份零漂移 |
| F2 引擎 | `webview.rs` module 执行面 | 新 execute 前 gen1 load 臂：`is_module && inlined_from_external && async_module_face`（谓词 `code.contains("await") \|\| code.contains("import(")` 与 es_module `async_body` 同源）→ `full.push_str("\nPromise.resolve().then(function () { <script_dispatch_script_event_by_ordinal(script_index, "load")> });")`——sync 端入队即检查点 gen1 位次（mt-4.1 后、依赖体前）；TLA 成功路径亦由本面单发 |
| F2 引擎防双发 | `webview.rs` s107 sync 端臂 | 限定 `!(inlined_from_external && async_module_face)` 跳过——**浏览器道外链 async 模块维持原臂零漂移**（外链无 gen1 臂，裸 `!async_module_face` 会令其 load 缺席，本切片修订）；真作者内联（无 data-inline）不发射，load-error-events-inline 判据不动 |
| F2 runner | `testharness.rs` `inline_local_scripts` | async module 单元（同谓词）省 load 尾片——引擎 gen1 臂单独承担；error 尾片（网络失败面）不动 |

## 验证

- **契约**：script-sandbox `test_module_tla_rejection_reported_to_window_s337`——
  全序列钉 `LOG:step-4.1-1,step-4.1-2,microtask-4.1,step-4.2-1,step-4.2-2,microtask-4.2`
  + `ERR:error`（rejection 经 catch 桥 reportError 恰一次、`step-4.1-3` 不执行）+
  宿主取回恰一次。sandbox lib **402P/0F**、webview lib **844P/1F**（红 =
  `proxy_mode_decodes_png_correctly` image-decoder 进程环境红，s168c/s256 两账在案
  HEAD 同挂）、runner bins **290P/0F**、engine lib **3807P/6F**（红集 = 已知六红精确，
  零新增；prelude 双跳零漂移）。
- **microtasks 域**（`logs/s337-microtasks-after.log`）：11 face = **11P/0F/0T**——
  s336 基线 10P/1T；order-4 Timeout→Pass，余 10 面 Pass 保持，P→F 零。
- **the-script-element 域 A/B**：`s337-script-element-after.log`
  **1058P/54F/11T** vs `s336-script-element-after.log` 1057P/54F/12T——
  **+1P/-1T，F 平，P→F 零**（file 级 comm：GAINED 仅
  `microtasks/evaluation-order-4.html`；subtest 级 comm：文件级 Timeout →
  真子测 `Module script queueing a microtask then top-level await` Pass）。
- **clippy/fmt**：`-D warnings` 三 crate（script-sandbox/webview/wpt-runner）全绿；
  rustfmt 全绿。

## 挂账

- **s338 候选**（域余 54F/11T 分诊，`module/dynamic-import` 簇 17 红为最大簇）：
  `dynamic-imports-credentials.sub.html` 10 子测（credentials mode 动态 import
  透传面）+ TLA deferred-evaluation 族（`tla-deferred-evaluation.tentative.html` /
  `cyclic-tla-defer-eval.tentative.html` / `basic-deferred-evaluation.tentative.html`）
  + `code-cache-base-url.html` Timeout。
- `window.onerror = fn` 直接赋值后 spec 要求的 per-fire `window.event` 语义偏差
  （承 s336，F2 施工维持既有面未扩大）。

## 证据索引

- `logs/s337-microtasks-after.log`——microtasks 域终态（11P/0T）
- `logs/s337-script-element-after.log`——the-script-element 域全域终态（1058P/54F/11T）
- `logs/s337-engine-lib.log`——engine lib 全量（3807P/6F 已知红集精确）
