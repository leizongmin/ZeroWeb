# 从 329 行 hobby commit 到独立浏览器引擎：Ladybird 七年演进、架构分层、工程坑实录与 WPT 驱动质量体系，及对 ZeroWeb 的启示

> 调研日期：2026-08-06 ｜ 调研对象：Ladybird（LadybirdBrowser/ladybird，含前身 SerenityOS 时期）
> 调研方式：多源交叉验证（官方博客/新闻稿、GitHub 仓库与 API、WPT 数据源、媒体与社区），重要结论 2+ 来源佐证
> 关联：本报告为 `/deep-research` 产物，可对接 `lei-spec-rfc` 流程进入需求规格与技术设计阶段

---

## 30 秒速览

Ladybird 是一个「从零自研」的独立浏览器引擎项目：HTML/DOM/CSS/布局/JS/WASM 引擎全部自研，不借用 Chromium/WebKit/Gecko 任何代码 [1]。它 2019 年 6 月以 SerenityOS 里的一个 329 行「富文本」实验 commit 起步 [9]，2022 年 7 月得名 Ladybird [3]，2024 年 6-7 月独立成库并成立美国 501(c)(3) 非营利组织（GitHub 联合创始人 Chris Wanstrath 捐 100 万美元）[1][2]，2026 年目标发布首个 alpha（Linux/macOS），当前 WPT（Web Platform Tests）绝对通过子测试数约 208.5 万（2026-08-05 nightly run 实测 93.33%）、test262 通过率 97.95% [17][27][61]。

核心要点：

- **时间**：从首 commit 到「渲染出完整网页」约 4 个月；到 Acid3 全过约 2 年 9 个月；到 alpha（2026 年内目标）约 7 年。前 4 年半是 hobby + 极小社区驱动，2023 年中才出现赞助雇来的 2 名全职，2024 年 7 月才成立组织、组建 4 人全职团队 [4][8][12]。
- **团队**：截至 2026-08 约 8 名全职（官方口径「小规模全职工程团队」）[14][29]；核心贡献者约 10 人（Kling、Tim Flynn、Sam Atkins、Aliaksandr Kalenik、Shannon Booth、Andrew Kaster、Jelle Raaijmakers 等）；社区共 428 名贡献者（Ladybird 仓库，GitHub API 实测）[30][51]。
- **架构**：38 个 `Lib*` 库；LibWeb 单体内含解析→样式→布局（formatting context 体系）→display list→绘制全管线；7 类进程（UI/WebContent/RequestServer/ImageDecoder/WebWorker/Compositor/WebDriver）+ LibIPC；2024 年 7 月起用 Skia 做光栅化；2026 年起 LibJS 前端、样式与布局迁往 Rust [21][22][20][16]。
- **质量**：官方只认 WPT 绝对子测试通过数（2026-07 为 2,079,020），由 wpt.fyi nightly 全量运行产出、每月公布；每修复/新特性必带测试、通过的新 WPT 测试立即导入仓库（书面政策）；test262 每 commit 全量跑并公开 dashboard；CI 16 个 workflow，含 PR 防 flaky 门禁、ASAN/UBSAN、多套 JS/WEB 基准 [17][23][24][25][27][28]。
- **关键转折**：2024-08 宣布采用 Swift、2026-02 放弃（约 18 个月插曲）；2026-02 起用 AI 助手（Claude Code/Codex）把 JS 前端管线移植为 Rust（2 周 2.5 万行、52,898 个 test262 零回归）；2026-06-05 起关闭公众 PR、只允许维护者合入代码 [16][32][15][40][41]。

一句话：Ladybird 是「自研派」浏览器里近 7 年唯一从 0 走到 alpha 门槛的案例，其时间节奏、架构取舍、WPT 驱动的质量体系，对 ZeroWeb（复用派）的路线校准、质量基础设施与治理设计有直接的对照价值。

---

## 全文来源分级总表

| 标记 | 含义 | 本文中的主要来源 | 占比 |
|------|------|------------------|------|
| **一手事实** 🟢 | 官方博客/新闻稿、GitHub 仓库文件与 API、官方数据源（wpt.fyi、libjs-data） | ladybird.org 公告/新闻稿、Kling 博客、serenityos.org 周年庆、ladybird 仓库 README/文档/workflows/脚本、GitHub REST API、wpt.fyi、libjs-data [1][2][3][4][9][10][15][16][17][18][19][20][21][23][24][25][27][28][29][30][33][36][43][47][48][49][50][51][52][55][56][57] | 主体 |
| **外部搜索** 🟡 | 权威媒体、社区转载、Wikipedia、LWN 等 | LWN、Simon Willison、The Register、gigazine、w3.org、Wikipedia、lobste.rs/HN 讨论 [11][12][13][14][34][37][38][39][40][41][42][45][46] | 辅助 |
| **二手分析** 🟡 | AI 仓库分析（deepwiki）、第三方博客 | deepwiki（基于源码的自动分析，索引 2026-06-11）[22][58] | 少量 |
| **⚠️ 假设 / 💡 推理** 🟠 | 基于多来源的逻辑推导，无直接证据 | 「首批 4 名全职名单」「WPT 排名第四」等标记处 | 少量 |
| **作者综合** 🟠 | 本文自行设计的图表/框架/对照表 | 阶段划分表、分层图、进程对照表、ZeroWeb 映射表 | 已标注 |

**排除项（调研中证伪或无法证实，不进入正文结论；全部经 2026-08-06 二轮复核，详见「验证更新记录」）**：
- ❌ 「2026-04-12 发布首个公开 alpha、WPT 88%」——仅 hotmolts 单一二手博客声称，与官方「Target: Alpha 2026」直接矛盾；二轮复核官方首页与 GitHub releases（2026-08-06）仍无任何 alpha 发布记录 [59][17]
- ❌ 「Ladybird 计划自研 UI 工具包 InkCanvas」——官方渠道（仓库 issue/PR 搜索、全部新闻稿）零命中；2026-08-06 定向复核（含专用网络搜索）仍零命中，确认为不存在
- ❌ 「$12M 大额融资」——2025-2026 公开记录的最大单笔为 2024 年 Wanstrath 的 $1M 与 2025-12 FUTO 的 $25 万 [48]，未发现 $12M 依据
- ❌ 「SerenityOS 1000+ 贡献者」——GitHub API 实测 SerenityOS 仓库 422 名、Ladybird 仓库 428 名（2026-08-06）[30]

> ⚠️ **调研环境说明**：本环境的 WebFetch 对部分域名（Wikipedia/LWN 等）不可达，一手页面（官方博客、GitHub 文件/API、wpt.fyi、libjs-data）均经 curl/API 直接抓取核实；Wikipedia/LWN 内容通过与多个独立转载源交叉比对后采用。所有数字均注明日期与口径。

---

## 验证更新记录（二轮验证，2026-08-06）

> 本节记录对初版报告中全部「待验证/未验证/假设/未决」位置的一手复核结果。复核方式：GitHub REST API（commit 链、仓库元数据、releases、用户资料）、wpt.fyi search API（复刻官方 wpt-fyi-indexer 算法）、libjs-data 数据仓库、官方站点首页/FAQ。

| # | 原状态 | 复核方式 | 复核结果 | 文档更新 |
|---|--------|---------|---------|---------|
| 1 | LibHTML→LibWeb 更名日期「约 2020 初，未验证」 | GitHub API commit 830a57c | **2020-03-07T09:32 UTC**（「LibWeb: Rename directory LibHTML => LibWeb」，与 LibJS 首 commit 同一天）[62] | §1.2、附录日期已更新 |
| 2 | SerenityOS 首 commit「2018-10-10（官方周年页）」 | GitHub API 最早提交链 | **确认 2018-10-10 09:53 UTC**（首条提交信息 "Import all this stuff into a single repo called Serenity."）；repo `created_at` 2018-12-02 为仓库在 GitHub 公开之日，两者不冲突 [31] | 附录表述精确化 |
| 3 | WPT 当前通过率「2025 年末快照 92.1%」 | wpt.fyi search API + 官方 indexer 算法复算（run 5965379171254272，2026-08-05，commit 9a400b7ec7） | **通过 2,085,135 / 总数 2,234,099 = 93.33%**（作者按官方 `SyncTotalsCommand` 算法复算：`total=max(1,total)`、`status=='P'` 时 `passes=max(1,passes)`）[61] | §4.6 新增 2026-08 行 |
| 4 | test262「2026-04 月报 97.8%」 | libjs-data results.json（run 2026-08-05 13:23 UTC）重新拉取 | **52,475 / 53,575 = 97.95%**（parser-tests 5,300/5,363），与初版一致，已复核 [27] | §4.5 标注已复核 |
| 5 | alpha「2026 年内目标，截至 2026-08-06 未发布」 | ladybird.org 首页 + GitHub releases 复核 | 首页横幅仍为 "Alpha release for Linux and macOS is coming in 2026"；GitHub releases 列表为空（无任何标签）[1][21] | 结论维持，证据增强 |
| 6 | Interop 参与情况（初版未验证） | web-platform-tests/interop 2026/README.md | **Ladybird 未出现在 Interop 2026 参与者名单** [63] | §4.6 新增 |
| 7 | Linus Groh「现居伦敦，Bloomberg 任职」 | GitHub 资料 + linus.dev 复核 | bio 仅 "former @SerenityOS maintainer"、现居伦敦；**「Bloomberg 任职」无任何来源佐证——初版表述有误** | §2.2 已修正 |
| 8 | 首批 4 名受薪工程师名单（⚠️ 假设） | 二次检索（"first paid engineers 2024"） | 仍无公开名单（媒体报道只报数量 4 人、不报名单）[14] | 维持 ⚠️ 假设 |
| 9 | 2026 团队规模「约 8 名（二手）」 | 官方 FAQ 复核 | 官方仅表述 "small full-time engineering team"，无公开人数 | 维持原表述（已注明二手口径） |
| 10 | blog.ladybird.org 域名状态 | 复核 | 仍不可达（DNS 解析失败），官方内容现位于 ladybird.org/posts 与 /newsletter | 维持 |
| 11 | InkCanvas 排除项 | 2026-08-06 定向复核 + 专用搜索 | Ladybird 仓库 issue/PR 与全网络仍零命中，确认为不存在 | 排除项已更新 |
| 12 | hotmolts「2026-04-12 alpha + 88%」排除项 | 官方首页 + releases 复核 | 官方仍无任何 alpha 发布记录，与该二手博客直接矛盾 | 排除项已更新 |
| 13 | GitHub stars / 贡献者数 | API 复核 | Ladybird 64,770 stars / 428 贡献者；SerenityOS 33,718 stars / 422 贡献者（与初版基本持平）[30] | §2.3 已刷新 |

---

## 执行摘要

### 核心结论表

| # | 问题 | 核心结论 |
|---|------|---------|
| 1 | 各阶段花了多少时间 | 7 个阶段约 7 年：最小管线 9 个月 → 联网/真实网页 14 个月 → 规范攻坚 10 个月 → 跨平台化 15 个月 → 全职化 12 个月 → 独立远征 25 个月+。**前 4.5 年（hobby 期）的产出密度远低于后 2.5 年（全职化后）**，全职化是最大的加速杠杆（§1.3） |
| 2 | 哪些贡献者做了哪些事 | 核心圈 ~10 人：Kling（创始人/总裁/lead，LibWeb+LibJS 起点）、Aliaksandr Kalenik（flexbox/grid 布局）、Sam Atkins（CSS，受薪工程师）、Shannon Booth（HTML 解析/DOM/导航 API）、Tim Flynn（WebDriver/Intl/Unicode，现董事会秘书）、Andrew Kaster（构建/CI/基础设施）、Linus Groh（LibJS，TC39 特邀专家，已离开）、Jelle Raaijmakers（现任 COO）等；社区 428 名贡献者（§2.2） |
| 3 | 架构与模块分层 | 38 个 Lib* 库 + 7 类进程 + LibIPC；LibWeb 单体内完成「解析→样式→布局→绘制」全管线（自研 formatting context 布局体系）；Skia 做光栅化；**核心引擎约 95 万行（LibWeb 54.2 万 + LibJS 19.4 万 + 其余），2026 年起 JS 前端/样式/布局迁 Rust（约 13.7 万行 Rust），另有 45,951 行 Flap 编译器**（§3、§6） |
| 4 | 兼容性与质量怎么保证 | **WPT 驱动**：nightly 全量跑 + 绝对数月度追踪 + 通过测试导入常驻 CI + 书面「每修复必带测试」政策；test262 每 commit 全量跑；四类内部测试（Text/Layout/Ref/Screenshot）；CI 16 workflows（sanitizer、flaky 门禁、基准）；20+ libFuzzer target（§4） |
| 5 | 对 ZeroWeb 有什么参考价值 | 5 条可直接借鉴（WPT 导入机制、绝对数度量、测试类型分层、书面测试政策、AI 移植验收方法）+ 3 条教训（技术选型插曲、治理收紧时机、进度预期校准）（§5） |

