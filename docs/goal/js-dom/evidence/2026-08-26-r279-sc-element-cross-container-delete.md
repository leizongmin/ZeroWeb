# R279 Evidence — sc=element 跨容器 deleteContents 分支（deleteContents 100%）

**日期**: 2026-08-26
**切片**: M4——R279(a) sc 元素端点跨容器删除族
**改动面**: `part06.js`（deleteContents R279 分支：尾部止于 ec 路径子 + 同树位守卫 + 塌缩序）+ `part23.rs`（+1 单测）
**commit**: `75ae63c21`

## 一、R278 遗留形态（本轮起点 117P/8F）

| 形态 | range | 特征 |
|---|---|---|
| 24,x | `[testDiv,2,paras[4],1]` | sc=DIV 元素 + ec=P#e 元素（R278 翻真的真缺口） |
| 48,x | `[testDiv,1,paras[2].firstChild,5]` | sc=DIV 元素 + ec=深后代 CharData |
| 49/50,x | `[docEl,1,body,0]` | cursor-only：同树位空删 + 塌缩位 |
| 53,x | `[paras[3],1,comment,8]` | sc=P#d 元素 + ec=DIV comment |

## 二、实现（R279 分支，插在 R268 与 R278 之间）

骨架复用 R268/R278（cac 定位 + 双侧爬升 + 中段开区间 + rmNode），
三个关键差异（每个都是首版跑红后修正的）：

1. **sc 元素尾部止于 ec 的路径子**：`[so, ecPathIdx)` 逆序移除——
   首版 `[so, end)` 全删使 24,x got 2 expected 5（把 ec 本体和 ec 后
   兄弟一并删了——partially-contained 语义：ec 是端点本体不动，ec 后
   的兄弟在树序上位于区间外）。
2. **同树位守卫**：`sc.childNodes[so] === ec && eo === 0` → (sc,so)
   与 (ec,0) 是同一树位，区间空、零删除；塌缩 **(sc,so)**（spec 塌缩
   序「sc 是 ec 的 ancestor container → (sc,so)」——首版塌 (ec,0) 使
   49,x 的 startContainer 断言失败）。
3. **塌缩序分叉**：`cac === sc`（sc 自身就是 ec 的祖先容器）→
   (sc,so)；否则 R268 式 (cac, sIdx+1)——首版对 cac===sc 形态把 sRef
   爬过 cac（BODY→doc）rIdx=-1 → 塌 (cac,0)，24,x expected (DIV,2)
   got 0。

## 三、验证（A/B 双全量 sweep，vs R278 轮）

| 项 | R278 | R279 | Δ |
|---|---|---|---|
| Range-deleteContents（light DOM） | 117P/8F | **125P/0F（100%）** | +8P |
| ranges 全量 | 37789P | **37797P** | +8 |
| dom 全量 | 52707P | **52717P** | +10 |
| set-diff 回归 | — | **0 条 only-R279 fail** | — |
| dom/nodes | 12661P（含 1 flaky） | **12663P**（flaky 回归） | +2 |
| events / traversal / collections | 579/1603/49 | 同 | 持平 |
| extract / clone / insertNode / surround | 156P/158P/1841P/1840P | 同 | 持平 |
| engine 单测 | 2412 | **2413** | +1（r279 三形态单测） |
| fmt / clippy | 干净 | 干净 | — |

**deleteContents 累计**（R266 起）：80P/49F → **125P/0F**（R266 +12 /
R267 +6 / R268 +4 / R269 +6 / R270 +5 / R273 +2 / R278 +5 / R279 +8，
十三轮 +45）。预存遗留：ShadowRoot 一例（`{<span>ABC</span>}` 全删
形态，R194 轻量 shadow 域）与 mutations 环境超时族（R261(a) 已归因）。

## 四、教训

- **element sc 的尾部规则**是 R268 首版教训的另一半：sc 元素
  partially-contained，尾部 [so, **ecPathIdx**) 而非 [so, end)——
  「删到 ec 路径子为止」这条边界对 sc/ec 双侧元素形态同样成立。
- **塌缩位与删除量同权重**：deleteContents 的断言面有 DOM 树 +
  cursor 位置两层，cursor-only 失败（DOM 已过）时先查塌缩位的
  ancestor-container 分支，不是删除逻辑。
- **单测三形态同构**：f24/f48/f49 三形态须用**独立 paras**（复用
  P#e 使 f24 的空壳化 bleed 进 f48 的期望树——第二轮才抓到）。

## 五、R280 靶点

- **(a) extractContents 32F / cloneContents 29F 重聚类**：R278 的
  oracle 复活 + R279 的 sc-element 语义使 expected 侧数据真实化——
  旧聚类（R227 时代）大概率过时，可能有 false-pass 翻真簇或直接
  可修簇（delete 侧十三轮的分支模式可直接移植到 extract）。
- **(b) deleteContents ShadowRoot 一例**（`{<span>ABC</span>}` 形态）。
- **(c) mutations 超时族**（环境慢，低 ROI 备档）。
