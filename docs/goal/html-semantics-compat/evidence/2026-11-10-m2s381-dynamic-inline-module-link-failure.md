# M2 切片 381（s381）——instantiation-error-8 翻绿（动态内联模块链接失败单列报告面，五态契约）+ Fix-1 时序假设证伪回退

日期：2026-11-10（承 s380 挂账：基建面 T=3+probe8/9/10 转正决策，或渲染流消化后全域 join；本轮取基建面可施工项）
域：html/semantics/scripting-1/the-script-element/module（instantiation-error-8，域 T 3→2）
性质：**施工轮——ie-8 文件级 Timeout→Pass（真案 -1T），engine shim + webview 双端同刀；script_gen.rs 证伪回退零净代码变化（只留勘根注记）**

## 案面与根因

**instantiation-error-8**：两相 promise_test。相 1 静态 `<script type=module src=instantiation-error-1.js>`（self-import 无导出 → 链接失败），window error 断言 `e.error.constructor === SyntaxError`。相 2 `document.head.appendChild(inline module: import "./instantiation-error-1.js")` 重导入已失败树，期待第二发 SyntaxError（"shouldn't crash" 语义）。红态：文件级 Timeout——相 2 的 once-listener `window.addEventListener('error', ..., {once:true})` 永不触发。

**根因=动态内联 module 报告面缺链接失败类**：webview `__zw_compile_inline_module`（s250 四态契约）把 `compile_dependency_iife` 的 `CompileError`（`ensure_module_export` 缺导出/星导出歧义，es_module.rs:2109/2112）折叠进空态 `""` 按网络错误处理 → 元素 fire error（不冒泡，s102 静态面同语义）→ window 无报告。spec 语义：fetch 成功后的失败类（compile/link 失败）走 window error 报告、元素照常 fire load；仅网络级取回失败才元素 error——静态面 s102 已同构，动态面缺同判。

## 施工（Fix-2 五态契约，双端）

1. **webview.rs**：`__zw_compile_inline_module` 增 `Err(ScriptError::CompileError(msg))` 臂 → 态 `"3\u001f<msg>"`。分界核实：网络/blocked/缺记录键失败走 `RuntimeError`（es_module.rs:2101 Module not found、4068 blocked null entry、4083 gone.js null entry）维持空态元素 error 不变；`InvalidInput`（空源）同样落 `Err(_)` 空态。
2. **part03.js**：动态内联 module prepare 增 tag-`'3'` 臂——`_zwReportListenerError(new SyntaxError(msg), null)` + `setTimeout` 元素 load（与态 `'2'` 属性违例臂同构）；契约注释四态→五态。

## Fix-1 证伪记录（勘根主课）

- **假设**：动态模块失败报告早于 promise_test 体/once-listener 注册 → 报告丢失。首版修复=script_gen.rs 两个静态面报告生成器（`script_report_module_failure`{`,_cached`}）改 deferred setTimeout 一拍派发。
- **A/B 三案 P→F**：error-type-1、instantiation-error-6/7。deferred 报告落到全部模块体求值之后，破坏文档序交错——ie-6 期望 `[exn,1,"6c","6d","6b",2]` 实得 `["6c","6d","6b",2,SyntaxError,1]`；error-type-1 log[0] 变 Number 2（sync onerror 挪位）。
- **双二进制同探针（zz_probe_s380h）证伪**：旧二进制已出 `p1body → err → p1-catch` 正确序——s342 桶 deferred 单元的失败报告在 batch 尾槽位派发（post-parse 相位，晚于全部 blocking classic），promise_test 体微任务（testharness.js:736 `Promise.resolve().then(body)`）先于尾槽。**时序本就 spec 同构；「报告过早」假设不成立**。ie-8 旧红完全由缺报告（第二相根因）造成。
- **处置**：Fix-1 全回退（script_gen.rs 只留两处勘根注记 doc comment），保留 Fix-2。**教训：修报告面先双二进制探针定相位再动手；报告「到达早」与「可达」是两个失败面，本案只缺后者。**

## dynamic-imports-script-error 漂移归因（非本刀，记账不修）

全域 sweep 唯一新见真案 Fail 行（`import() must reject with the same error object for each import when there is a parse error`）。stash A/B：基线二进制同键同态 Fail → 平行流既有漂移，归因 script-sandbox test262 commits `42ab0ee46`/`6d4dbc700`（s369 后落地）。组合态漂移归 script-sandbox/渲染流面处理，本流不修。

## 域口径变化

- the-script-element 域：**T 3→2**（剩 credentials-setTimeout + script-crossorigin-network，均 cookies.py/www2/cross-origin.py 重基建，非本流可施工面）
- 全域 sweep（FILTER=scripting-1/the-script-element，帧驱动 runner）：1132 Pass；红=2T（credentials/crossorigin）+1F（dynamic-imports-script-error 既有）+探针行（勘根毕删）
- A/B join（(key,state) 口径）：A-only 3 键=ie-8 文件级 Timeout + 探针 Timeout×2；B-only 7 键=ie-8 子测试 Pass + 探针 6 键；零真案 P→F、零改名。ie-8 为文件级 T→子测试级 Pass 键迁移（s368 判读口径人工合并）

