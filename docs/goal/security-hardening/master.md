# 安全加固 — 运行时控制面板（master.md）

**入口文档**: [../security-hardening.md](../security-hardening.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-12（立项）

---

## 当前状态

**专项定位**：zero-security 从「CSP 基础」推进到「CSP 主要指令完整 + report-only +
违规报告」+ Mixed Content 分级阻止 + HSTS + Permissions 语义层。WPT 三 corpus 为
验收标尺；CSP/Mixed Content 默认行为变更走 kill-switch + A/B 门禁（event-loop-spec ②
先例）。

**与兄弟 goal 的边界**：
- web-api-batch2 — 其 Clipboard 依赖本 goal DC-4 权限语义层（供数关系）；碰 engine
  shim 面 git log 核对
- cdp-protocol / devtools — CSP 违规 console 报告面共享管线，消费侧协调
- rendering-compat — crate 零重叠
- android-browser — 无共享面

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | zero-security CSP 现状盘点（指令覆盖面/report-only/上报语义） | ⏳ M1 前置勘察 |
| P2 | content-security-policy / mixed-content / secure-contexts 三 corpus 导入 + 基线 | ⏳ M1 纯资产 |
| P3 | CSP 主要指令引擎完整化 + report-only + 违规报告 | ⏳ M2 |
| P4 | Mixed Content 分级阻止 + HSTS | ⏳ M3 |
| P5 | Permissions API headless 语义层 + 事件 | ⏳ M4 |
| P6 | kill-switch + A/B + default-on 决策 | ⏳ M5 |

## 已完成切片

（立项轮，暂无）

## 下一步计划

1. **M1**：zero-security CSP 面勘察（零源码）→ 三 corpus fetch 脚本 + 导入 + 基线
   （照 fetch-observers-subset.sh 先例）
2. **M2-M4**：按入口文档里程碑逐面收敛

**待用户决策清单**：
- （暂无）

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — 勘察 + WPT 导入与基线 | ⏳ |
| M2 — CSP 指令引擎完整化 | ⏳ |
| M3 — Mixed Content + HSTS | ⏳ |
| M4 — Permissions 语义层 | ⏳ |
| M5 — 收口 | ⏳ |

## 验证基线

- 测试基线：立项时点全绿（`make test` 19,170P/0F，2026-09-12 口径；经 test-guard）
- 现状：zero-security 有 CORS/CSP 基础/同源策略/沙箱（CLAUDE.md 口径）；CSP 指令
  覆盖面/report-only/HSTS/Mixed Content/permissions 待勘察定界
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过；
  行为变更 kill-switch + 全量 A/B 零回归

**碰撞管理**：碰 engine shim 资源加载检查点前与 web-api-batch2 / cdp-protocol
`git log` 互核。
