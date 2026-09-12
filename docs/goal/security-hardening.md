# 安全加固 — CSP 完整实现 / Mixed Content / HSTS / 权限模型（M13 面）

**版本**: v1.0
**日期**: 2026-09-12
**状态**: Active
**执行模式**: WPT 驱动（上游 content-security-policy corpus 为验收标尺）+ 规范逐条；
遇深结构（站点隔离等多进程重构）→ 记「待用户决策」→ 跳过 → 继续其他面
**父目标**: `docs/goal/zero-web.md`（M13 性能优化 + 安全加固；DC Tier 1「安全」深化）

> **说明**
> 本文档是 ZeroWeb「安全加固」专项目标执行契约。`zero-security` 已有 CORS/CSP 基础/
> 同源策略/沙箱边界，本目标把安全模型从「基础」推进到「完整」：CSP 主要指令全语义 +
> report-only、Mixed Content 分级阻止、HSTS、权限模型语义层。
>
> **▶ 拆分动机（2026-09-12 用户决策）**：① M13 安全交付物明确未拆、范围可契约化；
> ② 上游 WPT `content-security-policy` corpus 巨大，验收标尺现成；③ `zero-security`
> 单 crate 主导、与渲染流域零重叠，适合独立并行。
>
> **▶ 基线事实（2026-09-12 实测）**：
> - **zero-security**：CORS / CSP 基础 / 同源策略 / 渲染进程沙箱已有（CLAUDE.md 架构
>   指南口径）；CSP 指令覆盖面、report-only、违规上报语义待实测盘点（M1 前置勘察）。
> - **net**：HTTPS 客户端 + cookie jar 在；HSTS 缺、Mixed Content 阻止缺（资源加载
>   语义在 engine/net 交界，勘察定界）。
> - **权限模型**：无 permissions API 面；剪贴板/通知等 capability 无权限语义
>   （web-api-batch2 goal 的 Clipboard 依赖本目标 DC-4 语义层或本地最小语义，见其边界）。
> - **验收标尺**：上游 WPT `content-security-policy/`（数百案）+ `mixed-content/` +
>   `secure-contexts/` 子集可导入（fetch 脚本照 observers/fs 先例）。

---

## Mission

以 **WPT 真实用例 + 规范逐条为验收标准**，把安全模型收敛到 Tier 1+ 水平：CSP 可有效
阻止违规资源加载（含 report-only 模式）、Mixed Content 按规范分级阻止、HSTS 强制
升级、permissions 语义层就位。

**关键约束**：
1. **WPT 标尺先行**：导入基线 → 逐簇修齐，照 rendering-compat/goal 通用打法
2. **零回归红线**：CSP/Mixed Content 是**资源加载行为变更**——默认行为变更须 kill-switch
   + 全量 A/B 零回归（DC-3 面照 event-loop-spec 门禁先例）
3. **站点隔离不在本目标**：深多进程重构，挂账等用户点名

覆盖范围：

1. **CSP 完整实现** — fetch/frame 主要指令（default-src/script-src/style-src/img-src/
   connect-src/frame-src/font-src/media-src/object-src/base-uri/form-action/frame-ancestors）
   + report-only 模式 + 违规报告（console/report-uri 面）
2. **Mixed Content** — HTTPS 页面 HTTP 子资源分级阻止（blockable/optionally-blockable）
3. **HSTS** — Strict-Transport-Security 解析/存储/强制升级 + max-age/.includeSubDomains
4. **权限模型基础** — Permissions API 语义层（query/state/request 的 headless 语义 +
   事件）；权限提示 UI 联动 desktop-browser goal（本目标只做语义层）

### 排除（明确不在范围内）

- 站点隔离（跨站 iframe 独立渲染进程）——深多进程，挂账等点名
- 证书钉扎 / TLS 栈深化 —— net 选型域
- 权限提示 UI —— desktop-browser goal 面（本目标供语义层）
- Credential Management / WebAuthn —— 未立项域

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| zero-security | CSP 指令引擎完整化、report-only、违规报告 | 单 crate 主导 |
| engine | CSP/Mixed Content 检查点接入资源加载决策（consume 侧） | 只加决策点不改加载管线 |
| net | HSTS 存储/升级、Mixed Content 判定支撑 | 观测与策略面 |
| WPT 资产 | content-security-policy / mixed-content / secure-contexts 子集导入 | fetch 脚本 + 账本 |
| 测试 | kill-switch + A/B 门禁、单测、语义层集成测试 | 行为变更门禁照先例 |

