# 嵌套簇算法刀定谳——期望钉语料校正 + E1 移除侧坐标终验（nested_cross_b 解钉）

- 日期：2026-11-13
- 数据源：`.acceptance/dps/dbg-nested-cross-b.log`（首轮全探针）、`dbg-nested-cross-b2.log`（路径归因 TEMP 探针）、`e1-targeted-format-apply.log`（解钉后四测全绿）、语料 `tests/wpt-runner/wpt-data/editing/data/bold.js` 同形行实录

## 定谳核心：旧期望钉 `fo<b>o<b>b</b>ar</b>baz` 系 spec 推演误读，Chrome 实录为 flat 合并

WPT 语料 bold.js 含本测试完全同形的选区行（styleWithCSS=false 变体，与测试默认一致）：

```
["fo[o<b>bar</b>b]az", [["stylewithcss","false"],["bold",""]], "fo<b>[obarb]</b>az", ...]
```

即 Chrome 对「完全含于选区的旧 <b>」处置 = 溶解 + 选区内全部文本并入**单个**新 `<b>`（无嵌套二次 re-wrap）。同族边界形实录互证：`fo[o<b>b]ar</b>baz → fo<b>[ob]ar</b>baz`（部分相交=并入既有 <b>）、`foo<b>ba[r</b>b]az → foo<b>ba[rb]</b>az`。旧钉嵌套形出自 vendored spec 算法推演，无实录依据——「vendored contained 集算法面二次 re-wrap 缺口」立项即不成立，本刀无算法改码。

multitest.js 语料零同形行（grep 0 hit）——mt 门零翻越预期成立，行为覆盖由解钉测承载（同 D2/B1 判读形态）。

## 调试链（B1 后 + D1 工作树态实测）

- **JS 视图**（D1 在树）：`fo<b>obarb</b>az`——**已等于语料实录**。contained 走查链 `o→bar→b→az` 不再截断（B1 后曾 `fo<b>o</b>barbaz`＝截断丢 'bar'/'b'），D1 跨父移动文本 `_zwForceParentLink` 重挂 parentNode 为必要条件（兄弟 getter 经 node.parentNode 动态求值）。
- **host replay**（E1 前）：`<b>obarb</b>az`——`'fo'` 丢失 + 旧 'bar'/'b' 被错删，JS/host 双视图分裂。
- **mutation 流**（19 条）：splitText 双发射正确；溶壳 `RemoveChildAt(b,0)+InsertAdjacentText(beforebegin,'bar')+Remove(b)`；wrap 序 `CreateElement(b)/InsertChildAt/RemoveHandle/AppendChildByHandle`；**两步净效果搬移发射坐标错**（见下）。

## E1 路径归因（TEMP 探针 `_rdPath/idx/fused`，dbg-nested-cross-b2.log）

