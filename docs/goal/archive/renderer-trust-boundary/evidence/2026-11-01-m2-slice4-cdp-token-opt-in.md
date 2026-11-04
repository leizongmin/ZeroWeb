# M2 切片 4：CDP opt-in token（`--remote-debugging-token`）+ 切片 4a 定性：`__zw*` 测试钩子族升级为设计切片

**日期**：2026-11-01
**HEAD（基线）**：`b5ef2f0e8`（切片 3 提交后）
**范围**：`apps/browser`（main.rs + headless）。

## 4：opt-in token（待决项的非破坏部分）

待决是「默认强制 token」（破坏既有自动化客户端，须用户授权）；本切片交付不破坏默认的 opt-in 面：

- `--remote-debugging-token=<t>` / `--remote-debugging-token <t>`（与 port 参数同款双形态）→ `run_headless` 经既有 `with_security(HeadlessSecurityConfig::new().with_token(t))` 装配。默认不设 = 行为零变化。
- 既有首消息认证路径复用（headless/mod.rs:469-487：无 token 消息拒答 `-32001 Authentication required`，带 `params.token` 消息正常派发）。
- stale 属性清理：`with_security`/`new`/`with_token` 的 `#[allow(dead_code)]` 因获得生产调用点移除；`with_origin` 仍仅测试使用，保留。

**常驻回归**：`cdp_token_opt_in_rejects_first_message_without_token`（tungstenite 真实 WS 客户端：无 token → `-32001`；带 token → id 回带非认证错误）。

## 4a：权限决策在 renderer——定性升级（未修）

核验结论（原切片 4 候选「__zwSetPermission 自授权」）：

- fullscreen 激活门（part04.js:4966-4970）确实消费 `__zwPermissionState('fullscreen')==='granted'` 免激活，页面可 `__zwSetPermission('fullscreen','granted')` 自授。
- 但 **`__zwUserActivate`（part06.js:446-452）同样页面无条件可见**，直接置瞬态激活——fullscreen 门存在不经权限注册表的等价绕过。问题面是整个 `__zw*` 测试钩子族装配策略（testdriver stub 在页面上下文运行 → 特权钩子必须页面可见：`__zwSetPermission`/`__zwUserActivate`/`__zwClipboardStoreWrite`/`__zwDeleteAllCookies` 等）。
- 修复需测试模式判定（renderer 级 automation flag 或 runner 装配路径）+ 三处 shim 文件接线 + GUI 真实输入激活态来源核验，且 part02/part04/part06 与 html-semantics 流当前工作文件正面重叠（run-rules §9 碰撞面）。
- **处置**：记 master 缺口表为设计切片（含产品取舍：普通浏览页不装配测试钩子 vs 保持现状+文档），不与本轮硬解。

## 验证命令与结果（test-guard 包裹）

- `cargo test -p zero-browser --bin zero-browser cdp_token_opt_in` → PASS
- 全量 `cargo test -p zero-browser --bin zero-browser` → 720P/2F（`paste_and_go_blurs_and_clears_preedit` 切片 1/2/3 已证既有确定性红；`default_actions_work_without_javascript` 切片 2 干净树对照轮已现的负载 flaky——无本切片独有成员）
- `cargo clippy -p zero-browser --bins`：本切片文件零警告

## 清理

无残留进程/目录。
