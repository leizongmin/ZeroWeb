# R303 Evidence — MO disconnect record 清空（disconnect 套件 2P/0F 100%；inner-outer 归因 R220）

**日期**: 2026-08-27
**切片**: M4——R303(a) MO 剩余 4F 续（disconnect 1F 全解 + inner-outer 1F 归因深结构）
**改动面**: `part01.js`（disconnect 清空 `_records`）+ `part24.rs`（+1 回归 + 1 归因探针单测）

## 一、disconnect 修复

WPT `MutationObserver-disconnect` "disconnect discarded some mutations"：
observe→mutate×2→disconnect→observe→mutate→disconnect→observe→mutate 序列后回调
期望仅 1 条 record（前两段被 disconnect 丢弃）。旧 `disconnect` 只清
`_targets`/`_targetProxies`，**不清 `_records`** → 收 4 条。

修：`disconnect` 补 `this._records = []`（spec `dom-mutationobserver-disconnect`
步骤 2「empty the observer's record queue」）。一行修复 + engine 回归单测
（三段序列后 counts === '1'）。

## 二、inner-outer 1F 归因（探针记档，非本轮修复）

WPT `MutationObserver-inner-outer` "innerHTML with 2 children mutation"：
`n01.innerHTML = "<span>new</span><span>text</span>"` 后期望
`addedNodes === [n01.firstChild, n01.lastChild]`（**断言时读**——消费方读 live 视图）。

sandbox 探针实证（微任务时点）：
- `recs=1 / adLen=2 / ad0=SPAN`（record 的 addedNodes 是 `_zwFragmentAdded`
  解析 wrapper）✓；
- **`fc=old text`**——`n01.firstChild`（sel-based 元素）仍读 **stale host 快照**
  （mutation 未 apply，JS 融合视图不含 added wrapper——wrapper 无 `__zwHandle`
  不入 pending 融合）；
- `fcEqAd0=false / lcEqAd1=false`——identity 断言必然失败。

**归档**：根因 = sel-based innerHTML 的**同 turn firstChild 可见性**（新子不并入
JS 读视图）——R56 注记的既有缺口（"added 不并入"当时以删缓存缓解，读取回落
stale 快照）。与 R302 cross-realm 的「工厂节点不可观察」、R299 mixed-case 的
「host 不见 append-in handle 子树」同属 **R220 live-view 视图桥域**——
handle/工厂节点挂入 sel 容器（或 innerHTML 替换）后，JS 侧读写视图需立即可见。
归 R220 统一方案（M1 L2 live Document 域的第一批消费用例）。

## 三、验证

| 套件 | 基线 | R303 | Δ |
|---|---|---|---|
| **MutationObserver-disconnect** | 1P/1F | **2P/0F（100%）** | +1P/-1F |
| MutationObserver 全族 | 115P/6F | **116P/5F** | +1P/-1F（恰 disconnect） |
| MutationObserver-takeRecords（disconnect 邻居） | 3P | 3P | 持平 |
| engine 单测 | 2440 | **2442**（r303 回归 + 归因探针） | +2 |
| make test | — | 1F = XOpenDisplayFailed 环境项 | 持平 |
| fmt / clippy | — | 干净 | — |

## 四、MO 剩余 5F 终局归属

- cross-realm 1F + inner-outer 1F → **R220 live-view 域**（工厂节点可观察 id /
  innerHTML 同 turn 视图桥）；
- document 3F → parse-time MO 基建（parser 插入记录——文档解析期不发 MO，
  深结构已记档多轮）。

nodes 域 selector 小簇 + MO 簇的**轻量修复面已尽**——剩余 F 全部归深结构域
（R220 视图桥 / parse-time / R188 元素域可观察 id）。
