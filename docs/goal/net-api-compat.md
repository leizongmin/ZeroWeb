# 网络 API 兼容 — fetch / XHR / URL / Streams / EventSource

**版本**: v1.0
**日期**: 2026-09-12
**状态**: Active（已立项，未启动——启动顺序由用户点名）
**执行模式**: WPT 驱动（上游网络面 corpus 为验收标尺）+ 语义修齐；
照 event-loop-spec / web-components / web-api-batch2 通用打法
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——网络 API 面）

> **说明**
> 本文档是 ZeroWeb「网络 API 兼容」专项目标执行契约。SPA / 登录 / 一切 API 调用
> 依赖 fetch/XHR 面；本目标把该面从「shim 有实现但从未度量」推进到「WPT 一致性
> 可追踪」。
>
> **▶ 拆分动机（2026-09-12 用户决策，四 goal 同批立项之一）**：① 真实可用浏览器
> 杀伤力排序第一——现代网页没有 fetch 面基本不可用；② shim 已有实现地基
> （fetch/Request/Response 面存在），立项即可跑基线出数字，投入产出比最高；
③ 2026-09-12 官网测试集总表（docs/compat/trends/wpt-suites.csv）建立后，
> 非 CSS 面需要持续有 goal 收口回填 planned 行。
>
> **▶ 基线事实（2026-09-12 实测，crates/engine/src/js_dom_shim/ 共 61,575 行 grep 计数）**：
> - **JS 面已存在、从未度量**：fetch 156 处 / Request 410 / Response 100 / Headers 56 /
>     XMLHttpRequest 28 / FormData 25 / EventSource 14——一致性未知，WPT fetch/、xhr/
>     corpus 存在，启动 M1 即出基线
> - **Streams 底座**：ReadableStream 19 / WritableStream 15 / TransformStream 16 /
>     TextDecoderStream 9 处——WPT streams/ 一致性未测
> - **URL API**：zero-web P1a 已落（解析 + setter + 双向 searchParams，见
>     rendering-compat/master.md R2921 记录）——`url/` corpus 可直接度量
> - **WebSocket：全 shim 0 处，完全缺**——需宿主层 socket 升级握手 + 帧协议
>     （zero-net 有 TCP/TLS 底座），挂二期切片（重入条件见 Mission 约束）
> - **mimesniff**：zero-net 已有 Content-Type 判别基础，WPT mimesniff/ 面未对齐

---

## Mission

以 **WPT 网络面真实用例为验收标准**（`fetch/` `xhr/` `url/` `mimesniff/` `streams/`
`eventsource/`），把 ZeroWeb 的 JS 网络 API 面收敛到可追踪一致性，使 SPA / API 调用
场景不再因网络语义缺位而不可用。

**关键约束**：
1. **WPT 标尺先行**：M1 纯资产切片（fetch 脚本 + 导入 + 基线，零源码改动）
2. **WebSocket 不在本目标首期**：需宿主 socket 面 + 升级握手/帧协议；二期切片，
   重入条件 = 用户点名或 zero-net WS 底座就位
3. **CSP/策略执行不在本目标**：协议语义归本 goal，策略检查归 security-hardening

覆盖范围：

1. **fetch** — Request/Response/Headers/Body（含流式 body）/fetch 方法语义/错误面
2. **XHR** — XMLHttpRequest 全状态机（readyState/responseType/事件序）
3. **URL** — URL/URLSearchParams 边缘语义（P1a 已落面上修齐）
4. **mimesniff** — MIME 解析对齐 WHATWG 规则
5. **streams** — Readable/Writable/Transform 流底座一致性（fetch body 的依赖）
6. **EventSource** — SSE 解析与重连语义

### 排除（明确不在范围内）

- **WebSocket** —— 二期切片（见 Mission 约束 2）
- **CSP / mixed-content / 策略执行** —— security-hardening goal 域
- **HTTP 栈本身** —— zero-net 已有；本 goal 只收敛其上 JS 语义
- **Service Worker 拦截 fetch** —— service-workers goal 已归档收口，扩展另行立项

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| engine shim | fetch/Request/Response/Headers/XHR/EventSource 语义修齐 | part 系 JS 语义 |
| engine | 网络 JS 桥（请求派发 → zero-net → 响应回填） | 不改 zero-net 协议栈核心 |
| WPT 资产 | 六 corpus 可执行子集导入 | fetch 脚本 + 账本（照 observers/fs 先例） |
| 测试 | 单测 + 集成 + 上游 corpus 通道（make testharness-*） | 照既有先例 |

