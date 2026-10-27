# M2 切片 110：prepare 家族提升 + force async 语义面（async IDL reflection / FIFO in-order 队列 / SVG src 门）

日期：2026-10-27。范围：`crates/engine/src/js_dom_shim/part01.js`（attributes 臂收敛 + async 属性加标注）+ `part03.js`（s96 家族提升 + 排队分支 + SVG src 门）+ `part04.js`（旧闭包家族删除 + async getter）+ `part05.js`（async IDL setter force async 标注）+ `part06.js`（parser 投递 force async 标注）+ `crates/engine/src/pipeline/extract.rs`（`script_type_is_javascript` pub）+ `tests/wpt-runner/src/testharness.rs`（3 钉 + 内联投递 type 透传 + load 等待面）。上轮：[M2 切片 109](2026-10-27-m2s109-mutation-retrigger.md)。

## 1. 入场定谳

- s110 承 s109 挂账两行：**122-126 marker 面**（createElement 工厂节点外链在 corpus strict 管线的执行死因）与 **122 async IDL reflection**（re-insert 后 `script.async` 反射 true）。
- **bin pin e1 缺席根因 = psp 惰性导出**：s96 prepare 家族原定义在 part04 元素实例闭包内、`globalThis.*` 导出语句在闭包体尾——导出随首个元素实例创建（如 `document.createElement`）才生效。早触发页（bin harness promise_test 于解析期执行，`readyState === 'loading'`）首个 script 插入记录先于任何实例创建到达，`typeof globalThis._zwPrepareSrcScript === 'function'` 守卫静默 no-op。corpus 页 promise_test 于 load 后跑，晚触发从未暴露——引擎沙箱（早触发）与 corpus（晚触发）两世界差异由此而来。
- **spec 定谳（/tmp/spec-scripting.html prepare-the-script-element + dom-script-async）**：async getter 步骤 = force async true → true；async 内容属性在场 → true；否则 false（属性在场返真**即使** force async 已 false）。prepare 步骤序：「parser document 非 null → 复位 parser document 且 force async=false」在 **type 判定之后**——type 非法早退不写，force async 保创建缺省 true（122 形 type=text/plain 外链 re-insert 后 `assert_true(script.async)` 为 current-spec 正确）；type 合法的 parser 投递单元写 false（引擎沙箱静态页 sc2 缺省读 false 面，r3040 现行断言真实浏览器语义）。
- 排队分支：async 内容属性在场或 force async 为真 → 「set of scripts that will execute as soon as possible」（取回即执行，原有行为）；否则「list of scripts that will execute in order as soon as possible」（FIFO 头排水，091/105/122-126 面）。

## 2. 刀

- **家族提升（part03 外层语句域）**：`_zwScriptForevBlocked` / `_zwExecFetchedImpl` / `_zwDrainQueueImpl` / `_zwFetchPendingImpl` / `_zwPrepareSrcScript` / `_zwPrepareSrcScriptGate` 整体从 part04 闭包迁至 part03 语句层（dedent −6），导出随 shim 求值恒在；part04 仅留调用点（R387b MO/trap 臂、insertBefore 臂），经作用域链直呼同名声明。gate 内升级：身份门放宽到 sel 形（重插入 parser 单元只有 `__zwSelector`——旧 handle 恒 bail）；run-once 记账按身份形态分表（handle → `_zwRanScripts`，sel → `_zwScriptStarted`）；排队分支 `_faAsyncQueue`（attr 在场 ∥ force async ≠ false）→ 非即插 `_zwInOrderScriptQueue` FIFO 项（ready/failed 置位后头排水，首元素未就绪即止——spec list 步骤 abort 语义）；`__zwScriptFetchPending` in-flight 计数（runner load 等待读）。
- **force async 标注三点**：part06 parser 投递（随投递非解析插入——type 非法不投递保 true）；part05 `async` IDL setter（步骤 1 恒置 false，value 真亦置）；part01 attributes 臂 async 属性加（spec 独立条款非 prepare 步骤；写后 hasAttribute 区分加/删）。
- **async getter（part04 get trap `_REFLECTED_BOOL` 分支顶部）**：表项 true → true；attr 在场（三族 helper **严格 `=== '1'`**——lw/handle/直读均返 '1'/'0' 串，真值判定把缺席面 '0' 当在场，r3040 sc2 假阳性实证）→ true；表项 false → false；工厂 handle 形 → true（创建缺省）；parser 已 started → true（重触发面，stale 模型不写 false）；parser 当前 type 非法 → true（parse bail 未 prepare）；parser type 合法未 started → false。
- **attributes 臂收敛（part01）**：触发属性 type 移除（spec 触发面无 type 变更条款；122-126 族 `removeAttribute('type')` 不重触发依赖）——剩 src/href/xlink:href 三属性。
- **SVG src 门（part03 `_zwScriptMaybeStartDynamic` 源解析）**：`namespaceURI` 为 SVG ns 的 script 跳过 `getAttribute('src')`（SVG script 取源属性是 href/xlink:href，src 内容属性无效——语料 146 期望零执行）。家族提升揭开了该臂（惰性导出期从未可达），顺手补门非顺手改行为。
- **runner 面**：`inline_local_scripts` 对非 JS type 外链单元整段透传（data-inline 改写剥 src，保 mutation 重触发 src 语义；`extract.rs script_type_is_javascript` 转 pub 复用同判据）；`loadStep5275` 重试条件扩读 `__zwScriptFetchPending`（外链 in-flight 等待）；3 钉（children-changed 外链 / textContent= 外链重触发 / 重插入外链 FIFO `[e1,e2]`——inc 页写 `std::env::temp_dir()` 锚，断言外提 .then）。

