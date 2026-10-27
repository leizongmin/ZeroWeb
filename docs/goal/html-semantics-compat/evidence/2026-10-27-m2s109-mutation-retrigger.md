# M2 切片 109：script mutation 重触发 prepare（触发臂 + 门序分派路由）

日期：2026-10-27。范围：`crates/engine/src/js_dom_shim/part01.js`（_mo_notify 三触发臂）+ `part03.js`（_zwScriptMaybeStartDynamic 门序分派）+ `tests/wpt-runner/src/testharness.rs`（3 回归钉）。上轮：[M2 切片 108](2026-10-27-m2s108-strict-continue-and-id-fidelity.md)。

## 1. 入场定谳

- s109 承 s108 主候选：mutation 重触发机制面七行（122 第二层 async IDL+时序、123-126、144/145）——script 元素运行期 type/text/插入变更须按 spec「prepare the script element」重触发。
- **Rust 侧嵌套执行证伪**：script-sandbox 回调签名 `Fn(&[String]) -> String + Send + Sync` 不持有 sandbox，`execute` 需 `&mut self`——回调内无法嵌套执行脚本。可行路径 = JS shim 层 indirect `(0, eval)`（同上下文、调用点同步）。
- 既有件盘点：`_zwSvgScriptMaybeStart`（part03:3725，SVG script 外链动态 prepare，仅 SVG NS）；`_mo_notify`（part01:6019，shim 全部 attr/childList 记录单一汇流点——part04 proxy 64 处 + part03 工厂 13 处皆经此）；s95 `text` getter（child text content）；`_zwPrepareSrcScript`（s96 外链单点 helper：run-once/nomodule/for-event/宿主让位门 + 页面 fetch + load/error 派发）；R387/s95 run-once 记账表（`_zwScriptStarted` sel 形 / `_zwRanScripts` handle 形）。
- 104/114/115 的通过机制（runner 装配把 parser 外链平铺为内联单元）与 mutation 族（type=text/plain 未投递单元）本质不同，不可复用。

## 2. 刀

- **触发臂（part01 `_mo_notify` 头部，attributes/childList 记录源三臂）**：①attributes@SCRIPT 且属性名为 type/src/href/xlink:href；②childList 目标为 SCRIPT（textContent=/.text setter/子内容变更）；③childList addedNodes 含 SCRIPT（插入面安全网——与 appendChild trap R387/R387b 同表互斥）。均以 proxy 反查身份键（`__zwSelector`/`__zwHandle`）+ `_realTag` 门。
- **门序分派路由（part03 `_zwScriptMaybeStartDynamic`）**——不自带执行器：connected（spec children-changed 步骤 1 / s96 set trap 同口径，detached 属性写入不 prepare）→ run-once（与 R387/s95/s96 **同表互斥**）→ type 门（空/JS 派生关键词；module 拒绝）→ nomodule → for/event → 分派：外链（src/href/xlink:href）单点复用 `_zwPrepareSrcScript`；内联 s95 同款调用点同步 eval + `_zwReportListenerError` 上报 + 执行即置记账。prepare 拒绝（type 非法/体空/未连接）不置记账——144 形「先 textContent=（type=text/plain 拒绝）后 type="" 重触发」的臂序依赖此。
- **设计要点（调试返工定谳）**：初版自带 setTimeout+`__zw_fetch_script` 取源执行器致双执行（钉 C 首版 log `e1,e2,e2`）——与既有 appendChild trap R387b 各跑一遍 marker。诊断手段：inc 埋 `new Error().stack` 分辨两条路径（run#1 promise 链 `_zwRunOrDeferPromiseReaction`/`fulfilled` = 既有 R387b；run#2 `__zw_fire_due_timers` = 初版 timer）。重设计为分派路由后三钉绿——既有执行臂与本面触发臂共享记账表，任一先到即独占，双触发面（textContent= 后重插入等）不双执行。

## 3. 验证

- 单测：zero-engine lib **3544/3544**；zero-webview lib **814/814**（持平——webview.rs 本轮零变更）；zero-wpt-runner bin **236/0**（+3 钉：type 移除同步内联 prepare 序 `[begin,s1,end]` / svg script textContent= 重触发 / 外链 type 移除取源 + marker FIFO `[e1,e2]`——钉 C 临时根下 inc 页锚，断言外提 .then 防 shim timer 吞错）。
- 全子树终扫（FILTER=scripting-1 口径，与 s108 基线同）：**380P/170F/37T+15N**（s108：377P/173F/37T+15N，**+3P**）；行级归一 diff（/tmp/s109-final-scripting-1.log 存档，基线 /tmp/s108-final-scripting-1.log）：**零 P→非P**（无 Pass 行消失、无新增非 P 行、消失非 P 恰为 3 销账页）——execution-timing **124**（type 移除内联重触发）/ **144**（svg script textContent=）/ **145**（空 svg script textContent=）三页 Fail→Pass。
- **122-126 余面根因定位（本切片边界，探针实录）**：四页共同缺项均为 marker（createElement 工厂节点）的外链执行——script#t 面（type 移除取源）已由本面修复（125/122 got 含 external #1；123/126 got 含 #2）。单变量钉测：`async=false` 无罪（钉绿）；query 串在钉入口杀 marker（run_testharness_html 的 script_source_fetcher 不剥 query；corpus wpt_data fetcher 剥——非 corpus 根因）；corpus strict 管线对工厂节点外链的投递/收集面为下切片探针点。
- fmt：wpt-runner clean；engine 侧 `paint/painter/text.rs:4130` 存量 fmt diff（渲染流文件，本流未触碰，仅提及不修）。clippy engine+wpt-runner `--all-targets -D warnings` clean。渲染面零依赖——reftest/product-smoke 未跑。
- 过程纪律：定向 run 首轮只存 `tail -50` 误读「41→6」——重以全扫文件计数（重踩记忆库已录 tail 截断坑，教训强化：全扫一律 tee 全量落盘再统计）。

## 4. 挂账

- **s110 主候选：122-126 marker 面**——createElement 工厂节点外链在 corpus strict 管线的执行死因探针定谳（疑点：strict 投递/重扫收集面对无 sel/handle 锚工厂节点的处理；钉面 R387b 可执行而 corpus 面不能）；122 子测试 1 async IDL reflection（re-insert 后 `script.async` 反射 true）。
- 130（append external script to script）/147/148（insert multiple inline scripts; first moves/deletes subsequent——复杂 scheduler 面）；142 write 臂 xlink:href；143 eventOrder 完整核验（承袭）。
- 承袭：063 Timeout、slow-loading 外链时序 ~11 行（pipe=trickle 建模）、non-external-no-import（module async）、parse-error 族、http-accept-header、referrer-policies、dynamic import() with 透传、classic 外链 SRI、collect 账本打包重构、接口对象基建面、FIXME 组、/tmp/zw-hb*.txt 插桩来源待查。
- **待决清单（须用户拍板，不阻塞）**：css/text 数据模块语料资源缺失（补语料属基准变更）。
- 本轮销账：execution-timing 124/144/145（3 页）。
