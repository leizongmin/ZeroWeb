# G5：headless 消费启动 URL

**日期**：2026-10-22
**对应**：[M1 证据 §4](../2026-10-22-m1-inventory-baseline/README.md) 缺口 G5（headless 位置参数 `<url>` 不生效，初始 tab 恒 `about:blank`——启动渲染腿曾因此 INCONCLUSIVE 改判）
**产物与脚本**：[run-g5-headless-startup-url.sh](run-g5-headless-startup-url.sh) / 原始输出 [g5-acceptance-run.log](g5-acceptance-run.log)（**14 断言全 PASS**）

## 1. 背景与实现

M1 记档 G5：`run_headless` 只传 port/viewport，CLI 位置参数启动 URL 被静默丢弃；M1 启动渲染腿与更新流 headless 验收便利性（DC-2 首步）都受影响。S3 记档同期注记：GUI 模式的 CDP 辅助会话若也消费启动 URL 会同 URL 双加载。

| 文件 | 内容 |
|---|---|
| `apps/browser/src/headless/mod.rs` | `HeadlessServer` 加 `startup_url` 字段；`run()` 在 session 创建后、mux 循环前消费——与 CDP `browsingContext.navigate` 同一管线（`shell.navigate` + `navigate_renderer`（含 `wait_for_load`）+ `on_page_loaded`），scheme 启发式与 GUI 地址栏同一 `normalize_url`；启动导航失败仅 `warn` 不阻断 server（保持可附接调试）。GUI 模式 CDP 辅助会话恒 `None`（启动 URL 归 GUI 窗口，避免双加载） |
| `apps/browser/src/main.rs` | `run_headless` 传入 `cli.startup_url`；usage 文本更新（headless 语义注明） |
| 测试构造点 | `headless/tests.rs`、`headless/gpu_screenshot_tests.rs` 全部 `HeadlessServer::new(..., None)` |

## 2. 真实进程验收（三腿 14 断言全 PASS）

隔离：`XDG_CONFIG_HOME`/`ZERO_STORAGE_DIR`/`ZERO_CACHE_DIR` 均在 work-dir；守卫 `systemd-run -p MemoryMax=8G timeout 300`。

| 腿 | 场景 | 断言 | 结果 |
|---|---|---|---|
| leg A | 本地 HTTP fixture + 位置参数 URL | server 起；`/json/list` 目标 URL 翻转为启动 URL；fixture 服务器收到 `GET /index.html`（真网络加载）；启动导航日志且无失败告警；SIGINT ≤10s 退出码 0（G4 优雅路径不受影响）；无 renderer 孤儿 | PASS ×7 |
| leg B | 不带位置参数 | 初始 tab 恒 `about:blank`（缺省回归）；干净退出 | PASS ×3 |
| leg C | 裸 `host:port` + 不可达 URL | 按地址栏同一 `normalize_url` 升级 `http://`；不可达启动导航失败仅告警、server 仍可发现可附接；干净退出 | PASS ×4 |

**排障记档**（不遮掩）：首轮验收三腿在 SIGINT 后 10s 未退（exit 999）。定位发现环境含 **M4 轮遗留的两个 6h+ 僵尸 headless 服务**（`target/packages/instroot`（已删）下的 pre-G4 二进制——正是 G4 修复前「SIGINT 永不退出」缺陷的活体残留，M4 清理审计时逃逸）；清理后同二进制 5/5 探针 0.1s 退出、验收 14/14 PASS。脚本相应硬化：开跑前预检本 repo 杂散 `zero-browser` 进程（发现即 FAIL），SIGINT 轮询改 G4 式逐腿内联 + `/proc` 僵尸态判别（命令替换子 shell 的 `wait` 取不到非自身子进程退出码，`kill -0` 无法区分僵尸与存活）。

## 3. 门禁

fmt PASS / `headless::` 单测 125/125 PASS / release 构建（守卫）PASS / clippy `--workspace --all-targets -D warnings` 零告警 / 全量 `make test` PASS（test-guard 包裹）/ 验收守卫 `systemd-run -p MemoryMax=8G timeout 300` / 提交前 lei-pre-commit-guard PASS。

## 4. 限制与挂账

- 启动导航在 mux 循环前阻塞至 `wait_for_load` 完成/超时（与 CDP navigate 同语义）；监听 socket 已先绑定，连接积压在 backlog 不丢。
- headless 不写 `session.json` 维持 G4 设计决策不变。
- 缺口表 G1–G5 全部关闭。剩余挂账：GUI 导出入口（RFC §4.4 可选）、反馈包 zip 形态；真机安装/签名待 P1/P4。
