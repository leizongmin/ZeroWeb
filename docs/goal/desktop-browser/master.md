# 桌面浏览器应用产品化 — 运行时控制面板（master.md）

**入口文档**: [../desktop-browser.md](../desktop-browser.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-12（立项）

---

## 当前状态

**专项定位**：browser-shell 数据模型 → 真窗口日常可用。Linux 真窗口端到端演示流为
第一验收平台，macOS/Windows 以 CI 编译 + 冒烟验收（差异记账）。

**与兄弟 goal 的边界**：
- android-browser — `apps/browser` 共享活跃并行流，碰前 git log 核对；Kotlin chrome
  产品面独立不共享 UI
- devtools — 其 M3（GUI 模式 CDP server 接线）与本目标同改 `apps/browser`，碰头排序
- rendering-compat — crate 零重叠

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | 真窗口端到端演示流（Linux）+ GPU 合成显示验收 | ⏳ M1 |
| P2 | 标签族/地址栏/导航控制交互面 | ⏳ M2 |
| P3 | 下载管理器交互面 | ⏳ M3 |
| P4 | engine 文本搜索 API（Ctrl+F 依赖，最小面评估） | ⏳ M3 |
| P5 | 缩放 viewport 联动（④ 桥遗产消费）+ 右键菜单 | ⏳ M3 |
| P6 | 收藏/历史/设置交互面 | ⏳ M4 |

## 已完成切片

（立项轮，暂无）

## 下一步计划

1. **M1**：winit 真窗口主链路演示流脚本化（启动/加载 URL/渲染/输入）→ GPU 合成
   显示验收（compositor dma-buf 链路）→ macOS/Windows CI 冒烟确认
2. **M2-M4**：按入口文档里程碑逐功能演示流

**待用户决策清单**：
- （暂无）

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — 真窗口主链路验收 | ⏳ |
| M2 — 导航与标签 | ⏳ |
| M3 — 内容工具 | ⏳ |
| M4 — 数据面 | ⏳ |
| M5 — 收口 | ⏳ |

## 验证基线

- 测试基线：立项时点全绿（`make test` 19,170P/0F，2026-09-12 口径；经 test-guard）
- 现状：browser-shell 数据模型落地（architecture.md 口径）；真窗口产品验收缺失
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过；
  演示流须可脚本重放才计入 evidence「绿」

**碰撞管理**：碰 `apps/browser` / `crates/browser-shell` 前先
`git log --since="14 days ago" -- apps/browser/ crates/browser-shell/` 核对
android-browser 活跃面；与 devtools goal M3 碰头排序。