### 关键发现

1. **「从零自研」的完整时间账本**：Ladybird 用约 7 年、8 人全职 + 400 余人社区走到 alpha 门槛（WPT 93% 上下，2026-08-05 run 实测 93.33%）[61]。对 ZeroWeb 最直接的含义是——**兼容性深水区（样式/布局/绘制集成）不因复用 parser/JS 而消失**，Ladybird 90%+ WPT 的增量主要发生在全职化后的 2 年（2024-07 至 2026-07）[4][8][12][17]。
2. **质量体系的骨架是「测试资产化」**：不是「跑多少 WPT」，而是把通过的新 WPT 测试 import 进仓库、随代码提交、进常驻 CI——官方 CodePolicy 明文要求 [23][24][25]。这保证了 WPT 通过数单向增长且不可回归。
3. **诚实度量的口径纪律**：官方月报只报绝对子测试数（2,079,020），明确「不可换算成百分比排名」；百分比快照需查 wpt.fyi [18][54]。与 ZeroWeb「以 Chromium Oracle 像素对比为诚实通过率」的理念同构（docs/architecture.md）。
4. **两次大转向都踩在「技术选型」上**：Swift 18 个月插曲（互操作未达标即放弃）、2026-02 用 AI 做 Rust 移植（以字节级输出一致 + 52,898 测试零回归验收）[16][32][40][41]。前者是教训，后者是可复用的工程方法。
5. **2026-06 的治理收紧**（关闭公众 PR、仅维护者合入）是 AI 时代开源浏览器项目的标志性事件：当「补丁即努力证明」被 AI 冲淡，项目选择收窄代码责任面 [15]。

### 对 ZeroWeb 的下一步建议（详见 §5）

| 建议 | 落地动作 |
|------|---------|
| ① 建立「WPT 绝对数 + nightly」度量 | wpt-runner 增加 nightly 全量跑 + 月度绝对数记录；PR CI 只跑 smoke 子集（Ladybird 为 33 个）+ 导入测试（Ladybird 约 9,700 个）[25][28] |
| ② 启用「通过即导入」机制 | 每修复兼容性问题时，把对应 WPT 测试拷入 `tests/wpt-runner` 常驻断言集，随代码提交 [23][24] |
| ③ 补「布局树 dump」类测试 | Ladybird Layout 测试 1,044 个，是布局回归的高性价比手段；ZeroWeb 的 IFC layout↔paint 度量不一致问题正适合此类测试先行钉住 [23] |
| ④ 书面化测试政策 + flaky 门禁 | 在 CLAUDE.md/编码准则中固化「每修复必带测试」；PR 新增测试对 base 重跑防 flaky [24][28] |
| ⑤ 校准阶段预期 | 以 Ladybird 阶段耗时表为参照，为 ZeroWeb 的「内核成形→alpha」阶段做诚实的时间预估，避免低估样式/布局深水区 [§1.3] |

---

## 第 1 章 发展历程：七个阶段与各自的耗时

> 本章按时间线组织（用户关注「演进历程」）。时间线图与阶段划分为**作者综合**（基于多来源事实整理，原始资料无此划分）；里程碑日期均有来源，冲突处已注明。

### 1.1 总览时间线（作者综合）

```
2018.10  SerenityOS 首 commit（Kling 康复后自疗项目）[4]
2019.06  LibHTML: Start working on a simple HTML library（329 行，浏览器引擎起点）[9]
2019.10  浏览器首次渲染出完整网页（SerenityOS 一周年纪念页）[4]
2020.03  LibJS（JS 引擎）首 commit；同月集成进浏览器 [10][5]
2020.05  TLS/HTTPS 支持，首次加载出 Google [5]
2021.05  LibWasm（WebAssembly 引擎）诞生 [31]
2022.03  Acid3 测试 100/100 通过 [46]
2022.07  Live-coding 中得名 Ladybird（Qt GUI 移植）[3]
2022.09  「Ladybird: A new cross-platform browser project」公告（仍在 SerenityOS 仓库）[3]
2023.06  Shopify 赞助 $100k；同年雇 2 名全职（Kalenik、Kaster）[8]
2024.06  fork 出独立仓库，Kling 卸任 SerenityOS BDFL [2]
2024.07  成立 501(c)(3) 非营利 Ladybird Browser Initiative，Wanstrath 捐 $1M，4 名全职 [1][14]
2024.07  Skia 成为默认光栅化后端 [20]
2024.08  宣布采用 Swift（记忆安全语言计划）[40]
2024.10  WebDriver 实现全部规范端点；WPT 绝对数 1,565,597 [36]
2025.10  WPT 通过 90% 门槛（1,861,180/2,033,861 ≈ 91.5%）[37][38]
2026.02  放弃 Swift（issue #933 关闭）[32][41]；AI 辅助 Rust 移植启动 [16]
2026.05  合成器独立进程；WebAssembly JIT [17]
2026.06  关闭公众 PR、仅维护者合入 [15]；GPU 访问移入沙箱化合成器 [17]
2026.07  样式系统与布局引擎迁 Rust；WPT 绝对数 2,079,020；alpha 冲刺中（Target: Alpha 2026）[17]
```

### 1.2 七阶段划分与耗时表

| 阶段 | 起止 | 耗时 | 阶段特征与终点能力 |
|------|------|------|-------------------|
| **S0 前史：OS 起步** | 2018-10-10 → 2019-06-15 | ~8 个月 | SerenityOS 自疗项目起步（内核/GUI/工具链），浏览器尚未存在；2019-06 首个浏览器 commit（LibHTML，19 文件 329 行，动机是「想要富文本，不如用 HTML」）[4][9] |
| **S1 最小渲染管线** | 2019-06-15 → 2020-03-07 | ~9 个月 | 建立「解析 HTML → 构建布局树 → 绘制」最小管线：2019-09 基本 CSS，2019-10-10 渲染出**完整网页**（刻意只用简单 HTML/CSS 的官方一周年页）；年底已有 `<br>`、`:hover`、http:// 加载、`<blink>`；2020-03-07 LibHTML 更名 LibWeb（与 LibJS 首 commit 同日）[4][9][62] |
| **S2 联网与真实网页** | 2020-03-07 → 2021-05 | ~14 个月 | LibJS 起步并集成（2020-03）[10][5]；2020-05-30 自研 TLS 后首次加载出 **Google** [5]；Acid2 合规打磨（2020-06）[5]；2021-01 flexbox 首次用于真实网站 [45]；2021-05 LibWasm 诞生 [31]；2021-10 周年报告「空白页与加载崩溃大幅减少」[6] |
| **S3 规范攻坚** | 2021-05 → 2022-03-31 | ~10 个月 | 集中在 CSS/JS 规范正确性与性能；2022-03-30 **Acid3 100/100**——「自 Acid3 发布以来首个达标的新开源浏览器」（Kling 语）[46]；2022-07 实测 22 个真实网站：serenityos.org 5/5、Google/Facebook 登录页 4/5、Amazon/Wikipedia/DuckDuckGo 3/5，短板为图片/字体/MutationObserver/fetch [45] |
| **S4 跨平台化** | 2022-03-31 → 2023-06-18 | ~15 个月 | 2022-07-04 live-coding 视频中给 LibWeb 套 Qt GUI，**得名 Ladybird**（起初仅作调试工具，两个月内成为 Kling 主力开发环境）[3]；2022-09-12 公告升级为跨平台浏览器项目（Linux/macOS/WSL/Android，仍在 SerenityOS 仓库内）[3]；2022 年底 Qt GUI 快速成型（标签页/地址栏/favicon/滚动，QtNetwork 接管网络栈）[3]；2023-05 起 CSS Grid 布局推进（Kalenik）[31] |
| **S5 全职化与工业化** | 2023-06-18 → 2024-07-01 | ~12.5 个月 | 2023-06-18 Shopify $100k 赞助（另有匿名 $100k×2 与 $10k）[8]；用赞助雇 2 名全职（Kalenik、Kaster）[8]；grid/布局正确性/Web API 大幅推进 [8]；2024-06-03 **fork 独立**——Kling 卸任 SerenityOS BDFL，独立仓库 LadybirdBrowser/ladybird，丢弃 SerenityOS 开发目标，**放宽 NIH 政策**（web 标准组件仍自研，其余可用第三方库，如 LibSQL→sqlite3）[2]；2024-07-01 非营利成立 [1] |
| **S6 独立远征（进行中）** | 2024-07-01 → 2026-08 | ~25 个月+ | 4 名全职起步 → 2025 年 7 名 → 2026 年约 8 名 [14][42]；Skia 光栅化（2024-07）[20]、WebDriver 全端点（2024-10）[36]、WPT 90% 门槛（2025-10）[37]、HTTP/3/reCAPTCHA/Trusted Types（2025-07）[60]、Wasm 3.0 套件（2025-10）[19]；Swift 18 个月插曲后放弃（2026-02）[32][41]；AI 辅助 Rust 移植（JS 前端 2026-02、样式+布局 2026-07）[16][17]；合成器独立进程（2026-05）、GPU 隔离（2026-06）[17]；2026-06-05 关闭公众 PR [15]；**2026 年内发布 alpha（Linux/macOS）为官方目标，截至 2026-08-06 尚未发布** [17] |

### 1.3 阶段耗时分析：什么是加速杠杆

> 本节推理链基于 1.2 的日期事实，推导部分标注 💡。

- **前 4.5 年（S0-S4，2019-06 至 2023-12）的产出曲线**：完成了「最小管线 → 真实网页 → Acid3 → Qt 跨平台」这条从 0 到 1 的路径，但引擎能力离「可用的现代浏览器」仍远——2024-09 时 LWN 的评估仍是 pre-alpha：CSS Selectors L1-3 100% / L4 53%（Firefox 71%），无 WebRTC，GitHub「渲染不错但慢」，Gmail 过不了登录 [11]。
- **加速拐点在 2024 年**：fork 独立（2024-06）→ 非营利与全职化（2024-07）之后，单月 880+ commits / 49 位作者（2024-09）[11]，Skia 落地、WebDriver 全端点、WPT 从 ~75% 量级（2024-10 绝对数 1,565,597）到 90% 门槛（2025-10）只用约 1 年 [36][37]。
- 💡 **推理：全职工程师密度是最大的单变量**。S5-S6 的加速与「2 人全职（2023）→ 4 人（2024）→ 7-8 人（2025-2026）」同步；S6 的 90%→92% 段（2025-10 至 2026-07 约 2.4 万新增子测试）反而趋缓，对应 Kling 自述「低垂果实摘完了」（State of the Browser 2025）[42]——兼容性推进是前快后慢的幂律曲线。
- 💡 **推理：规模策略是「先广度后深度」**。S1-S3 先把「解析→样式→布局→绘制→JS→网络」全链路打通（哪怕处处简陋），S5 之后才逐个啃深度兼容（flexbox/grid 规范、WebDriver 全端点、WPT 逐类导入）。这与「先让 pipeline 完整可跑、再逐层对齐规范」的工程次序一致。
- **参考数字**：从 0 到 alpha 门槛 ≈ 7 年（2019-06 → 2026 年内目标）；其中「最小可用浏览器」≈ 2 年 9 个月（Acid3，2022-03）；「能跑真实网站」≈ 3 年（2022-07 实测）；「90% WPT」≈ 6 年 4 个月（2025-10）[4][9][37][45][46]。

