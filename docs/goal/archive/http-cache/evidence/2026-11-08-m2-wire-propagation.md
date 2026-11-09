# M2——wire 传播 + B 簇收敛：request-cache 族 102/102，DC-2/DC-3 落定

**日期**: 2026-11-08（M2 完成；wire 传播 @ c6dcbce0a，trickle 回归修复 @ 本档同提交）
**前置**: M1-b 证据档（C 簇 48 绿，余 30F 全为 wire 传播面）。

> **勘误（2026-11-09）**：本档原记 commit `f2f9ee1cb`，11-08 晚 `pull --rebase`
> 改写后实为 `c6dcbce0a`——改动统计 14 文件 +395/-76 与下述交付内容逐字
> 吻合；本档及 M3/控制面所引 hash 已就地替换。

## 一、交付内容

### wire 传播（commit c6dcbce0a，14 文件 +395/-76）

1. **net**：`HttpRequest.cache_mode` 字段 + `CacheMode::from_request_cache`
   五模式映射（`default`/缺省 → None 走既有语义）；显式 `cache_mode` 优先于
   请求头 CC 启发式（fetch spec §4.6——模式态仅来自 `init.cache`，请求头
   不参与模式推导）；`Vary: *` 带验证器存为 `RevalidateOnly`（RFC 9111 §4.1
   只禁复用不禁存储），`only-if-cached`/`force-cache` stale 免验臂跳过
   `Vary: *` 条目。
2. **engine**：`fetch_bridge::FetchRequest.cache` 字段——**wire 第 12 参**
   （index 11；M0 说明书原记「第 11 参」实落为 12，index 10 已被 iframe
   通道 client-id 载荷占用——R115/S7 调用点在先）。旧宿主 10 参调用
   `get(11)=None` 向后兼容。shim part01.js 八调用点传播（主 fetch /
   重定向链逐跳 / 预检 OPTIONS 均携带）。
3. **双 app handler**（browser `tab_js_worker` + renderer `js_worker`）
   映射同构；webdriver/headless session 字段补齐。
4. **runner**：`wpt_cache_py_wrapped_handler` 六模式态全矩阵分派（镜像
   `ResourceLoader::submit`——no-store 不读不写、reload 不读覆盖写、
   no-cache 强制条件再验证、only-if-cached/force-cache stale 免验直回放 +
   504 语义、default 既有路径零变化）。IPC 契约零变化（M0 判定维持——
   全链路进程内接口）。

### B 簇收敛账（2026-11-08 定向实跑，`make testharness-net-api FILTER=request-cache`）

```
request-cache-default              8P / 0F
request-cache-default-conditional 40P / 0F
request-cache-force-cache         16P / 0F   （M1-b 14P/2F → 16P）
request-cache-only-if-cached      14P / 0F   （4P/10F → 14P）
request-cache-reload              12P / 0F   （4P/8F → 12P）
request-cache-no_store             8P / 0F   （0P/8F → 8P）
request-cache-no-cache             4P / 0F   （2P/2F → 4P）
─────────────────────────────────────────────
族账                            102P / 0F   （DC-4 族门 102/102 达成）
（filter 同窗 eventsource/request-cache-control.any.js 4P 零回归）
```

## 二、M2 引入的 3 条 P→F 与共享路径修复

全窗口 JSON-vs-JSON A/B（对 v3 终账 `/tmp/zw-results-m4b`，13,275P/1,412F/9T
基线）暴露 M2 shim 全局错误语义转换的 3 条真回归，全在
`request-keepalive-quota.html`：

- **根因链**：M2 shim 将 handler Err（`__zw_fetch_error:` wire）按 fetch
  spec 网络错误语义转 TypeError reject（request-cache 语料 only-if-cached
  504 臂判据所需，spec 正确）。runner py_routes 非 GET 兜底门（net-api
  M4-S18）对未知端点 POST 返回 handler Err——v3 终账下走「Err → status-0
  伪 resolve」兼容路径碰对；M2 后变真 reject。keepalive-quota 以 keepalive
  POST 打 `fetch/api/resources/trickle.py`（wptserve 延迟端点，corpus 无
  .py）撞门，under/at-quota 三臂 P→F。
