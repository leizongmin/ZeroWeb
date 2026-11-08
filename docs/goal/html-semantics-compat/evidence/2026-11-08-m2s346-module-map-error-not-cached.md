# M2-S346：动态外链 module join + HTTP 失败不入 module map——concurrent-failed-imports 双案 F→P

日期：2026-11-08。控制文档：`docs/goal/html-semantics-compat.md`（头部状态链 s346 段）。
本切片收口 s345 下一步候选 concurrent-failed-imports 簇（whatwg/html#10327「Don't cache
HTTP errors in the module map」判据面）：双案子测试键 F→P，语料口径 the-script-element
**26F→24F**，det P→F=∅，T=8 同集零位移。施工跨 429 打断两 session：勘根+首版修复+域 join
落盘于上一 session（18:20-18:53），合流 rebase + 复跑重锚定 + 落账在本 session。
spec 依据：
- module map 并发装载 join 单次取回、HTTP 失败不入 module map、joined load 各收自身取回
  结果 https://github.com/whatwg/html/pull/10327
- fetch a single module script
  https://html.spec.whatwg.org/multipage/webappapis.html#fetch-a-single-module-script

## 定谳

1. **动态插入外链 module 每元素独立取回，无 module map join**：旧 `_zwPrepareSrcScript`
   module 臂每元素 setTimeout 后独立 `__zw_compile_module`——5 并发同 URL 装载计 5 次取
   回（判据 1「only fetch once」红态 stat 面不可观测）。修复 = document 级
   `__zwDynModuleMap` join 单次取回，态机 absent → null 占位（**同步置位**——join 窗口
   覆盖同任务内后继插入）→ waiter 登记（entry 已在本臂上方入 in-order FIFO，队列位次
   保持）→ owner 契约入缓存 → waiter 逐个同面 settle；缓存命中走 setTimeout 直取完成
   （任务化 prepare 时序保持——s343 inline-async-inserted-execorder 同理由）。
2. **module-fetch-tester.py 端点不在语料且 runner 无仿真臂**（s227 语料资产勘根/s344
   delayed-modulescript.py 端点臂同族）：上游 wptserve handler 维护 key 维度请求计数状
   态机，缺臂时 stat 面空转（pre-fix 红态 `expected "1" but got ""`）。修复 = runner 双
   face 端点同调 `module_fetch_tester_serve` 共享 `MODULE_FETCH_TESTER_STASH`：script
   fetcher 臂（模块取回 face）+ PyRoute 页面 fetch() face，计数/应答逐字节一致；置于
   `__zw_mtype` intent 臂前（s267 同理由——.py 非 css MIME，intent 臂先接管即断观察面）。
   缺省 fail-first（首发 404 后续成功）、always-fail 恒 404、action=stat 只读回计数不
   计数；fetcher 契约无 status 通道，404=Err 加载失败面；页面 fetch() face 404 有完整
   status 应答（上游 wptserve 语义，"not found" 逐字镜像）。
3. **HTTP 失败不入 module map**（whatwg/html#10327 判据核心）：owner 取回失败**先删键**
   再逐面失败——兄弟 error handler 同步重插同 specifier 时占位已清、得**新取回**
   （fail-first 第二发成功 → script#3 load）；join 的 waiter 各收自身取回结果——
   script#2 恒 error、不观察内部占位也不吞 script#3 成功面（判据 2）。缓存仅记忆成功契
   约（同 IIFE `__moduleCache` 记忆化语义——模块体恰一次求值，load/error 每元素派发）。
4. **三态失败/成功语义 owner/waiter 两路逐字保持 s250 臂原语义**（共享 helper
   `_zwDynModuleFail346`/`_zwDynModuleSettle346`）：①取回失败 → error（UA 取回失败派
   trusted 事件，s129 fetch-src/failure isTrusted 断言面）；②specifier 解析失败 →
   load（空码经执行器即仅派 load，队列/直执同形）；③编译成功 → 执行。entry（队列）与
   直执两形同覆盖。

## 施工

| 面 | 位置 | 内容 |
|---|---|---|
| 动态 module join | engine `js_dom_shim/part03.js` `_zwPrepareSrcScript` module 臂 | document 级 `__zwDynModuleMap`/`__zwDynModuleWaiters` 态机（absent→null 占位→契约缓存）+ 失败先删键 + waiter FIFO 结转；映射挂当前连接文档（per-document，iframe 视图隔离——s341 `__zwScriptElStack` 同法）；键=原始 src 串（FIXME：spec 按解析 URL 记账，跨拼写同 URL join 未建模，语料无钉面） |
| 端点状态机 | wpt-runner `testharness.rs` | `module_fetch_tester_serve`（key/action/mode 状态机）+ `MODULE_FETCH_TESTER_STASH`（token() UUID 键，进程级只增有界）；script fetcher 臂 module-fetch-tester.py 判等（模块取回 face） |
| 页面 fetch() 通道 | wpt-runner `testharness_py_routes.rs` | `route_module_fetch_tester` PyRoute Exact 注册（`module-fetch-tester.py`；404 "not found" / 200 text/plain 应答形态） |
| 单测 | 同 testharness.rs | `module_fetch_tester_endpoint_state_machine_s346`（fail-first 首发 Err+计数 1/二发上游逐字节体；always-fail 恒 Err 计数递增；stat 只读；页面 fetch() 通道同 key 同计数 + 404/200 应答形态） |

