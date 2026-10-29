# M2-S10 — request-headers 方法面收口（Origin 第三分支 + string 体默认 CT + 头名大小写保真）（2026-10-29）

**切片**: goal v2.12 小片清单——「request-headers 方法面余 22F（runner 域）」。
**三文件 25F 全收口**（basic/request-headers 22F、request-headers-case 2F、
request/request-headers 1F）。

## 25F 归因分解（终态）

| 组 | 腿数 | 根因 |
|---|---|---|
| basic/request-headers Origin 缺失（POST/PUT/Chicken × 各 body 形态 + requestOriginHeader 五腿） | 17 | Origin 仅 cors 跨源与 beacon 两分支——同源非 GET/HEAD 普通请求无 Origin（FIXME(#append-a-request-origin-header) 挂账） |
| basic/request-headers string 体默认 CT 缺失（PUT with body / POST with text / Chicken with body） | 3 | 体分发 else 分支只落 `String(rawBody)` 不派生 Content-Type（Request ctor 已同型派生，fetch 主路径缺） |
| basic/request-headers Chicken bodyless CL:0 | 1 | 过程轮 CL:0 规则按「非 GET/HEAD」写——自定义方法 bodyless 不发 CL（仅 POST） |
| basic/request-headers-case 2 腿 | 2 | 头名大小写线上丢失：Headers 只存小写键 + wire 同名值不合并 + runner echo-headers title-case 回显 |
| request/request-headers Set-Cookie 禁止头 | 1 | forbidden request-header 名单缺 `set-cookie`（spec 2023 复列；Set-Cookie2 保持允许） |

## 实现面

### 1. Origin 第三分支（`part01.js`）

`_zwFetchMainImpl` Origin 注入：cors 跨源、beacon 策略两分支后补第三分支——
`method !== 'GET' && method !== 'HEAD'` 恒带文档 Origin（模式无关）。
https://fetch.spec.whatwg.org/#http-network-or-cache-fetch

### 2. string 体默认 CT + CL:0 收窄（`part01.js` 体分发）

- else 分支（string 体）补默认 `Content-Type: text/plain;charset=UTF-8`
（作者 CT 已设不覆盖；`_zwHasHeader` 门控——与 Request ctor string 体同型）。
- CL:0 规则从「非 GET/HEAD」收窄为 `method === 'POST'`（「Fetch with Chicken」
bodyless CL null 断言面；「PUT without body」上游语料实发 POST 不受影响）。

### 3. 头名大小写线上保真（`part02.js` Headers + `part01.js` `_headersToWire` + `part03.js` XHR）

- Headers 增 `_hCase`（lowername → 首见原始 name，append/set 新名记录、已有名
set 保首见 case、delete 清除）；**仅请求 wire 出口消费**——迭代/查找面恒小写
（spec HeaderList 归一语义零扰动，headers-basic/record/setcookie 24+13+23 全绿）。
- `_headersToWire`：头名发 `_hCase[k] || k`；同名值合并单条目 `a, b`（set-cookie
保持各值独立条目）；新增可选 `caseMap` 参数（XHR setRequestHeader 自持头原始
大小写回注——dict 源经 fill 无 `_hCase` 可依）。
- XHR 增 `_zwXhrHeaderCase`（setRequestHeader 首见原始名），同步路径两处 wire
转换传入。异步路径（fetchFn dict init）case 不保（语料无钉，记账 2）。
- 浏览器同型：API 面归一小写、网络面保用户 case（Chromium/Firefox 同行为）。

### 4. 内部直设头 Title-Case 化（`part01.js` 23 处 + `part03.js` 8 处）

`_zwAddHeader` 全部调用点头名按 HTTP 惯例 Title-Case（Content-Type/
Content-Length/Origin/Referer/User-Agent/Accept/Access-Control-Request-*
）。安全性核查闭环：shim 内 12 处 wire 解析点全部 `toLowerCase()` 比较、
Rust 侧（webview/fetch_bridge/net client）全部 `eq_ignore_ascii_case`/
`to_ascii_lowercase`、runner 全部 `req.headers` 消费点 `eq_ignore_ascii_case`
——唯一大小写敏感消费是 runner `echo-headers.py` 回显体本身。

### 5. runner echo-headers.py 逐字回显（`testharness.rs`）

去 title-case 循环改逐字回显（原始大小写断言面前提）；`Content-Length: N`
大小写敏感 includes 面由引擎侧 Title-Case 保住（xhr/request-content-length
2/2 保持）。

### 6. set-cookie 入 forbidden request-header 名单（`part02.js`）

`_ZW_FORBIDDEN_REQ_HEADERS` 补 `'set-cookie': 1`（写侧 guard 阻断 + wire 出口
过滤同源单一事实源）；Set-Cookie2 保持允许（语料 valid 名单钉）。
https://fetch.spec.whatwg.org/#forbidden-header-name

## 验证

- 构建 0 warning（`/tmp/m2s10-build-r2.log`）；probe r2 三文件全绿。
- 全通道 A/B（`netapi-m2s10-r2.json` vs **直接前轮** `netapi-m2s9-r1.json`，
  逐 subtest）：**+25 增益 / 0 回归 / MISSING 0**（10924 → 10949 pass）；
  per-file Δ 仅三目标文件（+22/+2/+1），其余全通道逐文件零变化；
  stderr **0 panic**。
- 过程轮 r1（Origin 第三分支 + 未收窄 CL:0）22F→6F 定界——CL 过宽与
  case 面分离后 r2 全收口。

## 记账（未清偿）

1. Headers 实例作 fetch init headers 源时原始大小写不保（fill 走 entries()
   sort-and-combine——headers-record Proxy trap 计数面不允许旁路直读；语料
   无钉，数组/record/setRequestHeader 三形态已保）。
2. XHR 异步路径（fetchFn dict init）setRequestHeader 原始大小写不保（同步
   路径已保；setrequestheader-case-insensitive.htm 同步形态可绿，不在
   runner 通道分母）。
