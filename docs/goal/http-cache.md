# 引擎 HTTP 缓存层 — freshness/条件请求/request.cache 模式态（内存态语义层）

**版本**: v1.0
**日期**: 2026-11-04（立项，用户拍板——v2 request-cache 98F standing exclusion 转正）
**状态**: Active——立项，M0 未启动
**执行模式**: 引擎专项（网络栈基建）+ WPT fetch/api 语料验收
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——网络面）
**前账**: `docs/goal/archive/net-api-compat-v2.md`（2026-10-29 收口——
fetch/api DC-4 按「剔除 request-cache 族 98F」口径达标；待决项经用户
2026-11-04 拍板转正为本 goal）

## 基线事实（2026-11-04 核实）

- **引擎无 HTTP 缓存层**：fetch/xhr/资源加载直连网络栈，无 freshness/
  条件请求语义；fetch/api 的 request-cache 族（default/no-cache/no-store/
  reload/force-cache/only-if-cached 模式态 + 缓存命中行为）98F 按
  standing exclusion 排除（v2 收口口径：剔除后 236F 已达线，含族差 34F）。
- **上游验收面**：fetch/api/request/request-cache\*（98F 所在簇）+
  fetch/api/http-cache 目录（分母在 M0 盘点，py stash 端点密集——语料
  前置依赖 46 号 runner-net-infra）。
- 架构边界（AGENTS.md）：网络代理/存储变化先核对 `zero-protocol` +
  `apps/renderer` IPC 契约，同步主进程与 renderer 两端；`zero-engine`
  不混写 renderer 职责。

## Mission

建引擎 HTTP 缓存语义层（**内存态**——磁盘持久化与产品缓存策略排除）：

1. **M0 对账**：98F 分簇（缓存头语义 / request.cache 模式态 / 上游语料
   冲突三分类）+ http-cache corpus 盘点 + 架构落点说明书（缓存 key、
   存储位置、与 fetch_bridge/net 栈的接口、IPC 契约影响面）；RFC 9111
   语义面清单落档；DC-4 数值门落定。
2. **缓存核心**（M1/M2）：内存态缓存存储 + freshness 计算（age/
   Cache-Control/max-age/no-cache/no-store/Expires）+ 条件请求
   （ETag/If-None-Match、Last-Modified/If-Modified-Since）与 304
   revalidation + Vary 匹配。
3. **request.cache 模式态**（M2/M3）：六模式态语义（含 only-if-cached
   的 504 语义）；fetch/xhr 双通道接线。
4. **收敛收口**（M3/M4）：98F 收敛（43 号 fetch/api 账承接）+
   http-cache corpus 分批入账；coverage 复测落数值门。

## Done Criteria

- **DC-1（M0 对账）**：98F 分簇 + corpus 盘点 + 架构落点说明书（含 IPC
  契约影响面判定）落 `docs/goal/http-cache/evidence/`；DC-4 数值门落定
  并回填本文件。
- **DC-2（缓存核心）**：freshness/条件请求/Vary 语义落引擎（RFC 9111
  对应条款注释锚定）；单测覆盖命中/过期/再验证三路径。
- **DC-3（模式态）**：request.cache 六模式态全落且 fetch/xhr 双通道
  行为一致（同一缓存层，无双实现）。
- **DC-4（收敛收口）**：request-cache 族 98F 收敛达 M0 数值门；http-cache
  corpus 入账（covered > 0，达数值门）；一期/二期通道既有账逐 subtest
  零回归；头条不回落。

**DC-4 数值门决策规则（立项即定）**：request-cache 族按 M0 分簇后的
地址池 × 上游同簇浏览器通过率落门；http-cache corpus 按 46 号解锁的
py 端点面实数落门（本 goal 不为 fixture 缺口背账）。

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

- **M0**：分簇/盘点/架构落点 + 数值门（DC-1）。
- **M1**：缓存核心——freshness + 条件请求 + 304（DC-2 前半）。
- **M2**：Vary + 模式态六态（DC-2 收口 / DC-3 前半）。
- **M3**：双通道接线 + 98F 收敛 + corpus 分批（DC-3/DC-4 前半）。
- **M4**：零回归复测收口（DC-4）。

## 依赖约束（run-rules §9）

- 与 43 号（net-api-compat-v3）共享 fetch 通道——缓存层改动同窗复验
  v2 终账（11,040 + S 系列）逐 subtest 零回归；43 号 request-cache 账
  以本 goal 交付承接（两 goal 控制面互指）。
- 与 46 号（runner-net-infra）py 端点前置——http-cache corpus 批次排期
  对齐 46 号 M3 框架交付。
- 缓存语义实现处加 RFC 9111 条款锚定注释；涉及导航资源加载的缓存路径
  改动与 rendering-compat 的加载性能门（benchmarks perf-gate）同窗核对，
  防缓存命中路径改变加载时序预算。
- 无人值守构建/测试走 test-guard 与 systemd-run 内存墙钟双限。