| 步 | 子 | 路径 | 发射 idx | 融合视图真位 | 后果 |
|---|---|---|---|---|---|
| [6] 溶壳 | bar（自 b） | base | 0 | 0 | 正确 |
| [13] wrap | bar（自 ce） | **stamp** | 0 | **2** | 戳=物化时父内索引（旧 <b> 内 0），RemoveChildAt(#ce,0) 打到 'fo' |
| [16] wrap | b | **base** | 1 | **2** | 基缓存 identity 命中带过期序——同批先行裸 AdjacentText 插入无 handle 不进 `_zwBaseIdxShiftForAdded` 平移，idx1 打到 b2 壳 |

## 修复面（part04.js slice24 RemoveChildAt 分支）

1. **D1（前轮在飞工作树，本刀收编）**：跨父移动文本两臂（带位/append）`_zwForceParentLink(newNode, _makeProxy(sel, handle))` 重挂 JS 视图父链——contained 集走查可达 'bar'/'b'。
2. **E1（本刀）**：三路径指数（基缓存 identity+位移算术→戳→融合线性）终验——计算位在融合视图须仍是本子，失配且线性可定位才改写（旁路视图融合 miss 保旧行为）。融合视图列表一次物化三消费共用（兜底/终验/兄弟 record——原兄弟块二次物化，净调用数不增）。
3. **测试**：`test_execcommand_format_apply_nested_cross_b_r3254_m3` 解 `#[ignore]`，期望钉校正 `fo<b>obarb</b>az`（语料实录 + 双视图一致），注记改记三笔前置（B1/D1/E1）；format_apply 块 ①b 注记与 doc_end 钉测 stale「仍 #[ignore]」引注同步更新。

## 验证

- 解钉后 format_apply 四测全绿（nested_cross_b / italic_after_prior_cmd / doc_end / flat 主体）`e1-targeted-format-apply.log` RC=0。
- enginelib 全量预检：见下方门禁批节（名册 7F 恒一 + 解钉测参与跑）。

## 门禁批收口（batch-nested-e1-gate-v1 ALL-DONE，2026-11-13 01:35）

v1 首射（pid 1742395 00:39:15，锚 ae4c866b5/bin 2233a311）ABORT-DOMAIN-DRIFT：在飞窗并行轮 pull --rebase 拉入 s414 重放刀体——协议性域漂移非行为红（s414=bd41f9bd2 part03.js +12 行纯 FIXME 注记惰性，commit 自证 sweep ×2 全等 8274P/214F/16T、join-cross 零位移；css-red.txt 语料补拉不触及 editing 腿）；已得 ab/probe 结果字节恒一可传递。r2 重锚重发射（EXPECTED_HEAD=ffdcd28af + bin 788d4b10c0710934d7e2c6a3e52ef131，pid 1752171 00:46:55 起，日志 `batch-nested-e1-gate-v1-r2.log`）48min46s 全绿零 ABORT。

| 腿 | 终值 | 判据 | 结果 |
|---|---|---|---|
| L0 probe zz-probe-outdent | 217/5 | 恰集（failkeys vs v2 diff 空） | OK |
| L3b ab fresh | 10092P/0F/0T wall=258s | 严格 + judge vs v2 post 四零 | OK |
| L1 outdent 主门 | rows 2550 恒一；F2P=0/P2F=0/lost=0/residF=4 恰键 | vs v2 post（复绑批基线严格门） | OK |
| L2 indent | 1302P/27F | vs v2 post 恒一 | OK |
| L3 mt 主页 | rows 10092 恒一；F2P=0（signed=0/unsigned=0）/P2F=0/po_fail=0/lost=0/residF=2 | vs v2 post；signed F2P 允许位未动用；残红=X37 双键（subscript qS-after + formatblock qV-after）维持 | OK |
| L4 singles×5 | iol 1759/iul 2061/ip 6989/ilb 1658/ihtml 1446 | vs v2 post judge VERDICT OK | OK |
| L5 exec×3 | 三页 judge VERDICT OK | 零漂移 | OK |
| L6 enginelib | **3911P/7F/0I** failed==7 逐名=名册恒一 | +1P/−1I=nested_cross_b 解钉测参与跑绿（italic 同形已于 B1 转 P） | OK |

- **判读**：八腿对 v2 复绑批 post（nested-b1-rb-*，bin 95ceadfe @e1488f6c8）零 flip 零漂移——E1（RemoveChildAt 发射坐标终验）+D1（跨父移动文本父链）修复域对全语料零意外暴露；mt 零翻越=语料零同形行（multitest.js grep 0 hit），行为覆盖由解钉测承载（同 D2/B1 判读形态）；enginelib 3911P=3910+1（解钉测参与跑）、0I=嵌套簇两枚 `#[ignore]` 全解除、7F 名册逐名恒一。
- **字节绑定**：全腿 bin=788d4b10c0710934d7e2c6a3e52ef131 恒一；锚复读特征串 E1=7/E1V=1/D1=3/B1=3/D2=2 全中；判读域=ffdcd28af（E1 刀体 × s414 组合态）。
- 并发纪律实录：同 session 三轮在 00:39–00:55 窗口竞发/让位/核验（双执行 loser 截断 winner 日志一例），按「先落地发射归发射轮」条款收敛，r2 判读收口归发射棒 rollover。
