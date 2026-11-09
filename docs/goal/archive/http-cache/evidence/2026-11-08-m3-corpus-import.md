# M3 corpus 批次——fetch/http-cache 端点表落地 + 首批入账（DC-4 corpus 门）

**日期**: 2026-11-08
**前置**: 46 号 runner-net-infra M3 框架交付（PyRoute 注册表化，切片一~五
已落 master；[py 端点普查档](../../../runner-net-infra/evidence/2026-11-08-m0-py-endpoint-census.md)
P-A 池分工「47 号只写端点表」）+ 本 goal M3 net 层语义（@ 72fcd2434，
zero-net lib 530P/0F）。

## 一、结论

- **DC-4 corpus 门达成**：`fetch/http-cache` 13 .any.js + 1 pragma html 入窗，
  终跑 **124P/138**（86.2%）vs Chrome 同簇窗 104/239（43.5%）——按 DC-4
  「实跑 pass ≥ 实跑分母 × Chrome 同簇通过率」聚合口径达标（聚合地板
  138 × 43.5% ≈ 60；按 10-29 窗绝对地板 104 亦达标）；covered = 138 > 0。
- **零回归门保持**：request-cache 族窗 **106P/0F**（= M2 账本
  族 102/102 + eventsource 4P，逐 subtest 一致）+ headers 族 **286/286**
  + zero-net lib **530P/0F**。
- **46 号 P-A 解锁面兑现**：http-cache.py 端点族在 PyRoute 注册表有注册面
  （3 行，先于非 GET 405 门）+ 端点模块自带单测 ≥1 例（http_date 三测），
  满足普查档「注册面 + 单测各 ≥1 例」验收口径（47 号侧消费账，46 号账面
  由其自记）。

## 二、交付面（端点表 = 本 goal 分工，框架 = 46 号已交付）

### 1. 端点表（新文件 `testharness_py_routes/http_cache.rs`）

- **`resources/http-cache.py` 1:1 移植**（上游 pinned rev
  315976933870b34d6ea30e3f6643403edae678ba）：
  - `Access-Control-Allow-Credentials: true` 分发前统一在场；OPTIONS
    preflight 先于 uuid 判定（ACO=origin 或 `*` / ACAM GET / ACAH=ACRM 或
    `*` / ACMA 86400）；缺/空 uuid → 404 `UUID not found`。
  - `dispatch=state`：stash take 的 JSON（取走即清空，缺记录回 `null`）。
  - `dispatch=test` 状态机：Test-Requests 头（base64 JSON）解析 →
    `requests[len(server_state)]` 取当前 config → response_headers 逐项
    magic 展开（Location/Content-Location 值拼 `request.url&target=`；
    Date/Expires/Last-Modified 整数值 → `now+delta` IMF-fixdate）→ 追加
    state（now/request_method/全部请求头小写化/NOTEHDRS 记名头）→
    ACO/CT 缺省补 `*`/text/plain → `Server-Request-Count` →
    expected_type 以 `validated` 结尾时按首条 state 记名 etag/LM 与请求
    If-None-Match/If-Modified-Since 全等生成 304，未命中改判 999
    `304 Not Generated` → 体 = response_body 或 uuid（204/304 置空）。
- **`cached_pragma_rand.py` / `cached_pragma_immutable_rand.py`**：CT
  text/html + X-XSS-Protection 0 + `Cache-Control: max-age=2592000,
  public|immutable` + `Pragma: no-cache` + 随机 token 体（SystemTime 纳秒
  + 进程内计数器；语料只比对两次体是否不同）。
- **HTTP-date 生成器**：IMF-fixdate（civil_from_days 算法，跨负值安全），
  与 zero-net 解析侧 `cookie::parse_expires_date` 互验单测 3 例。
- 案级 stash `HttpCachePyStash`（uuid → 请求态序列，take/put-back =
  remove/insert）入 `PyEndpointCtx`——per-case 构建、案内键自清、跨案零
  泄漏，与 cache.py 的 `CachePyStash` 同款。

