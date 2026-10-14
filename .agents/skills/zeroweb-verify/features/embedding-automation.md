# F10/F11 嵌入 API、自动化与调试

## 用户任务与预期

宿主构建 WebView，加载 HTML/URL、注入 CSS、resize、订阅事件并派发用户动作。
自动化客户端用 WebDriver/CDP 导航、查找元素、输入、执行脚本、截图、关闭 session；
DevTools 查看活 DOM、Console 和 Network。
契约来自 [WebView](../../../../crates/webview/README.md)、
[WebDriver](../../../../apps/webdriver/README.md)、
[CDP 矩阵](../../../../tests/playwright-matrix/README.md)，后者保存已有命令覆盖，不复制。

## 前置与操作

从 [主入口](../SKILL.md) 选具体子行为与 Doctor。WebView 的 JS feature/V8 依赖按既有
门禁准备；自动化需当前 renderer 等 peer，可用独立 loopback 端口及本次 profile。
相关生产/测试：
`crates/webview/src/webview.rs`、`src/tests/user_actions.rs`、
`tests/integration/src/webview_product_smoke.rs`、
`apps/webdriver/src/session.rs`、`apps/webdriver/tests/http_session.rs`、
`apps/browser/tests/cdp_navigate_evaluate.rs`、
`apps/browser/src/headless/domains`。阅读目标断言后用已有守卫定向运行，不默认全包或全图。

已准备 `target/release/zero-webdriver`/renderer 与守卫时，项目根启动：

```sh
./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- \
  ./target/release/zero-webdriver --port 19515
```

仅在端口无既有实例且数据路径已隔离时执行。另一终端用 curl `GET /status` Doctor，
`POST /session` 保存返回 id，`POST /session/{id}/url` 导航本次 localhost fixture，
Find Element、Send Value/Click、Get Text/Property、Execute Sync 读取页面结果，
最后 `DELETE /session/{id}`。具体 JSON/wire 格式直接按 WebDriver README/HTTP 测试，
不硬编码旧 session id。截图是 CPU 视口输出，不是生产窗口 GPU。

CDP 复用 `tests/playwright-matrix/scripts/capture-core-flow.mjs`；
脚本及 Makefile 的已有输出目录/参数须先核对，确保不会覆盖既有记录，再选已授权路径。
DevTools 经 `--remote-debugging-port` 的 loopback 实例连接，读取活 DOM、修改并观察、
Console 返回值、Network 请求行。服务启动或 `/json/version` 不是动作 PASS。

## 判据、清理与证据

嵌入检查真实标题/DOM/事件/图元与 resize/CSS 的变化，错误回调和资源释放。
协议动作检查 live 页值、事件与副作用；stale/no such element/javascript error 应为
正确协议错误而非空成功。HTTP 测试有真实 TCP/renderer；单纯 parser 测试边界不同。
保存请求响应、steps-report、session/PID 和错误；删除 session 后核对 renderer 退出，
只停本次服务/页面并清本次 profile，保留日志重读。异常重建本次 session 后 Doctor。
2026-10-14 F10/F11 全部 `NOT_RUN`；版本环境见 [初始化记录](../initialization.md)。
