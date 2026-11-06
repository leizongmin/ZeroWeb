# Network API 兼容三期 — fetch 族未导入目录 + websockets 全量 + xhr 残余

**版本**: v1.1
**日期**: 2026-11-04（立项，coverage-2026-11-02 对账驱动，用户「按建议立项」）；
2026-11-06 M0 对账甄别完成（三甄别档落 evidence/，DC-4 数值门按 <10k 分支落定）
**状态**: Active——M0 完成（三甄别档 + 数值门 <10k 分支），M1a 完成
（枚举器窗体解锁 +1,624P 零回归），M1b 完成（redirect 批次导入 +264P
零回归 + `.h2.` 排除记账），M2 进行中——websockets interfaces 批次与
auth 面批次已落（interfaces 74 案 +87P；auth 面 +15P 零回归，fetch auth
面全绿，xhr auth 12F 五簇归 M3），余 getResponseHeader 响应头控制面/
CORS preflight 端点扩表
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

**DC-4 数值门（2026-11-06 M0 落定，走 <10k 分支）**：M0 甄别（三档见
evidence/）判定三族地址池（window 可执行 + fixture 可 shim）≈ 1.5k–3k
子测试，**远低于 10k**——不设单点总量门，改按批次落门：

1. **批次门**：每批导入面通道实跑 pass ≥ 该批 ZW 实跑子测试分母 ×
   98.5%（M0 实测通道既有通过率下限：websockets 100% [377P/0F]、xhr
   98.7% [586P/7F/1T]、fetch 腿 90.9% [1,900P/2,091，含 request-cache
   族 98F 等划界挂账面——批次门只对新增导入面计，不与挂账面混算]）；
   批次导入面 wpt.fyi den 口径含变体膨胀，不做 den 口径对齐承诺（xhr
   档第五节记账）。
2. **DC-3 面门**：websockets 非 wss 依赖面 covered > 0 且 interfaces
   批次（den 291）pass ≥ 实跑分母 × 98.5%；wss 变体面移交 46 号兑现，
   不入门。
3. **零回归门**：一期/二期通道既有账逐 subtest 对照（M0 基线
   11,035P/12,078 见
   [基线档](net-api-compat-v3/evidence/2026-11-06-m0-channel-baseline.md)；
   M1a 后换代为 12,659P/14,221——换代经 A/B 交集 12,070 对逐对字节一致
   证零回归，见 [M1a 档](net-api-compat-v3/evidence/2026-11-06-m1a-enumerator-forms-unlock.md)；
   后续批次 A/B 以同树重跑对照为准）零回归；头条不回落（三族 coverage
   通过率不低于 2026-11-05 基线：fetch 14.25% / websockets 4.27% / xhr
   23.75%）。

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

- **M0 ✅**（2026-11-06）：对账甄别 + WebSocket P3 重估 + 数值门（DC-1）——
  三甄别档落 evidence/（fetch 逐目录 / websockets 清单+wss 交接 / xhr
  残余簇归因）；wss 面不阻塞非 wss 导入（interfaces 74 案零 .https 命名）。
