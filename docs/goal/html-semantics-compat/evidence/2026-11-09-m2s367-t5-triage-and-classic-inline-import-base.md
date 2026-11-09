# M2 切片 367：T=5 长尾勘定 + classic 内联投递动态 import 解析基恢复（code-cache-base-url T→6P，域 T 5→4）

**日期**: 2026-11-09
**目标对**: 挂账「T=5 长尾」逐案勘定——五案（load-error-events-3 / code-cache-base-url / dynamic-imports-credentials-setTimeout / instantiation-error-8 / script-crossorigin-network）归类施工候选 vs 基建缺口；本 slice 收敛可修面
**结果**: **code-cache-base-url 文件级 Timeout → 6 子测全 Pass**（语料补齐 6 静态件 + webview classic 内联投递解析基恢复一刀）；域 the-script-element **T 5→4**、残 F=0（零 P→F）；其余四案定谳挂账归属（两案引擎架构面、两案基建面）。

## T=5 逐案勘定

1. **code-cache-base-url.html（本轮收口）**：双重根因。
   - **语料缺口**：`module/dynamic-import/{alpha,beta,gamma}/{code-cache,import}.js` 六件
     未打包（`ls` 证 absent）→ 三外链 classic 404 → promise_test 零注册 →
     runner「completed without reporting registered tests」映射文件级 Timeout
     （s227 资产缺口同形态）。六件均纯静态（import.js 单行 `export const A`），
     按 s288/s304 协议 github raw/jsdelivr 按 pin 315976933 取件补入
     （raw 通道 200/404/hang 三态抖动，jsdelivr 镜像稳）。
   - **引擎缺口**（补件后 6F→修复后 6P）：runner 把外链 classic 脚本**内联投递**
     （s261 `data-zw-inlined data-inline=原始src`），内联体里 `import('./import.js')`
     走 s88 V8 原生宿主回调读**槽位 referrer**；referrer 旧值 = 页面基
     （`script_bases` 快照）→ 相对 spec 解析到页目录 → 404 → 空 contract →
     TypeError `Failed to fetch module`。spec 解析基准 = 调用脚本基 URL（外链
     classic = 自身 URL，ECMA-262 HostResolveImportedModule referencingScript）。
     **s336 同形修复只覆盖模块 lane**（InlineModule 的 _importMeta/依赖收集基），
     classic 槽位面缺位。修复：webview.rs referrer 计算对 `Inline(_)` 臂按
     `script_csp_info[idx].inlined_src` 以 s336 同款双形态归一（裸路径归一
     root-absolute）解析绝对 URL；真内联（无 data-inline）维持脚本基快照
     （= 文档基，spec 内联脚本基零漂移）。
2. **load-error-events-3.html（定谳：引擎架构面，挂账）**：write 注入
   `<link /common/slow.py>` + 内联 script（onLoad/onError/onExecute）。勘根三
   probe 面定谳：**同脚 strict 顶层 var 对 write 面 mid-body eval 不可见**
   （probe-e：`sameVar-REFERR`/`laterVar-REFERR`）——本仓 classic 脚本以 strict
   间接 eval 执行，var 绑定 eval 局部、accessor 导出在 eval 源尾（R201/s251
   机制）；真 case 写入体内 `onExecute(test6_load)` 对同脚 `var test6_load`
   ReferenceError → 永不 done → Timeout。spec：script 级 GDI 在**体求值前**建
   全局 var/fn 绑定（strict 不改变 script 级 var 语义）→ 真 browser mid-write
   eval 可见。修复路径探勘四条全堵：前缀注入破坏 directive prologue；指令序后
   插入仍留 eval 局部赋值（accessor 恒 stale）；post-hoc accessor 无法提前；
   write 面读不到调用方作用域。**结构性修复 = classic 脚本改 SCRIPT 编译**
   （GDI 原生语义；连带 sourceURL/sentinel/export-suffix 机器迁移）——独立大
   slice 挂账。另 `/common/slow.py` 为 wptserve handler（服务端 Python），静态
   runner 不执行——端点仿真臂需与 hoisting 修复同 slice 落。