## probe8/9/10 转正执行（s364 挂账收口）

**决策：转正（git add -f 入跟踪，不改名）**。s364 定性=行为钉非恒红（断言时序序列，本地 Pass），转正挂"低优先"；本轮漂移事件（dynamic-imports-script-error 实证平行流 commit 在动 module 面）补齐决策依据——本地钉在跨 clone/CI 不生效，恰是漂移暴露面。机制核实：wpt-data 整目录 gitignore（.gitignore:85），但主仓已有 42 文件 pinned fixture 先例（cache-storage-window-root/）；转正后 update-wpt-data.sh 重建（脚本 ：267 rm -rf）会删 pinned 文件，git status 报 deletion → `git checkout` 还原（s109 记忆口径）。7 文件：probe8-order.html+probe8-d2/d4.js、probe9-async.html+probe9-a1/d4.js、probe10-inserted.html——内容审计无本机路径/泄漏，本刀终扫三钉全绿（probe8-report/probe9-report/probe10 子测试 Pass）。**不改名**：gitignore 是目录级非文件名级（s364「文件名命中」记忆偏差），改名断 sweep 历史可比性。

## 平行流 fmt 漂移（机械修复，单独提交）

make test fmt 门禁前置快速失败：testharness.rs:4917 / testharness_wasm_core.rs:176/256 三处 `dedicated_worker_variant` 调用点格式漂移——引入者=平行流 wasm-compat commit `c181cccab`（worker 变体腿，该流提交流程未跑 fmt 前置）。工作树无该文件在途编辑（并发安全核实），`cargo fmt -p zero-wpt-runner` 机械修复零语义变化，单独 style 提交记账。

## 坑（入册）

1. **LC_ALL=C env 前缀在 pipeline 不穿透后续命令**（fish login shell）——`LC_ALL=C grep … | sort` 只约束 grep，sort/comm 仍按 locale 排序 → comm 假报「文件未排序」。须脚本顶部 `export LC_ALL=C` 一刀切（s287/s292 重证第三踩）。
2. **puppeteer-core 入口路径**=lib/puppeteer/puppeteer-core.js（package.json main），非 lib/esm/puppeteer/puppeteer-core.js。
3. **shim 拼装顺序**必须按 js_dom_bridge.rs:5967-6079 include_str! 顺序，单文件 node --check 假红（Unexpected token）。

## 验证

- instantiation-error-1..8 solo 全绿；error-type-1 / ie-6 / ie-7 回退后恢复基线绿
- shim 22-part 拼装 node --check PASS（Fix-2 终态）
- Chrome 真值探针（puppeteer-core + 本地 18801 静态服务）定相位：模块失败报告时机=插入脚本完成后 task tick（与 ZW 尾槽相位同构，佐公证伪）
- make test（fmt 前置过 + workspace v8 腿）：engine lib **3827P/7F**——7 红（img_runtime_deferred_settle_r5275 / svg_img_settle_density_r5277 / fetch_filtered_response_type_matrix / iframe_sw_messages_dispatch / response_body_readable_stream_r2967 / websocket_error_nonterminal / websocket_event_flow_open_message_close）与 s371 账本基线名单逐名精确同构，3827P 同构 s378（+1P 组合态漂移在案），零新增红；腿中止使后续腿未跑，影响面定向补测：
- zero-webview lib v8 定向：**848P/1F**——1F=`csp_gate::csp_connect_gate_blocks_fetch_sh1_m2s5` s348 在册环境红（image_decoder spawn 红本轮未触发，env 依赖在册），零新增红；script-sandbox 等其余二进制 workspace 腿全绿（134 result 行唯一 FAILED=engine lib）
- v8 clippy（zero-engine + zero-webview `--all-targets -D warnings`）rc=0；`cargo fmt --all -- --check` 净
- **quickjs 测试腿 HEAD 既有编译破坏（平行流，记账不修）**：zero-webview lib 引用 `zero_script_sandbox::v8::PinScope` + `module_parse_gate`（HEAD webview.rs:8517/8592/8625，平行流 commit `42ab0ee46` module parse 预检引入，script-sandbox `pub use v8` 走 v8 feature gate）——`--no-default-features --features quickjs` 下 zero-webview 不编译，quickjs 腿在 HEAD 即不可跑（git grep HEAD 实证非本刀引入；本刀 quickjs 覆盖在该 HEAD 上不可达，v8 腿已覆盖 webview/engine 面，提交说明注明）
- 探针 zz_probe_s380a–h + zz_ext_s380f/g 勘根毕删（wpt-data gitignore 面，无入库残留）

日志 /tmp/（本地：s381-domain-base.raw、s381-domain-new2.raw、base.kv、new2.kv、s381_join.sh、s381-make-test.log；探针源已删）。
