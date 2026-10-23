# G3：`--profile` CLI（独立 profile 路径收敛）

**日期**：2026-10-22
**对应**：[M1 证据 §4](../2026-10-22-m1-inventory-baseline/README.md) 缺口 G3（此前无 `--profile`，多 profile/隔离验收只能靠环境变量拼装）
**产物与脚本**：[run-g3-profile-acceptance.sh](run-g3-profile-acceptance.sh) / 原始输出 [g3-acceptance-run.log](g3-acceptance-run.log)（**15 断言全 PASS**）

## 1. 背景与设计

M1 记档 G3：浏览器所有持久化状态（配置/存储/缓存）此前只能靠 `XDG_CONFIG_HOME`/`ZERO_STORAGE_DIR`/`ZERO_CACHE_DIR` 三个环境变量拼装隔离，无单一 CLI 入口；且 `dirs` 对相对 `XDG_CONFIG_HOME` 值会静默忽略并回落真实 `HOME`（数据安全隐患）。

| 文件 | 内容 |
|---|---|
| `crates/runtime-config/src/lib.rs` | 权威清单新增 `ZERO_CONFIG_DIR`（[docs/runtime-environment.md](../../../../../runtime-environment.md) 同步） |
| `crates/browser-shell/src/profile.rs` | 新增收口助手 `config_base_dir()`：`ZERO_CONFIG_DIR` 优先，否则平台配置目录 |
| `crates/browser-shell/src/{bookmarks,settings,session,crash_report}.rs` | 4 处 `dirs::config_dir()` 站点统一改为 `config_base_dir().join("zeroweb")` |
| `apps/browser/src/main.rs` | `--profile[=]<dir>` 解析（两形态，空值拒绝）+ `apply_profile()`：相对路径按 CWD 绝对化后单点注入 `ZERO_CONFIG_DIR`/`ZERO_STORAGE_DIR=<root>/storage`/`ZERO_CACHE_DIR=<root>/cache`（CLI 显式覆盖环境变量，对齐 Chrome `--user-data-dir` 语义）；main() 重排——解析提前于 panic hook，使崩溃报告目录随 profile（含启动期 panic） |
| `apps/browser/src/tests.rs` | 单测 3 项：两形态解析、空值拒绝、env 布局契约（绝对化 + 三变量落位） |

**布局契约**：配置=`<root>/zeroweb/`（session/bookmarks/history/settings/crash-reports）、存储=`<root>/storage/`、缓存=`<root>/cache/`。renderer 子进程经 env 继承同一隔离，无需 IPC 变化。

**取舍**：
- 注入机制选环境变量而非穿参——storage/cache 在 renderer 进程内解析，穿参需要扩 IPC 契约；env 继承与既有 `ZERO_*` 模式同构。
- `--profile` 覆盖同名环境变量（CLI 最显式）；未传 `--profile` 时行为与此前完全一致。
- 相对路径显式 `std::path::absolute`，消除 `dirs` 静默忽略相对 XDG 值回落真实 HOME 的隐患。

## 2. 真实进程验收（三腿 15 断言全 PASS）

隔离：环境变量兜底均指向 work-dir（即使 `--profile` 失效，真实 `HOME` 零接触）；守卫 `systemd-run -p MemoryMax=8G timeout 300`。

| 腿 | 场景 | 断言 | 结果 |
|---|---|---|---|
| leg A | GUI + `--profile`，环境变量指向 scratch（覆盖语义） | GUI 起（CDP `/json/version`）；live renderer 继承 `ZERO_STORAGE_DIR=<profile>/storage` 与 `ZERO_CACHE_DIR=<profile>/cache`；SIGINT ≤10s 退出码 0；`session.json` 落 `<profile>/zeroweb/`；scratch XDG 零 session 写入 | PASS ×7 |
| leg B | renderer SIGKILL 崩溃 + CLI 面（GUI 无 CDP，S3 先例） | 报告落 `<profile>/zeroweb/crash-reports/`；`--list-crash-reports --profile` 列得；不带 `--profile` 时 scratch XDG 列 `NO_REPORTS`（双向隔离）；报告 `version`/`git_commit` 与 `--version` 贯穿一致（26.10.22 / `6067f280c794`）；SIGINT 后 browser 干净退出（1 crash < 循环守卫阈值） | PASS ×7 |
| leg C | 相对路径 profile | cwd=work-dir 下 `--profile=p2`（相对）列得同一报告（CWD 绝对化） | PASS ×1 |

注：`/proc/<browser>/environ` 只反映 exec 时刻环境，观测不到运行期 `set_var`——browser 侧注入由 renderer 继承断言与 session.json 落盘位置证明。

## 3. 门禁

fmt PASS / 定向单测 `profile_` 3/3 PASS + `zero-browser-shell` 366/366 PASS / `cargo build -p zero-browser` PASS / clippy（三 touched crate `--all-targets`）零告警 / release 构建（守卫）PASS / 验收守卫 `systemd-run -p MemoryMax=8G timeout 300` / 全量 `make test` PASS（test-guard 包裹）/ 提交前 lei-pre-commit-guard PASS。

## 4. 限制与挂账

- `--profile` 只收敛路径；GUI 内切换 profile、profile 加锁（同目录双开互斥）未做（无需求驱动）。
- headless 不消费启动 URL 仍挂 G5；GUI 导出入口、反馈包 zip 形态仍挂账。
- 真机安装/签名仍待 P1/P4 授权。