### 2. 包裹器扩展（`wpt_cache_py_wrapped_handler`，M1-b 包裹器按端点路径逐个扩展的兑现）

- **门控扩 4 路径**：cache.py + http-cache.py + 两个 pragma 端点；无体 GET
  过缓存层不变。
- **非 GET 臂（新）**：门内路径 POST/PUT/DELETE/OPTIONS 本体直通 base 后，
  RFC 9111 §4.4 unsafe 落账镜像 loader（复用新 pub 的
  `zero_net::resource_loader::apply_unsafe_response_to_cache`——失效目标 +
  POST/PATCH Content-Location 存储，DC-3 单实现）。
- **模式推导镜像**（loader `submit` 同款）：wire `init.cache` 显式模式
  权威；None/default → 请求 CC 头推导（新 pub 的
  `zero_net::resource_loader::request_cache_mode`——only-if-cached/
  no-store/no-cache；cc-request 语料判据面）。
- **落盘不设 3xx/206 形态门**（与 loader `submit_network` 有意分歧）：
  request-cache 族 only-if-cached redirect 腿依赖 302+验证器/显式新鲜度
  落盘复用（M2 族账锁定行为；本批曾试镜像该门 → 族窗 4F 回归，回退后
  106P/0F 恢复，见 §四）。可存性本体仍由 `put_with_headers` 内部
  storable_mode 裁决。

### 3. 语料导入注册

- `NET_API_CORPUS_SUBDIRS` + `fetch/http-cache`。
- `net_api_case_skipped` 排除：split-cache.html（多窗口 popup +
  BroadcastChannel——M0 §三排除口径）、basic-auth-cache-test.html /
  -ref.html（basic-auth 弹窗，Chrome 分母 0，M0 §三不入池口径）。

### 4. zero-net pub 面（最小新增，均有 runner 消费方）

- `resource_loader::request_cache_mode`（原私有 → pub，包裹器推导复用）。
- `resource_loader::apply_unsafe_response_to_cache`（新函数——loader HTTP
  直发臂原内联编排收敛，与 runner 包裹器共用；loader 调用点行为逐字节
  等价，仅锁粒度不变）。

## 三、运行账（2026-11-08，window 变体，三跑：首跑 123P + 绑定跑两度 124P）

终跑（绑定 fmt 后提交树，/tmp 记录与提交树同语义）per-file 对比
（Chrome = wpt.fyi 10-29 窗，M0 对账档 §三）：

| 文件 | Chrome | 本地终跑 | 判定 |
|---|---|---|---|
| 304-update.any.js | 5/5 | 5/5 | =Chrome，满门 |
| cache-mode.any.js | 5/5 | 5/5 | =Chrome |
| cc-request.any.js | 4/12 | 11/12 | **+7** 超 Chrome（唯 1F = 504 状态码腿，Chrome 同簇亦 8F） |
| credentials.tentative.any.js | 0/2 | 0/2 | =Chrome（×0 率门自然归零） |
| freshness.any.js | 17/17 | 17/17 | =Chrome |
| heuristic.any.js | 11/17 | 17/17 | **+6** 超 Chrome |
| invalidate.any.js | 6/15 | 15/15 | **+9** 超 Chrome |
| no-vary-search.tentative.any.js | 13/13 | 13/13 | =Chrome |
| partial.any.js | 3/9 | 0–1/9 | 缺口=Range/206 面（M0 已判 net 无 range 存储；两跑 ±1 抖动见下） |
| post-patch.any.js | 0/2 | 0/2 | =Chrome（上游判据面自身失败——响应体断言期望 uuid 而缓存复用 "abc"） |
| pragma-no-cache-with-cache-control.html | 1/2 | 1/2 | =Chrome（public 腿 Chrome 同败） |
| status.any.js | 26/26 | 26/26 | =Chrome |
| vary.any.js | 13/14 | 13/13 | 本地 pinned rev 分母 13 全绿（10-29 窗 14 系上游漂移面） |

- 聚合：**124P/138**（86.2%）；首跑 123P/138、绑定跑两度 124P/138
  （partial 簇 8F/9F ±1 抖动，同为 Range 语义缺口面）。
