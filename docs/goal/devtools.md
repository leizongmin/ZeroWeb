# DevTools 调试面 — 复用 Chrome DevTools frontend（Network / Elements / Console / Application）

**版本**: v1.0
**日期**: 2026-09-12
**状态**: Active（启动门控——cdp-protocol goal 达 M3（DOM/CSS 域就位）后解锁主线开发；
门控期间不停摆——转零碰撞面自主推进：bundle 获取/pin/许可核查、serve 骨架、面板可用度评估）
**执行模式**: 面板可用性驱动（每面板一条可演示调试流为验收）；协议域缺口回流 cdp-protocol
goal 或碰头协调
**父目标**: `docs/goal/zero-web.md`（M11 浏览器应用调试能力 + M10 自动化基础延伸）

> **说明**
> 本文档是 ZeroWeb「DevTools 调试面」专项目标执行契约。目标：把 pin 版本的 Chrome
> DevTools frontend（devtools-frontend bundle）本地 serve 并经 CDP 附接 ZeroWeb，使
> **查看网络请求、查看/编辑 cookie、JS console、审查元素**四个高频调试面板可用。
> 路线（2026-09-12 用户拍板）：**复用 Chrome DevTools frontend**，不自建面板 UI。
>
> **▶ 拆分动机（2026-09-12 用户决策）**：① 调试能力是浏览器日常可用的基本盘
> （父目标 DC-2 面）；② CDP 协议基座（cdp-protocol goal）落地后，复用 frontend 是
> 获得生产级调试 UX 的最低成本路径（自建瀑布图/REPL/DOM 检查器是一个完整的大 UI
> 工程）；③ 四面板能力对下游 e2e 排障与真实站点兼容性调查（Top-N 矩阵面）是直接生产力。
>
> **▶ 基线事实（2026-09-12 实测）**：
> - **CDP 基座**：`cdp-protocol` goal（2026-09-12 立项）——WS transport + 发现端点 +
>   Target/Runtime/Page/Input/DOM/CSS/Network/Emulation 域按 Playwright 矩阵收敛中。
>   本 goal 消费其 CDP 面，不自建协议层。
> - **DevTools UI 现状**：零——无任何内置调试面板。
> - **frontend 供给面**：devtools-frontend 仓库提供可自托管的 bundled 前端（`front_end/`
>   构建产物或 release 附属 bundle）；Chromium 系浏览器经 `devtools://` 内嵌同源。
>   本仓需要：bundle 获取与版本 pin、静态 serve（复用 net/hyper 服务面或 headless.rs
>   既有 HTTP 面扩展）、许可核查（Apache-2.0 系，非 MPL——入库前记账）。
> - **面板依赖的 CDP 域**：Elements→DOM+CSS+Overlay（高亮）；Console→Runtime+
>   Log+consoleAPICalled；Network→Network 域事件流（请求事件总线）；Application→
>   Storage/Network.getCookies（cookie jar 暴露）。域缺口以 cdp-protocol 账本为准回流。
> - **前端对 CDF（DevTools frontend）的宽容度**：frontend 对缺失域优雅降级（面板
>   灰置不崩溃）——「面板可用」以逐面板演示流判定，不以 frontend 零报错判定。

---

## Mission

以**逐面板可演示调试流为验收标尺**，把 Chrome DevTools frontend 附接到 ZeroWeb：
开发者打开 DevTools 即可查看网络请求（列表+详情）、查看 cookie、执行 JS console
（含对象求值）、审查元素（树+样式+高亮）。

**关键约束**：
1. **不自建面板 UI**：frontend bundle 之外零 UI 开发；缺能力一律落到 CDP 域层解决。
2. **bundle pin + 许可记账**：frontend 版本固定、来源与许可证（非 MPL）在 evidence/
   记账后方可入库。
3. **面板级验收**：每面板定义一条可自动化（或可脚本演示）的调试流；「可用」= 流全绿，
   不要求 frontend 全面板无灰。

覆盖范围：

1. **serve 骨架** — bundle 获取/pin、HTTP 静态 serve、`devtoolsFrontendUrl` 注入
   `/json` 发现端点（点开即用）
2. **Elements 面板** — DOM 树/样式/高亮的域面可用性（DOM/CSS/Overlay）
3. **Console 面板** — REPL evaluate + console 消息双向（Runtime/Log）
4. **Network 面板** — 请求列表/详情（headers/响应体预览）（Network 域事件流）
5. **Application 面板（cookie 面）** — cookie 查看/编辑（cookie jar 暴露域）
6. **桌面浏览器接线** — CDP server 在浏览器 GUI 模式可达（端口开关），非 headless-only

### 排除（明确不在范围内）

