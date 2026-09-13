# CDP 协议兼容 + Playwright 验证矩阵 — 运行时控制面板（master.md）

**入口文档**: [../cdp-protocol.md](../cdp-protocol.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-13（S53：静默轮——tip 与 S52 逐字节一致，S48 组合态复核结论全量引用；绿步维持 33）

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
| P3 | Target/Runtime/Page/Input/DOM/CSS/Network/Emulation 域实现 | 🚧 S39 后余 1 步：frames.access 翻绿（子帧元数据探测，纯 headless 面）；唯余 frames.click+evaluate（挂子帧文档加载+JS realm——engine 子帧能力，渲染流域协调） |
| P4 | Node/Playwright 测试链（pin + E2E 用例集 + make 入口） | ✅ S8：`make cdp-e2e`（test-guard 包裹，deterministic 双跑 + expected-green 回归门）；用例集=34 步全核心流 + DC-1 缺口补测（S25） |
| P5 | console 对象化（V8 侧结构化序列化，替换扁平字符串） | ✅ S11 value-only 面落地（consoleAPICalled 绿——shim 逐参值序列化 + `__zw_console_log` 三参 + headless 转事件，PW 消费面 msg.type()/text() 全通）；完整对象句柄化（remoteObject preview/objectId）挂账随 devtools 面需求 |
| P6 | net 请求事件总线（Network 域 + devtools Network 面板共用脊柱） | 🔶 雏形已建（S7 proxy_fetch 三事件 + S14 renderer FetchObserved + S17 dataReceived 双路径）；分块流式观测点待 net 窗口流式化——**net 近 14 天无外部流占用，窗口已开**（2026-09-12 实测） |
| P7 | WS 层 sessionId 多路复用（单连接扁平会话 → per-target session，响应回显 sessionId） | ✅ S4 收口：解析/回显/未附接校验（-32001）+ 附接注册表 + Target 域 per-target 会话（ServerEvent sessionId 盖章路由，Target 宣告事件除外）——实测复核 35 方法零漂移佐证 |
| P8 | `/json/version` 尾斜杠 404（Playwright 请求 `/json/version/`） | ✅ M1 切片 2（normalize_discovery_path 容忍尾斜杠；`/json`、`/json/list` 同步受益） |

## 已完成切片

- **S53（2026-09-13）静默轮 — 同 tip 复核（无代码变更，绿步维持 33）**：
  pull 零新提交（tip = 43359b848，即 S52 提交本身）——tracked 树硬核对：tip 为
  S48 组合态门验证代码树（a0a8146e2）之上仅 docs 增量（S49-S52 记录 +
  9825b8e54 的 docs/perf bot 数据，门禁图外已记档），非 docs 代码树零变更，
  S48 全部门结论全量引用免复跑（cdp-e2e 门 PASS 33 绿 deterministic +
  make test 19,259P/0F EXIT=0）。双解冻条件不变：① 渲染流零新提交（活跃面
  维持 inline walk 域，零子帧文档加载工作）；② DC-2 口径无新拍板记录。goal
  自有面零新缺口、无扩展面（S40-S52 重审结论延续）。
- **S52（2026-09-13）静默轮 — 同 tip 复核（无代码变更，绿步维持 33）**：
  pull 零新提交（tip = d7749fe22，即 S51 补记提交本身）——tracked 树硬核对：tip 为
  S48 组合态门验证代码树（a0a8146e2）之上仅 docs 增量（S49-S51 记录 + 9825b8e54
  的 docs/perf bot 数据，后者门禁图外已记档），非 docs 代码树零变更，S48 全部门
  结论全量引用免复跑（cdp-e2e 门 PASS 33 绿 deterministic +
  make test 19,259P/0F EXIT=0）。双解冻条件不变：① 渲染流零新提交（活跃面维持
  inline walk 域，零子帧文档加载工作）；② DC-2 口径无新拍板记录。goal 自有面
  零新缺口、无扩展面（S40-S51 重审结论延续）。
- **S51（2026-09-13）静默轮 — 同 tip 复核（无代码变更，绿步维持 33）**：
  pull 零新提交（tip = 60dc402aa，即 S50 提交本身）——tracked 树硬核对：tip 为
  S48 组合态门验证树（a0a8146e2）之上仅 S49/S50 docs 增量，非 docs 面零变更，
  S48 全部门结论全量引用免复跑（cdp-e2e 门 PASS 33 绿 deterministic +
  make test 19,259P/0F EXIT=0）。双解冻条件不变：① 渲染流零新提交（活跃面维持
  inline walk 域，零子帧文档加载工作）；② DC-2 口径无新拍板记录。goal 自有面
  零新缺口、无扩展面（S40-S50 重审结论延续）。**push 窗口扰动归因（rule 10）**：
  首推 non-fast-forward，rebase 拉入 9825b8e54（CI bot benchmarks dispatch 自动
  记账，三文件全在 docs/perf/ 基线/趋势数据）——门禁图外路径（make test / cdp-e2e
  零消费 docs/perf/，零编译面），免复跑判定成立，S48 结论延续。
- **S50（2026-09-13）静默轮 — 同 tip 复核（无代码变更，绿步维持 33）**：
  pull 零新提交（tip = 02d716667，即 S49 提交本身）——tracked 树硬核对：tip 为
  S48 组合态门验证树（a0a8146e2）之上仅 S49 docs 增量，非 docs 面零变更，S48
  全部门结论全量引用免复跑（cdp-e2e 门 PASS 33 绿 deterministic +
  make test 19,259P/0F EXIT=0）。双解冻条件不变：① 渲染流零新提交（活跃面维持
  inline walk 域，零子帧文档加载工作）；② DC-2 口径无新拍板记录。goal 自有面
  零新缺口、无扩展面（S40-S49 重审结论延续）。
- **S49（2026-09-13）静默轮 — 同 tip 复核（无代码变更，绿步维持 33）**：
  pull 零新提交（tip = a48ed5178，即 S48 提交本身）——tracked 树硬核对：
  `git diff a0a8146e2..HEAD -- ':!docs'` 为空（S48 组合态门验证树起点，非 docs
  面零变更），S48 全部门结论全量引用免复跑（cdp-e2e 门 PASS 33 绿 deterministic
  + make test 19,259P/0F EXIT=0）。双解冻条件不变：① 渲染流零新提交（活跃面
  维持 inline walk 域，零子帧文档加载工作）；② DC-2 口径无新拍板记录。goal
  自有面零新缺口、无扩展面（S40-S48 重审结论延续）。
- **S48（2026-09-13）R4311-R4313 组合态门复核 — walk default-on 翻转后新 tip 全绿（无本流代码变更，绿步维持 33）**：
  第二次推送时 rebase 拉入渲染流三提交：R4312-F（**FLAT_CHILD_WALK default-on
  翻转** + 块子门，ZW_FLAT_CHILD_WALK=0 kill-switch 保留）+ R4311-F（SVG 特例门，
  与 R4312 同提交入账）+ R4313-F（空包装层形状守卫）——inline walk 域默认行为
  翻转，组合态最需复验的变更类。R4312 自带全套验证（reftest default 14762±flake
  + make test 67 套件全绿 + product-smoke 双变体 + bench-gate GATE PASS），但
  其上的 R4313-F 记录仅有 reftest 验证 → 按协议组合态复跑（rule 10：单树全绿
  ≠ main 全绿）。**新 tip（a0a8146e2）组合态门**：cdp-e2e 门 **PASS 33 绿
  deterministic 双跑一致**（绿步集与 S39 基线零漂移）；make test 全量
  **19,259P/0F EXIT=0**（与 S39 时点基线精确一致——零单测漂移、零回归；验证
  全程上游零漂移）。归因 rule 10：工作面 = layout-engine inline walk，与本流
  headless/CDP 面零重叠，default 翻转对 CDP 面零影响（门全绿实证）。
  双解冻条件不变：① 渲染流活跃面仍 inline/quotes 域（walk 常态化收尾）——零
  子帧文档加载工作，子帧能力维持冻结；② DC-2 口径无新拍板记录。goal 自有面
  零新缺口、无扩展面（S40-S47 重审结论延续）。
- **S47（2026-09-13）R4310-F 组合态门复核 — 推送时 rebase 拉入渲染流 walk 竖排子门（无本流代码变更，绿步维持 33）**：
  pull 零新提交（tip = 759be6716，即 S46 提交本身）；记录入档时点 tracked 树硬核对
  `git diff cbf705e32..HEAD -- ':!docs'` 为空；**推送时 rebase 拉入渲染流 R4310-F**
  （6dabee66e walk 竖排子门 + per-node vertical 信号通道——layout-engine inline 域
  + engine/paint text 微触 3 行，walk 仍 default-off，自带 reftest A/B walk-on/off
  总数精确持平 14763）→ tracked 树变化触发复跑（下一步计划 #3）。**新 tip 组合态
  门**：cdp-e2e 门 **PASS 33 绿 deterministic 双跑一致**（绿步集与 S39 基线零漂移）；
  make test 全量 **19,259P/0F EXIT=0**（与 S39 时点基线精确一致——渲染流零新增单测；
  并行 quickjs clippy 腿 rc 聚合同过）。归因 rule 10：R4310-F 工作面与本流零重叠
  （headless/CDP 面零触），零回归。双解冻条件不变：① 渲染流活跃面仍 inline/quotes
  域（R4310-F = walk 竖排子门，非子帧文档加载）——子帧能力维持冻结；② DC-2 口径
  无新拍板记录。未跟踪探针维持调试资产不入门禁图（S37 核查结论延续）。
  goal 自有面零新缺口、无扩展面（S40-S46 重审结论延续）。
- **S46（2026-09-13）静默轮 — 同 tip 复核（无代码变更，绿步维持 33）**：
  pull 零新提交（tip = 06f6f41f6，即 S45 提交本身）——工作树与 S45 时点逐字节一致，
  S39 门禁验证树结论全量引用免复跑（cdp-e2e 门 PASS 33 绿 deterministic +
  make test 19,259P/0F）。双解冻条件不变：① 渲染流零新提交（inline/quotes 域，
  零子帧文档加载工作）；② DC-2 口径无新拍板记录。goal 自有面零新缺口、无扩展面
  （S40-S45 重审结论延续）。
- **S45（2026-09-13）静默轮 — 同 tip 复核（无代码变更，绿步维持 33）**：
  pull 零新提交（tip = f7747357b，即 S44 提交本身）——工作树与 S44 时点逐字节一致，
  S39 门禁验证树结论全量引用免复跑（cdp-e2e 门 PASS 33 绿 deterministic +
  make test 19,259P/0F）。双解冻条件不变：① 渲染流零新提交（inline/quotes 域，
  零子帧文档加载工作）；② DC-2 口径无新拍板记录。goal 自有面零新缺口、无扩展面
  （S40-S44 重审结论延续）。
- **S44（2026-09-13）静默轮 — 同 tip 复核（无代码变更，绿步维持 33）**：
  pull 零新提交（tip = 2dfe7fc50，即 S43 提交本身）——工作树与 S43 时点逐字节一致，
  S39 门禁验证树结论全量引用免复跑（cdp-e2e 门 PASS 33 绿 deterministic +
  make test 19,259P/0F）。双解冻条件不变：① 渲染流零新提交（inline/quotes 域，
  零子帧文档加载工作）；② DC-2 口径无新拍板记录。goal 自有面零新缺口、无扩展面
  （S40-S43 重审结论延续）。
- **S43（2026-09-13）静默轮 — 同 tip 复核（无代码变更，绿步维持 33）**：
  pull 零新提交（tip = 5dc9e202a，即 S42 提交本身）——工作树与 S42 时点逐字节一致，
  S39 门禁验证树结论全量引用免复跑（cdp-e2e 门 PASS 33 绿 deterministic +
  make test 19,259P/0F）。双解冻条件不变：① 渲染流零新提交（inline/quotes 域，
  零子帧文档加载工作）；② DC-2 口径无新拍板记录。goal 自有面零新缺口、无扩展面
  （S40-S42 重审结论延续）。
- **S42（2026-09-13）静默轮 — 同 tip 复核（无代码变更，绿步维持 33）**：
  pull 零新提交（tip = 1afb2ec17，即 S41 提交本身）——工作树与 S41 时点逐字节一致，
  S39 门禁验证树结论全量引用免复跑（cdp-e2e 门 PASS 33 绿 deterministic +
  make test 19,259P/0F）。双解冻条件不变：① 渲染流零新提交（inline/quotes 域，
  零子帧文档加载工作）；② DC-2 口径无新拍板记录。goal 自有面零新缺口、无扩展面
  （S40/S41 重审结论延续）。
- **S41（2026-09-13）静默轮 — 同 tip 复核（无代码变更，绿步维持 33）**：
  pull 零新提交（tip = 18ba0fe39，即 S40 提交本身）——工作树与 S40 时点逐字节一致
  （含 docs），S39 门禁验证树结论全量引用免复跑（cdp-e2e 门 PASS 33 绿 deterministic
  + make test 19,259P/0F）。双解冻条件不变：① 渲染流零新提交（活跃面结论延续：
  inline/quotes 域，零子帧文档加载工作）；② DC-2 口径无新拍板记录（docs/goal 零新
  提交）。goal 自有面零新缺口、无扩展面（S40 缺口重审结论延续）。
- **S40（2026-09-13）静默轮 — S39 同 tip 复核（无代码变更，绿步维持 33）**：
  pull 零新提交（tip = cbf705e32，即 S39 门禁验证提交本身）——S39 全部门结论直接
  延续：cdp-e2e 门 PASS 33 绿 deterministic + make test 19,259P/0F + fmt/clippy，
  免复跑。双解冻条件不变：① 渲染流活跃面仍为 inline/quotes 域（R4308-F font 度量、
  R4309-N ruby 竖排探查），零子帧文档加载工作；② DC-2 口径无新拍板记录（docs/goal
  零新提交）。goal 自有面缺口重审：S39 探测切片的后续语义边界（子帧 url 停留
  about:blank = 最诚实语义、DocumentWriteSettled 路径探测缺失已有记账且无用例依赖、
  closeTarget 后残留分组为无害死数据）均不构成账本缺口、无扩展面。
- **S39（2026-09-13）子帧元数据探测 — frames.access 翻绿，绿步 32→33（代码切片，纯 apps/browser 面）**：
  **缺口重审**：S18「三件套跨流域」论证针对 frames×2 整体；逐步拆解发现 `frames.access`
  仅断言 `page.frames().length >= 2`——纯元数据面（PW 的 Frame 对象来自
  `Page.frameAttached{frameId,parentFrameId}` 事件，PW 1.63 coreBundle 实证：带
  parentFrameId 即建子 Frame；不带会误触发主帧 id 改写分支），无需子帧文档加载/渲染/
  realm——本流可单方解。
  **实现**：① 导航事件族（emit_navigation_event_family）在 domContent 前经既有
  automation_request 探测 `return String(document.querySelectorAll('iframe').length)`
  （**ExecuteScript 函数体语义须带 return——首版缺 return 探测恒 0**）；② 为每个
  iframe 发 frameAttached + 文档换代时对旧记录发 frameDetached{reason:frameRemoved}；
  ③ getFrameTree childFrames 从记录填充；④ 探测失败按 0 容错不阻塞事件族。
  **多 target 串扰修复（回归定位）**：首跑门禁 frames.access 绿但
  page.second.lifecycle/target.attachDetach 双回归（`newPage: Frame has been
  detached`）——raw-CDP 探针实证：`active_child_frames` 原为 session 级扁平记录，
  p1 的记录被 p2 的导航误 detach（串扰事件盖 p2 会话）+ p2 getFrameTree 读到幽灵
  child——PW 沿 parent 链找 per-frame session 失败即抛。修复：记录改按主帧 id
  （=targetId）HashMap 分组，detach/attach/getFrameTree 均只操作本页分组。
  **语义边界记账**：子帧 url 停留 about:blank、无子帧 frameNavigated（无子帧文档
  加载）；frames.click+evaluate 仍挂子帧文档+realm（真跨流域）。
  **工具坑**：进程内 webview（cfg(test)）缺 querySelectorAll 宿主绑定（`__zw_query_all`
  未注册）——单测覆盖容错路径，生产 attach 面由 cdp-e2e 门验证。
  **验证**：cdp-e2e 门 PASS **33 绿** deterministic 双跑一致（零回归，两处串扰回归
  修复后消除）；headless 单测 98P（+2：探测容错/文档换代 detach 语义）；make test
  全量 **19,259P/0F EXIT=0**（S32 基线 19,257 + 2 新单测精确吻合）；fmt clean +
  clippy -D warnings 全过。
- **S38（2026-09-13）静默轮 — 同 tip 复核（无代码变更、无 docs 变化轮，绿步维持 32）**：
  pull 零新提交（tip = ff841c435，即 S37 提交本身——工作树与 S37 时点逐字节一致，
  含 docs）。同 tip 同日 → S37 全部复核结论直接延续：① 渲染流近 14 天活跃面维持
  inline walk/flatten 系列 + quotes glyph 探针，零子帧文档加载工作（轻量复查确认）；
  ② DC-2 口径无新拍板记录（docs/goal 近 2 天仅本流 + 渲染流 R43xx）；③ cdp-e2e 门
  免复跑（tracked 树 = S32 门禁验证树，S37 硬核对过 docs/ 外零变更）。本轮零文件
  变化，控制面仅入档轮次记录。goal 自有面零新缺口、无扩展面。
- **S37（2026-09-13）静默轮 — 条件复核（无代码变更，绿步维持 32）**：
  pull 零新提交（tip = 2e05c12a3，即 S36 时点）；**树一致性硬核对**：`git diff
  d96254ac5..HEAD`（S32 门禁验证树起点）docs/ 外零文件变更——cdp-e2e 门免复跑结论
  （S32 时点 PASS 32 绿 + 全量 19,257P/0F）逻辑链闭合。双解冻条件不变：① 渲染流近
  14 天活跃面 = inline walk/flatten 系列（R4297-R4307）+ quotes glyph 探针 + 表格
  列宽，零子帧文档加载工作（iframe 命中项仍为 editing/web-components goal 的 realm
  面遗留，S27 同型归因）；② DC-2 口径无新拍板记录（docs/goal 近 3 天提交为本流 +
  渲染流 R43xx rendering-compat 记账）。goal 自有面零新缺口、无扩展面。
- **S36（2026-09-13）静默轮 — 条件复核（无代码变更，绿步维持 32）**：
  pull 零新提交（tip = c7429f83e，其上仅 S33-S35 docs 变更——S32 门禁验证树
  19,257P/0F + cdp-e2e 32 绿结论延续，免复跑）；双解冻条件不变（渲染流无新工作、
  DC-2 口径无新拍板）。goal 自有面零新缺口、无扩展面。
- **S35（2026-09-13）静默轮 — 条件复核（无代码变更，绿步维持 32）**：
  pull 零新提交（tip = e36047e02，其上仅 S33/S34 docs 变更——S32 门禁验证树
  19,257P/0F + cdp-e2e 32 绿结论延续，免复跑）；双解冻条件不变（零上游提交 → 渲染流
  无新工作；DC-2 口径无新拍板）。goal 自有面收口清单（evidence 复现链/README/账本
  引用/learning/dispatch 结构完整性）此前各轮已闭合，零新缺口、无扩展面。
- **S34（2026-09-13）静默轮 — S32 纯搬移结构完整性补证（无代码变更，绿步维持 32）**：
  pull 零新提交（tip = c8c534e87，其上仅 S33 docs 变更——S32 门禁验证树结论延续，免复跑）；
  双解冻条件不变。**补证**：拆分前（93cabd241 domains.rs）与拆分后（HEAD domains/）
  `cmd_` 方法集合逐一对账 **45=45 零差集**——与 S32 cdp-e2e 32 绿行为门互证，账本
  dispatch 表 ground truth 结构完整。goal 自有面零新缺口。
- **S33（2026-09-13）静默轮 — S32 后引用修正（纯 docs 切片，绿步维持 32）**：
  pull 零新提交（tip = d96254ac5，即 S32 全门禁验证树本身——19,257P/0F 与 cdp-e2e
  32 绿结论直接延续，免复跑）；双解冻条件不变（零上游提交）。**S32 收尾**：账本
  ground-truth 路径引用由 `domains.rs` 更新为 `domains/` 子模块（2 处，含 S17 时点
  注记）；master.md 内 2 处 `domains.rs` 为 S32 历史记录保持原样。goal 自有面零新缺口。
- **S32（2026-09-13）headless/domains.rs 超限拆分 — 按域 10 子模块纯搬移（代码重构切片，绿步维持 32）**：
  **动机**：S25 后 `apps/browser/src/headless/domains.rs` 达 **2010 行**，超 CLAUDE.md §5
  2000 行上限（S2 拆 headless.rs 同款约束驱动；本流独占面）。**拆分**：`domains.rs` →
  `domains/` 子目录 10 模块——mod.rs（dispatch/dispatch_with_events 路由）+ remote_object
  （remoteObject/objectId/PNG 助手）+ bidi（goal 前遗留面）+ runtime/dom/page/input/
  emulation/storage/target（CDP 各域）；结构感知纯搬移（按 fn 名映射、逐行零语义变化），
  子模块方法统一 `pub(super)`（可见域仍限 domains 子树）；`emit_navigation_event_family`
  升 `pub(in crate::headless)`、再导出行加 `#[cfg(test)]`——tests.rs 零改动。
  **文件大小审计附记**：`renderer/js_worker.rs` 3528 / `protocol/message.rs` 2366 同超限
  ——跨流/共享面，按 §9 碰头纪律只记档不动手，留待协调。
  **验证**：cargo check --all-targets 0E/0W；fmt clean + clippy -D warnings 全过；
  cdp-e2e 门 **PASS 32 绿 deterministic 双跑一致**（真实 PW 客户端端到端验证拆分零语义
  漂移）；make test 全量 **19,257P/0F EXIT=0**（与 S25 时点基线精确一致——纯搬移零新增
  测试）。**flake 归因记档（rule 10）**：前两轮全量分别 1F/4F（均为 wpt-runner
  testharness 时序型 Timeout，两轮失败集不同）——隔离复跑 0.98s PASS + pristine HEAD
  stash 对照（双方 testharness 35P/0F 一致）+ crate 零依赖 → 并行 sweep 负载 flake
  （S16/S25 同型），第三轮全量干净通过。
- **S31（2026-09-13）静默轮 — 条件复核 + 收口面回归核对（无代码变更，绿步维持 32）**：
  pull 零新提交（tip = 427cad098；自 S28 门禁验证代码树 359679334 以来仅 docs 变更
  S29/S30——门结论延续有效，免复跑）。双解冻条件不变（零上游提交 → 渲染流域无新工作；
  DC-2 口径无新拍板）。S28/29/30 三轮收口面复核零新缺口：evidence 复现链闭合、README
  与账本版本头一致、learning 引用零悬挂。goal 自有面（DC-1/3/4 + DC-2 第 3 条）维持
  ✅；余 DC-2 前两条全挂外部（口径拍板 / 渲染流子帧），无新信息、无扩展面。
- **S30（2026-09-13）经验资产收口 — S25 工具坑补 learning（纯 docs 切片，绿步维持 32）**：
  pull 零新提交（tip = 631522221，其上仅 docs 变更——S28 时点门结论延续有效，免复跑）；
  双解冻条件不变。按 CLAUDE.md 经验沉淀契约核查 master.md 两处 learning 引用：S17
  execFileSync 死锁已入库（patterns/2026-09-13-node-sync-child-exec-deadlocks...）✓；
  **S25 exceptionDetails 工具坑缺失** → 补
  `docs/learnings/bugs/2026-09/2026-09-13-pw-cdp-session-send-exceptiondetails-not-throw.md`
  （问题描述/根因/解决方案三段；语义分界成文：协议错误码=传输/参数/未实现，
  exceptionDetails=页面/V8/桥层失败；断言兼容双形状以账本 releaseObjectGroup 步实测
  形态为准）。`make learnings-index` 重建 INDEX（175 条，格式校验过）。goal 自有面
  learning 引用就此零悬挂。
- **S29（2026-09-13）控制面一致性收口 — README 过时项修正 + 账本版本头（纯 docs 切片，绿步维持 32）**：
  pull 零新提交（tip = 359679334，即 S28 门禁验证时点——门免复跑）；双解冻条件不变
  （零上游提交，渲染流域无新工作）。**README 三处过时修正**
  （tests/playwright-matrix/README.md 于 S8 前夜落地，此后未随切片更新）：① `ws` 版本
  8.18.3→**8.21.0**（lockfile 事实核对）；② 全核心流清单补 console 采集 + S25 raw-CDP
  4 步；③ 结构节补 S28 入库的 probe-s17-capture.mjs。**账本版本头 v0.4→v0.5**（S25
  补测行/审计节 + S28 复核节此前未随版本行体现）。其余控制面核对一致无需改动：
  expected-green.json 32 步、Makefile cdp-e2e 入口、goal 入口文档（按设计不变）。
  **验证**：docs-only 豁免路径（git diff --check + pre-commit guard PASS）；无 Rust/
  门禁面变化，S28 时点门 PASS 32 绿维持。
- **S28（2026-09-13）evidence 复现链闭合 — S17 捕获探针入库 + 组合态捕获复核（测试资产小切片，绿步维持 32）**：
  **缺口**：已入库 evidence（`zeroweb-capture-2026-09-13-summary.json`）的复现命令引用
  未入库脚本 `probe-s17-capture.mjs`（账本 L201 自注「不入 git」）——复现链断裂，「evidence
  账本持久化」存在可复现性缺口。**收口**：脚本原样入库（101 行，相对路径零硬编码；依赖
  `cdp-capture-proxy.mjs`/`capture-core-flow.mjs`/chromium 基线 summary 全部已入库；不改
  一字保留证据产出溯源），账本复现注记同步；**实跑验证**复现链可执行并取得当前 tip
  组合态捕获复核：41 方法全为账本「实现」态、chromium-only 缺口 5→3（S25 补测覆盖
  detachFromTarget/setUserAgentOverride，余 3 挂账有因）、事件类型 17 不变——**零未登记
  漂移**（明细见账本 S28 节）。其余未跟踪探针维持调试资产不入 git（S10 历史注记不变）。
  **验证**：cdp-e2e 门 PASS 32 绿 deterministic 双跑一致（tracked 树变化触发复跑，S27
  规则）；cargo fmt --check clean；clippy 见下（零 Rust delta）。
- **S27（2026-09-13）静默轮 — 双解冻条件复核 + 门免复跑（无代码变更，绿步维持 32）**：
  pull 零新提交（tip = c61520efb，tracked 树与 S26 门禁验证时点逐字节一致——cdp-e2e 门
  按 S26 预案**免复跑**）。双解冻条件复核（2026-09-13 实测）：① 渲染流域子帧能力仍冻结
  ——engine/layout-engine 近 14 天活跃面 = filter/svg/inline 布局修复，无子帧文档加载
  工作（git log 命中的 iframe 项为旧 editing goal 的 realm 面遗留；layout "frame" 命中
  为 CSS border-box 术语，非 HTML 子帧）；② DC-2 口径无新拍板记录（docs/goal 近 3 天
  提交全为本流）。控制面自洽核查：expected-green.json = 32 步与记录一致；门禁图核查——
  verify-deterministic/capture-core-flow 仅 import 已入库模块，scripts/ 下未跟踪探针
  不入门禁图（「免复跑」判定在存在未跟踪文件时仍成立）。DC-2 口径维持待用户决策，无新
  信息、无扩展面。
- **S26（2026-09-13）静默验证轮 — R4300-N 组合态门复核（无代码变更，绿步维持 32）**：
  pull 拉入渲染流 R4300-N（inline flatten 保子序列 walk 探针，layout-engine 单文件，
  **default-off** 零默认行为变化）；新 tip 上 cdp-e2e 门 **PASS**（32 绿、deterministic
  一致，零漂移）。DC-2 口径维持待用户决策，无新信息。
- **S25（2026-09-13）DC-1 覆盖审计 + 缺口补测 — 绿步 28→32（代码+测试资产切片）**：
  **审计（Mission 验收标尺「已实现的全部验证过」逐条核对）**：以 S17 ZeroWeb 捕获
  方法集 × 矩阵实现态清单做差集——PW 高层流只触达 35 方法，「已实现但高层流不调用」
  4 项无任何 e2e 触达：`Target.getTargets`/`Target.detachFromTarget`/
  `Runtime.releaseObjectGroup`/`Emulation.setUserAgentOverride`（S8 时点 DC-1 ✅ 判定
  基于绿步 6，此后命令面大扩未复审）。
  **raw-CDP 建会话面补齐（探针实证 PW 客户端真实依赖）**：`context.newCDPSession(page)`/
  `browser.newBrowserCDPSession()` 先后发送 `Target.attachToBrowserTarget`（建会话）+
  `Target.attachToTarget`（绑页面 target）——两命令此前 -32601，raw-CDP-via-PW 面
  完全不可用。实现：attachToBrowserTarget 分配 sessionId 登记活跃 target（flat 模型
  浏览器级/页面级同面）；attachToTarget 校验 targetId（-32602/-32000）→ 登记 →
  `{sessionId}` 无事件；detachFromTarget 命令发起的事件盖发起会话 sessionId（发起方
  flat 路由可达，closeTarget 广播保持无盖章）。2 个新单测。
  **4 个新 e2e 步全绿**（`target.getTargets`/`target.attachDetach`/
  `runtime.releaseObjectGroup`/`emulation.userAgentOverride`）：绿步 28→**32**，
  deterministic 双跑一致，expected-green 扩至 32（全量 34 步，frames×2 挂起不变）。
  **UA override 语义边界记账（探针三路对照实证）**：注入面 = proxy 子资源路径
  （img 随文档加载携带 override UA）；renderer 直连 fetch（ResourceLoader 观测路径）
  不经 override——Chromium 全请求语义差距，随 renderer fetch 管线统一时收口（账本
  「语义边界记账」注记）。
  **工具坑（learning）**：桥 miss 语义经 `exceptionDetails`（200 形响应）传回而非协议
  错误——PW `session.send()` 不 throw，断言须查 exceptionDetails。
  **验证**：cdp-e2e 门 PASS 32 绿（deterministic 双跑一致）；make test 全量
  **19,257P/0F EXIT=0**（基线 19,254 + R4298 +1 + 本轮 +2 新单测，精确吻合；首轮遇
  webview SW activation 1 flake——隔离复跑 0.06s PASS、webview 全包 712P/0F，归因
  渲染流 reftest 重负载并发，复跑干净）；fmt clean + clippy -D warnings 全过。
  **R4299-N 组合态复核**：push 时 rebase 拉入渲染流 R4299-N（inline 空 span 水平
  padding 推进，layout-engine 单文件，零工作面重叠）——新 tip 上 cdp-e2e 门复跑
  **PASS 32 绿** deterministic 一致，rule 10 归因闭合。
- **S24（2026-09-13）静默轮 — 卡点通报 + 同树门复跑（无代码变更，绿步维持 28）**：
  两流零新提交（main = S23 docs 提交），代码树与在档 PASS 时点逐字节一致；cdp-e2e 门
  复跑 **PASS**（28 绿、deterministic 一致，零漂移）。**按 run-rules #7 飞书通报 DC-2
  口径卡点**（goal 收口仅剩：用户拍板口径——分支 B 可先行 DONE；或渲染流子帧能力解冻
  解 frames×2）——**后续轮次无新信息不重复通报**。无扩展面、无新信息。
- **S23（2026-09-13）例行验证轮 — R4298-F 后组合态 cdp-e2e 门复核（纯验证切片，绿步维持 28）**：
  渲染流 R4298-F（auto 表格列宽压缩 + cell 重排，layout-engine 域）落在 S22 全量刷新之后
  18 分钟，本流补跑自有回归门：**cdp-e2e 门 PASS**（28 绿、deterministic 双跑一致，与
  S22 基线零漂移）。全量 make test 未重复——R4298-F 提交在**同一棵树**（29caa06d4 直接
  子于本流 S22 提交 5a2352600）自带全量验证：make test 19,255P/0F（+1 R4298 单测）+
  fmt/clippy clean + product-smoke 全 fixture struct PASS + 定向 bench-gate GATE PASS
  （归因渲染流，rule 10；本流自 S22 零代码变更）。**挂起理由复核**：engine 近 7 天 =
  R4297-F inline border-box / R4296-N bleed / R4293 filter / R4291 svg，无子帧文档加载
  工作——frames×2 维持挂起；DC-2 口径维持待用户决策，无新信息、无扩展面。
- **S22（2026-09-13）例行验证轮 + R4297-F 后组合态全量刷新（纯验证切片，绿步维持 28）**：
  渲染流 R4297-F（inline border-box 几何重写）落在 S18 全量基线**之后**，组合态此前
  未做全量验证——本轮补齐（rule 10 归因纪律）。**结果**：cdp-e2e 门 PASS（28 绿、
  deterministic 双跑一致）；make test 全套 **19,254P/0F EXIT=0**（S18 基线 19,251 →
  +3 为并行流新增测试，零失败零跨流回归）；renderer lib 红灯维持 161P+2 已知
  form fixture（S16 归因不变）。**噪音定性（防后续轮次重复排查）**：`cargo build
  -p zero-browser` 日志出现 `match_media_to_json` dead_code 警告——单包构建 feature
  解析所致（script-runtime 不在 zero-browser 单包图内启用 → engine callbacks.rs
  子模块不编译 → 非 ctx 版函数仅剩测试引用）；CI/本地门禁 `--workspace` 全量统一
  feature 后该函数有调用方（quickjs 注册路径），已实证
  `cargo clippy -p zero-engine --lib`（默认 feature）PASS——**非门禁问题，不修**
  （共享面 engine 碰头纪律，无门禁影响）。DC-2 口径维持待用户决策，frames×2 维持
  挂起（渲染流近 14 天活跃面 = filter/svg/inline 布局修复，无子帧文档加载工作，
  2026-09-13 复核）。
- **S21（2026-09-13）DC-2 连接生命周期健壮性实证（验证切片，绿步维持 28）**：
  **实证**（真实 PW 客户端 + 裸 WS 探针）：① 顺序重连 ×3（connect → newPage →
  setContent → locator → close 循环）全通、无状态残留；② 异常断开 ×3（裸 WS 发一条
  命令后不发 close 帧 RST 直断）全部被吸收——transport read error 分支 break 内循环 →
  外循环接受下一连接，无进程崩溃、无句柄悬挂；③ 断后新连接完全可用 + `/json/version`
  存活。**DC-2「连接生命周期健壮（重复连接、异常断开）」就此验证**。
  **限制记账（非缺陷，结构注记）**：transport 为单连接 serve loop（一次服务一个 WS
  连接）——第一客户端存活期间第二并发客户端在 TCP backlog 等待（不报错不泄漏，仅
  不可用）；Chromium 支持多并发 CDP 客户端。多客户端多路复用需 HeadlessSession 共享
  化（Arc<Mutex> 重构）= 结构变更，按需立项（Playwright 典型用法单客户端；e2e 门不受
  影响）。
- **S18（2026-09-13）M5 定稿预案 + 子帧缺口实证（纯文档/验证切片，绿步维持 28）**：
  **子帧缺口探针实证**（挂起理由从假设升级为实测）：iframe 元素存在但
  `contentDocument`=null（引擎不加载子帧文档，无子帧 DOM）、`contentWindow`=object
  （stub）、`page.frames()`=1（无 frameAttached 事件源）——frames×2 需引擎子帧文档
  加载 + 子帧渲染面（子文档布局/iframe 区域绘制/child quads 坐标，属 layout-engine/
  paint 渲染流域专属 crate）+ 子帧 JS realm——三件套均跨流域，本流不可单方解。
  **M5 定稿预案**（双分支机械执行清单，见下一步计划 #2）：分支 A（等 30/30，维持
  门禁防回归）／分支 B（挂账剔除定稿，四步全 docs 一个提交）——DC-2 口径一决即执行。
  **全量基线刷新**：make test 全套经 test-guard（结果见验证基线）。
  **引擎碰撞核对**：`git log --since="14 days ago" -- crates/engine/` = 渲染流 paint
  域修复（R4285-R4296 filter/svg/bleed），无子帧相关工作——维持挂起不变。
  **跨流红灯跟踪**：renderer lib form fixture 2 失败仍在（S16 时点归因不变）。
- **S17（2026-09-13）Network dataReceived + 矩阵账本 v0.4 漂移刷新（绿步维持 28）**：
  **dataReceived**：protocol `FetchObservedParams` 增 `data_length`（末位追加）；renderer
  fetch 观测记录扩为六元组（loadingFinished 阶段带 body 字节数——`body_bytes` 原始字节
  优先、文本回退）；headless 双路径在 loadingFinished 前发 `Network.dataReceived`——
  proxy 子资源路径（session proxy_fetch，body 同步在握）+ renderer 观测路径（JS
  fetch/XHR）。**body 一次性到达语义**（dataLength=encodedDataLength=body 字节），
  分块流式随 net 观测点流式化（记账）。两类流量本就分路（子资源=proxy、页面
  fetch=ResourceLoader 直连观测），无重复发射。
  **账本 v0.4**：evidence/cdp-command-matrix.md 逐行以 domains.rs dispatch 表为 ground
  truth 核对——loadEventFired（雏形→✅ S5+S16）、lifecycleEvent（补 S16 重发）、
  setLifecycleEventsEnabled（门控语义记账）、executionContextDestroyed（❌→不实现-ok，
  contextsCleared 覆盖）、dispatchKeyEvent（S16 accel+A 注记）、dialog 行（S10 澄清）、
  dataReceived 行（✅ S17）、G4 请求事件总线（雏形已建）。
  **验证**：cdp-e2e 28 绿 deterministic 双跑一致；browser bin 449P/0F；integration
  781P/0F；renderer lib 161P+2 已知跨流失败（无新增）；clippy -D warnings + fmt 全过。
  **M5 实测复核（同切片）**：ZeroWeb 侧经捕获代理重跑全核心流——429 调用/35 方法 vs
  Chromium 基线 395/40（+34 调用=frames 失败重试放大）；35 个被调方法全部为账本「实现」态，
  **命令面与账本登记零漂移**（5 个 chromium-only 方法全部挂账有因：getFrameOwner=frames
  挂起下游、handleJavaScriptDialog=无事件源、setFontFamilies=-32601 容忍、
  detachFromTarget/setUserAgentOverride=drift 记账）；证据
  evidence/zeroweb-capture-2026-09-13-summary.json。工具坑：execFileSync 冻结父进程事件
  循环致父内嵌代理 × 子进程消费双向死锁——异步 spawn 解（learning 2026-09-13）。
- **S16（2026-09-13）keyboard Ctrl+A 编辑面 + document.open/write/close（绿步 26→28）**：
  **keyboard.type+press**：`Control+a` 此前被当普通可打印键注入 `'a'`（实测值
  'abca'）。修复：`apply_keydown_default` 增 `accel` 形参（CDP dispatchKeyEvent 路径传
  `ctrl||meta`；DispatchDomEventParams 路径无修饰键字段保持 `false`，协议不动）——
  accel+A 命中可打印分支时改走 `apply_select_all_at`（复用指针选区路径
  `set_pointer_text_selection` → shim setSelectionRange，UTF-16 偏移口径；非文本控件
  no-op）。
  **page.setContent（四层落点）**：① shim part06 `document` 补 `open/write/writeln/close`
  三连（PW setContent 在 utility world 执行 `open(); console.debug(tag); write(html);
  close();`——三函数此前缺失 → TypeError）。简化语义：open 清 body + 起缓冲、write 缓冲、
  close 把缓冲作 body innerHTML 一次性应用（live host 解析+重排版，探针验证查询/读回
  可达）；head/title 剥离、unload、隐式 open 未建模（FIXME 记档）。② **console tag 时序**：
  PW 在 tag console 消息到达时 `_onClearLifecycle()` 清 `_firedLifecycleEvents` 再等新
  'load'——headless 逐命令排空此前 network 队列先于 console 队列，load 族先到被清 → 挂起。
  修复：排空序改 console → network/Page（与空闲期 drain 一致）。③ **load 生命周期重发**：
  spec close() 解析结束触发 load（软导航语义）——新增 protocol `DocumentWriteSettled`
  （renderer → headless 单向事件，末位追加；shim close() 经 `__zw_document_write_settled`
  回调 → js_worker 共享队列 → runtime 尾 drain）→ headless 重发
  `Page.lifecycleEvent{DOMContentLoaded,load}` + `domContentEventFired`/`loadEventFired`
  （不发 frameNavigated/contextsCleared：文档对象与 JS context 未换代）。④ 清理 S14 残留
  诊断（headless 两处 println + runtime tick 内 /tmp 文件写——println 污染即 S12 事故根因类）。
  **验证**：绿步 26→28；deterministic 双跑一致；expected-green 基线扩至 28；
  integration 781P/0F（全仓一轮中 network_loading 单测并行负载下偶发 1 失败、隔离与整包
  重跑均绿，非本切片回归）；余 2 步 = frames×2（挂 engine 子帧可见性）。
- **S12（2026-09-13）hit-test 溢出剪枝修复 + CDP 空闲期 renderer 通道 drain**：
  **根因定位（插桩 PW coreBundle 注入诊断 + 点阵探测）**：`#btn-fetch` 点击失败的真因是
  **引擎 hit-test 溢出剪枝**——`deepest_node_at`/`collect_nodes_at` 对「祖先盒不含点」整棵
  剪枝，而裸页 body 盒高仅 6px（gBCR 实测 [8,8,784,6]）容不下 24.6px 的按钮 → 按钮在自身
  中心 `elementFromPoint` 返 html 兜底（点阵探测：按钮盒内仅 y∈[8,11] 命中，其余全 html）
  → PW `setupHitTargetInterceptor` 的 preliminary check 返回 `<html>` description → 无限
  重试。**修复**：hit-test 走树不再按包含剪枝（下探全树、仅记录含点的盒）——溢出内容
  （overflow:visible）可命中，与真浏览器绘制盒命中语义对齐；overflow:hidden 裁剪语义
  未建模（FIXME 记档）。**验证**：点击已真实落地（btn-fetch handler 的 fetch 触达测试
  服务器 API，apiHits=1）。
  **第二层（新发现，未解）**：点击落地后 PW click action 仍不完成——**host-dispatched
  listener 内的 fetch promise 不落定**（handler 内 `fetch()` 的 `.then` 链不执行，
  `__fetched` 恒 null；独立 evaluate 的 fetch 正常）——疑 FetchBridge 在宿主派发事件
  的 execute 内同步 resolve 的**重入死锁**（嵌套 sandbox.execute）。**第三层**：CDP 空闲
  期 renderer 通道无人消费（fetch 的 FetchRequest/console IPC 饿死）——已修：transport
  WS read 改 120ms 轮询 + `drain_renderer_channel`（fetch 代理 + console/network 事件
  即时推送，600s 空闲 deadline 语义保持）。
  **第四层（S13 定位+修复）**：fetch settle 路径 `__zwServiceWorkerFetchSettled →
  ensureDocument → __zw_sw_controller` 走 SW IPC client 同步等待（20s 超时），headless
  从不应答 `ServiceWorkerRequest` → JS worker 挂 20s、PW click 10s 超时。修复：headless
  `handle_renderer_message` 应答 SW 请求（Controller→无 controller、GetRegistrations→空、
  StateChanges→空、写类→NotFound——headless 无 SW 支持=正确语义）。
  **fetch 观测管线（FetchObserved IPC + renderer 观测 handler）已实现后回退**：队列 Arc
  双实例错接 + 诊断期 println 污染 IPC 帧流导致 renderer 通道崩溃（12 步回退事故）；
  已全部回退至 S12 等效状态，观测管线待独立切片以正确队列所有权重做。
  绿步维持 25（无回退）；deterministic 双跑一致。
- **S11（2026-09-13）console value-only 小切片 + emulation.media 接线**：
  **console.collect（P5 降级方案落地）**：shim `_zwConsoleEmit` 增逐参值序列化
  `_zwSerializeConsoleValue`（string/number/boolean 原样、undefined 标记串、对象 JSON
  round-trip）→ `__zw_console_log(level, text, args_json)` 三参（tracing 面不变）；
  renderer js_worker 后注册覆盖引擎回调（last-wins）推共享队列 → runtime 主循环 +
  脚本执行尾 drain → IPC `ConsoleLog`（protocol 末位追加）→ headless session
  `pending_console_events` → transport 逐命令盖章 `Runtime.consoleAPICalled`
  （value-only remoteObject args、executionContextId=1、level→CDP type 映射）。
  **时序要点**：console 事件须先于 AutomationResponse 转发（run_page_context_script 尾
  drain），否则 headless 在响应后才收到、要等下一条命令才排空（实测单命令消费面失效）。
  **emulation.media**：engine `match_media_to_json_ctx`（MediaContext 用户偏好注入）+
  renderer `MediaBridge` 重注册 `__zw_match_media`（共享 cell——SetColorScheme/
  SetMediaType 更新 prefers_color_scheme/media_type）→ matchMedia 读回真值。
  **绿步 23→25**（console.collect + emulation.media 翻绿）；deterministic 双跑一致；
  expected-green 基线扩至 25。
- **S10（2026-09-13）click hit-target 修复 — 合成输入事件面 + 视口真值**：
  S9 后 click 族卡「PW hit-target 拦截器判 `<html> intercepts pointer events`」，三层实测定位：
  ① PW `_hitTargetInterceptor` 读 `event.clientX/clientY` 复核命中点——宿主合成鼠标事件走
  `_makeEvent` 泛型面无坐标（undefined → elementFromPoint(undefined) → null → documentElement
  兜底）；② renderer `handle_mouse_event` 对 mousemove 直接跳过派发——拦截器挂 document
  mousemove 捕获收不到事件。修复：`DomEventDetail` 增 `client_x/client_y`（engine script_gen）
  + shim `__zw_dispatch_event` 新增鼠标类型分支（`new MouseEvent` 带 coords/click detail，
  UI Events §MouseEventInit）+ renderer 坐标随事件注入 + mousemove 照常派发（未命中目标时
  dispatch_dom_at 内部 no-op）。**click 事件保持泛型 Event 不入鼠标分支**——R108 pre-click
  activation/取消回滚协议与宿主激活事务（execute_shared_action）的 checked 翻转/取消语义按
  旧路径协作（实测：click 改 MouseEvent 会双重翻转 checked 且破坏三宿主 conformance）。
  ③ `screenshot.fullPage`：shim innerWidth/
  innerHeight 缺省 1280x800 与真实视口失配（PW `_fullPageSize` 以 scrollWidth 族测量）——
  js_worker 增 `SetViewportHint`（renderer 启动/SetViewport 时注入，快照换代后幂等校正）。
  **Playwright 绿步 17→23**（click.button/dblclick/withPosition + dialog.accept/confirm+prompt
  + screenshot.fullPage 翻绿）；deterministic 双跑一致；expected-green 基线同步扩至 23。
  诊断资产：tests/playwright-matrix/scripts/{raw-min,debug-zw-pw}.mjs（不入 git 调试脚本：
  局部复现 + 捕获代理 + 事件字段探测）。
- **S9（2026-09-13）objectId 全量 remoteObject 桥 — Runtime/DOM 域句柄面（用户拍板全量面）**：
  protocol `AutomationValue::Handle(AutomationHandleRef{id,node})` + 四操作
  `EvaluateRetaining/CallFunctionOnHandle/ReleaseHandle/ReleaseObjectGroup`（含 serde 契约
  测试）；renderer 侧 JS 句柄注册表（页面 context 全局单例、65536 上限、objectGroup 分组、
  primitive 按值/对象保留双尾；**导航换代经 `sandbox.reset_context` 整体失效 = CDP context
  destroyed 语义**，无需显式清理）+ `awaitPromise` 有界轮询（execute 边界 microtask drain +
  宿主 timer 泵，8s 超时）；headless Runtime.evaluate 双分支统一走桥（**表达式语义修复**：
  W3C ExecuteScript 是函数体语义、CDP evaluate 是表达式形态——裸表达式旧恒 undefined，
  PW 全管线的真实根因）+ callFunctionOn objectId（`arguments[].objectId` 实参顶层还原 +
  falsy 实参标记误判修复）+ `releaseObject/releaseObjectGroup` + **DOM 域 objectId 面**
  （scrollIntoViewIfNeeded/getContentQuads/getBoxModel/describeNode/resolveNode——经句柄桥
  对保留元素求值，rect 来自 shim gBCR/RectBridge 真实布局；`backendNodeId`=句柄 id，
  resolveNode 重保留新句柄支撑 PW adopt 流程）+ shim has-trap 白名单补
  nodeName/nodeType/tagName/validity 族/value（PW queryEngine `"nodeName" in element` 断言面）
  + 嵌套值纯 JSON 保真（remoteObject 嵌套不再包 type/value 外壳——PW `{o:[...]}` 线格式）。
  remoteObject 句柄形态带 `subtype:"node"`（PW ElementHandle 分叉点）。
  **Playwright 绿步 6→17**（evaluate 全族 5 步 + title + fill + locator.boundingBox +
  setContent 面前移 + screenshot.element + page.second.lifecycle + viewport.verified 翻绿）；
  deterministic 双跑一致；make test 19,238P/0F；workspace clippy -D warnings 全过。
  余 13 步根因已定位（见下一步计划）。
- **S8（2026-09-12）M5 收口预备 — cdp-e2e 门 + DC 盘点**：
  `make cdp-e2e` 入口落地（test-guard 包裹，spawn 独立 headless 双跑全核心流）：
  **deterministic 双跑一致**（两次入口运行均 YES）+ **expected-green 回归门**（6 步基线
  `expected-green.json`：context.default/page.new/setViewportSize/goto/cookies.roundtrip/
  screenshot.viewport，任一回退即门禁失败）。DC-1~4 盘点（见下）。CI 可行性记账：
  node 20.19 本机在位、playwright 缓存 chromium-1243 命中（npm install 仅装
  playwright-core+ws 两个包，lockfile 离线可复现）、CI 需 pre-step `npm ci` +
  `cargo build -p zero-browser`；CI 集成等 goal 收口时随 M5 定稿评估。
  **DC 盘点**：DC-1 ✅（账本 40 方法三态全登记 + goal 扩展面）；DC-2 ⏳（绿步 6/30，
  双跑 deterministic ✅ 已门禁化，全绿挂 objectId 桥决策）；DC-3 ✅（-32601/-32700/
  -32602/loopback/token-origin 语义保持；**超大 payload 实测**：100MB 消息触发
  tungstenite 16MB 帧上限干净拒绝（`Message too long` + 连接断开），服务器存活、
  后续连接正常——安全拒绝语义成立）；DC-4 ✅（make test 全绿 + clippy/fmt +
  cdp-e2e 门 + BiDi 既有面零回归）。
- **S7（2026-09-12）M4 — Storage cookie 域 + UA override + Network 事件总线雏形**：
  session 级 `CookieStore`（net 既有 jar 复用，goal 支持包络「net 只加观测点」——新增
  只读 `CookieStore::all()`）；`Storage.getCookies/setCookies/clearCookies` 实义
  （url/domain 作用域 + expires/secure/httpOnly，CDP cookie 形状）+ proxy_fetch 双向
  接线（Set-Cookie 捕获 + Cookie 请求头注入）→ **Playwright cookies.roundtrip 绿**；
  `Emulation.setUserAgentOverride`（proxy_fetch 注入 User-Agent）；`Network.enable/
  disable` 真实门控 + proxy_fetch 生命周期事件（requestWillBeSent/responseReceived/
  loadingFinished，session 盖章排空）——P6 net 观测点雏形。**Playwright 绿步 5→6**。
  console 对象化（P5）挂起：需 engine 宿主回调签名扩展（engine 为并行流活跃面，
  碰头管理延后）。make test 全绿（+6 M4 单测）。
- **S6（2026-09-12）M3 — viewport 桥 + 媒体仿真 + CDP 截图形状**：
  `Emulation.setDeviceMetricsOverride` 实义（→renderer SetViewport IPC + 服务器视口状态
  联动 getLayoutMetrics/captureScreenshot + Page.frameResized 事件；宽高 0=恢复默认；
  实测 page.setViewportSize 绿）；`setEmulatedMedia` 实义（prefers-color-scheme→
  SetColorScheme、media type→SetMediaType）；`Page.captureScreenshot` CDP 形状修正
  （`{data:"<b64>"}` 字符串形——此前对象形致 PW screenshot 直接报错，BiDi 对象形不动）
  + clip 原始 fb 行级裁剪（实测 screenshot.viewport 绿）。**Playwright 绿步 4→5**。
  挂起记档：iframe 子帧事件源需引擎子帧可见性（渲染流域协调），frames.access 步骤
  随引擎能力。make test 全绿（+5 M3 单测）。
- **S5（2026-09-12）M2 — 导航事件族 + Input 域 + getLayoutMetrics**：
  `Page.navigate` 实义化（`{frameId,loaderId,errorText?}` 形状 + Chromium 时序导航事件族
  frameStarted/StoppedLoading→frameNavigated→executionContextsCleared→新文档 context→
  domContent→load）；`addScriptToEvaluateOnNewDocument` 真执行 + 跨导航重放 + worldName
  登记/新文档 world context 重发（**实测修复 title/evaluate 在导航后永久挂起**——PW 的
  `utilityContext()` 等待 world context 事件）；`Page.getLayoutMetrics`（固定视口映射）；
  `handleJavaScriptDialog` stub；**Input 域全接**：dispatchMouseEvent（→renderer
  MouseEvent/ScrollEvent，released 按 clickCount 合成 Click/DblClick）、dispatchKeyEvent
  （Down/Up/Press + modifiers 位解码）、insertText（→ImeEvent Commit）——裸 API 实测
  keyboard.press/type、mouse click/wheel 全通。**Playwright goto 核心流绿**（47ms）；
  全流 30 步无挂起（此前 title/evaluate 挂起根因即 world context 缺失）。make test
  19,194P/0F（+10 M2 单测）。
- **S4（2026-09-12）M1 切片 3 — Target/Browser/Runtime 域 + Playwright 首连**：
  `Target.setAutoAttach`（flatten，浏览器级附接全部 page target + attachedToTarget 事件，
  会话级正确语义=仅子 target→无事件）/ `getTargetInfo`（浏览器级+按 targetId）/
  `createTarget`（autoAttach 自动附接）/ `closeTarget`（targetDestroyed+会话摘除）/
  `detachFromTarget` / `getTargets` CDP 形状修形（targetInfos）；`Browser.getVersion`/
  `setDownloadBehavior` stub；`Runtime.enable`（executionContextCreated + auxData 契约）/
  `Runtime.evaluate`+`callFunctionOn`（无 objectId 路径，renderer 类型化 AutomationValue →
  remoteObject returnByValue）/ `runIfWaitingForDebugger`；`Page.enable`/`getFrameTree`
  （**主 frame id = targetId**，CDP 硬契约）/`createIsolatedWorld`（utility world 记账）/
  init 命令族 stub；**ServerEvent 增 sessionId 盖章**（session 级事件客户端路由必需，
  Target 宣告事件除外）。**Playwright connectOverCDP 首连成功**：connect/attach/
  context.newPage 全通；evaluate 执行到 utilityScript 句柄处暴露 objectId 桥缺口
  （→ 待用户决策）。实测确认三条 CDP 硬契约：主 frame id=targetId、session 级事件必带
  sessionId、executionContextCreated 必带 auxData.frameId/isDefault。
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

1. **M5 收口评估（绿步 33/34，余 1 步）**：`frames.access` 已解（S39 子帧元数据探测，
   本流单方落地）；余 `frames.click+evaluate` 真挂子帧文档加载 + JS realm + child
   quads（S18 三件套论证对其成立）——渲染流域真协调。DC-2 口径决策：等子帧能力解冻后
   34/34 收口，or 以「挂账 + 口径剔除」先定稿（见待用户决策）。**实测复核已过**（S17：
   35 被调方法零漂移；S25：DC-1 覆盖审计缺口 4 项已补齐，实现态命令 e2e 全覆盖）——
   DC-2 口径一决即可定稿。
2. **M5 定稿（口径确定后）**：expected-green 基线定稿 → cdp-e2e 即 DC-2 门；挂账清单
   （不实现域）终稿；CI 集成可行性随收口评估（S8 记账：node 20.19 + lockfile 离线可复现）。
   **定稿预案（S18 预备，双分支机械执行）**：
   - **分支 A（等 30/30）**：goal 维持 Active；每轮门禁防回归；渲染流域子帧能力落地后
     解 frames.click+evaluate → 基线扩 34 → DC-2 ✅ → M5 定稿。挂账清单不豁免 frames 项。
   - **分支 B（挂账剔除定稿）**：① 矩阵账本「ZeroWeb 侧实测捕获」节加口径注记（frames×2
     记「挂账：随引擎子帧能力，M5 定稿时点不阻收口」）；② master.md 里程碑 M3/M5 改
     ✅（口径挂账注记）；③ goal 入口文档 DC-2 行加挂账口径注记（不改判定语义原文，仅
     注记）；④ expected-green 基线维持 28 不动（frames 步骤继续跑、不门禁）；⑤ CI 集成
     评估出结论记账。四步全 docs，一个提交。
3. **持续推进**：每轮 pull → cdp-e2e 门防回归（基线 **33** 步；**免复跑条件**：pull 后
   HEAD 未变且 tracked 树无变更——S26 验证过的 tip 可引用其结论，S27 复核未跟踪探针
   不入门禁图）；tracked 树变化时门 + make test。余项按窗口逐个解冻。

**待用户决策清单**：
- **DC-2 收口口径（2026-09-13 新入，S39 后语境收窄维持）**：余 1 步（frames.click+evaluate）
  真挂子帧文档加载 + JS realm + child quads——S18 探针实证：`iframe.contentDocument`
  为 null（引擎不加载子帧文档）；且子帧**渲染面**属 layout-engine/paint——渲染流域专属
  crate，本流不可单方解（frames.access 元数据面已于 S39 本流单方解）。「等子帧能力后
  34/34 收口」vs「挂账剔除先定稿」。口径不清则 M5 无法判定完成（**M5 定稿预案见下，
  口径一决机械执行**）。
- ~~dialog 事件源~~ **绿步已过、语义挂账（S10 现状澄清 2026-09-13）**：dialog.accept/
  dialog.confirm+prompt 绿因**引擎无阻塞对话框语义**——shim alert no-op、confirm/prompt
  立即返回（无 javascriptDialogOpening 事件、无挂起）、`Page.handleJavaScriptDialog` 为
  stub——步骤「不挂起即过」。真对话框事件面（引擎阻塞语义 + 事件源）仍属跨流域立项，
  不阻 M5 收口（PW 消费面绿）。
- ~~objectId 句柄桥~~ **已拍板（2026-09-12）：全量 remoteObject 桥**——✅ S9 落地。

**维持挂起**：iframe 子帧内容面（frames.click+evaluate 1 步）——渲染流域真协调
（子帧文档加载/渲染/realm 三件套），rendering 流 R41xx-R43xx 高频活跃，维持挂起合理。
（frames.access 元数据面已于 S39 解除挂起——纯 headless 面落地。）

**跨流红灯记录（S16 时点归因，非本流）**：`cargo test -p zero-renderer --lib` 2 失败
（page_scripts::tests::form_interaction_fixture_complete_sequence /
…_dispatches_idless_reset_and_submit_buttons——`apply_reset_on_click` 断言）。stash 验证
干净树同败（S16 变更无关），疑似 `514f07b29`（tick_observers per-task 重构）或渲染流
4fed099dc 组合态引入——归 event-loop-spec/渲染流修，本流不碰 page_scripts.rs 工作面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — 传输/发现/Target 基座 + Playwright 首连 | ✅ S9 收口：连接面 + evaluate 全族（literal/function/withArgs/object/async）+ releaseObject(Group) 全通 |
| M2 — Page/Input 域 → 点击/填充/键盘/导航流 | ✅ S16 收口：goto/title/fill/click 全族/dialog/键盘 type+press（Ctrl+A 全选）/导航事件族全绿 |
| M3 — DOM/CSS/Emulation → locator 流 | 🚧 S39：locator.boundingBox/viewport/媒体/截图 clip+element+fullPage/frames.access 绿；iframe 内容面维持挂起（frames.click+evaluate） |
| M4 — Network/cookies/console 对象化（cookie 落点=Storage 域） | ✅ S17 收口：cookie 域 + UA override + Network 事件族（含 dataReceived，S17）+ consoleAPICalled（value-only）绿；分块流式观测记账 |
| M5 — 矩阵收口 | 🚧 绿步 33/34（expected-green 基线同步扩至 33，S39）；余 1 步挂子帧文档+realm（DC-2 口径待决策） |

## 验证基线

- 测试基线：立项时点全绿（`make test` 19,170P/0F，2026-09-12 变基后口径；S9 后
  19,238P/0F；S18 全量刷新 19,251P/0F；S22 组合态刷新 19,254P/0F EXIT=0；
  S23 时点 19,255P/0F；S32 时点 19,257P/0F；**S39 时点 19,259P/0F EXIT=0**
  （+2 子帧探测单测，精确吻合）；禁止裸跑 cargo test，经 test-guard。注：make test
  的 workspace 腿 exclude zero-renderer——renderer lib 单测不在全量门内，跨流红灯
  （form fixture×2）经显式 `-p zero-renderer --lib` 跟踪）
- **CDP E2E 基线（S39，2026-09-13）**：绿步 33/34，deterministic 双跑一致，
  expected-green 基线 33 步（S39：frames.access 翻绿——子帧元数据探测；余
  frames.click+evaluate 挂子帧文档+realm）
- CDP 现状：`Page.navigate` / `Runtime.evaluate` / `Target.getTargets` 3 命令 +
  `/json/version` + `/json` 发现（headless.rs L782-796/L571/L1159）——历史基线，现行面
  见缺口清单 P3/P4 与切片记录
- **命令矩阵捕获基线（S1，2026-09-12）**：playwright-core 1.63.0 @ Chromium 153.0.8010.12
  （chromium-1243 缓存），全核心流 30 步全绿，395 调用/40 方法/30 事件；
  `evidence/chromium-capture-2026-09-12.md` + `…-summary.json`（生成物，复现命令见账本头）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过；
  Playwright E2E 用例须双跑 deterministic 才计入账本「绿」

**碰撞管理**：碰 `apps/browser` 前先 `git log --since="14 days ago" -- apps/browser/`
核对 android-browser 活跃面。
