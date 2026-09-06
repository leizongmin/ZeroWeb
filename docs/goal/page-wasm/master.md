# 页面 WASM 深化 — 运行时控制面板（master.md）

**入口文档**: [../page-wasm.md](../page-wasm.md)
**创建日期**: 2026-09-07（goal 拆分 bootstrap）
**最后更新**: 2026-09-07（立项——M1 待启动）

---

## 当前状态

**专项定位**：父目标 M12「页面 WASM 与 JS 互操作」唯一无 goal 认领的未勾大项。底座已闭环
（JS polyfill → `__WASM_BRIDGE__` → webview host → zero-wasm-sandbox），深化导出面/类型/
链接语义 + WPT 基线从零建立。

**与兄弟 goal 的边界**：
- rendering-compat — 渲染流域 crate 域零重叠；`dom_bridge.rs` / `webview.rs` 共享大文件
  按 run-rules §9 `git log` 核对后再动
- event-loop-spec — 无共享面（该流改 js_dom_shim/part01.js 与 script-sandbox）
- storage-opfs — 无共享面

## 实测基线（2026-09-07 立项时）

### 现有实现

- ✅ Rust 执行底座：`crates/wasm-sandbox/` 1392 行，wasmtime/wasmi/stub 三后端 feature gate；
  compile/instantiate/call/read_memory/write_memory/set_fuel/LinkerConfig 全有
- ✅ host 执行闭环：`webview.rs` 行 4530–4611（`__WASM_BRIDGE__:`/`__WASM_COMPILE__:`
  探测 → 编译/实例化）、行 4795–4840（`_callQueue` 排空回注）；`tests/wasm_bridge.rs`
  398 行 15 测试全绿
- ✅ JS polyfill：`dom_bridge.rs` 行 361–530——compile/instantiate/instantiateStreaming/
  validate/callQueue 协议
- ⚠️ 导出面 stub：`exports.memory.buffer` 固定 64KB、`grow` 恒返 1、`__host_backed__:false`
- ⚠️ 参数映射仅 I32（`WasmValue` 枚举有 I64/F32/F64，桥接层未转换）
- ⚠️ `validate` 仅查 `\0asm` 魔术字节；instantiateStreaming 是 arrayBuffer 回退
- ⚠️ WPT 覆盖为零（wpt-data 无 wasm 目录、无 fetch 脚本、imported-tests.txt 零命中）

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | WPT 用例覆盖为零（fetch 脚本 + 导入 + 基线报告） | ⬜ M1 |
| P2 | 参数/返回类型仅 I32 | ⬜ M1 |
| P3 | 导出面 stub（函数表/memory/global/table） | ⬜ M1-M2 |
| P4 | 实例化语义（importObject 链接、错误分类、validate、streaming） | ⬜ M2-M3 |

## 下一步计划

1. **M1 切片 1**：`scripts/fetch-wasm-subset.sh` + `wasm/jsapi` 用例导入 + 分类通过率基线
   （stub 导出下的表现即验收清单）
2. **M1 切片 2**：`WasmValue` 桥接层类型转换全映射（I64/F32/F64）
3. **M1 切片 3**：exports 函数表真实化 + 失败聚类 → 修复队列

**碰撞管理**：开工前先 `git log --since="14 days ago" -- crates/engine/src/dom_bridge.rs
crates/webview/` 核对活跃面；有活跃编辑则先做零碰撞面（wasm-sandbox 单测、WPT 导入）。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT 基线建立 + 类型扩展 | ⬜ 待启动 |
| M2 — Memory/Global/Table + 实例化语义 | ⬜ |
| M3 — host function + 流式 + 收尾 | ⬜ |

## 验证基线

- 测试基线：立项时点全绿（`make test` / `make reftest` 入口，经 test-guard 包裹；
  禁止裸跑 cargo test）
- WASM 用例面：无基线（未导入/未建）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
