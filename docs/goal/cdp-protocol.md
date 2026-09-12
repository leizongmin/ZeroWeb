# CDP 协议兼容 + Playwright 验证矩阵 — Chrome DevTools Protocol 服务与自动化验收

**版本**: v1.0
**日期**: 2026-09-12
**状态**: Active
**执行模式**: WPT/webdriver-goal 同款「协议逐域收敛 + 真实客户端 E2E 账本」模式；遇深结构
（多进程 Target 路由重构、V8 对象句柄桥）→ 记「待用户决策」→ 跳过 → 继续其他域
**父目标**: `docs/goal/zero-web.md`（M10 WebView API 与自动化基础延伸 + M11/M14 自动化验收基建）

> **说明**
> 本文档是 ZeroWeb「CDP 协议兼容」专项目标执行契约。目标：让 Playwright（pin 版本）经
> `connectOverCDP` 完整驱动 ZeroWeb——导航、点击、填充、截图、cookie、console、网络事件、
> 对话框、viewport 仿真全流可用，且**每个已实现的 CDP 命令都有对应 E2E 验证用例**。本目标
> 同时是 devtools goal（Chrome DevTools frontend 复用）的协议基座。
>
> **▶ 拆分动机（2026-09-12 用户决策）**：① 自动化验证能力——Playwright 是事实标准的
> 浏览器自动化客户端，接通后为本仓与下游提供 e2e 测试资产；② DevTools 调试面（用户拍板
> 复用 Chrome DevTools frontend 路线）以 CDP 为唯一协议基座，本目标为其前置；③ 现状
> headless.rs 已有 CDP 雏形但距 Playwright 可用差一个量级，需专项收敛。
>
> **▶ 基线事实（2026-09-12 实测）**：
> - **CDP 服务器雏形**：`apps/browser/src/headless.rs`（2256 行）——WS 服务器 +
>   HTTP 发现端点（`/json/version` L1159、`/json` L571）+ 「Phase 3 CDP 最小兼容子集」
>   （L9-10 注记）：仅 `Page.navigate`（L782）/ `Runtime.evaluate`（L795）/
>   `Target.getTargets`（L796）三命令 + BiDi 风格事件（browsingContext.load 等）；
>   `HeadlessSecurityConfig`（L333-360）带 token/origin 校验。默认端口 9222
>   （main.rs L425-429，headless 模式）。
> - **Playwright 差距**：`connectOverCDP` 需要的 `Target.setAutoAttach`、`Runtime.enable`、
>   `Page.getLayoutMetrics`、`Input.dispatchMouseEvent/dispatchKeyEvent`、`Emulation.*`、
>   `Network.enable` 事件流、`DOM.*`/`CSS.*` 域等全部缺失——现状连接即失败。
> - **console 管线**：shim console（part02.js L4370）→ `__zw_console_log(level, 扁平化
>   字符串)` 宿主回调——对象结构丢失，不足以支撑 `Runtime.consoleAPICalled` remoteObject
>   形态与 REPL evaluate（需 V8 对象句柄）。
> - **网络/cookie 面**：`crates/net` 纯客户端；cookie jar 在 `net/cookie.rs`；**无请求
>   事件总线**——`Network` 域事件流与 devtools Network 面板共用此缺口。
> - **架构先例**：webdriver goal（已归档）——`apps/webdriver` W3C 协议服务器（34 endpoint）
>   + 真实 TCP + 真实 renderer 子进程集成测试模式，直接平移；多 tab Target 路由经
>   `zero-protocol` IPC（SetViewport/FetchParams 同款消息面）。
> - **文件大小约束**：headless.rs 2256 行已超 2000 行上限（CLAUDE.md §5）——M1 须先按
>   职责拆分（transport / discovery / domains / session）再扩展。

---

## Mission

以 **pin 版本 Playwright 实际发送的 CDP 命令全集为账本、Playwright E2E 用例为验收标尺**，
把 ZeroWeb 的 CDP 兼容面收敛到「自动化可用」：连接/附接健壮、核心域命令语义正确、
未实现命令标准报错。验收口径（2026-09-12 用户拍板）：**Playwright 命令矩阵账本**——
「已实现的全部验证过」，不承诺「实现全部协议」。

**关键约束**：
1. **不追全协议**：CDP 全协议数百方法跨数十域，本目标只收敛 Playwright 矩阵 + devtools
   面板所需面；账本三态登记（实现 / 部分 / 不实现），「不实现」必须返回 `-32601` 标准错误。
