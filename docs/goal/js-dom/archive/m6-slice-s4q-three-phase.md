# M6 S4q 完整化 — QuickJS 三阶段 capture/bubble 派发（R67）

**日期**: 2026-08-16
**Commit**: `a8fa85b5`
**前置**: R66（S4q DOMException 错误路径，`4024e525`）
**证据**: [evidence/2026-08-16-r67-quickjs-s4q-three-phase.json](../evidence/2026-08-16-r67-quickjs-s4q-three-phase.json)

## 背景

R63 的 dispatchEvent 是 target 站单段派发（无传播链）。master.md「R66 后下一步首选」第 ② 项：镜像 V8 R3128/R3135 的三阶段派发。

## 实现

`dispatch_event_method` 重写（`crates/engine/src/quickjs_dom_bindings.rs`）：

1. **事件对象标准化**：对象原样读 `.type`；字符串包 `{type}`（R63 兼容）。
2. **parent 链收集**：`with_dom` 闭包内纯读收集 `[target, parent, ..., root]`，释放 borrow 后派发（防再入 panic）。
3. **visits 模型**（同 V8）：`chain[1..].rev() × phase 1`（capture 倒序）→ `(target, 2)` → `chain[1..] × phase 3`（bubble 正序）。
4. **phase 过滤**（spec invoke）：capture 阶段仅 capture 监听器；target 阶段全部（注册序）；bubble 阶段仅 bubble。
5. **stop 语义**：`stopPropagation` 当前节点监听器全尽后止后续节点；`stopImmediatePropagation` 立即止。方法缺失时注入（具名 `fn(This<Object>)` 写 flag——绕闭包 HRP），flag 每次派发复位。
6. **存活检查**：`listener_present`——派发期间被 removeEventListener 的监听器 skip（spec inner-invoke 步骤 5）。
7. **复位**：派发后 `currentTarget=null`/`eventPhase=NONE(0)`；返值 `!(cancelable && defaultPrevented)`。

## 验证

- PoC 断言四组：全链三阶段序（`p:1:P|m:2:T|g:3:G`——capture 标志双向过滤）、`bubbles:false` 跳 bubble、capture 站 stopPropagation 止后续、派发后复位
- engine quickjs **1419** / v8 **2153** 全绿零回归；clippy 双矩阵零警告；fmt 无 diff
- pre-commit-guard PASS

## 过程教训（重要）

1. **断言失败放大器**：初版期望值按直觉写成「每个监听器在 capture+bubble 双站触发」，assert 失败 → panic 跳过 Persistent 清理 → QuickJS `JS_FreeRuntime` gc_obj_list 断言 SIGABRT。一度误判为生产代码泄漏走了一轮 bisect——**SIGABRT 出现在失败断言后 ≠ 内存 bug**，先看断言 diff。
2. **误用 `git checkout` 清工作树**：bisect 中途想回 HEAD 对照，用了 checkout 而非 stash，R67 实现被清。从上下文完整重建（幸运：全部代码文本在会话中）。**回退工作树一律 stash，不用 checkout**。
3. spec 期望值推导：capture 监听器（`addEventListener(type, fn, true)`）只在 capture 站触发、bubble 监听器只在 bubble 站——双向过滤，不写「全序」。

## M6 剩余

observedAttributes 过滤 + oldValue 写前捕获 → S0q 续 weak/finalizer → S1q 复合对象（attributes/classList/dataset）→ Event 构造器 → DOMException 构造器 instanceof 面。