### 依赖约束（run-rules §9 碰撞管理）

- **与 rendering-compat**：crate 零重叠（css-parser/style-system 等不碰）。
- **与 web-api-batch2（同日新立）**：其 Clipboard 依赖权限语义——DC-4 语义层为它
  供数；两边碰 `engine` shim 面时 git log 核对。
- **与 cdp-protocol / devtools**：CSP 违规经 console 报告面与 Runtime.consoleAPICalled
  共享管线——消费侧协调，不互改。
- **与 android-browser**：无共享面（apps/android-browser 不触）。

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: WPT 导入与基线

- [ ] 上游 `content-security-policy` / `mixed-content` / `secure-contexts` window 可执行
      面子集导入（fetch 脚本 + 常驻通道 + imported 账本）
- [ ] 分类通过率基线（文本 + JSON）落 evidence/

### DC-2: 四面语义收敛

- [ ] CSP：主要指令全语义（清单见覆盖范围）+ report-only + 违规报告；WPT 驱动逐簇
      修齐，通过率有可追踪提升
- [ ] Mixed Content：分级阻止语义 + WPT 修齐
- [ ] HSTS：解析/存储/强制升级 + 单测（含 includeSubDomains/subdomain 剔除语义）
- [ ] Permissions：query/state/request headless 语义层 + change 事件 + 单测

### DC-3: 行为变更门禁

- [ ] CSP/Mixed Content 默认行为变更走 kill-switch + 全量 A/B 零回归后 default-on
      （照 event-loop-spec ② 先例，门禁记录落 evidence/）
- [ ] 默认关闭期零影响验证（switch OFF 时全量 `make test` 零 delta）

### DC-4: 测试与质量不可退让

- [ ] `make test` 全绿零失败 + clippy `-D warnings` + fmt 干净
- [ ] 每语义切片带单测/集成测试；`make reftest` 零回归（CSP 不影响无 CSP 页面渲染）

---

## 活跃里程碑

### M1 — 勘察 + WPT 导入与基线

**目标**：zero-security CSP 现状盘点 + 三 corpus 导入 + 通过率基线（纯资产切片先行）。

### M2 — CSP 指令引擎完整化

**目标**：主要指令语义 + report-only + 违规报告；WPT 逐簇修齐；kill-switch + A/B。

### M3 — Mixed Content + HSTS

**目标**：分级阻止 + HSTS 存储/升级；WPT 修齐。

### M4 — Permissions 语义层

**目标**：Permissions API headless 语义 + 事件；为 web-api-batch2 Clipboard 供数。

### M5 — 收口

**目标**：default-on 决策（A/B 零回归后）+ DC 逐项判定 + 挂账。

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
`cargo build` + `make test` + `cargo clippy` 全过；master.md 内部自洽，evidence 持久化。

---

## Execution Protocol

### 自主执行原则

1. **基线先行**：M1 纯资产切片零源码改动
2. **行为变更走门禁**：kill-switch → A/B 零回归 → default-on，不硬推
3. **逐簇收敛**：WPT 失败聚类 → 逐簇修齐 → 账本更新

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮
2. **A/B 有回归**：回退本切片记录结论，换下一切片
3. **跨域归因**（资源加载管线）：记账协调，不越界硬改

---

## Document Control / Archive Policy

- **入口文档**（本文件）：定义 Mission、Done Criteria、执行协议和文档治理规则。
  仅在目标本身实质变化时修改；每轮执行不重写。
- **运行时控制平面** `docs/goal/security-hardening/master.md`：当前真实状态唯一控制面板。
- **归档区域** `docs/goal/security-hardening/archive/`：只追加不修改。
- **证据区域** `docs/goal/security-hardening/evidence/`：WPT 基线/修齐账本、A/B 门禁记录。
