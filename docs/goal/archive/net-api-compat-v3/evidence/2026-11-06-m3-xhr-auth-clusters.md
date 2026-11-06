# M3（三）：xhr auth 五簇收敛——12F → 0F，+12P 零回归（13,211 → 13,223P）

日期：2026-11-06。前置：M3（二）fetch redirect 七簇（全量基线
**13,211P**/1,465F/16T，分母 14,692 行）。本批改
`crates/engine/src/js_dom_shim/part02.js`（auth 会话缓存 + 挑战值门 helper）与
`part03.js`（open() 非 null 覆写 / 凭据延迟注入 / sync preflight ACAH 镜像 /
401 透明重试 sync+async 四刀），零 fixture 变化。

## 一、五簇收敛（引擎改动逐簇）

1. **open() 凭据覆写矩阵 8F**（competing-names-passwords 13/13 全绿）：
   open() username/password 参按 xhr.spec §the-open()-method 步骤 13-14 改
   **非 null 才覆写** URL 对应字段（旧 `!== undefined` 判把
   `String(null)="null"` 打进 URL——`user in URL` 行 URL 凭据被 "null" 覆毁、
   `user in open()` 行密码成字面 "null"）。同字段 open() 参覆盖 URL userinfo、
   异字段独立回退（「user/pass in URL; user in open()」→ open 参 user +
   URL pass）。
   https://xhr.spec.whatwg.org/#the-open()-method
2. **401 挑战流 2F**（send-authentication-basic x-challenge DID +
   basic-setrequestheader-existing-session DID/DID-NOT 两腿）：open()/URL
   userinfo 凭据改**延迟注入**——不再首跳转 Authorization（旧实现首跳带凭据
   → fixture 挑战流无 401 可应 → x-challenge 恒 DID-NOT）。userinfo 剥离保留
   （线上恒无凭证 URL），凭据暂存请求域；401 + `WWW-Authenticate: Basic` +
   **同源** → 透明重试一次（401 不透传 JS），显式凭据入 UA 会话缓存（origin
   键，发起重试时落账）——repeat-no-args 无参第二请求复用缓存凭据。
3. **跨源挑战门 1F**（basic-cors 断失败面）：跨源 401 挑战在凭据在场（显式
   或会话缓存）→ **网络错误不透传**（async 走 error 事件 status 0、sync 抛
   NetworkError）——无同源重试路径，挑战响应体不得暴露给跨源 JS。上游 Edge
   159 basic-cors 绿，本实现随语料同绿。
4. **sync preflight ACAH enforcement 1F**（cors-setrequestheader-no-cred
   auth8）：同步预检补 **ACAH 覆盖镜像**（异步 M2-S11 统一面）——逐项大小写
   不敏感、authorization 恒须显式列出（`*` 永不覆盖）、其余名 `*` 仅无凭据
   通配。失败分型：preflight **HTTP 级失败**（非 2xx / 缺 ACAO）维持同步抛
   （basic-cors-not-enabled `assert_throws_dom("NetworkError")` 面），
   **ACAH 覆盖失败**走 error 事件路径非同步抛（语料 auth8 断 errorFired +
   status 0——上游 Edge 159 该腿 1/2 红，语料期望为标尺反超）。
   https://fetch.spec.whatwg.org/#cors-preflight-fetch
5. **同步预检名单单一事实源**（中间迭代回归修复）：ACAH enforcement 首刀后
   全量 A/B 拦截 `access-control-basic-allow-non-cors-safelisted-method`
   PUT P→F——同步预检手写恒入名单把 safelisted `Content-Type: text/plain`
   混入 ACRH/ACAH 覆盖。根因修复：同步预检名单改与异步共用
   `_zwPreNamesOf`（值条件安全名单单一事实源——CT essence 三形、accept 值
   域、range 单段等），ACRH 内容随异步同规。

作者 Authorization 显式头恒优先（header wins——setrequestheader-and-arguments
面不捕获 URL 凭据、不参与重试）；fetch() API 面零触碰（401 照旧 surfacing
——fetch authentication 7P 维持）。

## 二、浏览器基准（wpt.fyi）

Edge 159.0.4378.0 master（run bccd09efe0）auth 面 9 文件 **8 绿**
（competing 13/13、basic/existing-session/repeat-no-args/setrequestheader
族/cors-basic/basic-cors(-not-enabled) 全绿），唯
cors-setrequestheader-no-cred 1/2（auth8 腿红——本仓按语料 error 事件语义
反超）。方法：`/api/runs?paths=` 取 run → `/api/search?run_ids=&q=` 取逐文件
legacy_status（per-test artifact zip 命名不含该路径，summary_v2 分片不含
xhr auth 面）。

## 三、定向与全量 A/B

- 定向：`FILTER=xhr/send-authentication` **23/23 全绿**（五簇 12F 清零，
  既有 11 行维持）；回归面 `FILTER=access-control-basic-allow-non-cors-
  safelisted-method` sync/async 双腿绿；xhr 面 465P/147F/4T（失败全在账
  ——setrequestheader-content-type 31F 等既有簇，非本批面）。
- 全量：同语料同树基线 **13,211P**/1,465F/16T（stash 本批后重跑，P 数与
  M3（二）基线逐位一致）↔ cur **13,223P**/1,453F/16T（+12P）：分母
  14,692 行不变，**changed 12 全 F→P、零 P→F、零 T 变化**——即五簇 12 行，
  逐行可溯源。

## 四、挂账

M3 余项不变：`__custom` 排序簇 1F（getAllResponseHeaders 浏览器头序）+
重定向终跳 sync cors-filter 同规 + xhr 在账失败簇（formdata/submitter 4F +
overrideMimeType Shift-JIS 接线 1F + cookies.http.html 1T + redirect-keepalive
1T；permissions-policy 2F 策略面排除）→ M4 coverage 复测收口。
