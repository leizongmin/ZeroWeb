# R250 Evidence — R249 修复后双树深度 3/5 重 dump：主幽灵 P 已消，残余新签名（探针轮）

**日期**: 2026-08-25
**切片**: M4——R250(a) 13/14,x 残余首差重 dump（无代码 land）
**基线**: surround 1806P/34F 复核零漂移

## 一、重 dump 结果（R250-probe，深度 3 → 5）

1. **深度 3**：A/E 双侧 ROOT 树**完全一致**——13,0 的
   `HTML{P|a{HEAD{TITLE},BODY{DIV|test{}}}}`、14,0 的
   `HTML{HEAD{TITLE},P|a{BODY{DIV}}}`：surround 的 HEAD/BODY 移入
   newParent 结构正确，R246 时代的大形态分歧（BODY 缺失/P 缺失）
   **全部消失**——R245/R248/R249 三轮修复在树形态层生效。
2. **深度 5（DIV 子树展开）发现残余**：
   - A 的 `DIV|test` = **7 子**：`[P|a{HEAD{}}, P|b, P|c, P|d, P|e,
     P{cdata…}, comment]`
   - E 的 `DIV|test` = **6 子**：`[P|b, P|c, P|d, P|e, P{cdata…},
     comment]`
   - **新签名**：A 残留的 `P|a{HEAD{}}` 是**无 TITLE 的 HEAD** 单子
     形态——而真 P|a（已正确上移到 HTML 级）含完整 `HEAD{TITLE}+
     BODY` 双子。真对象已移走（R249 own removeChild 的 splice 生效），
     残留是一个**独立对象**（factory headEl 字面量特征：TITLE 为
     build 期 push，克隆时序不同可产生无 title 变体）。

## 二、结论与 R251 靶点

- R249 修复**树级生效**（双向树一致到 DIV 层）；残余 = DIV 内一个
  `P|a{HEAD{titleless}}` 幽灵对象（非真 P|a 的视图残留）。
- **R251**：定位该幽灵的创建点——候选：surround clone 循环
  （`newParent.appendChild(kids[i].cloneNode(true))`）的 kids[0]
  是否误取 factory headEl（`_coveredChildren` 的 sc.childNodes 视图
  在 iframe-doc 域的混取）；或 `_rmSnap` 通知路径的包装重建。探针
  对象 id 标记（`Object.assign(el, {__r250:1})` 溯源）可一步定位。

## 三、验证

- 探针清理后基线复核：surround 1806P/34F 零漂移；无代码 land →
  无回归面。
