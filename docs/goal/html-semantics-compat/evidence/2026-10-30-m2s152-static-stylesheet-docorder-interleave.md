# M2 s152 · 静态 link rel=stylesheet 文档序交错触发——Face C script-blocking 时序面收口

日期：2026-10-30 · 切片：html-semantics M2 · 前序：s151（stylesheet MIME/nosniff
判定矩阵）

## 入场定刀位

s151 下一步候选①：Face C render-blocking/script-blocking 时序面——
link-{error,load}-fired-before-scripting-unblocked（the-link-element/）2F。
conditionally-block-rendering-on-link-media-attr（interactions-of-styling-and-
scripting/）同面 2F 并入勘根（结论：不可修，维持挂账，见下）。

## 勘根（探针链 8 文件五轮二分定谳）

1. **语料形态**：两目标文件 = 前置 inline script 声明 `var saw_link_*` +
   `var t=async_test(...)` → `<link href=nonexistent.css|style.css?pipe=trickle(d3)
   rel=stylesheet onerror/onload=inline content attribute>` → 后随 script
   `t.step(assert_true(saw_link_*))`。事件注册用 **inline content attribute**
   （`onerror="..."`），区别于 s151 三文件的 IDL 属性形——派发面
   `_ensureInlineHandler`（part03，`__zw_get_attr(sel,'on'+type)` 编译）在位，
   s151 动态臂已证。
2. **spec 语义**：script-blocking stylesheet 只阻塞**文档序位于其后的** pending
   parsing-blocking script（prepare-a-script + the-link-element「A style sheet
   that is blocking scripts」）。宿主「全解析后统一执行脚本」管线缺静态 link
   触发点（s150 `_zwStylesheetMaybeStart` 仅动态臂：part01 childList /
   part04 setAttribute / part05 IDL）。
3. **v1 刀法定谳失败**：runner 脚本阶段起点一次触发全部静态 link——把 link
   **之前**的 script 也推迟（探针三截面：事件链通 p1/p2 Pass，p3 addEventListener
   探针自身注册晚）→ 真根因 = 前置 `var t` 声明 script 被推迟，onerror fire 时
   `t.step` 撞 ReferenceError → saw 恒 false。
4. **v2 序号对齐依据**：runner `extract_page_scripts_indexed`（extract.rs:175）
   的 script 序号 = 全部 `<script>` 元素文档序（过滤前递增，与 shim
   `getElementsByTagName('script')` 经 `_zwDocAllElements` 返回序在册对齐——
   currentScript 面 part06:401/5894 同口径；template 子树两侧均不枚举）。
5. **trickle 仿真单根**（load 案修复拦路虎）：v2 交错触发就位后 error 案
   即 Pass、load 案仍 Fail。探针链五轮二分——probe-log 包装 fetch 记
   `f0|f1:function` 无 f2（promise 创建但永不结算，80ms 轮询不含 eventual）；
   probe-wire 直调 `__zw_fetch` 宿主通道三案（plain/pipe/missing）**全部同步
   wire**；probe-iso 直接 `fetch()` + text() 对 plain/pipe/headers 三案全 Pass
   → 隔离出唯一变量 = **扫描窗内发起的取回体**。根：shim fetch wrapper 的
   wptserve trickle 仿真（part01，`[?&]pipe=trickle\((?:[a-z]+)?(\d+)\)` →
   `setTimeout(_issueFetch, N×1000)`）——`trickle(d3)` 把取回体发出推迟 3s，
   load 随之推迟到后随 script 断言之后。上游语义 parser 会阻塞 3s 等 load
   （断言真值成立）；runner 无 parser 阻塞，延迟只破坏时序。probe-iso 过测系
   promise_test 等 3s 定时器（runner 泵送），非同窗结算。
6. **conditionally-block 案级定谳（不可修）**：案 1 `?delay=2` 延迟投递在
   `__zw_fetch` 同步契约下不可仿真（即时仿真下 brown sheet 立即注册，
   assert_false 恒 Fail）；案 2（3s 后两 sheet 都注册）即时仿真可过，但
   per-sheet `styleExists` 断言踩 document.styleSheets 单条合成 blob 条目
   （part06:5920，R5114）不可满足 → 整文件维持挂账。

## 施工（3 文件净 +103/−1）

1. **shim part03 `_zwStylesheetStaticScanUpto(scriptOrdinal)`**：首次调用单次
   建交错表（`_zwDocAllElements()` 文档序遍历，SCRIPT/LINK 分型——LINK 不预过
   rel，`_zwStylesheetMaybeStart` 门内 rel 分词自甄别零成本返回）+
   `scriptPrefix[]` 预计算（每 script 前 link 累计数，越界收敛全部）+
   `_zwScanFiredCount` 前缀游标单调推进（O(1)/次调用）。触发窗置位
   `__zwScanCollapseTrickle`，同步段结束即清（.then 链内 @import 内层取回与
   页面 fetch() 零波及）。FIXME 注记交错表快照语义（脚本执行期动态插入的序移
   未追踪——动态 link 走既有触发臂，幂等门兜底）。
