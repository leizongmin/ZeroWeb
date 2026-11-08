# M0 对账——代码现状核实、102F 分簇、corpus 盘点、数值门

**日期**: 2026-11-08（M0，git main @ 68684de21）
**方法**: 控制面基线（2026-11-04）逐条对当前源码核实；request-cache 族定向实跑
（`make testharness-net-api FILTER=request-cache`，test-guard 包裹）；上游浏览器
通过率取 wptfyi-cache 10-29 summary（Chrome）。

## 一、控制面基线修正（重要）

控制面「基线事实」写于 2026-11-04，其中 **「引擎无 HTTP 缓存层」已过时**。
当前源码实况：

| 控制面原判断 | 当前实况（2026-11-08 核实） |
|---|---|
| 引擎无 HTTP 缓存层，无 freshness/条件请求语义 | `zero-net` 已有完整缓存层：`http_cache.rs`（1846 行，内存+磁盘分层、freshness、条件再验证、304 元数据合并、Vary 匹配、must-revalidate、按顶级站点分区、负缓存）、`cache_policy.rs`（370 行：Cache-Control/Expires/Age/Date/s-maxage 私有缓存忽略、`Vary: *` 不可存）、`cache_key.rs`、`disk_cache.rs`（1031 行）、`negative_cache.rs`。RFC 9111 条款锚定注释已随代码落地（§4.1/§4.2.1/§4.2.3/§5.2 等，R3233/R3235/R3392 修复记录在注释） |
| fetch/xhr 直连网络栈 | 浏览器 fetch/xhr 通道 **default 模式已经过缓存层**：shim `__zw_fetch` → handler（`apps/renderer/src/js_worker.rs` `default_fetch_handler` / `apps/browser/src/tab_js_worker.rs` 同构）→ `ResourceLoader::submit_http` → GET 无体请求转 `ResourceRequest` 走 `ResourceLoader::submit` 缓存路径（freshness 命中/条件再验证/304 展开）；POST 等非 GET 走直连 + RFC 9111 §4.4 失效（`unsafe_invalidation_targets`） |
| request.cache 六模式态全缺 | **部分存在**：shim 已解析并校验 `init.cache` 六枚举 + `only-if-cached` 须配 `mode:'same-origin'` 的 TypeError（`part01.js:7040-7084`）；net 侧 `CacheMode` 已有 4 态（Default/NoCache/NoStore/OnlyIfCached），OnlyIfCached 未命中已有 504 语义（`"only-if-cached cache miss (504)"`）、NoStore 跳过读写、NoCache 强制再验证；另支持从请求头 Cache-Control 推导模式（`request_cache_mode`） |

**仍然成立的三处缺口**（本 goal 实际剩余工作面）：

1. **模式态传播断裂**：JS `cache` 模式在校验后**从未传出 JS**——`__zw_fetch`
   wire 十参数（id/method/url/headersWire/body/clientId/referrer/mode/redirect/
   credentials）无 cache 字段，`FetchRequest` 结构无 cache 字段；net `CacheMode`
   缺 `Reload`/`ForceCache` 两态。用户请求头里恰好带 `Cache-Control: no-cache`
   时净效果碰对，其余模式（reload/force-cache/only-if-cached）网络侧完全不生效。
2. **runner 通道无缓存层**：WPT testharness 的 fetch handler
   （`wpt_data_fetch_handler`）是文件服务 + 逐案 py fixture shim，**不经过
   zero-net HttpCache**——request-cache 族的「服务器侧观察到的请求次数」断言
   （stash 计数）需要 runner 通道在 fixture 前面有真实缓存语义。
3. **cache.py fixture 缺失**：request-cache 七文件全部引用
   `resources/cache.py`（wptserve stash 端点），runner 语料只有
   `request-reset-attributes-worker.js` 一个文件——全部请求 404。

## 二、request-cache 族实账与分簇

### 实跑账（2026-11-08，window 变体）

```
Fail 102 / Pass 0 / Timeout 0
  request-cache-default-conditional  40F
  request-cache-force-cache          16F
  request-cache-only-if-cached       14F
  request-cache-reload               12F
  request-cache-no_store              8F
  request-cache-default               8F
  request-cache-no_cache              4F
（filter 同窗另有 eventsource/request-cache-control.any.js 4P——非本族）
```

失败样本全部为 `404 Not Found`（cache.py 缺位的 fixture 缺口形态），无语义级
红腿。分母 102 与 wpt.fyi 10-29 snapshot Chrome 的 window 变体分母逐文件同构
（40/16/14/12/8/8/4）——**地址池 = 102 子测试**。v2 账面 98F → 现 102F 为
上游语料漂移（+4，only-if-cached 族扩展），以 102 为准记账。

