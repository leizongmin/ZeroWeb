# 引擎 HTTP 缓存层 — freshness/条件请求/request.cache 模式态（内存态语义层）

**版本**: v1.1
**日期**: 2026-11-08（M0 完成——基线修正、102F 分簇、corpus 盘点、数值门落定）
**状态**: Active——DC-1 达成（[M0 对账档](http-cache/evidence/2026-11-08-m0-reconciliation.md)、[架构落点说明书](http-cache/evidence/2026-11-08-m0-architecture.md)），下一步 M1
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
   covered > 0 为通道存在性底线；批次排期对齐 46 号 M3。
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
- **M1**：缓存核心接线——net CacheMode 扩六态 + 矩阵单测（M1-a）→ runner
  cache.py fixture + CaseHttpCache 包裹（M1-b）→ C 簇 48（default/
  default-conditional）收敛（DC-2 前半）。
- **M2**：wire 传播（wire 第 11 参 + FetchRequest.cache + shim 8 调用点 +
  双 app handler 映射）+ 模式态六态 B 簇 54 收敛（DC-2 收口 / DC-3）。
- **M3**：102/102 复测 + v2/v3 零回归 A/B + corpus 批次对齐 46 号排期
  （DC-3/DC-4 前半）。
- **M4**：零回归复测收口（DC-4）。

## 依赖约束（run-rules §9）

- 与 43 号（net-api-compat-v3，已收口归档
  [archive/net-api-compat-v3.md](archive/net-api-compat-v3.md)）共享 fetch
  通道——缓存层改动同窗复验 v2 终账（11,040 + S 系列）与 v3 终账
  （13,256P，2026-11-07 收口）逐 subtest 零回归；43 号 request-cache 账以
  本 goal 交付承接（两 goal 控制面互指）。
- 与 46 号（runner-net-infra）py 端点前置——http-cache corpus 批次排期
  对齐 46 号 M3 框架交付。
- 缓存语义实现处加 RFC 9111 条款锚定注释；涉及导航资源加载的缓存路径
  改动与 rendering-compat 的加载性能门（benchmarks perf-gate）同窗核对，
  防缓存命中路径改变加载时序预算。
- 无人值守构建/测试走 test-guard 与 systemd-run 内存墙钟双限。
