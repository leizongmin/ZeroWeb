# WebAssembly 兼容扩面 — wasm/core + wasm/webapi

**版本**: v1.3
**日期**: 2026-10-29（M3 收口 + 挂账结清：webapi streaming 27F→2F→0F，残差由 net-api M2-S18 清偿；memory64 定性翻案 Chrome 同形、coverage 复测闭环）
**状态**: Archived（✅ 2026-10-29 收口 Done——DC-1~4 全部达成，M1/M2/M3 全部完成（webapi 残差同日由 net-api M2-S18 清偿至 272/272）后模式 A 归档；终态见 master.md 及 M3 收口 evidence
[evidence/2026-10-29-m3-close.md](wasm-compat/evidence/2026-10-29-m3-close.md)）
**执行模式**: WPT 驱动（上游 wasm corpus 为验收标尺）+ 实现与语义修齐
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——wasm 面）

**当前卡点**: 无。挂账移交结清：webapi streaming 残差 2F（ReadableStream 体含
裸 ArrayBuffer chunk 须 TypeError）已由 net-api goal M2-S18 清偿——Body mixin
arrayBuffer() 返 spec ArrayBuffer 后 enqueue 产物撞既有体消费类型门，wasm/webapi
全子集 272/272（[net-api S18 证据档](../net-api-compat/evidence/2026-10-29-m2-s18-body-arraybuffer-wasm-stream.md)）。
余项不变：
- memory64 系 9F 定性为 V8 16GB memory64/table64 声明上限 validation 期检查，
  Chrome stable 同形（102/107、39/41、102/104）——不修，随 drift 巡检重估。

## coverage 对账（coverage-2026-10-27 基线 → 2026-10-29 M3 收口更新）

| WPT 面 | 全量分母（子测试） | M3 实跑 | 说明 |
|---|---:|---|---|
| wasm/jsapi | 2,545 | **878/878 全绿** | 底线重记账：2026-09-12 的 715/945+4T 中 4 个 Timeout 根因收敛（见 evidence）；通道即 window 可执行子集（31/244 文件，立项口径） |
| wasm/core | 188,385 | **188,280/188,289 = 99.995%**（258 wrapper 页 + 273 案中 core 面） | 残差 9F = memory64 系定性账（Chrome 同形）；分母差额 96 = pin 漂移 87 + 上游新增 cast.wast 9（pin 后文件），pin 内文件覆盖无洞 |
| wasm/webapi | 1,066 | **272/272 = 100%**（window 可执行子集 15 案） | M1 残差 27F → M3 后 2F → **0F**（net-api M2-S18 清偿 streams 域挂账，见[证据档](../net-api-compat/evidence/2026-10-29-m2-s18-body-arraybuffer-wasm-stream.md)）；未拉 66 文件 = worker-only/esm-integration/idlharness（通道 scope 边界） |

coverage 复测（07ce6e987a aligned stable 三家）scoped 报告见
[evidence/2026-10-29-m3-coverage.md](wasm-compat/evidence/2026-10-29-m3-coverage.md)
（只 wasm 通道实跑入分子，勿全量引用）。

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
- **DC-4（coverage 复测）✅**：jsapi 侧 715 基线逐 subtest 零回归 + 4 记账
  Timeout 收敛（878/878）；wasm/core、wasm/webapi 未覆盖子测试清零路径收口——
  pin 内文件覆盖无洞（core 258/259，缺者系 pin 后上游新增），残差全部归口
  （9F 定性账 / 2F streams 域挂账 / worker-only 通道 scope），见
  [evidence/2026-10-29-m3-close.md](wasm-compat/evidence/2026-10-29-m3-close.md)。

## 里程碑

- **M1 ✅（2026-10-29）**：语料预置 + runner 通道 + Makefile + 首轮全量基线。
- **M2 ✅（2026-10-29）**：基线分类 + 缺口簇清单（两簇，见 evidence；threads
  在本 pin 的 core 语料无独立目录，SIMD 全绿）。附带产出：runner 页面环境
  async wasm 编译面收敛（`compile`/`instantiate`/`compileStreaming`/
  `instantiateStreaming` 以原生同步原语重接，spec 语义钉齐——jsapi 4 个记账
  Timeout 的共同根因，净增 +163 零回归）。
- **M3 ✅（2026-10-29）**：webapi streaming 簇 27F→2F（runner 面五处：.wasm
  MIME、同名头去重、Promise<Response> 入参、CT 整值判定、Module 不可序列化
  登记；jsapi 878/878 零回归）；memory64 簇定性翻案（flag 实验假设证伪——V8
  已 shipped 无门控 flag，9F = 16GB 声明上限，Chrome stable 同形）；coverage
  复测收口（DC-4）。残差 2F 挂账移交 net-api 域。

## 依赖约束（run-rules §9）

- wasm 运行时变化不越 `wasm-sandbox` feature gate 与宿主桥接边界。
- jsapi 基线是回归底线：扩面轮只增不降，任何 P→非P 先修再收
  （2026-10-29 起底线为 878/878）。
