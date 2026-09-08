# WebDriver 服务完善 — 运行时控制面板（master.md）

**入口文档**: [../webdriver.md](../webdriver.md)
**创建日期**: 2026-09-07（goal 拆分 bootstrap）
**最后更新**: 2026-09-08（**GB-20260908 巡检——screenshot 待决策已飞书征询**（msg
`om_x100b6535d918a8acc121a65238fec1b`，三方案建议；renderer back/forward 跨流
rule 10 留档告知随同批发送）；两决策行补征询凭据。无代码变更。）
**（注：下方「最后更新」2026-09-08 终验复核块与 M3 推进块为前轮记录，保留作历史）**
**最后更新**: 2026-09-08（终验复核——DC 逐项判定，跨流红灯归因后收口态）
**最后更新**: 2026-09-08（M3 推进——window 族落地 + screenshot 摸底结论 + 验证通道文档）

---

## 当前状态

**专项定位**：`zero-webdriver` 从 9 endpoint 最小子集补齐到「驱动自动化验证够用」+
接线为 CI 可用验证基建。W3C 协议为准，每 endpoint 一切片（实现 + wire format 测试 +
HTTP 全链路测试）。

**M1 已收口（2026-09-08）**：endpoint 9 → 16（+status/session readback/url/back/forward/
refresh/timeouts GET+POST）；CI 接线完成（v8 app tests + quickjs 矩阵均含
`-p zero-webdriver`）；W3C 兼容性清单基线落地（evidence/endpoint-matrix.md）。

**M2 切片 1+2 已收口（2026-09-08）**：endpoint 16 → 25。+GET /source、find_elements、
text/rect/enabled/selected/attribute/property/css value、clear。协议新增
`AutomationOperation::FindElements`/`ElementState`/`ElementClear` +
`AutomationResult::Elements` + `AutomationStateQuery`（wire roundtrip 测试同步）。
集成测试 8 个全绿。

**M2 已整体收口（2026-09-08）**：endpoint 25 → 26。+POST /execute/async（callback 完成
语义：同步/定时器路径 + 未调用超时 javascript error）。集成测试 9 个全绿。
M2 元素交互族 + 执行族全部落地。

**M3 已收口（2026-09-08）**：endpoint 26 → 33。window 族 7 endpoint（handle/handles/
window GET、rect GET/POST、maximize、fullscreen）+ 验证通道文档
（evidence/verification-channel.md）。screenshot 摸底结论：技术可行但转换层在 browser
域，属深结构 → 记待用户决策。集成测试 10 个全绿。

**终验与 DC 复核（2026-09-08）**：见「Done Criteria 判定」章节。workspace clippy
全绿；webdriver 全部测试面（v8 + quickjs 两 feature 组）全绿；`make test` 主矩阵
780 passed + 1 failed——failed 项 `test_css_container_query_style_integration` 经归因
属 rendering-compat 流今日 R4124-R4127 @container 行为变化（详见碰头信号记录），
与本流改动零交集。

**与兄弟 goal 的边界**：
- rendering-compat — 渲染流域 crate 域零重叠
- event-loop-spec — **apps/renderer 属该流活跃域**；本流只碰 renderer 的 Automation
  消息处理段，不碰事件循环/observer tick 段；发现要碰即暂停记入本表（碰头信号，run-rules §9）
- keyboard-* / editing-contenteditable — 本流为其提供验证通道能力，不替它们写用例
- 共享面：crates/protocol（Automation 消息族）、apps/renderer（处理端）——碰之前
  `git log --since="14 days ago" -- crates/protocol/ apps/renderer/` 核对

**碰头信号记录（2026-09-08）**：`make test` 终验发现
`zero-integration-tests::cross_crate_integration::test_css_container_query_style_integration`
红灯（@container 视口匹配 → R4124 起按最近 container-type 祖先求值，测试 DOM 无
container-type 祖先不再命中）。归因：rendering-compat 流今日 R4124-R4127 系列
（26be41952..8e5b9675d，style-system 域）；本流 6 提交零触碰 css-parser/style-system。
按 run-rules §10 不硬解跨流红灯——该测试断言与 R4124 后的规范行为（container query
须有容器）已不符，应由 rendering-compat 流更新测试（给 DOM 加 container-type 容器）。
本流终验以本流测试面 + clippy 全绿为准。

## 实测基线（2026-09-08 M1 收口时）

### 已实现（33 endpoint）