- 面板 UI 的任何自建/改造（frontend 原样使用）
- Debugger 断点调试面（Sources 面板）——frontend 会显示但域不实现即灰置，记账不验收
- Performance/Profiler/Security 等其余面板——同上灰置记账
- 移动端远程调试 UX

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| bundle 供给 | devtools-frontend 获取、版本 pin、许可核查、入库或构建脚本 | evidence 记账先行 |
| serve 面 | 静态 serve + `/json` 发现端点注入 `devtoolsFrontendUrl` | headless.rs HTTP 面扩展 |
| CDP 域补齐 | 面板所需域「可用度」缺口收敛（Overlay 高亮、响应体获取等） | 缺口回流 cdp-protocol 或碰头 |
| 浏览器接线 | GUI 模式 CDP server 启停 + 端口面 | apps/browser 共享面（android-browser 并行流，碰前 git log） |
| 验收资产 | 每面板演示流脚本化（Playwright 或 CDP 原语驱动）+ 截图/记录 evidence | 「可用」可判定 |

### 依赖约束（run-rules §9 碰撞管理）

- **与 cdp-protocol goal（上游）**：强依赖——启动门控 = 其 M3（DOM/CSS 域就位）。
  本 goal 发现域缺口时：小缺口（单方法语义）碰头协调；结构性缺口（新域/新事件总线）
  记账回流上游 goal。
- **与 android-browser（活跃并行流）**：`apps/browser` 共享——碰前 git log 核对。
- **与 rendering-compat**：crate 零重叠。

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: bundle 供给合规

- [ ] devtools-frontend 版本 pin（来源 URL/commit/版本号记账）；许可证核查记账
      （确认非 MPL 线，与父目标排除条款兼容）
- [ ] serve 骨架可用：HTTP 端点返回 bundle；`/json` 列表附 `devtoolsFrontendUrl`，
      浏览器打开即进入附接态

### DC-2: 四面板可用性

- [ ] **Elements**：演示流——审查指定元素 → 树中定位 + computed/matched styles 展示
      + overlay 高亮；样式改动（domain 支持范围内）生效或记账灰置项
- [ ] **Console**：演示流——页面 console.log 到达面板 + REPL evaluate 返回值正确
      （含对象求值）
- [ ] **Network**：演示流——页面发起 N 个请求 → 面板列表可见 + 单请求详情
      （headers/状态/响应体预览范围内）可查
- [ ] **Application（cookie）**：演示流——cookie jar 内容在面板可见 + 编辑回写生效
      （domain 支持范围内）
- [ ] 每条演示流脚本化留档（evidence/，可重放）

### DC-3: 桌面浏览器接线

- [ ] GUI 模式 CDP server 可开关（CLI/默认关），开启后 DevTools 附接流程与 headless 一致
- [ ] `make test` 全绿 + clippy `-D warnings` + fmt 干净；生产渲染/导航路径零回归

---

## 活跃里程碑

### M0 — 门控期自主面（cdp-protocol M3 前推进）

**目标**：bundle 获取/pin/许可核查记账 + serve 骨架（对 Chromium 可自验 serve 正确性）
+ 对 Chromium 空跑的面板可用度基线评估（判定「面板可用」的现实判据）。

### M1 — 附接与 Elements/Console

**目标**：devtoolsFrontendUrl 注入 + frontend 附接 ZeroWeb；Elements/Console 演示流绿。

### M2 — Network/Application

**目标**：Network 面板请求列表/详情演示流绿；Application cookie 演示流绿。

### M3 — 桌面接线 + 收口

**目标**：GUI 模式接线 + 全 DC 判定 + 挂账（灰置面板/域清单）。

---

## Final Output Protocol

### 输出规则

| 情况 | 输出 | 说明 |
|------|------|------|
| Done Criteria 全部满足 | `DONE` | 见下方「DONE 允许条件」 |
| 进展仍可推进 | `CONTINUE: <下一步>` | **这是默认输出** |
| 真正的外部阻塞 | `BLOCK: <原因>` | 罕见使用（含上游 cdp-protocol 门控未解锁且自主面做完） |

### DONE 允许条件

**同时满足**：DC-1~3 全部满足；演示流基于真实 frontend 对真实 ZeroWeb CDP 端点
（无 mock 充数）；`cargo build` + `make test` + `cargo clippy` 全过；master.md 内部自洽。

---

## Execution Protocol

### 自主执行原则

1. **门控期不停摆**：M0 全部为自主面（零 CDP 域改动）
2. **每面板独立 land**：演示流绿一个收一个，evidence 同步落
3. **域缺口不硬啃**：结构性缺口回流上游 goal，本 goal 只做消费侧收敛

### 遇到问题时的处理原则

1. **frontend 版本行为差异**：以 pin 版本实际行为为准，记账
2. **面板灰置不是失败**：域不实现的面板灰置记账，不影响「面板可用」判定
3. **许可疑问即停**：bundle 许可核查有疑点 → BLOCK 等用户裁决，不入库

---

## Document Control / Archive Policy

- **入口文档**（本文件）：定义 Mission、Done Criteria、执行协议和文档治理规则。
  仅在目标本身实质变化时修改；每轮执行不重写。
- **运行时控制平面** `docs/goal/devtools/master.md`：当前真实状态唯一控制面板。
- **归档区域** `docs/goal/devtools/archive/`：只追加不修改。
- **证据区域** `docs/goal/devtools/evidence/`：bundle/许可记账、面板演示流记录，持续追加。
