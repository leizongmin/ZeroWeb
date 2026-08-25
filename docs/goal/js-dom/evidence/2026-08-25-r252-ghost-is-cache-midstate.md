# R252 Evidence — 幽灵定性收口：wrapperChurn 实证，幽灵 = 序列化缓存中间态快照（探针轮）

**日期**: 2026-08-25
**切片**: M4——R252(a) 幽灵插入者栈捕获（定性收口，无代码 land）
**基线**: surround 1806P/34F 复核零漂移

## 一、两连否定实验

1. **方法包装否定**：run() 期 wrap testDiv.appendChild/insertBefore
   （stack 捕获）——surround 期间**零调用**（`WRAP td=OK` 后无任何
   记录）。幽灵的"插入"不经过 testDiv 的变异方法。
2. **数组替换标记**：run() 期给 testDiv.childNodes 数组打
   `__r252arr` 标记——positionTests 读到 `REPLACED`。

## 二、定性判定（wrapperChurn）

3. **身份中继实验**：`window.__r252td` 中继 + walk 对比——
   **`wrapperChurn`**：positionTests 由 actualRoots[0] walk 到的
   `div#test` 与 run() 时的 testDiv **不是同一对象**。

**结论**：R250/R251 观察到的"幽灵 P|a{HEAD-only}"**不在活树**——
活树经 R248/R249 修复已正确（真 P|a 上移、无插入）；幽灵存在于
**wrapper/序列化缓存域**：positionTests 侧 walk 命中的是**重建的
wrapper**，其内容来自序列化缓存——缓存在 surround clone 循环**中途**
（P|a 只含首个 HEAD-clone 时）被烘焙（baked），之后未失效。R251 的
`gp=DIV`/无标签/非 paras[0] 全部由此解释（wrapper 域对象的属性均与
活对象脱钩）。

## 三、R253 修复方向

surround 的 clone 循环对 testDiv 视图的中途烘焙： 克隆 build 到
detached 容器，完成后一次 append（不在中途触碰可触发序列化的
getter）； 或 surround 收尾统一失效 `_zwQWrapCache`/序列化源
（现有 `_zwQWrapGen++` 清理的时机补强——clone 循环期间 suppress）。
验证路径：修复后 walk dump 幽灵消失 + 13/14,x subtest 翻绿。

## 四、验证

- iframe 还原（R252 标记 0）+ 基线复核：surround 1806P/34F 零漂移；
  无代码 land → 无回归面。
