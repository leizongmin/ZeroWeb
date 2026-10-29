---
date: 2026-10-29
modules: engine,script-sandbox,wpt-runner
---

# QuickJS 全局 `var` 重声明不转换自有 accessor——WPT 全局变量拼接值静默丢失

## 问题

net-api 二期 M2-S11（cors-preflight-referrer 收敛轮）发现：`cors-preflight-referrer.any.js` 顶层 `var origin = get_host_info().HTTP_ORIGIN + "/";` 在断言里读到的值是 `"http://wpt.test:8000"`（丢了 `+ "/"` 拼接），且赋值无任何报错。测试期望值算错，导致引擎实际发送正确的 `origin + "/"` Referer 反而被判 Fail，6 条腿连片翻红。

## 根因

- js_dom_shim 曾以 `Object.defineProperty(globalThis, 'origin', { configurable: true, get })` 把 `Window.origin` 定义为 globalThis 的**自有访问器属性**（`crates/engine/src/js_dom_shim/part01.js` 旧 8747 区）。
- 按 spec（GlobalDeclarationInstantiation → CreateGlobalVarBinding），全局 `var origin = X` 遇已有**可配置 accessor** 应把它转换为数据属性再赋值。QuickJS 不落实该转换：赋值被静默吞掉，读仍走 accessor getter（返回 `location.origin`，无尾斜杠）。
- 真浏览器不受影响：`window.origin` 是 `Window.prototype` 上的 accessor，全局 `var` 在 globalThis 上建自有数据属性自然遮蔽原型 accessor——这正是上游 WPT 语料大量使用 `var origin` 等名字的前提。

## 定位手法（可复用）

用一次性 wpt-data 诊断用例（gitignored，跑完即删）把可疑值直接 `assert_equals` 进 runner 输出 JSON：先证 `HTTP_ORIGIN + "/"` 求值正常，再加「顶层 `var origin` + 异步回调后复读」两腿，装载期即错 → 排除异步污染，锁定全局绑定语义。比在引擎里加日志快得多。

## 解决方案

shim 的全局 accessor 改为**数据属性快照**（`globalThis.origin = location.origin || 'null'`），走 QuickJS 常规 var 重声明路径。两个约束：

1. **时机**：快照求值依赖 `location.href`（内部再依赖 `_hist_current` 与 history state），part01 安装期这些未就绪会 TypeError——放到 shim 安装尾（part07 闭合 IIFE 前）。
2. **活性取舍**：快照值装载即定。runner 单文档页 origin 不会变；若未来支持同全局上下文跨源导航，需重新审视。

## 避免方式

- shim 给 globalThis 定义**任何**自有 accessor（现仅 `location` 与 `origin`）时，都要评估「上游语料全局 `var` 同名重声明」的冲突面；`location` 因 setter 语义保留 accessor，但语料无 `var location` 用法。
- 反向排查信号：测试里「引擎行为明明正确却 Fail、期望值像被截断/替换成另一语义的值」→ 先怀疑全局名字被 shim 属性遮蔽，再查引擎实现。
