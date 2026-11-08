# HTTP 缓存层架构落点说明书——六模式态接线与 runner 缓存层（Spec 摘要 + RFC + 实施交接）

**版本**: v1.0
**日期**: 2026-11-08（M0 交付）
**需求来源**: `docs/goal/http-cache.md`（Mission/DC-2/DC-3/DC-4，用户 2026-11-04 拍板立项）
**现状对账**: [M0 对账档](2026-11-08-m0-reconciliation.md)（代码现状、102F 分簇、corpus 盘点）
**流程**: lei-spec-rfc 标准模式——需求材料以控制面为权威，本文只交付设计（RFC）与实施交接

## 0. 执行摘要

**目标**: 把 request.cache 六模式态从 JS 一路接到缓存层，并让 WPT runner 通道与
浏览器通道共享同一缓存实现，达成 102F 收敛（DC-3/DC-4）。

**核心事实**: zero-net 缓存核心（freshness/条件请求/304/Vary/504）**已存在**且
浏览器通道 default 模式已在用；真正缺的是三段接线——JS→wire 传播（shim 十参
wire 无 cache 字段）、net 四态→六态（缺 Reload/ForceCache）、runner 通道缓存
包裹（当前文件服务直通，无缓存语义）。

**关键决策**（依据与取舍见 §3.1/§3.5）:
- wire 加第 11 位置参数 `cache`（空串=缺省），不做 shim 层头翻译。
- runner 缓存包裹层**复用 zero-net `HttpCache`**（DC-3「无双实现」硬约束），
  以 fixture handler 为"网络"后端，per-case 生命周期。
- cache.py 端点按 get-set-cookie.py 案级 stash 先例落 runner 内嵌 handler——
  request-cache 族 102F 收敛的本 goal 自有前置，不属 46 号 corpus 基建。

**首个落地步骤**: net 侧 CacheMode 扩六态 + 六态读写矩阵单测（M1 首批，纯
`zero-net` 域，无 wire 改动）。

**IPC 契约影响面判定: 零变化**——`__zw_fetch` wire 与 FetchRequest 是 engine
内部 JS 桥接面（同进程回调），fetch handler 到缓存层到网络栈全链路在
renderer/browser 进程内；`zero-protocol` 与 `apps/renderer` IPC 消息集不动。
详见 §5。

---

## 1. 现状分析

### 1.1 双通道现状图

```
浏览器通道（apps/browser 单进程 tab_js_worker / apps/renderer js_worker）:
  JS fetch()/XHR → shim __zw_fetch(id,method,url,headersWire,body,clientId,
                    referrer,mode,redirect,credentials)      ← 10 参，无 cache
    → FetchBridge::register → FetchRequest{url,method,headers,body,bytes,
       credentials,mode,redirect}                            ← 无 cache 字段
    → default_fetch_handler → HttpRequest
    → ResourceLoader::submit_http → ResourceRequest{cache_mode}   ← 仅 4 态
    → HttpCache.lookup（freshness/Vary/条析）
       ├ Hit     → 立即回（revalidate-only 条目走条件再验证）
       ├ Revalidate → 条件 GET → 304 合并 / 200 覆盖
       └ Miss    → 网络 → storable policy 决定 put
    模式态来源：request_cache_mode(&headers) 仅从用户 Cache-Control 头推导
    （no-cache/no-store/only-if-cached 三面碰对；reload/force-cache 无头可推）

runner 通道（zero-wpt-runner testharness）:
  JS fetch()/XHR → 同一 __zw_fetch → FetchBridge → wpt_data_fetch_handler
    → 路径归一 → py fixture 路由 / 静态文件服务                ← 无缓存语义
    request-cache 族 cache.py 缺位 → 全部 404 → 102F
```

### 1.2 既有可复用资产（实现来源闭合）