- 超出 Chrome 的语义面全部来自本 goal M3 交付的净增面：失效矩阵（§4.4
  unsafe 落账 + Location/Content-Location 目标失效）、启发式新鲜度（LM×10%）、
  请求 CC 指示（max-age/max-stale/min-fresh lookup 消费 + only-if-cached/
  no-store/no-cache 头推导）。

## 四、失败面归因（余 14F）

1. **partial 8–9F**：Range 存储/服务面（206 切片、suffix/byte-range-spec、
   部分补全）——zero-net 无 Range 支持（RFC 9111 §3.1 不存部分内容为
   *正确* 行为，补 Range 支持另行立项）。与 Chrome 差 −3，聚合门吸收。
2. **post-patch 2F**：上游判据面自身矛盾（`assert_equals(resBody, uuid)`
   vs 缓存复用的 "abc" 体）——Chrome 全红同因，非引擎缺口。
3. **credentials.tentative 2F**：匿名/凭据混合计数面——Chrome 全红（M0
   §三已记 0/2），CORS+credentials 语义待后续批次评估。
4. **cc-request 1F（504 腿）**：期望 HTTP 504 响应；本引擎 only-if-cached
   miss 走 TypeError（fetch 网络错误语义，与 Chrome 对齐——Chrome 该腿
   亦红；`request-cache-only-if-cached` 族的 `response:["error"]` 判据
   同款语义，M1-b 已定账）。
5. **pragma 1F（public 腿）**：`assert_not_equals` 期望不被缓存——RFC 9111
   §5.4 Pragma 已弃用、响应侧无指令语义，CC max-age 胜出 → 缓存复用体
   相同 → 腿红；Chrome 同红（该文件唯 immutable 腿应过，本地同）。

## 五、零回归账（同轮实测）

| 窗 | 账本基线 | 本轮 | 判定 |
|---|---|---|---|
| request-cache 族窗（FILTER=request-cache） | 106P/0F（M2 账） | **106P/0F** | ✓（中途 102P/4F 回归经 §二.2 门回退修复，教训入 §四） |
| headers 族（FILTER=fetch/api/headers） | 286/286（M1-b 账） | **286/286** | ✓ |
| zero-net lib | 530P/0F（M3 账） | **530P/0F** | ✓（apply_unsafe_response_to_cache 收敛后行为等价） |
| runner 端点单测 | — | **3P/0F**（http_date 族） | 新增 |

- 影响面界定：包裹器门控为精确路径匹配（4 路），共享 fetch 通道其余路径
  零触碰；受影响窗 = request-cache 族 + 本 corpus + headers 抽窗，均已实测。
  全窗 A/B 复测留 M4 终验（本批改动面有界，无全窗重跑必要）。

## 六、过程教训

**落盘形态门镜像的零回归边界**：loader 的 `storeable_response_shape`
（3xx/206 不落盘）是 loader 通道自身的重定向链语义缺失补偿（FIXME
挂账），不是 RFC 9111 语义本体；runner 包裹器镜像该门会连带截断语料
依赖的 302+验证器落盘复用（only-if-cached redirect 4 腿 P→F）。教训：
**DC-3「单实现」指缓存语义层（HttpCache/storable_mode），通道落盘策略
（loader 形态门 vs 语料兼容面）可合法分歧**，分歧处必须注记归因。

## 七、核实方法

- `make testharness-net-api FILTER=fetch/http-cache`（两跑，GITHUB_TOKEN
  认证拉取面 no-op 复核）；`FILTER=request-cache`；`FILTER=fetch/api/headers`。
- `cargo test -p zero-net --lib`（530P）；`cargo test -p zero-wpt-runner
  --bin zero-wpt-runner http_cache`（3P）。
- clippy：zero-net `-D warnings` 零输出；runner 现存 90 needless_borrow
  全在 46 号切片迁移文件（在飞声明，master 头即红，非本批引入——本批
  新文件零输出）。
- 无人值守全程 systemd-run MemoryMax 双限 + test-guard（900s 墙钟）。
