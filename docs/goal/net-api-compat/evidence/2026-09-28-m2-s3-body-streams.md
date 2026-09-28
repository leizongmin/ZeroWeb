# M2-S3 — consume body 统一层 + 流 disturbed/locked 语义 + tee clone

**日期**: 2026-09-28
**切片**: M2-S3（response-stream-disturbed 簇 + request-consume Body 消费面 + response-clone 流分叉）
**原始数据**: [fetch](2026-09-28-m2-s3-body-streams-fetch.json) ·
[xhr](2026-09-28-m2-s3-body-streams-xhr.json) · [url](2026-09-28-m2-s3-body-streams-url.json) ·
[mimesniff](2026-09-28-m2-s3-body-streams-mimesniff.json) ·
[streams](2026-09-28-m2-s3-body-streams-streams.json) ·
[eventsource](2026-09-28-m2-s3-body-streams-eventsource.json)

## 结果（六 corpus 分类通过率，对照 M2-S2）

| corpus | M2-S2 | M2-S3 | Δ pass |
|---|---|---|---|
| fetch | 926/1725（53.7%） | **1081/1725（62.7%）** | +155 |
| xhr | 104/437（23.8%） | 104/437（23.8%） | 0 |
| url | 344/511（67.3%） | 344/511（67.3%） | 0 |
| mimesniff | 1898/1898（100%） | 1898/1898（100%） | 0 |
| streams | 253/590（42.9%） | 253/595（42.5%*） | 0 |
| eventsource | 2/32（6.2%） | 2/32（6.2%） | 0 |
| **合计** | **3527/5193（67.9%）** | **3682/5198（70.8%）** | **+155** |

\* streams 分母 +5（case 注册面次生），pass 零回归。全 Δ ≥ 0。

## 修齐内容（part01.js / part02.js）

1. **`_zwConsumeBodyBytes` 统一消费路径**（Body mixin text/json/blob/arrayBuffer/bytes/
   formData 共用）：① unusable 判定（已消费 / stream disturbed|locked → reject
   TypeError）；② 用户 ReadableStream 源读全量（error 传播 + 非 Uint8Array chunk →
   TypeError——error-from-stream **14/14**、bad-chunk **6/6**）；③ 字节/文本路径。
   消费后 stream 补建并锁定+扰动（disturbed-5 消费后 getReader throw）。
2. **Response(ReadableStream) 源**：构造保留流对象，body getter 返回原流；read/cancel
   反向标记 bodyUsed（disturbed-6 **5/5**）。Request BufferSource body → `_bodyBytes`
   （request-consume 14→**43/45**）。
3. **disturbed/locked 全谱**：reader.read/cancel、stream.cancel 直调、消费后锁定——
   disturbed-2 **12/12**、disturbed-3 **12/12**、disturbed-4 4→12/12、disturbed-1 修复
   （getReader+releaseLock 不消费——bodyUsed = disturbed 语义，非锁定语义）。
4. **clone()**：unusable（disturbed/locked/已消费）→ TypeError；流源 clone → 最小 tee
   （pull-ahead 全缓冲双分支——原分支保原 chunk 引用、克隆分支结构化克隆拷贝
   （typed array slice / DataView 重建）；分支 cancel 隔离）——response-clone 5→**20/21**。
5. **formData 收敛**：essence 白名单（multipart/urlencoded 之外 → TypeError——含缺
   Content-Type）；multipart 解析 RFC 2046 分隔符行规则（分隔符须行首/前置 CRLF 或
   closing `--`、closing 后仅容 CRLF、缺 closing → TypeError）——response-form-data
   7→**13/14**、consume-empty 10→**13/14**。
6. **body 派生默认 Content-Type**（initialize）：URLSearchParams → urlencoded、字符串 →
   text/plain;charset=UTF-8（response-consume「URLSearchParams to formData」解锁）。
7. **防投毒**：read-result 对象 null 原型 + 字节结果自有 `then: undefined`（
   response-stream-with-broken-then——Object.prototype.then 注入经 thenable adoption
   劫持 resolution）。
8. **bytes()**（Body mixin）补全（Request/Response）。

## 存量引擎测试期望翻新（5 处，随 consume-once 语义收敛）

`test_response_request_constructors_r2968` / `test_fetch_response_final_url_and_blob_mime_type`
/ `test_cache_api_page_shim_host_roundtrip`（clone 先于消费）、
`test_blob_stream_response_body_readers_r2978` / `test_request_body_readers_r2982`
（formData 需 urlencoded essence，测试补 Content-Type）——旧宽松消费行为锚，非回归。

## 门禁注记

`make test` 连续两轮各触一个**已知争用窗 flaky 族**（`stale_etag_revalidation` /
`navigator_skip_waiting`——rally 巡检账本在册争用窗形态，与 shim 无涉：本轮零 webview/
net .rs 改动）；隔离复跑双绿（R4867 判例同型）；机器空闲后全量 `make test` 68 套全绿
收口。

## 残余簇（记账）

- fetch 62.7% 余量：response-error 族（`Response.error()` body 语义 0/10）、
  request-upload/echo（.py fixture——P7）、错误面甄别余量。
- streams 分母 +5 案与 eventsource 23 Timeout：P7 fixture 通道。
- XHR 腿（header-values 61）：M3 setRequestHeader 校验。