| 资产 | 位置 | 对本设计的贡献 |
|---|---|---|
| HttpCache（内存+磁盘分层、分区、负缓存） | `crates/net/src/http_cache.rs` | runner 包裹层与浏览器通道共用（§3.4） |
| freshness/年龄/TTL | `cache_policy.rs`（RFC 9111 §4.2.1/4.2.3 注释锚定） | 直接复用，不重写 |
| 条件请求头生成 + 304 合并 | `http_cache.rs`（conditional_headers_with_request / 304 元数据合并） | 直接复用 |
| Vary 匹配 + `Vary: *` 不可存 | `cache_policy.rs`（§4.1） | 直接复用 |
| CacheMode 四态 + 504 语义 | `resource_loader.rs:61-70`（OnlyIfCached miss → Err） | 扩两态，504 语义保留 |
| 请求头→模式推导 | `resource_loader.rs request_cache_mode` | 保留（用户显式头优先语义不变） |
| POST 失效（RFC 9111 §4.4） | `resource_loader.rs unsafe_invalidation_targets` | 不动 |
| 案级 stash fixture 先例 | `testharness.rs:9861 get-set-cookie.py`、`WPT_TOKEN_STASH`（take/put）、vary.py | cache.py 仿真按此落地 |
| shim 六枚举校验 + only-if-cached/same-origin TypeError | `js_dom_shim/part01.js:7040-7084` | 不动；`v.cache` 已挂 Request 对象（7145），派发处直接取 |

### 1.3 七维分析摘要

| 维度 | 状态 | 关键发现 |
|---|---|---|
| 意图 | ✅ | 102F 收敛达 DC-4 门（地址池×Chrome 100% = 102/102） |
| 涉众 | ✅ | 本 goal（engine+net+runner）；43 号 v3 终账零回归同窗约束；46 号 corpus 边界 |
| 引入 | ✅ | 新数据：wire cache 参数、CacheMode 两态、runner per-case cache 生命周期、cache.py stash 态 |
| 逆向 | ✅ | 缓存错误命中/错误 504 的回滚切点：wire 新参缺省空串=现行为，net 新态缺省 Default——分级回滚见 §6 |
| 系统集成 | ✅ | 双通道同层；SW Cache API 正交不动；导入/模块脚本 ResourceLoader 面不改行为 |
| 完整性 | ✅ | 六态×读/写/验/可观察头矩阵（§3.3）闭合全部 102 腿依赖 |
| 质量 | ✅ | 可测性：net 单测（矩阵）+ 定向 runner 实跑（102 账）+ v2/v3 全量 A/B 零回归 |

无阻塞性待澄清项——需求、验收、边界已由控制面与 WPT 语料（可执行 oracle）钉死。

---

## 2. 目标状态

```
JS fetch(input, {cache}) / Request.cache / XHR（恒缺省）
    → shim __zw_fetch(..., credentials, cache)                ← +1 参
    → FetchRequest{..., cache: Option<String>}                 ← +1 字段
    ├─ 浏览器: handler 映射 → ResourceRequest.cache_mode       ← 六态
    │    → HttpCache 六态矩阵（§3.3）
    │    → 网络层注入 spec 可观察头（§3.3 末列）
    └─ runner: handler 映射 → CaseHttpCache 六态矩阵
         → miss/再验证时调 fixture 后端（"网络"）→ 按 policy put
         → cache.py 端点仿真（案级 stash 记录网络可见请求）
```

验收锚（§3.3 矩阵每行）→ 102 族腿（M0 对账档 §二）+ net 单测。

## 3. RFC 详细设计

### 3.1 wire 协议：`__zw_fetch` 第 11 参

**决策: 位置参数追加，空串=缺省**。调用点 8 处（part01 ×5、part03 ×2、
part06 ×1）+ `fetch_bridge.rs::register` 的 `args.get(10)`。

- shim 派发处（part01.js:6502 主 fetch、6354 redirect hop、4988 CORS 预检）
  传 validated request 的 `_zwCache` 字段；XHR（part03）与 img/脚本辅助通道
  恒传 `''`（default 语义）。
