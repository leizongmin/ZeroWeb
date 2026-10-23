# M3-S2 更新/回滚切片证据

**日期**：2026-10-22
**RFC 依据**：[m2-delivery-rfc](../../m2-delivery-rfc.md) FR-2（本地源更新）、FR-3（显式回滚）、FR-7（机器可读结果）、IF-2（manifest schema v1）、D3（dpkg-deb 暂存 + symlink 翻转 + 回滚）
**验收判据**：DC-2（DELIVERY-02-update-interruption-rollback）
**构建基线**：HEAD `c0fc68c4e5b0`（工作树含本切片改动后构建）

## 1. 实现清单

| 文件 | 内容 |
|---|---|
| `crates/browser-shell/src/update.rs` | 更新模块全量：manifest 解析与安全校验（schema/版本格式/版本序/artifact 名防穿越/SHA256 十六进制）、版本比较（`YY.M.D` 字典序）、复制 + SHA256 校验、`dpkg-deb -x` 暂存、暂存自检（前缀内二进制 `--version` = manifest 版本）、`releases/<版本>` 提升 + `current` 临时链接 rename 原子翻转、健康检查失败自动回滚、显式 `rollback()`。链接目标用 `releases/<版本>` 相对形式（安装根搬移后仍有效） |
| `apps/browser/src/main.rs` | `--check-updates=<dir>` / `--rollback-update` CLI + stdout 单行机器可读结果（`UPDATE_APPLIED <v> (previous <old|none>)` / `UPDATE_ROLLBACK <v>` / `UPDATE_REJECTED <code>`，成功 0 / 拒绝 1）；安装根 = `ZERO_UPDATE_INSTALL_ROOT` 覆盖或 `current_exe` 上溯三级 |
| `crates/browser-shell/Cargo.toml` | `sha2 = { workspace = true }`（依赖图已有，无 lock 变化） |

## 2. 单元测试（8 项全绿）

`cargo test -p zero-browser-shell update`：`test result: ok. 22 passed; 0 failed`（含 update 模块 8 项）

| 测试 | 针对行为 |
|---|---|
| `parses_valid_manifest` | 合法 manifest 解析 |
| `rejects_malicious_and_malformed_manifests` | schema 错误、版本非法、artifact 穿越（`../`、绝对路径、隐藏文件）、SHA256 非十六进制 |
| `compares_versions_in_ymd_order` | `YY.M.D` 字典序：相等/升/降 |
| `applies_update_end_to_end_with_fake_extract` | 复制→校验→暂存→自检→翻转→rollback 链接全链 |
| `rejects_up_to_date_downgrade_missing_artifact_and_bad_hash` | UP_TO_DATE / DOWNGRADE / ARTIFACT_MISSING / HASH_MISMATCH 拒绝分支 |
| `broken_package_fails_staging_check_and_keeps_old_active` | 暂存自检失败：拒绝 + 旧版本保持活跃 |
| `failed_health_check_auto_rolls_back_to_previous` | 翻转后健康检查失败：自动翻回上一版本 |
| `stale_staging_is_cleared_on_next_apply` | 中断残留 staging 在下次应用时清空 |

实现期间测试抓到一处真实 bug：`apply_update` 版本序 match 臂与 `compare_versions` 语义相反（Greater 误判为 Downgrade），已修正并以测试固化。

## 3. 端到端验收（DC-2 六腿 22 断言全 PASS）

产物与脚本：[run-dc2-acceptance.sh](run-dc2-acceptance.sh) / 原始输出 [dc2-acceptance.log](dc2-acceptance.log)。

| 包 | SHA256 |
|---|---|
| `zero-browser_26.10.22_amd64.deb`（当前版本） | `0b3c541ce5d74a6f9fe9bc4155583d82e020afcc8ba967a79f8414d13eda4229` |
| `zero-browser_26.10.23_amd64.deb`（更新包，`ZERO_BUILD_VERSION=26.10.23`） | `80a4d799eba9e32b122a797f8d74aef4d4572b2cc380abbae5c77ae6fbfa38c1` |

运行方式：`systemd-run --user --scope -p MemoryMax=8G timeout 900`（无人值守守卫）；安装根与更新源全部位于 `/tmp/zeroweb-dc2-acceptance/`（运行后清理），通过 `ZERO_UPDATE_INSTALL_ROOT` 注入，未触系统安装与真实 profile。

| 腿 | 场景 | 结果 |
|---|---|---|
| leg1 | 正常更新：校验→暂存→翻转，`UPDATE_APPLIED 26.10.23 (previous 26.10.22)`，current/rollback 链接正确 | PASS |
| leg1b | 显式回滚：`UPDATE_ROLLBACK 26.10.22`，旧版本可启动 | PASS |
| leg2 | 损坏包（SHA256 不符）：`UPDATE_REJECTED HASH_MISMATCH`，旧版存活，staging 清理 | PASS |
| leg3 | 降级版本（26.10.21）：`UPDATE_REJECTED DOWNGRADE`，旧版存活 | PASS |
| leg4 | 版本标签与包内容不符（manifest 报 26.10.24 / deb 实为 26.10.23）：`UPDATE_REJECTED STAGING_CHECK_FAILED`，旧版存活 | PASS |
| leg5 | 更新执行期 SIGKILL：旧版存活，重跑成功应用 | PASS |
| leg6 | 解包期 SIGKILL（轮询 4 次后命中）：旧版存活，重跑成功应用 | PASS |

中断腿说明：leg5 轮询计数 0（WSL2 下首轮轮询前复制已入盘，kill 落点偏向更新执行早期）；leg6 确认落在解包期。两腿共同验证 SIGKILL 硬中断后活跃版本不受污染、重试收敛；staging 残留清理由单测 `stale_staging_is_cleared_on_next_apply` 补充覆盖。

## 4. 过程修复记录

| 问题 | 处置 |
|---|---|
| 版本序 match 臂反转（单测暴露） | 修正 `apply_update` 比较语义 |
| clippy `needless_borrows_for_generic_args`（`fs::remove_dir_all(&staging)`） | 去多余借用 |
| 端到端暴露 symlink 为绝对路径（安装根搬移后失效） | `repoint_link` 改 `releases/<版本>` 相对目标，验收脚本同步断言相对形式 |

## 5. 限制与挂账

- 更新后新版本生效需重启浏览器；headless 无优雅关停（G4，[M1 证据 §5](../2026-10-22-m1-inventory-baseline/README.md)），重启语义挂账 DC-4 会话恢复切片评估。
- 本渠道未签名（RFC D2）：SHA256 校验只防传输损坏与源内篡改，不防源被整体替换；签名验收待决 P1。
- 真机系统安装（`dpkg -i`）验收 NOT_RUN，属 M4；本轮为解包前缀隔离验收。
- macOS/Windows 更新管线未实现（RFC D1 定 Linux 首平台），M4 记审计。

## 6. 门禁

fmt PASS / 定向测试 PASS（8 update + 会话过滤 22）/ `make test`（test-guard 包裹）PASS / workspace clippy `-D warnings` PASS（修复 1 处 warning 后）/ 提交前 lei-pre-commit-guard PASS（记录见提交）。
