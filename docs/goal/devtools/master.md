# DevTools 调试面 — 运行时控制面板（master.md）

**入口文档**: [../devtools.md](../devtools.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-17（守成验证轮：cdp-e2e 33 绿 + make test 全绿复测，基线无漂移）

---

## 当前状态

**专项定位**：复用 Chrome DevTools frontend（pin bundle）经 CDP 附接 ZeroWeb，四面板
（Elements / Console / Network / Application-cookie）达到「逐面板演示流可判定可用」。
**启动门控**：**已解锁**——cdp-protocol goal 2026-09-16 M5 定稿收口（Done）。
**goal 状态：Done（2026-09-17 M3 收口，转守成态）**。全 DC 判定：
- **DC-1 ✅** bundle pin + BSD-3 许可 + serve 骨架（M0）；
- **DC-2 ✅** 四面板演示流全绿（Elements 树+Computed/Console REPL/Network 请求行/
  Application cookie 可见+回写；probe 21/21 门禁化）；
- **DC-3 ✅** GUI 模式 CDP 可开关（CLI 显式/默认关，本机 X 实测 /json 服务）+
  make test/clippy/fmt 全绿 + 生产路径零回归（GUI 路径仅 additive 线程派生）。
**守成门**：`make cdp-e2e`（33 绿）。**挂账（碰头协调/用户点名）**：GUI 标签页桥接
（深结构）、Styles matched rules（engine 侧协议面）、overlay 高亮渲染；灰置面板：
Sources/Performance/Security 等（frontend 显示但域不实现，符合 goal 预期形态）。

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
| P4 | Elements/Console 演示流 | ✅ M1——Elements 活 DOM 树 + Console REPL + Computed 侧栏（盒模型+计算值）+ Styles（inline 起步，matched rules 记账碰头项） |
| P5 | Network/Application 演示流 | ✅ M2——N1 渲染 + N2 事件路由 + N3 cookie 面板 + N4 请求行演示流全落地（DC-2 判据全通，probe 21/21 门禁化） |
| P6 | 桌面 GUI 模式 CDP server 接线 | ✅ M3——CLI 显式开关（默认关）+ 独立辅助会话形态（GUI 标签页桥接挂账） |

## 已完成切片

- **2026-09-16 M0 全部**（P2=e6364b8f0，P1+P3=bcbc72bdb）：见 evidence/M0-bundle-provision.md。
- **2026-09-16 M1-S1 per-page WS 路由**（5c485bd21）：升级路径捕获 + per-tab
  devtoolsFrontendUrl + idle 计时重置修复。单测 +3。
- **2026-09-16 M1-S2/S3 附接试验台 + 域缺口账本**：`attach-zeroweb-probe.mjs` 可重放；
  frontend console -32601 全清单 39 条去重族 + 结构判断（Playwright 形状 vs
  DOM-nodeId 树流）。
- **2026-09-17 M3 GUI 模式 CDP 接线**：main.rs GUI 路径 `--remote-debugging-port`
  显式开关（默认关）→ 后台线程 HeadlessServer（mux 全量复用，loopback）。**验收**：
  本机 X display GUI 运行中 /json/version + /json 服务实测（附接流程与 headless 全同）。
  GUI 标签页桥接挂账（深结构）。门禁：465P/0F + clippy clean + cdp-e2e 33 绿 + make test 全绿。
- **2026-09-17 M2-N4 Network 事件形状对齐**：Chrome 实捕获基准（订阅 page-target
  抓原始帧）→ ZeroWeb 两条事件源（proxy_fetch 导航路径 + FetchObserved 页面 fetch
  路径）全对齐（秒基 timestamp/documentURL/wallTime/type/headers/initialPriority 族）；
  **决定性发现：requestWillBeSent 的 `type` 字段缺失 = 行不渲染**。验收：probe 21/21
  全绿（frontend-network-requests 转门禁，面板 footer 计数器 ≥1）。门禁全绿。
- **2026-09-16 M2-N3 Application cookie 面板**：Network 域 cookie 三件（getCookies/
  setCookie/clearBrowserCookies，与 Storage 同 jar）+ transport 反压双层修复（probe
  排空管道 + idle tick 日志降 trace）。**验收**：Application 面板（&panel=resources，
  面板 id 注记）Cookies 树展开 example.com origin → cookie 表渲染
  （attach-zeroweb-application.png）+ Network.setCookie 编辑回写 jar 反映 edited-v2。
  probe 全绿 exit=0。门禁：465P/0F + clippy clean + cdp-e2e 33 绿 + make test 全绿。
- **2026-09-16 M1-S3b 样式侧栏数据面**：`domains/css.rs` 新模块——nodeId→`__zwSelector`
  注册表（getDocument 随树捕获）+ `CSS.getComputedStyle`（host 计算值桥消费，15 属性
  清单）+ `CSS.getMatchedStylesForNode`（inline 起步 + 规则内省记账）。**验收**：probe
  15/15 全绿；frontend UI 实测 Computed 侧栏盒模型+计算值渲染
  （attach-zeroweb-computed-sidebar.png）。单测 +2。门禁全绿。
- **2026-09-16 M1-S3a 最小域集**：`DOM.getDocument`（renderer JS 探测序列化 →
  `convert_cdp_node` CDP 形状，零 protocol-crate 改动）+ `Page.getResourceTree` +
  `Target.getTargetInfo` 无参页面分类修复（误报 browser → frontend 装载 ScreencastView
  崩溃链根因）+ enable 型 ack 族 40+ + 带返回形状三件（getIsolateId/getStorageKey/
  getNavigationHistory）。**验收双绿**：Elements 活 DOM（attach-zeroweb-elements.png）
  + Console REPL `1+1`→`2`（attach-zeroweb-console.png）；-32601 归零。单测 +1。
  门禁：cargo test 464P/0F + clippy clean + cdp-e2e 33 绿 + make test 全绿。

## 结构性限制记账

- ~~单连接串行 accept 循环~~ ✅ 已解（M1-S1.5 单线程 socket 多路复用；frontend lazy
  import 与多客户端并存不再饿死）。~~Network 域事件 drain 归属~~ ✅ 已解（M2-N2 订阅制
  广播；双客户端协议级实测收全事件序列）。

## 下一步计划（守成态）

1. **守成门**：每轮 `make cdp-e2e`（33 绿）+ `make test`；本 goal 文件面改动先
   pull-rebase
2. **碰头协调项（用户点名后启）**：GUI 标签页桥接（深结构）；Styles matched rules
   （engine 侧协议面）；overlay 高亮渲染

**待用户决策清单**：
- （暂无；碰头协调项均已在挂账区记账，待用户点名）

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M0 — 门控期自主面 | ✅ Done（2026-09-16） |
| M1 — 附接与 Elements/Console | ✅ Done（2026-09-16：S1+S3a+S1.5+S3b 全落地，三面板渲染+样式侧栏） |
| M2 — Network/Application | ✅ Done（2026-09-17：N1 渲染 + N2 事件路由 + N3 cookie 面板 + N4 请求行演示流全落地，DC-2 判据全通） |
| M3 — 桌面接线 + 收口 | ✅ Done（2026-09-17：GUI 开关 + 全 DC 判定 + 挂账清单） |

## 验证基线（守成态）

- 测试基线：`make test`（每轮以实测为准）；headless 面防回归 `make cdp-e2e`
  （33 绿 deterministic YES 实测维持）
- **2026-09-17 守成验证轮**：c782afbe6 上复测——`make cdp-e2e` 33 绿 deterministic
  YES；`make test` 全绿（0F）。另记共享面观察：`cargo build -p zero-browser`
  （default 无 v8/script-runtime）下 zero-engine `match_media_to_json` 报 dead_code
  warning——workspace 根对该依赖 `default-features = false`，单包 build 时 callbacks
  注册块被 cfg 掉所致；workspace 统一 feature 图（renderer/webview-demo 开 v8）不出现，
  clippy/CI 门不受影响（`cargo clippy -p zero-engine --lib` 实测干净）。engine 属
  兄弟流共享面，按 run-rules §9/§10 不单方面修，记账待归属流收敛。
  **运维观察**：`make cdp-e2e` 的 verify-deterministic.mjs 退出时不 reap 其 spawn 的
  zero-browser --headless + Playwright Chromium 进程树，每轮各漏一对（本轮清理前本树
  累计 7 对 + 4 棵 Chromium 树，全部为本树测试遗留、已清）；长期守成轮次会持续累积，
  待 cdp-protocol 流收敛（属 tests/playwright-matrix 面，不单方面修）。注意本树存在
  符号链接别名入口（`$HOME/work/<repo>` 形式），进程 cwd/exe 归属判断须以
  `readlink -f` 解析为准（勿按 argv 路径二分）。
- **挂账清单（灰置/碰头）**：①GUI 标签页桥接（深结构，用户点名）；②Styles
  matched rules（engine 协议面）；③overlay 高亮渲染；④灰置面板（预期形态）：
  Sources/Performance/Security/Profiler 等 frontend 显示但域不实现
- **已知 flake 记账（zero-web 流 crate，非本 goal 面）**：
  `zero-webview service_worker_runtime::navigator_skip_waiting_activates_replacement_version`
  在 make test 全量并行负载下偶发超时（M0 轮与 S1.5 轮各一次；单测隔离 0.09s 必绿）。
  按 run-rules §10 不单方面修兄弟流 crate；两轮均已隔离复跑归因后继续。
  同族：`zero-integration-tests network_loading::stale_etag_revalidation_is_coalesced`
  （M3 轮一次；隔离 6/6 必绿 0.11s）——全量并行负载时序敏感型 flake 家族，记账待
  兄弟流收敛。
- DevTools UI：官方 bundle pin `9bd6a496c3394422674c62a19e9faa627817c56e`（配方
  evidence/fetch-devtools-frontend.sh；机器本地 `~/.cache/zeroweb/devtools/`，不入库）
- 质量门禁：`cargo fmt` + `cargo clippy -D warnings` 全过；面板演示流须可脚本重放
  才计入 evidence「绿」

**碰撞管理**：碰 `apps/browser` 前先 `git log --since="14 days ago" -- apps/browser/`
核对 android-browser 活跃面。
