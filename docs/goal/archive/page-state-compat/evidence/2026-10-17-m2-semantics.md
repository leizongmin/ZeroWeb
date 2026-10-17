# page-state-compat M2 语义实现（2026-10-17）

M1 基线 27 子测试 P14/F3/T10 = 51.9% → M2 后 **41 子测试 41P = 100%**
（Timeout 案真实完成后的子测试总数从 27 增至 41；全量 JSON 见本目录
`2026-10-17-m2-semantics.json`）。

## 实现面

**sendBeacon spec 化**（`crates/engine/src/js_dom_shim/part02.js`，替换 R2931
accept-and-return-true stub）——Beacon processing model
（https://www.w3.org/TR/beacon/#sec-processing-model）：

- WebIDL 必选参缺省/undefined → TypeError；URL 解析失败 / scheme 非 http/https →
  TypeError（spec step 3）
- 数据类型面：string → text/plain;charset=UTF-8；FormData → multipart；
  URLSearchParams → urlencoded；Blob → type；ArrayBuffer/View → 无 CT——体编码复用
  `_zwFetchMain` 既有分发（R3014/R3015/R3020），零重复实现
- 64KB keepalive 配额超 → 返 false（in-flight 共享配额池未实现——
  FIXME(keepalive-quota-inflight) 记账）
- corsMode：无体或 CT safelisted → no-cors，否则 cors（spec step 5/6）
- **先 return true 再并行 POST**（fire-and-forget；credentials include；
  initiator type "beacon" 经 `_zwRtPendingInitiator` → 资源计时条目）；拒绝静默吞

**beacon 通道 Referer/Origin 注入**（`part01.js` `_zwFetchMain`，内部旗标
`__zwBeaconReferrerPolicy`，普通 fetch 通道零回归）：

- no-cors POST 同样注入 Referer（fetch spec determine request's referrer 不分模式）
- origin-only Referer 用真实 scheme origin（lenient http 宽化仅保留在 net-api 通道）
- 降级收紧（https→http 不外发：n-r-w-d / strict-origin / s-o-w-c-o 降级条款）与
  same-origin 策略跨源不外发——仅旗标生效
- **Origin header 非 cors 非 GET/HEAD 分支落位**（原 FIXME）：按 request referrer
  policy 序列化 origin-or-null——same-origin 跨源 → `Origin: null`
  （fetch spec §append-a-request-origin-header；header-origin-same-origin.html 的
  meta `content='same-origin'` + 跨源腿 `"null"` 期望，2024-02 上游 #44731 引入）

**document 级 referrer policy**：sendBeacon 读 `<meta name=referrer content=...>`
（缺省 strict-origin-when-cross-origin）——sendBeacon 无 init 入口，policy 唯一来源。

**runner fixture 三端点**（`tests/wpt-runner/src/testharness.rs`
`wpt_data_fetch_handler`，take 语义共享 stash，per-case 隔离）：
`beacon/resources/inspect-header.py`（put 存头值/get 回 x-request-<header>）、
`content-type-and-body.py`（put 存 `CT: body`/get take 或 400）、`url-echo.py`
（store 存原始 query 串/stat take 或空体）——上游 pin 版 .py 源行为等价。

**windowless 文档可见性面**（`part03.js` `_makeDetachedDocument`）：
`hidden=true` / `visibilityState='hidden'`（Page Visibility：无 browsing context
恒 hidden）——page-visibility `test_default_view.html` 2F 收口。

**页锚 scheme 双态**（`testharness_page_state.rs`）：`beacon/headers/` 无 `.https.`
中缀案走 http 页——上游 referrer/origin 案期望按 http 页语义写死（HTTP_ORIGIN 同源腿
期望全 URL Referer；https 页锚下该腿被误判跨源实证），net-api M5-S5 先例。

**R2931 断言更新**（apps/renderer `js_worker.rs` + apps/browser `tab_js_worker.rs`
镜像）：缺参 `false` → TypeError（`e instanceof TypeError` 断言）；URL 校验面用绝对
https 地址（snapshot 页锚 about:blank 下相对地址解析失败为 spec 行为）。

## 验证

- `make testharness-page-state`（test-guard 4GB/8GB/1800s）全量 41/41 Pass
  （文本日志口径；JSON 落本目录）
- renderer/browser R2931 定向跑 2/2 Pass
- 全量回归门禁（M3 收口轮，2026-10-18）：
  - `make test` 全绿（exit 0；首跑暴露两处红——本轮 csv notes 字段半角逗号违反
    15 列契约（已修），以及**预存** `extract_css_urls_unterminated_at_style_close`
    失败：923f91cda 把 `<style>` 块间分隔符括进注释后 tokenizer 侧拼接扫描把块尾
    未终止 url() 判 bad-url 丢弃——与本 goal 无关的生产图片预取回归，以独立
    修复提交落位（extract_html_css_image_urls 逐块 EOF 边界），
    engine/wpt-runner/webview 定向测试 + 新增 2 个 engine 单测钉住）
  - `make reftest` 712/712 = 100% 零回归
  - product-smoke exit 0 全 struct-check PASS；welcome-800 oracle diff 17.07% ≤ 20%
    门，双跑 PNG sha256 逐字节恒值（`c483bd9c95f82793`）；与 2026-10-14 前轮记录
    15.35% 的基线漂移非本轮面——`ZW_FONT_USE_GATE=0` 复跑恒 17.07% 排除字体门禁，
    本轮改动面（shim beacon/可见性 + 资源提取逐块扫描）经 reftest 712/712 零回归
    佐证无渲染交集

## 已知偏差（记账）

- 混合内容阻断未实现——https 页锚 beacon → http 目标照常投递（corpus 对阻断腿以
  mayBeBlockedAsMixedContent 容忍 false；beacon/headers 页锚已翻 http 后降级腿消解，
  残余面仅 .https. 案的 http 腿，期望值仍成立）——FIXME(beacon-mixed-content)
- keepalive in-flight 配额池未实现（静态 64KB 体大小判定）——
  FIXME(keepalive-quota-inflight)

## 归档后复核（2026-10-18）

复核轮在 main 实跑发现：T58 fmt 门禁（`cargo fmt --all -- --check` 前置快速失败）
在本提交（fd0a5c415）引入的 beacon fixture 三处方法链上失败——上文「make test 全绿」
结论在 fmt 门禁该点未复现；以纯格式化 style 提交修复（c745b81f4），零语义变化。

修复后全门禁重跑均绿：`make test` 21814P/0F、`make reftest` 712/712、
product-smoke welcome 17.07%（81944/480000 px）双跑逐字节恒值；welcome-800 单独
渲染 PNG sha256 `c483bd9c95f82793` 与本 evidence 收口记录恒值；
`make testharness-page-state` 41/41 Pass 复跑成立。csv 增补 2026-10-18
status=completed 收口行。
