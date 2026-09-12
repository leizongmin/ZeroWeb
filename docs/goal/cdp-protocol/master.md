# CDP 协议兼容 + Playwright 验证矩阵 — 运行时控制面板（master.md）

**入口文档**: [../cdp-protocol.md](../cdp-protocol.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-12（立项）

---

## 当前状态

**专项定位**：把 `apps/browser/src/headless.rs` 的 CDP 雏形（3 命令）收敛到 Playwright
（pin 版本）`connectOverCDP` 可用——命令矩阵账本为验收标尺，Playwright E2E 全绿收口。
本 goal 是 devtools goal（Chrome DevTools frontend 复用）的协议基座（下游门控）。

**与兄弟 goal 的边界**：
- android-browser — `apps/browser` 共享活跃并行流，碰前 `git log --since="14 days ago"`
  核对
- devtools — 下游消费方：只消费本 goal CDP 面；改 CDP 域实现须本 goal 收口或碰头
- rendering-compat — crate 零重叠

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | Playwright 命令矩阵账本（pin 版空跑导出命令全集 + 三态登记） | ⏳ M1 前置纯资产切片 |
| P2 | headless.rs 职责拆分（2256 行超 2000 上限；transport/discovery/domains/session） | ⏳ M1 |
| P3 | Target/Runtime/Page/Input/DOM/CSS/Network/Emulation 域实现 | ⏳ M1-M4 |
| P4 | Node/Playwright 测试链（pin + E2E 用例集 + make 入口） | ⏳ M1 起随域建 |
| P5 | console 对象化（V8 侧结构化序列化，替换扁平字符串） | ⏳ M4 |
| P6 | net 请求事件总线（Network 域 + devtools Network 面板共用脊柱） | ⏳ M4 |

## 已完成切片

（立项轮，暂无）

## 下一步计划

1. **M1 前置纯资产切片**：pin Playwright 版本（package.json + lockfile 入库
   `tests/playwright-matrix/`）→ 对 Chromium `connectOverCDP` 空跑全核心流 →
   DEBUG 日志导出 CDP 命令全集 → `evidence/cdp-command-matrix.md` 初稿（三态登记
   由人工判定补全）
2. **M1**：headless.rs 拆分 + Target 域 + Runtime enable/evaluate → Playwright 首连
3. **M2-M4**：按入口文档里程碑逐域收敛，每域 E2E 用例随 land

**待用户决策清单**：
- （暂无）

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — 传输/发现/Target 基座 + Playwright 首连 | ⏳ |
| M2 — Page/Input 域 → 点击/填充/键盘/导航流 | ⏳ |
| M3 — DOM/CSS/Emulation → locator 流 | ⏳ |
| M4 — Network/cookies/console 对象化 | ⏳ |
| M5 — 矩阵收口 | ⏳ |

## 验证基线

- 测试基线：立项时点全绿（`make test` 19,170P/0F，2026-09-12 变基后口径；禁止裸跑
  cargo test，经 test-guard）
- CDP 现状：`Page.navigate` / `Runtime.evaluate` / `Target.getTargets` 3 命令 +
  `/json/version` + `/json` 发现（headless.rs L782-796/L571/L1159）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过；
  Playwright E2E 用例须双跑 deterministic 才计入账本「绿」

**碰撞管理**：碰 `apps/browser` 前先 `git log --since="14 days ago" -- apps/browser/`
核对 android-browser 活跃面。