2. **shim part01 fetch wrapper trickle 折叠**：`_delayMs` 计算的 trickle 臂加
   `globalThis.__zwScanCollapseTrickle !== true` 门——扫描窗内发起的取回体折叠
   trickle 延迟即时发出；fetch API 面 timeout 竞速用例不在扫描窗内，延迟语义
   零变化。
3. **runner webview.rs `run_page_scripts_impl`**：v1 全前置块删除；脚本循环内
   每脚本体求值前（sandbox 重借后、CSP 分支前）独立 execute
   `globalThis._zwStylesheetStaticScanUpto(script_index)`——独立求值保证
   `__zw_fetch` 同步契约 + fetch promise 微任务在本次求值尾清空（load/error
   先于本脚本体；`__zw_begin_script` 与体同求值不可用）。CSP/混合内容拦截
   script 的序号仍占位，游标随下一执行脚本补齐（仅触发时机提前，无漏触发，
   注记在位）。E0499 教训：插入块用循环体既有 sandbox 重借，不另开
   `self.js_sandbox` 借用。

## 验证

- **探针链 8 文件（验后已删）**：probe（v1 事件链三截面）→ probe-load（p2
  load 恒不 fire → 非时序问题）→ probe-net（设计误差：断言同步跑在脚本体，
  then 回调本该在 checkpoint——探针方法论教训在册）→ probe-wire（宿主通道
  三案同步 wire 排除 handler）→ probe-chain（promise_test 真等 text() 链，
  expected 串笔误但证结算）→ probe-iso（plain/pipe/headers 全 Pass，隔离
  扫描窗变量）→ probe-log（f0|f1:function 无 f2 钉死 fetch 未结算）→ probe-m
  （位置变体复证）。踩坑在册：探针文件名含 `-manual` 尾缀被 runner
  `timing_base_case_skipped` 的 `-manual.html` skip 规则跳过（零输出 exit 1，
  改名后正常回报）。
- **定向 fired-before-scripting：2F→2P**（error 案 + load 案全 Pass；
  基线 s151 双 Fail）。
- **全量案级 join（对 s151 基线 /tmp/zw-s151-full.log，(key,state) 对 sort -u
  去重）**：8080 对 vs 8081 对（7186P/806F/88T vs 7185P/808F/88T）——**真修复
  2 案 F→P**（两目标文件）、**F 新增 0、P→F 0**、P 消失 1 案（link-multiple
  media 触碰案——定向复验稳定 P，s151 在册「全量负载 flake 少报」同形，非本刀
  位移）。
- **门禁**：clippy workspace --all-targets -D warnings 零告警；fmt 过；
  shim 规范序拼接 node --check SYNTAX_OK 110217 行；zero-webview lib
  809 passed/9 failed 与 stash 基线逐字一致、zero-engine lib 3600/9F 同
  （9F 均既有环境依赖：image-decoder 进程启动/cache_storage URL 面等，非本刀
  引入）。

## 挂账（本切片更新）

- **收窄：Face C script/render-blocking 时序面**——fired-before-scripting 2 案
  收口；conditionally-block-rendering-on-link-media-attr 整文件维持挂账
  （案 1 delay=2 延迟投递同步契约不可仿真 + 案 2 styleSheets 单条 blob 分条
  面 R5114；修法前置：styleSheets 按 link 分条建模）。
- **新记方法论**：全量 join 的 P 消失键先定向复验再归因（s151/s152 连续两轮
  link-multiple media 案同形——全量负载下少报、定向稳定 P，在册 flake 族）。
- **承袭不变**：跨源静态资源 ACAO 面（s151 新记，2 案 + https Load 案 1F——
  通用静态臂无 ACAO，需影响面盘点）；adopt/innerHTML 身份双表示深水面
  （document-without-browsing-context 2F）；LinkStyle sheet 属性面 1F；
  rel=preload fetch 面（link-multiple 案 5 + 案 6 页级 10s 时序必然）；
  stylesheet-media-change-no-fetch 1F；@import 多层递归链；meta long ×
  corpus 通道放宽失效；details 族 lib flake；promise_test 串行启动面；待决
  三项（css/text 语料资源、green-2x2.png、parser form pointer 缓行）须用户
  拍板。

## 下一步（s153 候选）

①跨源静态资源 ACAO 面影响面盘点（s151 新记——通用静态臂加 ACAO 的全语料
波及评估，2+1 案修复面，收益最大）；②adopt/innerHTML 身份双表示深水面
（document-without-browsing-context 2F）；③rel=preload fetch 面（link-multiple
案 5 + 缓存复用）；④LinkStyle sheet 属性面 1F。
