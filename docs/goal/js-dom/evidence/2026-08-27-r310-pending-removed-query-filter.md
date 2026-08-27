# R310 Evidence — pending-removed 过滤：同 turn remove 后查询不再含已移除元素（L2「查询读 live」removed 面）

**日期**: 2026-08-27
**切片**: M4/L2——R310(a) pending-removed 过滤轻量切片 + (c) dom 全域 sweep 复核启动
**改动面**: `part04.js`（sel-proxy querySelectorAll 的 host 结果 removed 过滤）+ `part24.rs`（+1 单测）

## 一、成果

| 套件 | 基线（main = R309 后） | R310 | Δ |
|---|---|---|---|
| ParentNode-querySelector 全族 | 2054P/0F | **2055P/0F** | +1P（removed 过滤解锁被移除元素残留卡住的 subtest） |
| Element-matches / MutationObserver / Node-insertBefore / closest | 全基线持平 | 同 | 持平（MO 117P/4F 既存） |
| Element-getElementsByTagName | 35P/1F（既存） | 同 | 持平（HTMLNess 变体既存失败） |
| vue e2e（integration） | 3P | 3P | 持平（remove 路径与 Vue unmount 无冲突） |
| engine 单测 --lib | 2447 | **2448** | +1（r310 三段断言） |
| make test | 1F 环境项 | 同 | 持平 |
| fmt / clippy | — | 干净 | — |

## 二、根因与修复

**复现**（sandbox 探针）：`container.removeChild(a2)` 后同 turn
`container.querySelectorAll('a.test')` 仍返 `a1,a2`；`a1.remove()` 后仍 2 个——
remove 经 host 异步 apply，快照查询恒含已移除元素（spec
`dom-parentnode-queryselectorall` 查 live 树）。

**修复**（part04 sel-proxy querySelectorAll 的 host 结果后处理）：本容器 pending 桶
`removed` 非空时，对每个 removed 节点经 `_zwHCCollectSubtree` 展开子树，取有
`__zwSelector` 的条目构造剔除集，从 host 返回的 sel 列表中过滤。probe 后：
`afterRemove=a1|afterMethodRemove=0`（removeChild 与 `.remove()` 方法两形态全对）。

## 三、L2 主线状态

R309（innerHTML 替换域重建）+ R310（removed 过滤）= 主文档域「查询读 live」的两面
轻量切片。剩余（较深，需统一方案）：
- **普通 append 域的 identity 双源**（R309 教训：基底快照 wrapper 与 pending wrapper
  不同对象——重建域判据不可扩到 append，需 wrapper identity 归一）；
- **querySelector（单数）路径**（本轮只修 querySelectorAll——单数路径同源语义下轮）。

## 四、dom 全域 sweep 复核（(c)）

后台启动全量 `make testharness-dom`（TIME_LIMIT=2400，约 30 分钟）——R306–R310 五轮
行为面变化后的跨文件漂移审计。结果落 `evidence/` 追记（若与逐套件 A/B 数字一致则
确认无漂移；不一致处归因）。

## 五、教训

后台 `make` 会拾取工作树未提交变更重建 binary——混合态 sweep 不严谨；先 kill 再以
定稿树重启（本轮即如此处理）。
