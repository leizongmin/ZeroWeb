# M3-S1：版本追溯——`--version` 与构建 commit 注入

**日期**：2026-10-22
**对应**：[M2 RFC](../../m2-delivery-rfc.md) FR-1 / IF-1；缺口 G1/G2 关闭
**HEAD**：实施时记录（见 §3 产物哈希行）

## 1. 实现

- `build-support/product_version.rs`：新增 `resolve_commit()`——显式 `ZERO_BUILD_COMMIT` env 优先，其次 `git rev-parse --short=12 HEAD`，均不可得 `unknown`；`emit_cargo_env()` 增发 `cargo:rustc-env=ZERO_BUILD_COMMIT` + `rerun-if-env-changed` + `rerun-if-changed`（`.git/HEAD`、`.git/logs/HEAD`，尽力跟随本地 HEAD 变化；打包路径显式传 env 保证确定性）。
- `crates/product-version/src/lib.rs`：`BUILD_COMMIT` 常量 + `version_string()` → `ZeroBrowser <YY.M.D> (commit <hash>)`。
- `apps/browser/src/main.rs`：`parse_args()` 早退分支 `--version` → stdout 单行 + exit 0（与 `--help` 同位，先于 GUI/headless 启动）；usage 补 `--version` 行。
- 三打包脚本（linux/macos/windows）：导出 `ZERO_BUILD_COMMIT`（`git -C "$PROJECT_ROOT"` / `git -C $ProjectRoot`，无 git 回退 `unknown`）。

## 2. 验证

| 检查 | 结果 | 证据 |
|---|---|---|
| `cargo fmt --all -- --check` | PASS（无 diff） | 本地运行 |
| `cargo test -p zero-product-version` | PASS 6/6（含 `version_string_contains_version_and_commit`、`embedded_commit_is_short_hash_or_unknown`） | test-guard 包裹（4G/8G/300s） |
| `make test`（workspace） | PASS（exit 0，末段 310 passed / 0 failed） | test-guard 包裹 |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS（exit 0 零告警） | systemd scope 24G 包裹 |
| `make browser-build` 后 `./target/release/zero-browser --version` | PASS：stdout 单行 `ZeroBrowser 26.10.22 (commit 9b0703937ae8)`，与 `git rev-parse --short=12 HEAD` 逐字符一致；stderr 既有启动日志不影响 stdout 纯净（IF-1） | systemd scope 24G 包裹 |
| .deb 重打包后前缀内 `--version` 与宿主 git 一致 | PASS：解包前缀 `./instroot/usr/bin/zero-browser --version` 输出同上 | §3 |

## 3. 产物记录

- HEAD：`9b0703937ae8...`（提交 20a28fb84 rebase 后的 main，工作树含本切片改动）
- .deb：`target/packages/zero-browser_26.10.22_amd64.deb`，SHA256 `638b63ef786a0cfd560e95ddfc9062e62a8f399ae88d12ac9f73b4ffd9251d9c`（`sha256-26.10.22.txt`；覆盖 M1 基线产物 `9cb178f6...`）
- 版本行：`ZeroBrowser 26.10.22 (commit 9b0703937ae8)`（宿主直跑与前缀解包两路一致）
- 已知限制：非打包的本地增量构建在 HEAD 变化后 hash 可能滞后（rerun-if-changed 尽力覆盖 `.git/HEAD` 与 `.git/logs/HEAD`，packed-refs 场景除外）；打包与验收一律显式传 `ZERO_BUILD_COMMIT`（三脚本已内置）或全量重编。

## 4. 清理

解包前缀 `target/packages/instroot` 已删除；.deb 与哈希文件保留作旧安装产物（goal 契约）。无残留进程。