> **📌 来源说明（第 1 章）**
>
> - **一手事实** [1][2][3][4][5][6][8][9][10][31][36]：各里程碑日期来自 SerenityOS 官方周年庆文章、Kling 博客、官方 commit 页与 Ladybird 官方公告/新闻稿（curl 直抓核实）。
> - **外部搜索** [11][37][38][40][41][45][46]：LWN 回顾、The Register（Acid3）、Simon Willison（Swift 始末）、gigazine（90% 门槛）、Linus Groh 博客（真实网站实测）。
> - **⚠️ 假设**：S0-S6 的「七阶段划分」与「阶段耗时表」为作者综合归纳，原始资料无此划分；各阶段边界日期均为可验证的锚点事件。
> - **💡 推理**：§1.3 的「全职化是最大加速杠杆」「先广度后深度」为基于日期事实的推导，非官方表述。
> - **口径提醒**：「2022-03 通过 Acid3」有多个来源一致（HN、The Register 2022-03-31）；网上个别资料误记为 2022 年 7 月，无来源支持，已排除。「2022 年 WPT 约 50%」仅见一处第三手博客，置信度低，正文未采用。

## 第 2 章 关键贡献者与治理结构

### 2.1 Andreas Kling：从 WebKit 工程师到独立引擎创始人

- **背景**：曾在苹果 Safari、诺基亚 WebKit 团队工作，并接触 KHTML [1][47]。2018 年在瑞典完成戒毒康复后失业，以「写一个自己的操作系统」为自疗项目，2018-10-10 首 commit，SerenityOS 得名于《宁静祷文》（Serenity Prayer）[4]。
- **全职化路径**：2021-05-28 辞职全职投入，靠 Patreon（约 $2,000/月）+ YouTube + 周边维持 [47]——这是「个人项目 → 全职开源」的典型样本。
- **技术贡献**：LibHTML（2019-06）与 LibJS（2020-03）的首个 commit 均出自他手 [9][10]；Ladybird 仓库累计 19,803 commits（含 SerenityOS 导入历史，GitHub API 实测）[30]。
- **现角色**：Ladybird Browser Initiative 总裁 + 首席开发者；2026-06 主导「关闭公众 PR」治理改革 [15]。

### 2.2 核心贡献者与其贡献领域（已验证）

> 数据口径：提交数来自 GitHub API（2026-08-05 快照，Ladybird 仓库含从 SerenityOS 导入的完整历史）；职责领域来自官方 CODEOWNERS、维护者名单、提交主题与个人公开资料。

| 贡献者（网名） | 职责领域 | 状态 | 证据 |
|---|---|---|---|
| **Andreas Kling**（awesomekling） | 创始人/总裁/lead；LibWeb、LibJS 起点；19,803 commits | 在职 | [1][9][10][30] |
| **Aliaksandr Kalenik**（kalenikaliaksandr） | **flexbox/grid 布局**（grid 匿名块、百分比轨道、flex/grid baseline 等） | 在职，2023 年起受薪 [8] | [31][8][30] |
| **Timothy Flynn**（trflynn89） | **WebDriver 代码所有者；LibJS Intl / LibUnicode / LibTimeZone 代码所有者；LibGfx** | 在职；2024-09-16 起任董事会秘书 | [51][29][30] |
| **Sam Atkins**（AtkinsSJ） | **CSS 系统**（以 sam@ladybird.org 身份参与 CSS Houdini 规范草案） | 在职受薪工程师 | [52][30] |
| **Shannon Booth**（shannonbooth） | **LibWeb HTML 片段解析 / DOM / 会话历史与 Navigation API / WebIDL** | 在职 | [30]（提交历史） |
| **Andrew Kaster**（ADKaster） | **构建系统（vcpkg/devcontainer/flatpak）、CI/基础设施** | 在职，2023 年起受薪 | [51][8][30] |
| **Linus Groh**（linusg） | **LibJS**（Promises/Temporal/ES Modules），TC39 特邀专家；"This Week in Ladybird" 系列作者 | **已离开**（GitHub 自述 former @SerenityOS maintainer，现居伦敦；⚠️ 任职单位无公开佐证——初版「Bloomberg 任职」说法经 2026-08-06 复核无来源支持，已修正） | [30][45] |
| **Jelle Raaijmakers**（gmta） | 早期核心维护者 → **现任 COO** | 在职 | [51][29] |
| **Mike Shaver** | Mozilla 1998 创始成员、前 Mozilla VP Engineering；2024-10-18 起任董事会财务 | 董事会 | [50] |
| **Chris Wanstrath** | GitHub 联合创始人；首捐 $1M；**2025-07-11 退出董事会** | 已退出 | [1][29] |

其他高提交量贡献者（未逐人细分领域）：alimpfard 2,400（LibWasm 作者）、tcl3 2,093、nico 1,887、Lubrsi 1,354、supercomputer7 1,306、Zaggy1024 1,072 等 [30]。2024-07 官方维护者名单共 11 人：alimpfard、Kalenik、Kling、Kaster、BertalanD、gmta、Lubrsi、AtkinsSJ、Tim Flynn、tcl3、timschumi [51]。

⚠️ **假设**：2024-07 首批 4 名受薪工程师的具体名单无公开来源；Kling 在 State of the Browser 2025 的表述是「每位受薪员工都曾是志愿者贡献者」，据此推断首批 4 人大概率出自上述维护者名单，但无法确定是哪四位 [42]。

### 2.3 治理：501(c)(3) 非营利与资金原则

- **法律形式**：美国 501(c)(3) 非营利组织「Ladybird Browser Initiative」，EIN 99-2154861，旧金山 [29]。
- **董事会（2026-08 现状）**：Andreas Kling（President）、Timothy Flynn（Secretary，2024-09-16）、Mike Shaver（Treasurer，2024-10-18）；创始董事 Wanstrath 已于 2025-07-11 辞职 [29][50]。
- **资金原则**（官方明文）：只接受非限制性捐赠与赞助；**永不签默认搜索引擎交易、无广告、无加密代币、无任何用户变现**；董事会成员是「专家而非捐赠者」，企业买不到席位；章程与 IRS 1023 全部公开 [1][29]。
- **资金来源时间线**：Shopify $100k（2023-06）[8] → Wanstrath $1M（2024-07）[1][14] → Cloudflare 赞助（2025-09，金额未披露）[49] → FUTO $250k（2025-12）[48] + 多笔小额（Shopify/Proton 持续、Scraping Fish、SerpApi、Primeagen 等）；Kling 在 State of the Browser 2025 称资金约 2 年 runway [42]。
- **许可证**：BSD-2-Clause [21]。
- **社区规模**（GitHub API，2026-08-06 复核）：Ladybird 64,770 stars / 428 贡献者；SerenityOS 33,718 stars / 422 贡献者 [30]。

### 2.4 贡献模式：从开放 PR 到维护者制（2026-06 标志性转向）

- **开放期（至 2026-06-04）**：公开 PR + 11 名维护者评审合并，Discord #code-review 为实际评审入口；月度活跃度样本：2025-05 261 PR/53 贡献者、2025-07 319 PR/47、2026-04 333 PR/35（含 7 名首次贡献者）[17][51]。
- **2026-06-05 起**（官方公告《Changing How We Develop Ladybird》）[15]：
  - 关闭全部待审公开 PR，**代码只允许维护者合入**，不设任何影子贡献通道（issues/邮件/fork 补丁均不视为评审队列）；
  - 官方理由：AI 工具让「补丁即努力证明」失效；浏览器处理全网不可信输入，需要收窄代码责任面——「重要的是代码进入浏览器后由谁负责」；
  - 非代码贡献仍开放：bug 报告、reduction、网站测试、标准讨论、安全报告。
- 💡 **推理**：这一转向是 AI 编程时代开源浏览器的治理实验。对 ZeroWeb 的启示：若未来开放协作，需要提前设计「贡献责任边界」而不只是「贡献渠道」。

> **📌 来源说明（第 2 章）**
>
> - **一手事实** [1][2][8][9][10][15][29][30][47][49][50][51][52]：Kling 背景、组织页（董事会/章程）、CONTRIBUTING.md 快照、GitHub API 提交数、Cloudflare 公告。
> - **外部搜索** [14][42][45]：Simon Willison 转载（含 Kling 直接引语）、State of the Browser 2025、Linus Groh 博客。
> - **⚠️ 假设**：首批 4 名受薪工程师名单（无公开来源）；部分头部贡献者职责领域未逐人展开。
> - **💡 推理**：§2.4 治理实验的启示为本文推导。

---

## 第 3 章 架构与模块分层

### 3.1 分层模型（作者综合）

```
┌─────────────────────────────────────────────────────────┐
│ UI 层：Qt（Linux/Windows）+ AppKit（macOS）+ Android     │ ← chrome 逻辑在跨平台 LibWebView 层，自绘外观 [22][43]
├─────────────────────────────────────────────────────────┤
│ 浏览器/嵌入层：LibWebView（Application / ViewImplementation / HelperProcess）[22]
├─────────────────────────────────────────────────────────┤
│ 引擎层：LibWeb（HTML/DOM/CSS/布局/绘制/平台 API 全管线）[21][43]
│         LibJS（JS 引擎，2026 前端管线已迁 Rust）[16]
│         LibWasm（WebAssembly）｜ LibGC（GC）｜ LibGfx（2D/图像解码）
├─────────────────────────────────────────────────────────┤
│ 基础设施层：LibCore（事件循环/OS 抽象）｜ LibIPC（进程间通信）
│         LibCrypto/LibTLS/LibHTTP/LibDNS/LibURL｜LibUnicode｜LibMedia
│         LibRegex/LibXML/LibCompress/LibTextCodec/LibSandbox 等 38 库 [21]
├─────────────────────────────────────────────────────────┤
│ 进程层：Browser(UI) + 每标签页 WebContent + RequestServer
│         + ImageDecoder + WebWorker + Compositor + WebDriver [21][22][43]
└─────────────────────────────────────────────────────────┘
```

### 3.2 核心库职责（Lib*，官方 README 列出的主干）

| 库 | 职责 |
|---|---|
| **LibWeb** | 整个 Web 渲染引擎：HTML 解析/事件循环/导航、DOM、CSS（解析/选择器/级联/计算值）、Layout（formatting context 体系）、Painting（Paintable 树/StackingContext/DisplayList）、Compositor，以及 Fetch/XHR/WebSockets/WebGL/WebAudio/IndexedDB/WebDriver/Worker/SVG/MathML/CSP/TrustedTypes 等平台 API——**单体内完成从字节到像素** [21][43] |
| **LibJS** | ECMAScript 引擎（自研，无 JIT；2026-02 起 lexer/parser/AST/作用域收集/字节码生成器为 Rust，解释器仍 C++）[16][21] |
| **LibWasm** | WebAssembly 解析与执行 [21] |
| **LibGfx** | 2D 图形、图像编解码（PNG/JPEG/GIF/WebP…）、**Skia 集成点** [20][21] |
| **LibGC** | LibJS 与 LibWeb 共用的垃圾回收器 [22] |
| **LibIPC** | 进程间通信：`.ipc` 协议文件构建期生成代理/桩；传输层 Unix 域套接字 / macOS Mach 端口 / Windows 套接字 [21][22] |
| **LibWebView** | 浏览器/嵌入层：Application、ViewImplementation、WebContentClient、进程启动 [22] |
| **LibSandbox** | 沙箱（Linux seccomp 等）[22] |
| 其余 28+ | LibCore（事件循环/OS 抽象）、LibCrypto/LibTLS/LibHTTP/LibDNS/LibURL、LibUnicode、LibMedia（ffmpeg）、LibWebSocket、LibXML、LibRegex、LibRequests、LibImageDecoders、LibDatabase（IndexedDB 存储，sqlite3）、LibDevTools（Firefox DevTools 协议过渡方案）、LibThreading/LibSync/LibTest 等 [21] |