- **M1**：**M1a ✅**（2026-11-06，[证据档](net-api-compat-v3/evidence/2026-11-06-m1a-enumerator-forms-unlock.md)）——
  枚举器 `extra_forms` 门解锁 `.htm`/`.window.js`（默认关，仅 net-api 通道
  开启）：通道 **+1,624P**（11,035 → 12,659P；url +1,272 / xhr +317 /
  eventsource +22 / fetch +12 / streams +1），案 543 → 684，A/B 交集
  12,070 对逐对字节一致零回归；新 Fail 面 = 解锁面真实缺口显影（
  access-control preflight 端点 / h1-parsing 原始线缆端点等），转 M1b/M2
  fixture 分批与 M3 归因输入。**M1b ✅**（2026-11-06，[redirect 批次档](net-api-compat-v3/evidence/2026-11-06-m1b-redirect-batch.md) +
  [簇归因图](net-api-compat-v3/evidence/2026-11-06-m1b-folder-txt-and-xhr-cluster-map.md)）：
  folder.txt 补拉（+2P）；fetch/api/redirect 14 案导入（+364 子测试
  264P/99F/1T）+ redirect.py fixture 升上游 per-token stash 语义（键
  命名空间化防 preflight.py 误读，曾致 cors-redirect 20 案 P→F）+
  redirect-empty-location.py / method.py 新 fixture + fetch shim 六项
  重定向修复（error/empty-location/data-scheme/循环上限/303 HEAD/redirected）；
  全量 A/B 12,655 → 12,924P，changed 15 全 F→P 零 P→F；`.h2.` 形态案
  正式排除（26 子测试移出分母）；redirect 残余 99F+1T 七簇归 M3
  （referrer 63 / mode 11 / origin 10 / method 9 / escape 4 /
  back-to-original 2 / keepalive 1T），xhr redirect 引擎缺口并簇 M3。
- **M2（进行中，[ws interfaces 批次档](net-api-compat-v3/evidence/2026-11-06-m2-ws-interfaces-batch.md)）**：
  websockets/interfaces 74 案导入（+132 子测试 88P/45F，全量 A/B
  changed 0 零回归）+ ws_fixture `/echo_raw`（裸字节回写，保留 opcode
  注入面）与 `/sleep_10_v13`（握手延迟 10s）两新端点——close-connecting
  10s 时序面一次通过；45F 九簇全引擎侧归 M3（接口常量 ReadOnly 16F /
  EventTarget dispatchEvent 9F / 属性描述符 6F / 事件处理器 handleEvent 形
  4F / send 状态机 3F / 协议错误断连 3F / CONNECTING fail 语义 2F /
  事件序 1F / CloseEvent ctor 1F），批次门 66.2% 随 M3 收敛复验。
  **auth 面批次 ✅**（2026-11-06，[证据档](net-api-compat-v3/evidence/2026-11-06-m2-auth-face.md)）：
  auth fixture 套件（xhr authentication.py + auth1-11 三变体 token stash
  挑战流 / fetch authentication.py 固定凭据版 / dump-authorization-header
  OPTIONS strip_auth_header 500 探测臂）+ 引擎 cors check 按 spec 步序
  重排（`*` 通配先行，opaque origin 仅换比对基准——`null`；曾致
  cors-redirect 十案回退的中间迭代见
  [learning](../../learnings/bugs/2026-11/2026-11-06-cors-check-asterisk-step-precedes-origin-compare.md)）。
  全量 A/B changed 17（16 F→P + 1 诚实翻转 send-authentication-basic-cors
  vacuous P）零 P→F 外溢，+15P 至 13,005P；fetch auth 面全绿 7P，
  xhr auth 12F 五簇（URL/open() userinfo 凭据 8F / 401 挑战会话 2F /
  跨源透明 auth 拒绝 1F / preflight ACAH enforcement 1F）归 M3。
  余 M2：getResponseHeader 响应头控制面 / CORS preflight 端点扩表。
- **M3**：redirect 引擎语义簇收敛（fetch redirect 批次 99F+1T 七簇 +
  xhr redirect 残余——referrer 63F / redirect-mode 11F / origin 10F /
  method 9F / location-escape 4F 等）+ ws interfaces 九簇（WebIDL 常量/
  属性描述符基建、EventTarget dispatchEvent、fail-connection 语义等）+
  xhr auth 五簇（URL/open() userinfo 凭据解析 8F / 401 挑战流 UA 会话 2F /
  跨源透明 auth 拒绝 1F / preflight ACAH enforcement 1F）+
  xhr 在账失败簇收敛（formdata/submitter 簇 4F + overrideMimeType
  Shift-JIS 接线 1F + cookies.http.html 挂起 1T；permissions-policy 2F
  策略面排除）（DC-3）。
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
