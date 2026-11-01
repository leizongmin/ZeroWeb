# M4 DC-5：已交付修复面的生产 IPC 复测（headless CDP 腿 + GUI 腿）

**日期**：2026-11-01
**HEAD（基线）**：`5a2aafaf9`（M4 前置 push 后；release 二进制按同 HEAD 重建）
**范围**：DC-5「生产接收方复测」对**已交付**修复面先行执行，不等决策 ①②③④；决策落定后新增修复面的复测仍留 M4 收口。

## 一、headless CDP 腿（多进程生产形态；8/8 PASS）

真实 `target/release/zero-browser --headless --remote-debugging-port=<p>`（多进程 renderer 子进程，日志 `Tab runtime: multi-process`），CDP 生产面驱动（Runtime.evaluate / Page.navigate / HTTP 发现），双 http origin（127.0.0.1:18801/18802）+ 一次性 Xvfb：

| 场景 | 修复切片 | 结果 |
|---|---|---|
| evil Host 头（`evil.example.com`）访问 `/json/version` | M2 切片 2 | 拒绝（403 或 RST，服务端 `non-local Host header rejected` WARN 在册） |
| 本机 Host 访问 `/json/version` | M2 切片 2 | `200 OK`，`Browser=ZeroWeb/26.11.1` |
| JS `localStorage.setItem`+`getItem`（同文档） | 生产面健全性 | set/read 一致 |
| 同 origin 重导航后 `getItem` | — | `null`（见发现 1：每文档 stub） |
| `document.cookie` set+read（renderer 本地 jar 写读） | 切片 3 前置 | `zwk=zwv` 读回 |
| token 实例 HTTP 发现（无 token 门） | M2 切片 4 | `200 OK`（token 只门 WS，与设计一致） |
| WS 首消息无 token | M2 切片 4 | `code=-32001 "Authentication required"`，连接终止 |
| WS 首消息带 token | M2 切片 4 | `Runtime.evaluate` 正常应答 |

## 二、GUI 腿：ClearHttpCookies 广播全语义链（M2 切片 3；3/3 PASS）

headless 无 App 入口（见发现 3），`zero://settings/clear/cookies` 为 App 专有拦截（app.rs `try_apply_settings_url`），生产链在 GUI 形态验证（真实窗口 + 一次性 Xvfb，`ZW_RENDERER_STDERR_ECHO=1` 收 renderer 子进程日志）：

1. **renderer jar 写入+消费**：http 服务回 `Set-Cookie: zwk=zwv` 文档页，页内脚本开 WS——WS 握手请求携带 `Cookie: zwk=zwv`（服务器侧记录）。jar 的生产写路径（`FetchResponse`→`store_set_cookie_headers`）与消费路径（`ws_handshake_cookie`）均真实经过。
2. **App 拦截**：Ctrl+T（地址栏聚焦全选）输入 `zero://settings/clear/cookies` 回车——窗口 title 变「设置 - ZeroBrowser」（`apply_settings_clear_cookies` 末语句 `open_settings_page` 到达即整函数执行毕：共享 jar clear + `tabs.clear_renderer_http_cookies()` 广播均已发生）。
3. **广播接收**：两个 renderer 子进程（tab1 持 cookie 方 + tab2）同刻日志 `renderer 本地 cookie jar 已按浏览器指令清空`（06:48:46.371）。

同腿顺带复核 M3 修复面：compositor `seccomp 网络/exec 过滤已启用 (gpu=true)` + `landlock 文件系统沙箱已启用 (gpu=true)` 零降级；renderer `seccomp 沙箱钩子已启用（P3 占位）`×3（决策 ③ 现状锚点）。

## 三、复测中的发现（如实记档）

1. **JS Web Storage 为每文档内存 stub（既有功能缺口，非本 goal 修复回归）**：`dom_bridge.rs` Web Storage API Stub 自述 "Data is session-scoped (not persisted to disk); real persistence by host runtime"——重导航后 `getItem` 恒 `null`。browser 侧 StorageManager + StorageOp IPC + 服务端 origin 绑定（切片 1 修复面）机制在 IPC handler 级常驻回归覆盖；JS 可见面与 host 存储的接线是独立功能缺口，记档不修（非本 goal 范围）。
2. **HTTP 请求不附着 cookie 为既有设计（t8k-delta F1）**：`resource_loader.rs:389` FIXME——cookie 注入因 context-free 超发 SameSite 已摘除，jar 消费面 = WS 握手。首版 probe（`<img>` 子资源带 Cookie 头）观察面选错即源于此，改 WS 握手头后通过。
3. **headless `Page.navigate` 到 `zero://settings/*` 阻塞 mux（自动化面缺口，非安全问题）**：headless 会话无 App，URL 直达 renderer `wait_for_load` 无 load 事件，mux 单线程阻塞 ~15s（`[mux-debug] slow tick 15001ms`）；同连接后续命令全部饿死至等待超时。清除链在 GUI 腿覆盖，此缺口记档。
4. **evil Host 拒绝的 403/RST 竞速**：mux 以非阻塞 Peek 读头（不消费），`reject_http_host` 写 403 后连接带未读数据关闭→Linux 发 RST 代 FIN，客户端可能读不到 403 字节。拒绝语义（WARN + 连接必拒）不受影响。

## 四、DC-5 剩余项（M4 收口时补）

- 决策 ①②③④ 对应新修复面的复测（拍板后）。
- 多进程 HSTS 注册生产面：需受信 TLS + STS 服务端（回环 STS 门回归在册常驻，生产面重放受证书信任所限，记复测方法限制）。
- fd 通道 dma-buf 端到端：DRI3 环境受限在册。

## 五、环境事故（诚实记档）

本轮早前以 `pkill -x Xvfb` 清理自起实例，模式误杀 **:20 用户实时 xfce 桌面 Xvfb**（非本任务所起）。会话（xfce4-panel 等）无法盲恢复，已如实报告用户；教训固化：清理只 kill 自起实例的已记录 PID，禁止按进程名模式杀 Xvfb。本会话后续所有一次性 Xvfb 均按 PID 精确回收。

## 验证命令

- headless 腿：`python3 /tmp/rally-cdp/prod_retest.py` → `SUMMARY: 8/8 passed`（驱动脚本仓外，命令与断言已如实录上）
- GUI 腿：`bash /tmp/rally-cdp/gui_cookie_clear.sh`（Xvfb :54 自起自收，输出三 PASS 见上）
- 清理核查：无 zero-browser/zero-renderer/zero-compositor/自起 Xvfb 残留（仓外 4 天旧 http.server 属并行网站流，未触碰）
