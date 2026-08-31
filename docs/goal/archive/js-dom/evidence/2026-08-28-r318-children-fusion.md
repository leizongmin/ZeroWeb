# R318 Evidence — children 融合路由 + live 维护（dom/collections 与 dom/events 双绿，+2P/-2F）

**日期**: 2026-08-28
**切片**: M4——R318(a) HTMLCollection live 边缘复核（R50 Proxy 承载后的残留面）
**改动面**: `part04.js`（children getter 的 sel 分支融合路由 + live matches）+ `part24.rs`（r318 探针测试保留为域事实断言）

## 一、缺口与修复

**Element-children "HTMLCollection edge cases 1"**（setup append 两个 createElementNS img
后枚举期望 6 得 4）：`children` 的 sel 分支直读 host `__zw_element_children` 快照——与
R317 修的 childElementCount/first/lastElementChild 同根因（同 turn append 不可见），但
children 是独立分支漏修。修：同款融合路由——优先 `_childNodeList(sel, null)` 过滤
nodeType 1，空则回落快照。探针 `base=4|after1=5|after2=6|own=…baz` 实证。

**ParentNode-children "should be a live collection"**（集合先建、append li 后 length 期望
5 得 4）：`length` 动态读 `state.els`，但 children 集合构建无 liveSpec——`_zwHCLiveInvalidate`
只记账 pending 桶不回写已建集合。修：补 `matches` 回调（nodeType 1 全收 + scope 槽），
`_zwMakeHTMLCollection` 构建期从 pending 表并入、mutation 后维护。

## 二、回归归因与回退决策（getElementsByClassName live 尝试）

同轮尝试给 `getElementsByClassName` 的 handle 分支补 liveSpec——`Event-dispatch-single-activation-behavior`
131F 全红当场抓回。根因：`_zwPendingAdded` 是**全文档级** pending 表，matches 只判类名
不判子树归属，把其它容器的同名类元素并进集合 → 用例的 `getElementsByClassNameInclusive`
拿错 click target → 激活序列断。scope 上行判定补丁后 activation 58P 仍非全绿（pending
期元素 parentNode 链的归属判定域）→ **按最小改动原则整体回退**，该 1F 维持既存备档。

**教训**：liveSpec.matches 的消费方（构建期并入 + mutation 期维护）都以「matches(el) 真
即归属」为前提——**scope 归因必须内建在 matches 语义里**，调用方无法事后过滤。无
子树归属原语的表（pending 无容器归因字段）上做集合 live 是结构性缺口，轻量修不安全。

## 三、A/B

| 套件 | R317 基线 | R318 | Δ |
|---|---|---|---|
| Element-children | 0P/2F | **2P/0F** | +2 |
| ParentNode-children | 0P/1F | **1P/0F** | +1 |
| Element-childElementCount ×3 / HTMLCollection 全家 / collections 目录 | 全绿 | 全绿 | 持平 |
| **全量 dom sweep** | 54138P/61F/21T | **54140P/59F/21T** | **+2P/-2F，Fail set 恰 -2、Timeout 集恒等** |
| Event-dispatch-single-activation-behavior | 132P | 132P | 零回归（回退验证链确认）|
| engine --lib（v8/quickjs）| 2455/1460 | **2456**/1460 | +1（r318 探针域事实）|
| events 目录 582P/4F / traversal 1604P/0F | — | 同 | 持平 |
| fmt / clippy（v8 guarded + quickjs）| — | 干净/0 | — |

## 四、域状态

- dom/collections：49P/0F 维持；HTMLCollection live 边缘 3F 全部收口（getElementsByClassName
  live 维持备档，属 pending 表 scope 归因结构域）
- dom/events：4F 备档维持；activation 132P 守稳
- 全量 Fail set 59 项全部为既存备档面