- `FetchRequest` 增 `pub cache: Option<String>`（`None` = wire 未给/旧宿主）。
  结构体加字段是**源码级破坏性变更**——`FetchRequest` 以字面量构造的全部
  构造点（engine 测试、各 app handler 测试）同窗补 `cache: None`。

**否决方案（头翻译）**: 在 shim 把模式译成请求头（如 no-cache → 追加
`Cache-Control: no-cache`）。否决依据：
1. `force-cache`/`only-if-cached`/`reload` 没有可一一反推的头语义，净层
   `request_cache_mode` 无法从头还原六态。
2. 可观察头注入（reload → `Cache-Control: max-age:0`、no-cache →
   `Cache-Control: no-cache`/`Pragma: no-cache`）是 fetch spec HTTP-network
   阶段的**网络层**职责，属六态矩阵的输出列（§3.3），不应与模式传播混在
   shim——混放会让 shim 修改用户可见 headers 列表（`req.headers` 断言面），
   与"注入发生在 HTTP-network fetch"的规范时序不符。
3. 显式参数保留模式全量信息，为 net 六态与 runner 六态提供同一信号源。

### 3.2 net CacheMode 扩展与请求映射

`resource_loader.rs::CacheMode` 增两态：

```rust
pub enum CacheMode {
    Default, NoCache, NoStore, OnlyIfCached,
    Reload,      // 新：不读缓存；网络成功后按 policy 覆盖写缓存
    ForceCache,  // 新：有 stored 条目即用（含 stale，不验）；零条目走网络并入缓存
}
```

映射点（browser handler，`js_worker.rs` / `tab_js_worker.rs` 同构）：
`req.cache.as_deref()`（''→Default）→ `ResourceRequest::with_cache_mode`。
净层 `submit()`/`lookup_cache()`/`store_streamed_response()` 按矩阵处理；
`ResourceRequest::identity_key` 已含 `cache={:?}`——**不同 cache 模式的在途
事务天然不合并**（reload 与 default 同 URL 不共享网络事务，符合 spec）。

`request_cache_mode(&headers)` 保留：用户显式请求头推导优先级低于 wire 显式
模式（wire 给了模式就用 wire 的；wire 未给回落头推导——现行为零变化）。

### 3.3 六态语义矩阵（缓存层行为定义）

读=是否查缓存；写=网络响应后是否按 storable policy 入缓存；验=条件再验证；
`504`=仅 only-if-cached 未命中。

| 模式 | 读 | 写 | 验 | 504 | 网络可观察头注入 |
|---|---|---|---|---|---|
| `default` | freshness 决定（fresh 用/stale 验） | ✓ policy | stale 即验 | — | 无 |
| `no-cache` | 不用未验证条目 | ✓ policy | 有条目必验；无 validator 全量取 | — | `Cache-Control: no-cache`（请求未含时；fetch spec HTTP-network-or-cache fetch） |
| `no-store` | 不读 | ✗ | — | — | 无 |
| `reload` | ✗ | ✓ 覆盖写（含旧条目替换） | — | — | `Cache-Control: max-age=0`（Chromium 形态——WPT expected_max_age_headers 判据） |
| `force-cache` | 有条目即用（含 stale，**不验**） | ✓ 仅网络路径后 | 仅零条目走网络 | — | 无 |
| `only-if-cached` | 有条目即用（**含 stale**，不验；`Vary:*`/变体失配=未命中） | ✗ | — | 未命中 | 无；限 same-origin（shim 已校验） |

矩阵的可执行 oracle（M2 逐腿实现依据，全部在仓）:
- stale 条目即用不验：`request-cache-only-if-cached` 臂 1/2、force-cache 臂 1/2
  （`expected_validation_headers: [false]`——第二请求零网络）。
- reload 覆盖写：`request-cache-reload` `["default","reload","default"]`——
  第三请求（default）读到的是 reload 存的新体且 expiry 重算。