### 分簇（goal 定义三分类）

| 簇 | 文件（window 变体分母） | 依赖引擎语义 | 状态 |
|---|---|---|---|
| A. fixture 缺口（全部现红的直接根因） | 七文件全部 102 | cache.py 端点仿真（token stash 状态机：逐请求记录 If-None-Match/If-Modified-Since/Pragma/Cache-Control 观察头；`querystate` 取走 JSON；ETag/Last-Modified/Expires/Vary/Cache-Control 按参数回显；304 条件命中；302 redirect；ACO `*`） | M1 前置：runner 内嵌 cache.py handler（先例充分：get-set-cookie.py 案级 stash、preflight-cache WPT_TOKEN_STASH take/put、vary.py） |
| B. request.cache 模式态（engine 主增量） | reload 12 / force-cache 16 / only-if-cached 14 / no-store 8 / no-cache 4 = 54 | wire 传播 + net CacheMode 扩 Reload/ForceCache + 六态在缓存层的读写矩阵（含 only-if-cached stale 条目语义） | M2/M3 主交付 |
| C. 缓存头语义（已有实现待 fixture 显影） | default 8 / default-conditional 40 = 48 | freshness/条件请求/304/Vary——zero-net 已实现，runner 通道接缓存层后由 fixture 显影 | M1/M3 接线后收敛 |
| D. 上游语料冲突 | 暂记 0 | — | fixture+模式态落地后 M1/M2 归因时落定；当前无语义级红腿证据 |

依赖注记：default-conditional 40 与 reload/no-cache/force-cache 各文件均为
etag/If-None-Match 与 lm/If-Modified-Since 双变体结构（helper 注释：每案双跑
独立 token）；only-if-cached 族含 redirect 臂（cache.py 302 + ACO `*`，跨源臂
走 shim 既有 CORS 面）。

### 上游同簇浏览器通过率（数值门输入）

wptfyi-cache 10-29 summary-chrome.json，window 变体（`.any.html`）：
**102/102 = 100%**（逐文件全绿）。serviceworker/sharedworker/worker 变体
runner 无管道（既有「无 worker 文档管道」口径），不入地址池。

## 三、http-cache corpus 盘点（`fetch/http-cache`，上游 2026-11-08 master）

控制面原文写「fetch/api/http-cache 目录」——上游实际路径为
**`fetch/http-cache`**（`fetch/api/` 下无此目录），控制面已按此修正。

### 文件清单与 Chrome 通过率（window 变体）

| 文件 | Chrome (window) | py 端点依赖 | runner 执行面判定 |
|---|---|---|---|
| 304-update.any.js | 5/5 | http-cache.py | 可执行（py 仿真） |
| cache-mode.any.js | 5/5 | http-cache.py | 可执行 |
| cc-request.any.js | 4/12 | http-cache.py | 可执行；Chrome 也仅 33%——语义面（请求 CC 复用约束 max-age/min-fresh/max-stale），净增面 |
| credentials.tentative.any.js | 0/2 | http-cache.py | Chrome 全红（上游浏览器也未过）——按 DC-4 规则「×上游通过率」自然归零 |
| freshness.any.js | 17/17 | http-cache.py | 可执行 |
| heuristic.any.js | 11/17 | http-cache.py | 可执行；启发式新鲜度（RFC 9111 §4.2.2 Last-Modified 10%）——net 未实现，新语义面 |
| invalidate.any.js | 6/15 | http-cache.py | 可执行；失效面（POST/304 invalidate 矩阵） |
| no-vary-search.tentative.any.js | 13/13 | http-cache.py（No-Vary-Search 头） | 可执行；tentative（No-Vary-Search 是 URL 语法扩展头，缓存键面） |
| partial.any.js | 3/9 | http-cache.py | Range/206 面——net 无 range 存储；Chrome 也仅 33% |
| post-patch.any.js | 0/2 | http-cache.py | Chrome 全红 |
| pragma-no-cache-with-cache-control.html | 1/2 | cached_pragma_rand.py / cached_pragma_immutable_rand.py | 可执行 |
| split-cache.html | 4/4 | —（partition 断言） | **popup 依赖**（window.open 子窗 + iframe）——runner 无多窗口管道，M0 判排除（同 redirect-keepalive multi-window 口径），重入挂 46 号后评估 |
| status.any.js | 26/26 | http-cache.py | 可执行 |
| vary.any.js | 13/14 | http-cache.py | 可执行 |
| basic-auth-cache-test.html | 0/0 | securedimage.py | Chrome 分母 0（basic-auth popup 面）——不入地址池 |
| http-cache.js（helper） | — | — | 随批导入 |

