# WebDriver 服务完善 — W3C 协议补齐目标

**版本**: v1.0
**日期**: 2026-09-07
**状态**: Active
**执行模式**: 轻量修复优先（永不停）；遇需用户决策项或深结构方向 → 记入「待用户决策」清单 → 跳过 → 继续其他轻量修复
**父目标**: `docs/goal/zero-web.md`（DC2 交互式网站可用性验证的自动化基建 + M11 端到端测试）

> **说明**
> 本文档是 ZeroWeb「WebDriver 服务完善」专项目标执行契约。`zero-webdriver` 已实现 W3C
> 协议 9 个 endpoint 的最小子集（session/navigate/title/find_element/click/send_keys/
> active_element/execute sync/delete session），经 `zero-protocol` Automation 消息族驱动
> live renderer 子进程；但相对 W3C 标准全集缺约 25+ 类 endpoint，且**零外部接线**
> （Makefile/CI/wpt-runner 均不使用）。目标是以 W3C WebDriver 规范为标准补齐核心 endpoint，
> 并把它接线为本仓自动化验证基建。本文定义 Mission、边界、Done Criteria、执行协议和文档
> 治理规则，供后续 `rally run` 会话作为稳定输入。日常进展、evidence、active milestone
> 更新写入 `master.md`。
>
> **▶ 拆分动机（2026-09-07 用户决策）**：从父目标拆出。理由：① 父目标 DC2 的已知缺口
> 「交互式网站可用性」需要可编程的端到端验证通道，WebDriver 是行业标准形态（W3C 协议），
> 做完即是其他 goal（keyboard-*、editing-contenteditable）的自动化验证基建 force
> multiplier；② 已有 9 endpoint 最小闭环（`a22261236` 曾用其驱动 live renderer），补齐是
> 增量而非新建；③ 改动域（apps/webdriver + protocol Automation 消息族）与 rendering-compat
> 渲染流域**零重叠**，可安全并行；④ 零外部接线意味着无回归风险面——先立 CI 门禁再扩面。
>
> **▶ 基线事实（2026-09-07 实测）**：
> - **已实现（9 endpoint）**：`apps/webdriver/src/main.rs`（299 行）`handle_request`
>   match L134 起 + `src/session.rs`（582 行）`Driver`：POST `/session`（create_session
>   L65，spawn `zero-renderer` 子进程）、POST `/session/{id}/url`（navigate L106）、
>   GET `/title`（L110）、POST `/element`（find_element L121）、POST `/element/{ref}/click`
>   （L133）、POST `/element/{ref}/value`（send_keys L141）、GET `/element/active`
>   （L149）、POST `/execute/sync`（L158）、DELETE `/session/{id}`（L102）。
> - **基建**：零依赖 HTTP 服务（单线程、loopback-only、默认 9515、CORS `*`）；元素引用
>   映射上限 4096；`parse_webdriver_keys`（L402，修饰键/特殊键）；Automation IPC 经
>   `zero-protocol`；错误码映射 no such session / no such element / stale element
>   reference → 404。
> - **测试**：`tests/http_session.rs` 集成测试（spawn 真实二进制 + 真实 TCP 全链路，
>   New Session / Navigate / Get Title / Delete Session）。
> - **缺失（25+ 类）**：GET `/status`、GET `/session/{id}`、GET `/url`、GET `/source`、
>   screenshot、cookies 全族、find_elements（复数）、元素 text/rect/enabled/attribute/
>   property/css、clear、frame/parent、window 全族（handles/rect/maximize/fullscreen）、
>   alert 全族、actions、timeouts、forward/back/refresh、execute/async、print。
> - **接线现状**：Makefile grep webdriver 零命中、CI 零命中、wpt-runner 不使用——
>   唯一消费者是自身集成测试与历史提交 `a22261236`。

---

## Mission

以 **W3C WebDriver 规范 + 上游 webdriver-tests 可执行面为验证标准**，把 `zero-webdriver`
从 9 endpoint 最小子集补齐到「驱动自动化验证够用」水平，并接线为本仓 CI 可用的端到端
验证基建。分阶段里程碑校准执行预期：

