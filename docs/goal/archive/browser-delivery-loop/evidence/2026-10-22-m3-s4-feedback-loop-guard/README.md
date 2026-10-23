# M3-S4 反馈导出 + 崩溃循环守卫验收（DC-3 后半 + DC-4）

日期：2026-10-22 · 分支：main · 依赖：S3 崩溃报告（5220be556）

## 实现清单

- `crates/browser-shell/src/feedback.rs`（新增，FR-5/IF-4）：报告列举（文件名/角色/时间戳/大小，按时间排序）、文件名信任边界校验（仅 `crash-<数字>-<角色>.json`，拒绝穿越/分隔符）、单份删除、反馈包导出——**目录形态**（依赖图无 zip crate，按 RFC §3 回退，IF-4 由 `.zip` 调整为目录）：`<目标>/zeroweb-feedback-<版本>-<YYYYMMDD>/` 内含 `reports/`（勾选报告逐字节原样）+ `environment.json`（schema/version/git_commit/os/arch/generated_at/报告清单）+ 可选 `session-summary.json`（tab 计数 + 崩溃循环跳过 tab 数；URL 仅显式勾选时落盘）。同名包已存在拒绝（不覆盖不合并）；导出中途失败清理残缺目录。日期换算用 civil-from-days（无 chrono 依赖）。
- `crates/browser-shell/src/session.rs`（FR-6）：`TabSnapshot.crash_count: u32`（`#[serde(default)]`，旧 session.json 照常加载）+ `CRASH_LOOP_SKIPPED_THRESHOLD=2`、`CRASH_LOOP_OBSERVATION_WINDOW_SECS=10`（RFC P5 终值，常量集中）+ `is_crash_loop_skipped()`。
- `crates/browser-shell/src/tab.rs` + `browser.rs`：Tab 崩溃计数存取（新增/恢复/副本归零）、`save_session`/`restore_session` 携带计数、shell 级递增/清零/读取。
- `apps/browser/src/process_backend.rs`：`handle_renderer_ipc_lost`（S3 单一收口）追加崩溃 tab 事件入队；`tab_manager.rs` 转发。
- `apps/browser/src/app.rs`：事件循环消费崩溃事件——计数递增 + 即时落盘（`persist_running_session`，从启动检查提取共用）；观察窗到期清零并落盘；`apply_restored_session` 对 `crash_count ≥ 2` 的 tab 跳过自动加载（置空 about:blank，shell 侧 URL 保留供手动重载参考），带非零计数的恢复 tab 一律进入观察窗。
- `apps/browser/src/shutdown_signal.rs`：补 Unix SIGINT handler（原为 no-op——默认 SIGINT 直接杀进程，跳过 `persist_user_data` 且留孤儿 renderer，与 Windows 侧动机相同；模块文档原挂账项）。handler 仅 atomic store（async-signal-safe）。
- `apps/browser/src/main.rs`（M3 最小 CLI 面，RFC §4.4）：`--list-crash-reports`（预览行 `REPORT <file> <role> <ts> <size>`）、`--delete-crash-report=<file>`、`--export-feedback=<dir>`（+ `--feedback-report=<file>` 可重复勾选，缺省全部；无报告拒绝不产空包）+ `--include-session-summary` / `--include-session-urls`。

## 单元测试

- browser-shell：feedback 6 项（文件名解析/穿越拒绝、列举排序跳过外来文件、删除、导出结构+environment 字段+摘要默认无 URL、URL 显式勾选、重复/非法名/缺失/空勾选与无副作用）+ session 2 项（旧文件缺字段加载为 0、计数 roundtrip 达阈值）。全包 366 测通过。
- zero-browser bin：`restore_skips_tabs_that_reached_crash_loop_threshold`（DC-4 回归：达阈值 tab 置空 about:blank、未达阈值照常恢复、shell 侧 URL 保留、计数留待观察窗清零）。

## 真实进程验收（17/17 PASS）

执行：`sh run-dc3b-dc4-acceptance.sh <repo> <work>`（systemd-run MemoryMax=8G timeout 900 守卫），日志见 [acceptance-run.log](acceptance-run.log)。

- **leg A（崩溃循环守卫，真实产品 GUI）**：隔离环境启动 → SIGKILL 唯一 tab renderer → session.json `crash_count=1` 即时落盘 → respawn → 观察窗内第二次 SIGKILL → `crash_count=2` → SIGINT 干净退出（`clean_exit=true`）→ 重启自动恢复：达阈值 tab 跳过自动加载（日志 `skipping auto-load of crash-loop tab`，置空 about:blank）→ 存活观察窗 ~10s 后计数清零落盘。
- **leg B（反馈导出，真实二进制 CLI）**：`--list-crash-reports` 预览行合规 → `--export-feedback` 生成目录包（reports/ 与源逐字节一致、environment version/commit 与 `--version` 一致、摘要默认无 URL 字段、整包隐私 grep `http`/`file://`/`/home/`/work 路径全 0）→ 重复导出拒绝不覆盖 → `--delete-crash-report` 删除生效、`../../etc/passwd` 拒绝 → `--include-session-urls` 显式勾选后 URL 才进摘要 → 无报告导出拒绝（`FEEDBACK_REJECTED NO_REPORTS`）且不产生任何文件（拒绝=不发送）。

## 排障记录（影响实现与脚本）

- 首轮验收发现 Unix SIGINT 缺口：`shutdown_signal::install()` 非 Windows 为 no-op，SIGINT 直接杀进程 → `clean_exit` 永为 false，自动恢复路径不可达。已按模块文档挂账方向补 `libc::signal(SIGINT, ...)` handler（见上）。
- 脚本首版两次 kill 落在同一 unix 秒：报告文件名 `crash-<unix秒>-<角色>.json` 同名互相覆盖（残留 1 份）。等待锚点改为「报告 #1 文件名时间戳」严格跨秒后才补第二刀。
- 导出验收断言修正：拒绝路径目录不存在（`! -e`）才是期望结果。

## 限制与挂账（非本切片）

- 导出为目录形态（IF-4 偏差，RFC §3 已预授权回退）；zip 形态待依赖图引入 zip crate 后另行评估。
- GUI 菜单导出入口未做（RFC §4.4：视菜单基建成本为可选增强，不阻塞验收）。
- CLI 导出默认勾选全部报告；逐项预览内删除经 `--delete-crash-report` 组合表达。
- 观察窗清零依赖事件循环存活；进程在窗口期内死亡则计数保留，下次恢复仍按阈值跳过（保守方向，符合守卫目的）。
- SIGINT graceful 化后，事件循环卡死时 Ctrl+C 不再强杀（Windows 侧同款取舍）；SIGTERM 保持强杀。