resources/：http-cache.py（5141B，stash 状态机端点，同 cache.py 形态但含
per-request 变体语义）、cached_pragma_rand.py（599B）、
cached_pragma_immutable_rand.py（630B）、securedimage.py（709B）、两个 popup
html（split-cache 用，随排除不入账）。

### 账面汇总（window 变体口径）

- Chrome 全簇 417/554 = 75.3%
- 全绿面（fixture 到位即可平移）：304-update 5 + cache-mode 5 + freshness 17
  + no-vary-search 13 + status 26 = 66
- 净增语义面（Chrome 未满，但子测试在账）：cc-request +8、heuristic +6、
  invalidate +9、partial +6、pragma-no-cache +1、vary +1 = +31 分母（Chrome
  在这些面合计 39/70——本 goal 按 DC-4 规则「×上游同簇通过率」落门，不为
  上游浏览器自身未过的语义背账）
- 排除面：split-cache（multi-window）、basic-auth-cache-test（分母 0）、
  serviceworker/sharedworker/worker 变体（无管道）

**corpus 导入前置**：`fetch/http-cache` 目录未在 `NET_API_CORPUS_SUBDIRS` /
`20-net-api-compat.sh` DIRS（20 号脚本显式不拉记账），py 端点 shim 机制上
本 goal 可按既有先例逐案落（http-cache.py 与 cache.py 同为 stash 状态机形态），
但 goal 划界明确「corpus 的 py 端点由 46 号解锁，本 goal 只消费」——批次排期
对齐 46 号 M3 框架交付，本 goal 不代做。request-cache 族的 cache.py 属 102F
收敛自身前置（族已在账、收敛归本 goal），按 v2/v3 逐案 fixture 先例落，不算
代 46 号做 corpus 基建。

## 四、DC-4 数值门（落定）

按立项决策规则（地址池 × 上游同簇浏览器通过率）：

1. **request-cache 族门（M3/M4 验收）**：地址池 102（window 变体）× Chrome
   100% → **102/102 = 100%**。runner 通道实现引入的确定性偏差（如时间敏感腿）
   逐腿归因记账，不批量放宽；上游语料冲突腿（D 簇）按逐腿 wpt.fyi 佐证排除，
   排除后分母同缩。
2. **http-cache corpus 门（M3/M4，按批次）**：每批导入面 **window 变体实跑
   pass ≥ 实跑分母 × Chrome 同簇通过率**（逐文件取 wptfyi 10-29 基准数，
   见第三节表）；corpus 整体在账分母预期 ≈ 86（102 族外全绿 66 + 净增 31 中
   可执行面）±语料漂移，实数以 46 号解锁后批次导入为准。covered > 0 为
   corpus 通道存在性底线。
3. **零回归门**：v2 终账（11,040 + S 系列增量）与 v3 终账（13,256P）既有账
   逐 subtest 对照零回归；fetch 头条不低于 v3 复测值 2,266P/17.07%（wpt.fyi
   universe 口径）。

## 五、核实方法与证据

- 定向实跑：`make testharness-net-api FILTER=request-cache`（2026-11-08，
  test-guard 包裹，release runner @ 68684de21）→
  `/tmp` 会话日志（本轮会话产物，不落仓——计数已录本档表格）
- 源码核实：`crates/net/src/{http_cache,cache_policy,cache_key,disk_cache,
  negative_cache,resource_loader,request}.rs`、`crates/engine/src/fetch_bridge.rs`、
  `crates/engine/src/js_dom_shim/part01.js`（Request 构造 7040-7145、fetch 派发
  6502）、`apps/{renderer/src/js_worker.rs,browser/src/tab_js_worker.rs}`、
  `tests/wpt-runner/src/testharness.rs`（NET_API_CORPUS_SUBDIRS 4494-4547、
  wpt_data_fetch_handler 7205、fixture 先例 9861）
- 上游事实：wpt master `fetch/http-cache` 目录 + `fetch/api/request/resources/
  cache.py`（GitHub contents API，2026-11-08，GITHUB_TOKEN 认证）；
  wptfyi-cache 10-29 summary-chrome.json（仓内缓存快照）
- 历史账面：`docs/goal/archive/net-api-compat-v2.md`（98F 口径）、
  `net-api-compat-v3.md`（13,256P 终账、request-cache 归 47 号划界）