2. **安全边界不退化**：现有 token/origin 校验语义保持；CDP 端口默认仅监听 loopback。
3. **生产路径零回归**：headless 既有自动化面（wpt parity / smoke）语义不变；浏览器
   GUI 模式不因 CDP server 引入回归。

覆盖范围：

1. **传输与发现** — WS transport、`/json/version`、`/json`、`/json/list` target 枚举
2. **Target 域** — setAutoAttach / attachToTarget / targetInfo（多 tab 经 zero-protocol 路由）
3. **Runtime 域** — enable/disable、evaluate（remoteObject 句柄）、consoleAPICalled
   （console 对象化：V8 侧结构化 arg 序列化）
4. **Page 域** — 导航事件族、getLayoutMetrics、handleJavaScriptDialog、截图（复用
   webdriver-screenshot 遗产 `zero-paint-convert` + CPU render 路径）
5. **Input 域** — dispatchMouseEvent / dispatchKeyEvent（接 host 事件管线）
6. **DOM/CSS 域** — getDocument / querySelector / getMatchedStylesForNode（NodeId↔
   backendNodeId 稳定句柄桥；复用 event-loop-spec 遗产 `unique_selector_for_node` 经验）
7. **Network 域** — 请求事件总线（net 请求生命周期 → CDP 事件）、getCookies/setCookie
   （接 cookie jar）
8. **Emulation 域** — setDeviceMetricsOverride（viewport）、setEmulatedMedia
9. **Node/Playwright 测试链** — pin 版本 + E2E 用例集 + 命令矩阵账本（evidence/）

### 排除（明确不在范围内）

- Chrome DevTools frontend serve 与面板可用性 —— **devtools goal**
- W3C WebDriver 协议维护 —— 已归档 goal 面（webdriver）
- Tracing / Profiler / Debugger 断点深域 —— 账本登记「不实现」，等点名
- 性能优化（CDP server 开销预算仅在 bench-gate 常规门禁内看）

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| apps/browser | headless.rs 职责拆分 + CDP 域实现；浏览器模式 CDP server 接线 | 文件大小约束驱动重构 |
| zero-protocol | Target 生命周期 IPC 消息（tab 创建/销毁/附接） | 照 SetViewport 先例 |
| engine/webview | console 对象化、请求事件总线、DOM/CSS 句柄桥 | CDP 域的页面侧支撑 |
| net | 请求生命周期事件发射点 | 只加观测点不改行为 |
| 测试链 | tests/ 下 Node + Playwright（pin）E2E 工程 + make 入口（test-guard 包裹） | Node 为测试-only 依赖 |
| 账本 | 命令矩阵（域 × 方法 × 三态 × 用例映射）持久化 evidence/ | 收口判据 |

### 依赖约束（run-rules §9 碰撞管理）

- **与 android-browser（活跃并行流）**：`apps/browser` 共享——碰前
  `git log --since="14 days ago" -- apps/browser/` 核对；发现要碰其活跃段即暂停记档。
- **与 devtools goal（本 goal 下游）**：devtools 只消费本 goal 的 CDP 面；它若需改 CDP
  域实现，须本 goal 已收口或碰头协调。
- **与 rendering-compat**：crate 零重叠（本流不触 css-parser/style-system/layout-engine/
  render-foundation）。
- **webdriver 归档遗产**：仅模式参考（集成测试形态、`zero-paint-convert` 截图路径复用），
  不共进程。

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: 命令矩阵账本

- [ ] 从 pin 版本 Playwright 导出其 `connectOverCDP` 全核心流实际发送的 CDP 命令全集
      （按域分组），落 `evidence/cdp-command-matrix.md`
- [ ] 矩阵逐方法三态登记（实现 / 部分 / 不实现）+ 对应验证用例映射（「实现」态每命令
      ≥1 个 E2E 用例）

### DC-2: Playwright E2E 套件全绿

- [ ] E2E 用例集覆盖：navigate / click / fill / keyboard / screenshot / cookies /
      console 消息采集 / network 事件 / dialog / frames（同进程 iframe）/ viewport
      emulation / evaluate
- [ ] 全套用例对 ZeroWeb CDP 端点全绿（双跑 deterministic）
- [ ] 连接生命周期健壮：connect → 操作 → close 无进程/句柄泄漏（重复连接、异常断开）

