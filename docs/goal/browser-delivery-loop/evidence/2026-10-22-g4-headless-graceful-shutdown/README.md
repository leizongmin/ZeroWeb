# G4：headless 优雅关停（SIGINT + CDP Browser.close）

**日期**：2026-10-22
**提交**：`696ddeba6`（fix(browser): headless 优雅关停）
**对应**：[M1 证据 §4](../2026-10-22-m1-inventory-baseline/README.md) 缺口 G4、[S2 证据 §5](../2026-10-22-m3-s2-update-rollback/README.md) 挂账（重启语义）、M4 挂账清单
**产物与脚本**：[run-g4-headless-shutdown.sh](run-g4-headless-shutdown.sh) / [ws-browser-close.js](ws-browser-close.js)（裸 RFC6455 驱动，环境无 python WS 库）/ 原始输出 [g4-acceptance-run.log](g4-acceptance-run.log)（**11 断言全 PASS**）

## 1. 背景与本轮发现

M1 记档 G4：headless 不接信号、不落 `session.json`，更新流"退出旧版→换新→重启"走 headless/信号路径时会话以 crash 语义留存。S4 已附带修复 `shutdown_signal` 的 Unix no-op（SIGINT → 全局 flag），但这暴露了一个此前被默认处置掩盖的新问题：**headless mux 循环不消费该 flag**——GUI 事件循环每轮检查 flag，headless 无人检查，SIGINT 从"默认立即死"变成"永不退出"（进程悬挂、端口占用、renderer 孤儿）；M4 串测走 GUI 腿未触及此路径。

同族缺口：CDP `Browser.close` 是空壳 stub——返回 `{"result":"closing"}` 但不关停任何东西，自动化标准停止路径（playwright/puppeteer 语义）失效。

## 2. 实现

| 文件 | 内容 |
|---|---|
| `apps/browser/src/headless/mod.rs` | `HeadlessServer` 加 `shutdown_requested: AtomicBool`；mux 复用循环顶（3ms tick）检查 `shutdown_signal::is_set() \|\| shutdown_requested` → break → `Ok(())`。`session` 随作用域 Drop 经 `RendererHandle::drop`（zero-protocol）回收 renderer 子进程；`run()` 文档同步 |
| `apps/browser/src/headless/domains/bidi.rs` | `cmd_browser_close` 置位 flag；响应形状 `{"result":"closing"}` 不变（自动化兼容，既有两处断言不破） |
| `apps/browser/src/headless/tests.rs` | `test_dispatch_browser_close` 加 flag 断言：前置 false → dispatch → true（回归固化 stub→真实） |

**设计决策（验收断言固化）**：headless **不写 `session.json`**。headless target 是自动化临时面，写 `clean_exit=true` 反会吞掉 GUI 侧待恢复的崩溃提示——会话语义仍归 GUI 生命周期管。M1 记档的"不落 session.json"由此从缺陷转为明确决策。

## 3. 真实进程验收（两腿 11 断言全 PASS）

隔离：`XDG_CONFIG_HOME`/`ZERO_STORAGE_DIR`/`ZERO_CACHE_DIR` 均在 work-dir；守卫 `systemd-run -p MemoryMax=8G timeout 300`。

| 腿 | 场景 | 断言 | 结果 |
|---|---|---|---|
| leg A | SIGINT（Ctrl+C / `kill -INT`） | server 起动（stdout DevTools URL + `/json/version` 应答）；1 个 renderer 子进程；`kill -INT` → ≤10s 退出码 0；关停日志 `Headless server shutting down`；无 renderer 孤儿（/proc exe 归属本次构建）；**无 session.json 产生** | PASS ×6 |
| leg B | CDP `Browser.close`（裸 WS 握手驱动） | server 起动；WS 响应 `{"id":1,"result":{"result":"closing"}}`；≤10s 退出码 0；关停日志；无 renderer 孤儿 | PASS ×5 |

两条路径共用同一消费点（循环顶 flag 检查），leg A 证机制、leg B 证 CDP 置位臂，端到端覆盖 stub 修复。

## 4. 门禁

fmt PASS / 定向单测 `headless::tests::test_dispatch*` 26/26 PASS / `cargo build -p zero-browser` PASS / clippy `--workspace --all-targets -D warnings` 零告警 / `make test` PASS（test-guard 包裹）/ 提交前 lei-pre-commit-guard PASS。

## 5. 限制与挂账

- SIGTERM 保持默认强杀语义（`shutdown_signal` 模块文档既有取舍，GUI/headless 一致，未在本切片扩大）。
- G4 关闭；缺口表余 G3（无 `--profile` CLI）/ G5（headless 不消费启动 URL）。
- 更新流"退出旧版→换新→重启"的 headless 路径现可在优雅关停前提下推进（会话留存仍按 GUI 语义，headless 不干预）。
