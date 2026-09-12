# 导航与浏览上下文兼容 — history / navigation / iframe

**版本**: v1.0
**日期**: 2026-09-12
**状态**: Active（已立项，未启动——启动顺序由用户点名；M3 深结构切片须用户门控）
**执行模式**: WPT 驱动（上游导航面 corpus 为验收标尺）+ 语义修齐；
iframe 浏览上下文为**用户门控深结构切片**（先例：R1043 vertical-mode / Phase A IFC）
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——导航与浏览上下文面）

> **说明**
> 本文档是 ZeroWeb「导航与浏览上下文兼容」专项目标执行契约。会话历史（前进/后退）、
> 导航事件、iframe 嵌入是真实网页的基础设施（广告/视频/社交组件无处不在）；本目标
> 按「轻量语义先行、深结构门控」两段推进。
>
> **▶ 拆分动机（2026-09-12 用户决策，四 goal 同批立项之一）**：① 真实可用杀伤力
> 排序第二——iframe 嵌入与历史导航是页面基础设施；② P1a 已落 location 读侧 +
> pushState 面，语义一致性从未度量；③ iframe 树是深结构，须显式立项 + 门控，
> 避免渲染流误触。
>
> **▶ 基线事实（2026-09-12 实测）**：
> - **History/事件 JS 面**：shim 内 pushState 14 处 / popstate 17 / hashchange 23 /
>     sessionHistory 0——面部分存在，WPT history/ corpus 未度量
> - **iframe 现状**：dom crate 仅 parser/serializer 认得元素
>     （crates/dom/src/tests/tests_7_parser.rs / serializer.rs）；**engine 无浏览
>     上下文树（frame tree）**——嵌套 document、window.frames、跨 frame target 全缺
> - **location 读侧**：zero-web P1a 已落（见 rendering-compat/master.md R2921 记录）
> - **WPT corpora**：`html/browsers/` `history/` `navigation-api/` +
>     `html/semantics/embedded-content/the-iframe-element-*`（testharness 为主）

---

## Mission

以 **WPT 导航面真实用例为验收标准**，先收敛会话历史与导航事件语义（轻量面），
再以门控切片落地 iframe 浏览上下文（深结构面），使「多页面 + 嵌入内容」类站点
的基础导航行为正确。

**关键约束**：
1. **WPT 标尺先行**：M1 纯资产切片零源码改动
2. **iframe 深结构须用户门控**：frame tree 触 engine 页面管线重构，M3 启动前
   须用户点名批准（先例：R1043 / Phase A IFC 门控）
3. **轻面/深面分序**：M2（history/事件）不依赖 M3，可独立收口出数字

覆盖范围：

1. **Session history** — pushState/replaceState/state、history.length/back/forward/go、
   会话历史条目语义
2. **导航事件** — popstate/hashchange/navigatesuccess 等 navigation-api 评估面、
   location 写侧（赋值导航语义）
3. **iframe 浏览上下文**（门控切片）— 嵌套 browsing context 树、contentWindow/
   contentDocument、window.frames/parent/top、frame 元素属性语义

### 排除（明确不在范围内）

- **跨进程 fission** —— frame 的多进程归属触 zero-protocol 契约，如触先与
  protocol/goal-blockers 协商，不在本 goal 单方面推进
- **bfcache / 页面恢复** —— 未立项域，挂账
- **srcset/响应式图** —— rendering-compat 挂账域
- **iframe 渲染合成（OOIF）** —— 合成层面归 compositor 域，本 goal 只做文档/语义面

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| engine shim | history/location 写侧/导航事件语义 | part 系 JS 语义 |
| engine | 导航管线（assign/reload/back-forward 语义）+ 门控切片的 frame 树 | 深面按切片门控 |
| dom | iframe 元素属性面（src/name/sandbox 语义评估） | 不改 parser 核心 |
| WPT 资产 | html/browsers + history + navigation-api + iframe 子集导入 | fetch 脚本 + 账本 |

### 依赖约束（run-rules §9 碰撞管理）

