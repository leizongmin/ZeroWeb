# Layer 2：ZW_RENDERER_LANDLOCK 默认翻开（①-a）交付 + DC-4 转 DONE

**日期**：2026-11-04
**拍板依据**：①-a（2026-11-03 用户拍板，[决策包](../2026-11-02-decision-package-landlock-default-and-file-scope.md)：接受收窄翻默认，收窄即 Chrome parity）

## 变更

- `apps/renderer/src/sandbox.rs`：`renderer_landlock_enabled()` 由 `enabled_when_true`（默认关 opt-in）翻为 `enabled_by_default`（默认开 kill-switch，`ZW_RENDERER_LANDLOCK=0` 显式豁免）；模块注释与函数文档同步（默认关的历史原因、①-a 收窄口径、双腿实证引用）；`sandbox_env_gates` 断言翻为新语义；`seccomp_default_face_blocks_fork_keeps_spawn` 子进程 env 由 `env_remove` 改显式 `ZW_RENDERER_LANDLOCK=0`（默认已开，不显式关会连带拒绝该探针的 fs 读检查；landlock 面由完整栈用例自钉）。
- `crates/runtime-config/src/lib.rs`：注册表 `ZW_RENDERER_LANDLOCK` 行 default `disabled`→`enabled` + 描述补 kill-switch。
- `docs/runtime-environment.md`：同行操作文档同步（`0`/`false` disables / enabled）。
- `apps/browser/tests/cdp_file_url_navigation.rs`：两腿重排——默认腿（`file_url_page_loads_headless`，无 env）即 landlock 默认开面（file:// 主文档 + 静态模块链 + 样式子资源全链可加载，产品流保全的常驻回归）；原显式 `=1` 腿改 `=0` kill-switch 腿（`file_url_page_loads_with_renderer_landlock_disabled`，操作员豁免面）；模块注释更新（①-a 已交付、动态 import 收窄记 Chrome parity 不钉）。

## 验证（全部 test-guard 包裹，2026-11-04 本机 Linux）

- `cargo test -p zero-renderer --lib sandbox::`：4P/0F（env gates 新默认语义 + seccomp 默认面探针 + landlock 允许清单完整栈真实 restrict 探针）。
- `cargo test -p zero-renderer --lib` 全量：371P/0F。
- `cargo test -p zero-browser --test cdp_file_url_navigation -- --test-threads=1`：2P/0F（默认腿 + kill-switch 腿，真实 browser+renderer 进程链）。
- 邻近回归：`cdp_file_scope_policy` + `cdp_navigate_evaluate` 全绿（Layer 1 范围策略与 CDP 链在 landlock 默认开下不回归）。
- `cargo test -p zero-runtime-config`：1P/0F（注册表唯一性）。
- `cargo fmt --all -- --check` clean；`make guarded-clippy`（workspace `-D warnings`）零警告。

## 默认面一次性探针（/tmp 产物不落仓，跑完即删）

真实进程双腿判别式（同 2026-11-02 预执行核查配方，动态 import 为唯一 renderer 本地直读臂）：

- 默认环境（无任何 env）：file:// 页静态模块链 `__modOk=dep-42` 通 + 模块体动态 import `__dynMod=REJECTED`（landlock 允许清单不含任意路径读，renderer 本地 `open` 被拒——收窄在生产进程链生效）。
- `ZW_RENDERER_LANDLOCK=0`：同页动态 import 恢复 `dyn-ok`（操作员豁免面有效）。
- 探针附加发现（记实现事实非缺陷）：renderer 子进程 stderr 不汇入 browser stderr，landlock 安装日志面由 `landlock_allowlist_real_restrict` 隔离子进程真实 restrict 用例钉住（该用例本轮随全量绿）。

## DC-4 判定：DONE

- landlock 默认值翻转交付（本证据）→ DC-4 最后一个可执行余项收口。
- 真机验收（macOS 真机 / android 编译腿 + service bootstrap 接线）按 2026-11-03 用户拍板口径 B 降级接受为 DC-4 终态：非 Linux 平台证据按「Linux-only 编译门控 + 显式降级日志」平台契约接受（代码级对照见 [M4 前置](2026-11-01-m4-preflight-platform-policy-dc-audit.md)），真机实证作为接受残余、可独立立项补。
- 接受残余清单：file:// 页模块体动态 import 默认拒（Chrome parity，`ZW_RENDERER_LANDLOCK=0` 操作员豁免；仓库内使用面 grep 零命中）；`__zwModalGateBypass` 接受残余（切片 8）；DevTools frontend page-direct fail-closed（决策 ⑥B 文档化接受）。