3. **dynamic-imports-credentials-setTimeout.sub.html（定谳：基建面）**：
   `/cookies/resources/set-cookie.py` 服务端 Set-Cookie + www2 跨源 cookie jar
   ——runner 静态语料不可执行（testharness_cookies.rs 在册口径，s124 可达性
   待决同族）。维持挂账。
4. **instantiation-error-8.html（定谳：基建面）**：import attributes
   `with{type:"css"}` + .py 端点（s102 F→T 27 页归因在册）。维持挂账。
5. **script-crossorigin-network.sub.html（定谳：基建面）**：cookies + www2
   （s124 在册）。维持挂账。

## 首版缺陷（probe 自身两轮，域跑前 solo 抓获）

1. probe-a/b/c/e 复制真 case 形状时把 `var order` 声明与 `document.write`
   放同脚本——write 面 mid-body eval 对同脚 var ReferenceError（恰为待勘根因
   本体！）→ order 恒空假象。probe-d 四臂拆分（数组前置声明）定位；教训：
   探针复刻目标形状前先隔离「声明脚本 vs 写脚本」边界（s251 同脚 var 竞态
   第 20 条的写面变体）。
2. probe-g 首版 `<script src="alpha/import.js">` classic 引 module 体 →
   `Unexpected token 'export'` 整页红；剥该行（import 面自取）后四臂全绿。

## 验证

- shim/Rust：仅 webview.rs referrer 臂一刀（fmt PASS）。
- solo code-cache-base-url：修复前 6F（Failed to fetch module ×6，补件后）→
  修复后 **6P**。
- 域 join（A=s365-b 1132 pair vs B=s367-b 域跑探针剥除后 1137 pair）：
  公共键 1131，状态 diff **唯一 = Fail 143 → Pass 143**（s366 修复在本二进制
  基线 s365-b 中尚未落地——预期位移非回归）；A-only 1 = code-cache-base-url
  文件级 Timeout 键（旧形态）、B-only 6 = 六子测 Pass 键（新形态）——合即
  **该页 T→6P**；T 名单 5→4（credentials-setTimeout / instantiation-error-8 /
  script-crossorigin-network / load-error-events-3 维持）、NotRun=5 同构、
  零 P→F。域 the-script-element 残 F=0 保持。
- **join 形态新坑入册**：文件级 Timeout 键 = `path :: path`（测试名即路径），
  与子测键 `path :: name` 恒异——键级 join 中 A 侧文件超时翻 B 侧子测集**
  不会出现 A-only↔B-only 配对假象**（A-only 是旧键、B-only 是新键，需人工
  合并判读）；s366 轮 143 案 A 侧为真 Fail 子测键故无此面。
- make test：v8 workspace 腿完整 **9326P/7F**（与 s365/s366 基线总数逐名同构，
  零新增红——7 红既有集不变：img_runtime_deferred_settle_r5275 /
  svg_img_settle_density_r5277 / fetch_filtered_response_type_matrix /
  iframe_sw_messages_dispatch / response_body_readable_stream_r2967 /
  websocket_error_nonterminal / websocket_event_flow）。quickjs 腿未达（make 于
  v8 腿既知 7 红 abort EXIT=2 同形态）；referrer 臂改动仅在 webview 交付循环
  runner 车道激活（inlined_src 在场形），engine lib 单测无此形态覆盖面。

## 挂账 / 下一步

- **classic 脚本 SCRIPT 编译切片**（load-error-events-3 解锁键）：GDI 原生
  var/fn 提升、export-suffix/sentinel/sourceURL 机器迁移、`/common/slow.py`
  runner 仿真臂；设计勘根本轮已备（四条修复路径排除记录）。
- 基建面两案（cookies.py/www2、import attributes/.py）维持 s102/s124 挂账。
- probe8/9/10 转正决策；142「xlink:href write 注入面」挂账行退役（s114 已修
  ——s366 哨兵 142 Pass 复证，控制面本轮同步清账）。

## 日志

- evidence/logs/s367-probe-a.log / s367-probe-d.log / s367-probe-e.log /
  s367-probe-f.log / s367-probe-g.log / s367-ccb.log（勘根链）
- evidence/logs/s367-b-script-element.log（域 B 面）、s367-fix.log（solo 修复面）