> 注：布局引擎不是独立库——在 `Libraries/LibWeb/Layout/` 内（SerenityOS 时代的独立 LibLayout 计划未沿用）。2026-07 起样式与布局迁至 `Libraries/LibWeb/Rust/`（cbindgen FFI）[21]。

### 3.3 进程架构（7 类进程 + LibIPC）

| 进程 | 职责 | 备注 |
|---|---|---|
| **Browser（UI）** | 标签页/窗口管理，spawn 与监控子进程，宿主 Qt/AppKit 前端 | 主进程 [22] |
| **WebContent** | 每标签页一个：HTML/CSS/布局/绘制/JS 执行；**跨源导航切换新进程（站点隔离）** | 沙箱化 [21][22] |
| **RequestServer** | 共享单例：全部 HTTP/HTTPS 请求与磁盘缓存 | 出进程抗恶意内容 [21] |
| **ImageDecoder** | 共享单例：图像解码，隔离编解码器漏洞 | 沙箱化 [21][22] |
| **WebWorker** | 后台 worker 脚本进程池 | [22] |
| **Compositor** | **2026-05 起独立进程**：最终合成、异步滚动、canvas/WebGL 命令回放光栅化；**2026-06 起 WebContent 不再直接访问 GPU** | 沙箱化 [17] |
| **WebDriver** | W3C WebDriver 协议端点（自动化/测试驱动） | 2024-10 全端点 [36] |

进程间通信全部走 **LibIPC**（`.ipc` 协议文件构建期代码生成 + 套接字传输）；崩溃检测/重启有 ProcessManager 与防死循环重载计时器 [22]。

### 3.4 渲染管线（每阶段实现位置）

