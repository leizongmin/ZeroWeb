# 编码兼容 — 运行时控制面板（master.md）

**入口文档**: [../encoding-compat.md](../encoding-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-02（收口——DC-1~4 全满足）

## 当前状态

**目标 Completed（2026-10-02，M4 收口判定）**：DC-1~4 全满足，全语料 36 案
**12070/12075 = 99.96%**（轨迹 6.1% → 98.4% → 99.96%）。门禁：make test 68 suites
全绿 + clippy -D warnings + fmt 干净 + make reftest 700/700 零不一致。
判定文档：[evidence/2026-10-02-m4-dc-verdict.md](evidence/2026-10-02-m4-dc-verdict.md)。

残差 5/12075 已分类定稿：detached-AB/SAB transfer 结构面 ×4（重入 = transfer/
structured-clone 管道落地）+ encoding_rs iso-2022-jp fatal-stream 状态契约 ×1（随
encoding_rs 升级重评）。文档级编码嗅探面已双向记账转 html-syntax-compat（其控制面 P5）。

里程碑台账：M1（10-01）corpus 通道 + 基线 6.1%；M2（10-02）labels 全表 + legacy 解码
（encoding_rs host native + 有状态 decoder 表）98.4%；M3（10-02）utf-8 spec 状态机 +
TextEncoder/streams 语义 99.96%。逐轮证据见 evidence/（m1-baseline / m2-labels-legacy-
decode / m3-textencoder-modes-streams / m4-dc-verdict）。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | encoding/ corpus 导入 + 基线 | ✅ M1（2026-10-01） |
| P2 | labels 标签匹配全表（数据化） | ✅ M2（api-invalid-label 3421/3421） |
| P3 | TextDecoder legacy 编码解码 | ✅ M2（legacy-mb 100%、single-byte 336/336、iso-2022-jp 34/34） |
| P4 | TextEncoder + BOM/fatal/ignore 模式 + 编码往返 | ✅ M3（99.96%；残差 5 记账） |
| P5 | 文档级编码嗅探挂账定稿 | ✅ M4（转 html-syntax-compat P5，双向记账 2026-10-02） |

## 后续重入条件（记账，非待办）

- transfer/structured-clone 管道落地（workers/navigation 流域）→ 残差 4 案（detached-AB/
  SAB 面）可清偿。
- encoding_rs 升级（>0.8.35）→ iso-2022-jp fatal-stream 状态契约残差 1 案重评。
- html-syntax-compat M1 落地 → 文档级嗅探面（bom-handling/eof-*/utf-32*/sniffing，
  wpt-data 已拉取）随其 corpus 一并基线。
- 数字维护：全量复跑 `make testharness-encoding` 后向 wpt-suites.csv 追加行。

**待用户决策清单**：（空）
