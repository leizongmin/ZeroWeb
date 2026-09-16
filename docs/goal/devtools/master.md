# DevTools 调试面 — 运行时控制面板（master.md）

**入口文档**: [../devtools.md](../devtools.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-16（M0 三缺口全收口：P1 bundle 供给 + P2 serve 骨架 + P3 判据基线）

---

## 当前状态

**专项定位**：复用 Chrome DevTools frontend（pin bundle）经 CDP 附接 ZeroWeb，四面板
（Elements / Console / Network / Application-cookie）达到「逐面板演示流可判定可用」。
**启动门控**：**已解锁**——cdp-protocol goal 2026-09-16 M5 定稿收口（Done）。
**M0 已完成**（2026-09-16）：bundle 供给/serve 骨架/判据基线三件全落地，下一里程碑 M1。

**与兄弟 goal 的边界**：
- cdp-protocol — 上游协议基座（已收口进入守成态）：本 goal 只消费其 CDP 面；其守成门
  `make cdp-e2e`（基线 33 绿）是本 goal 动 headless 面时的防回归门
- android-browser — `apps/browser` 共享活跃并行流，碰前 git log 核对（2026-09-16 实测：
  近 14 天 apps/browser 动面为渲染流域 R4376-R4378 字体管线，与本 goal 的 headless/
  discovery 面零重叠）
- rendering-compat — crate 零重叠

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | devtools-frontend bundle 获取/版本 pin/许可核查 | ✅ M0——pin `9bd6a496c3394422674c62a19e9faa627817c56e`，npm 源码包+漂移闭环校验+自建 gn v2465+ninja 1711 targets 全绿；许可实测 **BSD-3-Clause**（非 MPL，实质关切成立）；可重放脚本 `evidence/fetch-devtools-frontend.sh`；bundle 不入库（机器本地 cache） |
| P2 | serve 骨架 + `/json` 发现端点注入 `devtoolsFrontendUrl` | ✅ M0——`headless/devtools_serve.rs` + `ZW_DEVTOOLS_FRONTEND_DIR`（runtime-config 注册）+ `/json` 双形态 URL；未配置时 503/`devtools://` 占位（cdp-protocol 守成面零漂移，cdp-e2e 33 绿实测） |
| P3 | 对 Chromium 空跑的面板可用度基线（判定判据现实化） | ✅ M0——probe 全绿（frontend 完整 boot + Elements 活 DOM + Console/Network 面板直开 + 零致命错误）；判据四条现实化结论见 evidence §3 |
| P4 | Elements/Console 演示流（DOM/CSS/Runtime 域消费） | ⏳ M1（首个切片=per-page WS 路径路由，见下方注记） |
| P5 | Network/Application 演示流（Network 域事件流 + cookie jar） | ⏳ M2 |
| P6 | 桌面 GUI 模式 CDP server 接线（apps/browser 共享面） | ⏳ M3 |

## 已完成切片

- **2026-09-16 M0-P2 serve 骨架**（commit e6364b8f0）：`devtools_serve.rs` 新模块 +
  discovery 路由 + runtime-config 注册 + docs/runtime-environment.md + 单测 4。
  门禁：make test 19,328P/0F（67 组全 ok）+ make cdp-e2e 33 绿 deterministic YES +
  clippy -D warnings 零告警。
- **2026-09-16 M0-P1/P3 bundle 供给 + 判据基线**：见 `evidence/M0-bundle-provision.md`
  （渠道定谳/第三方包缺陷存证/官方构建配方/本地补丁清单/基线判定表）。

## M1 预研注记（M0 实测得出的结构缺口）

- **per-page WS 路径路由**（M1 首个切片）：Chrome 形态 `devtoolsFrontendUrl` 带
  `ws=<host:port>/devtools/page/<targetId>`，frontend 在该 socket 上说**非包裹**的
  page 域协议；ZeroWeb 现为单一 browser 级 WS 端点（accept 不分路径，Playwright 的
  Target.setAutoAttach 扁平化模型）。需 WS 升级请求按路径路由（`/devtools/page/<id>`
  → 直接以 page 会话态应答，无 sessionId 包裹）。碰 cdp-protocol 守成面前跑
  `make cdp-e2e` 防回归。
- frontend CSP `connect-src ... ws://127.0.0.1:*`——ws 参数须用 127.0.0.1 形态
  （headless 默认绑 127.0.0.1，天然满足）。
- 面板直开入口：`inspector.html?ws=<...>&panel=<console|network|...>`（M2 Application
  面板演示流可复用；Application 面板名待 M2 实测）。

## 下一步计划

1. **M1-S1 per-page WS 路由切片**：transport 面按路径分发 + page 会话态直连
2. **M1-S2 frontend 附接 ZeroWeb**：`/json` devtoolsFrontendUrl 指向 per-page ws →
   frontend 检查 ZeroWeb 页面（Elements 树/Console evaluate 演示流，逐条落 evidence）
3. **M1-S3 域缺口回流**：frontend 附接 ZeroWeb 时逐方法 -32601 清单对照 cdp-protocol
   账本，结构性缺口记账回流上游 goal
4. **门禁**：每轮 `make test` + `make cdp-e2e`（headless 面防回归）

**待用户决策清单**：
- （暂无）

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M0 — 门控期自主面（bundle/serve/判据基线） | ✅ Done（2026-09-16） |
| M1 — 附接与 Elements/Console | ⏳ 下一个（S1=per-page WS 路由） |
| M2 — Network/Application | ⏳ |
| M3 — 桌面接线 + 收口 | ⏳ |

## 验证基线

- 测试基线：`make test` 19,328P/0F（2026-09-16 实测，67 组全 ok，= R4405 组合态
  19,324 + 本 goal serve 骨架单测 4）；headless 面防回归 `make cdp-e2e`（33 绿
  deterministic YES 实测维持）
- DevTools UI：官方 bundle pin `9bd6a496c3394422674c62a19e9faa627817c56e`（构建配方
  evidence/fetch-devtools-frontend.sh；bundle 机器本地 `~/.cache/zeroweb/devtools/`，
  不入库）；serve 面 `ZW_DEVTOOLS_FRONTEND_DIR` 接入
- 质量门禁：`cargo fmt` + `cargo clippy -D warnings` 全过；面板演示流须可脚本重放
  才计入 evidence「绿」

**碰撞管理**：碰 `apps/browser` 前先 `git log --since="14 days ago" -- apps/browser/`
核对 android-browser 活跃面。