- **与 rendering-compat**：viewport/滚动/渲染面不碰（crates/style-system、
  layout-engine、render-foundation 不在 envelope）；iframe 触发渲染面缺口时记账回流。
- **与 event-loop-spec（已归档）**：其 IO/RO 与事件循环遗产为消费基础，非碰撞。
- **与 zero-web P1a**：location 读侧已落——写侧导航语义归本 goal，改动走本 goal 账本。
- **与 zero-protocol / 多进程**：fission 排除（见上）；如 M3 触进程模型，BLOCK 上报
  等用户决策，不单方面改契约。

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: WPT 导入与基线

- [ ] html/browsers + history + navigation-api + iframe corpus window 可执行子集导入
- [ ] 分类通过率基线落 evidence/，并回填 docs/compat/trends/wpt-suites.csv planned 行

### DC-2: 会话历史与导航事件收敛（轻面）

- [ ] history pushState/replaceState/state/length/back/forward/go 语义修齐
- [ ] popstate/hashchange 事件序 + location 写侧导航语义修齐

### DC-3: iframe 浏览上下文（深面，用户门控）

- [ ] 门控获批后：嵌套 browsing context 最小面（contentWindow/contentDocument/
      frames/parent/top）+ iframe 元素属性语义
- [ ] 未获批期间：本 DC 保持 pending，不阻塞 DC-2/DC-4 的收口记账

### DC-4: 测试与质量不可退让

- [ ] `make test` 全绿零失败 + clippy `-D warnings` + fmt 干净
- [ ] 每语义切片带单测/集成测试；`make reftest` 零回归

---

## 活跃里程碑

### M1 — WPT 导入与基线

**目标**：四 corpus fetch 脚本 + 导入 + 基线（纯资产零源码改动）；corpus fetch 用编号脚本 `tests/wpt-runner/scripts/goals/60-navigation-compat.sh`（索引 README；幂等续拉，GitHub API 限流时部分目录跳过属预期）。

### M2 — 会话历史与导航事件

**目标**：history/location 写侧/导航事件逐簇修齐（轻面独立出数字）。

### M3 — iframe 浏览上下文（用户门控）

**目标**：frame tree 最小面 + iframe 属性语义；**启动前须用户点名批准**。

### M4 — 收口

**目标**：DC 逐项判定（DC-3 未获批时如实标注 pending）+ bfcache/fission 挂账定稿。

---

## Final Output Protocol

### 输出规则

| 情况 | 输出 | 说明 |
|------|------|------|
| Done Criteria 全部满足 | `DONE` | 见下方「DONE 允许条件」 |
| 进展仍可推进 | `CONTINUE: <下一步>` | **这是默认输出** |
| 真正的外部阻塞 | `BLOCK: <原因>` | 罕见使用 |

### DONE 允许条件

**同时满足**：DC-1/2/4 满足 + DC-3 满足**或**用户明确豁免；验证基于上游真实 WPT
用例；`cargo build` + `make test` + `cargo clippy` 全过；master.md 自洽，evidence
持久化；bfcache/fission 挂账定稿。

---

## Execution Protocol

### 自主执行原则

1. **基线先行**：M1 纯资产切片零源码改动
2. **轻面先行**：M2 不等 M3 门控，独立推进出数字
3. **门控纪律**：M3 深结构未经用户点名不启动（8 轮净负回退先例的教训成文化）

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮
2. **frame 树牵出一Ini渲染面缺口**：记账回流 rendering-compat，不越界
3. **location 写侧导航死循环风险**：导航触发须有重入 guard，切片带防回归单测

---

## Document Control / Archive Policy

- **入口文档**（本文件）：定义 Mission、Done Criteria、执行协议和文档治理规则。
  仅在目标本身实质变化时修改；每轮执行不重写。
- **运行时控制平面** `docs/goal/navigation-compat/master.md`：当前真实状态唯一控制面板。
- **归档区域** `docs/goal/navigation-compat/archive/`：只追加不修改。
- **证据区域** `docs/goal/navigation-compat/evidence/`：WPT 基线/修齐账本，持续追加。