- ✅ 会话：POST /session、DELETE /session/{id}、GET /session/{id}（capabilities 回读）、GET /status
- ✅ 导航族：POST /url、GET /url、back、forward、refresh、GET /title
- ✅ 超时：GET/POST /timeouts（字段可选、部分更新、400 校验）
- ✅ 元素定位：POST /element、POST /elements（复数）、GET /element/active
- ✅ 元素交互：click、value（send keys）、clear
- ✅ 元素状态族：text、rect、enabled、selected、attribute/{name}、property/{name}、css/{name}
- ✅ 执行：POST /execute/sync、POST /execute/async、GET /source
- ✅ 窗口：GET /window、GET /window/handle、GET /window/handles、GET+POST /window/rect、
  maximize、fullscreen（单窗口架构；rect 走 SetViewport 既有链路）
- ✅ 基建：零依赖 HTTP（loopback-only、默认 9515）、元素引用映射 4096 上限 +
  history_epoch 跨导航守卫、parse_webdriver_keys、错误码映射（400 invalid argument /
  404 no such session·element·stale / 408 timeout / 500 unknown）+ W3C 错误包络 stacktrace
- ✅ 集成测试：tests/http_session.rs 10 个全链路用例（真实 TCP + 真实 renderer 子进程）+
  session.rs 3 + renderer automation 3 单元测试

### 缺失（M4 面——本 goal Done 判定不阻塞）

- ⬜ screenshot（待用户决策，见下）
- ⬜ 定位策略扩展（link text / tag name / xpath——css 之外）
- ⬜ frame/parent 深化（依赖 renderer iframe 语义）
- ⬜ minimize / window close / new window（多窗口语义）

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | CI 接线（集成测试入 CI） | ✅ M1（ci.yml v8 app tests + quickjs 矩阵） |
| P2 | W3C 兼容性清单基线 | ✅ M1（evidence/endpoint-matrix.md） |
| P3 | 会话/导航族 endpoint | ✅ M1 |
| P4 | 元素状态族 + 执行族 endpoint | ✅ M2（26 类全落地） |
| P5 | 窗口/截图能力评估 + 兄弟流验证通道文档 | ✅ M3（screenshot 结论记待决策；验证通道文档落地） |

## 下一步计划

**已到收口判定点**。剩余事项均已归类：
- screenshot → 待用户决策（不阻塞 DONE）
- 跨流红灯（container query）→ rendering-compat 流归因，见碰头信号记录
- 若用户决策开放 screenshot：从 master.md「待用户决策」表起新一轮切片

**碰撞管理**：碰 protocol/renderer 前 `git log --since="14 days ago" --
crates/protocol/ apps/renderer/` 核对 event-loop-spec 流活跃面（apps/renderer/src/runtime.rs
有 keyboard-default-actions 流活跃提交——**不碰 runtime.rs**；automation.rs 2026-09-08
实测 14 天零提交，已安全扩展 FindElements/ElementState/ElementClear 三操作）。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — 门禁就位 + 会话/导航族 | ✅ 2026-09-08 |
| M2 — 元素交互族 + 执行族 | ✅ 2026-09-08 |
| M3 — 窗口/截图评估 + 接线收尾 | ✅ 2026-09-08（screenshot 记待决策） |

## 关键决策记录

| 决策 | 理由 |
|------|------|
| back/forward 历史边界 no-op 返回成功 | renderer 栈边界直接回 Ok 不发生加载；与 ChromeDriver 边界行为一致，W3C 未规定边界必须报错 |
| 会话默认超时 10s script / 15s pageLoad / 0s implicit（偏离 W3C 300s 默认） | 自动化驱动 fail-fast 更安全；既有集成测试依赖；记录于 SessionTimeouts doc comment |
| 元素引用加 webdriver 本地 `history_epoch` 守卫 | renderer back/forward 路径（`reload_history_entry`）不 bump `document_generation`，同 node handle 跨历史条目会被误判存活——本层保守判 stale（W3C 正确错误码），避免误触新文档元素；不碰 renderer（event-loop-spec 活跃域） |
| GET /url 用会话态而非 ExecuteScript location.href | 页面 shim location.href 可被脚本重写/不可靠；会话态由 renderer UrlChanged 消息维护（重定向跟随真值），零 renderer 改动 |
| refresh 走 Reload IPC + epoch+1 | renderer Reload 内部已带 `navigation_epoch.wrapping_add(1)`（runtime.rs:2381），与 navigate 语义一致 |
| timeout 错误映射 HTTP 408 | W3C §processing-model 错误码-状态映射表；此前 500 是 wire format 偏差 |
| execute/async 走 webdriver 层 ticket 全局 + ExecuteScript 轮询（非协议新操作） | renderer runtime struct 在 event-loop-spec 活跃域不可加字段；探测间隙 renderer 主循环每拍 drain_pending_script_mutations 自然驱动 microtask/定时器；超时（timeouts.script）由会话持有方管理，语义更贴合 W3C session scope timeout |
| window 族按单窗口架构最小实现 | 本服务 session = 单 renderer 子进程 = 单文档；句柄固定值 + rect 经既有 SetViewport；maximize/fullscreen 记状态（headless 无宿主窗口语义）；minimize/close/new window 不做（多窗口语义超出范围） |
| screenshot 记待用户决策 | 技术路径存在：renderer Legacy 模式 ViewPainted 图元表 + headless.rs 已验证 render_full_scene+PNG 路线；但 IPC→RenderPrimitives 转换与 ImageCache 累积在 apps/browser/src/paint_ipc.rs（browser 域），复用需抽公共层或跨流依赖——属深结构改动，须用户拍板 |

