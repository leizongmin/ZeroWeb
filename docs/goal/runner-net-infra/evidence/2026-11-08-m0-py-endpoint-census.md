# M0 审计二：py 端点依赖普查（逐下游 goal 可 shim / 需动态 / 排除分类）

日期：2026-11-08。方法：wpt-data 本地树逐案 `.py` 引用提取 × testharness.rs
既有 inline shim 注册面比对 × 下游 goal 控制面（43/39/47 号）划界核对。
上游 wptserve 语义基准 rev 同 wss 审计档。

## 一、现状：py 端点承接面已成的既定事实

- **runner 无 wptserve，全量 py 端点走 Rust 等价 fixture**：语料树内
  `fetch/`、`xhr/` 目录 **0 个 `.py` 文件落盘**——所有端点在 runner 内按案
  shim。testharness.rs 内联 dispatch（`wpt_data_fetch_handler_with_stash`
  闭包，testharness.rs:7217 起）现有 **153 处 `.py"` 引用匹配臂**，横跨
  fetch handler / script fetcher / worker script fetcher / service worker
  fetcher 四个闭包，体量 ~3.5k 行（testharness.rs 7217-10700 段主体）。
- 先例按 v2/v3 逐案累积：echo-content 族、inspect-headers.py（POST 豁免 +
  headers 回显）、redirect.py（per-token stash 命名空间化）、
  authentication.py/auth1-11（401 挑战流）、preflight.py（OPTIONS 态机）、
  get-set-cookie.py（cookie 状态机表）、eventsource 五端点、beacon 三端点等。
- **S12 本地补丁**：cors-preflight-redirect 对齐为 testharness.rs 内联分支
  （8126/8132 段落穿重定向逻辑），语料仓 update 脚本缺口记账（见 §四）。

结论：**「逐案 shim」已到规模化临界**——端点间共享的 stash 语义、查询参数
解析（`wpt_query_value`/`_bytes`）、头集合操作、重定向/预检委派逻辑在闭包内
重复展开，新端点落点与既有分支的次序耦合（先命中先返回）使 A/B 风险随臂数
线性涨。框架化的必要性由分支密度本身成立。

## 二、逐下游普查清单

### P-A：47 号 http-cache（http-cache.md 划界「不代 46 号做 fixture」）

| 目录池 | case 数 | py 端点 | 分类 | 语料现状 |
|---|---:|---|---|---|
| fetch/http-cache | 19 | `http-cache.py`（v3 甄别档 py 引用 2） | **需动态端点**——上游 handler 含 stash/计数/延迟语义（缓存命中探测的响应态机） | **未拉取**（wpt-data 无该目录） |
| fetch/stale-while-revalidate | 7 | 6 案引用（v3 甄别档） | **需动态端点**（revalidation 计数语义） | 未拉取 |
| fetch/origin | 1 | `redirect-and-stash.py` | **需动态端点**（stash 跨请求编排；v3 判「shim 成本/收益不成比」挂账） | 未拉取 |

- den 口径合计 **606**（554 + 9 + 43，v3 甄别档第二节）。
- 精确端点行为清单以 47 号 M0 盘点为准（其语料拉取前置本 goal M3 框架）；
  本 goal 交付 = 框架使「动态 stash 端点」可注册可单测，47 号只写端点表。
- 同目录 `fetch/api/request/request-cache*` 98F 簇（v2 standing exclusion）
  的端点面已有 shim 在账（redirect/status 等），其收敛归 47 号缓存语义。

### P-B：39 号 gap-batch1 挂账簇（referrer-policy / preload 判据端点）

- gap-batch1 已收口归档，卡点面「referrer-policy/preload `.py` 判据端点
  仿真」挂 runner 基建（入口文档「当前卡点」节）。
- 判据端点形态 = **referrer-checker.py 家族**（判据回显：请求所带 Referrer/
  policy 头 → 断言文本），与既有先例同族：html/semantics 的 referrer-checker.py
  三变体已 shim（testharness.rs:6119-6121, 6361）、preload/resources/
  echo-referrer.py 已 shim（:6392）。
- 分类：**可静态 shim**（判据回显无跨请求 stash）；39 号若重启专面通道，端点
  落框架注册表即可，无新语义。

