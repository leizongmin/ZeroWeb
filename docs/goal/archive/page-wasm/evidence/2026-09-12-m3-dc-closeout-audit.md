# M3 切片 3 — Done Criteria 收尾核账（DONE 判定依据）

**日期**: 2026-09-12
**判定**: **DC-1~4 全部满足 → goal DONE**（跨流卡点显式记账，见 §4）

## 1. DC 逐条核账

### DC-1: WPT 用例导入与通过率基线 — ✅
- ✅ `wasm/jsapi` window 可执行面 31 案导入（上游 31597693 pinned）；`wasm/spec` 内核类
  按 goal 排除条款 skip（fetch 脚本头注释 + master.md 记账）
- ✅ fetch 脚本：`tests/wpt-runner/scripts/fetch-wasm-subset.sh`（jsdelivr 首选 raw 回落；
  落点沿用仓内 17 个同类脚本的既有目录约定，goal 文档所写 `scripts/` 前缀按仓库惯例
  归位——偏差已记账）
- ✅ 分类通过率报告（文本 + JSON）：`evidence/2026-09-12-m1-wasm-jsapi-baseline.{md,json}`
- ✅ driving 用例入账本：`imported-testharness.txt` +31 行（PWASM-M1-baseline）
- ✅ 报告持久化 `docs/goal/page-wasm/evidence/`

### DC-2: 导出面与类型真实化 — ✅
- ✅ `Instance.exports` 真实函数表（host 注入真实导出包装，`__host_backed__:true`）
- ✅ `Memory.buffer` 真实映射（R3352 真实页数字节数）+ grow 同步（M2 切片 1：host
  `grow_memory` + buffer 替换，spec 增长前页数语义）
- ✅ I32/I64/F32/F64 参数与返回值全映射（M1 切片 2：签名驱动类型化协议，i64 ↔ BigInt
  双向 >2^53 精度 e2e）
- ✅ Global/Table 导出可查询（M2 切片 2：Global 值对象 `.value`/`valueOf`、Table
  `length`；快照语义的协议性限制记 master.md 已知限制）

### DC-3: 实例化语义 — ✅
- ✅ importObject 链接语义 + LinkError 分类（M3 切片 1：JS 函数真作 import——HostFn
  同步重入全链；缺失/签名不匹配/非函数形态 → LinkError 显式失败）
- ✅ validate 真实校验接线（M3 切片 2：`WasmSandbox::validate()` 三后端全量校验；
  host compile/instantiate 路径即全量校验。JS 侧 `WebAssembly.validate` 保留魔术字节
  快速检查——同步返回值无法经异步桥请求 host 校验，按 DC-3「或明确记录」条款记账于
  master.md 已知限制）
- ✅ `compileStreaming`/`instantiateStreaming` 真实 Response 路径（M3 切片 2：Content-Type
  校验 TypeError per spec + body 消费；arrayBuffer 聚合为兼容路径保留，增量流式编译记
  已知限制）

### DC-4: 测试与质量不可退让 — ✅
- ✅ `make test` 全绿零失败：M3 切片 2 收尾跑 19,168 passed / 0 failed（此后源码零变更）
- ✅ `cargo clippy --workspace --all-targets -- -D warnings` 零警告（本轮复跑确认）
- ✅ `cargo build --workspace` 通过（本轮复跑确认）
- ✅ 每项修复有对应单测/e2e（wasm-sandbox 207 测试、wasm_bridge 31 测试）+ driving
  WPT 用例资产化（31 案账本）

## 2. 基线复跑（本轮，release runner 重建后）

| 指标 | M1 基线（2026-09-12 晨） | 收尾复跑（本轮，重建后） |
|---|---|---|
| subtests | 715/719 = 99.4% | **715/719 = 99.4%（零回归）** |
| 全绿用例 | 27/31 | 27/31 |
| 非 Pass | Timeout ×4（constructor/ 异步面） | 同左（跨流卡点，见 §4） |

原始数据：[2026-09-12-m3-dc-closeout-baseline.json](2026-09-12-m3-dc-closeout-baseline.json)

## 3. 交付物清单（commit 链）

| commit | 内容 |
|---|---|
| 37da92898 | M1 切片 1：31 案导入 + testharness-wasm + 基线 99.4% + 两条架构级发现 |
| d9565dddd | M1 切片 2：WasmValue 全类型桥接协议（i64 BigInt/f32/f64/多返回值） |
| 9cdae5f69→b4474cfcb | M1 切片 3：Module.exports() 描述数组（compile/instantiate 双路径） |
| 731de158c | M2 切片 1：Memory.grow 真实接线 |
| a8912b310 | M2 切片 2：Global/Table 导出接 JS 面 |
| 422a6645d | M2 切片 3：错误分类面 + WA 桥状态幂等安装修复 |
| fa3fc7bc3 | M3 设计（spec-rfc，Lint 23P/2W/0F）+ import 签名反查/注册面 |
| 37efb24b0 | M3 切片 1b/1c：HostFn 同步重入全链（JS 函数真作 import） |
| 4c1750110 | M3 切片 2：validate API + compileStreaming/Content-Type |

## 4. 显式记账的跨流卡点（不阻塞本 goal DC）

4 案 `constructor/` 异步面 Timeout 的根因 = **V8 前台消息循环全仓从不泵**
（`v8_runtime.rs` 只泵 microtask；异步 compile/instantiate 的 resolve 任务滞留
platform foreground queue）。修复落点 `crates/script-sandbox` 属 event-loop-spec 流域
（run-rules §9 碰头信号），本 goal 按执行模式记入跳过清单并移交流协调——落地后复跑
预期 99.4% → 100%。详见 evidence/2026-09-12-m1-wasm-jsapi-baseline.md「关键发现二」。

## 5. 协议性已知限制（非缺陷，master.md 已知限制节收录）

Global 导出快照值 / Table 仅 length / `__wasm_errors__` 查询面（Promise 已 resolve 不可
回溯 reject）/ import 回调 NaN → trap / JS validate 魔术字节快速检查（真实校验在
`WasmSandbox::validate()`）/ streaming 聚合缓冲（增量编译待内核流式 API）。
