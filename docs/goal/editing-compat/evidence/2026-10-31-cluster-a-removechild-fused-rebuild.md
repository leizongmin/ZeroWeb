# 簇 A glue 残余归因：sel 域 removeChild 每 removal 3 次融合视图全量物化（O(n²)）——R126→R5340 同调用列表复用收刀

**日期**: 2026-10-31
**前置**: [簇 A clone 归因](2026-10-30-cluster-a-clone-perf-attribution.md) §4/§5 挂账——~650µs text-child 差额
「需 trap 内部插桩才能收口」+ §8 验收线「remove 侧每子 ~250µs 为下轮热点」；本文 = 该挂账的插桩收口轮。
**结果**: 探针 w2（sel 域 removeChild×2000）tape 归因每 removal **3 次 `_childNodeList`
全量物化**（占墙钟 98%）；同调用复用收刀后 w2 均值 **6661→4109ms（−38%）**，`_childNodeList`
6001→4001 次恰 −1/3，其余 7 窗口零位移。

## 1. 方法

tape 插桩沿 [ring-registry 学习](../../../learnings/performance/2026-10/2026-10-30-ring-registry-length-gate-hidden-quadratic.md)
的方法沉淀：shim 临时顺序 delta 段带 + 闭包持有逐调用累计（不走 globalThis 热路径，仅
reset/dump 两读出函数挂 globalThis），`Date.now()`（µs 级段差在本轮 ~ms 级段上精度足够）。
检查点 24 处：innerHTML setter 10-19（物化前/后、host set、CE 断连/连接、R304/R3k/stamp、
notify）、removeChild trap 20-33（R126 后、R87/slice24/R5340 各臂、R294/host remove/notify、
R195/R125 门）、`_r126IsChildOf` 40-42、`_childNodeList` 逐调用统计（n/ms/条目累计）。
探针页 8 窗口 ×3 样本（同二进制）：sel 域（gEBI 容器）fresh-set/re-set/removeChild×2000、
handle 域（createElement 容器）同三窗、text 子 build/remove 臂。测完 tape 整体 `git checkout`
回退（探针页 gitignore 域，不入库），干净二进制复跑核数。

## 2. 归因结果（tape，3 样本）

| 窗口 | wall ms（3 样本） | 主项 |
|---|---|---|
| w2 sel 域 removeChild×2000 | 6800/6064/7119（均值 6661，3.3ms/次） | **`_childNodeList` 6001 次共 6641ms = 墙钟 98%**，均值 ~2000 条/次 |
| w0/w1 sel 域 set/re-set | 71-100 | `_zwFragmentAdded` 50ms + notify/PAIdRemove 尾段 |
| w3/w4 handle 域 set/re-set | 103-186 | t8r 物化 + notify 尾段 |
| w5 handle 域 removeChild×2000 | 887 | 28>30 段（`_unrecordHandleChild`+notify+LiveNLSync）548ms；fused 重建未触发（`!sel` 快速返空） |
| w6/w7 text 子 build/remove | 159/991 | w7 与 w5 差 +12% |

四点结论：

1. **sel 域 removeChild villain 钉死**：每 removal 恰 3 次 `_childNodeList(sel, handle)`
   全量物化——① trap 入口 `_r126IsChildOf` 融合子判定 1 次、② R5340 臂 pending 索引
   扫描 1 次、③ R5340 体尾 `_zwLiveNLSync[sel]()` live childNodes 重建 1 次。每次物化 =
   host 串回环 + `JSON.parse`（~2000 条）+ 逐子 `_wrapNodeEntry` wrapper 构造（~1.1ms），
   填充容器上 = O(n²)。`_zwChildBaseCache` 对此无效：mutation 经 notify/LiveNLSync 每
   removal 失效基底缓存，下一轮全量重建（缓存「每 mutation 失效」设计与 mutation 循环
   天然相克）。handle 域同循环不触发（`_childNodeList` `!sel` 快速返空，R126 走
   `_handleChildren` 平面数组）——热点是 sel 域特有。
2. **setter 面已不热**：两域 set/re-set 71-186ms——前轮 §7 三笔（globalThis miss/R155
   反查/插入侧二次方）+ R263b/R262b 预筛后，innerHTML 路径残余为 `_zwFragmentAdded`
   解析 + 逐子簿记常数项。探针无 Selection/MO/range 语境，绝对值不可直接对 §6 验收线
   （S5 ≤300 等在真语料语境测量）；机制结论（ setter 非残余主项）可迁移。
3. **§4 ~650µs text-child 差额定谳为历史态**：本轮 w7 vs w5 仅 +12%（991 vs 887ms），
   差额不复现——该数字系前轮修复前测量，挂账关闭。
4. **R5340 体尾 LiveNLSync 与 R126 物化**为修复后残余双主项（见 §5）。

## 3. 修复：R126→R5340 同调用融合列表复用（R5340-glue，part04）

三次物化中前两次（R126 子判定 + R5340 索引扫描）在同一 removeChild 调用内**背靠背执行、
中间零 mutation**（只有 stamp/门条件纯读，无用户回调、无 host 往返）——同一列表建两次
纯浪费。复用实现（part04，+23 行）：