### P-C：fetch/api 未导入目录（v3 甄别档排除面复核）

v3 已将以下面**排除**（策略/栈本体/新 API 域，非 py 框架解锁对象）：
fetch/metadata（record-header.py 等 25 引用，Sec-Fetch-* 策略面）、
fetch/cross-origin-resource-policy（CORP 策略面）、fetch/nosniff（策略面）、
fetch/orb|corb（读取算法）、fetch/range（HTTP 栈）、fetch/content-encoding
（内容编码栈）、fetch/compression-dictionary 与 fetch-later（新 API）、
fetch/local-network-access（许可面）、fetch/connection-pool（栈）、
fetch/api/policies（CSP 策略面）、fetch/security（dangling-markup）、
fetch/redirect-navigate（多页编排）。

**本普查确认：这些面的阻塞不是 py 端点框架**——是引擎策略/栈语义面，46 号
不解锁（不代下游做语义收敛，划界）。其 py 引用只在对应引擎域 goal 立项时
经本框架注册。

### P-D：worker 面引用（照旧排除）

`.py` 端点被 worker 执行案（`.worker.js`/`importScripts` 链、service-worker
update 族 ~20 端点）引用者如实标记：**worker 面排除口径不变**（run-rules
既有通道先例；本 goal 不解锁 worker 管道）。涉及端点：resources/*-worker.py
族、update-*-worker.py 族、bytecheck-worker 族等——已由 service-workers
通道既有账/排除记账覆盖，框架迁移时保持臂位等价即可。

### P-E：websockets/handlers/*.py（34 文件——非 py 路由框架对象）

上游 wptserve WebSocket handler 服务端行为面：由 **ws_fixture 等价承接**
（模块注释逐文件对应），不进 py 路由表。wss 变体只改传输层（TLS 包裹），
handler 行为表不变——M1 泛化 IO 时逐 endpoint 行为零变化（A/B 复验）。

## 三、框架化范围（M3 实施蓝图）

**注册表形态**（对齐既有先例语义，不引入新行为）：

```
端点注册项 = { 匹配谓词 (路径全等 / 目录+文件名后缀，沿既有臂口径),
               方法豁免 (inspect-headers POST 先例),
               handler: Fn(&req, &EndpointCtx) -> FetchResponse }
EndpointCtx = { query 解析 (wpt_query_value/_bytes 复用),
                stash 表 (per-case Arc<Mutex<HashMap>>，v3 M3 cookie 表先例),
                头构造助手 (wpt_add_fetch_metadata、CORS 回显族) }
```

- **迁移案（DC-3 硬门）**：既有 153 臂全部迁入注册表，行为**逐字节等价**
  ——迁移前先固化 A/B 基线（net-api 13,275P 面 + 涉 fixture 的其他通道面），
  迁移后逐 subtest 对照全等；臂序（先命中先返回）在注册表内保序。
- **S12 收编**：cors-preflight-redirect 内联补丁落注册表臂；语料仓 update
  脚本缺口（本地补丁对应的上游语料修正未回灌）在框架落地提交中清偿——
  核对语料 pin 树内该案形态，若上游已修正则销账，否则补 update 脚本条目。
- **模板**：reflector 族（inspect-headers/echo-content/判据回显类）抽公共
  模板臂，新端点 = 注册项声明，不再动 dispatch 闭包。
- 零回归面（DC-3/DC-4）：四既有 fixture 面——①websockets echo 族（377P 面）
  ②S3b `/echo-query_v13`（constructor/016）③S12 cors-preflight-redirect 面
  ④inspect-headers POST 豁免——迁移前后 A/B 全等。

## 四、DC-4 py 池数值门（M0 落定，回填入口文档）

- **解锁数**：P-A 三目录池（den 606）+ P-B 判据端点族——框架承接面落地即
  「解锁」（下游 goal 据此导入，导入实数归各 goal 账，本 goal 不代背）。
  门 = P-A stash 端点族与 P-B 判据族在框架上有注册面 + 单测各 ≥1 例，
  47 号/39 号控制面互指确认可消费。
- **等价硬门**（立项即定，不设通过率）：迁移案逐字节等价 100%——A/B
  changed 集为空（除本 goal 显式声明的 wss 面）。零回归面四项各自落证。
