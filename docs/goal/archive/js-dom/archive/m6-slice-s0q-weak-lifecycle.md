# M6 S0q 续 — weak/finalizer 生命周期实验 + 结论（R74）

**日期**: 2026-08-16
**Commit**: `4efd76aa`
**前置**: R73（DOMException 构造器，`be431968`）
**证据**: [evidence/2026-08-16-r74-quickjs-s0q-weak-lifecycle.json](../evidence/2026-08-16-r74-quickjs-s0q-weak-lifecycle.json)

## 背景

S0q 遗留注记：NODE_OBJECTS/LISTENERS 的 strong Persistent 是泄漏面（页面 JS 丢引用后包装对象仍被 Rust map 强持）。V8 侧 R3133 用 Weak + guaranteed finalizer 对等治理。本切片探索 QuickJS 对等物。

## API 事实（先决探索）

- QuickJS C API：`quickjs.h` 仅 `JS_AddIntrinsicWeakRef`（JS 内置开关），**无 `JS_NewWeakRef` 导出**（`js_weakref_constructor` 是 static）。
- rquickjs sys 层：未绑 JSWeakRef 记录类型。
- **Rust 侧无 weak 句柄、无 finalizer 回调**——V8 R3133 方案的两个支柱在 rquickjs 都缺。
- JS 侧 `WeakRef`/`FinalizationRegistry` intrinsic 在 `Context::full` 下可用（正向锚点断言）。

## 两方案实验（实现 → 实测 → 回退）

| 方案 | 实现 | 失败断言（PoC 实证） | spec 违背 |
|------|------|----------------------|-----------|
| ① NODE_OBJECTS 存 JS WeakRef（map 持 WeakRef 本体，目标可回收） | deref 读 / miss 重建 | `el.parentNode === element_for_id('main')` → false | 两次 eval 间 GC 回收树内节点包装 → 同 NodeId 重建新对象 → identity 断裂 |
| ② removeChild 时机 evict 子树缓存 | DFS 收集子树 NodeId 出 map | `childNodes[0] === held` → false（remove→re-append 场景） | JS 持旧引用 + 重建新对象 → spec remove→re-append 须同 identity |

## 结论（终态）

**strong Persistent + `reset_quickjs_state` 导航换代全清**是当前唯一正确形态：
- 有界泄漏面 = 单文档生命周期内的 detached 节点包装（页面级，导航即释放）——与真浏览器「树内节点由 document 强链保活」模型一致。
- 根本阻塞：weak 方案需要 finalizer 回调清 Rust map——rquickjs 缺该钩子（上游 TBD；暴露后可重启）。

## 附带发现

QuickJS GC 比 V8 激进——eval 语句间即回收无引用对象（V8 需显式 GC），weak 承载对 identity 的破坏在 QuickJS 下更早暴露。

## 验证

engine quickjs **1419** / v8 **2153** 全绿零回归；clippy 双矩阵零警告；fmt 无 diff；pre-commit-guard PASS。

## M6 状态

S0q（weak 项以结论关闭）后仅剩 whenDefined 真 pending（微任务域）——M6 接近全量收口。
