# R251 Evidence — 幽灵对象溯源：真插入的 P|a{HEAD-only}（gp=DIV），R252 栈捕获靶点（探针轮）

**日期**: 2026-08-25
**切片**: M4——R251(a) 幽灵创建点对象 id 溯源（无代码 land）
**基线**: surround 1806P/34F 复核零漂移

## 一、溯源链（R251-iframe 标记 + R251-probe walk dump）

1. **标记实验**：run() 时点给 paras[0..5]/docEl 子/testDiv 打
   `__r251tag`——dump 结果**全部 NONE**：testDiv 当前子列表中的对象
   （含幽灵）**均非 setup 期标记对象**（或标记被克隆丢失）。
2. **身份实验**（window `__r251p0` 中继）：`isLive=false`——
   `window.paras[0]` **不在** testDiv 内（真 P|a 已正确上移 docEl，
   R249/R248 修复生效）；幽灵 P|a 是**另一个对象**。
3. **唯一性**：全树 walk 仅**一个** `div#test`（n=7 = 6 合法子 + 1
   幽灵）——排除 referenceDoc 克隆残留双 DIV 假设。
4. **父链实验**：幽灵 `gp=DIV`——幽灵的 parentNode **就是 testDiv**：
   幽灵不是数组残留（R249 的单向断链形态），而是**被真实插入**的
   对象。

## 二、幽灵签名与候选机制

- 签名：`P|a{kids=1, k0=HEAD}`（**只含 HEAD 一个子**，无 TITLE 深度
  信息）+ 无 setup 标签 + 非 paras[0] + parentNode=testDiv。
- 与 surround clone 循环对照：`newParent.appendChild(kids[i]
  .cloneNode(true))` 首迭代（只 append 了 HEAD-clone）时的 P|a 快照
  形态**完全一致**——幽灵像是「P|a 只含 HEAD 时」的一个副本被插入
  testDiv。
- 候选（R252 验证）： MutationObserver 通知路径
  （`_mo_notify` 的 wrapper 重建/回放）； in-window 双调用
  （harness assert_throws_dom 内层 + 外层各一次 surround 的中间态
  交叉）； 克隆循环中 testDiv 侧的意外 append。

## 三、R252 靶点

in-window wrap testDiv.appendChild/insertBefore（R249 栈捕获同款
技术——run() 期安装、Error().stack 记录插入栈），一步定位幽灵插入者。

## 四、验证

- 探针清理 + iframe 还原（R251 标记 0）后基线复核：surround
  1806P/34F 零漂移；无代码 land → 无回归面。
