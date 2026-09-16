# DevTools 调试面 — 运行时控制面板（master.md）

**入口文档**: [../devtools.md](../devtools.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-16（M1 首轮：S1 per-page WS 路由落地 + S2 附接试验台 + S3 域缺口账本定谳）

---

## 当前状态

**专项定位**：复用 Chrome DevTools frontend（pin bundle）经 CDP 附接 ZeroWeb，四面板
（Elements / Console / Network / Application-cookie）达到「逐面板演示流可判定可用」。
**启动门控**：**已解锁**——cdp-protocol goal 2026-09-16 M5 定稿收口（Done）。
**M0 已完成**（2026-09-16）。**M1 进行中**：S1 路由 ✅，S2 试验台就绪，S3 账本定谳
（核心缺口 = DevTools 形状的 DOM/CSS/Overlay/Page 域族）。

**与兄弟 goal 的边界**：
- cdp-protocol — 上游协议基座（已收口进入守成态）：本 goal 只消费其 CDP 面；其守成门
  `make cdp-e2e`（基线 33 绿）是本 goal 动 headless 面时的防回归门。M1-S3 判读：
  frontend 所需域族是本 goal 消费侧扩展（headless/domains/），非上游矩阵面缺口
- android-browser — `apps/browser` 共享活跃并行流，碰前 git log 核对（2026-09-16 实测：
  近 14 天 apps/browser 动面为渲染流域字体管线，与本 goal 的 headless/discovery 面零重叠）
- rendering-compat — crate 零重叠

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | devtools-frontend bundle 获取/版本 pin/许可核查 | ✅ M0——pin `9bd6a496c3394422674c62a19e9faa627817c56e`；许可 BSD-3-Clause；可重放脚本 + 机器本地 cache |
| P2 | serve 骨架 + `/json` devtoolsFrontendUrl 注入 | ✅ M0（e6364b8f0） |
| P3 | 对 Chromium 空跑的面板可用度基线 | ✅ M0（probe 全绿 + 判据四条） |
| P4 | Elements/Console 演示流 | 🔨 M1 进行中——S1 路由 ✅；Elements 树空（`DOM.getDocument` 缺口）→ S3a 最小域集为下一切片 |
| P5 | Network/Application 演示流 | ⏳ M2（依赖 S1.5 并发连接切片解锁多步流） |
| P6 | 桌面 GUI 模式 CDP server 接线 | ⏳ M3 |

## 已完成切片

- **2026-09-16 M0 全部**（P2=e6364b8f0，P1+P3=bcbc72bdb）：见 evidence/M0-bundle-provision.md。
- **2026-09-16 M1-S1 per-page WS 路由**：升级路径捕获（`extract_request_path`）+
  per-tab `devtoolsFrontendUrl`（`/devtools/page/<targetId>` 形态）+ idle 计时重置修复
  （600s 无消息误杀长活连接——历史重构丢失重置语义，DevTools UI 必踩）。单测 +3。
  门禁：cargo test 463P/0F + clippy -D warnings clean + cdp-e2e 33 绿。
- **2026-09-16 M1-S2 附接试验台**：`evidence/attach-zeroweb-probe.mjs`（可重放）。
  实证 frontend 经 per-page ws 附接 ZeroWeb 页面、Elements 面板骨架渲染；Elements 树
  空根因定谳 = `DOM.getDocument` -32601（见 S3 账本）。
- **2026-09-16 M1-S3 域缺口账本**：frontend console -32601 全清单（39 条去重族）落
  `evidence/M1-attach-testbed.md` §3。核心缺口四域：`DOM.enable/DOM.getDocument`、
  `CSS.enable` 族、`Page.getResourceTree`、`Overlay.enable` 族；enable 型 no-op ack
  21 条；结构判断 = ZeroWeb CDP 面为 Playwright 形状（backendNodeId 盒模型流），
  frontend 为 DOM-nodeId 树流——`DOM.getDocument` 全树序列化是 Elements 的唯一数据源。

## 结构性限制记账

- **单连接串行 accept 循环**（headless transport，本 goal 工作面）：一次只服务一个
  连接，第二 WS 客户端在 backlog 饿死（attach 试验台实测）。M1 Elements 单连接流
  可先行；Console/Network 多步流与多客户端场景需 **并发连接切片**（thread-per-connection
  + HeadlessSession 共享化边界），排 M1-S1.5。

## 下一步计划（按序）

1. **M1-S3a 最小域集**：`DOM.enable` + `DOM.getDocument`（zero-dom 全树序列化 +
   nodeId 分配器，`headless/domains/dom.rs` 扩展）+ `Page.getResourceTree` +
   `CSS.enable`/`Overlay.enable` + 账本 enable 型 no-op ack 族 → 验收 = Elements
   树渲染被调试页活 DOM（单连接流闭环）
2. **M1-S1.5 并发连接切片**：transport 线程化，解锁 Console REPL / Network 演示流
3. **M1-S3b `CSS.getMatchedStylesForNode`**：样式侧栏数据源（style-system 消费面）
4. **门禁**：每轮 `make test` + `make cdp-e2e`

**待用户决策清单**：
- （暂无；S3 账本的实现路径已按 goal 协议定为本 goal 消费侧扩展，不动 style-system
  内部结构，不触上游 cdp-protocol 守成面语义）

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M0 — 门控期自主面 | ✅ Done（2026-09-16） |
| M1 — 附接与 Elements/Console | 🔨 S1 ✅ / S2 试验台 ✅ / S3 账本 ✅ → S3a 实现中 |
| M2 — Network/Application | ⏳ |
| M3 — 桌面接线 + 收口 | ⏳ |

## 验证基线

- 测试基线：`make test`（本切片前基线 19,328P/0F / 67 组；每轮以实测为准）；
  headless 面防回归 `make cdp-e2e`（33 绿 deterministic YES）
- DevTools UI：官方 bundle pin `9bd6a496c3394422674c62a19e9faa627817c56e`（配方
  evidence/fetch-devtools-frontend.sh；机器本地 `~/.cache/zeroweb/devtools/`，不入库）
- 质量门禁：`cargo fmt` + `cargo clippy -D warnings` 全过；面板演示流须可脚本重放
  才计入 evidence「绿」

**碰撞管理**：碰 `apps/browser` 前先 `git log --since="14 days ago" -- apps/browser/`
核对 android-browser 活跃面。
