# CDP 协议兼容 + Playwright 验证矩阵 — 运行时控制面板（master.md）

**入口文档**: [../cdp-protocol.md](../cdp-protocol.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-12（M1 切片 1 落地：headless.rs 拆分 9 模块）

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
| P1 | Playwright 命令矩阵账本（pin 版空跑导出命令全集 + 三态登记） | ✅ 初稿落地（evidence/cdp-command-matrix.md；随域更新三态） |
| P2 | headless.rs 职责拆分（2256 行超 2000 上限；transport/discovery/domains/session） | ✅ M1 切片 1（headless/ 9 模块，纯搬移零语义变化，make test 19,170P/0F 与基线一致） |
| P3 | Target/Runtime/Page/Input/DOM/CSS/Network/Emulation 域实现 | ⏳ M1-M4 |
| P4 | Node/Playwright 测试链（pin + E2E 用例集 + make 入口） | 🔶 pin 工程已入库（tests/playwright-matrix/，playwright-core 1.63.0）；E2E 用例与 make 入口随 M1 建 |
| P5 | console 对象化（V8 侧结构化序列化，替换扁平字符串） | ⏳ M4（矩阵 G5） |
| P6 | net 请求事件总线（Network 域 + devtools Network 面板共用脊柱） | ⏳ M4（矩阵 G4） |
| P7 | WS 层 sessionId 多路复用（单连接扁平会话 → per-target session，响应回显 sessionId） | 🔶 M1 切片 2 传输面完成：解析/回显/未附接校验（-32001）+ 附接注册表；per-target 真路由随切片 3 Target 域落地 |
| P8 | `/json/version` 尾斜杠 404（Playwright 请求 `/json/version/`） | ✅ M1 切片 2（normalize_discovery_path 容忍尾斜杠；`/json`、`/json/list` 同步受益） |

## 已完成切片

- **S3（2026-09-12）M1 切片 2 — 传输层 sessionId 复用 + 发现端点修正**：`ClientRequest/
  ServerResponse` 增 `sessionId`（camelCase rename，回显 + 未附接 `-32001`）；`/json/version`
  尾斜杠容忍（P8/G2 收口）；`/json`、`/json/list` 按真实标签页枚举（`zeroweb-tab-<n>`，
  url/title 取自 shell 模型）；会话提升为服务器级（target 跨连接持续，CDP 语义）；**实测
  拦截两个传输层存量 bug**——tungstenite 0.29 `write()` 小消息不落盘（响应滞留缓冲）+
  peek 阶段 5s read timeout 未恢复（空闲误杀连接），已修并沉淀 learning（2026-09-12
  tungstenite-write-buffer-stale-read-timeout）。Playwright 直连 smoke：WS 往返已通，
  connect 推进至 `Browser.getVersion` -32601（切片 3 范围）。`make test` 19,174P/0F
  （基线 19,170 + 新增 4 传输层单测）。
- **S2（2026-09-12）M1 切片 1 — headless.rs 职责拆分**（P2/G6 收口）：`apps/browser/src/headless.rs`
  （2256 行超限）→ `headless/` 9 模块（mod=transport / protocol / session / security /
  discovery / domains / client / tests / gpu_screenshot_tests），纯搬移零语义变化，
  `pub(super)` 子树内可见，测试代码零改动；`make test` 19,170P/0F 与拆分前基线一致，
  workspace clippy `-D warnings` 全过。
- **S1（2026-09-12）M1 前置纯资产切片**：`tests/playwright-matrix/` pin 工程
  （playwright-core 1.63.0 + lockfile）+ CDP 捕获代理 + 全核心流空跑脚本（30 步全绿
  @ Chromium 153.0.8010.12）→ 命令全集 395 调用/40 方法/30 事件 →
  `evidence/cdp-command-matrix.md` 初稿（三态登记 + 6 条关键契约发现 + G1-G6 结构缺口）。
  关键修正：cookie 走 **Storage 域**（非 Network.getCookies 族）；Playwright 不调
  `Target.getTargets`（连接靠 setAutoAttach flatten）；locator 流不用 DOM.getDocument/
  CSS.*，脊柱是 Runtime.callFunctionOn（156 次）→ objectId 桥。

## 下一步计划

1. **M1 切片 3**：Target 域（setAutoAttach flatten / getTargetInfo / createTarget /
   attachedToTarget 事件 + attach_session 注册表接线）+ `Browser.getVersion` +
   Runtime.enable / evaluate remoteObject 雏形 / releaseObject / runIfWaitingForDebugger
   stub → `CDP_ENDPOINT_URL=… npm run capture:chromium` 首连验收（steps-report 为差距
   清单；smoke 已确认 connect 首个缺口是 `Browser.getVersion` -32601）
2. **M2-M4**：按矩阵三态逐域收敛；cookie 落点以 **Storage.getCookies/setCookies/
   clearCookies** 为准（发现 #2 修正）；CSS.getMatchedStylesForNode 与 DOM.getDocument
   归 M3 goal 扩展面（devtools 前置）

**待用户决策清单**：
- （暂无。V8 对象句柄桥（Runtime.callFunctionOn/objectId）按入口文档先走「自主实现」；
  若 M1 首连实测其深结构拦路，再记此处）

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — 传输/发现/Target 基座 + Playwright 首连 | 🚧 传输/发现面完成（S2 拆分 + S3 sessionId/尾斜杠/枚举）；Target/Runtime 域未开始（首连缺口 = `Browser.getVersion`） |
| M2 — Page/Input 域 → 点击/填充/键盘/导航流 | ⏳ |
| M3 — DOM/CSS/Emulation → locator 流 | ⏳ |
| M4 — Network/cookies/console 对象化（cookie 落点=Storage 域） | ⏳ |
| M5 — 矩阵收口 | ⏳ |

## 验证基线

- 测试基线：立项时点全绿（`make test` 19,170P/0F，2026-09-12 变基后口径；禁止裸跑
  cargo test，经 test-guard）
- CDP 现状：`Page.navigate` / `Runtime.evaluate` / `Target.getTargets` 3 命令 +
  `/json/version` + `/json` 发现（headless.rs L782-796/L571/L1159）
- **命令矩阵捕获基线（S1，2026-09-12）**：playwright-core 1.63.0 @ Chromium 153.0.8010.12
  （chromium-1243 缓存），全核心流 30 步全绿，395 调用/40 方法/30 事件；
  `evidence/chromium-capture-2026-09-12.md` + `…-summary.json`（生成物，复现命令见账本头）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过；
  Playwright E2E 用例须双跑 deterministic 才计入账本「绿」

**碰撞管理**：碰 `apps/browser` 前先 `git log --since="14 days ago" -- apps/browser/`
核对 android-browser 活跃面。
