# Network API 兼容三期 — fetch 族未导入目录 + websockets 全量 + xhr 残余

**版本**: v1.0
**日期**: 2026-11-04（立项，coverage-2026-11-02 对账驱动，用户「按建议立项」）
**状态**: Active——立项，M0 未启动
**执行模式**: WPT 驱动（上游 corpus 为验收标尺）+ 语料扩面 + fixture 甄别
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——网络面）
**前账**: 一期 `docs/goal/archive/net-api-compat.md`（2026-10-04 收口）；
二期 `docs/goal/archive/net-api-compat-v2.md`（2026-10-29 收口，通道七域
基线 9,354 → 11,040，收口后小片 M2-S5..S19 持续 +40 级增量）

## coverage 对账（coverage-2026-11-02 基线，全量头条 67.54%）

| 面 | 全量分母 | zw 覆盖 | 通过率 | 未覆盖 |
|---|---:|---:|---:|---:|
| fetch | 13,274 | 133/957 文件，1,894P | 14.27% | **11,380** |
| websockets | 2,389 | 8/799 文件，102P | 4.27% | **2,287** |
| xhr | 2,333 | 110/440 文件，554P | 23.75% | **1,779** |

三族合计未覆盖 **15,446**。通道现状：`testharness-net-api` 已入
NET_API_CORPUS_SUBDIRS 十四目录（fetch/api/{headers,request,response,
body,basic,abort,credentials,cors} + data-urls/content-type/content-length/
h1-parsing/images/redirects 等）——**fetch/api 之外的上层目录与
fetch/api 内 py 端点密集目录从未导入**（http-cache、redirect py 面、
privacy、corp、streams、nosniff、range、multipart、schemes、destination、
vintage 等，M0 逐目录甄别）；websockets 仅 8 文件（?wss 变体面被 scheme
门筛减）；xhr 余量未收。

前账挂账锚点（2026-11-04 用户拍板拆分落位）：

- **WebSocket wss 面**（TLS 端点 fixture + 通道 scheme/端点寻址）——
  **已拆基建 goal 46 号 runner-net-infra**（v2 挂账的两前置审计随其 M0
  落地）；本 goal 只收**非 wss 依赖面**，wss 面以 46 号 DC-2 为前置。
- **request-cache 族 98F**——**已拍板立项 47 号 http-cache**（引擎
  HTTP 缓存层）；本 goal 沿 v2 剔除口径，47 号交付后由其承接收敛。
- 一期/二期零回归底线继续有效（通道分母逐 subtest 对照）。

## Mission

1. **对账甄别**（M0）：fetch 未导入目录逐目录甄别——纯 window 面、
   `.py` wptserve 端点依赖面（fixture shim 可行性——v2 已有 echo-content/
   /echo-query 等先例）、worker 执行面（如实排除）；websockets 全量文件
   清单与 ws fixture 端点需求；xhr 残余失败簇。
2. **fetch 目录批量导入**（M1/M2）：过 net-api 模板（DIRS 扩面 + skip
   规则 + Makefile target + fixture shim 按需），py 端点面分批。
3. **websockets 非 wss 面入账**（M2/M3）：非 wss 依赖面全量入账；wss
   依赖面清单移交 46 号（fixture 需求 + 寻址形态），不阻塞本 goal。
4. **xhr 残余收敛 + 复测收口**（M3/M4）：失败簇归因；coverage 复测落
   数值门。

## Done Criteria

- **DC-1（M0 对账）**：fetch 未导入目录 + websockets 全量 + xhr 残余的
  构成甄别落 `docs/goal/net-api-compat-v3/evidence/`（逐目录：文件数、
  py 端点依赖、批次归属）；wss 依赖面清单与 46 号交接档落档；DC-4
  数值门落定并回填本文件。
- **DC-2（通道扩面）**：M1/M2 批次目录入 testharness-net-api 常驻通道
  （DIRS + skip + Makefile + fixture 四件套）；新增 fixture 走 v2 先例
  （路由表 + echo 族）。
- **DC-3（websockets/xhr）**：websockets 非 wss 依赖面入账（covered > 0
  且通过数达数值门）；wss 面交接 46 号（清单 + fixture 需求）落档；xhr
  残余簇归因落档。
- **DC-4（coverage 复测）**：fetch/websockets/xhr 三面升至数值门；
  一期/二期通道既有账零回归（分母逐 subtest 对照）；头条不回落。

**DC-4 数值门决策规则（立项即定）**：M0 甄别后按地址池分档——地址池
（window 可执行 + fixture 可 shim）≥ 10k 子测试：全量推进，数值门 =
地址池 × 通道既有通过率（M0 实测后填入）；< 10k：优先导入非 py 端点
目录，py 密集目录按 fixture 可行性分批，worker 面如实排除挂账。

## 划界

- **url/mimesniff 不重开**：v2 收口账（IdnaTestV2 100% / url 94.1% /
  mimesniff 残余语料冲突记账）零回归门；重入条件仍归 v2 口径。
- **request-cache 98F 归 47 号**：http-cache goal（2026-11-04 立项）承接
  收敛；本 goal 沿 v2 剔除口径，不重复实现缓存语义。
- **wss 面归 46 号**：TLS 端点 fixture 与 scheme/端点寻址归
  runner-net-infra；本 goal 只消费其交付。
- **CORS 引擎语义不重开**：`crates/security/src/cors.rs` 既有实现（s153）
  零回归门；cors topdir 专面通道归 gap-batch1（39 号）。
- **service-workers fetch 拦截面**归 service-workers 通道既有账；本 goal
  改动同窗复验其零回归。
- **encoding/streams** 归 37 号 encoding-compat-v2 划界（ReadableStream
  面不属本 goal）。

## 里程碑

- **M0**：对账甄别 + WebSocket P3 重估 + 数值门（DC-1）。
- **M1**：fetch 非 py 目录批次导入（DC-2 起步）。
- **M2**：py 端点目录 fixture 分批 + websockets 非依赖面入账（DC-2/3）。
- **M3**：xhr 残余簇归因收敛（DC-3）。
- **M4**：coverage 复测收口（DC-4）。

## 依赖约束（run-rules §9）

- net-api 通道改动（DIRS/skip/fixture/shim）后，一期/二期通道基线
  （v2 终账 11,040 + S 系列增量）逐 subtest 零回归复验。
- ws fixture 与 ws_host_factory 接线只挂 websockets 用例（run-rules §10
  通道级隔离先例），CSP/cookies 通道 WebSocket 用例零耦合。
- weekly 分片预算：fetch 大目录单套件耗时对照分片表，必要时套件级分批
  （form-encode 先例：单套件单进程超 14400s 即拆）。
- fixture shim 语义改动（路由/echo 族）先落 evidence 再合，防 v2 通道
  既挂 fixture 的面（S3b `/echo-query_v13` 等）回归。