- **文本账解析伪影教训**：文本日志对含控制字符（\x00/\n/\r）的子测试名
  折行——首轮文本 A/B 报 17 条 P→F 中 14 条为解析伪影（url-setters/
  mimesniff/lone-cr/open-method 族）；JSON-vs-JSON 逐 subtest 对比定谳
  **真回归仅 3 条**。后续零回归门一律 JSON 对 JSON。
- **修法**（共享路径优先、对齐 v3 fixture 先例）：testharness.rs
  `wpt_trickle_py_wrapped_handler`——trickle.py 非 GET 回 200 空体（HTTP
  响应语义；测试断言只涉 promise 完成），GET 穿透既有静态 404 路径
  （response-cancel-stream 等既有消费方账面零扰动）。不动 46 号正在
  重构的 `testharness_py_routes.rs`（碰撞面避让）。FIXME(runner-net-infra)
  挂账：真实 trickle 分块延迟 + fetch keepalive 配额池（beacon 池未共享）。
  修复后定向复测 3P/3F，与 v3 终账逐 subtest 持平（超出 m4b 的 F→P 提升
  不在本 goal 范围——延迟/配额池建模属 46 号）。

## 三、全窗口零回归账（JSON-vs-JSON，DC-4 门 #3）

- 基线：v3 终账 `zw-results-m4b/testharness-net-api.json`
  13,275P/1,412F/9T（分母 14,696 键，逐字核对 v3 M4 档）。
- M2 态（含 trickle 修复前）：13,8xxP——F→P 105（request-cache 族 102 +
  mode-same-origin 2 + keepalive-quota 超额臂 1）、真 P→F 仅 3（§二已
  修复）、T 变化 0、其他状态变化 0。修复后终账见本档 §五（最终跑）。
- M2 语义面影响圈定：M2 shim reject 转换的爆炸半径经全窗口证实仅
  trickle.py POST 一条路径——其余 handler Err 消费方（XHR 通道）语义
  未动（M2 错误转换只在 fetch 闭包，XHR errTag 窄化逻辑既有）。

## 四、碰撞登记（run-rules §9/§10）

- **clippy 红灯**：`cargo clippy -p zero-wpt-runner --all-targets
  -D warnings` 29 错全部位于 `testharness_py_routes.rs`（needless_borrow /
  needless_return）——46 号流切片一/二 py 路由迁出引入（最新 push
  b01a2b098 仍活跃编辑该文件），origin/main 既有红，非本流引入。按
  不硬解原则记录待 46 号自收；本流文件（testharness.rs / net / engine /
  apps）clippy 零告警。
- wpt-runner 为双流共享面：本流只改 testharness.rs（M1-b/M2 既有领地），
  py 路由文件零触碰。

## 五、验证链与最终全窗口账

- `cargo test -p zero-net --lib`（test-guard 包裹）499P/0F/1I；
- `make testharness-net-api FILTER=request-cache` 106P/0F（族 102/102 +
  对照 4P）；
- `make testharness-net-api FILTER=request-keepalive-quota` 3P/3F（= v3
  终账持平）；
- **全窗口 JSON-vs-JSON A/B（修复后终跑，
  `/tmp/zw-results-httpcache-m2/testharness-net-api.json`，2026-11-08
  20:48）**：M2 终账 **13,868P/1,308F/9T**（键 15,185 = 基线 14,696 +
  新增 489）对 v3 终账 13,275P/1,412F/9T——
  - **真 P→F = 0（DC-4 零回归门 ✓）**；T 变化 0；移除键 0；
  - F→P 104 = request-cache 族 102（DC-4 族门收敛面）+
    `fetch/api/basic/mode-same-origin.any.js` 2（M2 same-origin 重定向
    链跨源拒绝语义的顺带显影，改进面）；
  - 新增键 489 全 Pass（websockets `?wss` 变体——并行流 ws 面，新增
    无基线不属回归口径）；新增键 Fail 0。
  - v2 终账（11,040 + S 系列）覆盖说明：v3 终账经 v3 全轮零回归复验
    （v3 M4 档），对 v3 终账零 P→F 即传递覆盖 v2 链；fetch 头条
    （wpt.fyi universe 17.07%）为 weekly cron 产出面，本窗为本地账。
- `cargo fmt --check` 干净；本流 crate clippy 零告警（§四）。
- 提交门禁 lei-pre-commit-guard PASS 后落账。
