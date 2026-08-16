# M6 S5q 完整化 — whenDefined 真 pending（R75，M6 全量收口）

**日期**: 2026-08-16
**Commit**: `4bfa87e3`
**前置**: R74（weak/finalizer 结论，`4efd76aa`）
**证据**: [evidence/2026-08-16-r75-quickjs-s5q-whendefined.json](../evidence/2026-08-16-r75-quickjs-s5q-whendefined.json)

## 背景

whenDefined 此前是 PoC 同步简化（立即 resolve）——`customElements.whenDefined('not-yet').then(...)` 在 define 前就该 pending。这是 M6 清单最后一项。

## 实现

1. **CE_WHEN_DEFINED 等待列表**：`tag → Vec<Persistent<Value>>`（resolve 函数）；`reset_quickjs_state` 清理。
2. **ce_when_defined**：已定义 → 立即 `resolve(ctor)`；未定义 → resolve 入列表，返 pending promise。
3. **ce_define**：成功注册后取走该 tag 等待者清列表，逐个 `resolve(ctor)`——then 回调经微任务天然排队。

## rquickjs 关键经验

- **`execute_pending_job` 不能在 Ctx 借用存活时调用**（`ctx.with` 闭包内 → safe_ref RefCell 重入 panic）——drain 须在 `ctx.with` 块之间（测试镜像生产 `quickjs_runtime.rs` 的 eval 后 drain 循环 / webview 事件循环职责）。
- **长 PoC 断言用全新 tag**：前序切片已注册的 tag（如 R65 的 my-el2）会让 whenDefined 走已定义分支且 resolve 携带旧 ctor——identity 断言误败。

## 验证

PoC 四组断言（pending 不跑 / drain 后 already / define flush / drain 后 resolved 双值序贯）；engine quickjs **1419** / v8 **2153** 全绿零回归；clippy 双矩阵零警告；fmt 无 diff；pre-commit-guard PASS。

## M6 里程碑收口

**S0q–S5q 全部完整落地**：
- S0q：骨架/工厂/身份缓存（weak 项以 R74 结论关闭——strong+reset 终态）
- S1q：属性族 + 复合对象三件套（attributes/classList/dataset）
- S2q：写入 + 子树 mutation + 树读回
- S3q：查询族
- S4q：EventTarget + 三阶段派发 + DOMException/Event 构造器
- S5q：customElements 五件套 + 完整 ctor 执行 + observedAttributes + whenDefined 真 pending

QuickJS native 具备与 V8 对等的元素面生产能力。**下一步主线**：M1 L2（polyfill-live 合一）或 M4 WPT dom 基线扩展。
