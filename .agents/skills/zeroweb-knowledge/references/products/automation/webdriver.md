# WebDriver 会话与 renderer 命令桥

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

定位 `zero-webdriver` HTTP 路由、会话、live DOM 元素引用、同步/异步脚本与 CPU 截图。main.rs 处理 W3C 外形，session.rs 管每会话 RendererHandle，renderer automation.rs 执行页面动作；它不是桌面浏览器 UI 驱动。

关键入口：`Driver::create_session`、`find_element`、`execute_script_async`、`Session::request`、`register_element`、`session_screenshot`。IPC 与 renderer 消息职责见[renderer 导航循环](../../runtime/renderer/navigation-and-loop.md)。

验证导航：[自动化地图](../../../../zeroweb-verify/features/embedding-automation.md)，HTTP 回归入口为 `apps/webdriver/tests/http_session.rs`，包含 missing/stale 元素和真实表单值断言；本轮未执行，地图为工作区新增，不归上述 HEAD。

## Gotchas
- 服务绑定 loopback 并串行处理 HTTP，单请求头/体有 1MiB 上限；Access-Control-Allow-Origin:* 不是身份认证，/status ready 也不表示客户端要求的所有 capabilities 都满足。 (`apps/webdriver/src/main.rs`)
- 每个 WebDriver session 独立启动 renderer，设 Legacy frame publish 和 800×600、device_scale_factor=1；它与 headless 多连接共享一 session 的资源模型不同。 (`apps/webdriver/src/session.rs`)
- 默认 script=10 秒、pageLoad=15 秒、implicit=0；implicit 目前只是可读写存储态，find_element/find_elements 直接发一次请求，不因 POST timeouts 成功就实现隐式重试等待。 (`apps/webdriver/src/session.rs`, `apps/webdriver/tests/http_session.rs`)
- 元素 ID 为本 session 的 e<hex> 不透明编号，同 AutomationElementRef 复用编号，最多登记 4096 个；不能把 DOM nodeId 或另一 session 的编号当有效元素引用。 (`apps/webdriver/src/session.rs`)
- stale 检查同时依赖 renderer document_generation 与宿主 history_epoch，后者补 back/forward 未换代的路径；导航历史变化后旧引用仍须失效。 (`apps/webdriver/src/session.rs`, `apps/webdriver/tests/http_session.rs`)
- find_element 无匹配报 no such element，find_elements 无匹配返回空数组；把二者统一成空成功会破坏 W3C 调用方的控制流。 (`apps/webdriver/src/session.rs`, `apps/webdriver/tests/http_session.rs`)
- 窗口句柄是单窗口架构的固定值，rect/maximize/fullscreen 处理 renderer viewport，不创建真实 OS 窗口；窗口端点成功不能证明系统最大化或 GUI 布局改变。 (`apps/webdriver/src/session.rs`, `apps/webdriver/tests/http_session.rs`)
- screenshot 用 paint snapshot 和跨帧 ImageCache 累积图片 payload 后做 CPU 光栅，丢弃 stale epoch 帧；base64 PNG 是视口输出，不是生产窗口/compositor GPU 一致性证据。 (`apps/webdriver/src/session.rs`)
- HTTP timeout 当前映射 408，而不是 W3C 常用的 500；元素/会话类错误映射 404，invalid argument 映射 400，客户端兼容排查要看真实状态与 error 字段。 (`apps/webdriver/src/main.rs`)
- WebDriver service-worker 查询桥当前返回空结果，注册返回稳定 unsupported 错误；页面其他网络动作成功不能推断此宿主支持 service-worker 注册。 (`apps/webdriver/src/session.rs`)
- execute_script_async 的 callback 未调用最终报 javascript error/HTTP 500，不走 timeout/408；轮询前每次 automation request 也受 script timeout，外层 deadline 不表示整个 HTTP 动作精确在该时刻终止。 (`apps/webdriver/src/session.rs`, `apps/webdriver/src/main.rs`, `apps/webdriver/tests/http_session.rs`)

## Architecture
- execute_script_async 在宿主注入 callback 完成状态并轮询，script timeout 由 session 持有；同步脚本请求和异步回调完成是两层不同的等待，返回初始执行结果不代表异步任务完成。 (`apps/webdriver/src/session.rs`, `apps/webdriver/tests/http_session.rs`)
- renderer automation 的查找、点击、输入以 live Document 和 typed AutomationOperation 执行，session 只保留 opaque 引用与请求关联；元素状态应向 renderer 查询，不从旧截图或 HTML 字符串推断。 (`apps/renderer/src/automation.rs`, `apps/webdriver/src/session.rs`)
- renderer 挂起 Evaluate/句柄操作时以 pending 表长度变化拦住 Empty 占位响应，主循环随后按原 request_id 补答；不能把内部返回 Empty 当外部操作完成，补答还需执行与同步路径相同的状态收尾。 (`apps/renderer/src/automation.rs`)

## Testing Strategy
- HTTP 表单测试把 Unicode Send Keys、Tab 焦点移动、checkbox/button 点击和同 live page ExecuteScript 连起来，断言 Zoé、checked 与 title；只有 click 端点 200 不能证明输入到达控件。 (`apps/webdriver/tests/http_session.rs`)
- screenshot 回归同时检查 about:blank 无帧错误、PNG 魔数、800×600 解码尺寸和蓝块中心像素；它证明 CPU 视口内容，不覆盖元素裁剪、多窗口或真实 GPU 呈现。 (`apps/webdriver/tests/http_session.rs`, `apps/webdriver/src/session.rs`)
