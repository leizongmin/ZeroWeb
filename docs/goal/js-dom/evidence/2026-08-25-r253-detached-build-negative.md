# R253 Evidence — detached-build + 缓存失效实验：负结果（R252 假设证伪，已回退）

**日期**: 2026-08-25
**切片**: M4——R253(a) 缓存中途烘焙修复实验（负结果轮，无代码 land）
**基线**: surround 1806P/34F；回退后复核一致

## 一、实验

按 R252 方向 实现：surround clone 循环改为 detached 数组攒齐克隆后
一次 append（中途不产生半完成 newParent 形态）+ append 完成后统一
失效 `_zwQWrapCache`（`_zwQWrapGen++` + clear）。

## 二、结果（负结果）

- surround 1806P/34F **净 0**（subtest diff=0）；ranges 全量
  set-diff **0/0**——13/14,x 失败形态与消息完全不变。
- **结论**：R252 的「clone 循环中途烘焙」假设被证伪——幽灵快照的
  烘焙点不在 clone 循环（或不止此处）；`[object Object]` 首差形态
  在修复前后逐字一致。

## 三、处置

按最少代码准则（CLAUDE.md §2/§3：每行修改可追溯到需求、无证据的
行为变更不 land）：**已回退**实验改动（part06 恢复 main 状态，
diff 0 行，基线复核 1806P/34F 一致）。

## 四、R254 靶点（烘焙点重定位）

- 幽灵快照烘焙点候选重排： removal 循环（`kids[k].remove()`
  期间的序列化读取）； `_rmSnap` 构造期（`previousSibling/
  nextSibling` getter 触发的兄弟链序列化）； harness domTests 的
  `assert_equals(actualIframe.contentWindow.unexpectedException,
  null)` 等前置断言读取（在 surround 前触发了一次烘焙、surround
  后未失效）。
- 方法：在 R249 同款 in-window wrap 上加 `_zwQWrapGen` 代际读数
  探针（surround 各步骤前后的 gen 值 + cache size），定位「surround
  后未失效」的确切窗口。
