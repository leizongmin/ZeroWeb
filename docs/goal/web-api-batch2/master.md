# Web API 第二批 — 运行时控制面板（master.md）

**入口文档**: [../web-api-batch2.md](../web-api-batch2.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-12（立项）

---

## 当前状态

**专项定位**：M12 余面收编——Clipboard API + Fullscreen API 语义落地，WPT 两 corpus
为验收标尺。DnD 排除挂账（宿主拖拽输入管线深依赖）。

**与兄弟 goal 的边界**：
- security-hardening — 权限语义供数关系（本 goal 最小权限查询面，其 DC-4 落地时对齐）
- rendering-compat — `:fullscreen` 样式面跨域记账，`crates/style-system` 不碰
- cdp-protocol / devtools — 无协议面耦合
- android-browser / desktop-browser — 全屏窗口语义触 host-runtime 平台面时 git log 核对

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | clipboard-apis / fullscreen 两 corpus fetch 脚本 + 导入 + 基线 | ⏳ M1 纯资产 |
| P2 | navigator.clipboard 四方法 + ClipboardEvent + 内存后端 | ⏳ M2 |
| P3 | 最小权限查询面（security-hardening DC-4 对齐点） | ⏳ M2 |
| P4 | Fullscreen 事件/状态面 + viewport 联动 | ⏳ M3 |
| P5 | 平台剪贴板后端（host-runtime 能力评估）或差异记账 | ⏳ M4 挂账定稿 |

## 已完成切片

（立项轮，暂无）

## 下一步计划

1. **M1**：两 corpus fetch 脚本（照 fetch-observers-subset.sh 先例）+ 导入 + 通过率
   基线（零源码改动纯资产）
2. **M2-M4**：按入口文档里程碑逐面收敛

**待用户决策清单**：
- （暂无）

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT 导入与基线 | ⏳ |
| M2 — Clipboard 语义 + 后端 | ⏳ |
| M3 — Fullscreen 语义 | ⏳ |
| M4 — 收口 | ⏳ |

## 验证基线

- 测试基线：立项时点全绿（`make test` 19,170P/0F，2026-09-12 口径；经 test-guard）
- 现状：navigator.clipboard / Fullscreen 面全缺；WPT clipboard-apis / fullscreen
  corpus 未导入（wpt-data 无目录、imported 账本零命中）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过

**碰撞管理**：碰 engine shim 面前与 security-hardening / cdp-protocol `git log` 互核。