## 验证

- solo 三实例绿：pre-fix 双 F 勘根（`s346-concurrent-failed-imports-solo.log`——stat=""
  fetch() 通道缺失 + script#3 error 无 join 无重取）→ 修复后 `s346-solo-fix.log`（18:51
  首版修复二进制）→ 合流后新二进制复跑 `s346-solo-merged.log`（本 session 20:24，
  含 46号 M3 切片三 12 臂合流的 release runner）双案全绿。
- 域 join（the-script-element 子域、键级 stable 集语义——`LC_ALL=C`、`grep -a`、
  (key,state) `sort -u`）：A = `s345-b2-domain-rerun.log` the-script-element 子集
  **1093P/26F/8T**；B = `s346-b-script-element.log`（18:53 修复后 scoped 跑）
  **1095P/24F/8T**（raw 1096P 含 1 重复 Pass 键 execution-timing/149——s344 同现象，
  `uniq -d` 实证）。det F→P=**2**（concurrent-failed-imports 双案逐键吻合）；det
  P→F=**∅**；T=8 同集零位移（execution-timing 063/fetch-src empty 双案/load-error-
  events-3/credentials setTimeout 变体/code-cache-base-url/instantiation-error-8/
  script-crossorigin-network 在册长尾）。语料口径 **26F→24F**（自建 probe 恒红 2F
  扣除承前序切片口径）。
- 单测（test-guard 包裹，合流树重跑）：runner **298P/0F**（s344 基线 294P + 46号 M3
  切片二/三新增 + 本切片 1 钉；含 `module_fetch_tester_endpoint_state_machine_s346`
  定向通过）；engine **3817P/6F**——6 案逐名同 s345 挂账⑦既有集（t8k websocket 对 =
  ba03fd7ff；img/svg/fetch/iframe SW 四案 = f021e11fb），非本切片引入，part03.js 变更
  零新增红。
- clippy：engine **0 error**；runner 29 error **全部为 46号 M3 流既有债**
  （needless_return ×23 + needless_borrow ×6，HEAD stash 后实证同红；位置 ≥281 行均在
  本切片新增域外——切片二/三 PyRoute 臂迁移引入），本切片新增代码零命中；碰头按
  run-rules #9 挂账不硬解（见挂账 2）。rustfmt 净（本切片测试体 2 处机械格式
  `cargo fmt` 收口）。
- 合流注记：本切片 WIP 与 46号 M3 切片三（beacon/xhr/fetch 12 臂 PyRoute 迁入）在
  testharness_py_routes.rs 碰撞，rebase stash pop 冲突人工合流（双方注册共存，路由体
  无交集；pull 后远端新增 4 提交——editing-compat F1'-A / runner-net-infra M4 前置 /
  layout-engine R5466 / 46号 切片四前置）。

## 挂账（承继 + 更新）

1. **join 键=原始 src 串非解析 URL**：跨拼写同 URL join 与跨基解析差异未建模——FIXME
   在码，语料无钉面（spec 按 resolved URL 记账 module map）。
2. **runner 定向 clippy 29 既有 M3 债**（碰头挂账，run-rules #9）：46号 M3 切片二/三
   迁移臂 needless_return/needless_borrow——该流活跃收敛后由其流或独立机械刀处理；
   修复前 runner 定向 clippy 门禁维持红态记录（engine 与其余 crate clippy 不受影响，
   本切片新增代码零命中）。
3. s345 既有承继原样：包装级 parse 原子性（choice-of-error-1/compilation-error-2 双
   案，需 compile-only API 或宿主取回面）、内联 module 错误行号（evaluation-error-5）、
   getter 安装失败回落、同名多星号首见 FIXME（均语料无钉面）、engine 6 案 M3 回归修复
   （挂账⑦，视 46号 流合流或单开切片）、s341-s344 承继、域长尾 22F 在册面。
4. 下一步候选：evaluation-error-5 行号传播面（单钉面小）或 parse 原子性双案（勘
   compile-only API 可行性优先）——下轮勘定；engine 6 案回归修复按 46号 M3 收敛插队。