| 阶段 | 目标 | 说明 |
|---|---|---|
| 第一阶段 | **门禁就位** | webdriver 集成测试入 CI + endpoint 兼容性清单基线 |
| 中期 | **核心补齐** | 导航族（forward/back/refresh/status/url）、元素状态族（text/rect/attribute/property/css/enabled）、find_elements |
| 长期 | **自动化基建** | timeouts/execute async/screenshot/window 基础 + 作为其他 goal 的验证通道接线 |

**关键约束**：验收以 **W3C endpoint 行为语义**为准（请求/响应 JSON wire format、错误码、
成功条件），每补一个 endpoint 必须带 HTTP 全链路集成测试（照 http_session.rs 模式）。
不引入新依赖（零依赖 HTTP 服务是既有架构决策，维持）。

覆盖范围：

1. **会话与状态** — GET `/status`、GET `/session/{id}`（capabilities 回读）、timeouts
2. **导航族** — GET `/url`、forward、back、refresh
3. **元素交互族** — find_elements（复数）、text、rect、enabled、selected、attribute、
   property、css value、clear
4. **执行族** — execute/async、GET `/source`
5. **窗口与截图** — window handle(s)/rect、screenshot（依赖 renderer 截图能力评估）
6. **接线** — CI 集成测试 job、供兄弟 goal 使用的验证通道文档

执行方式：**endpoint 逐个补齐** — 每个 endpoint 一个切片（实现 + wire format 测试 + 全链路测试）。

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| HTTP 层 | main.rs 路由新增 endpoint、错误码映射 | 维持零依赖架构 |
| Driver 层 | session.rs Automation 命令扩展 | 经 zero-protocol 既有消息族 |
| 协议层 | Automation 消息族按需新增消息 | 需同步 apps/renderer 端处理（protocol crate 属共享面，见依赖约束） |
| 集成测试 | 每 endpoint HTTP 全链路测试（真实 TCP + 真实 renderer 子进程） | 照 http_session.rs 模式 |
| CI 接线 | webdriver 集成测试入既有 CI job | 只加不改既有 job 结构 |

### 不在范围内（明确排除）

- **W3C 远程端兼容性（WebDriver BiDi / CDP）** — node_modules 里的 chromium-bidi 是
  wpt-runner 的内部依赖，与本服务无关；不碰
- **alert 全族 / print** — 依赖 host 对话框/打印能力，先记「待用户决策」评估
- **actions（指针/触摸序列完整语义）** — 依赖输入管线深化，先做 keyboard 面够用的子集，
  完整 actions 记入后续
- **权限/权限提示相关 endpoint** — 浏览器 shell 域
- **多 session 并发语义深化** — 单 session 顺序执行是既有架构，维持

### 依赖约束

- **与 rendering-compat 流边界（run-rules §9）**：本流改动域 = `apps/webdriver/` +
  `crates/protocol/` Automation 消息族 + `apps/renderer/` Automation 处理端 + 本 goal
  控制面。与渲染流域 crate 域零重叠，但 **protocol/renderer 属多进程共享面**——碰之前
  `git log --since="14 days ago" -- crates/protocol/ apps/renderer/` 核对，有活跃编辑
  则先做零碰撞面（webdriver 自身 HTTP 层、集成测试）。
- **与 event-loop-spec 流**：apps/renderer 属该流活跃域（runtime.rs/page_scripts.rs）。
  本流只碰 renderer 的 Automation 消息处理段，不碰事件循环/observer tick 段；发现要碰
  即暂停记入 master.md（碰头信号）。
- **作为其他 goal 的消费基建**：keyboard-*/editing-contenteditable 后续可用本服务做
  自动化验证——本流提供能力，不替兄弟流写用例。

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: 门禁与基线

- [ ] webdriver 集成测试（现有 + 新增）入 CI（make test 或独立 job 步骤）
- [ ] W3C endpoint 兼容性清单基线（已实现/部分/缺失三态 + 行为注记）持久化到
      `docs/goal/webdriver/evidence/`
- [ ] W3C wire format（JSON 响应结构、错误码 404/400/500 语义）有对照测试

### DC-2: 核心 endpoint 补齐

