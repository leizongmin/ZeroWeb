# WebAssembly 兼容扩面 — wasm/core + wasm/webapi

**版本**: v1.0
**日期**: 2026-10-27（立项，coverage 对账驱动）
**状态**: Active——M1 语料预置未启动
**执行模式**: WPT 驱动（上游 wasm corpus 为验收标尺）+ 实现与语义修齐
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——wasm 面）

## coverage 对账（coverage-2026-10-27 基线，47 套件干跑口径，终稿待 `docs/compat/wptfyi/coverage-2026-10-27.md` 复核）

| WPT 面 | 全量分母（子测试） | 覆盖 | 说明 |
|---|---:|---:|---|
| wasm/jsapi | 2,545 | 31/244 文件 | 既有通道，干跑通过 715/945 = 75.7% |
| wasm/core | 188,319 | 0/258 文件 | 本 goal 主扩面 |
| wasm/webapi | 1,066 | 0/81 文件 | fetch/wasm 集成面，小域顺带 |

## Mission

把 wasm 测试面从 jsapi 子集扩到 core 全量（含 .any.js window 变体通道），
摸清 jsapi 75% 之外的失败集中在哪些 proposal（threads/SIMD/exceptions 等），
按簇挂账或收敛；webapi 小域顺带预置。

## Done Criteria

- **DC-1（语料全量）**：32 号脚本拉全 wasm/core + wasm/webapi（deep + any.js）。
- **DC-2（runner 通道）**：扩面运行通道就绪（既有 testharness-wasm 子命令扩
  语料面或新增子命令，M1 rally 轮定）+ per-dir skip 规则 + Makefile target。
- **DC-3（基线）**：分类通过率（文本 + JSON）落 `docs/goal/wasm-compat/evidence/`；
  失败按 proposal 归簇（jsapi 已知 24.3% 缺口同样归簇对照）；
  `docs/compat/trends/wpt-suites.csv` planned 行转数据行。
- **DC-4（coverage 复测）**：wasm/core、wasm/webapi 未覆盖子测试清零路径成立；
  与 jsapi 基线逐 subtest 对照零回归。

## 里程碑

- **M1**：语料预置 + runner 通道 + Makefile。
- **M2**：基线分类 + proposal 缺口簇清单（预期 threads/SIMD 大头，对照
  wasm-sandbox feature gate 现状）。
- **M3**：可收敛簇修齐或挂账 + coverage 复测收口。

## 依赖约束（run-rules §9）

- wasm 运行时变化不越 `wasm-sandbox` feature gate 与宿主桥接边界。
- jsapi 既有 75.7% 基线是回归底线：扩面轮只增不降，任何 P→非P 先修再收。
