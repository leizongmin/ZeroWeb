# M3 切片 4：renderer 进程沙箱（③A-renderer + ④ fail-closed）

**日期**：2026-11-02
**HEAD（起点）**：`fc52efc70`（切片 3 交付态，本轮 rebase 于 `235f07cd1` 之上推送为 `a0bebfe52`）
**性质**：修复切片（决策包 ③A 第二刀 + ④ renderer fail 语义拍板落地，拍板记录见 [决策包](../2026-11-01-decision-package-1234.md)）。
**代码提交**：`a0bebfe52`（feat(renderer)，含 CString 越界读修复）。

## 背景与范围

缺口表 helper 行：renderer 此前为 RFC §六 P3 占位（仅 env 剥离 `LD_*`）。决策包 ③A-renderer 拍板原文：「landlock 允许清单：字体 ro、profile/缓存 rw、/dev/shm fd socket rw；seccomp exec 过滤不动网络」+「④ renderer fail 语义随 ③A-renderer 切片定」。IO 面以 M3 切片 2 审计为基础本轮复核，**两笔决策包外事实进入实现**：

1. **renderer 必须保留 `execve`**：renderer 是 image-decoder / media-host helper 的合法 spawn 方（`webview::image_decoder` / `webview::media_proxy` 经 std `Command` → posix_spawn → `clone(CLONE_VM|CLONE_VFORK)` + `execve`，不经 fork/vfork syscall）。故 seccomp 档案为「exec 收窄」而非「exec 阻断」：阻断 `execveat`（memfd + AT_EMPTY_PATH 无文件执行通路）、`fork`/`vfork`、`ptrace`；`execve` 保留、`clone` 保留（线程/posix_spawn）。执行型子探针钉住「fork 拒 + Command spawn 存活」判别式。
2. **file:// 主文档由 renderer 直读**（`zero_net::read_file_url`，client.rs:643；产品流：地址栏本地文件、gui_smoke/parity_smoke fixtures），与 landlock 允许清单（不含任意路径读）直接冲突——**landlock 默认关，opt-in `ZW_RENDERER_LANDLOCK=1`**，file:// 取舍记控制面待决（goal 文档「发现额外产品取舍记待决」授权此处理），拍板后翻默认。

④ fail 语义（本切片拍板）：**fail-closed**——任一安装失败 `Err` 上抛 → `run_desktop_role` 传播 → main exit(1)，浏览器侧呈现为 renderer 启动失败（页面错误可见），不降级裸奔。操作员出口：`ZW_RENDERER_SANDBOX=0`（全关）、`ZW_RENDERER_SECCOMP=0`、`ZW_RENDERER_LANDLOCK=0`。非 Linux 平台编译门控跳过 + 显式日志（与 compositor/image-decoder 同口径）。

## 实现要点

- `apps/renderer/src/sandbox.rs` 重写（627 行）：三 env 门控（`ZW_RENDERER_SANDBOX`/`_SECCOMP` 用 `enabled_by_default` 默认开、`ZW_RENDERER_LANDLOCK` 用 `enabled_when_true` 默认关）→ `sanitize_env` → seccomp → landlock；`apply_early_if_enabled() -> Result<(), String>`；`runtime.rs` `run_desktop_role` 首语句 `apply_early_if_enabled()?`（时序契约：先于一切子线程，切片 2 审计已钉此入口）。
- landlock 允许清单 8 组规则（每条对应一处在环 IO 面）：① 字体 ro（/usr/share/fonts、/usr/local/share/fonts、/etc/fonts；render-foundation 字体清单全在第一处）② HTTP 磁盘缓存 rw（`zero_net::default_cache_dir`，最近存在祖先授予保证建目录成功）③ /dev/shm rw（fd 通道 + 后续 media C2 帧路径）④ /etc 运行时必需 ro（localtime→JS Date 时区、resolv.conf/hosts/nsswitch.conf/gai.conf/host.conf/services→getaddrinfo）⑤ 动态库目录 ro+EXECUTE（dlopen/NSS/gconv）⑥ helper 同目录 EXECUTE+读（sibling 部署约定；EXECUTE 路径级收窄——/usr/bin 不在授予面）⑦ /proc/self/smaps_rollup ro（T13 S7 PSS 探针）⑧ 四个 trace env 路径条件授予（env 未设不授任何写面）。
- landlock `PR_SET_NO_NEW_PRIVS` 自设（与 seccomp 开关解耦，镜像 image-decoder）；optional 规则路径缺失静默跳过（发行版差异非关键面），`ZW_RENDERER_LANDLOCK_DEBUG` 门控 eprintln 定位安装失败。
- BPF UAPI 本地定义（android libc 缺类型，同 image-decoder 切片理由）；aarch64 所需 libc syscall 常量（SYS_landlock_* / SYS_seccomp / SYS_execveat / SYS_ptrace / O_PATH / PR_SET_NO_NEW_PRIVS）逐一核对 libc 0.2.186 android b64/aarch64 模块在册。