- [ ] 会话与状态：GET /status、GET /session/{id}、timeouts
- [ ] 导航族：GET /url、forward、back、refresh
- [ ] 元素交互族：find_elements、text、rect、enabled、selected、attribute、property、
      css value、clear
- [ ] 执行族：execute/async、GET /source

### DC-3: 每端点全链路测试

- [ ] 每个新增 endpoint 有 HTTP 全链路集成测试（真实 TCP + 真实 renderer 子进程，
      照 http_session.rs 模式）
- [ ] 错误路径有测试（no such element、stale element reference、invalid argument）

### DC-4: 测试与质量不可退让

- [ ] `make test` 全绿，零失败
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 零警告
- [ ] W3C 兼容性清单随 endpoint 落地持续更新（evidence/ 可追溯）

---

## 活跃里程碑

### M1 — 门禁就位 + 会话/导航族

**目标**：CI 接线 + 兼容性清单基线；GET /status、GET /session/{id}、timeouts、
GET /url、forward/back/refresh。

**切片建议**：
1. 兼容性清单基线（对照 W3C 规范全集逐项标注——纯文档，零源码改动）
2. GET /status + GET /session/{id}（不碰 renderer，最薄切片）
3. 导航族（Automation 消息按需扩展）

### M2 — 元素交互族 + 执行族

**目标**：find_elements + 元素状态族全 endpoint + execute/async + /source。

### M3 — 窗口/截图评估 + 接线收尾

**目标**：window handle(s)/rect、screenshot 能力评估（renderer 截图链路摸底，可行则做，
不可行记「待用户决策」）、作为兄弟 goal 验证通道的使用文档 → DC 全满足判定。

---

## Final Output Protocol

### 输出规则

| 情况 | 输出 | 说明 |
|------|------|------|
| Done Criteria 全部满足 | `DONE` | 见下方"DONE 允许条件" |
| 进展仍可推进 | `CONTINUE: <下一步>` | **这是默认输出** |
| 真正的外部阻塞 | `BLOCK: <原因>` | 罕见使用 |

### DONE 允许条件

**同时满足**：DC-1~4 全部满足；每个新增 endpoint 有全链路测试；`make test` +
`cargo clippy` 全通过；master.md 内部自洽，archive 已建立。screenshot/alert 等按
「待用户决策」明确记录不算未满足 DC。

---

## Execution Protocol

### 自主执行原则

1. **自主探索**renderer Automation 消息族现有能力（每个新 endpoint 先查协议面够不够）
2. **自主补齐** endpoint，实现 + wire format 测试 + 全链路测试同步交付
3. **自主验证**：`make test` + clippy + 全链路测试确认行为正确
4. **自主更新**兼容性清单（evidence/）
5. **持续推动**，直到 Done Criteria 全部满足

### 轻量修复优先

1. **主线 = 轻量修复**：一个 endpoint 一个切片，根因清楚、改动面小。
2. **永不停**：遇需拍板事项（screenshot 链路、actions 完整语义）记「待用户决策」清单
   并跳过，继续下一个 endpoint。
3. **碰撞管理**：protocol/renderer 是多进程共享面（event-loop-spec 流活跃域）——碰之前
   `git log` 核对；有活跃编辑则先做 webdriver 自身 HTTP 层与测试。

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮。当作当前任务的一部分修复，直到稳定可重复。
2. **endpoint 行为分歧**：以 W3C 规范文本为准，不确定处记录到 master.md 并选保守实现。
3. **技术决策**：在 master.md 中记录关键决策及其理由。

---

## Document Control / Archive Policy

- **入口文档**（本文件）：定义 Mission、Done Criteria、执行协议和文档治理规则。**修改条件**：
  仅在目标本身发生实质性变化时修改。**禁止行为**：每轮执行不重写本文件。
- **运行时控制平面** `docs/goal/webdriver/master.md`：当前真实状态的唯一控制面板。
  治理规则：持续演进、不允许无限增长（过时内容压缩或归档）、各章节必须自洽。
- **归档区域** `docs/goal/webdriver/archive/`：只追加不修改。
- **证据区域** `docs/goal/webdriver/evidence/`：兼容性清单、测试证据，持续追加。