- `Vary: *` 强制网络：force-cache vary 臂（`[false, true]`）。
- only-if-cached 跨源重定向拒绝：`redirect: "cross-origin"` 臂——第二请求
  零网络触达且 TypeError。**依赖重定向响应从缓存重放**（302+Location 入缓存，
  redirect 链解析时逐跳查缓存、跨源跳即断——`is_cacheable_status` 已含 30x，
  重放/断链为 M2 新实现面）。
- 同源重定向缓存复用：`redirect: "same-origin"` 臂（`[false, false]`）。

条款锚定：六态语义=fetch spec §http-cache-fetch/§http-network-or-cache-fetch/
§http-redirect-fetch（cache mode 进 network 语义面）；底层新鲜度/年龄/失效
=RFC 9111 §4.2/§4.2.3/§4.4（net 已锚）。M2 实现处按此补引用注释。

### 3.3a RFC 9111 / fetch spec 语义面清单（Mission M0 交付项）

状态列：✅=net 已实现（条款注释已锚定，本 goal 只复用）；🆕=本 goal 新增面；
➖=划界外。

| 条款 / spec 节 | 语义面 | 状态 |
|---|---|---|
| RFC 9111 §3.8 | 存储键（method+target URI；fragment 剥离） | ✅ `cache_key.rs`（cache_lookup_key/strip_url_fragment） |
| RFC 9111 §4 | 构造与更新（跨进程共享➖划界外） | ✅（进程内） |
| §4.1 | Vary 匹配、`Vary: *` 永不命中 | ✅ `cache_policy.rs`（vary_field_changed） |
| §4.2/§4.2.1 | freshness lifetime（max-age>Expires>s-maxage 私有忽略；Expires−Date 基准） | ✅ `compute_ttl_secs`（R3235/R3392） |
| §4.2.2 | 启发式新鲜度（Last-Modified 10%） | 🆕 corpus heuristic 族面（M3 批次评估；request-cache 102 不依赖） |
| §4.2.3 | 年龄计算（Age/apparent_age/resident_time） | ✅ `compute_initial_age`（R3233） |
| §4.3.1 | 验证时点（stale/no-cache） | ✅ lookup→Revalidate |
| §4.3.2/§4.3.3 | If-None-Match/If-Modified-Since 发送（ETag 优先） | ✅ `conditional_headers_with_request` |
| §4.3.4 | 304 处理（头合并、元数据更新、体保留） | ✅（R3231/validators refresh） |
| §4.3.5 | 部分内容（Range/206） | ➖ corpus partial 面（net 无 range 存储；Chrome 同簇 33%） |
| §4.4 | 失效（同 URI 的非 GET 成功响应；Location 头失效目标） | ✅ `unsafe_invalidation_targets` |
| §5.2.1.1-5.2.1.4 | 请求侧 no-cache/no-store/max-age/min-fresh/max-stale | ✅ `parse_request_cache_control`（P0 面）；min-fresh/max-stale 消费🆕（cc-request 族，M3 评估） |
| §5.2.2.x | 响应侧 no-cache/no-store/public/private/must-revalidate/s-maxage | ✅ `parse_cache_control`/`storable_mode` |
| fetch spec §http-cache-fetch | only-if-cached 命中/504、same-origin 限定 | ✅ 四态 OnlyIfCached；stale 条目复用与重定向重放🆕（M2） |
| §http-network-or-cache-fetch | 六模式读/写/验选择、可观察头注入 | 🆕 六态矩阵（§3.3） |
| §http-redirect-fetch | 重定向缓存复用、only-if-cached 跨源断链 | 🆕（M2；30x 已可存 `is_cacheable_status`，重放/断链新实现） |

### 3.4 runner 通道缓存包裹（CaseHttpCache）

**决策: 复用 zero-net `HttpCache`，per-case 实例，fixture handler 为网络后端**。

