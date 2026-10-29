# M2-S3b — 真回归 3 legs 清偿 + 页锚结构面 + HTTP-redirect fetch 语义补齐（2026-10-29）

**切片**: M2-S3 收口挂账的 3 条真回归清偿（xhr json UTF-16、xhr CORS 同源
重定向、ws constructor/016）；连带 runner 页锚结构面（net-api 页锚对齐
substituted 端口）与 shim 侧 redirect 语义两项（cors 跳 Origin 归一门、
Location credentials network error）。

## 根因与实现

### 1. xhr responseType json 走 UTF-8 decode（`part03.js`，commit cc66ad989）

json 分支复用 `_zwXhrDecode`（BOM 嗅探 final encoding）→ UTF-16 BOM 面字节
解码失败。xhr.spec §the response attribute 的 JSON response steps 基于
**UTF-8 decode**（Encoding #utf-8-decode——lossy + BOM 剥离），不走 BOM
嗅探。改 `JSON.parse(new TextDecoder().decode(bytes))`：
`json.any.js「Ensure UTF-16 results in an error」`清偿。

### 2. ws fixture `/echo-query_v13` 路由别名（`ws_fixture.rs`，commit a01fdaaf9）

`websockets/constructor/016.html` 上游经 wptserve handler 变体名
`/echo-query_v13`（行为与 `/echo-query` 等价）请求回显；runner KNOWN_PATHS
无此路由 → 连接拒绝。补路由别名 + KNOWN_PATHS 条目：016 清偿 1/1。
（charset lossy decode 假设排除——`read_corpus_to_string` 已做 meta charset
prescan，probe 先证 fixture 路由即翻绿。）

### 3. net-api 页锚 `wpt.test` → `wpt.test:8000`（`testharness.rs`）

net-api `.html` 语料页锚无端口，而 `get_host_info()` token 替换产出带
`:8000` 的 origin 字符串——同源判定按字符串严格相等时撕裂：

- `request-init-001.sub.html`「referrer /relative/ressource」：锚
  `http://wpt.test` 解析相对 referrer 得 `http://wpt.test/...`，期望
  `http://wpt.test:8000/...`（M2-S3 记账 1）。
- `xhr/access-control-and-redirects-async`「redirected to a same-origin
  resource file」：同源重定向腿被锚差判跨源，onerror 提前。
- `response-url.sub.any.js`、`referrer.any.js` origin-when-cross-origin 两腿
  同族。

修复：`page_authority` 对 net-api 语料锚到 `wpt.test:8000`（**对齐
substituted 实际端口**，非 s116 曾试后回退的「映射默认端口 80/443」——
`new URL().origin` 序列化剥默认端口会造成两形撕裂；host 段-only 比较不可
取，会破坏 `HTTP_ORIGIN_WITH_DIFFERENT_PORT`（:8081）合法跨源腿）。配套
runner 侧两处 `:8000` 成员：image fetcher prefix strip、
`wpt_fetch_response_type` same_origin 集合。SW 通道（https :9443）不受影响。

### 4. cors 跳 Origin 归一门放宽（`part01.js`，M4-S23 修正）

M4-S23 的 per-hop Origin 重写以「Location origin ≠ 文档 origin」为门。页锚
对齐后文档 origin 与重定向目标 origin 相等，「跨跳回到文档 origin」的
Location（cors-redirect「cors to same origin」族，期望 Origin `null`）不再
触发重写 → 20 legs 回归暴露。修正：cors 跳 Origin **恒归一**（opaque 置
`null`，否则文档 origin），规则对齐 fetch spec HTTP-redirect fetch——opaque
判定按键「当前跳 origin ≠ Location origin」（`_zwHopOriginOpaque` 条件未
变），「Location origin 是否等于文档 origin」不在规则内。
https://fetch.spec.whatwg.org/#http-redirect-fetch

### 5. Location credentials → network error（`part01.js`，M2-S3b 新增）

页锚对齐后 `cors-redirect-credentials`「remote to same origin with
credentials」15 legs 改判：Location userinfo 此前被静默剥离后按同源 200 走
完，期望 TypeError。按 fetch spec HTTP-redirect fetch 两条 network error
规则补齐：① mode cors 且 request's origin 与 locationURL origin 不同源；
② response tainting 已是 "cors"（跨源 cors 链任意跳，含同源收尾）。shim
新增 `_zwRespTaintingCors` 追踪（初值：mode cors 且初始 URL 跨源；跨源跳转
置位不回退），命中 → `TypeError` reject。同族复用面：M4-S22 credentials
mode 升级（ACAO `*` 失效）从 cors check 失败改走更早的 network error，观测
面一致（access-control-and-redirects-async user-info 腿保持失败语义）。
https://fetch.spec.whatwg.org/#http-redirect-fetch

## 验证

- 全通道 A/B（`netapi-m2s3b-r3.json` vs `netapi-m2s3b-r2.json`，逐 subtest）：
  **0 回归，+11**（10574 → 10585）。EXIT=0 需零回归零 MISSING——8 MISSING
  逐条核对均为页锚对齐导致的 subtest 改名（名称内嵌 origin 字符串追加
  `:8000`），且全部 Pass→Pass：

| 文件 | 改名 legs | 状态 |
|---|---:|---|
| fetch/api/cors/cors-multiple-origins.sub.any.js | 3 | Pass→Pass |
| eventsource/request-cache-control.any.js | 2 | Pass→Pass |
| websockets/Create-invalid-urls.any.js | 3 | Pass→Pass |

- 增益分解：cors-redirect +5（same origin to cors 5 codes）、referrer +2、
  request-init-001/response-url/access-control-and-redirects-async/
  access-control-basic-allow-access-control-origin-header 各 +1。
- **真回归 3 legs 全清**：xhr json UTF-16 Pass；access-control-and-redirects-
  async 7/7；ws constructor/016 1/1。
- 过程轮（页锚翻转后 shim 修正前）cors-redirect 20 回归 / 5 增益——归因
  即 4（Origin 归一门），修正后 cors-redirect 15→20/20、净 +5。
- 预存注记：通道 stderr 的 url crate panic（lib.rs:2881，每轮 2 次）与
  基线同现，非本切片引入；根因排查挂 runner 域。
- 构建无 warning（`build-m2s3b-r5.log`）。

## 记账（未清偿）

1. **cors-redirect-preflight 族**：preflight-after-redirection 期望重定向后
   重新 preflight 协商（`x-did-preflight` 计数），runner 无 per-hop
   preflight 机制——runner 基建面，非本切片。
2. **request-upload 余 4 legs + h2 流块 TypeError 3 legs**（承 M2-S3 记账 2）。
3. **abort/general 流-中止族余 4 legs**（承 M2-S3 记账 3，需引擎入口同步
   cancel）。
4. **url crate panic 根因**（runner 域，与 M2-S2「serialize_url panic 守卫」
   同族线索）。
5. **subtest 改名对账**：页锚对齐后名称内嵌 origin 的 subtest 名称变化，
   后续 A/B 若跨该切片基线需按改名映射核对（本轮 8 legs 已逐条核对
   Pass→Pass）。
