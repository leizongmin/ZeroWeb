# M3 切片 3：image-decoder 进程沙箱（③A-image-decoder + ④C fail-closed）

**日期**：2026-11-01
**HEAD（起点）**：`fe125f5bc`（切片 8 交付态，本轮 `git pull --rebase` 后 rebase 于 `0e6fda61a` 之上推送为 `2bf01e2ba`）
**性质**：修复切片（决策包 ③A 第一刀 + ④C 该进程 fail-closed 语义，拍板记录见 [决策包](../2026-11-01-decision-package-1234.md)）。
**代码提交**：`21a478717`（feat(image-decoder)）；附带 `727506d2a`（fix(browser) 切片 8 残留 workspace clippy 红，见「发现与修正」）。

## 背景与范围

缺口表 helper 行：image-decoder 此前无任何沙箱钩子（role 入口直接 stdio 循环）。决策包 ③A 拍板「image-decoder 先行」，④C 拍板「image-decoder 新档案即 fail-closed」。IO 面已查实（M3 切片 2 审计 + 本轮复读 `lib.rs run_role`）：纯 stdio 帧循环 → 内存 decode → rgba，无 fs/socket 访问——档案取最紧形态：

- **seccomp 黑名单**：网络 socket 全族（accept/bind/connect/get\*opt/listen/recv\*/send\*/setsockopt/socket/socketpair）+ exec/fork（execve/execveat/fork/vfork）+ ptrace。与 compositor 基础档案同构，但**无 AF_UNIX 域门控**（解码器零 socket 需求，stdio 为 exec 期继承 fd，getsockopt 一并阻断）。
- **landlock 空 allowed-set**：ruleset handled 四访问位（EXECUTE|WRITE_FILE|READ_FILE|READ_DIR）+ **零规则直接 restrict**——生效后全文件系统拒绝；已打开 fd（stdin/stdout pipe）与 exec 期已映射库不受影响，为空 allowed-set 成立前提（启动后无 fs 依赖已查实）。
- **fail-closed（④C）**：任一安装失败（含老内核 landlock ENOSYS、受限容器 EPERM）`main` exit(1) 拒绝运行——隔离模式下 renderer 侧走解码错误面（`image_decoder.rs` IF-006 订正语义：SPAWN/进程死无内联回落），降级显式可见。操作员降级出口：`ZW_IMAGE_DECODER_LANDLOCK=0`（seccomp 仍生效）。
- 非 Linux 平台：编译门控跳过 + 显式 INFO 日志（与 compositor 同口径；macOS 若强套 fail-closed 会使解码永不可用，不合拍板意图）。

## 实现要点

- 新增 `apps/image-decoder/src/sandbox.rs`（408 行）：三 env 门控（`ZW_IMAGE_DECODER_SANDBOX`/`_SECCOMP`/`_LANDLOCK`，`zero_runtime_config::enabled_by_default` 默认开、`0` 显式豁免）→ `sanitize_env`（剥 `LD_PRELOAD`/`LD_LIBRARY_PATH`/`DYLD_*`，与 renderer `apply_early_if_enabled` 同款面）→ seccomp → landlock；`apply_early_if_enabled() -> Result<(), String>`。
- `main.rs` 接线：`apply_early_if_enabled()` 在 stdio transport init 之前，`Err` → eprintln + exit(1)；tracing 级别 WARN→INFO（沙箱安装成功日志需可见，stderr 面不变）。
- **BPF UAPI 本地定义**（发现驱动，见下）：`sock_filter`/`sock_fprog`/`BPF_*` 在 libc 0.2.186 的 android target 缺失（仅 desktop linux_like 定义），而 `android-browser` 链接 `zero_image_decoder::run_role`（`target_os = "linux"` 在 android 同样成立）——本地 `repr(C)` 定义 + 常量（稳定内核 UAPI，seccomp_filter.rst），全 Linux 目标单一代码路径。
- landlock 自设 `PR_SET_NO_NEW_PRIVS`（`restrict_self` 契约前置）：解除对 seccomp 先行的顺序依赖，两开关独立可用（compositor 模板依赖 seccomp 先行满足，此处为解耦改进）。
- 时序契约：进程单线程，main 首段调用即满足「先于任何子线程」。

## 新增常驻回归（5 用例）

