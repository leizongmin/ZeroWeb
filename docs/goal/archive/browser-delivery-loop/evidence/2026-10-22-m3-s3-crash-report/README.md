# M3-S3 本地崩溃报告验收（DC-3 前半）

日期：2026-10-22 · 分支：main · 基线：be38b2ee1（S2 更新回滚）

## 实现清单

- `crates/browser-shell/src/crash_report.rs`（新增）：`CrashReport` 白名单结构（IF-3 全字段）、`crash_reports_dir()`（`<配置目录>/zeroweb/crash-reports`）、`write_report()`（64 KiB 上限 + 保留 10 份删最旧 + 原子写）、`install_browser_panic_hook()`（钩子链：记录后调原钩子；AtomicBool 重入保护）、panic message 2 KiB 截断（UTF-8 字符边界）、`repo_relative_location()`（非仓路径丢弃）。
- `apps/browser/src/main.rs`：main() 入口安装 browser panic 钩子（版本/commit 取自 `zero-product-version`）。
- `apps/browser/src/process_backend.rs`：报告写入收口在 `handle_renderer_ipc_lost` 开头，覆盖 `handle_crashes`（显式检查）与 poll IPC 断开两条路径。
- `crates/browser-shell/examples/crash_probe.rs`：验收探针，真实进程安装生产钩子后注入 panic。

## 单元测试（8 项全绿）

`cargo test -p zero-browser-shell`：真实进程 panic 钩子白名单字段精确断言、renderer 报告无 panic 字段、`renderer_exit_status` 剥离 stderr 尾巴、非仓 location 丢弃、长 message 字符边界截断、保留上限滚动删除（10 位宽时间戳字典序）、超 64 KiB 拒写、IF-3 文件名布局。

## 两腿验收（12/12 PASS，复跑两次一致）

执行：`sh run-dc3-s3-acceptance.sh <repo> <work>`（systemd-run MemoryMax=8G timeout 900 守卫），日志见 [acceptance-run.log](acceptance-run.log)。

- **leg A（browser panic）**：`crash_probe` 真实进程退出 101，恰 1 份报告，字段集与 IF-3 精确匹配，`panic_location` 为仓内相对路径（`crates/browser-shell/examples/crash_probe.rs:14`），隐私 grep（`http`/`file://`/`/home/`/work 路径）全 0。
- **leg B（renderer SIGKILL）**：GUI 模式启动真实产品二进制（隔离 XDG_CONFIG_HOME/ZERO_STORAGE_DIR/ZERO_CACHE_DIR，无 CDP 端口），确认唯一非僵尸 tab renderer 后 `kill -9`：报告落盘 `crash-<unix秒>-renderer.json`，`renderer_exit_code: "signal: 9 (SIGKILL)"`，version/git_commit 与 `--version` 输出一致，stderr 尾巴与 URL 不进报告；browser 存活、renderer respawn（新 pid）。

落盘报告样例（leg A / leg B）：

```json
{"schema_version":1,"role":"browser","version":"26.10.22","git_commit":"probe","os":"linux","arch":"x86_64","thread":"main","timestamp":1792668853,"panic_message":"intentional crash for FR-4 acceptance","panic_location":"crates/browser-shell/examples/crash_probe.rs:14"}
{"schema_version":1,"role":"renderer","version":"26.10.22","git_commit":"be38b2ee1828","os":"linux","arch":"x86_64","thread":"main","timestamp":1792668856,"renderer_exit_code":"signal: 9 (SIGKILL)"}
```

## 排障记录（影响脚本设计）

- GUI 模式加 `--remote-debugging-port` 会额外 spawn CDP 辅助会话自己的 renderer，其启动期即 channel EOF 且进程残留（Sl 不 reap）；验收脚本若按 pgrep 顺序选 renderer 会命中它，kill 无检测效果。脚本已去掉 CDP 端口，并断言恰好 1 个 renderer 子进程。
- 报告写入最初只挂在 `handle_crashes`，而 renderer 死亡实际多经 poll IPC 断开路径；移入 `handle_renderer_ipc_lost` 单一收口后两路径均覆盖。

## 限制与挂账（非本切片）

- headless 模式是独立 renderer 管线、无崩溃轮询，renderer 异常退出不写报告（G4 面，既有缺口）。
- GUI + CDP 辅助 renderer 启动即断链死亡且进程不 reap（devtools goal 域）。
- leg B 的 `renderer_exit_code` 断言存在竞态可能：正常退出路径理论上也会产生报告，但 SIGKILL 场景下信号段稳定复现（两次验收均 `"signal: 9 (SIGKILL)"`）。
- 启动期观察到既有的 `Rejected mismatched navigation start/commit` 警告，与崩溃报告无关。
