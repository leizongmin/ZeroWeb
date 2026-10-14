# 桌面浏览器交互

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖真实 ZeroBrowser 窗口、chrome UI、标签页导航及页面输入。入口 `main.rs` 建 HostRuntime，`BrowserApp` 同时协调 Shell 业务状态、页面运行时和本地呈现；不要把 headless CDP 的页面当成 GUI 标签页。

### User Task Navigation
- 窗口/DPI/失焦或菜单绘制异常：`main.rs` → `app_platform.rs/app_render.rs` → [窗口与 UI](chrome-and-window.md)。
- 地址栏/标签切换/页面点击/IME/复制：`app_input*.rs` → `tab_manager.rs` → [导航与输入](navigation-and-input.md)。
- CPU/GPU/跨进程页面帧：沿 [图形领域](../../graphics/index.md) 与 [运行时领域](../../runtime/index.md) 深入；真实窗口验收查 [验证地图](../../../../zeroweb-verify/features/index.md)，本轮未实跑。

## Gotchas
- 两个 TabManager 分工不同：browser-shell 管业务导航账本，apps/browser 管页面后端、快照和异步回执；修改标签切换需同时保持活动ID、地址栏、CSS视口及页面输入目标一致。 (`apps/browser/src/app.rs`, `apps/browser/src/tab_manager.rs`, `crates/browser-shell/src/tab.rs`)
- 正式桌面选择多进程后端，而测试默认进程内worker；测试helper的近似idle、页面求值或CPU截图不等于发布renderer与真实GPU窗口链路验证。 (`apps/browser/src/tab_manager.rs`, `apps/browser/Cargo.toml`)

## Architecture
- chrome UI 的物理尺寸和页面CSS视口分开：窗口DPI影响全窗，页面zoom只作用页面；导航、命中、候选窗和页面帧需核对各自换算，不共用一个含糊的scale。 (`apps/browser/src/app.rs`, `apps/browser/src/app_input.rs`, `apps/browser/src/app_platform.rs`)

## Child Knowledge Nodes
- [窗口、浏览器 UI 与平台行为](chrome-and-window.md) — 装饰、DPI、失焦、呈现选择与菜单几何。
- [标签导航与页面输入](navigation-and-input.md) — URL启发式、回执默认动作、滚动、IME、复制与查找边界。
