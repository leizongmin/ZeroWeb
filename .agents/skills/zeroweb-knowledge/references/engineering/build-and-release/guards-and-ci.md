# CI、安全、内存、超时与磁盘门禁

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

定位不同守卫的限制对象和 CI 实际覆盖，避免把“命令受保护”当作所有阶段都有同样约束。未运行守卫、构建或测试。

### Decision Entry
构建分支从 Makefile 和 CI matrix 进入，运行限制在 test-guard 与 fuzz-repro-guard；更完整的环境及 feature 条件见 runtime/conditions。

## Branching Table
| 维度 | 条件 | 实际行为 |
|---|---|---|
| cargo test 相位 | `--compile-first` | 先无运行阈值编译，逐 artifact 重新启动守卫 |
| 直接命令 | 无 `--compile-first` | 对命令和后代每 250ms 采 RSS，独立墙钟截止 |
| 默认与 Makefile | 裸守卫 / make test | 6GB/16GB/1800s / 显式 4GB/8GB/900s |
| 病态复现 | fuzz-repro-guard | 地址空间默认 8192MiB，加 30s KILL timeout |
| CI macOS Intel | `run_tests: false` | 编译/检查保留，测试执行跳过 |
| CI Windows ARM64 | matrix continue-on-error | 已知 FreeType NEON 问题允许不阻断 |
| GPU capability | Makefile adapter probe | 探测成功才跑 GPU 专项，不成功明确跳过 |
| CI 触发 | 手动 / OWNER approved review | 执行 CI；普通 push 不是此 workflow 触发器 |

表格证据：`scripts/test-guard.rs`、`scripts/fuzz-repro-guard.sh`、`Makefile`、`.github/workflows/ci.yml`。

## Affected Scope
| 路径 | 影响 |
|---|---|
| `Makefile` | 本地测试相位和阈值入口 |
| `scripts/test-guard.rs` | RSS/超时/子进程树回收 |
| `scripts/fuzz-repro-guard.sh` | 病态复现地址空间限制 |
| `scripts/target-disk-guard.sh` | 构建产物清理 |
| `.github/workflows/ci.yml` | 平台与 feature 测试矩阵 |
| `.config/nextest.toml` | nextest slow-timeout |
| `.github/workflows/security.yml` | 依赖漏洞、来源和 bans |
| `.github/workflows/fuzz.yml` | nightly 随机测试及 crash 资产 |

## Gotchas
- `--compile-first` 的墙钟限制逐 artifact 重置，编译阶段不受这个运行阈值约束；无人值守仍需外层总时限与编译内存保护。 (`scripts/test-guard.rs`, `Makefile`)
- RSS 获取依赖 `ps`，采样失败时当轮跳过内存判定；不能把守卫存在视为硬内核内存限制，Linux systemd scope 属另一层保护。 (`scripts/test-guard.rs`, `Makefile`)
- `ZW_GUARD_RUNNER_PREFIX` 仅 compile-first 生效且按空白分词，不支持 shell 引号语义；它只改变 artifact 的启动前缀。 (`scripts/test-guard.rs`)
- disk guard 默认超过 50GB 时删增量缓存、仍超则清空 target，并每次删仓库根 core；du 失败放行，既不是 OOM 守卫，也不是无副作用 doctor。 (`scripts/target-disk-guard.sh`)
- fuzz-repro 的 ulimit 是虚拟地址空间而非 RSS，先编译再设限以免链接器被杀；脚本依赖 `timeout` 与 `ulimit -v`，跨平台使用须先确认实际可用。 (`scripts/fuzz-repro-guard.sh`)
- Unix kill 的负进程组 ID 必须放在 `--` 之后，当前 helper 已固定 `["-TERM","--","-<pgid>"]`；缺分隔符会把组目标误当选项，后代兜底不能替代正确组终止。 (`scripts/test-guard.rs`, git:01a9c2634add86041157d1db5b8e1997e0a67cbe)
- CI reftest 当前用 `--json`，旧 `--format json` 被手写 CLI 当 filter 后匹配零案并空转绿；判断历史 CI 成绩必须区分此修复前后的真实执行集合。 (`.github/workflows/ci.yml`, `tests/wpt-runner/src/main.rs`, git:c3456421dba60452f84654b744b3b0e926d30a00)

## Architecture
- test-guard Unix 子进程独立 process group，触发后先组 TERM、再按 ps 后代 KILL、短等后复扫，退出 124；正常退出透传状态，启动失败为 127。 (`scripts/test-guard.rs`)
- 本地 Makefile 已改回 cargo test 以共享字体缓存，CI 当前仍使用 nextest；nextest 配置存在不证明本地 make test 按每测试独立进程运行。 (`Makefile`, `.config/nextest.toml`, `.github/workflows/ci.yml`)
- security workflow 执行 audit 与 deny advisories/bans/sources，不执行 deny licenses；依赖安全检查不能代替完整许可证交付核验。 (`.github/workflows/security.yml`)
- fuzz workflow 的 nightly parser/tokenizer 各有 600s 总时、5s 输入时限和不同 max_len，crash artifacts 只保留 14 天；事故复现流程应另沿 fuzz-repro guard，不直接照搬随机 fuzz 命令。 (`.github/workflows/fuzz.yml`, `scripts/fuzz-repro-guard.sh`)

## Branching Behavior
- CI 的发布构建只在手动触发且 `run_release_builds` 为 true 时执行；review 触发的绿灯不包含 release 打包完成。 (`.github/workflows/ci.yml`)
