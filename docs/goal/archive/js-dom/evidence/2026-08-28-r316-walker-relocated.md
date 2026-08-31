# R316 Evidence — walker 着陆点 relocated 置位（TreeWalker-walking-outside-a-tree 全文件转绿，traversal 域 1F→0F）

**日期**: 2026-08-28
**切片**: M4——R316(a) walking-outside-a-tree 残余第 4-6 断言（root 重挂后的 nextNode live 导航）
**改动面**: `part06.js`（previousNode 两处导航着陆点 + `syncOrderPosTo` 返回值）+ `part24.rs`（r315 测试扩为完整 WPT 六步断言）

## 一、根因

R315 后 walking-outside 残余：`p.appendChild(body)`（root 进 currentNode 子树）后
`nextNode()` 期望 P→**BODY**，实测 `n2=null`。探针（r316 probe，后清理）：

- `pos=TITLE`（previousNode 着陆 title ✓）→ `n1=P` ✓ → `n2=null` ✗

**机制**：previousNode 的导航式步进着陆 title 后调 `syncOrderPosTo(title)`——title 不在
order 快照（构造期 [body,p]）内 → `orderPos=-1`。但 `relocated` 标志未置位 → 后续
nextNode 的分派条件 `orderPos < 0 && relocated` 不满足，仍走 **order-scan 路径**：
`orderPos=-1` 落到 fresh 起点 `i=1`（快照 [body,p] 的 p 位），从 p 的结构序后继找——
regraft 后 body 已移入 p 子树、快照整体 stale，order-scan 越界恒 null。R97 的
`nextNodeOffOrder`（live 导航，沿真实 getter 沿 firstChild/nextSibling 步进）正是为此
形态而建，只是入口条件漏了「导航着陆点在快照外」这一来源。

## 二、修复（part06 三处）

1. `syncOrderPosTo` 返回定位值（-1 = 快照外）；
2. previousNode 的 sibling 循环着陆点与 climb 着陆点：`syncOrderPosTo(node) < 0` →
   置 `relocated = true`（快照 stale 信号，复用 R97 live 导航通道）。

零新增状态、零新扫描路径——只是把既有的 stale 检测面从「setter 重定位」扩展到「遍历
着陆点出快照」。spec 语义一致：TreeWalker 按结构序从 currentNode 继续，快照不含
currentNode 时必须 live 导航。

## 三、A/B

| 套件 | R315 | R316 | Δ |
|---|---|---|---|
| TreeWalker-walking-outside-a-tree | 0P/1F（第 5 断言 expected __n3 got null） | **Pass（全文件 100%）** | +1P/-1F |
| dom/traversal 全目录 | 1603P/1F | **全 0F** | traversal 域收口 |
| TreeWalker.html / NodeIterator 全族 | 761P / 795P | 同 | 持平 |
| 全量 dom sweep | 54131P/68F/21T | **54128P/67F/25T** | Fail set 恰消失 walking-outside 一项零新增；4 个新 Timeout 单跑全 Pass（并发环境噪声，Timeout 双向漂移同款）|
| engine 单测 --lib | 2454 | 2454 | r315 测试扩为完整六步断言（+0）|
| lit 21P / e2e 20P / clippy（v8+guarded）/ fmt | — | 全绿/干净 | — |

## 四、域状态

- **traversal：0F**（R314 root 止步 + R315 identity 归一 + R316 relocated 置位三轮收口）
- collections：0F；Node-properties / ParentNode 全族 / Element-matches：0F
- dom 域剩余 Fail 全部为既存备档（events 4F 深结构、MO 4F 备档、R222-probe/zz-r54 探针自留件、shadow/crash 家族既存）

## 五、教训

「快照外着陆」与「setter 重定位」是同一 stale 状态的两个来源——入口条件要按**状态语义**
（快照是否含当前位置）而非**触发路径**枚举。R97 修 setter 路径时未穷尽着陆来源，残留
三年直到三轮 regraft 修复链（R314/R315/R316）才暴露。
