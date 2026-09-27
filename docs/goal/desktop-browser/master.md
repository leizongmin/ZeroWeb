# 桌面浏览器应用产品化 — 运行时控制面板（master.md）

**入口文档**: [../desktop-browser.md](../desktop-browser.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-27（M3-S2 落地：右键菜单 + 缩放联动演示流全绿 + 剪贴板 X11
失主缺陷修复；M1/M2 Done 维持）

---

## 当前状态

**专项定位**：browser-shell 数据模型 → 真窗口日常可用。Linux 真窗口端到端演示流为
第一验收平台，macOS/Windows 以 CI 编译 + 冒烟验收（差异记账）。

**与兄弟 goal 的边界**：
- android-browser — `apps/browser` 共享活跃并行流，碰前 git log 核对；Kotlin chrome
  产品面独立不共享 UI
- devtools — 其 M3（GUI 模式 CDP server 接线）**已 Done 转守成态**（2026-09 实测，
  `apps/browser` 近 14 天改动均属该 goal 收官切片）；碰头排序压力解除
- rendering-compat — crate 零重叠；本流发现的正文空格塌缩缺陷已按 §9 记档移交
  （见 evidence §5.1，归其 R4096-N font-stack 统一专项族）

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | 真窗口端到端演示流（Linux）+ GPU 合成显示验收 | ✅ M1 首切片（三腿全绿，evidence/M1-real-window-main-path.md） |
| P1' | macOS/Windows CI 启动冒烟（`--headless` CDP boot 探活步接进 ci.yml 矩阵） | ✅ Done（2026-09-27 CI run [36311188270](https://github.com/leizongmin/ZeroWeb/actions/runs/36311188270) 三平台 job 绿，launch smoke 两步全 ✓，DC-1 三条全勾） |
| P2 | 标签族/地址栏/导航控制交互面 | ✅ Done（M2-S1 多标签+导航控制 + M2-S2 地址栏键入/补全/加载指示，evidence/M2-{tabs-navigation,addressbar-autocomplete-loading}.md；主页按钮项并入 M4 设置流） |
| P3 | 下载管理器交互面 | 🔄 M3-S1：attachment 拦截 → 落盘 → 记账 → 面板/下载页 → Show in folder 全链绿（evidence/M3-download-manager.md）；余分块进度采样/条目级动作挂账 |
| P4 | engine 文本搜索 API（Ctrl+F 依赖，最小面评估） | ⏳ M3 |
| P5 | 缩放 viewport 联动（④ 桥遗产消费）+ 右键菜单 | ✅ M3-S2（Ctrl+± 缩放状态 + 页面区 reflow 像素断言 + 复位；右键菜单 Page/Selection 分发 + 检查元素入口 + 剪贴板读回，evidence/M3-context-menu-zoom.md） |
| P6 | 收藏/历史/设置交互面 | ⏳ M4 |

## 已完成切片

- **M3-S2（2026-09-27）右键上下文菜单 + 缩放联动演示流**：
  - 资产：`apps/browser/src/menu_zoom_smoke.rs`（--menu-zoom-smoke-base/-dir CLI）+
    smoke 只读面新增 context_menu/clipboard/zoom 访问器 + m3 脚本第二腿
    menu-zoom-flow + `examples/m3-downloads/menu.html`
  - 流程（真实输入路径）：右键弹 Page 菜单（条目断言）→ reload 行点击（epoch 前进
    + URL 不变）→ inspect 行点击（zero://inspect 新标签 + Ctrl+W 收尾）→ 拖拽选中
    → Selection 菜单 copy（剪贴板读回 25 字节）→ Ctrl+＋（1.0→1.1 + 页面区 reflow
    像素签名）→ Ctrl+0 复位
  - **缺陷修复**：clipboard.rs 每次读写新建 arboard::Clipboard 即弃——X11 selection
    所有权随实例存活，写完即失主（复制内容谁都读不回）；改进程内 OnceLock 保活
    单实例。菜单行点击几何与渲染/命中测试同源（分隔行紧凑高度累积），均匀行高
    公式点 inspect 行会落空且无报错——断言必须锚定动作副作用（方法论入 evidence）
  - 结果：menu-zoom-flow 全绿 + 复跑通过（与 download-flow 同轮双 PASS）；
    [evidence/M3-context-menu-zoom.md](evidence/M3-context-menu-zoom.md)
- **M3-S1（2026-09-27）下载管理器真实链路 + 演示流**：
  - 基线事实：DownloadManager 此前为纯状态模型（无落盘/无触发/无打开目录动作）。
    本切片补齐最小真实链路（全部 apps/browser，零 net/engine/renderer 改动——§9
    边界内自决）：attachment 响应拦截（Content-Disposition 解析 + filename 清洗
    信任边界）→ TabManager 转交（沿用 pending_loaded 模式）→ 落盘 + 管理器三态
    记账 + 面板自动弹出；面板扩展最近完成项 + Show in folder 按钮（命中矩形与
    渲染同源）；xdg-open/open/explorer 随平台打开下载目录
  - 演示流：`apps/browser/src/download_smoke.rs` + `scripts/desktop-browser-m3-smoke.sh`
    + `examples/m3-downloads/{page.html,file.zip}`；下载落盘隔离到演示流目录
    （结束恢复用户设置空值）；磁盘文件与 fixture 逐字节 cmp
  - 结果：download-flow 全绿 + 复跑通过；DC-3 第一条收口，映射与余账（分块进度
    采样/条目级动作）见 [evidence/M3-download-manager.md](evidence/M3-download-manager.md)
- **M2-S2（2026-09-27）地址栏真实窗口演示流**：
  - 资产：`apps/browser/src/addressbar_smoke.rs`（`--addressbar-smoke-base/-dir` CLI）
    + smoke 只读面新增 autocomplete 建议列表/高亮位访问器 + `examples/m2-tabs/slow.html`
    （本地服务延迟 1.5s）+ m2 脚本第二腿 addressbar-flow
  - 流程（真实输入路径）：Ctrl+L 聚焦全选 → 逐字符键入完整 URL → Enter 导航 →
    键入 `two` → 补全弹出（搜索建议 + 历史项，ArrowDown 键盘选中）→ Enter 按建议
    导航 → 延迟页 loading 窗口采样（Poll tick 主动渲染抓帧：tab spinner/停止钮/
    地址栏 spinner 实证）
  - 方法论记档：页面加载期事件循环可整段空闲（16ms Poll tick 是唯一保证推进点，
    RedrawRequested 链会断）——mid-load 采样挂 Poll tick（`sample_mid_load`），
    同为 M2-S1 帧自驱动的根因补注
  - 结果：addressbar-flow 全绿 + 复跑通过（与 tab-flow 同轮双 PASS）；DC-2 收口见
    [evidence/M2-addressbar-autocomplete-loading.md](evidence/M2-addressbar-autocomplete-loading.md)
- **M2-S1（2026-09-27）标签族/导航控制真实窗口演示流**：
  - 资产：`apps/browser/src/tab_smoke.rs`（状态机，`--tab-smoke-url-one/-two/-dir`
    CLI，与既有 smoke 模式互斥）+ `apps/browser/src/app_smoke_state.rs`（release 可用
    的 smoke_* 状态只读面）+ `scripts/desktop-browser-m2-smoke.sh`（可重放入口）+
    `examples/m2-tabs/{one,two}.html`
  - 流程（全部真实输入路径）：Ctrl+T 新标签 → 第二标签导航 → Ctrl+Tab 切回 →
    Alt+Left/Right 后退前进（真实历史条目）→ F5 刷新 epoch 断言 → 标签拖拽重排
    （顺序 + 标签条像素签名可视变化）→ Ctrl+W 关闭
  - **缺陷修复**：`cycle_active_tab` 补 `update_address_bar_from_active_tab` +
    `on_active_tab_changed`（Ctrl+Tab 后地址栏跟随，与点标签/Ctrl+1~9 路径对齐）；
    单测 `ctrl_tab_updates_address_bar_to_active_tab` 红→绿锚定
  - 语义观察记档：启动欢迎页不进历史（不经 shell.navigate），后退从首个文档导航
    起算——与 Chrome NTP 可回退不同，M4 复核
  - 结果：全流程绿 + 复跑通过；DC-2 映射见
    [evidence/M2-tabs-navigation.md](evidence/M2-tabs-navigation.md)
- **M1-S2（2026-09-27）三平台 CI 启动冒烟接线**：
  - `scripts/browser-launch-smoke.sh`：headless CDP boot 探活（不开窗、runner 无需
    display；`/json/version` 断言 `Browser: ZeroWeb/` + `webSocketDebuggerUrl`，
    `/json` 断言 page target 枚举；60s 轮询；trap 杀进程收尾）
  - `.github/workflows/ci.yml`：`matrix.launch_smoke` 旗标 + 两步（构建 zero-browser
    本体 bin——测试链 lib 不产 bin；跑探活脚本）接入 build-and-test 的
    linux-x86_64 / macos-aarch64 / windows-x86_64 三条目；renderer/compositor/
    image-decoder 子进程 bin 由矩阵既有步骤产出到同目录（current_exe 上溯解析）
  - 取证闭环：本地 Linux release 目录全链路 PASS；CI run
    [36311188270](https://github.com/leizongmin/ZeroWeb/actions/runs/36311188270)
    （commit 1e5d0477c）三平台 job 全绿 + launch smoke 两步全 ✓
    （linux 3m45s / macos-arm64 4m29s / windows 7m9s）——DC-1 三条全勾
  - **跨流记账（CI benchmarks job 连日红）**：ci.yml 每日 workflow_dispatch 的
    benchmarks job「Run benchmarks and gate」连日 FAIL（2026-09-24 起连续观察，
    2026-09-26 run 36264551226 = 5 指标超预算：webview_load_html_with_css
    266,282 vs 预算 231,884ns、page/medium/layout_ms 164.82 vs 164.44ms、
    page/medium/first_paint_wall_ms 393.93 vs 376.99ms 等），本地六点门禁同期全绿
    （R4843-R4845 med_fp 带内）——CI 共享 runner 噪声域 vs 本地基线带宽的结构性
    张力，属 perf-gate 政策域（基线放宽须走 record-bench-baseline.sh --relax +
    justification，是否放宽待用户拍板）；与本流切片无关（零运行时代码变更，
    build-and-test 矩阵不含 benchmarks job），已飞书告知
- **M1-S1（2026-09-27）真窗口主链路三腿演示流**：
  - 资产：`scripts/desktop-browser-m1-smoke.sh`（可重放入口，无 display 自动
    xvfb-run；每腿 test-guard 包裹）+ `examples/m1-real-window/index.html`（本地
    http fixture；file:// 会短路输入交互腿故必须 http）
  - 腿 a：cpu renderer + compositor 事件链 + bitmap 采纳；腿 b：gpu renderer 真窗口
    surface 呈现；腿 c：`ZW_COMPOSITOR_SCROLL_TRANSFORM=0` GPU 导入链
    （compositor_dmabuf_adopted / gpu_direct）
  - 断言：gui_smoke 步骤 ×4 + 滚动/缩放像素签名 visual_change + compositor 全事件链
    + 零 fallback（Compositor disconnected / legacy_view_painted / fallback=true 均
    红线）
  - 结果：三腿全绿复跑确认；证据与 DC-1 映射见
    [evidence/M1-real-window-main-path.md](evidence/M1-real-window-main-path.md)

## 下一步计划

1. **M3 余项（P4 页面查找）**：Ctrl+F 查找栏 UI 与 find_state 已在（engine 侧搜索
   通道存疑）——先核现状：真窗口里 Ctrl+F 键入后高亮/计数是否端到端可用；可用则
   直接补查找演示流（P4 收口）；不可用则 engine 文本搜索 API 最小面评估（入口文档
   执行协议：最小面自主做，超范围记「待用户决策」）
2. **M4 数据面**：收藏（添加/删除/文件夹/收藏栏）/ 历史（记录/搜索/清除）/ 设置
   （默认搜索引擎/主页/隐私）演示流；主页按钮项在此收口（先配 home_url 为本地
   fixture 再按 Alt+Home）

**待用户决策清单**：
- 正文空格塌缩（product 可见）已移交 rendering-compat 流（其 R4096-N font-stack 统一
  专项为 user-gated 深结构批次）——本流仅记档，处置归该专项收口时定界
- CI benchmarks job 连日红（perf-gate 5 指标超预算，CI 共享 runner 噪声域）：是否对
  CI runner 环境单独建基线或放宽阈值（record-bench-baseline.sh --relax），须 perf-gate
  政策域拍板——本流不触碰

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — 真窗口主链路验收 | ✅ Done（DC-1 三条全勾：Linux 演示流 + GPU 合成显示记账 + 三平台 CI 启动冒烟绿证据 run 36311188270） |
| M2 — 导航与标签 | ✅ Done（S1 多标签+导航控制 + S2 地址栏键入/补全/加载指示全绿；主页按钮项并入 M4 设置流记账） |
| M3 — 内容工具 | 🔄 S1 下载管理器 + S2 右键菜单/缩放联动全绿；余页面查找（P4：先核 Ctrl+F 现状再定最小面/记决策） |
| M4 — 数据面 | ⏳ |
| M5 — 收口 | ⏳ |

## 验证基线

- 测试基线：`make test` 全绿（经 test-guard；2026-09-27 本轮复验，随提交记账刷新计数）
- 演示流门禁：`bash scripts/desktop-browser-m1-smoke.sh` 三腿全绿（M1 守成门）+
  `bash scripts/desktop-browser-m2-smoke.sh` 双腿全绿（M2 守成门）+
  `bash scripts/desktop-browser-m3-smoke.sh` 全绿（M3 守成门）——渲染/组合器/窗口/
  标签·地址栏·导航·下载链路任何触碰后必跑
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过；
  演示流须可脚本重放才计入 evidence「绿」

**碰撞管理**：碰 `apps/browser` / `crates/browser-shell` 前先
`git log --since="14 days ago" -- apps/browser/ crates/browser-shell/` 核对
android-browser 活跃面；与 devtools goal 碰头排序（当前其已 Done，压力解除）。
