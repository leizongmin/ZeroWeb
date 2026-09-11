# 页面 WASM 深化 — 运行时控制面板（master.md）

**入口文档**: [../page-wasm.md](../page-wasm.md)
**创建日期**: 2026-09-07（goal 拆分 bootstrap）
**最后更新**: 2026-09-12（M2 切片 3 落地——错误分类面 + WA 桥状态幂等安装；M2 全部完成）

---

## 当前状态

**M1 进行中**：切片 1（WPT 基线）已落地。基线实测 **715/719 = 99.4%**（31 案，
Timeout × 4），证据：[evidence/2026-09-12-m1-wasm-jsapi-baseline.md](evidence/2026-09-12-m1-wasm-jsapi-baseline.md)。

### 架构级发现（2026-09-12 基线实测，修正立项基线假设）

1. **页面路径跑的是 V8 原生 WebAssembly**。生产 `run_page_scripts`（js_dom_shim）不安装
   `generate_dom_api_polyfill()`——`globalThis.WebAssembly` 是 V8 原生完整实现；polyfill
   （stub 导出面/仅 I32）只装在 `execute_script_with_dom`（embedder execute 路径 +
   `tests/wpt-runner` wasm_bridge 类测试）。→ M1 原计划的「类型映射/exports 真实化」
   切片针对的 polyfill 不在 WPT window 验收路径上；DC-2 仍对 polyfill 路径有效
  （embedder 面真实消费方），但优先级重排到异步修复之后。
2. **4 案 Timeout 根因 = V8 前台消息循环从不泵**。异步 `WebAssembly.compile()/
   instantiate()` 的 promise 永不 settle（V8 后台编译完成后 resolve 任务投递在 platform
   foreground task queue，`v8_runtime.rs` 只 `perform_microtask_checkpoint`，全仓无
   `PumpMessageLoop`）；同步 `new Module/Instance` 全绿。修复位置 =
   `crates/script-sandbox`（v8_runtime 泵消息循环）——**跨流卡点**，见下。

### 跨流卡点（碰头信号，run-rules §9/§11）

- **V8 消息循环泵**（解锁 4 案 Timeout，99.4% → 100%）：改动落在 `crates/script-sandbox`
  （event-loop-spec 流域，本 goal 与其声明的边界即「无共享面」）。消息循环泵本就是
  event loop spec 化的天然组成件（timer/微任务/前台任务统一处理），建议归并该流推进；
  若用户希望本流接手，须明确授权跨域。
  - 已知影响面不止 wasm：一切以 V8 前台任务队列收尾的异步 builtin 均挂。

**与兄弟 goal 的边界**：
- rendering-compat — 渲染流域 crate 域零重叠；`dom_bridge.rs` / `webview.rs` 共享大文件
  按 run-rules §9 `git log` 核对后再动
- event-loop-spec — script-sandbox 归其域（见上跨流卡点）；`dom_bridge.rs` WebAssembly
  段归本流
- storage-opfs — 无共享面

## 实测基线（2026-09-12，M1 切片 1）

- **WPT wasm/jsapi**：31 案 / 719 subtests，**99.4% Pass**（715/719，Timeout×4 集中在
  `constructor/` 异步面）；27/31 案全绿
- **polyfill 路径**（execute_script_with_dom → `__WASM_BRIDGE__` → zero-wasm-sandbox）：
  `tests/wasm_bridge.rs` 15 测试全绿（基线不变）
- 既有实现盘点（2026-09-07 立项时）仍准确：wasm-sandbox 1392 行三后端；polyfill stub
  导出面/仅 I32/validate 魔术字节/instantiateStreaming 回退

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | WPT 用例覆盖 + 通过率基线 | ✅ M1 切片 1（31 案入账本，evidence/ 落盘） |
| P2 | 异步 compile/instantiate promise 永挂（V8 消息循环不泵） | ⬜ 跨流卡点——待 event-loop-spec 流或用户授权 |
| P3 | polyfill 路径参数/返回类型仅 I32 | ✅ M1 切片 2（i64 BigInt 双向含 >2^53 精度 / f32 / f64 / 多返回值 / 零返回 undefined） |
| P4 | polyfill 导出面 stub | ✅ M1 切片 3（`Module.exports()` 描述数组）+ M2 切片 2（Global 值对象 / Table length）+ M2 切片 1（Memory.grow 真实接线） |
| P5 | 实例化语义（错误分类） | ✅ M2 切片 3（CompileError/LinkError/RuntimeError 构造器 + host 分类注入；WA 桥状态幂等安装修复跨 execute id 错位） |
| P6 | importObject JS 函数链接 + validate + streaming | ⬜ M3（host function 重入设计前置） |

## 下一步计划

1. **M3 切片 1**：host function 设计前置——wasm 执行中同步回调 JS 需宿主重入设计
   （`instance.call` 内再进 JS sandbox），importObject 的 JS 函数表传递与其同一机制，
   先出设计记本文档再动代码（DC-3/DC-4 深水区）
2. **M3 切片 2**：validate 真实校验接线（wasm-sandbox 补 validate API → 桥接，替换
   魔术字节检查）；`compileStreaming`/`instantiateStreaming` 真实 Response body 路径
   （fetch_handler 已有，接线 + Content-Type 校验）
3. **跨流协调**：V8 消息循环泵归 event-loop-spec 流（或用户授权跨域），落地后
   `make testharness-wasm` 复跑验证 4 案翻绿（99.4% → 100%）
4. **已知限制（协议性，非缺陷）**：Global 导出为快照值（不可变全局精确；可变全局
   wasm 侧写入后不回读——桥异步协议无同步 getter 通道，live 值需协议扩展）；Table
   仅 length 快照（get/grow 未接线）；`__wasm_errors__` 通道为查询面（instantiate
   Promise 已 resolve，无法回溯 reject——spec 形态的 promise 语义需桥协议演进）

**碰撞管理**：开工前先 `git log --since="14 days ago" -- crates/engine/src/dom_bridge.rs
crates/webview/` 核对活跃面；有活跃编辑则先做零碰撞面（wasm-sandbox 单测、WPT 导入）。
2026-09-12 实测：webview/ 昨日（09-11）有 event-loop-spec MO 提交——webview.rs 仅动
wasm 段（`process_wasm_bridge` 及 `_callQueue` 排空区），避开 MO 区。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT 基线建立 + 类型扩展 | ✅ 切片 1（基线 99.4%）+ 切片 2（类型全映射）+ 切片 3（导出描述面）；残余 = P2 跨流卡点 |
| M2 — Memory/Global/Table + 实例化语义 | ✅ 切片 1（Memory.grow 真实接线）+ 切片 2（Global/Table 导出面）+ 切片 3（错误分类面 + WA 状态幂等安装）；残余 = importObject JS 函数链接归 M3 重入设计 |
| M3 — host function + 流式 + 收尾 | ⬜ 切片计划见上（重入设计前置） |

## 验证基线

- 测试基线：`make test` / `make reftest` 入口（test-guard 包裹；禁止裸跑 cargo test）
- WASM 用例面：`make testharness-wasm`（31 案，99.4%；4 Timeout 为 P2 跨流卡点）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过
