# WebAssembly 兼容扩面 — wasm/core + wasm/webapi

**版本**: v1.1
**日期**: 2026-10-29（M1 收口：语料全量 + 通道 + 基线 99.98%；异步编译面收敛 +163）
**状态**: Active——M1 完成；M2 缺口归簇完成（memory64 9F + webapi streaming 27F）；剩 M3 coverage 复测收口
**执行模式**: WPT 驱动（上游 wasm corpus 为验收标尺）+ 实现与语义修齐
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——wasm 面）

**当前卡点**: 无阻塞。M3 收口清单——① coverage 复测（wpt.fyi 对账，wasm/core、
wasm/webapi 未覆盖子测试清零路径）；② M2 两簇缺口的收敛或挂账定性
（memory64 64 位 index 面 / webapi streaming Response 集成面，见 evidence）。

## coverage 对账（coverage-2026-10-27 基线 → 2026-10-29 M1 实跑更新）

| WPT 面 | 全量分母（子测试） | M1 实跑 | 说明 |
|---|---:|---|---|
| wasm/jsapi | 2,545 | **878/878 全绿** | 底线重记账：2026-09-12 的 715/945+4T 中 4 个 Timeout 根因收敛（见 evidence） |
| wasm/core | 188,319 | **188,280/188,289 = 99.99%**（258 wrapper 页 + 273 案中 core 面） | M1 首跑，残差 9F（memory64 64 位 index 面） |
| wasm/webapi | 1,066 | **245/272 = 90.07%**（window 可执行子集 15 案） | 残差 27F（streaming Response 集成面，单根因） |

## Mission

把 wasm 测试面从 jsapi 子集扩到 core 全量（含 .any.js window 变体通道），
摸清 jsapi 75% 之外的失败集中在哪些 proposal（threads/SIMD/exceptions 等），
按簇挂账或收敛；webapi 小域顺带预置。

**M1 实跑修正立项预估**：预估「75% 起步、缺口集中 threads/SIMD」被证伪——
simd/gc/exceptions/relaxed-simd/bulk-memory/multi-memory 全绿（runner 页面环境
回挂的原生 V8 已实现上述 proposal）；真实缺口仅 memory64 + webapi streaming 两簇。

## Done Criteria

- **DC-1（语料全量）✅**：32 号脚本拉全 wasm/core + wasm/webapi（deep + any.js +
  wasm 资产；606 文件 47MB 0 空文件）。
- **DC-2（runner 通道）✅**：新增 `testharness-wasm-core` 子命令 +
  `wasm_core_case_skipped` per-dir skip 规则 + `make testharness-wasm-core` target
  （test-guard 包裹）；jsapi 底线通道 `testharness-wasm` 字节不动。
- **DC-3（基线）✅**：分类通过率落
  [evidence/2026-10-29-m1-baseline.md](wasm-compat/evidence/2026-10-29-m1-baseline.md)
  （md + json）；失败按簇归口（memory64 9F / webapi streaming 27F，jsapi 缺口随
  4 Timeout 收敛清账）；`wpt-suites.csv` 新增 wasm-core/webapi/jsapi 三条数据行。
- **DC-4（coverage 复测）**：jsapi 侧已闭环——715 基线逐 subtest 对照零回归 +
  4 记账 Timeout 收敛（878/878）；wasm/core、wasm/webapi 未覆盖子测试清零路径
  待 coverage 复测收口（M3）。

## 里程碑

- **M1 ✅（2026-10-29）**：语料预置 + runner 通道 + Makefile + 首轮全量基线。
- **M2 ✅（2026-10-29）**：基线分类 + 缺口簇清单（两簇，见 evidence；threads
  在本 pin 的 core 语料无独立目录，SIMD 全绿）。附带产出：runner 页面环境
  async wasm 编译面收敛（`compile`/`instantiate`/`compileStreaming`/
  `instantiateStreaming` 以原生同步原语重接，spec 语义钉齐——jsapi 4 个记账
  Timeout 的共同根因，净增 +163 零回归）。
- **M3**：两簇缺口修齐或挂账定性 + coverage 复测收口（DC-4 后半）。

## 依赖约束（run-rules §9）

- wasm 运行时变化不越 `wasm-sandbox` feature gate 与宿主桥接边界。
- jsapi 基线是回归底线：扩面轮只增不降，任何 P→非P 先修再收
  （2026-10-29 起底线为 878/878）。
