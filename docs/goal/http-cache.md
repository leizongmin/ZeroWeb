# 引擎 HTTP 缓存层 — freshness/条件请求/request.cache 模式态（内存态语义层）

**版本**: v1.6
**日期**: 2026-11-09（M4 终验收口——全窗 A/B 零回归 + fetch 头条 +103P 不回落；DC-1~DC-4 全达成）
**状态**: **Done**——DC-1 ✅、DC-2 ✅（[M2 证据档](http-cache/evidence/2026-11-08-m2-wire-propagation.md)，族 102/102 + 真 P→F=0）、DC-3 ✅（双通道同层单实现）、DC-4 ✅（[M4 终验证据档](http-cache/evidence/2026-11-09-m4-final-verification.md)：全窗 13,992P 真 P→F=0 对 v2/v3 双基线、fetch 头条 2,369P/17.85% 对 v3 复测 +103P、corpus 124P/138 入账）；越界发现一项引擎回归（ddb1e73cf shadow 栈溢出，非本 goal 面）记[待决项](#待决项)
**执行模式**: 引擎专项（网络栈基建）+ WPT fetch/api 语料验收
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——网络面）
**前账**: `docs/goal/archive/net-api-compat-v2.md`（2026-10-29 收口——
fetch/api DC-4 按「剔除 request-cache 族 98F」口径达标；待决项经用户
2026-11-04 拍板转正为本 goal）

## 基线事实（2026-11-08 修正——M0 逐条对源码核实）

- **net 缓存核心已存在**（2026-11-04「引擎无缓存层」基线过时）：`zero-net`
  有完整缓存层（http_cache/cache_policy/cache_key/disk_cache/negative_cache，
  freshness/条件再验证/304 合并/Vary/must-revalidate/分区/负缓存，RFC 9111
  条款注释锚定）；浏览器 fetch/xhr 通道 default 模式已经过它
  （`ResourceLoader::submit_http`）。
- **实际缺口三处**（[M0 对账档](http-cache/evidence/2026-11-08-m0-reconciliation.md) §一）：
  ① JS 六模式态在 shim 仅校验未传播——`__zw_fetch` wire 十参无 cache 字段，
  net `CacheMode` 四态（缺 Reload/ForceCache）；② runner 通道
  （`wpt_data_fetch_handler`）无缓存语义；③ request-cache 族 fixture
  `resources/cache.py` 缺位——全族请求 404。
- **request-cache 族现账 102F**（2026-11-08 定向实跑，分母较 v2 的 98F 上游
  漂移 +4）；地址池 102（window 变体）与 wpt.fyi 10-29 Chrome 分母逐文件
  同构，Chrome 同簇通过率 100%。
- **上游验收面**：fetch/api/request/request-cache\*（102F 所在簇）+
  `fetch/http-cache` 目录（**路径修正：上游无 fetch/api/http-cache，实为
  fetch/http-cache**；Chrome window 变体 417/554=75.3%；py stash 端点依赖
  46 号 runner-net-infra，本 goal 只消费）。
- 架构边界（AGENTS.md）：M0 判定 **IPC 契约零变化**——全链路（shim wire/
  FetchBridge/handler/ResourceLoader/HttpCache）均为进程内接口，
  `zero-protocol` 不动（[架构落点说明书](http-cache/evidence/2026-11-08-m0-architecture.md) §5）。

## Mission

建引擎 HTTP 缓存语义层（**内存态**——磁盘持久化与产品缓存策略排除）：

1. **M0 对账**：98F 分簇（✅ 2026-11-08 实测 102F——缓存头语义 / request.cache
   模式态 / 上游语料冲突三分类；net 缓存核心已在的基线修正一并落档）+
   http-cache corpus 盘点 + 架构落点说明书（缓存 key、存储位置、与
   fetch_bridge/net 栈的接口、IPC 契约影响面）；RFC 9111 语义面清单落档；
   DC-4 数值门落定。
2. **缓存核心**（M1/M2）：内存态缓存存储 + freshness 计算（age/
   Cache-Control/max-age/no-cache/no-store/Expires）+ 条件请求
   （ETag/If-None-Match、Last-Modified/If-Modified-Since）与 304
   revalidation + Vary 匹配。
3. **request.cache 模式态**（M2/M3）：六模式态语义（含 only-if-cached
   的 504 语义）；fetch/xhr 双通道接线。
4. **收敛收口**（M3/M4）：request-cache 族 102F 收敛（43 号 fetch/api 账
   承接）+ http-cache corpus 分批入账；coverage 复测落数值门。

## Done Criteria

- **DC-1（M0 对账）**：98F 分簇 + corpus 盘点 + 架构落点说明书（含 IPC
  契约影响面判定）落 `docs/goal/http-cache/evidence/`；DC-4 数值门落定
  并回填本文件。
- **DC-2（缓存核心）**：freshness/条件请求/Vary 语义落引擎（RFC 9111
  对应条款注释锚定）；单测覆盖命中/过期/再验证三路径。
- **DC-3（模式态）**：request.cache 六模式态全落且 fetch/xhr 双通道
  行为一致（同一缓存层，无双实现）。
- **DC-4（收敛收口）**：request-cache 族 102F 收敛达 M0 数值门（102/102）；
  http-cache corpus 入账（covered > 0，达数值门）；一期/二期通道既有账
  逐 subtest 零回归；头条不回落。

**DC-4 数值门（2026-11-08 M0 落定，细则见 M0 对账档 §四）**：

1. **request-cache 族门**：地址池 102（window 变体）× Chrome 同簇 100% →
   **102/102**。确定性偏差逐腿归因；上游语料冲突腿以 wpt.fyi 佐证逐腿排除、
   分母同缩。
2. **http-cache corpus 门（按批次）**：每批导入面 window 变体实跑 pass ≥
   实跑分母 × Chrome 同簇通过率（逐文件 wptfyi 10-29 基准见对账档 §三）；
   covered > 0 为通道存在性底线；批次排期对齐 46 号 M3。**✅ 首批 2026-11-08
   达成**：124P/138 vs 地板 ≈60（聚合）/104（10-29 窗绝对），covered 138
   （[M3 corpus 证据档](http-cache/evidence/2026-11-08-m3-corpus-import.md)）。
3. **零回归门**：v2 终账（11,040 + S 系列）与 v3 终账（13,256P）逐 subtest
   零回归；fetch 头条不低于 v3 复测值 2,266P/17.07%（wpt.fyi universe）。

## 划界

- **内存态语义层，不做磁盘持久化**：产品级磁盘缓存/缓存淘汰策略/
  多进程缓存共享排除挂账（产品化另行立项）；进程内语义完整。
- **service-workers Cache API 不动**：zero-storage 的 Cache API（已收口
  归档域）是存储 API，与本 goal 的 HTTP 缓存层正交；拦截场景（service
  worker fetch 拦截 + HTTP 缓存叠加）按 M0 架构说明书记账。
- **不代 46 号做 fixture**：http-cache corpus 的 py 端点由 46 号解锁，
  本 goal 只消费。
- **IPC 契约**：缓存层若跨进程（renderer/主进程归属按 M0 架构说明书），
  `zero-protocol` 契约变化两端同步，不绕边界。

## 里程碑

- **M0 ✅**（2026-11-08，DC-1）：基线修正（net 缓存核心已存在，实际缺口
  三段接线）+ 102F 分簇（fixture 缺口/模式态 54/缓存头语义 48/语料冲突 TBD）
  + corpus 盘点（fetch/http-cache 20 文件、Chrome 417/554）+ 架构落点
  说明书（wire 第 11 参、CacheMode 扩六态、runner CaseHttpCache 复用
  zero-net HttpCache、IPC 零变化）+ 数值门落定回填。
- **M1 ✅**（2026-11-08，DC-2 前半）：M1-a（CacheMode 扩六态 + 矩阵单测，
  @ f1fb5956b）+ M1-b（runner cache.py fixture 1:1 仿真 + 案级 HttpCache
  包裹 + net 注入映射修正——M1-a 的 no-cache/reload 注入映射与语料相反已
  改正、stale-at-birth 可存性 RFC 9111 §3，@ 666d1f293）。实测 request-cache
  族 **76P/30F**：C 簇 48/48 绿（default 8 + default-conditional 40）；headers
  族 286/286 零回归；余 30F 全为 B 簇 wire 传播面（[证据档](http-cache/evidence/2026-11-08-m1b-c-cluster.md)）。
- **M2 ✅**（2026-11-08，DC-2 收口/DC-3）：wire 传播（**实落第 12 参**——
  index 10 已被 iframe client-id 占用，M0 说明书原记 11 参修正）+
  FetchRequest.cache + shim 八调用点 + 双 app handler 映射 + runner 六模式
  态全矩阵分派；显式模式权威（fetch spec §4.6）+ Vary:* 仅再验证
  （RFC 9111 §4.1）@ f2f9ee1cb。族 **102/102**（B 簇 30F 全收敛）；
  M2 shim 错误语义转换引入的 3 条 P→F（keepalive POST 撞 py_routes 非
  GET 兜底门）经 trickle.py POST 端点臂修复（v3 fixture 先例，账面 3P/3F
  持平）；全窗口 JSON-vs-JSON 真 P→F=0、F→P 104、T 零变化（[M2 证据档]
  (http-cache/evidence/2026-11-08-m2-wire-propagation.md)）。
- **M3**：102/102 复测 ✅（本账即复测）；**net 层语义前置 ✅**（2026-11-08
  ——RFC 9111 五缺口收口：§3 可存条件重写 + §4.2.2 启发式新鲜度 +
  No-Vary-Search 缓存键（自描述键，语料优先语义带 FIXME）+ §3.2 304
  合并泛化 + §4.3.4 更新独立于 304 可存 + §4.4 失效门 2xx|3xx 与
  POST/PATCH 存储路径；zero-net lib 530P/0F，multiset 零回归 NEW==M2
  全等、对 v3 终账真 P→F=0，[M3 证据档]
  (http-cache/evidence/2026-11-08-m3-net-semantics.md)）；**corpus 批次 ✅**
  （2026-11-08，DC-4 corpus 门达成）：fetch/http-cache 端点表 1:1 移植
  （http-cache.py 状态机 + 双 pragma 端点，46 号 PyRoute 注册表 3 行）+
  包裹器门控扩 4 路径（非 GET §4.4 unsafe 落账镜像 + 请求 CC 头推导镜像
  ——`request_cache_mode`/`apply_unsafe_response_to_cache` pub 复用）+
  语料入窗 13 .any.js + 1 pragma html（split-cache/basic-auth 按 M0 §三
  排除）。实跑 **124P/138**（86.2%）vs Chrome 同簇 104/239（43.5%），
  超出 Chrome 面 = 失效矩阵 +9、启发式 +6、请求 CC +7（[M3 corpus 证据档]
  (http-cache/evidence/2026-11-08-m3-corpus-import.md)）；零回归：族窗
  106P/0F + headers 286/286 + zero-net 530P/0F（落盘形态门镜像回退教训
  ——DC-3 单实现指缓存语义层，通道落盘策略可合法分歧，见证据档 §六）。
- **M4 ✅**（2026-11-09，DC-4 收口）：全窗 A/B 终验——net-api 全窗 952 文件
  13,992P/1,322F/9T，对 M2 全窗账真 P→F=0/F→P=0/subtest 集合零漂移（唯一
  差异 = fetch/http-cache 13 新文件 124P/14F 纯增量），对 v3 终账真 P→F=0、
  F→P 104 恰为记账收敛集（[M4 证据档](http-cache/evidence/2026-11-09-m4-final-verification.md)）；
  fetch 头条 **2,369P/17.85%**（post-goal CI 对账 run 42499865933）对 v3
  复测基线 2,266P/17.07% **+103P 不回落**，fetch/api 面 2,098→2,202P；
  正式快照被环比护栏拦下（web-components/shadow-dom 面 −100% 塌陷系
  越界发现项 ddb1e73cf 引擎栈溢出的真实后果），头条数值由同 run artifact
  本地同脚本聚合产出（证据档 §五）；weekly 管线两层容错落地（套件级
  6b3a7d00c + 空文件级 757f4db84）。

## 依赖约束（run-rules §9）

- 与 43 号（net-api-compat-v3，已收口归档
  [archive/net-api-compat-v3.md](archive/net-api-compat-v3.md)）共享 fetch
  通道——缓存层改动同窗复验 v2 终账（11,040 + S 系列）与 v3 终账
  （13,256P，2026-11-07 收口）逐 subtest 零回归；43 号 request-cache 账以
  本 goal 交付承接（两 goal 控制面互指）。
- 与 46 号（runner-net-infra）py 端点前置——**46 号 M4 互指（2026-11-08）**：
  py 框架主表承接面已落——85 行保序注册表 + stash 端点先例 route_cache_py
  注册且 http-cache 通道 102/102 行为验证，fetch 通道 corpus 批次可消费；
  script/service-worker fetcher 分表随 46 号切片六在途（判定端点族不涉
  本 goal corpus 面），完整交付以 46 号 DC-3 收口为准。**消费落地**：端点
  表已由本 goal 落地（2026-11-08 corpus 批次，普查档 P-A 池分工兑现），
  本批 additive 面与切片六无冲突。
- 缓存语义实现处加 RFC 9111 条款锚定注释；涉及导航资源加载的缓存路径
  改动与 rendering-compat 的加载性能门（benchmarks perf-gate）同窗核对，
  防缓存命中路径改变加载时序预算。
- 无人值守构建/测试走 test-guard 与 systemd-run 内存墙钟双限。

## 待决项（2026-11-09 M4 终验越界发现，移交处理）

1. **深嵌套树样式递归栈溢出（bisect 首坏 ddb1e73cf，2026-11-08 02:20 落地）**：
   `build-deep-detached-shadow-then-append-text.html`（servo#43998 同型，
   999 层嵌套 attachShadow 后挂入文档）主线程栈溢出。**根因帧在
   style-system**（2026-11-09 gdb backtrace 定位，[M4 证据档]
   (http-cache/evidence/2026-11-09-m4-final-verification.md) §四b）：
   `compute_styles_recursive` 逐树层自递归（999 嵌套 ≈2,000 帧 ≈8MB 主栈
   打满），经 `compute_styles_with_shadow` 进入；ddb1e73cf 的内层 shadow
   物化（正确行为）首次把 2,000 层 composed 树送进该递归。产品面同样
   暴露：tab-worker 线程默认 ~2MB 栈（apps/browser/src/tab_worker.rs spawn
   未设 stack_size），浅得多的深嵌套页即可打爆。**修复归属渲染流**
   （style-system 为 run-rules §9 渲染流专属面 + perf 关键路径，改迭代
   化遍历须 perf-gate；引擎侧物化为正确行为不回退）——本流（zero-web/
   net）按碰头纪律不代修。影响面：web-components 套件三面全废、CI
   html-sem 分片常红、weekly 快照 custom-elements/shadow-dom 面记 0
   （环比护栏拦快照回写）。
2. **快照补写口径**：若希望在回归修复前先有正式快照落盘（web-components
   /shadow-dom 两面如实记 0），需 `--no-guard` 重出——属环比护栏的人工
   确认豁免，未拍板前不执行；默认等修复后自然回落。