## 待用户决策

| 项 | 状态 | 说明 |
|----|------|------|
| screenshot 链路 | ⬜ 待决策（已征询） | M3 摸底结论：技术可行（ViewPainted 图元 + render_full_scene + PNG），但 IPC→图元转换层在 apps/browser 域（paint_ipc.rs），复用需抽公共库或允许 webdriver 依赖 zero-browser lib——深结构改动须拍板。现状：按 DONE 允许条件记待决策不算未满足 DC。**GB-20260908 巡检飞书征询 msg `om_x100b6535d918a8acc121a65238fec1b`（含三方案建议：①抽公共截图转换层 crate【推荐】/②webdriver 依赖 zero-browser lib/③维持不做）** |
| alert 全族 / print | ⬜ 排除 | 依赖 host 对话框/打印能力 |
| actions 完整语义 | ⬜ 排除 | 依赖输入管线深化；keyboard 够用子集先做 |
| renderer back/forward 不 bump document_generation | ⬜ 待决策（跨流告知已发） | 本层 history_epoch 守卫已兜底（保守 stale）；若要 renderer 侧根治需碰 runtime.rs（event-loop-spec 活跃域），记待决策不阻塞。**GB-20260908 巡检按 rule 10 跨流留档告知（随 screenshot 征询同批 msg `om_x100b6535d918a8acc121a65238fec1b`）——event-loop-spec 流后续动 runtime.rs 时可顺手根治** |
| shim innerWidth 固定 1280 不跟随 SetViewport | ⬜ 待决策 | engine shim 域（js_dom_shim/part01.js:3546）；几何验证用 element/rect 真值可绕过，不阻塞 |

## 验证基线

- 测试基线（2026-09-08 终验）：
  - v8 主矩阵：zero-webdriver 10 集成 + 3 单元全绿（`make test` workspace 段实测）；
    zero-protocol 313 全绿（含 FindElements/Elements wire roundtrip）；
    zero-renderer automation 3 单元全绿
  - quickjs feature 组：zero-webdriver 10 + 3 全绿、renderer automation 3 全绿
  - `cargo clippy --workspace --all-targets -- -D warnings`：全绿（guarded-clippy 实测）
  - `make test` 主矩阵 780 passed + 1 failed——failed 属 rendering-compat 流（见碰头信号）
- W3C 兼容性清单：evidence/endpoint-matrix.md（33 endpoint 落账，终态）
- 质量门禁：`cargo fmt --all -- --check` 无 diff
- CI：v8 + quickjs 矩阵均含 zero-webdriver（ci.yml）

## Done Criteria 判定（2026-09-08）

### DC-1: 门禁与基线 — ✅

- ✅ webdriver 集成测试入 CI（ci.yml v8 app tests + quickjs 矩阵）
- ✅ W3C endpoint 兼容性清单基线持久化（evidence/endpoint-matrix.md，三态 + 行为注记）
- ✅ wire format 对照测试（错误包络 stacktrace、404/400/408/500 语义，集成测试全覆盖）

### DC-2: 核心 endpoint 补齐 — ✅

- ✅ 会话与状态：GET /status、GET /session/{id}、timeouts（GET+POST）
- ✅ 导航族：GET /url、forward、back、refresh
- ✅ 元素交互族：find_elements、text、rect、enabled、selected、attribute、property、
  css value、clear
- ✅ 执行族：execute/async、GET /source

### DC-3: 每端点全链路测试 — ✅

- ✅ 10 个 HTTP 全链路集成测试（真实 TCP + 真实 renderer 子进程，http_session.rs 模式）
- ✅ 错误路径全覆盖：no such element（404）、stale element reference（跨导航/历史守卫）、
  invalid argument（非法策略/非法超时/空脚本 400）、no such session（404）、
  async callback 超时（javascript error）

### DC-4: 测试与质量不可退让 — ✅

- ✅ 本流全部测试面绿（见验证基线）；`make test` 全 workspace 有 1 个跨流红灯，
  经归因属 rendering-compat 流 R4124-R4127 行为变化（碰头信号记录，本流无责）
- ✅ clippy `-D warnings` 全绿（guarded-clippy）
- ✅ 兼容性清单随 endpoint 落地持续更新（endpoint-matrix.md 7 次同步）

**判定**：DC-1~4 全满足；screenshot/alert/actions 按待用户决策记录在案（DONE 允许条件）。
