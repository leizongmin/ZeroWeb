# 归档：M4 切片 4 — native node mutation DomError→DOMException（append/insert/remove/replace）

**日期**: 2026-08-13
**轮次**: R4
**Milestone**: M4（WPT dom 上游基线 + 按聚类驱动修复）
**切片**: M4 切片 4（node 树 mutation 错误转 spec DOMException——native 路径）
**基线**: `d9e18396`（R3 land 后）

## 切片目标

R1 基线聚类：`Node-appendChild.html` / `Node-replaceChild.html` 的闭环插入（inclusive ancestor）、child 不在 parent 等应抛 HierarchyRequestError / NotFoundError。核实发现 **dom crate 已有完整 spec 校验**（`DomError` 枚举 + `is_ancestor` + cycle/not-a-child 检测），但 dom_bindings 层 `.is_ok()` **吞掉了错误类型**，best-effort 留 undefined。

## 实现产物

### DomError → DOMException 映射（native）
**`crates/engine/src/dom_bindings/node.rs`**：新增 `dom_error_exception(&DomError) -> (name, message)`：
- `WouldCreateCycle` / `CannotInsertDocumentRoot` → `HierarchyRequestError`
- `NotAChild` → `NotFoundError`
- `NodeNotFound` → best-effort `HierarchyRequestError`
- `NotAnElement` / `AlreadyHasShadowRoot` → `InvalidStateError`（attachShadow 专用）

### 4 个 invoke 改 match Err 抛 DOMException
`native_append_child_invoke` / `native_insert_before_invoke` / `native_remove_child_invoke` / `native_replace_child_invoke`：从 `.is_ok()` 改为 `match Ok/Err`，Err 经 `dom_error_exception` + `throw_dom_exception` 抛 spec 合规 DOMException。dom crate 校验逻辑零改动（复用既有）。

### native 单测（tests_dom_api.rs）
- `native_append_child_cycle_throws_hierarchy_request`：`a.appendChild(a)` 自身闭环 + `b.appendChild(a)` 祖先闭环 → HierarchyRequestError
- `native_replace_child_not_a_child_throws_not_found`：`a.replaceChild(b,c)` c 不在 a → NotFoundError
- `native_remove_child_not_a_child_throws_not_found`：`a.removeChild(orphan)` → NotFoundError

## 验证证据

| 矩阵 | 命令 | 结果 |
|------|------|------|
| zero-engine v8 lib | `cargo test -p zero-engine --features v8 --lib` | ✅ 2072 passed（+4：3 错误测试 + helper） |
| clippy v8 + quickjs | `cargo clippy -p zero-engine ...` | ✅ 双矩阵零警告 |
| 既有 dom_bindings 测试 | `cargo test ... dom_bindings` | ✅ 194 passed（合法 append/insert/remove/replace 路径 0 回归） |

## 关键限制与决策（重要）

**testharness-dom 基线不提升**：`run_testharness_html_inner` 用 `WebViewConfig::default()`（native_dom=false → **polyfill 路径**）。R4 仅改 native（node.rs），polyfill 路径未改，故 dom/nodes 基线维持 56.45%（R3 值）。

**polyfill appendChild 闭环架构限制**（记「未解决问题」）：polyfill 桥 mutation 经 `__zw_append_child` 回调延迟批处理（`apply_dom_mutations` 脚本后 apply），shim `_makeProxy` 只有 selector/handle 无 live 祖先链——无法在 appendChild 调用点同步抛。待 M1 L2 polyfill-live 合一（shim 改读 live Document 后才有祖先链）。

**为何仍 land native 修复**（net≥0）：
1. native 是 default-on 后的生产路径（M5），规范合规必须做
2. dom crate 校验已就绪，dom_bindings 只是把吞掉的错误转正确 DOMException——改动面小、风险低
3. native 单测验证（3 个错误场景 + 194 合法路径无回归）
4. 为 M1 L2/default-on 铺路：native 路径每个校验点都 spec 合规后，default-on 才安全

## 关键发现（架构洞察）

- **dom crate 是 spec 校验权威**：DomError 枚举 + is_ancestor 已完整，dom_bindings 只是薄映射层。后续其他 mutation 校验（normalize、clone 等）复用同模式。
- **polyfill 桥的同步性限制**：延迟批处理架构使 appendChild 类同步校验无法在 polyfill 路径实现——这是 L2（polyfill-live 合一）要解决的核心问题之一，印证入口文档 L2 优先级。

## 下一步（M4 切片 5 候选）

按重排 ROI：① **testharness-dom native 路径对照**（让 R2/R3/R4 native 修复基线可见 + DC-3 硬要求）② createProcessingInstruction（44）③ 扩 dom/events。
