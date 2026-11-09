# M4 终账：DC-4 下游消费验收 + 零回归收口

- 日期：2026-11-09（M3 收口同日续轮）
- 基树：4d9128d12（M3 收口 docs 提交后 HEAD；代码态 = 870689dcf 6b 落库树，
  与当日 net-api/行为面 B 跑二进制一致）
- 承接：[wss 审计档](2026-11-08-m0-wss-audit.md) §四 + [py 普查档](2026-11-08-m0-py-endpoint-census.md)
  §四（M0 落定门型）+ 控制面「数值门」「零回归复验面」两节。

## 一、wss 池数值门

- **covered = 101 键 > 0**（M2 落证，[M2 档](2026-11-08-m2-wss-wiring.md) §二；
  普查 102 的 cookies/003-005 预排除归因同档）。
- **批次门 = 实跑分母 × 98.5%**：终账实测（2026-11-09 net-api 全量 A/B
  双跑提取）`?wss` 变体子测试 **489/489 Pass = 100.00%**，双侧（A 锚
  87efb6aa9 / B 终态树）同值——超门且零漂移。
- h2 变体 92 实例维持排除（分母不计）；mixed-content.https（引擎
  mixed-content 缺口并记）与 third-party-cookie-accepted（语料 helper
  缺口并记）不入门如实落账（M0/M2 已记，本账无变化）。

## 二、py 池数值门

- **P-A 三目录池（47 号 http-cache，den 606）**：注册面 =
  `testharness_py_routes/http_cache.rs` `route_http_cache_py`（主表
  mod.rs:138 行）+ `route_cached_pragma_py`（pragma-no-cache 对照端点
  ×2）；单测钉 = `http_cache_py_endpoint_state_machine_dc4_pa_pool`
  （本批新增，经主表注册面端到端：OPTIONS preflight / 缺 uuid falsy
  404 / dispatch=test 状态序列 Test-Requests base64 配置按发序逐条取 +
  Server-Request-Count 递增 + NOTEHDRS 记名 / dispatch=state 取走即清
  二次取 null 跨案零泄漏）+ http-date helper 三钉（模块内）；下游互指
  ✅（47 号 corpus 门承接互指，2026-11-08）；动态证据 = 47 号语料批次
  首批 124P/138 入账（30584b1e7）+ 本日 net-api A/B http-cache 域双侧
  全等（A 锚 vs 切片五锚 +138 与 30584b1e7 账逐字对上）。
- **P-B 判据族（39 号 gap-batch1，referrer-policy/preload）**：注册面 =
  script fetcher 表 T6/T12/T13（referrer-checker 三路径 + echo-referrer，
  6b 落库）+ 主表 preload 臂；单测钉 = s116 referrer-checker endpoint
  and host alias + s116 module referer policy table + s268 data-module
  referrer checker sims + s270 referrer checker img endpoint sims +
  s124 modulepreload endpoint sims（≥5 例）；下游互指 ✅（39 号 P-B
  判据族互指，2026-11-08）。
- 门裁决：「框架承接面落地即解锁」（M0 门型）——两族注册面 + 单测
  各 ≥1 例 + 互指三要素齐备，**py 池门过**；导入实数归各 goal 账
  （47 号已入账），本 goal 不代背。

## 三、等价硬门 + 零回归复验面（五面终账）

锚：A = `/tmp/m3s6b-base-run.log`（87efb6aa9，13,993P/1,321F/9T =
15,323 行）vs B = 终态树全量跑（`/tmp/m3s6b-b-run.log`），per-subtest
计数敏感 multiset：

1. **websockets 通道既有面**：A/B 双侧全等（websockets 域 Pass 999
   行，multiset 内零差异；M2 锚 110 案 381 行被全通道面覆盖并扩展）。
2. **net-api 全通道**：changed = 1——`xhr/send-timeout-events.htm`
   「timeout is not 0」P→F；等负载受控探针（并行编译腿 load 465 窗）
   HEAD 3/3F vs B 3/3F 同型 → 环境归因（6b 档 §四.4），等效判定成立。
3. **S3b `/echo-query_v13`（constructor/016）**：4 行 A/B 全等。
4. **S12 cors-preflight-redirect 面**：10 行 A/B 全等。
5. **inspect-headers.py POST 豁免面**：现行 net-api 套件组成 0 行
   （双侧同缺——行使案不在 NET_API_CORPUS_SUBDIRS 拉取面）；注册面在
   主表（mod.rs:571 `route_inspect_headers`），普查档 §三.1④ 迁移 A/B
   全等承证；套件缺席对称，changed 集不增。
   另：**行为面**（html-semantics FILTER=the-script-element，script
   fetcher 14 臂行使）473 行 multiset 空集（6b 档 §四.5）。
- 汇总：**changed 空集达成**（唯一 1 项环境归因已探针钉死），五面 +
  行为面零回归落证。

## 四、头条不回落

net-api 通道头条：切片五锚 13,614P → 终态锚 13,993P（+379 = http-cache
批次 +138 + websockets 面 +258 域级归因 + 零星）→ B 终态树与锚 multiset
等价——**P 头条不回落** ✅。

## 五、门账汇总（DC-4 三项）

| 门 | 判据 | 结果 |
|---|---|---|
| wss 池 | covered > 0 + 批次门 ≥98.5% | ✅ 101 键 + 489/489 = 100% |
| py 池 | P-A/P-B 注册面 + 单测 ≥1 例/族 + 互指 | ✅ 两族齐备（P-A 钉本批补齐） |
| 等价硬门 | A/B changed 空集（wss 面声明除外） | ✅ 五面 + 行为面落证，唯一 1 项环境归因 |

**DC-4 ✅**——goal 四项 DC 全过，收口。
