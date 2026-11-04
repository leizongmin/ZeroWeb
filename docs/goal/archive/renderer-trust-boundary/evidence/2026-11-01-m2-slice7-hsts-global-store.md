# M2 切片 7：多进程 HSTS 升级 per-profile 全局存储（对齐真浏览器语义 + 无痕面收口）

**日期**：2026-11-01
**HEAD（基线）**：`a9ece4a79`（远端 main，s211 rebase 后）
**范围**：`crates/security/src/context.rs`、`apps/browser/src/fetch_proxy.rs`（附 `apps/compositor/src/seccomp_linux.rs` 一行 rustfmt 折行补齐，上轮 d4768c825 遗留门禁红，独立提交）。

## 缺口

切片 5 在册对齐项：多进程 HSTS 注册落 per-tab `SecurityContext`（各持独立 `HstsStore`），真浏览器为 per-profile 全局存储——RFC6797 动态注册应对后续所有到该 host 的连接生效，per-tab 粒度下 tab 间升级面劈叉（tab A 注册后 tab B 同 host 不升级）。

## 修复

- `zero-security`：`SecurityContext::hsts_store` 由裸 `HstsStore` 改为 `SharedHstsStore = Arc<Mutex<HstsStore>>`（默认仍独享，单进程路径语义不变）；全部访问收口到 `hsts()` 锁助手（锁毒化 `into_inner` 恢复——纯内存映射无中间态，恢复优于放大失败）；移除零调用点的 `hsts_store_mut`；新增 `hsts_store_handle` / `share_hsts_store` / `detach_hsts_store`（脱离共享回独立存储并重播预加载表）。
- `TabFetchProxy`：新增 `global_hsts` 句柄；`ensure_tab` 普通 tab 接入全局（首个普通 tab 落座全局面，后续以句柄接入，动态注册跨 tab 生效）；无痕 tab 维持独立存储（不读全局、注册不外泄）；`set_tab_private(true)` 即 `detach_hsts_store` 摘离。全部安全决策路径（on_navigate / fetch 入口）先经 `ensure_tab`，无旁路。
- 无痕回灌收口（本切片实现中补的第二个缺口）：首个普通 tab 落座用全新 `SecurityContext`，不用 `or_default()` 既有上下文——否则「无痕会话内注册→退出无痕→恰为首普通 tab」会把无痕注册提升为全局（隐私泄漏；真浏览器退出无痕即丢弃会话状态）。

## 常驻回归（沿用切片 5 两测试，语义升级 + 反例双证）

- `hsts_document_response_registration_propagates_across_normal_tabs`（取代 per-tab 隔离语义）：普通 tab 动态注册跨 tab 生效、includeSubDomains 覆盖子域；无痕 tab 不读全局、注册不外泄；转无痕即摘离（后续注册不落全局）；**退出无痕不回灌**（无痕注册不得保留/升为全局，落座的全局存储正常承接后续普通 tab）；非 HTTPS/无头/坏头不注册。反例证据：回退 ensure_tab 全新落座为 `or_default` 捕获，新臂在「退出无痕后无痕会话注册不得保留」断言 FAILED，恢复后 PASS。
- `drain_document_response_does_not_register_hsts_over_http`：原样通过（https-only 门端到端）。

## 性能核查（guideline #23 口径）

`check_resource_url` 为资源加载路径热函数，HSTS 访问新增一次 `Mutex` 锁。现有 `security_bench` 不含 `check_resource_url`/HSTS 条目（bench-gate 无该面比较对象）；实际开销为每资源决策一次无竞争锁（原子对，~ns 级），且 `TabFetchProxy` 本身 `&mut self` 串行化全部 SecurityContext 访问，锁恒无竞争；同函数内 URL 解析成本远高于此。不引入新基准。

## 验证命令与结果（test-guard 包裹，DISPLAY=:52 自起 DRI3 headless 显示）

- `cargo test -p zero-security` → 404 passed / 0 failed
- `cargo test -p zero-browser --bin zero-browser hsts` → 2 passed / 0 failed
- `cargo test -p zero-webview mixed_content` → 5 passed / 0 failed（单进程路径不受影响）
- 全量 `cargo test -p zero-browser --bin zero-browser -- --test-threads=1`（peer bins 先 `cargo build` 刷新）→ 730P/1F：唯一 F = `default_actions_work_without_javascript`，跨流观察见下
- `cargo fmt --all -- --check` → clean；`make guarded-clippy`（workspace all-targets -D warnings）→ 零警告

## 跨流观察（非本 goal 面，记档不追）

`default_actions_work_without_javascript` 今日 :52 下 5/5 确定性失败（期望提交查询 `name=after`，实得 `name=` 空；输入型文本后 reset 再输不再生效的形态）。归因：clean main（WIP stash 对照）同败 2/2——非本切片；s211 `d0a7598a7` shim 双臂核验（part03/04 回退 e73eb4cbd 重建 renderer，`grep -a -c _zwButtonEffType` 5→0 证二进制内容切换）仍败——非 s211。切片 2/4 曾归因「负载 flaky」、切片 5 记失败集稳定——环境相关性（:20 xfce 偶现 vs :52 kwin/Xwayland 恒现）或更早回归；:20 属用户实时桌面不可复测，精确 commit 归因超出本 goal 面（GUI 输入/表单 reset 语义）。engine/输入流后续全量跑带 display 时按此证据链接管。

## 清理

无残留进程/目录；`/tmp/fp-fixed.rs` 反例临时副本已删；:52 显示设施（kwin --virtual + Xwayland，自起 PID 记 `/tmp/zw-wayland-rt/`）按 M4 补验先例保留供后续验收；:20 用户桌面未触碰。