## 3. 调试定谳（本轮主发现）

- **getter 假阳性二分**：r3040 sc2.async 读 true 而新 getter 断言 false。探针二分：get trap 顶部 trace 证 `G:async` 可达 → 预门 trace `rb:SCRIPT/lw:0` 证 `_realTag` 门通过、`__zw_has_attr_lw('#sc2','async')` 返 **'0'** → 真值 `if (_faHas)` 把 '0' 当在场 → 严格 `=== '1'` 修复。教训：attr helper 族返 '1'/'0' 串非布尔，generic presence 读 `_rbHit === '1'` 同约定，新增消费方必须同口径。
- **part01 addedNodes `_makeProxy` 预实例化返工**：为 bin pin e1 缺席所加（锚定形先实例化再入臂）经证非根因（真因 = psp 惰性导出），已回退——hoist 单独承重，三套测试绿实证。

## 4. 验证

- 单测：zero-engine lib **3544/3544**；zero-webview lib **814/814**（持平）；zero-wpt-runner bin **236/0**（+3 钉）。
- execution-timing 定向扫：**124P/28F/1T**（s109 基线 110P/40F/1T，+14 绿，零 P→非P）——081/082/091/092/095/097/105/119/122×2/123/125/126 翻绿，118 恢复。行级 diff 基线 /tmp/s110-base-execution-timing.log，本轮 /tmp/s110-execution-timing2.log。
- 全子树终扫（FILTER=scripting-1）：**395P/155F/37T**（s109：380P/170F/37T+15N，**+15 行绿/14 页**，T 持平）。行级归一 diff（/tmp/s110-final2-scripting-1.log 存档，基线 /tmp/s109-final-scripting-1.log）：**零 P→非P**（P109→非P110 连接为空、新增非 P 行为零）——execution-timing **081/082/091/092/095/097/105/119/122×2/123/125/126/130** + moving-between-documents/ordering/**delay-load-event-2**（load 等待面扩读 `__zwScriptFetchPending` 的连带收益）。
- fmt：testharness.rs clean（knife A 注释缩进 rustfmt 归位）；engine 侧 `paint/painter/text.rs:4130` 存量 fmt diff（渲染流文件，未触碰仅提及）。clippy engine+wpt-runner `--all-targets -D warnings` clean；fmt 后 runner bin 复跑 236/0 确认。
- 渲染面零依赖——reftest/product-smoke 未跑。探针遗留：临时 probe 页/inc js（gitignored wpt-data）与临时 probe 测试文件已删，引擎 shim 无残留插桩（grep TEMP/_zwFaTrace/_zwGetTrace 零命中）。

## 5. 挂账

- execution-timing 余面：050/085/086/088/111/112（slow-loading 外链时序 pipe=trickle 建模 ~11 行）、147/148（insert multiple inline scripts——first moves/deletes subsequent 复杂 scheduler 面）、142 write 臂 xlink:href、143 eventOrder 完整核验（承袭）。
- 承袭 s109 挂账：063 Timeout、non-external-no-import、parse-error 族、http-accept-header、referrer-policies、dynamic import() 透传、classic 外链 SRI、collect 账本打包、接口对象基建面、FIXME 组。
- **已知取舍**：nomodule/for-event 门压制单元 parser 投递不标 force async（spec 语义 false 此处保 true）——仅影响其后 mutation 重触发面排队分支归属，不改变执行/不执行结果。
- 待决清单（须用户拍板，不阻塞）：css/text 数据模块语料资源缺失（承袭）。
- 本轮销账：execution-timing 081/082/091/092/095/097/105/119/122/123/125/126/130（13 页）+ moving-between-documents/ordering delay-load-event-2（1 页，连带）。
