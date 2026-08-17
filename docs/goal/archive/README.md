# Goal 文档归档

已完成（或 scope 全落地）的顶层 goal RFC / 设计稿移入此目录，活跃 goal 仍保留在 `docs/goal/` 根目录。

| 文档 | 归档日期 | 状态 |
|---|---|---|
| [render-threading-rfc-2026-08-07.md](./render-threading-rfc-2026-08-07.md) | 2026-08-11 | 已实施（S1/S2/S3 ✅） |
| [compositor-process-rfc-2026-08-07.md](./compositor-process-rfc-2026-08-07.md) | 2026-08-11 | 已实施（P0–P3 ✅；frame_flow 17/17；Linux 默认 GPU dma-buf 链路） |
| [ai-refactor-acceptance.md](./ai-refactor-acceptance.md) | 2026-08-11 | 已落地（compositor 切片验收记录见 compositor-process-rfc 第八节） |
| [network-loading-performance-2026-08-14.md](./network-loading-performance-2026-08-14.md) | 2026-08-15 | 已实施（P0/P1/P2 ✅；仅支持 HTTP/1.1/2） |
| [canvas-2d.md](./canvas-2d.md) + [canvas-2d/](./canvas-2d/)（master/evidence/archive） | 2026-08-17 | 已完成（DC-1~4 ✅：WPT 919 文件导入、testharness 全绿、oracle-pass 100%/不一致 0、Mission 中期 80% 达成；完成日 2026-08-16） |
| [html-compat.md](./html-compat.md) + [html-compat/](./html-compat/)（master/test-matrix/completion-audit） | 2026-08-17 | 已完成（M0-M4 ✅：FR-001~012 / NFR-001~008 / IF-001~005 / 门禁全项 complete，completion-audit 2026-08-13；完成日 2026-08-13） |
| form-validation（模式 B 入口自归档——入口在 [../form-validation/archive/form-validation-goal-v1-2026-08-17.md](../form-validation/archive/form-validation-goal-v1-2026-08-17.md)，master/evidence 原地保留） | 2026-08-17 | 已完成（M1-M3 ✅：constraints 919 Pass / 0 Fail、提交阻断全链路；完成日 2026-08-17，归档由该流 commit `eb44f905e` 执行，本表补登记） |

`rendering-compat/` 子目录的已完成设计稿见 [`../rendering-compat/archive/`](../rendering-compat/archive/)。
