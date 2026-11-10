# M2 切片 382（s382）——dynamic-imports-credentials-setTimeout 翻绿（string timer 文档上下文面，域 T 2→1）+ 三层勘根假象剥除

日期：2026-11-10（承 s381 挂账：基建面 T=2 勘根——credentials-setTimeout / script-crossorigin-network 可施工性盘点）
域：html/semantics/scripting-1/the-script-element/module（dynamic-imports-credentials-setTimeout.sub.html）
性质：**施工轮——文件级 Timeout→12 子测全 Pass（真案 -1T），engine shim 双文件四编辑，零 Rust 变化**

## 案面与勘根（三层假象剥除）

案形：顶页 `runTestsFromIframe('../resources/dynamic-import-credentials-setTimeout-iframe.sub.html')`——helper 先双 set-cookie fetch（same-origin + www2 no-cors include）+ window load → iframe.src + `fetch_tests_from_window(iframe.contentWindow)`；iframe 六 script 块（classic/module × 无 crossOrigin/anonymous/use-credentials）各 `runTest(setTimeoutWrapper(x => setTimeout(x,0)), …)` ×2（same+cross）。链路末环 `setTimeoutFunc('import("url").then(window.resolve)')` 字符串定时器。

**假象一（前轮「重基建」定性）证伪**：credentials 全栈已在位——s338 check-cookie.py 仿真 `src.contains("check-cookie.py")` 路径无关（testharness.rs:6177）、set-cookie.py/cookie jar/www2 路由在册、姊妹案 dynamic-imports-credentials.sub.html（direct import，同 helper 同 iframe 形）12/12 绿。基建不是断点。

**假象二「iframe 字符串定时器不开火」证伪**（探针 4/5 队列 at 值 dump）：前轮探针「final pending 卡死 __zwtid:2/9」实为**顶页 harness 的 10s 全局超时定时器**（at≈9929/10000ms，文件 1.2s 终止故天然未消费）；字符串定时器 arm delay=0 完好入队（10@0），fire 循环按 at 升序正常弹出、pending 条目正常消费。定时器机制全程健康。

**假象三「import 挂起」证伪**（探针 8 S2 臂）：绝对 URL import 从编译体发起正常 settle（S2-ok）。早期探针「T1-ok/T1-err 双缺席」表象 = `.then` 回调体内 `dump`/`window.__o` 解析到顶窗（undefined）抛错被 fire 循环吞——import 管线无恙。

**真根因**（探针 8 A/B 对照定谳）：
- A 臂（字符串 timer 编译体）：`window === globalThis` **true**、`window.__zwSent`（iframe 求值窗置位）**undefined**——`new Function(fn)` 编译体在全局作用域求值，裸 `window` 读 `globalThis.window`（=globalThis，part06:7668 顶窗身份），非注册段合并体形参（子窗，R339 `new Function('window','self',…)` 参数绑定）。
- B 臂（函数 timer 闭包）：`window === globalThis` **false**、读子窗 ✓——闭包经作用域链解析合并体形参。姊妹案 direct import 在 promise_test 体内运行（s339 `_zwScopeBody` 换窗）故绿。
- 断裂链：fire 窗编译体 `.then(window.resolve)` → 顶窗无 `resolve` → **`.then(undefined)` 合法直通** → import 成功（swap base/nonce 面正常）但 harness promise 永不 resolve → 12 promise_test 全挂 → 文件级 Timeout。
- credentials 槽同缺陷：fire 窗 `__zwActiveScriptCred` = 段序末尾复位行值（最后一段的 mode），非注册 script 元素 mode——s340 注册窗捕获/体执行窗恢复机制只覆盖 promise_test 体，字符串 timer 臂缺位。

## 施工（s382 string timer 文档上下文面，双文件四编辑）