- `_r126FuseMemo`/`_r126FuseMemoKey` 声明于 get-trap 闭包体作用域（与
  `_r126IsChildOf` 同层）——每次属性访问独立实例，跨访问天然隔离，无跨调用陈旧面。
- `_r126IsChildOf` sel 路径构建后盖章 `(parentSel, parentHandle)` 键；
- R5340 臂键匹配即消费、消费即清；不匹配照旧重建（保守回落）。
- removeChild 入口防御清——同一闭包被页面留存复用时（`var rm = el.removeChild`）两次
  调用间必有 mutation，跨调用复用不成立。
- replaceChild（R127）等他消费者只置不清，随闭包实例消亡，无泄漏面。

语义等价：同调用内两次 `_childNodeList` 入参全同、其间无 mutation，identity 扫描与
索引定位作用于同一数组 = 逐位同结果。R5340 臂自身逻辑（wire/bucket retract/通知/断链）
零改动。

**否决的替代方案**（留档）：stamp+bucket 快路（`_zwSelPendingParent` 匹配即跳物化）——
overlay 语义含跨代际门（R383/R383a）、种子替换（R5273/R5356/R5356b）、逐条目 ghost 确认
（R382）、CE walk 旁路（R4620），廉价谓词与 `_zwOverlayPendingChildNodes` 全语义不
等价，编辑语料恰是这些边缘形态的高发域，不取。

## 4. A/B（tape 同在，3+3 样本同二进制类）

| 指标 | 修复前 3 样本 | 修复后 3 样本 | Δ |
|---|---|---|---|
| w2 wall | 6800/6064/7119（6661） | 3792/4006/4530（**4109**） | **−38%** |
| w2 `_childNodeList` 次数 | 6001×3 | **4001**×3 | 恰 −1/3（R5340 重建消除） |
| w0-w1/w3-w7 其余 7 窗 | 71-100/103-186/887/162/991 | 73-98/103-186/901/159/988 | 噪声带内零位移 |

干净二进制（tape 剥离）复跑：w2 **4076ms**，其余窗口 73-978ms 与带 tape 版同带——
tape 自身开销 ~1%，不影响方向。

## 5. 门禁

干净修复二进制（R5340-glue 在、tape 剥离，`grep -a -c _r126FuseMemo`=7 /
`__zwTkReset`=0）逐面复验，全部 systemd `MemoryMax=24G` 守卫，逐位零位移：

| 面 | 结果 | 对现行账 |
|---|---|---|
| engine lib | **3,635P/1F** | 1F = s24 flip 已知红唯一（现行账 fetch 簇 s179/s180 清零后 1F 基线），逐名恒等 |
| webview lib | **832P/0F** | 830P/0F 上一账 + 并行流新测 2P，零红 |
| inline-host（insertparagraph-in-inline-editing-host） | **880P/0F** | 880P/0F 恒等 |
| join-different 三案（lrlp+lprl+paragraphs 单调用合跑） | **7,518P/0F/0T** | 7,518P/0F 全扫分母逐位恒等（lrlp 2607 + lprl 1455 + paragraphs 3456）；paragraphs 3,456 变体面本轮 **0T**（前数轮 1T/2T 为 R5342 预算墙沿随机截断，本轮 TIME_LIMIT=2400 未触墙） |

修二进制中途换入跑批的混合二进制面：gate 电池先起后因 ihost 面误选（见 §6）重启，
join 前段作废，终账以上述单干净二进制完整复跑为准。

**组合态复验（rebase 并入并行流 eb141f7e1 S8 移除侧收口 part05 + 0e976c863 s182
part03/part04 后，4e3db848d 单干净二进制）**：s182 part04 触点在 ~257/~12486（progress
数值分支），与本文 fix（~7636+）无区域重叠；S8 三笔全在 part05。复跑四门零位移——
engine lib 3,635P/1F（红唯一 s24 逐名）、webview lib 832P/0F、inline-host 880P/0F、
join 三案 7,518P/0F/0T；探针 w2=3925ms 修复效果保持（与 §4 修复后带 4109 同带，S8
侧并行修复无交互回退）。

## 6. 残余与挂账

- **每 removal 仍余 2 次物化**（R126 子判定 1 + LiveNLSync 1），总量仍 O(n²)。彻底
  O(n) 化两条路均触及深语义面：① R126 子判定廉价谓词需 overlay 全语义等价（§3 否决
  理由）；② LiveNLSync 增量化 / `_zwChildBaseCache` 扩 handle 域需重审 R55/R304 身份
  稳定副作用与 `_zwHCLiveInvalidate` 失效覆盖面。留独立整轮（沿 §5 时代「整体替换
  需独立整轮」同判据），本文件 A/B 数字即其基线。
- 探针页 zz-probe-rm-glue.html（editing/run/）与前轮 zw-probe-ihost-*.html 同域保留
  （gitignore），后续轮复用。
- run 面残余大簇重排（下一步 ②）待本轮门禁入账后按现行账推进。