```
wpt_data_fetch_handler_with_stash（每案构建时）:
  let case_cache = HttpCache::new();          // 纯内存，案生案灭
  handler 闭包内:
    match cache_mode(req) {                   // 六态映射（§3.3 同表）
      NoStore → 直通 fixture（不读不写）
      OnlyIfCached → lookup: Hit(含 stale)→服务；否则 Err(504 形态)
      _ → lookup:
        Hit(reload 外) → 直接回（FetchResponse 转换）
        Revalidate → 带条件头调 fixture → 304: 合并服务 / 200: 覆盖
        Miss/Reload/ForceCache-miss → 调 fixture → policy 允许则 put
```

- **类型适配**: fixture 产出 `zero_engine::fetch_bridge::FetchResponse`；
  `HttpCache::put_with_headers`/`lookup` 消费 `zero_net::HttpResponse`。
  两侧字段一一映射（status↔status_code、headers、body_bytes↔body、url、
  redirect_count 恒 0），薄适配 fn 两个，不引入第三类型。
- **生命周期**: handler 每案构建（既有先例同款），`HttpCache::new()` 纯内存
  无磁盘根——案间零泄漏；request-cache 族 URL 带 per-test token（uuid），
  键天然不跨案碰撞，双保险。
- **"网络"边界**: fixture handler 即 HTTP-network 阶段的替身——stash 记录的
  「服务器观察到的请求」= 真实网络触达。304/302 由 cache.py 语义按条件头
  返回（缓存层条件头生成正确性直接被 stash 断言检验）。
- **零回归风险**: 包裹层只对带 cache.py 引用的案生效？——不，按设计对全部
  runner fetch 请求生效（default 模式）。既有账面零回归约束（DC-4）要求
  A/B 对照兜底：runner 通道静态文件（无 freshness 头）TTL=None 不可存
  （`storable_mode` → None），包裹层 miss 后直通——静态文件行为不变；
  带 Expires/CC 头的既有 fixture 面（S295 image-alternate-no-store 等）
  逐案 A/B 核对。

### 3.5 runner cache.py 端点仿真

上游语义（wpt master `fetch/api/request/resources/cache.py`，67 行，已取回
存档）: token stash 状态机——每请求 append 观察头四元组（InM/IMS/Pragma/CC，
`ignore` 参数时记空对象）→ `querystate=1` 取走全量 JSON；响应头按参数回显
ETag（`"tag"` 引号包裹）/Last-Modified/Expires/Vary/CC/ACO `*`；条件命中
（InM==tag 或 IMS==date）→ 304；`redirect` 参数 → 302+Location；否则 200
text/plain + content。

落地形态（§1.2 先例）: `testharness.rs` fetch handler 加 `cache.py` 路由臂，
案级 `Arc<Mutex<HashMap<String, Vec<观察头>>>>`（handler per-case 构建自清），
JSON 响应与上游逐字段对齐（断言消费 `expected_*` 列表按序比数）。304 须带
既有 `x-zero-*` 元数据头豁免形态不适用——304 是真实响应，shim 侧
null-body-status 已处理（`part01.js _makeResponseFromWire` 304→body null）。

### 3.6 设计选项总表

| 决策点 | 选定 | 否决项与理由 |
|---|---|---|
| 模式传播载体 | wire 第 11 参 | shim 头翻译（§3.1 三条否决依据） |
| runner 缓存载体 | zero-net HttpCache 复用 | runner 自写简易缓存——违反 DC-3「无双实现」 |
| cache.py 归属 | 本 goal 案级 fixture | 等 46 号 stash 框架——102F 收敛是本 goal DC-4 主门，族 fixture 是自有前置；46 号框架化后可收编（挂账注记） |
| corpus（fetch/http-cache） | 只消费 46 号交付 | 本 goal 不代做（划界明确）；批次门按 46 号解锁实数落 |

## 4. 安全考虑

- **信任边界不变**: 缓存键含分区（顶级站点），跨分区绝不共享——既有
  HttpCache 分区机制不动；runner per-case cache 天然无跨案串味。
- **Cache-Control/响应头注入面**: 网络层注入（§3.3）只增不改用户显式头；
  shim 不再翻译头——`req.headers` 用户可见面零扰动。