1. **part01.js setTimeout 字符串臂**（s90/s91/s92 注释组追加 s382）：注册期捕获 `_zwCapCred`（`__zwActiveScriptCred`）+ `_zwCapWin`/`_zwCapDoc`（`__zwActiveIframeWin/Doc`），传入换基 helper 第 6-8 参。
2. **part01.js setInterval 字符串臂**：同捕获同传参（s91/s92 同形）。
3. **part03.js `_zwCallHandlerSwapBase(fn,thisArg,ev,base,nonce,cred,win,doc)`**：cred 非 undefined → 换入 credentials 槽（finally 恢复含缺席 delete）；win 在场 → `globalThis.window/self/document` 三属性换形参化（均普通可写赋值定义，part01:12357/part06:7668）+ s339 双槽置位，finally 全量换回。**inline 处理器臂（`_zwCallHandlerDocBase`，5 参形）新参 undefined → 全臂 no-op 零漂移**。
4. **part01.js 子文档合并体求值窗**：`fn.call(win,…)` 前置 `__zwActiveIframeWin/Doc` 置位 + finally 恢复（与 s339 `_zwReg339Pre` 同形）——子文档**顶层**字符串 timer 的注册期捕获通道（credentials 有 s340 段序前置复位行，window/document 原无等价通道）。

配套闭环核实：动态 import IIFE 包裹在**求值时**读 `__zwActiveIframeWin`（es_module.rs:198-214 s339 桥）——fire swap 置双槽使模块体 `window[id]="found"` 落子窗；promise_test 体断言 `assert_equals(window[id],…)` 在 s339 体窗读同一子窗——链路两端一致。`.then(window.resolve)` 的函数值在 `.then()` 调用时同步求值捕获（swap 窗内），微任务 settle 在 restore 后调用捕获值——不受窗恢复影响。check-cookie.py 仿真路径无关使 base 陈旧对本案不可见（same 臂相对 URL 解析目录无论对错均命中 string-match）。

## 伪证经验（探针方法论，入册）

1. **同代码两窗行为不同**：0ms（合并体求值窗）dump 正常、~30ms（fire 窗）dump 静默失败——插桩先辨窗再断言「不开火/挂起」。
2. **队列「卡死键」先查 at 值**：全局 harness 长延迟监视定时器（10s）在短文件里天然不消费，pending 键存活≠泄漏。
3. **回调体名解析面隔离**：探针回调只用 `globalThis.` 直引 + 绝对 URL import，排除裸名解析干扰后再断言机制面。
4. 前轮「重基建」定性在勘根前即写入控制文档挂账——本轮证伪。定性措辞应区分「已知依赖面」与「已验证断点」。

## 域口径变化

- the-script-element 域：**T 2→1**（剩 script-crossorigin-network，cross-origin.py 语料资产缺失 + CORS 强制面状态未勘，下轮候选）
- 域 A/B join（stash 二进制 A vs 修复 B，(key,state) 口径，探针毕删后双跑）：**A=1130P/1F/2T（1137 键）→ B=1142P/1F/1T（1148 键）**；A-only 恰 1 键 = credentials-setTimeout 文件级 Timeout 旧形态，B-only 恰 12 键 = 新子测 Pass，**零 STATE-DIFF**（probe8/9/10 转正钉全保持 Pass，s381 在册 1F=dynamic-imports-script-error 平行流漂移原样）
- 语料曝光面：semantics 全树字面字符串定时器（`setTimeout("…")` 直引形）0 文件；变量传递形 = credentials 族（本刀翻绿）+ probe8/9/10 钉（无位移）。engine/webview 单测面由 make test 覆盖。

## 验证

- solo（FILTER=dynamic-imports-credentials）：credentials-setTimeout **12/12 Pass**（Timeout→Pass 首跑即中）+ 姊妹案 12/12 Pass 保持
- shim 22-part 拼装 node --check PASS（编辑后终态）
- 域 A/B join 见上节（零真案 P→F、零状态翻转）
- make test（fmt 前置过 + workspace v8 腿，guard --compile-first）：聚合 **9397P/7F**——7 红（img_runtime_deferred_settle_r5275 / svg_img_settle_density_r5277 / fetch_filtered_response_type_matrix / iframe_sw_messages_dispatch / response_body_readable_stream_r2967 / websocket_error_nonterminal / websocket_event_flow_open_message_close）与 s371 账本基线名单逐名精确同构，零新增红；腿中止形态与 s381 同（后续腿未达），本刀零 Rust 变化、engine shim 面由 engine lib（js_dom timer/bridge 单测群）覆盖
- clippy/fmt：零 Rust 变化，例行核验过门禁
- 探针 zz_probe_s382a/b/c + zz_probe_s382_ifr + zz_ext_s382 勘根毕删（gitignore 面，无入库残留）

日志 /tmp/s382/（domain-base.raw、domain-new.raw、a.kv、b.kv、join.sh、make-test.log；探针源已删）。
