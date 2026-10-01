# M4-S22 — XHR preflight cache + 跨源重定向语义

**日期**: 2026-10-01
**基线**: M4-S21（2026-10-01，9047/11337 = 79.8%，xhr 545/583 = 93.5%）
**结果**: **9059/11338 = 79.9%**，xhr **557/583 ≈ 95.5%**（Δ+12 全 corpus 零回归；
fetch 71.5% / url 73.0% / mimesniff 100% / streams 88.6% / eventsource 35/35 = 100%
逐 corpus 同值）。make test 19534 passed / 0 failed（68 套全绿）+ clippy
`-D warnings` 干净 + fmt 干净。

## 变更清单

### shim — CORS-preflight cache（fetch spec §cors-preflight-cache）

- 条目 `{key=(目标 origin|credentials mode), methods, headers, star, expires}`；
  命中（未过期 + method ∈ ACAM + 自定义头 ⊆ ACAH，`*` 通配）→ 跳过 OPTIONS
  （preflight-cache「second request without preflight」面）。
- store：preflight 成功后按 ACAM/ACAH/Max-Age 落缓存（缺省 5s——spec default；
  cache.py 10s / timeout.py 1s / invalidation.py 10s 三 fixture 形态全覆盖）；
  过期条目惰性清除。
- **ACAM 覆盖判定 trim 修复**：`"PUT, XMETHOD"` 分割后带前导空格，原实现
  indexOf 失配 → preflight 误判 NetworkError（invalidation-by-method /
  cors-upload withCredentials 双腿根因）。

### shim — 跨源重定向语义（fetch spec HTTP-redirect fetch）

- **credentials 升级**：Location URL 带 userinfo 且 credentials mode 非 include →
  升级 include（后续 cors check 按 include 判定——ACAO `*` 失效 → network error；
  redirects-async「user info redirected to ACAO=*」面）。

### runner — .sub 脚本替换面收口

- `wpt_data_script_fetcher` 替换集从 3 项（host/domains[www1]/ports[https][0]）
  收口到 `apply_wpt_substitutions` 全集——原 `{{ports[http][0]}}` 残留 → html 页
  `HTTP_REMOTE_ORIGIN` 带字面量占位符 → 跨源链路 302 不跟随（探针二分定位：
  .any.js 内联替换正常、.html 页经 script fetcher 外联服务裸源——
  expose-headers-on-redirect 页根因）。

### runner fixture 九件 + token stash

- `WPT_TOKEN_STASH`（LazyLock<Mutex<HashMap>>——上游 wptserve request.server.stash
  等价物；**空值 = Uninitialized** 语义对齐 Python falsy——首跑 NetworkError 根因）。
- `reset-token.py`（上游逐字等价）+ `access-control-basic-preflight-cache.py` /
  `-timeout.py` / `-invalidation.py`（上游状态机逐字等价——OPTIONS 到达序断言
  preflight cache 行为）+ `access-control-preflight-request-header-returns-origin.py`
  / `-allow-headers-returns-star.py`（OPTIONS: ACAO 回显或 \* + ACAH X-Test 或 \*；
  GET: X-Test 在 → PASS）+ `echo-content-cors.py`（ACAO/ACAH/ACAM 回显 +
  X-Request-\* 探针 + 体回显——cors-upload 双腿）+ `access-control-basic-put-allow.py`
  （PUT 回显体——non-cors-safelisted method 双腿）+ `common/blank.html`（wpt-data
  未拉静态本体；redirect 链暴露头面 + ?pipe= 管道头载体）。
- 前轮探针残留页（zz-probe-pfc ×2 / zz-probe-redir / zz-probe-bom）已随轮清理。

## 解锁明细（+12 对照 S21，REGRESSED 0）

- preflight cache 族 4：access-control-basic-allow-preflight-cache{,-timeout,
  -invalidation-by-method,-invalidation-by-header}
- preflight request 面 2：access-control-preflight-request-{header-returns-origin,
  allow-headers-returns-star}
- redirect 面 2：access-control-and-redirects-async（user-info leg）+
  access-control-expose-headers-on-redirect（.sub 替换收口连带）
- cors-upload 2（页级 Timeout 一并解除）
- non-cors-safelisted method 2：access-control-basic-allow-non-cors-safelisted-
  method{,-async}

## 残余（27 Fail + 3 Timeout）

- **page-scheme 失配 2**（access-control-and-redirects-async「same-origin resource
  file」/ access-control-basic-allow-access-control-origin-header）：runner 页面
  锚为 `https://wpt.test`，WPT 宇宙 get_host_info 恒 `http://wpt.test`——同源判定
  与 Origin 回显断言跨 scheme 失配。需 runner 页面 scheme 对齐（跨切面改造——
  testharness.rs 34 处 + shim iframe 解析 1 处），单独立片评估。
- **sync XHR 结构面 4**、**bad-chunk 注入 2**、**formdata 族 5**、**template-element
  XML 文档 3**、**timeout 族 2**、**preflight-request-\* 余量/其他散腿 ~9**——
  见 master.md 下一步计划。