| 用例 | 面 | 位置 |
|---|---|---|
| `filter_blocks_network_and_exec` | 阻断清单含 execve/execveat/socket/connect | sandbox.rs 单元 |
| `sandbox_execution_denies_fs_exec_keeps_stdio` | **执行型**：隔离子进程真实安装双沙箱 → /etc/passwd 拒读、execve 拒、**/dev/shm 写也拒（空 allowed-set 判别式，与 compositor 白名单档案的关键差异）**、stdio（继承 fd）存活 | sandbox.rs cfg(test) 子探针 |
| `landlock_invalid_ruleset_surfaces_err` | 执行型：非法 handled 保留位 → create_ruleset EINVAL → Err 上抛（fail-closed 输入面） | 同上 |
| `real_decoder_binary_decodes_with_default_sandbox` | 真实 `zero-image-decoder` 二进制（`CARGO_BIN_EXE` + `child_process_args(ProcessRole::ImageDecoder)` + `PipeTransport`，生产同构）默认沙箱在环解码 1x1 PNG 往返（width=1/rgba 精确断言/error None） | tests/sandbox_real_process.rs |
| 既有 `ipc_decode_*` 2 用例 + `role_returns_decode_error_without_crashing` 原样绿 | 主路径无回归 | 既有 |

子探针测试带 ENOSYS/EPERM 能力预探测（内核不支持 landlock → 退出码 5 → 父进程按环境受限跳过，不冒充行为失败）+ `--exact` 全路径 + stdout "running 1 test" 兜底（M3 切片 1 教训全沿用）。

## 验证（test-guard 包裹，systemd scope 内存封顶）

- `cargo test -p zero-image-decoder`：**7P/0F**（2 既有 IPC + 1 新集成 + 4 lib 含 2 执行型子探针）。集成测试日志全程：seccomp 启用 → landlock 空 allowed-set 启用 → ready → 解码往返 → IPC peer disconnected（EOF 优雅退出）。
- 生产 proxy 链路：`cargo test -p zero-webview --lib image_decoder` **4P/0F**——`proxy_mode_decodes_png_correctly` 经 `ImageDecoderProxy` spawn 真实（已沙箱化）二进制往返成功，renderer→decoder 生产拓扑等价面实证。
- clippy：`-p zero-image-decoder --all-targets -D warnings` 清零；全 workspace `make guarded-clippy -D warnings` 通过（含附带修复）。
- fmt clean；`cargo check --target aarch64-linux-android` 通过（NDK 28.2 工具链，BPF 本地 UAPI 后 android 目标零错误）。

## 发现与修正

1. **测试首跑 hang（测试自身缺陷，非沙箱缺陷）**：集成测试 recv 后直接 `child.wait()`，而 transport 未 drop——子进程 run_role 以 EOF 为退出信号，stdin 管道被父进程持有则子进程永阻塞下一轮 recv，wait() 永不返回。修正：transport 作用域限定，recv 后 drop 关 stdin → 子进程 EOF 退出。沙箱行为全程正常（安装成功、解码成功、日志完整）。
2. **切片 8 残留 workspace clippy 红**：`tab_js_worker.rs` counter-plugin 装配嵌套 if 触发 `clippy::collapsible_if`——切片 8 当时 clippy 覆盖未含 zero-browser lib（v8 五 crate 表述不完整），本轮 workspace 同口径暴露。独立提交 `727506d2a` 修复（let-chains，行为等价），后续切片 clippy 覆盖表述以「workspace 全量」为准。
3. **android libc BPF 类型缺失**（上文实现要点）：首轮 `cargo check --target aarch64-linux-android` 未跑即提交前自查发现——若直接推 desktop-only 验证会在 Android 构建腿爆编译。本地 UAPI 定义后 android check 通过。

## 范围与残余

- **Android service bootstrap 未接线**：`android-browser/rust/src/lib.rs:385` 直调 `run_role`，未走 `apply_early_if_enabled`（编译在环、运行不装沙箱）。接线属 Android 宿主切片，记残余。
- renderer ③A（landlock 允许清单 + seccomp exec 过滤 + ④ renderer fail 语义）为下一刀，本切片不触及。
- seccomp 黑名单形态固有局限（未知攻击面 syscall 放行）与 compositor 档案一致；允许清单化升级为后续能力切片候选（决策包 ③ 记录）。
- M4 收口时补生产接收方复测：headless 链路解码错误面形态（安装失败 exit(1) 时 renderer 侧报错可见）+ 默认沙箱下图像加载回归（本轮已由 webview proxy 用例钉住库路径，生产浏览器链路随 M4 决策面增量复测带过）。
