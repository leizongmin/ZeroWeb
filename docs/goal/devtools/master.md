# DevTools 调试面 — 运行时控制面板（master.md）

**入口文档**: [../devtools.md](../devtools.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-16（M0 首轮：门控解锁确认 + P2 serve 骨架落地 + P1 渠道定谳 + P3 探测脚本）

---

## 当前状态

**专项定位**：复用 Chrome DevTools frontend（pin bundle）经 CDP 附接 ZeroWeb，四面板
（Elements / Console / Network / Application-cookie）达到「逐面板演示流可判定可用」。
**启动门控**：**已解锁**——cdp-protocol goal 2026-09-16 M5 定稿收口（Done），M3 DOM/CSS
域在位；本轮起进入主线 M0/M1 推进。

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
| P1 | devtools-frontend bundle 获取/版本 pin/许可核查 | 🔨 M0 进行中——渠道定谳完成（见 evidence/M0-bundle-provision.md §1）；正式路径=官方仓库 main tarball pin commit + 从源构建（gn 自建 v2465 + ninja + node）；许可实测 **BSD-3-Clause**（入口文档预期「Apache-2.0 系」修正记账，实质非 MPL 关切不受影响）；tarball 下载中，产物 sha256 待补录 |
| P2 | serve 骨架 + `/json` 发现端点注入 `devtoolsFrontendUrl` | ✅ M0 本轮落地——`headless/devtools_serve.rs` + `ZW_DEVTOOLS_FRONTEND_DIR`（runtime-config 注册）+ `/json` 双形态 URL；对真实 Chromium 实测 discovery/entry/chunk/穿越拒绝全绿 |
| P3 | 对 Chromium 空跑的面板可用度基线（判定判据现实化） | 🔨 M0 进行中——probe 脚本（evidence/panel-baseline-probe.mjs）就绪可重放；第三方预构建包实测白屏缺陷（node:worker_threads 静态 import 被 CSP 拒 → 模块图死，存证截图）；待官方构建产物重跑定基线 |
| P4 | Elements/Console 演示流（DOM/CSS/Runtime 域消费） | ⏳ M1（另见下方 M1 预研注记：per-page WS 路由缺口） |
| P5 | Network/Application 演示流（Network 域事件流 + cookie jar） | ⏳ M2 |
| P6 | 桌面 GUI 模式 CDP server 接线（apps/browser 共享面） | ⏳ M3 |

## 已完成切片

- **2026-09-16 M0-P2 serve 骨架**（`devtools_serve.rs` 新模块 + discovery 路由 + runtime-config
  注册 + docs/runtime-environment.md + 单测 4：entry/asset 解析、bare 前缀、穿越拒绝、非法转义；
  另 tests.rs 补 devtoolsFrontendUrl 占位形态断言）。预期对 cdp-protocol 守成面零漂移
  （未配置 bundle 时 `/json` 输出与旧形态逐字节一致）。

## M1 预研注记（本轮实测得出的结构缺口）

- frontend 入口 `inspector.html?ws=<host:port>/<path>`：Chrome 形态是 **per-page WS 路径**
  （`ws=<host:port>/devtools/page/<targetId>`，frontend 在该 socket 上说**非包裹**的 page
  域协议）；ZeroWeb 现为单一 browser 级 WS 端点（`ws://host:port/`，accept 不分路径），
  CDP 面板附接需 WS 升级请求按路径路由（`/devtools/page/<id>` → page 会话态直连）。
  归属 M1 切片（headless transport 面，属本 goal 工作面；碰 cdp-protocol 守成面前跑
  `make cdp-e2e` 防回归）。
- frontend CSP `connect-src ... ws://127.0.0.1:*`——ws 参数须用 127.0.0.1 形态
  （headless 默认绑 127.0.0.1，天然满足）。

## 下一步计划

1. **P1 收口**：main tarball 落地 → pin commit 记账 → gn gen + ninja 构建
   （绕 depot_tools：直接 gn + ninja + npm devDeps；vpython3 缺位时 PATH shim python3）→
   产物 sha256 补录 evidence → provision 脚本（scripts/fetch-devtools-frontend.sh）
2. **P3 收口**：官方 bundle 重跑 panel-baseline-probe → 四面板基线判定落 evidence
3. **M1 启动**：per-page WS 路由切片 → frontend 附接 ZeroWeb → Elements/Console 演示流
4. **门禁**：每轮 `make test` + `make cdp-e2e`（headless 面防回归，cdp-protocol 守成门）

**待用户决策清单**：
- （暂无；第三方 `@chrome-devtools/*` 预构建包经实测对浏览器 serve 缺陷 + 许可声明不符，
  已排除，不构成待决事项；官方 BSD-3-Clause 与入口文档「非 MPL」实质关切一致，无需裁决）

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M0 — 门控期自主面（bundle/serve/判据基线） | 🔨 P2 ✅ / P1、P3 进行中 |
| M1 — 附接与 Elements/Console | ⏳ 门控已解锁（per-page WS 路由为首个切片） |
| M2 — Network/Application | ⏳ |
| M3 — 桌面接线 + 收口 | ⏳ |

## 验证基线

- 测试基线：`make test`（2026-09-16 起点树 = R4405 组合态 19,324P/0F 口径；本轮落地后
  以实测为准）；headless 面防回归加跑 `make cdp-e2e`（基线 33 绿）
- DevTools UI 现状：frontend bundle 走机器本地 `~/.cache/zeroweb/devtools/`（不入库）；
  serve 面已接（`ZW_DEVTOOLS_FRONTEND_DIR`）
- 质量门禁：`cargo fmt` + `cargo clippy -D warnings` 全过；面板演示流须可脚本重放才计入
  evidence「绿」

**碰撞管理**：碰 `apps/browser` 前先 `git log --since="14 days ago" -- apps/browser/`
核对 android-browser 活跃面。
