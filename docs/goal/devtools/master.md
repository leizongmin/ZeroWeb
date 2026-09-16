# DevTools 调试面 — 运行时控制面板（master.md）

**入口文档**: [../devtools.md](../devtools.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-16（M1-S1.5 并发多路复用落地：Elements/Console/Network 三面板全渲染，M2-N1 清账）

---

## 当前状态

**专项定位**：复用 Chrome DevTools frontend（pin bundle）经 CDP 附接 ZeroWeb，四面板
（Elements / Console / Network / Application-cookie）达到「逐面板演示流可判定可用」。
**启动门控**：**已解锁**——cdp-protocol goal 2026-09-16 M5 定稿收口（Done）。
**M0 已完成**。**M1 收口中**：S1 路由 ✅ + S3a 最小域集 ✅（Elements 活 DOM +
Console REPL）+ S1.5 并发 ✅（单线程多路复用；Network 面板渲染 M2-N1 提前清账）。
余项：S3b 样式侧栏。

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
| P4 | Elements/Console 演示流 | 🔨 M1 核心双绿——Elements 活 DOM 树 ✅ + Console REPL evaluate ✅（S3a 域集落地）；样式侧栏（S3b CSS.getMatchedStylesForNode）⏳ |
| P5 | Network/Application 演示流 | ⏳ M2（NetworkLogView 渲染遗留 = M2-N1 首项） |
| P6 | 桌面 GUI 模式 CDP server 接线 | ⏳ M3 |

## 已完成切片

- **2026-09-16 M0 全部**（P2=e6364b8f0，P1+P3=bcbc72bdb）：见 evidence/M0-bundle-provision.md。
- **2026-09-16 M1-S1 per-page WS 路由**（5c485bd21）：升级路径捕获 + per-tab
  devtoolsFrontendUrl + idle 计时重置修复。单测 +3。
- **2026-09-16 M1-S2/S3 附接试验台 + 域缺口账本**：`attach-zeroweb-probe.mjs` 可重放；
  frontend console -32601 全清单 39 条去重族 + 结构判断（Playwright 形状 vs
  DOM-nodeId 树流）。
- **2026-09-16 M1-S3a 最小域集**：`DOM.getDocument`（renderer JS 探测序列化 →
  `convert_cdp_node` CDP 形状，零 protocol-crate 改动）+ `Page.getResourceTree` +
  `Target.getTargetInfo` 无参页面分类修复（误报 browser → frontend 装载 ScreencastView
  崩溃链根因）+ enable 型 ack 族 40+ + 带返回形状三件（getIsolateId/getStorageKey/
  getNavigationHistory）。**验收双绿**：Elements 活 DOM（attach-zeroweb-elements.png）
  + Console REPL `1+1`→`2`（attach-zeroweb-console.png）；-32601 归零。单测 +1。
  门禁：cargo test 464P/0F + clippy clean + cdp-e2e 33 绿 + make test 全绿。

## 结构性限制记账

- **单连接串行 accept 循环**（headless transport，本 goal 工作面）：一次只服务一个
  连接，第二 WS 客户端在 backlog 饿死（attach 试验台实测）。M1 Elements 单连接流
  可先行；Console/Network 多步流与多客户端场景需 **并发连接切片**（thread-per-connection
  + HeadlessSession 共享化边界），排 M1-S1.5。

## 下一步计划（按序）

1. **M1-S3b `CSS.getMatchedStylesForNode`**：样式侧栏数据源（style-system 消费面；
   现 Elements 树可选中和展开，样式栏 "No matching selector or style"）
2. **M2 Network 域事件多客户端路由**：事件 drain 归属保证（面板所在连接优先）→
   Network 请求行/详情演示流；随后 Application cookie 面板（`&panel=application`）
3. **M3 桌面接线 + 收口**：GUI 模式 CDP server 可开关（CLI/默认关）
4. **门禁**：每轮 `make test` + `make cdp-e2e`

**待用户决策清单**：
- （暂无）

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M0 — 门控期自主面 | ✅ Done（2026-09-16） |
| M1 — 附接与 Elements/Console | 🔨 S1/S3a/S1.5 ✅（三面板全渲染）；余 S3b 样式侧栏 |
| M2 — Network/Application | 🔨 N1 面板渲染提前清账；余事件路由 + cookie 面 |
| M3 — 桌面接线 + 收口 | ⏳ |

## 验证基线

- 测试基线：`make test`（每轮以实测为准）；headless 面防回归 `make cdp-e2e`
  （33 绿 deterministic YES 实测维持）
- **已知 flake 记账（zero-web 流 crate，非本 goal 面）**：
  `zero-webview service_worker_runtime::navigator_skip_waiting_activates_replacement_version`
  在 make test 全量并行负载下偶发超时（M0 轮与 S1.5 轮各一次；单测隔离 0.09s 必绿）。
  按 run-rules §10 不单方面修兄弟流 crate；两轮均已隔离复跑归因后继续。
- DevTools UI：官方 bundle pin `9bd6a496c3394422674c62a19e9faa627817c56e`（配方
  evidence/fetch-devtools-frontend.sh；机器本地 `~/.cache/zeroweb/devtools/`，不入库）
- 质量门禁：`cargo fmt` + `cargo clippy -D warnings` 全过；面板演示流须可脚本重放
  才计入 evidence「绿」

**碰撞管理**：碰 `apps/browser` 前先 `git log --since="14 days ago" -- apps/browser/`
核对 android-browser 活跃面。
