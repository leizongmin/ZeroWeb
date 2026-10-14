# WebSocket、文件资源与响应策略

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

读取 WebSocket 握手、消息时序、页面 reset 后串线、本地资源与脚本 MIME 问题。网络协议实现位于 `zero-net`，异步事件与导航代际由 engine `ws_bridge` 承担；JS 构造器与 Web API 面由 scripting 领域维护。

## Gotchas
- WebSocket::new 不建立连接，connect_with_handshake 只接受 ws/wss；普通 http/ftp/file URL 会提前返回 InvalidUrl，而不是交给握手层产生模糊连接错误。 (`crates/net/src/websocket.rs`)
- WebSocket 握手的 Origin、Cookie 和有序 protocols 由宿主提供，101 响应 Set-Cookie 单独交回宿主；裸调用 connect 不会自动获得页面的这些上下文。 (`crates/net/src/websocket.rs`)
- WsHost::connect 必须在 JS 回调线程同步捕获导航代际，但 TCP/TLS 与读泵必须异步；额外 wrapper 线程会推迟代际捕获并破坏 close-before-insert 仲裁。 (`crates/engine/src/ws_bridge.rs`)
- WebSocket 的 SetCookie 内部事件先于 Open，页面不收到 cookie 消息事件；连接错误和最终 Close 分开，不能仅凭 onerror 将连接状态提前终结。 (`crates/engine/src/ws_bridge.rs`)
- 本地 file URL 读取会加载同名 .headers sidecar，已有 Content-Type 优先于扩展名猜测；该读取函数没有目录白名单，dot-dot 规范化不是文件访问授权。 (`crates/net/src/file_url.rs`)

## Architecture
- WebSocket 事件 wire 的 reason、protocol、error 会剥离字段分隔符，text/binary 数据放末字段；每连接无 pending 时排队，上限 4096，溢出主动合成策略关闭而不是无限保留。 (`crates/engine/src/ws_bridge.rs`)
- JavaScript MIME 判定函数接收 MIME essence，并兼容 legacy text/javascript1.x、text/jscript 等类型；调用方必须先去掉参数，不能把完整带 charset 的 Content-Type 原样当 essence。 (`crates/net/src/script_mime.rs`)
- ParsedUrl::to_url_string 省略 fragment、保留 userinfo 与 query；它是手工重建的网络 URL，不是通用 WHATWG origin 或安全日志序列化器。 (`crates/net/src/url_parser.rs`)
- 每源限流使用 url crate 的 origin ASCII 序列化，默认每源 6、全局 24；解析失败时退回原字符串作分组，不代表 URL 已通过传输层合法性校验。 (`crates/net/src/resource_policy.rs`)
- WsBridge 顶部仍有 renderer CSP 未接线的历史说明，安全强制性必须另查当前宿主注入与执行检查点，不能把这段注释或 API 存在作为所有产品路径的最终结论。 (`crates/engine/src/ws_bridge.rs`, `apps/browser/src/fetch_proxy.rs`)
