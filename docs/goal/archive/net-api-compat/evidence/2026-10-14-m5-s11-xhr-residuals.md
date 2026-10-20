# M5-S11 — XHR 小残量四腿销账：bad-chunk XHR 面 + wire 头保真 + open() 重置（2026-10-14）

**背景**：master.md 下一步计划第 3 项 M5-S11 候选——XHR 残量三簇（response-body-errors
2 + getallresponseheaders-after-reuse 1 + event-error.sub 1）。实施后四腿全销。

## 结果（六 corpus 受控 A/B，对照 M5-S10 树）

| corpus | 基线（M5-S10 树） | M5-S11 | Δ |
|---|---|---|---|
| fetch | 1504P/583F/2T | 1504P/583F/2T | 逐字节持平（wire 头保真零回归实证） |
| xhr | 571P/22F/1T | **575P/18F/1T** | **+4P/−4F** |
| url | 4639P/1426F | 4639P/1426F | 逐字节持平 |
| mimesniff | 1033P/865F | 1033P/865F | 逐字节持平 |
| streams | 905P/108F/5T | 905P/108F/5T | 逐字节持平（5T 先在） |
| eventsource | 35P | 35P | 逐字节持平 |
| websockets | 376P | 376P | 逐字节持平 |

xhr +4 腿（Fail → Pass）：

1. `response-body-errors.any.js` :: Synchronous XMLHttpRequest should throw on bad chunk
2. `response-body-errors.any.js` :: Asynchronous XMLHttpRequest should clear response on bad chunk
3. `event-error.sub.any.js` :: error while reading body should report zeros for loaded and total
4. `getallresponseheaders-after-reuse.html` :: getAllResponseHeaders() must not return
   the previous response's headers

残余 18F 集与基线逐条相同（既有记账：formdata 5、template-element 3、
overridemimetype shift-jis 8、permissions-policy 2）；cookies 页级 Timeout 1 保留
（跨域记账）。

## 根因与修齐

### ① bad-chunk XHR 面（part03.js，M5-S9 机制扩 XHR 路径）

M5-S9 的 `x-zero-body-error-after` 标记头此前只接 fetch 流路径；XHR `_zwFillFromResponse`
一次性读 `_bodyBytes`/`_bodyText`，标记被无视 → load 照发（response-body-errors 异步腿
onload=unreached 直接 Fail）。补三处：

- **异步**：fill 中 `_bodyErrorAfter` 置位时先按 HEADERS_RECEIVED + LOADING 兑现 good
  字节（rs3 handler 断言部分体可见——`TEST_CHUNK` in responseText），随后 request error
  steps（响应全清 + rs4 + error/loadend (0,0,false)，`_zwReqGen`/aborted 守卫）——
  「clear response on bad chunk」+ event-error zeros 面。
- **同步**：sync 重定向循环终填点判标记 → send() 同步抛 `NetworkError`（DOMException
  name 对齐 assert_throws_dom）。
- **解码共享**：fill 的 BOM/override/replacement 解码链提取 `_zwXhrDecode`（fill 主路径
  与 bad-chunk 部分体一处维护，语义逐字不变）。

### ② fixture 后缀匹配（testharness.rs）

event-error.sub 用相对 URL `resources/bad-chunk-encoding.py` → `/xhr/resources/` 落点；
原分支精确匹配 `fetch/api/resources/bad-chunk-encoding.py` 不命中 → 文件缺失 404 →
load 照发（首轮「load Reached unreachable」根因）。改 `ends_with("/bad-chunk-encoding.py")`
（上游两目录各一份同型端点）。

### ③ wire 响应头保真（part01.js `_makeResponseFromWire`）

shim Response ctor 按 fetch spec initialize 对 string body 注入默认
`content-type: text/plain;charset=UTF-8`（页面构造语义正确）；wire 响应复用该 ctor
→ 上游 `.asis` 无 content-type 时可见头表凭空多出该头（getallresponseheaders 首腿
「仅 foo-test」期望面根因，非 reuse 泄漏——首腿即 Fail）。构造前记录 wire 头有无
content-type，构造后无则 `headers.delete('content-type')`（页面 Response 语义不动）。
fetch corpus 1504P 逐字节持平 = 零回归实证。

### ④ open() 响应重置（part03.js）

xhr.spec open() 置 response 为 network error——响应面（status/statusText/responseText/
response/responseURL/头表）全清。原实现 open() 不清头表 → 复用对象第二次 fill 头表
叠加（getallresponseheaders 第二腿期望面）。首腿修复（③）后此腿经实测验证同过。

## 碰头核对（run-rules §9）

共享面 last-touch = 本流自身（part01/03、testharness.rs = 93b32c587 M5-S10），无并行
编辑。域外挡门延续 M5-S10 记录（渲染流 runtime.rs/postprocess.rs fmt+clippy，未处置）。

## 门禁

- `make test` 全绿 **20664P/0F**（与 M5-S10 持平——wire 头保真不破任何存量断言）。
- `cargo clippy -p zero-wpt-runner --all-targets -- -D warnings` 干净；`cargo fmt`
  本流文件干净。
- 六 corpus 受控 A/B 同树 stash 对照双跑（上表）。

## 残余（本切片后 xhr 域）

- formdata 5F（M5-S8 记账：structural 2 / wire 1 / 坐标跨域 2）、template-element 3
  （XML DOM 深水）、overridemimetype shift-jis 8（encoding 域）、permissions-policy 2
  （跨域）、cookies 页级 Timeout 1（跨域）。
- xhr 域 goal 内可实施面就此清空——余量全部为跨域记账或 XML DOM 深水切片。
