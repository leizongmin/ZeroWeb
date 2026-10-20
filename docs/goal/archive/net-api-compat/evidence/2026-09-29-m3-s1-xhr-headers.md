# M3-S1 — XHR setRequestHeader 校验 + SAB send 守卫 + inspect-headers.py fixture

**日期**: 2026-09-29
**切片**: M3-S1（XHR 状态机首片：header 校验面 + send body 类型校验 + runner fixture
通道首件）
**原始数据**: [fetch](2026-09-29-m3-s1-xhr-headers-fetch.json) ·
[xhr](2026-09-29-m3-s1-xhr-headers-xhr.json) · [url](2026-09-29-m3-s1-xhr-headers-url.json) ·
[mimesniff](2026-09-29-m3-s1-xhr-headers-mimesniff.json) ·
[streams](2026-09-29-m3-s1-xhr-headers-streams.json) ·
[eventsource](2026-09-29-m3-s1-xhr-headers-eventsource.json)

## 结果（六 corpus 分类通过率，对照 M2-S3）

| corpus | M2-S3 | M3-S1 | Δ pass |
|---|---|---|---|
| fetch | 1081/1725（62.7%） | **1209/1725（70.1%）** | +128 |
| xhr | 104/437（23.8%） | 118/437（27.0%） | +14 |
| url | 344/511（67.3%） | 344/511（67.3%） | 0 |
| mimesniff | 1898/1898（100%） | 1898/1898（100%） | 0 |
| streams | 253/595（42.5%） | 253/595（42.5%） | 0 |
| eventsource | 2/32（6.2%） | 2/32（6.2%） | 0 |
| **合计** | **3682/5198（70.8%）** | **3824/5198（73.6%）** | **+142** |

## 修齐内容（part03.js + testharness.rs）

1. **setRequestHeader 校验**（xhr.spec §4.6.3）：须 OPENED 且未发送（`_zwXhrSent`
   状态位——send 后调用 → InvalidStateError）；name = header name token、value =
   header value（ByteString 无 NUL/LF/CR、无首尾 HTTP tab/space，先 Normalize 剥首尾）
   → **SyntaxError** DOMException；同名多值 combine `', '`。复用 M2-S2 Headers 校验
   helper（同 IIFE 作用域）。header-values / header-values-normalize 的 throw 腿
   （i=0 fail 腿 ×3 断言）全绿。
2. **send(body) 类型守卫**（xhr.spec §4.7.2）：SharedArrayBuffer / 其 typed view →
   TypeError——send-data-sharedarraybuffer 0/14 → **14/14**。
3. **runner fixture 通道首件（P7）**：`inspect-headers.py`（上游逐字等价——
   `?headers=a|b|c` → 每个存在的请求头回 `x-request-<name>: <value>`（多值 ', '
   合并）；`?cors` → AC 族 + expose；content-type: text/plain）。fetch headers 面
   （header-values(-normalize) 的合法值回读腿 ×~120 + headers-basic/normalize 回读）
   解锁：fetch 926→1209。fixtures 位于 `wpt_data_fetch_handler` 内置生成器
   （与 trickle.py/redirect.py 同层），零 .py 文件落盘。

## 残余簇（下一步）

- xhr 27%：responsetype 状态机（24）、overridemimetype（74）、blob-range（27）、
  progress 事件序——M3-S2。
- url 67.3%：percent-encoding 39 / default port ~40 / URL.parse 8——M3-S3。
- eventsource 23 Timeout / fetch status-0 簇 / request-upload echo：P7 fixture 后续
  件（event-stream / echo-content / delay 族）。