## 发现与修正

**OsStr 路径未 NUL 终止传 `libc::open`（本切片引入，调试轮定位修复）**：`open_path_beneath` 初版用 `path.as_os_str().as_bytes().as_ptr()` 直接传 `libc::open`——OsStr 字节不带 NUL 终止符，open 读越界、路径尾随堆上垃圾字节，表现为全部允许清单规则「随机」ENOENT/ENOTDIR（连确定存在的 /usr/share/fonts、/etc/hosts 也失败）。`ZW_RENDERER_LANDLOCK_DEBUG` 门控 eprintln 一轮定位（错误形态为路径查找类而非权限类 → 指针而非规则问题）；改经 `CString::new`（compositor `landlock_linux.rs` 既有同款），修复后全部规则干净安装、零 optional 失败。

## 新增常驻回归（4 用例，sandbox::tests）

| 用例 | 面 |
|---|---|
| `sandbox_env_gates` | 三开关语义（默认开×2 / opt-in×1） |
| `seccomp_blocklist_targets_exec_family` | 阻断清单含 execveat/ptrace；execve/clone/socket 不在列（helper spawn 保全 + 网络不动判别式） |
| `seccomp_default_face_blocks_fork_keeps_spawn` | 执行型子探针（默认生产面 = seccomp only）：raw `SYS_fork` 拒（glibc `fork()` 实走 SYS_clone，须原始 syscall 才测得到过滤器）、`Command /bin/true` 存活（posix_spawn 通路）、ptrace 拒、execveat EPERM、stdio 存活、fs 未受限 |
| `landlock_allowlist_real_restrict` | 执行型子探针（完整栈 = seccomp + landlock opt-in）：/dev/shm 写读 ok、/usr/share/fonts read_dir ok、/etc/passwd 拒（允许清单判别式）、/bin/true execve 拒（EXECUTE 收窄）、stdio 存活；ENOSYS/EPERM 能力预探测 exit 7 → 环境受限跳过 |

子探针 env 由父进程 `Command .env()` 钉死（并行用例改共享进程 env 会污染子进程开关态——本轮实测教训：初版子进程侧继承到关态假红）；`--exact` 全路径 + stdout "running 1 test" 兜底（M3 切片 1 教训沿用）。

## 验证（test-guard 包裹）

- `cargo test -p zero-renderer sandbox:: -- --test-threads=1`：**4P/0F**。
- `cargo test -p zero-renderer --lib` 全量：366P + 2F——`page_scripts::tests::mutation_arm_does_not_apply_across_document_reset_window` 与 `tick_per_task_tests::reset_clears_writes_landed_after_main_thread_clear_d1`，两用例均为计时窗断言，隔离单跑双绿；**clean HEAD stash 复跑同现**（364P/1F，tick_per_task 独现）→ 归因为并行满载下既有 flaky，与本切片无关（本切片仅 sandbox.rs + runtime.rs 一行，均不在其依赖图）。
- workspace `make guarded-clippy -D warnings` 清零（本切片 4 处 `collapsible_if` let-chains 化后过）；fmt clean。
- android：全量 `cargo check -p zero-renderer --target aarch64-linux-android` 本环境不可行（v8 registry crate 无 android bindings，android renderer 构建走 v8 源码编译路径，30-40min + 代理 + 六项环境）；quickjs 特征链替代 check 亦不可行（rquickjs-sys bindgen 需 NDK sysroot 配置，同为环境既有限制）。以 libc 常量逐一静态核对替代（见实现要点），android 编译腿残余风险记 M4。

## 范围与残余

- **Android service bootstrap 未接线**（image-decoder + renderer 两处，`android-browser/rust/src/lib.rs:385/365` 直调 run_role）——接线属 Android 宿主切片，记残余。
- **file:// 取舍待决**（控制面）：三选项——browser 侧预解析 file:// → LoadHtml（renderer 零文件读）/ 允许清单按 profile 显式扩权（用户指定目录）/ file:// 收窄为错误面。拍板后翻 `ZW_RENDERER_LANDLOCK` 默认。
- seccomp 黑名单形态固有局限（未知攻击面 syscall 放行）与 compositor/image-decoder 档案一致；网络面不动为拍板范围，fetch_proxy 出网收窄另立切片。
- BPF UAPI 本地定义现为三处复制（compositor/image-decoder/renderer），hoist 候选记 M4 清理。
- renderer 受限操作验收（DC-4 余项）+ 生产接收方复测随 M4 收口。
