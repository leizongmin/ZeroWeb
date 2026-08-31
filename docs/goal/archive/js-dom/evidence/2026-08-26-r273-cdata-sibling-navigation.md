# R273 Evidence — CDATA 兄弟导航 getter（+2P，七轮累计 +35P）

**日期**: 2026-08-26
**切片**: M4——R273(a) 6,x oracle 遍历终止根因修复
**改动面**: part03 createCDATASection 字面量补 nextSibling/previousSibling
动态 getter + part23.rs（+1 回归单测）

## 一、定位链（R272 假设修正）

R272 归因到「wrapper identity churn」——方向接近但机制更简单。本轮
oracle 相位追踪探针（globalThis.__r273rm 相位标记，经 iframe window 读回）：

1. **AFTER-WALK-n=0**——oracle 树遍历零命中；
2. 手动 `isContained(midN)` = after/before/同根——**逻辑全对**；
3. walk 循环逐节点 dump：w[0]=CDATA:1234 后立即终止；
4. **sc.nextSibling=ODD:undefined**（双侧一致）——**CDATA 字面量没有
   nextSibling 槽**（undefined 非 null）→ common.js nextNode 的爬升
   `while (cn && !cn.nextSibling)` 对 undefined 视为无兄弟 → 一路上行到根
   → 遍历提前终止。

**根因**：createCDATASection 字面量（part03 R51）只有 parentNode 无兄弟
getter——克隆域 append（R220 的 createCDATASection 重建路径）只设 parentNode
不接兄弟链（`_zwMText`/`_zwMComment` 工厂有 `_zwMDefineSiblings` 而 CDATA
字面量没有）。

## 二、修复

CDATA 字面量补 nextSibling/previousSibling 动态 getter（`parentNode.childNodes
.indexOf(n4)` 现算——`_zwMEl` 域 `_zwMDefineSiblings` 同款模式；detached /
边界返 null）。

## 三、验证（vs R271 基线）

| 项 | R271 | R273 | Δ |
|---|---|---|---|
| Range-deleteContents | 113P/16F | **115P/14F** | **+2**（6,x 两 subtest） |
| Range-extractContents | 160P/32F | 160P/32F | 持平 |
| Range-cloneContents | 162P/29F | 162P/29F | 持平 |
| Range-surroundContents | 1840P/0F | 1840P/0F | 持平（100%） |
| Range-insertNode | 1841P/0F | 1841P/0F | 持平（100%） |
| Range-mutations-removeChild/insertBefore | 20P/76P 全绿 | 同 | 持平（100%） |
| engine 单测 | 2410 | **2411** | +1（r273 单测：detached null + parented 链）全绿 |
| fmt / clippy | 干净 | 干净 | — |

deleteContents 七轮累计：80P/49F → **115P/14F（+35）**。

## 四、方法论沉淀

**oracle 相位追踪**：给内联 oracle 加 globalThis 相位数组（ENTER →
BEFORE-WALK → stop info → walk 逐节点 → AFTER-WALK-n → 各 deleteData 段），
从 **iframe window** 读回（oracle 跑在 expected iframe 的 globalThis——
parent 的 globalThis 读不到，首轮探针空日志教训）。比「整体跑完 dump 结果」
定位快一个量级。

## 五、R274 靶点（残余 14F）

- **element 端点跨容器**（22/48/52/53,x）：方向分支 contained 递归。
- **28,x**（`[testDiv,0,comment,5]` 深形态）/ **49/50,x**（cursor-only）。
- extractContents 32F / cloneContents 29F 独立聚类（R273 相位追踪模板可
  直接复用于 expected 侧归因）。