### 依赖约束（run-rules §9 碰撞管理）

- **与 security-hardening**：CSP 对 fetch 的策略执行归其；本 goal 提供语义钩子位，
  不实现策略判定。碰 engine shim 面 git log 互核。
- **与 service-workers（已归档）**：其 fetch 通道已收口；如 SW 拦截语义需扩展，
  另行记账不越界。
- **与 zero-web P1a**：URL/URLSearchParams 既有实现为本 goal 修齐对象，非碰撞——
  改动走本 goal 账本。
- **与 rendering-compat / 其他渲染流**：无共享 crate 面（网络语义不触
  style-system/layout-engine/painter）。

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: WPT 导入与基线

- [ ] 六 corpus（fetch/xhr/url/mimesniff/streams/eventsource）window 可执行子集导入
      （fetch 脚本 + 常驻通道 + imported 账本）
- [ ] 分类通过率基线（文本 + JSON）落 evidence/，并回填
      docs/compat/trends/wpt-suites.csv planned 行

### DC-2: fetch 面收敛

- [ ] Request/Response/Headers/Body + fetch 方法语义逐簇修齐，通过率有可追踪提升
- [ ] 错误面（网络错误/abort/超时语义）与 spec 对齐

### DC-3: XHR / EventSource / URL / mimesniff / streams 收敛

- [ ] XHR 状态机与事件序修齐；EventSource 解析/重连；URL 边缘语义；mimesniff 对齐
- [ ] streams 底座一致性（fetch body 流式读的基础）

### DC-4: 测试与质量不可退让

- [ ] `make test` 全绿零失败 + clippy `-D warnings` + fmt 干净
- [ ] 每语义切片带单测/集成测试；`make reftest` 零回归（网络面不触无关节点渲染）

---

## 活跃里程碑

### M1 — WPT 导入与基线

**目标**：六 corpus fetch 脚本 + 导入 + 基线（纯资产零源码改动）；corpus fetch 用编号脚本 `tests/wpt-runner/scripts/goals/20-net-api-compat.sh`（索引 README；幂等续拉，GitHub API 限流时部分目录跳过属预期）。

### M2 — fetch 面收敛

**目标**：fetch/Request/Response/Headers 逐簇修齐（预期最大簇）。

### M3 — XHR / EventSource / URL / mimesniff

**目标**：XHR 状态机 + SSE + URL/mimesniff 边缘语义。

### M4 — streams + 收口

**目标**：streams 一致性 + WebSocket 二期切片挂账定稿 + DC 逐项判定。

---

## Final Output Protocol

### 输出规则

| 情况 | 输出 | 说明 |
|------|------|------|
| Done Criteria 全部满足 | `DONE` | 见下方「DONE 允许条件」 |
| 进展仍可推进 | `CONTINUE: <下一步>` | **这是默认输出** |
| 真正的外部阻塞 | `BLOCK: <原因>` | 罕见使用 |

### DONE 允许条件

**同时满足**：DC-1~4 全部满足；验证基于上游真实 WPT 用例（本地等价用例如实标注）；
`cargo build` + `make test` + `cargo clippy` 全过；master.md 内部自洽，evidence 持久化；
WebSocket 二期切片挂账已定稿。

---

## Execution Protocol

### 自主执行原则

1. **基线先行**：M1 纯资产切片零源码改动
2. **逐簇收敛**：WPT 失败聚类 → 逐簇修齐 → 账本更新 → suites CSV 回填
3. **跨域记账**：CSP 策略面缺口记账回流 security-hardening，不越界硬改

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮
2. **zero-net 栈层缺陷**：如根因在 HTTP 栈，记账并评估最小修——不动协议栈核心契约
3. **streams 深依赖**：如 fetch body 流式语义需 streams 深改，先落 buffer 路径并记账

---

## Document Control / Archive Policy

- **入口文档**（本文件）：定义 Mission、Done Criteria、执行协议和文档治理规则。
  仅在目标本身实质变化时修改；每轮执行不重写。
- **运行时控制平面** `docs/goal/net-api-compat/master.md`：当前真实状态唯一控制面板。
- **归档区域** `docs/goal/net-api-compat/archive/`：只追加不修改。
- **证据区域** `docs/goal/net-api-compat/evidence/`：WPT 基线/修齐账本，持续追加。