来源：[官方文档 LibWebFromLoadingToPainting.md](https://github.com/LadybirdBrowser/ladybird/blob/master/Documentation/LibWebFromLoadingToPainting.md)（一手）+ deepwiki 交叉验证 [22][43]。

| 阶段 | 实现位置 | 要点 |
|---|---|---|
| 资源加载 | LibWeb → IPC → RequestServer | 请求走网络进程 [43] |
| HTML 解析 | `LibWeb/HTML/` | 按 HTML 规范自研 tokenizer+parser（2026-05 迁 Rust）[43][17] |
| CSS 解析 | `LibWeb/CSS/` | 构建 CSSOM；变量值留 unresolved 待级联解析 [43] |
| JS 解析执行 | LibJS（Rust 前端 + C++ 解释器） | 字节码生成 2026-04 起移出主线程（线程池）[18] |
| 样式计算 | `LibWeb/CSS/`（StyleComputer/Selector/Cascade） | 选择器右向左求值 + 分桶缓存；级联分 UA/author 起源；产出 computed values [43] |
| 布局树构建 | `LibWeb/Layout/` | DOM×style → box tree；匿名盒/表格盒/list-item marker 修正 [43] |
| 布局 | 2026-07 起在 **`LibWeb/Rust/src/layout/`**（block/flex/grid/inline/table/svg 各 formatting context 的 .rs 文件）；C++ 侧 `LibWeb/Layout/` 仅剩 box 类型 + `LayoutRustBridge` 桥 [64] | **BFC/IFC（LineBuilder）/FFC（Flexbox L1 §9 步骤 1-16）/GFC（Grid L2，四遍 auto-placement）/TFC（CSS 2.2 §17.5.2）/SVG FC**；全用 CSSPixels 定点数 [22][64] |
| 提交 | `LayoutState::commit()` | 几何结果转入 Painting::Paintable 树（可缓存）[22] |
| 绘制记录 | `LibWeb/Painting/` | StackingContext 树按 CSS2 附录 E 分 6 个 phase；每条命令 = (VisualContextIndex, DisplayListCommand) [22] |
| 回放/光栅化 | **Skia**（DisplayListPlayerSkia）→ Gfx::PaintingSurface（Vulkan/Metal） | 2024-07 起 Skia 为默认可插拔光栅化后端；独立 RenderingThread（主线程录 display list → 传线程 → Skia）；2026-04 起每 Navigable 独立线程 [20][17] |
| 命中测试 | Paintable 树逆绘制序遍历 | [22] |

**渲染架构演进时间线（2024-2026）**：自研光栅化 → Skia（2024-07）[20] → display list 命令缓存（2026-02）[33] → 每 Navigable 独立线程光栅化（2026-04）[18] → 合成器独立进程（2026-05）[17] → GPU 访问移入沙箱化合成器、WebGL 共享内存省主线程（2026-06）[17] → 合成器负责视频帧选择与局部重绘（2026-07）[17]。

### 3.5 UI 层与构建系统

- **前端**：Qt（Linux/Windows）、原生 AppKit（macOS）、Android；GTK4 前端 2026-04 加入、2026-07 即移除（「专注把 Qt 端口做扎实」）[18][17]；2026-05 起 chrome 自绘化（tab strip/omnibox 不走 Qt 默认外观）[17]。
- **浏览器 UI 逻辑**（标签页/书签/omnibox/设置）在跨平台共享层 **LibWebView**，各前端实现 `ViewImplementation` 接口 [22]。
- **构建**：CMake + Ninja + vcpkg（manifest 模式）；大量构建期代码生成（WebIDL → JS 绑定、CSS Properties.json → PropertyID、Default.css 嵌入）；vcpkg 三方依赖含 skia/vulkan/curl/openssl/ffmpeg/harfbuzz/simdjson/sqlite3 [22]。
- **Rust 组件（2026）**：根目录已有 Cargo.toml/rust-toolchain.toml/rustfmt.toml；LibJS 前端管线（2026-02，~25k 行，AI 辅助 2 周）→ 样式系统与布局引擎（2026-07，cbindgen FFI，为并行 restyle 铺路）→ 路线图：共存 → 选择性移植（点名候选：网络栈、CSS parser）→ 全量迁移；官方定位「Full rewrites are not interesting to us」[16][17]。

### 3.6 安全与隔离

- **进程隔离**：渲染/网络/图像解码/worker/合成器各自独立进程，每标签页沙箱化 + 站点隔离（跨源导航新建 WebContent）[21][22]。
- **沙箱**：LibSandbox（Linux seccomp；各进程独立 SandboxLinux/SandboxMacOS）；2026-07 起每 profile 独立辅助进程沙箱规则 [22][17]。
- **GPU 隔离**：2026-06 起 WebContent 不再直接访问 GPU [17]。
- **内存安全**：ArrayBuffer/Wasm 内存「caging」——所有 buffer 放单一 4 TiB 保留区，越界无法触及对象/栈/代码 [17]。
- **CSP/CORS/混合内容/SRI/Trusted Types**：均在 LibWeb 内按规范模块实现（`LibWeb/ContentSecurityPolicy`、`LibWeb/Fetch`（CORS 算法属 Fetch 规范）、`LibWeb/MixedContent` 等）[21]。

### 3.7 与主流引擎的架构差异

1. **零借用**：官方承诺不使用 Chromium/WebKit/Gecko 任何代码；LibWeb/LibJS 均为全新实现（Kling 有 WebKit 背景但自称仅受「inspiration」影响）[1][3]。
2. **JS 引擎自研**（LibJS，无 V8/JSC 血统）；2026 年起走「C++ → Rust」路线，与 Firefox/Chromium 引入 Rust 的路线趋同（官方自述受二者先例影响）[16]。
3. **光栅化直接采用 Skia**（可插拔），而非自研光栅器——这是独立引擎里少见的「复用派」决策 [20]。
4. **进程模型相对简单**：每标签页一个 WebContent + 共享 RequestServer/ImageDecoder + 独立合成器，站点隔离粒度仍较粗（💡 推理：与 Chromium 的 site-instance 级隔离相比）[21][22]。

> **📌 来源说明（第 3 章）**
>
> - **一手事实** [16][17][18][20][21][22][33][36][43]：GitHub 仓库目录/README/官方文档（LibWebFromLoadingToPainting、ProcessArchitecture）、2024-08 至 2026-07 官方新闻稿。
> - **二手分析** [22]：deepwiki 的库/进程/渲染章节与一手来源一致的部分。
> - **⚠️ 假设**：§3.7.4 进程模型对比为作者综合推理；`InkCanvas`（自研 UI 工具包）经查无此物已排除；`LibLayout` 独立库仅存在于 SerenityOS 时代规划。
> - **作者综合**：§3.1 分层图、§3.4 渲染管线表（阶段划分结合官方文档与代码目录整理）。

## 第 4 章 兼容性与质量保证：WPT 驱动的三层测试体系

Ladybird 的质量体系可以概括为**一个度量（WPT 绝对子测试数）+ 三层防线（外部套件 nightly 全量 / 导入测试常驻 CI / 内部四类测试）+ 书面政策**。

### 4.1 质量理念与书面政策（一手原文）

- `Documentation/CodePolicy.md`（Testing policy 原文）：「代码变更在修复 bug 或添加新特性时**应包含测试**。如果变更与 WPT 测试相关——尤其是让 Ladybird 通过了此前未通过的 WPT 测试——**应考虑把那些测试导入 Ladybird 树，并随代码一起提交**。」[24]
- `Documentation/Testing.md`（原文）：「**每个加入 LibWeb 的特性或 bug 修复都应在 Tests/LibWeb 有对应测试**，按特性类型选 Text、Layout、Ref 或 Screenshot。」[23]
- 💡 推理：这两条书面政策 + 导入机制（4.2）共同保证了「WPT 通过数单向增长」——通过即资产化，杜绝回头路。

### 4.2 WPT 驱动：工具链与流程（一手）

| 环节 | 机制 | 细节 |
|---|---|---|
| 官方入口 | `Meta/WPT.sh` | 子命令：`update`（从 wpt.git 同步）、`run`（封装 WPT 官方 wptrunner，`--webdriver-binary` 指向自研 WebDriver，headless；Linux 用 overlayfs 只读底层 + 可写上层；CI 用 `WPT_PROCESSES=4` 并行）、`compare`、`import`、`bisect`（二分定位回归 commit）[25] |
| 官方工具支持 | wpt 仓库 `tools/wpt/browser.py` | `class Ladybird(Browser)`，2023-09-27 commit「Add Ladybird WebDriver runner」加入——**Ladybird 是 WPT 官方产品矩阵成员** [35] |
| 导入机制 | `Meta/import-wpt-test.py` | 从 `http://wpt.live/` 下载测试与脚本，拷入 `Tests/LibWeb/<type>/input/wpt-import/`，生成 expected 文件，**随代码一起提交**；目前导入树内 WPT 测试 **9,710 个文件** [23][25] |
| PR CI 范围 | 33 个 smoke 测试 | `Tests/LibWeb/WPT/ci-smoke-tests.txt`（infrastructure/worker/secure-context/WebSocket/reftest/crashtest/testdriver 等，全必过，5 分钟超时）——**完整 WPT 不进 PR CI** [25][28] |
| 全量运行 | wpt.fyi **nightly** | 官方 WPT 基础设施对 ladybird product 每日全量跑（2026-08-05 实测 run 存在，结果存档可下载）[26][54] |
| 月度度量 | `LadybirdBrowser/wpt-fyi-indexer` | PHP 应用 `app:sync-totals` 从 wpt.fyi `/api/search` 拉各产品 subtest 汇总入 Postgres——**月报绝对数即此数据** [57] |

### 4.3 内部四类测试（Tests/LibWeb，2026-08-06 仓库统计）

| 类型 | 数量 | 说明 |
|---|---|---|
| **Text** | 12,800 | JS 写 API 输出 println 文本对比 |
| **Layout** | 2,126 | **布局树 dump 对比**（布局回归的高性价比手段）[23] |
| **Ref** | 2,382 | 测试页与参考页双截图比对（`<link rel="match">`）[23] |
| **Screenshot** | 242 | 与预存 PNG 比对（官方建议少用，敏感）[23] |
| **C++ 单测** | 各库 TestFoo.cpp | LibTest 框架；另有 LibTest/Randomized 属性测试库（内部收缩）[21][23] |

另有 LibJS 1,411 个 .js 测试 + 5 个 C++、AK 86 个 .cpp [21]。**总数级：约 1.8 万内部测试 + 9,726 导入 WPT（`Tests/LibWeb/.../wpt-import/`，2026-08-06 本地源码统计）+ 外部套件 nightly**。

### 4.4 CI 体系（.github/workflows 16 个 workflow，一手）

- **主 CI**（Lagom 矩阵 7 组合）：Linux x86_64 Release；Linux x86_64 / arm64、macOS arm64、Windows x86_64 的 **Sanitizer（ASAN+UBSAN）**；Linux **Fuzzers** 编译门禁；Linux All_Debug。统一官方容器 `ghcr.io/ladybirdbrowser/ladybird-ci` [28]。
- **每 PR 门禁**：`ctest --timeout 1800` + WPT smoke（33 个）+ **防 flaky**：`Meta/check-test-flakiness.py` 把 PR 新增测试对 base ref 重跑（1200s 截止）[28]。
- **test262 CI**：`libjs-test262.yml` 在 **push master 与每个 PR 都跑完整 tc39/test262 + parser-tests + WASM spec**；`per_file_result_diff.py` 与上次结果 diff（回归检测）；结果部署到公开仓库 `libjs-data` [28][27]。
- **基准 CI**：web-benchmarks（Speedometer2/3、StyleBench，self-hosted）+ js-and-wasm-benchmarks（ARES-6、JetStream、SunSpider、Kraken、Octane、WasmCoremark 等）[28]。
- **Nightly**：每日 cron 跑 Linux arm64 Sanitizer + 三平台 Distribution + Flatpak + Android [28]。
- **Lint**：clang-format 全套 + Flap（Rust 字节码解释器）fmt/clippy/test + 提交信息 lint [28]。

### 4.5 JS 引擎质量：test262 + fuzzing

- **test262 运行器**：`Utilities/test262-runner.cpp` + Python 封装（并发、每测试 10s 超时、内存 512MB 上限）[28]。
- **公开数据**（一手，2026-08-05 run，2026-08-06 二轮复核重新拉取确认）：test262 **52,475/53,575 ≈ 97.95%**；parser-tests 5,300/5,363；公开 dashboard：ladybirdbrowser.github.io/libjs-website/test262 [27]。月报口径（2026-04，WPT 导入后）：52,045/53,207 = 97.8% [18]。
- **跨引擎对比**（二手）：LibJS 维护者 trflynn89 在 HN 引 test262.fyi：LibJS 96.9%（含 experimental），对照 SpiderMonkey 98.3%、V8 97.9%、JavaScriptCore 93.2%；并说明 Ladybird **不用 feature flags 隐藏未实现提案**，故口径与主流引擎不同 [38]。
- **Fuzzing**：`Meta/Fuzzers/` 20+ libFuzzer target（FuzzJs、FuzzRegexECMA262、FuzzCSSParser、FuzzWasmParser、FuzzURL、图像/字体解码器、FuzzilliJs 集成等）[55]；⚠️ 勘误口径：仓库 Fuzzers/README「OSS-Fuzz 持续运行」的说法继承自 SerenityOS 时代，2026-08 实测 google/oss-fuzz 仅有 `projects/serenity`（仍 clone SerenityOS/serenity），**Ladybird 仓库尚无独立 OSS-Fuzz 项目**，CI 的 Fuzzers 预设仅做编译门禁 [55][56]。

### 4.6 量化进展与口径纪律

**WPT 绝对子测试通过数（官方月报口径，一手）**：

| 日期 | 绝对数 | 备注 |
|---|---|---|
| 2024-10 | 1,565,597 | WebDriver 全端点后开始规模化 [36] |
| 2025-07 | 1,831,856 | 单月 +13,090（7 月月报，经第三方转述）[60] |
| 2025-10-05 | 1,861,180（≈91.5%，分母 2,033,861） | **通过 90% 门槛**（Kling 宣布，关联 Apple iOS 替代引擎资格讨论）[37][38] |
| 2025-10-31 | 1,964,649 | 含 Wasm 3.0 套件更新 +100,751 [19] |
| 2025 年末 | 92.1%（1,984,880/2,154,492） | wpt.fyi 快照（二手引述），同期 Chrome 97.2% / Firefox 95.9% / Safari 95.1% / Servo 86.6% [39] |
| 2026-02 | 1,998,398 | [33] |
| 2026-03 | 2,003,690 | 破 200 万 [18] |
| 2026-04 | 2,067,263 | 含 WPT 新导入的 test262 53,207 项 [18] |
| 2026-06 → 07 | 2,078,912 → 2,079,020 | 单月 +108——低垂果实摘完后增速放缓 [17] |
| **2026-08-05（实测）** | **2,085,135 / 2,234,099 = 93.33%** | wpt.fyi nightly run（commit 9a400b7ec7）；作者按官方 wpt-fyi-indexer 算法复算 [61] |

**口径纪律**（官方明确）：月报数字是**绝对子测试通过数，不可换算成百分比排名**；百分比快照需以 wpt.fyi 检索日期为准 [18][54]。**Interop 参与**：Ladybird 未出现在 Interop 2026 官方参与者名单（2026-08-06 核实 interop 仓库）[63]。**无「100% WPT/test262」官方目标**；最接近的官方表述是 Kling 的「committed to catching up eventually」[42]，维护者解释原因为规范与测试套件本身是移动目标 [38]。

**能力状态参考（2024-09，LWN）**：CSS Selectors L1-3 100%、L4 53%（Firefox 71%）；无 WebRTC；GitHub「不错但慢」、Gmail 过不了登录 [11]。

> **📌 来源说明（第 4 章）**
>
> - **一手事实** [17][18][19][23][24][25][26][27][28][33][35][36][54][55][56][57]：官方测试手册/代码政策/WPT.sh/CI workflow/libjs-data/wpt.fyi 均直接抓取核实。
> - **外部搜索** [11][37][38][39][42]：gigazine（90% 门槛）、HN/lobste.rs（对比数据与口径）、LWN、SOTB 2025。
> - **勘误口径**：OSS-Fuzz 归属（Ladybird 仓库无独立项目）；「88%」数字仅单一不可靠来源，已排除（见来源分级总表）。
> - **💡 推理**：§4.1 的「测试资产化保证通过数单向增长」与 §4.6 增速放缓分析为本文推导。

## 第 5 章 对 ZeroWeb 的参考价值

> 本章为**作者综合 + 推理**：Ladybird 事实（§1-§4）→ ZeroWeb 现状（docs/architecture.md、docs/research/rust-cross-platform-browser-research.md）→ 落地建议。所有 Ladybird 侧陈述已在前面章节标注来源；ZeroWeb 侧陈述以仓库文档为准。

### 5.1 路线对照：Ladybird（自研派）vs ZeroWeb（复用派）

| 组件 | Ladybird | ZeroWeb | 对照含义 |
|---|---|---|---|
| HTML 解析 | LibWeb/HTML **自研** | zero-dom **复用 html5ever** | Ladybird 2024 年放宽 NIH 政策（web 标准组件自研、其余可复用第三方）[2]——ZeroWeb 的「permissive 模块复用 + 自建集成层」路线与其放宽后的策略同构，方向正确 |
| JS 引擎 | LibJS **自研**（2020 起，6 年+持续投入，无 JIT） | zero-script-sandbox **复用 V8/QuickJS** | LibJS 是 Ladybird 最大的单点投入；2026 年其 Rust 移植仍需 2.5 万行/AI 辅助 2 周——**ZeroWeb 复用 V8/QuickJS 是已被验证的正确杠杆** [16] |
| CSS/样式 | LibWeb/CSS 自研 | zero-css-parser + zero-style-system 自研 | 双方一致：样式系统（级联/继承/计算值）是兼容性主战场，无法靠复用绕过 |
| 布局 | 自研 FormattingContext 体系（BFC/IFC/FFC/GFC/TFC） | zero-layout-engine：taffy 复用 + 自建 inline 等 | Ladybird 按规范步骤实现 flex/grid（FFC 对 Flexbox L1 §9 步骤 1-16、GFC 四遍 auto-placement）是 WPT 高分的关键 [22]；taffy 只覆盖 flex/grid/block，**ZeroWeb 的 inline/IFC 深水区（layout↔paint 度量不一致）正是 Ladybird 也需自建的部分** |
| 光栅化 | LibGfx + **Skia（复用）** | zero-render-foundation（wgpu 自研图元） | 反向对照：Ladybird 在光栅化上选择「复用现成后端」（2024-07）[20]，ZeroWeb 选择自研图元——两侧各有理由，但 Ladybird 证明「渲染后端可插拔复用」是一条低风险路径 |
| 多进程 | 7 类进程 + LibIPC（站点隔离、每标签页 WebContent） | apps/renderer + zero-protocol（IPC 契约） | 理念一致；Ladybird 的 **ImageDecoder 独立进程**（隔离解码器漏洞）与 **Compositor 独立进程 + GPU 隔离**（2026-05/06）[17] 可作为 ZeroWeb 多进程演进路线图的参考目标 |
| UI | Qt/AppKit 前端 + LibWebView 共享逻辑 | zero-host-runtime（winit）+ browser-shell | 相同分层思路：UI 逻辑与平台窗口解耦；Ladybird 2026-05 起 chrome 自绘化 [17]，与 ZeroWeb 的 browser-shell 数据模型 + 自绘方向一致 |
| 测试 | WPT.sh + 四类测试 + 导入机制 + nightly + test262 CI | tests/wpt-runner + Chromium Oracle + product-smoke | 同理念不同实现：**ZeroWeb 的 Chromium Oracle 像素对比比 Ladybird 的 Ref/Screenshot 更强**；Ladybird 的「布局树 dump」测试是 ZeroWeb 可低成本补上的 |
| 语言 | C++ → Rust（2026 起渐进迁移） | **Rust（起点即 Rust）** | Ladybird 2026 年才开始 Rust 化并公开认可 Rust 生态（「Rust 2024 年曾被评估后否决，2026 年因生态成熟而选择」）[16]——是对 ZeroWeb Rust 选型的正面印证；ZeroWeb 无需重走「先 C++ 再迁移」的路 |

### 5.2 可直接借鉴的实践

**P1 测试资产化：通过的新 WPT 测试立即导入常驻 CI（优先级最高）**
- Ladybird 事实：CodePolicy 书面要求 + `import-wpt-test.py` 把通过的新 WPT 测试拷入 `Tests/LibWeb/.../wpt-import/` 随代码提交；目前 9,700 个导入文件常驻 CI，PR CI 另跑 33 个 smoke [23][24][25][28]。
- ZeroWeb 现状：已有 `tests/wpt-runner` + reftest + Chromium Oracle，但「通过即导入」的资产化机制未成文。
- 落地：每次修通一个 WPT/reftest 用例时，把该用例拷入 `tests/wpt-runner` 常驻断言集并随修复提交；PR CI 拆分「smoke 子集（秒级）+ 全量 nightly（过夜）」两级。

**P2 度量纪律：绝对数 + nightly 全量 + 月度追踪**
- Ladybird 事实：官方只认 WPT 绝对子测试通过数（2,079,020，2026-07），数据来自 wpt.fyi nightly 全量运行，月报公布；明确「绝对数不可转百分比」[17][18][54][57]。
- ZeroWeb 现状：已有「诚实度量」原则（Oracle 像素对比为诚实通过率，同源 reftest 仅自一致性参考）[architecture.md]。
- 落地：为 `make reftest-oracle` 增加 nightly 定时全量 + 月度通过数记录（绝对数而非百分比），形成趋势基线；百分比只在固定套件快照下比较。

**P3 测试类型分层，补「布局树 dump」类**
- Ladybird 事实：Text 5,714 / **Layout 1,044** / Ref 1,150 / Screenshot 111 四类；Layout 测试是「布局树 dump 对比」，专门钉布局回归 [23]。
- ZeroWeb 现状：reftest（像素）+ product-smoke（结构断言）两类为主。
- 落地：给 zero-layout-engine 增加布局树 dump 对比测试（`tests/wpt-runner` 内新增 `--dump-layout` 模式），对 IFC 度量不一致等结构性回归先行钉住——比像素 diff 更快定位、比结构断言更细。

**P4 书面测试政策 + PR 防 flaky 门禁**
- Ladybird 事实：`Documentation/CodePolicy.md`「每修复/新特性必带测试」成文；`check-test-flakiness.py` 把 PR 新增测试对 base ref 重跑 [24][28]。
- ZeroWeb 现状：CLAUDE.md 有「目标驱动执行」（修复 bug → 先写复现测试）但未成文 WPT 导入规则；已有 test-guard OOM 包裹与 plateau-guard。
- 落地：在 CLAUDE.md 编码准则中补充「渲染兼容性修复必须附带对应 WPT/reftest 用例，优先导入 wpt-runner 常驻集」；CI 增加新增测试 flakiness 重跑门禁。

**P5 AI 辅助移植/重构的验收方法（与 ZeroWeb 的 AI-first 开发直接相关）**
- Ladybird 事实：2026-02 用 Claude Code + Codex 把 LibJS 前端管线 C++→Rust，~2.5 万行/约 2 周；验收标准是 **lockstep 字节级输出一致 + 52,898 个 test262 + 12,461 个回归测试 0 回归 + 无基准回退**，默认启用 + 环境变量回退（`LIBJS_CPP=1`）[16]。
- ZeroWeb 现状：AI-first 开发流程已建立（AI 生成大量代码）。
- 落地：任何「AI 大规模重写/移植」任务，先建双管线对照（新旧实现可切换、逐字节/逐值对比），以全套件回归 0 差异为合入门禁——这套方法在 Ladybird 已被验证可行。

**P6 透明度与节奏：月度工程报告**
- Ladybird 事实：「This Month in Ladybird」月报（2024 年 7 月起持续至今）：绝对数、当月 PR/贡献者数、技术亮点，是项目工程记忆与社区信任的载体 [17][18][19][36]。
- 落地：ZeroWeb 可建立同样的月度/周度工程记录（docs/ 内已有 goal/ 文档体系），把「做了什么 + 诚实数字」沉淀为仓库的一部分。

### 5.3 需要警惕的教训

**L1 技术选型插曲：Swift 18 个月的沉没成本**
- 事实：2024-08 宣布采用 Swift（记忆安全愿景），2026-02 放弃（C++ 互操作「never quite got there」、非 Apple 平台支持有限；官方自述「又一年原地踏步」），issue #933 关闭、PR 移除全部 Swift 代码 [32][40][41]。
- 对 ZeroWeb：选型要优先「生态成熟度」而非「愿景完整度」。具体而言——之前调研已确认 V8/QuickJS 双引擎等价支持成本高 [research-2026-05-30]；Ladybird 的教训进一步支持「**首期只稳定支持一个默认 JS 引擎**，trait 抽象预留但不承诺等价」的结论。

**L2 AI 时代的贡献治理：2026-06 关闭公众 PR 的两难**
- 事实：官方以「AI 让补丁即努力证明失效 + 浏览器处理不可信输入需收窄责任面」为由关闭公众 PR，社区有争议（批评者担心治理封闭）[15]。
- 对 ZeroWeb：spec 目标里有「开源协作下保持核心代码与演进节奏可控」——建议提前设计「贡献责任边界」（哪些模块接受外部 PR、合入者责任归属、AI 生成代码的审查政策），而不是等冲突爆发再收紧。

**L3 进度预期校准：兼容性是幂律曲线**
- 事实：WPT 90%（2025-10）→ 92%（2026 初）→ 2,067,263（2026-04）→ 2,079,020（2026-07，三个月 +12k）；Kling 自述「低垂果实摘完了」[17][18][42]。
- 对 ZeroWeb：当前 broad 一致率约 57%、strict 处于 plateau（docs/goal/rendering-compat.md）——**57%→90% 与 90%→95% 的边际成本完全不同**；建议用 Ladybird 的阶段耗时表（§1.3）为 ZeroWeb 的「内核成形 → 可用 → alpha」三档做诚实时间预估，避免对「最后一个百分点」低估。

**L4 指标口径：绝对数 vs 百分比**
- 事实：Ladybird 官方月报明确「绝对数不可转百分比」[18]；ZeroWeb 已有「同源 reftest 存在假通过，仅自一致性参考」的诚实度量意识 [architecture.md]——两者是同一纪律：**先定义分母，再谈通过率**。

### 5.4 综合结论

> ### 💡 推理分析：自研 vs 复用不是兼容性的分水岭，集成层才是
>
> **观察**：Ladybird 在 HTML 解析/JS 引擎上自研，但在光栅化上复用 Skia、2024 年放宽 NIH 政策（LibSQL→sqlite3）[2][20]；其 WPT 高分主要来自 LibWeb 的样式/布局/API 集成层，而非 parser 或 JS 引擎。
>
> **推理**：ZeroWeb 复用 html5ever/V8/taffy 并不构成兼容性短板——真正的瓶颈（也是 Ladybird 花费 5+ 年积累的部分）是 CSS 级联/继承/计算值、格式化上下文布局、绘制顺序、Web API 语义这些**集成层**，ZeroWeb 的 zero-css-parser/style-system/layout-engine/engine 恰好在这条线上。
>
> **结论**：ZeroWeb 与 Ladybird 是同一问题的两种资源分配：Ladybird 用 7 年 + 400 人社区验证了「从零到 alpha 的完整账本」；ZeroWeb 用复用省下的时间应全部投入到集成层深度与质量体系，并直接借用 Ladybird 验证过的工程机制（P1-P6）。

**按优先级排序的落地建议**：
1. **P1 + P2**（测试资产化 + 绝对数度量）——立即做，改动最小、收益最大，直接强化当前「渲染兼容性」主线；
2. **P3**（布局树 dump 测试）——下一个兼容性攻坚窗口（IFC 度量问题）配套做；
3. **P4 + P5**（书面政策 + AI 重构验收方法）——随 CLAUDE.md 更新与下一次大规模重构时落实；
4. **L1-L4 教训**——写入决策备忘，特别在 JS 引擎选型、开源协作章程、进度预期三处显式引用。

> **📌 来源说明（第 5 章）**
>
> - **一手事实** [2][16][17][18][20][22][23][24][25][28][32][40][41][54][57]：Ladybird 侧全部引用可回溯至官方来源。
> - **一手事实（ZeroWeb 侧）**：docs/architecture.md、docs/research/rust-cross-platform-browser-research.md（2026-05-30）、CLAUDE.md。
> - **💡 推理**：§5.1 对照表、§5.4 综合结论为本文推导，非任何单一来源的结论。
> - **作者综合**：P1-P6 落地建议与 L1-L4 教训为本文结合双方事实设计的行动清单。

---

## 第 6 章 源码深潜（2026-08-06 本地全量分析）：代码规模、2026 架构现状与工程坑实录

> 本章为**一手源码分析**：2026-08-06 将 ladybird master（tarball 44.7MB）解包至本地全量阅读（26,420 文件，源码约 95 万行）。所有行数为本地 `wc -l` 统计，测试文件数为文件树 API 统计 [64]。

### 6.1 代码规模全景（本地统计）

| 组件 | 行数（.cpp/.h/.rs） | FIXME 数 | 说明 |
|---|---:|---:|---|
| **LibWeb** | **542,176** | 2,327 | 整个渲染引擎 + 平台 API；CSS 4.7MB / HTML 4.9MB（字节口径）为最大子模块 |
| **LibJS** | 194,090 | 147 | JS 引擎（2026 前端管线已 Rust 化，另有 Flap 编译器 45,951 行 Rust） |
| **LibWasm** | 32,149 | 17 | 29 个文件的高密度实现 |
| AK（基础库） | 43,435 | — | 容器/字符串/ErrorOr+TRY 基础设施 |
| LibGfx | 25,230 | 35 | 2D 图形 + Skia 集成（PainterSkia/PathSkia/SkiaVulkanMemoryAllocator） |
| LibWebView | 29,941 | — | 浏览器/嵌入层 |
| LibGC | 6,837 | — | 自研精确追踪 GC（Cell/HeapBlock/Conservative 容器） |
| Services（各进程） | 23,162 | 97 | Compositor/WebContent/RequestServer/ImageDecoder 等 |
| **Rust 总量**（Libraries/*/Rust） | **137,412** | — | LibJS 前端 + LibWeb 样式/布局 + Flap |

**最直观的事实**：这是一个约 95 万行的从零自研引擎——比 Servo 之外任何现代引擎都更接近「手写」的极端。LibWeb 一家占 54 万行，且其中 2,327 处 FIXME 多为「规范步骤未实现」标记（见 4.1 的规范驱动文化）。

### 6.2 2026 架构现状：Rust 迁移后的真实结构（本地验证）

- **LibWeb/Layout C++ 目录只剩「壳」**：box 类型（BlockContainer/InlineNode/TextBox 等 86 个文件） + `LayoutRustBridge`（FFI 桥），**formatting contexts 已全部移入 `LibWeb/Rust/src/layout/`**（block/flex/grid/inline/table/svg 各 .rs + tree_builder/line_builder/used_values/abspos_engine）[64]。
- **LibWeb/Rust 完整结构**（50 文件，2.6MB）：`css/`（style_compute/selector_engine/cascaded_properties/computed_values/css_pixels/css_tokenizer/animation/transition/calc）+ `layout/`（上述）+ encoding_detection + FFI 支撑（ffi_support/ffi_stats/retained_fly_string）[64]。
- **LibJS/Rust**：lexer/parser/AST/scope_collector/bytecode（codegen/generator/validator/native_disassembler）——「前端管线」的完整范围即此 [64]。
- **FFI 边界设计**（cbindgen.toml + LayoutRustBridge.h）：
  - cbindgen 生成 C++ ABI 头（`rename_types = "PascalCase"`，无 includes、只带 stdint/stddef）[64]；
  - Rust 侧通过 `FfiLayoutFcCallbacks`（回调 C++ 取数据）与 `FfiCommitSink`（把几何结果提交回 C++）双回调工作；
  - **关键注释**（LayoutRustBridge.h）：「Rust 布局 pass 在栈上时，计算值绝不能被替换——pass 缓存了解码后的样式并借用载荷指针，替换会使其失效」→ 全局守卫 `layout_pass_currently_running()` [64]。

### 6.3 工程坑与解决方案（源码 + 月报证据）

> 每个坑：**现象 → 根因 → 解法 → 证据**。

**坑 1：跨语言 FFI 的借用失效窗口（C++↔Rust 混合引擎的元问题）**
Rust 布局借用 C++ 持有的对象（box 树/计算值）时，C++ 侧任何「替换计算值」都会让 Rust 侧借用悬垂。
解法：① 全程标记 `layout_pass_currently_running()`，在该窗口内禁止替换计算值；② 提交路径走回调 sink（Rust 产出 → C++ 应用）；③ JS 侧跨 FFI 传递 GC 对象用 `GC::Root` 防止中途被回收（RustIntegration.h 注释 "NB: Uses GC::Root to prevent collection while the result is in transit"）；④ 样式字段跨 FFI 用「惰性解码 + 句柄 + 显式 release」（`ladybird_layout_release_anchor_name_handle`）[64]。

**坑 2：跨语言内存布局漂移（Flap 汇编需要 C++ struct 偏移）**
Flap 编译器为 LibJS 生成原生解释器 handler，必须按 C++ 结构体真实偏移生成汇编，而编译器/平台/构建 flag 都会改变布局。
解法：**`GenerateLayout.cpp` 构建期小工具，用与 LibJS 相同的编译 flag 编译，打印各 struct 字段偏移为 DSL 常量**（注释原文："Compiled with the same flags as LibJS so layouts match exactly"）——把「布局一致」变成构建期强制 [64]。

**坑 3：布局精度与跨平台确定性**
浮点布局在不同平台/优化级别产生不一致结果，且规格允许的维度上限会溢出浮点。
解法：**CSSPixels 定点数**（PixelUnits.h，2023）：6 位小数（1/64 精度）、饱和运算、三种舍入模式（Nearest/Floor/Truncate）；最大维度值 17895700 直接参考 Firefox（注释："Apparently the largest value allowed by Firefox. Probably enough for us as well."）；DevicePixels 用 int 物理像素，CSS 像素与设备像素分层 [64]。

**坑 4：C++ 引擎的对象生命周期（DOM/JS 对象图）**
C++ 无 GC，DOM 节点 ↔ JS 对象互相引用，引用计数会产生环。
解法：**自研精确追踪 GC（LibGC，6,837 行）**：`Cell` 类型 + 分块分配器（CellAllocator/HeapBlock）+ 栈扫描（StackInfo）+ 保守容器（ConservativeHashMap/Vector/HashTable，供非 GC 感知代码持有引用）+ `CrossHeapMember`/`Weak`/`DeferGC`/IdleCollectionPolicy 全套 [64]。

**坑 5：内存安全（ArrayBuffer/Wasm 越界访问）**
JS 不可信代码的 buffer 越界 = 任意内存读写漏洞。
解法：**caging（PrimitiveStorage）**：所有原始存储放进**单一 4 TiB 幂等保留区**（`default_cage_size = 4ull * TiB` + offset mask 寻址），越界无法触及保留区外的对象/栈/代码；TypedArray 视图用 `contiguous_bytes_from` 处理**跨 cage 边界的视图拆分**；LibWeb 侧 `ExternalPrimitiveStorage` 让宿主对象投影到 ArrayBuffer 而无需教 LibJS 认识它们 [64]。配套成效：2026-07 月报称重型页面内存占用降约 80% [17]。

**坑 6：字节码解释器的可维护性（手写汇编 handler 的尽头）**
为 JS 引擎写高性能解释器 = 维护数千行手写汇编 handler，跨架构翻倍。
解法：**Flap——45,951 行 Rust 的类型化汇编编译器**：用 DSL 描述解释器 handler 语义，经 HIR → **SSA（含 SCCP 常量传播、内联、memory_optimize）** → low_ir → 寄存器分配 → **x86_64/AArch64 双后端**，构建期 `generate-libjs-bytecode` 生成汇编，并带 `machine_verify`/`verify` 验证阶段；lib.rs 注释明确其安全契约（只接受可信构建输入）[64]。

**坑 7：自研光栅化器的工程化成本**
自研 2D 光栅化器（曲线/滤镜/合成）工程量大、难以追平 Skia/Chromium 质量。
解法：**可插拔 Painter 接口 + Skia 后端**（LibGfx：PainterSkia/PathSkia/SkiaVulkanMemoryAllocator/SkiaBackendContext，2024-07 起为默认）——引擎侧只依赖 Gfx::Painter 抽象，光栅化可替换 [20][64]。这是独立引擎里少见的「复用派」决策。

**坑 8-10（治理/选型/度量）**：Swift 18 个月选型插曲（2024-08 宣布 → 2026-02 放弃，§5.3 L1）[32][40][41]；AI 时代贡献治理（2026-06 关闭公众 PR，§2.4）[15]；WPT 绝对数口径与低垂果实（§4.6）[18][42]。

**质量文化注记**：LibWeb 全库 **4,750 处规范链接注释 + 2,327 处 FIXME + 系统性 `// OPTIMIZATION:` 注释**（StyleInvalidation/FontComputer/StyleSheetInvalidation 等），配合 LibWebPatterns.md 书面规范（错误分层 ErrorOr→ExceptionOr→SimpleException/DOMException、spec 步骤逐条注释）——「规范驱动 + 显式标记未完成」是 90%+ WPT 的代码层基石 [24][64]。

### 6.4 对 ZeroWeb 的直接可迁移项（与 §5.2 互补）

| 坑/解法 | ZeroWeb 对应点 |
|---|---|
| 坑 1 FFI 借用守卫 | 若未来 Rust 组件跨 C++ 式所有权边界（如 V8 对象 ↔ Rust DOM），同一「窗口守卫 + 句柄释放」模式可直接套用 |
| 坑 3 定点数布局 | ZeroWeb 布局用 CSSPixels 概念可对照：**布局中间量用定点/固定精度而非 f32/f64**，可消除跨平台 diff（当前 Chromium Oracle 对比正受浮点敏感性影响时尤其值得验证） |
| 坑 5 caging | ZeroWeb 的 V8（rusty_v8）自带隔离；若未来接 wasmi 处理不可信内存，caging 是可选加固方向 |
| 坑 7 可插拔光栅化 | 印证 §5.1 对照：渲染后端抽象（Painter 接口）是 Low-risk 决策，ZeroWeb 的 render-foundation 分层已具备同等形态 |
| 坑 2 布局一致性工具 | 「构建期生成跨语言常量」模式对 ZeroWeb 的 IDL/binding 生成同样适用（已有类似代码生成基础设施） |

> **📌 来源说明（第 6 章）**
>
> - **一手事实** [64]：全部为本地源码阅读（2026-08-06，master），含行数统计、注释原文、文件结构；[17][20][24][42] 用于坑 5/7/质量文化与月报数据。
> - **作者综合**：6.1 表格与 6.4 迁移对照为本文整理。
> - 说明：行数统计口径为各组件目录下 .cpp/.h/.rs 文件 `wc -l` 合计；FIXME 为 grep 计数（含注释与字符串内出现）。

---

## 参考资料

| # | 来源 | 类型 | 引用章节 | 备注 |
|---|------|------|---------|------|
| [1] | [Announcing the Ladybird Browser Initiative（ladybird.org）](https://ladybird.org/posts/announcement/) | 一手 | §2、§5.1 | 2024-07-01 官方公告：501(c)(3)、$1M、资金原则 |
| [2] | [Forking Ladybird and stepping down as SerenityOS BDFL（ladybird.org）](https://ladybird.org/posts/fork/) | 一手 | §1、§5.1 | 2024-06-03 fork 公告，NIH 政策放宽 |
| [3] | [Ladybird: A new cross-platform browser project（Kling 博客，web.archive.org 归档）](https://web.archive.org/web/20250301233033/https://awesomekling.github.io/Ladybird-a-new-cross-platform-browser-project/) | 一手 | §1 | 2022-09-12 公告，含命名史 |
| [4] | [SerenityOS 一周年（serenityos.org/happy/1st/）](https://serenityos.org/happy/1st/) | 一手 | §1 | 2018-10-10 首 commit、2019-06 LibHTML、2019-10-10 首屏渲染 |
| [5] | [SerenityOS 二周年（serenityos.org/happy/2nd/）](https://serenityos.org/happy/2nd/) | 一手 | §1 | 2020-03 LibJS、2020-05-30 Google、Acid2 |
| [6] | [SerenityOS 三周年（serenityos.org/happy/3rd/）](https://serenityos.org/happy/3rd/) | 一手 | §1 | 2021-10 性能与稳定性进展 |
| [8] | [SerenityOS 五周年（serenityos.org/happy/5th/）](https://serenityos.org/happy/5th/) | 一手 | §1、§2 | 2023-06 Shopify、2 名全职雇佣 |
| [9] | [LibHTML 首 commit（GitHub）](https://github.com/SerenityOS/serenity/commit/a67e823838943b31fb7cea68bd592093e197cf16) | 一手 | §1 | 2019-06-15，19 文件 329 行 |
| [10] | [LibJS 首 commit（GitHub）](https://github.com/SerenityOS/serenity/commit/f5476be702009968468731df5e23cdeb68fdb6e0) | 一手 | §1 | 2020-03-07 |
| [11] | [Ladybird browser spreads its wings（LWN）](https://lwn.net/Articles/976822/) | 外部搜索 | §1、§4 | 2024-09 深度回顾；pre-alpha 全景 |
| [12] | [W3C TPAC 2024 会议页（Ladybird 官方时间线）](https://www.w3.org/events/meetings/8e1ca708-fdbf-4264-a79b-4c953fa85248/) | 外部搜索 | 执行摘要 | 2026/2027/2028 路线图 |
| [13] | [Ladybird (web browser)（Wikipedia）](https://en.wikipedia.org/wiki/Ladybird_(web_browser)) | 外部搜索 | 背景综述 | 与多源交叉核对后采用 |
| [14] | [Announcing the Ladybird Browser Initiative（Simon Willison）](https://simonwillison.net/2024/Jul/1/the-ladybird-browser-initiative/) | 外部搜索 | §1、§2 | 含 Kling 直接引语（4 名全职） |
| [15] | [Changing How We Develop Ladybird（ladybird.org）](https://ladybird.org/posts/changing-how-we-develop-ladybird/) | 一手 | §1、§2、§5.3 | 2026-06-05 关闭公众 PR |
| [16] | [Ladybird adopts Rust, with help from AI（ladybird.org）](https://ladybird.org/posts/adopting-rust/) | 一手 | §1、§3、§5 | 2026-02-23；Rust 移植验收标准 |
| [17] | [This Month in Ladybird – July 2026（ladybird.org）](https://ladybird.org/newsletter/2026-07-31/) | 一手 | §1、§3、§4、§5 | 最新官方状态；WPT 2,079,020 |
| [18] | [This Month in Ladybird – April 2026](https://ladybird.org/newsletter/2026-04-30/) | 一手 | §1、§4 | test262 97.8%；WPT 2,067,263 |
| [19] | [This Month in Ladybird – October 2025](https://ladybird.org/newsletter/2025-10-31/) | 一手 | §1、§4 | Wasm 3.0 套件；WPT 1,964,649 |
| [20] | [This Month in Ladybird – August 2024（buttondown）](https://buttondown.com/ladybird/archive/this-month-in-ladybird-august-2024) | 一手 | §1、§3 | Skia 成为默认光栅化器 |
| [21] | [LadybirdBrowser/ladybird 仓库（README 与目录）](https://github.com/LadybirdBrowser/ladybird) | 一手 | §3 | Lib* 清单、进程清单、BSD-2-Clause |
| [22] | [deepwiki: LadybirdBrowser/ladybird 架构分析](https://deepwiki.com/LadybirdBrowser/ladybird) | 二手分析 | §3 | 索引 2026-06-11；仅采用与一手一致部分 |
| [23] | [Documentation/Testing.md](https://github.com/LadybirdBrowser/ladybird/blob/master/Documentation/Testing.md) | 一手 | §4、§5 | 四类测试、WPT 导入手册 |
| [24] | [Documentation/CodePolicy.md](https://github.com/LadybirdBrowser/ladybird/blob/master/Documentation/CodePolicy.md) | 一手 | §4、§5 | Testing policy 原文 |
| [25] | [Meta/WPT.sh](https://github.com/LadybirdBrowser/ladybird/blob/master/Meta/WPT.sh) | 一手 | §4 | WPT 官方入口 |
| [26] | [wpt.fyi API runs（ladybird）](https://wpt.fyi/api/runs?product=ladybird) | 一手 | §4 | nightly 全量运行记录 |
| [27] | [libjs-data test262 results.json](https://github.com/LadybirdBrowser/libjs-data/blob/master/test262/results.json) | 一手 | §4 | 52,475/53,575（2026-08-05） |
| [28] | [.github/workflows（ci/lagom-template/libjs-test262 等 16 个）](https://github.com/LadybirdBrowser/ladybird/tree/master/.github/workflows) | 一手 | §4 | CI 结构、flaky 门禁、基准 |
| [29] | [Ladybird Browser Initiative 组织页](https://ladybird.org/organization/) | 一手 | §2 | 董事会、EIN、章程公开 |
| [30] | [Ladybird 贡献者页（GitHub REST API 实测）](https://github.com/LadybirdBrowser/ladybird/graphs/contributors) | 一手 | §2 | 428 名贡献者、提交数 |
| [31] | [SerenityOS 仓库（含 LibWasm 等 git log）](https://github.com/SerenityOS/serenity) | 一手 | §1、§2 | LibWasm 2021-05、grid 提交 |
| [32] | [issue #933 Swift 6.0 Blockers（GitHub）](https://github.com/LadybirdBrowser/ladybird/issues/933) | 一手 | §1、§5.3 | 2026-02 关闭；放弃 Swift |
| [33] | [This Month in Ladybird – February 2026](https://ladybird.org/newsletter/2026-02-28/) | 一手 | §1、§4 | Rust 采用；WPT 1,998,398 |
| [34] | [Indie web browser Ladybird flutters toward Rust（The Register）](https://forums.theregister.com/forum/all/2026/02/23/ladybird_goes_rusty/) | 外部搜索 | §5 | 2026-02-23 Rust 转向报道 |
| [35] | [wpt commit: Add Ladybird WebDriver runner](https://github.com/web-platform-tests/wpt/commit/74963009db) | 一手 | §4 | 2023-09-27，ladybird product 入 WPT 官方 |
| [36] | [This Month in Ladybird – October 2024](https://ladybird.org/newsletter/2024-10-31/) | 一手 | §1、§4 | WebDriver 全端点；WPT 1,565,597 |
| [37] | [Ladybird 通过 Apple 90% WPT 门槛（gigazine）](https://gigazine.net/gsc_news/en/20251007-ladybird-apple-alternative-browser-engines/) | 外部搜索 | §1、§4 | 2025-10-05 90% 门槛 |
| [38] | [Hacker News（Acid3 / 90% / trflynn89 test262 评论）](https://hn.svelte.dev/item/45493358) | 外部搜索 | §1、§4 | 口径与跨引擎对比 |
| [39] | [lobste.rs: Servo 2025 Stats 讨论（wpt.fyi 对比快照）](https://lobste.rs/s/6vnavr/servo_2025_stats#c_auskxu) | 外部搜索 | §4 | 92.1% 快照（日期近似） |
| [40] | [Ladybird set to adopt Swift（Simon Willison）](https://simonwillison.net/2024/Aug/11/ladybird-set-to-adopt-swift/) | 外部搜索 | §1、§5.3 | 2024-08-11 |
| [41] | [Ladybird abandons Swift（Simon Willison）](https://simonwillison.net/2026/feb/19/ladybird/) | 外部搜索 | §1、§5.3 | 2026-02-19 |
| [42] | [Andreas Kling @ State of the Browser 2025](https://2025.stateofthebrowser.com/speaker/andreas-kling/) | 外部搜索 | §1、§2、§4 | 7 名受薪、runway、低垂果实 |
| [43] | [Documentation/LibWebFromLoadingToPainting.md](https://github.com/LadybirdBrowser/ladybird/blob/master/Documentation/LibWebFromLoadingToPainting.md) | 一手 | §3 | 渲染管线官方文档 |
| [45] | [Trying real websites in the SerenityOS browser（Linus Groh）](https://linus.dev/posts/trying-real-websites-in-the-serenityos-browser/) | 外部搜索 | §1、§2 | 2022-07 22 个真实网站实测 |
| [46] | [The SerenityOS browser now passes Acid3（The Register）](https://www.theregister.com/software/2022/03/31/serenityos-a-remarkable-achievement-for-a-small-project/) | 外部搜索 | §1 | 2022-03-31 |
| [47] | [I quit my job to focus on SerenityOS full-time（Kling 博客）](https://awesomekling.github.io/I-quit-my-job-to-focus-on-SerenityOS-full-time/) | 一手 | §2 | 2021-05-28 全职化 |
| [48] | [This Month in Ladybird – December 2025](https://ladybird.org/newsletter/2025-12-31/) | 一手 | §2 | FUTO $250k |
| [49] | [Cloudflare: Supporting the future of the open web](https://blog.cloudflare.com/supporting-the-future-of-the-open-web/) | 一手 | §2 | 2025-09-22 赞助 |
| [50] | [Mike Shaver joins the Ladybird board（ladybird.org）](https://ladybird.org/posts/mike-shaver-joins-board/) | 一手 | §2 | 2024-10-18 |
| [51] | [CONTRIBUTING.md（2024-07 快照，11 名维护者）](https://github.com/fwcd/ladybird/blob/master/CONTRIBUTING.md) | 一手 | §2 | 维护者名单 |
| [52] | [Sam Atkins 个人站](http://samatkins.co.uk/) | 一手 | §2 | CSS 工程师、受薪员工 |
| [54] | [wpt.fyi: ladybird 结果页](https://wpt.fyi/results/?product=ladybird) | 一手 | §4 | 官方指定查询入口 |
| [55] | [Meta/Fuzzers/（20+ fuzz target）](https://github.com/LadybirdBrowser/ladybird/tree/master/Meta/Fuzzers) | 一手 | §4 | FuzzJs/Fuzzilli 等 |
| [56] | [google/oss-fuzz projects/serenity](https://github.com/google/oss-fuzz/tree/master/projects/serenity) | 一手 | §4 | Ladybird 无独立 OSS-Fuzz 项目（勘误口径） |
| [57] | [LadybirdBrowser/wpt-fyi-indexer](https://github.com/LadybirdBrowser/wpt-fyi-indexer) | 一手 | §4 | 月报绝对数来源 |
| [58] | [deepwiki: WPT and Compliance Testing 章节](https://deepwiki.com/LadybirdBrowser/ladybird/8.2-wpt-and-compliance-testing) | 二手分析 | §4 | 与一手文件交叉验证 |
| [59] | [hotmolts: Ladybird's 88% WPT score…](https://www.hotmolts.com/post/ladybirds-88-wpt-score-is-a-legitimate-signal-4fef6f79-e8b1-40f6-a1a4-9becc650b6c5) | 不可靠 | 排除记录 | 声称 2026-04-12 alpha + 88%，与官方矛盾，仅作排除 |
| [60] | [Ladybird Browser Major Milestones（finance.biggo.com，转述 2025-07 月报）](https://finance.biggo.com/news/202508021312_Ladybird_Browser_Major_Milestones) | 外部搜索 | §1、§4、附录 | 2025-07 月报数据：WPT 1,831,856、reCAPTCHA/HTTP3 |
| [61] | [wpt.fyi: Ladybird 最新 run（2026-08-05，run 5965379171254272）](https://wpt.fyi/results/?run_id=5965379171254272) | 一手（作者复算） | §4、验证记录 | 通过 2,085,135 / 2,234,099 = 93.33%，按官方 wpt-fyi-indexer 算法复算 |
| [62] | [SerenityOS commit 830a57c: LibHTML 更名 LibWeb（2020-03-07）](https://github.com/SerenityOS/serenity/commit/830a57c6b23430c749395811761252d1999f3559) | 一手 | §1、附录、验证记录 | 更名日期精确化 |
| [63] | [web-platform-tests/interop 2026 README](https://github.com/web-platform-tests/interop/blob/main/2026/README.md) | 一手 | §4 | Ladybird 未参与 Interop 2026 |
| [64] | Ladybird master 源码（本地全量分析，2026-08-06） | 一手（源码阅读） | §3、§4、§6 | tarball 44.7MB 解包至本地临时目录（`~/temp/ladybird`）；26,420 文件；行数/FIXME/注释均为本地统计 |

> 注：编号 [7][44][53] 预留未用；[13] 仅作背景综述；[59] 为排除记录（不可靠来源）；[60] 为 2025-07 月报的第三方转述。一手来源均于 2026-08-05/06 直接抓取核实。

---

## 附录：完整里程碑时间线（2018-2026）

| 日期 | 里程碑 | 来源 |
|------|--------|------|
| 2018-10-10 | SerenityOS 首 commit（09:53 UTC，"Import all this stuff into a single repo called Serenity."；Kling 康复后自疗项目；repo 于 2018-12-02 公开上 GitHub） | [4][31] |
| 2019-06-15 | LibHTML 首 commit（19 文件 329 行，「想有富文本，不如用 HTML」） | [9] |
| 2019-09-29 | LibHTML 基本 CSS 支持 | [4] |
| 2019-10-10 | 首次渲染完整网页（官方一周年页） | [4] |
| 2020-03-07 | LibHTML 更名 LibWeb（与 LibJS 首 commit 同日） | [62] |
| 2020-03-07 | LibJS 首 commit | [10] |
| 2020-03-14 | LibJS 集成进浏览器（`<script>` 解析、DOM 绑定） | [31] |
| 2020-05-30 | 自研 TLS；首次加载出 Google | [5] |
| 2020-06-30 | Acid2 合规工作 | [5] |
| 2021-01 | flexbox 首次用于真实网站（linus.dev） | [45] |
| 2021-05 | LibWasm 诞生（alimpfard） | [31] |
| 2021-10 | 性能工作；崩溃大幅减少 | [6] |
| 2022-03-30/31 | **Acid3 100/100 通过** | [46] |
| 2022-07-04 | Live-coding 得名 Ladybird（Qt GUI 移植） | [3] |
| 2022-07-07 | 22 个真实网站实测（Linus Groh） | [45] |
| 2022-09-12 | 「A new cross-platform browser project」公告 | [3] |
| 2022 底 | Qt GUI 成型（标签页/地址栏/favicon/滚动/QtNetwork） | [3] |
| 2023-05 | CSS Grid 布局推进（Kalenik） | [31] |
| 2023-06-18 | Shopify $100k 赞助 | [8] |
| 2023 年内 | 用赞助雇 2 名全职（Kalenik、Kaster） | [8] |
| 2024-06-03 | **fork 独立仓库**，Kling 卸任 SerenityOS BDFL，放宽 NIH | [2] |
| 2024-07-01 | **501(c)(3) 非营利成立**，Wanstrath $1M，4 名全职 | [1][14] |
| 2024-07 | Skia 成为默认光栅化器 | [20] |
| 2024-08-11 | 宣布采用 Swift（记忆安全计划） | [40] |
| 2024-09 | LWN 回顾：CSS Selectors L1-3 100%/L4 53%；单月 880+ commits/49 作者 | [11] |
| 2024-09-25 | W3C TPAC：官方路线图 2026 alpha / 2027 beta / 2028 stable | [12] |
| 2024-10 | WebDriver 全规范端点；WPT 1,565,597 | [36] |
| 2025-07 | reCAPTCHA/HTTP3/Trusted Types/120Hz；WPT 1,831,856 | [60] |
| 2025-09-22 | Cloudflare 赞助（金额未披露） | [49] |
| 2025-10-05 | **WPT 90% 门槛**（1,861,180/2,033,861） | [37][38] |
| 2025-10 | Wasm 3.0 套件更新（+100,751 子测试）；WPT 1,964,649 | [19] |
| 2025-12-31 | FUTO $250k 续捐 | [48] |
| ~2025 末 | wpt.fyi 快照 92.1%（1,984,880/2,154,492） | [39] |
| 2026-02-18/19 | **放弃 Swift**（issue #933 关闭、PR 移除全部 Swift 代码） | [32][41] |
| 2026-02-23 | **AI 辅助 Rust 移植**（LibJS 前端 2.5 万行/2 周；test262 52,898 零回归） | [16][33] |
| 2026-03 | WPT 破 200 万（2,003,690） | [18] |
| 2026-04 | WPT 2,067,263；test262 97.8%；PDF 查看器/GTK4 前端/线程池字节码生成 | [18] |
| 2026-05 | 合成器独立进程；WebAssembly JIT；MSE 默认开启；Cloudflare Turnstile | [17] |
| 2026-06 | 下载管理/OS 沙箱化/GPU 隔离/WASM GC | [17] |
| 2026-06-05 | **关闭公众 PR，仅维护者合入** | [15] |
| 2026-07 | 样式系统与布局引擎迁 Rust；隐私浏览/profile；WPT 2,079,020；内存占用降 ~80%（官方自报） | [17] |
| 2026 年内（未达） | **Alpha 发布（Linux/macOS）**——官方目标，截至 2026-08-06 尚未发布 | [17] |
| 2027（计划） | Beta（可下载应用，Linux/macOS） | [12] |
| 2028（计划） | Stable（面向公众） | [12] |

---

## 联动说明

本报告调研结论可作为 `lei-spec-rfc` 流程的输入：执行摘要 → spec 背景；§5.2 P1-P6 落地建议 → 需求规格；§5.1 架构对照与 §3 架构事实 → 技术设计基础。如需启动，请告知。

