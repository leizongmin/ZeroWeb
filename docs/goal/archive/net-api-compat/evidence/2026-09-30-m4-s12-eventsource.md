# M4-S12 — EventSource spec 化重做 + P7 fixture 续件（message.py 族 + 重连语义）

**日期**: 2026-09-30
**切片**: M4-S12（eventsource 簇——上一轮 CONTINUE 指向的 P7 续件评估 + P3 EventSource 语义）
**原始数据**: [eventsource](2026-09-30-m4-s12-eventsource.json)

## 结果（六 corpus，对照 M4-S11）

| corpus | M4-S11 | M4-S12 | Δ pass |
|---|---|---|---|
| fetch | 1229/1766（69.6%） | 1229/1766（69.6%） | 0 |
| xhr | 240/451（53.2%） | 240/451（53.2%） | 0 |
| url | 4431/6065（73.1%） | 4431/6065（73.1%） | 0 |
| mimesniff | 1898/1898（100%） | 1898/1898（100%） | 0 |
| streams | 903/1019（88.6%） | 903/1019（88.6%） | 0 |
| eventsource | 2/32（6.2%） | **33/34（97.1%）** | **+31** |
| **合计** | **8703/11231（77.5%）** | **8734/11233（77.8%）** | **+31** |

eventsource 页级：32 页中 **31 页全绿**（此前 23 页 Timeout + 7 页 Fail → 仅余
`request-cache-control` 的跨域半——见残余记账）。分母 32→34：cache-control 页从
页级 Timeout 单案变为 4 子案计数（同源 2 案 Pass + 跨域 2 案随页 Timeout）。

## 修齐内容

### runner 内置 fixture（testharness.rs，上游逐字等价、零 .py 落盘——inspect-headers.py 先例）

- `eventsource/resources/message.py`（31 页引用）：`?mime=`（默认 text/event-stream）
  `&message=`（默认 `data: data`）`&newline=none`（尾 `\n\n` 省略）`&sleep=`（ms，capped
  10s）——https://github.com/web-platform-tests/wpt/blob/3159769/eventsource/resources/message.py
- `eventsource/resources/message2.py`：上游无限循环流式写——headless 有限流模型取一轮
  循环体（覆盖页面断言的前三个事件）
- `eventsource/resources/last-event-id.py`：带 `Last-Event-ID` 请求头回 `data: <id>`，
  否则回 `id: <idvalue 默认 …>\nretry: 200\ndata: hello`（重连面）
- `accept.event_stream` / `cache-control.event_stream`：上游 wptserve 模板文件
  （`data: {{headers[<name>]}}`）→ 按文件名回显对应请求头

### shim EventSource 重做（part05 构造器 + part06 原型，spec §9.2）

- **构造（§9.2.2）**：`new URL(url, base)` 解析 + 失败抛 `SyntaxError` DOMException
  （`globalThis.DOMException` 优先——词法闭包构造器致 "wrong global"，part02 先例）；
  `url` 属性 = 序列化（constructor-empty-url / url-bogus 两腿）。base 取 location.href
  （getter 抛错/缺宿主环境兜底 about:blank）。
- **连接处理（§9.2.3）**：status ≠ 200 或 Content-Type essence 非 text/event-stream →
  fail the connection（CLOSED + error，不再重连——mime-bogus/valid-bogus/
  trailing-semicolon 三页）；网络错误 / body 结束 → reestablish（CONNECTING + error +
  等重连时间重连）。
- **重连（§9.2.4）**：`Last-Event-ID` 经新增 `init.__zwInternalHeadersWire` wire 旁路
  内部直设——**根因**：id 值 `…`（U+2026）过公共 init.headers 的 ByteString 校验
  （>U+00FF → TypeError "Invalid header value"）→ fetch 拒绝 → 重连死循环 → id 双页
  Timeout。spec：SSE 于 request header list 内部直设，不经 JS 可见校验；公共路径校验
  维持（fetch header-values TypeError 面零回退）。
- **解析（§9.2.5/§9.2.6）**：派发条件改 spec（仅 data 缓冲非空；缓冲空仍清 event type
  缓冲）；`id` 缓冲含 NUL 忽略；`retry` 仅全 ASCII 数字接受（`03000`→3000、`1000x`/
  空 忽略）；**last event ID 缓冲以源上 string 播种**（重连延续——format-field-id
  要求跨连接 `lastEventId "…"`，data-before-final-empty-line 要求未派发的 `id:test`
  随流丢弃 `lastEventId ""`——两页共同钉死「缓冲逐流重置 + 以 string 播种」模型）；
  **末尾行终止符弹栈**（`data:x\n` 的 split 尾 `''` 是终结符产物非空行——
  data-before-final-empty-line 的 EOF 误派发根因）。
- **WebIDL 面**：常量挂 interface prototype object（`source.OPEN`/`this.CLOSED` 断言
  ——onopen / mime 三页；须挂于 prototype 整体重赋值之后）；派发事件 `isTrusted=true`
  （`__zwTrusted` 内部口——onmessage-trusted 页）。

## 门禁

- `make test` 全绿（19506 passed / 0 failed——EventSource 桥测随 spec 翻新：handler
  收满即 close 终止重连风暴【test sandbox setTimeout 零延迟泵送】+ 新增重连
  Last-Event-ID wire 断言 + nf sandbox 补注册 DOM 回调【裸 sandbox 无 __zw_parse_url】）；
  `cargo clippy --workspace --all-targets -- -D warnings` 干净；`cargo fmt --all` 干净。
- 五 corpus（fetch/xhr/url/mimesniff/streams）全量复核与 M4-S11 逐字节同值（零回归）。

## 残余记账

- **request-cache-control 跨域半**（2 案）：`www2.wpt.test` 第二源 + `cors.py?run=
  cache-control` fixture + EventSource 跨域 CORS 检查——跨源基础设施切片，与
  fetch/api/cors 域并轨评估（同一次 runner 多源能力落地解锁）。
- transfer/detach 族结构性挂账不变（streams——需宿主 V8 detach，14 腿）。