- **隐私模式**: 既有 private-mode 内存缓存路径不动；OnlyIfCached/Reload/
  ForceCache 三新态在分区缓存选取逻辑（`cache_for_partition`）上自动继承。
- **磁盘层**: 新态经 `storable_mode` policy 决定存储——reload 覆盖写走与
  default 同一 policy 门，不新增磁盘写入面。
- **DoS/资源**: runner per-case cache 案生案灭，无常驻增长；浏览器侧新态
  不改变缓存容量上限（50MB 常驻/8MB 导航收缩既有预算）。

## 5. IPC 契约影响面判定

**结论: `zero-protocol` 与 `apps/renderer` IPC 消息集零变化。**

依据：全链路组件（shim 沙箱、FetchBridge、fetch handler、ResourceLoader、
HttpCache、runner fixture）都在单一进程内的线程间传递——renderer 进程的
js_worker 线程模型、browser 单进程的 tab_js_worker 同构。`__zw_fetch` wire
是 JS 沙箱与宿主回调之间的**进程内**接口（engine 内部契约，随本 goal 同步
两端：shim JS 调用侧 + fetch_bridge 解码侧），不是 IPC。
主进程/renderer 进程间无缓存状态共享（划界明确多进程缓存共享排除挂账）。

---

## 6. 实施交接（Implementation Handoff）

### 6.1 文件清单与职责映射

| 文件 | 职责 | 批次 |
|---|---|---|
| `crates/net/src/resource_loader.rs` | CacheMode 扩 Reload/ForceCache；`submit`/`lookup_cache`/`store_streamed_response` 六态矩阵；网络层可观察头注入 | M1 |
| `crates/net/src/http_cache.rs` | reload 覆盖写路径（put 覆盖旧条目含磁盘层）；30x 重定向条目重放辅助 | M1/M2 |
| `crates/net/src/cache_policy.rs` | 六态矩阵单测（读/写/验/504 全组合） | M1 |
| `crates/engine/src/fetch_bridge.rs` | wire `args.get(10)` 解码；`FetchRequest.cache` 字段 | M2 |
| `crates/engine/src/js_dom_shim/part01.js` / `part03.js` / `part06.js` | 8 个 `__zw_fetch` 调用点补 cache 参；fetch 派发/redirect hop/预检取 validated cache；XHR 恒 `''` | M2 |
| `apps/renderer/src/js_worker.rs` | `default_fetch_handler` 映射 cache→CacheMode | M2 |
| `apps/browser/src/tab_js_worker.rs` | 同上同构 | M2 |
| `tests/wpt-runner/src/testharness.rs` | cache.py 路由臂 + 案级 stash；CaseHttpCache 包裹层（FetchResponse↔HttpResponse 适配）；cache 模式映射 | M2/M3 |
| `tests/wpt-runner/src/main.rs` / Makefile | 无新 target——`testharness-net-api FILTER=` 既有面 | — |
| `docs/goal/http-cache.md` | 里程碑/DC 账面回填 | 每批 |

### 6.2 推荐修改顺序（依赖序）

1. **M1-a（net 六态核心）**: CacheMode 扩态 → 矩阵实现 → 矩阵单测。
   纯 `zero-net` 域，wire 未动即先行；`make test` 定向 `cargo test -p zero-net`
   经 test-guard（`make test` 全量跑 engine lib 腿受已知红影响时按
   [make-test-aborts 记忆]定向验证）。
2. **M1-b（runner cache.py fixture + CaseHttpCache 包裹）**: fixture 臂 →
   包裹层 → 定向 `make testharness-net-api FILTER=request-cache`：
   C 簇 48（default/default-conditional）应先翻绿（模式态缺省 default 路径
   + 缓存头语义已有）——M1 验收点（C 簇收敛 + 新增面零回归）。
3. **M2-a（wire 传播）**: FetchRequest.cache → shim 8 调用点 → 两 app
   handler 映射。同窗浏览器侧验证：`make test`（engine/webview）+ 定向。
