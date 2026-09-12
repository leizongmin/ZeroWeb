# DevTools 调试面 — 运行时控制面板（master.md）

**入口文档**: [../devtools.md](../devtools.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-12（立项）

---

## 当前状态

**专项定位**：复用 Chrome DevTools frontend（pin bundle）经 CDP 附接 ZeroWeb，四面板
（Elements / Console / Network / Application-cookie）达到「逐面板演示流可判定可用」。
**启动门控**：cdp-protocol goal M3（DOM/CSS 域就位）解锁主线；门控期推进 M0 自主面。

**与兄弟 goal 的边界**：
- cdp-protocol — 上游协议基座：本 goal 只消费其 CDP 面；结构性域缺口回流上游，
  小缺口碰头协调
- android-browser — `apps/browser` 共享活跃并行流，碰前 git log 核对
- rendering-compat — crate 零重叠

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | devtools-frontend bundle 获取/版本 pin/许可核查（非 MPL 记账） | ⏳ M0（门控期可做） |
| P2 | serve 骨架 + `/json` 发现端点注入 `devtoolsFrontendUrl` | ⏳ M0 |
| P3 | 对 Chromium 空跑的面板可用度基线（判定判据现实化） | ⏳ M0 |
| P4 | Elements/Console 演示流（DOM/CSS/Runtime 域消费） | ⏳ M1（门控解锁后） |
| P5 | Network/Application 演示流（Network 域事件流 + cookie jar） | ⏳ M2 |
| P6 | 桌面 GUI 模式 CDP server 接线（apps/browser 共享面） | ⏳ M3 |

## 已完成切片

（立项轮，暂无）

## 下一步计划

1. **M0 自主面**：bundle 供给三件事（获取/pin/许可记账）→ serve 骨架 → 对 Chromium
   空跑定「面板可用」判据基线，落 evidence/
2. **门控观察**：cdp-protocol M1/M2 进度跟踪；M3 解锁后启动 M1

**待用户决策清单**：
- （暂无；bundle 许可核查有疑点时按入口文档协议 BLOCK 等裁决）

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M0 — 门控期自主面（bundle/serve/判据基线） | ⏳ |
| M1 — 附接与 Elements/Console | ⏳ 门控：cdp-protocol M3 |
| M2 — Network/Application | ⏳ |
| M3 — 桌面接线 + 收口 | ⏳ |

## 验证基线

- 测试基线：立项时点全绿（`make test` 19,170P/0F，2026-09-12 变基后口径；经 test-guard）
- DevTools UI 现状：零（无内置面板）；CDP 面见 cdp-protocol goal 账本
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过；
  面板演示流须可脚本重放才计入 evidence「绿」

**碰撞管理**：碰 `apps/browser` 前先 `git log --since="14 days ago" -- apps/browser/`
核对 android-browser 活跃面。
