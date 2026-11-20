# inline-host ①簇跨批渗漏——两层根因归因与碰头仲裁（本流独立实现撤改记录）

**日期**: 2026-10-31
**前置**: [attr 门落地](2026-10-31-attr-gate-land.md)（inline-host 80P/800F，失败采样两簇：跨子测文本渗漏 + D1 族 text-view 生命周期）。
**结局**: 本流独立实现同根修复与并行流 ghost text 轮撞车，按「消费域收敛、不回退对方修复面」先例采纳先落地对方版本（f8fe57abd），本流同面机制撤改零残留；本文保留两层根因归因、r259/r2823 bisect 副产物（独立佐证对方「视图补偿不落」决策）与 part92 引擎层钉测。落地账见 [ghost text 宿主镜像 evidence](2026-10-31-inline-host-ghost-text-host-mirror.md)。

## 1. 根因：两层叠加（本流独立归因，与并行流 zz-probe-abc 结论同根同判）

followed-by-text 族（`document.createTextNode` + `container.appendChild` + `finally remove()`）的跨子测渗漏（`abc`→`abcabc`→`abcefgabcefg` 累积）由两层缺陷叠加：

1. **同 turn 层**（上一轮已修，未独立成轮）：sel 父（文本句柄挂主文档元素）的
   `ChildNode.remove()` 无视图记账——append 记入 pending-added 的条目 remove 时不对冲，
   融合序列化（`_zwFusedInnerHtmlSel` 文本臂无 `_zwIsRemovedNode` 门）恒端出已移除文本。
   修 = `_zwHCLiveInvalidate` 直落视图层（不经 `_mo_notify`——其 removedNodes 臂清
   `_zwNodeParent` 反链，曾破坏 surroundContents HRE 回滚依赖）。
2. **跨批层**（本轮双方各自定位）：append 批 flush 后文本已落引擎基底；runner 快照重建
   （`refresh_if_html_changed` → `render()` 全量重建）清 `persistent_handle_nodes`，
   文本无 selector 不可 `rebind_handle_node` → 后续 `RemoveHandle` 引擎侧永久静默 no-op。
   基底残留随序列化永久渗漏。引擎 apply 层本身正确——part92 单测
   `ihost_cross_batch_text_remove_handle_resolves_via_persistent_nodes`（批 A
   CreateTextNode+AppendChild 共享 persistent 表，批 B RemoveHandle，断言容器空）钉绿。

## 2. 碰头与仲裁

| | 并行流（采纳，先落地） | 本流（撤改） |
|---|---|---|
| 判别面 | `_zwChildBaseCache` 扫 nodeType+data 首见索引 | `_zwTextRemoveDataOf`（建时登记+编辑缓存）+ `__zw_child_nodes` 原始基底扫描末个同内容文本 |
| 发门 | 跨代际桶门（stamp≠apply 代际，R383 同款）；engine 纯测试环境代际恒 0 全块惰性 | `_zwSuppressRemoveRecord` 帧豁免 + 墓碑登记表（代内、apply 换代清除） |
| 视图半区 | 有意不落（R294 两版 A/B 证伪） | `_zwHCLiveInvalidate` 消零 + `_childNodeList` 墓碑过滤 + extractContents R47 同帧扩展 |
| RemoveHandle | 恒发（journal 序：先删后补 childAt） | childAt 命中时跳发（防同批双发二次摘除） |
| 语料 | 80P→160P/720F（提交体账），合并域收缩后组合态 170P/710F | 80P→160P/720F（独立验证等额） |
| engine lib | 11F 已知红集 identical | 13F→11F（r259/r2823 两笔回归 bisect 修正后） |

仲裁依据：对方面更小（71 行纯新增 vs 本流四笔横跨 part04/05/06）、r259/r2823 结构性免疫
（代际门使 engine 测试环境全块惰性；本流为同等效果付出两笔回归修正的维护面）、语料等额、
已先落地且对方域收缩轮组合态门禁已验。按「消费域收敛、不回退对方修复面」先例执行：
part04/05/06 取对方版本（rebase 冲突 `git checkout origin/main` 解），本流墓碑登记表、
建时数据登记、R47 帧扩展全数撤改零残留；part92 钉测保留（双方修复共同依赖的引擎侧前提）。

## 3. 本流 bisect 副产物：独立佐证「视图补偿不落」

本流实现期 engine lib 曾 13F，bisect 三态（双关/单开②/单开①）归出两笔回归，均指向
remove 时点视图/缓存补偿的脆弱性——与对方「R294 两版证伪、视图半区补偿不落」的决策
相互独立地收敛到同一结论：

1. **r259_leaf_hre_extract_first_boundaries**（`bodyLast=leaf` 应 `#text`）：extract
   覆盖子摘除经 `kids[j].remove()` 落 R129 臂，`_zwHCLiveInvalidate` 消零把**仍
   pending-added** 的文本从 addedSet 对冲剔除，而 leaf-newParent 抛 HRE 的 sim 中间态
   不回滚视图（基线行为 = sel 父文本移除视图不可见，`bodyLast` 断言依赖此）。修正 =
   R47 帧豁免。**含义**：任何 remove 时点的视图补偿都会翻转依赖该中间态可见性的钉面——
   对方以代际门把整个补偿块在测试环境惰性化，结构性回避此类风险。
2. **test_character_data_methods_r2823**（splitText(2) 原节点应保 `'ab'`）：建时数据
   播种进 R121 读缓存会遮蔽 splitText 的 wire 写（`__zw_set_text_handle` 不经
   `_zwTextDataSet`）。修正 = 独立登记表。**含义**：为判别面引入的缓存状态有遮蔽
   既有写通道的风险——对方直接 `_makeProxy(sel,handle).data` 现读，零新增状态。

## 4. 验证账（解决树 = origin/main shim 逐字节一致 + part92/docs）

| 门 | 解决树 | 对方组合态账 |
|---|---|---|
| inline-host 语料 | **170P/710F**（独立复跑） | 170P/710F ✓ |
| engine lib | 3,613P/**11F**（= 对方 3612P + part92；11 红清单逐名恒等，r259/r2823 复绿未现） | 3612P/11F identical ✓ |
| join 三案 | 引对方组合态账（shim 逐字节一致，本提交余量仅测试+文档） | 7,518P/0F 全扫 |
| white-space-style / cannot-have-div | 引对方组合态账 | 616P/488F / 420P/0F 零位移 |
| webview lib | 引对方组合态账 | 822P/1F 已知红唯一 |
| fmt | clean | — |

## 5. 边界与挂账

- **同 turn 视图窗残留**（对方 t0 已知留置，本流 probe t1-t4 同窗）：remove 后至下次
  flush 之间的视图读仍见已移除文本——语料不断言该窗，设计上留置（见 §3：补偿该窗
  的两版尝试均翻红既有钉面）。
- **混合流**（同 turn CharacterData 编辑 → 跨批 remove）：本流方案曾以编辑缓存优先
  处理，撤改后该形态回落对方基底扫描（nodeType+data 现读）——编辑值与基底不一致时
  失配不移除（不劣化旧 RemoveHandle no-op 行为）；语料无该形态，维持记录不扩修。
- **②簇/主簇（残余 710F）未动**：对方定性 = insertLineBreak/insertParagraph 行内
  host 行断插入语义（LF+`<br>`）缺失，display 五型对称分布；本流 probe t5-t8 宿主级
  形态正确、容器级序列化缺失/多笔（`[]abcd`→空 span、`{}<br>`→多一个 br、`abcd[]`→
  尾 br 缺失）与该定性相容（行断未插入 → 序列化面缺行断产物）。下轮首位。