4. **M2-b（B 簇 54 收敛）**: reload/force-cache/only-if-cached/no-store/
   no-cache 六态 runner 全绿；only-if-cached 重定向重放/跨源断链面。
5. **M3（收口）**: 102/102 复测 + v2/v3 全量 A/B 零回归 + corpus 批次对齐
   46 号排期 + 数值门回填控制面。

### 6.3 首批提交建议

- `feat(net): CacheMode 六态——reload/force-cache 语义与矩阵单测`（M1-a，
  独立可回滚：net 域自洽，缺省行为零变化）
- `fix(runner): request-cache cache.py fixture + 通道缓存包裹`（M1-b）
- `feat(engine): request.cache 六模式态 wire 传播——fetch/xhr 双通道接线`
  （M2-a/b，可按 wire/矩阵拆两枚）

### 6.4 回滚切点与风险缓解

| 批次 | 回滚切点 | 风险缓解 |
|---|---|---|
| M1-a | net 单独 revert——wire 未动，缺省=现行为 | 矩阵单测钉死每态；Default 路径 diff 应为零 |
| M1-b | 包裹层 feature 开关？——**不设**：包裹层只在 runner 通道（测试工具），产品面零影响；A/B 对照为回滚判据 | 既有账逐 subtest 对照；静态文件 TTL=None 行为不变论证（§3.4） |
| M2-a | wire 第 11 参缺省空串=现行为，revert shim 即回滚 | FetchRequest.cache=None 全旧构造点显式 None——编译期暴露漏点 |
| M2-b | 102 族收敛面；唯一产品可见行为变化=reload/force-cache/only-if-cached 模式从 no-op 变为生效（**规范对齐方向**，非回归） | 与 43 号 v3 终账 A/B 同窗核对（DC-4 零回归门） |

### 6.5 验证入口

- net 单测：`make test`（或定向 `ZERO_WEB_BENCH` 无关；测试经 test-guard）
- runner 定向：`make testharness-net-api FILTER=request-cache`
- 零回归：`make testharness-net-api`（全量 A/B 对照 v3 终账 13,256P 分母
  14,620/14,621 键）；fetch 头条 wpt.fyi universe 口径复测（≥2,266P）
- perf 门：net 缓存路径属网络关键路径——M3 收口前
  `ZERO_WEB_BENCH_CRATES=zero-net make bench-gate` 定向 + 合入前全量
  （run-rules §12；命中路径未改热循环，预期 NEW/PASS）

## 7. Spec Lint 报告

| 检查 | 结论 |
|---|---|
| 需求来源 | Pass——控制面 Mission/DC 全引用，无自增需求；cache.py fixture 归属为划界内的实现判断（§3.6 已论证非 46 号域） |
| 实现来源闭合 | Pass——§1.2 逐项列既有资产；无新增第三方依赖（纯仓内复用） |
| 接口定义 | Pass——wire 第 11 参、FetchRequest.cache、CacheMode 扩态、矩阵表为主定义区 |
| 验收可验证 | Pass——矩阵每行有 102 族 oracle 腿 + net 单测锚；数值门引 M0 对账档 §四 |
| 区分 Decision/Assumption/TBD | Pass——四项决策明示（§3.6）；TBD 仅 corpus 批次实数（46 号前置，DC-4 规则已定义落门方式）；「corpus 可执行面 ≈86」标注为预期值非承诺 |
| 默认动作显式化 | Pass——wire 缺省空串、wire 缺失时 None→现行为、XHR 恒 `''`、旧构造点 `cache: None` |
| 交叉引用 | Pass——§1.2/§3.1/§3.3/§3.6/§6 相互引用核对 |
| 信息分层/去重 | Pass——现状细节归 M0 对账档，本文不重复 102F 表格 |
| 阻塞项 | 无 |

已知假设（记录，不阻塞）: 上游 cache.py 以 2026-11-08 master 版本为准取回
（wpt 无 pin 文件机制，语料漂移由 M1-b 定向实跑显影）；wptfyi 10-29 snapshot
与本地语料分母已核对同构（102）。


