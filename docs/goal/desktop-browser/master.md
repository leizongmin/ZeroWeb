# 桌面浏览器应用产品化 — 运行时控制面板（master.md）

**入口文档**: [../desktop-browser.md](../desktop-browser.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-27（M1 首切片落地：三腿真窗口演示流全绿 + GPU 合成显示记账）

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
| P1' | macOS/Windows CI 启动冒烟（`--headless` CDP boot 探活步接进 ci.yml 矩阵） | ⏳ M1 余项 |
| P2 | 标签族/地址栏/导航控制交互面 | ⏳ M2 |
| P3 | 下载管理器交互面 | ⏳ M3 |
| P4 | engine 文本搜索 API（Ctrl+F 依赖，最小面评估） | ⏳ M3 |
| P5 | 缩放 viewport 联动（④ 桥遗产消费）+ 右键菜单 | ⏳ M3（缩放的 Ctrl± 输入面已被 M1 腿消费，页面侧联动断言待 M3 收口） |
| P6 | 收藏/历史/设置交互面 | ⏳ M4 |

## 已完成切片

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

1. **M1 余项**：ci.yml 三平台矩阵加 `--headless` 启动冒烟步（macos/windows runner：
   启动 zero-browser headless CDP → /json/version 探活 → 退出码门禁），关闭 DC-1
   「三平台可编译启动」条目
2. **M2**：标签族（创建/关闭/切换/拖拽排序）+ 地址栏（自动补全/加载进度）+ 导航控制
   （前进/后退/刷新/主页）演示流——复用 M1 的 gui_smoke 状态机扩展步骤，或以
   parity_smoke 场景 JSON 形态扩交互面
3. **M3-M4**：按入口文档里程碑逐功能演示流（下载/查找/缩放联动/右键菜单/收藏/历史/设置）

**待用户决策清单**：
- 正文空格塌缩（product 可见）已移交 rendering-compat 流（其 R4096-N font-stack 统一
  专项为 user-gated 深结构批次）——本流仅记档，处置归该专项收口时定界

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — 真窗口主链路验收 | 🔄 Linux 演示流 + GPU 记账完成；余 macOS/Windows CI 启动冒烟 |
| M2 — 导航与标签 | ⏳ |
| M3 — 内容工具 | ⏳ |
| M4 — 数据面 | ⏳ |
| M5 — 收口 | ⏳ |

## 验证基线

- 测试基线：`make test` 全绿（经 test-guard；2026-09-27 本轮复验，随提交记账刷新计数）
- 演示流门禁：`bash scripts/desktop-browser-m1-smoke.sh` 三腿全绿（M1 守成门，
  渲染/组合器/窗口链路任何触碰后必跑）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过；
  演示流须可脚本重放才计入 evidence「绿」

**碰撞管理**：碰 `apps/browser` / `crates/browser-shell` 前先
`git log --since="14 days ago" -- apps/browser/ crates/browser-shell/` 核对
android-browser 活跃面；与 devtools goal 碰头排序（当前其已 Done，压力解除）。
