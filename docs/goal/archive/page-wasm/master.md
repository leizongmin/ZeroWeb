# 页面 WASM 深化 — 运行时控制面板（master.md）

**入口文档**: [../page-wasm.md](../page-wasm.md)
**创建日期**: 2026-09-07（goal 拆分 bootstrap）
**最后更新**: 2026-09-12（**goal DONE**——DC-1~4 全满足，M1/M2/M3 全部落地，归档建立）

---

## 当前状态：DONE（2026-09-12）

**判定依据**: [evidence/2026-09-12-m3-dc-closeout-audit.md](evidence/2026-09-12-m3-dc-closeout-audit.md)

### Done Criteria 核账

| DC | 内容 | 状态 | 依据（commit / evidence） |
|----|------|------|--------------------------|
| DC-1 | WPT 用例导入 + 通过率基线 | ✅ | 31 案（pinned 31597693）+ fetch 脚本 + 基线 715/719=99.4%（文本+JSON evidence）+ 账本 +31 行 |
| DC-2 | 导出面与类型真实化 | ✅ | 真实函数表 / Memory.buffer+grow（731de158c）/ I32~F64 全映射（d9565dddd）/ Global/Table 查询（a8912b310） |
| DC-3 | 实例化语义 | ✅ | importObject JS 函数链接 + LinkError 分类（37efb24b0）；validate 接线——`WasmSandbox::validate()` 全量校验 + JS 面魔术字节快速检查按「或明确记录」条款记账（4c1750110）；compileStreaming/instantiateStreaming Content-Type 校验（4c1750110） |
| DC-4 | 测试与质量不可退让 | ✅ | make test 19,168/0 全绿（M3 切片 2 收尾跑，源码此后零变更）；workspace clippy -D warnings 零警告；cargo build 通过（均本轮复跑）；每修复带单测/e2e + WPT 资产化 |

**基线复跑（收尾，release runner 重建后）**：715/719 = 99.4%，与 M1 基线零回归；
27/31 案全绿；4 案 Timeout 为跨流卡点（见下），非本 goal DC 缺口。

**执行日志归档**: [archive/2026-09-12-m1-m3-execution-log.md](archive/2026-09-12-m1-m3-execution-log.md)
（M1/M2/M3 各切片执行记录、关键决策与 bug 根因，只追加不修改。）

---

## 移交后续的事项（不阻塞本 goal）

### 跨流卡点（P2，碰头信号——run-rules §9/§11）

- **V8 前台消息循环泵**：4 案 `constructor/` 异步面 Timeout 根因——异步
  compile/instantiate 的 promise resolve 任务滞留 platform foreground queue
  （`v8_runtime.rs` 只泵 microtask，全仓无 PumpMessageLoop）。修复落
  `crates/script-sandbox`（event-loop-spec 流域）。落地后 `make testharness-wasm`
  复跑预期 99.4% → 100%。影响面不止 wasm（一切 V8 前台任务收尾的异步 builtin）。

### 协议性已知限制（非缺陷）

- Global 导出为快照值（不可变全局精确；可变全局 wasm 写入后不回读——桥异步协议
  无同步 getter 通道）；Table 仅 length 快照
- `__wasm_errors__` 通道为查询面（instantiate Promise 已 resolve，无法回溯 reject）
- import 回调 JS 返回 NaN 经 JSON 序列化变 null → 按 trap 处理
- JS 侧 `WebAssembly.validate` 为魔术字节快速检查（同步返回值无法经异步桥全量校验；
  真实校验在 `WasmSandbox::validate()`，host compile/instantiate 路径即全量校验）
- compileStreaming/instantiateStreaming body 经 arrayBuffer 聚合（增量流式编译待
  内核流式 API）
- spec-rfc TBD（docs/specs/page-wasm-host-function-spec-rfc.md §10）：QuickJS 后端
  importObject 重入可行性（TBD-1）、iterator-protocol 多返回值 import（TBD-2）

### fetch 脚本落点记账

goal 文档所写 `scripts/fetch-wasm-subset.sh` 实际落 `tests/wpt-runner/scripts/`——
沿用仓内 17 个同类 fetch 脚本的既有目录约定（与 `fetch-cache-storage-window-subset.sh`
同目录），偏差已在此记账。

---

## 实测基线（收尾时点）

- **WPT wasm/jsapi**：31 案 / 719 subtests，**99.4%**（715/719，Timeout×4 = 跨流卡点）；
  27/31 案全绿；复跑零回归
- **polyfill 路径**（execute_script_with_dom → `__WASM_BRIDGE__` → zero-wasm-sandbox）：
  wasm_bridge 测试族 31 测试全绿（含 importObject 链接/错误分类/streaming/grow/
  Global/Table/类型化全链）
- **wasm-sandbox**：207 测试全绿（validate/import_signatures/export_descriptors/
  grow_memory/grow 链接执行等新增面均有单测）
- **质量门禁**：make test 全绿 + `cargo clippy --workspace --all-targets -- -D warnings`
  零警告 + `cargo build --workspace` 通过（收尾复跑）

## 验证入口（后续复跑）

- `make testharness-wasm`（31 案基线；FILTER 按路径子串）
- `make test` / `make reftest`（test-guard 包裹；禁止裸跑 cargo test）
- `cargo clippy --workspace --all-targets -- -D warnings`