### DC-3: 协议健壮性

- [ ] 未实现命令返回 JSON-RPC 标准错误 `-32601`（含 method 回显），不静默、不崩溃
- [ ] 畸形输入（非法 JSON / 非法 params / 超大 payload）安全拒绝
- [ ] token/origin 校验语义保持；默认仅 loopback 监听

### DC-4: 测试与质量不可退让

- [ ] `make test` 全绿零失败；`cargo clippy --workspace --all-targets -- -D warnings`
      零警告；`cargo fmt` 干净
- [ ] Node/Playwright 测试链有 make 入口（test-guard 包裹）并在本地可一键执行；
      CI 可行性评估记账（Node 可用性与缓存面）
- [ ] headless 既有自动化面（wpt parity / smoke 入口）零回归

---

## 活跃里程碑

### M1 — 传输/发现/Target 基座 + Playwright 首连

**目标**：headless.rs 职责拆分（transport/discovery/domains/session）；Target 域
（setAutoAttach/attachToTarget/targetInfo）；Runtime.enable + evaluate（remoteObject 雏形）。
**验收**：Playwright `connectOverCDP` 成功附接 + `page.evaluate` 返回值正确。

### M2 — Page/Input 域 → 点击/填充/键盘/导航流

**目标**：Page 导航事件族 + getLayoutMetrics + dialog；Input dispatch 双域接 host 管线。
**验收**：Playwright `page.goto / click / fill / keyboard` 核心流绿。

### M3 — DOM/CSS/Emulation → locator 流

**目标**：DOM getDocument/querySelector 句柄桥 + CSS getMatchedStylesForNode +
Emulation viewport/media。
**验收**：Playwright locator 定位/断言流绿；viewport 设定生效（复用 ④ viewport 桥遗产）。

### M4 — Network/cookies/console 对象化

**目标**：net 请求事件总线 → Network 事件流；cookie jar → getCookies/setCookie；
console V8 侧结构化 → consoleAPICalled。
**验收**：Playwright network/cookies/console 断言流绿。

### M5 — 矩阵收口

**目标**：命令矩阵账本定稿 + E2E 双跑 deterministic + 挂账清单（不实现域）。
**验收**：DC-1~4 逐项判定。

---

## Final Output Protocol

### 输出规则

| 情况 | 输出 | 说明 |
|------|------|------|
| Done Criteria 全部满足 | `DONE` | 见下方「DONE 允许条件」 |
| 进展仍可推进 | `CONTINUE: <下一步>` | **这是默认输出** |
| 真正的外部阻塞 | `BLOCK: <原因>` | 罕见使用 |

### DONE 允许条件

**同时满足**：DC-1~4 全部满足；E2E 验证基于真实 Playwright 客户端对真实 CDP 端点
（无 mock 充数）；`cargo build` + `make test` + `cargo clippy` 全过；master.md 内部自洽，
evidence 账本持久化。

---

## Execution Protocol

### 自主执行原则

1. **自主勘察**：M1 前先以 pin 版 Playwright 对 Chromium 空跑一轮，导出命令全集建立
   矩阵初稿（零源码改动的纯资产切片）
2. **自主实现**：逐域收敛，每域 kill-switch 不需要（CDP 面为增量服务面，不影响页面管线），
   但 headless.rs 拆分与域扩展每切片独立 land + `make test`
3. **自主验证**：每切片跑对应 Playwright E2E；矩阵账本随域更新
4. **持续推动**直到 Done Criteria 全部满足

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮
2. **协议语义分歧**（Playwright 期望 vs CDP spec 文档）：以 Playwright 实际行为为准
   （它是验收客户端），分歧记账
3. **深结构拦路**（V8 对象句柄桥 / 多进程 Target 路由）：记「待用户决策」并跳过，
   先做其他域

---

## Document Control / Archive Policy

- **入口文档**（本文件）：定义 Mission、Done Criteria、执行协议和文档治理规则。
  仅在目标本身实质变化时修改；每轮执行不重写。
- **运行时控制平面** `docs/goal/cdp-protocol/master.md`：当前真实状态唯一控制面板。
- **归档区域** `docs/goal/cdp-protocol/archive/`：只追加不修改。
- **证据区域** `docs/goal/cdp-protocol/evidence/`：命令矩阵账本、E2E 报告，持续追加。
