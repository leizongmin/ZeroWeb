# B3 诊断轮：gtr beforeinput 908F 分解 + join 首触发事件丢失根因链（未落树）

日期：2026-11-16。范围：纯诊断 + 修复实验（shim 改动已回滚，零净码变）。

## 908F 真构成重分类（全语料 473P/2885F 态，探针 + 消息分类实证）

| 簇 | F | 形态 | 落点 |
|---|---|---|---|
| list-items 删除 | 592 | `<ul><li>list-item1{</li><li>list]-item2</li></ul>` 元素锚 range 选区 | `__zw_ce_delete_impl` 12026 sc 非 text silent return；**no-op 无树变**（O2 探针实证） |
| dl join | 152 | `<dl>…{</dl><dl>…]</dl>` 元素锚 | 同上（O3） |
| non-collapsed selection | 76 | 跨块 range 选区 | 同簇 + ④ 臂 defer |
| backspace/forwarddelete 主文件 | 90 | 混合：宿主起点 caret / src 非末子保守不并 / run 中段（br/img 前邻）/ 纯 div-p join | 多点（见刀清单） |
| got=2 双派 | 342 | 多案上下文 | **本轮修复实验直接暴露同机制嫌疑（下）** |

**事件丢失集群（探针 P15g 形纯 div-p-text join）在 908F 中占比极小**；此前 round-3 工作日志把它当主簇是误判。

## join 首触发事件丢失：根因链（探针 b3bi→b3k→b3l→b3m→b3n 全链）

现象：join 形（`<div><p>abc</p>[]def</div>`）**同页首触发** bi=[] input=0 但树变完成（abcdef）；二触起 bi=1 input=1。与 promise_test 序号无关（b3m M2 第 2 测 join 首触发仍丢），与 join 形首触发强相关。

定位数据（zz-probe-b3l/b3n）：

```
L1/N1 首触发: hook=1 disp=0 docCap=1 bi=[] input=0 after=abcdef
N1 补测:      querySelector(sel) 与 gEditor same=false sameNode=false（首跑未合流）
N2 二触发:    same=true sameNode=true（合流）→ bi=1 input=1
```

机制：首触发必走 `__zw_ce_caret_range` 世界同步回灌（10212 `!el.contains(锚)` → `el.innerHTML=_htmlSync` 幂等重建）+ 回植；此后 impl 世界内 querySelector/锚链上溯产物全落 query 家族代理，外部 listener 挂点（页面家族 gEditor 代理）在 impl 内不可达——派发沿 query 家族链冒泡（document capture 收 1 个 beforeinput）但宿主 listener 表零触发，且壳落位段跨家族引用抛异常丢 input（docCap=1 而 input=0）。二触起 contains 命中零开销跳过、家族合流 → 全通。

## 修复实验（4 刀递进，isolated 全兑现、语料面净 -2 → 回滚）

1. impl 内 re-query el：无效（家族身份由对象创建路径决定，不随树态）。
2. 派发目标换锚链上溯宿主（`_zwEvHost`）：isolated N1 bi=1 兑现；语料 P 473→471（-2），non-collapsed 双案 got=2。
3. `__zwCeOrigEvHost` global（caret_range 回植前捕获页面家族宿主代理供 impl 派发）：isolated N1 **bi=1 input=1 全兑现**（壳段抛异常也消失）；语料 div-p 案 **got 0→2**。
4. 条件化（`el.isSameNode(origHost)` 同节点保 el）：双派未消（多案态 isSameNode=false——代理换代后同节点不同对象）。

**语料面双派机制**（未定谳，下轮续）：每案 initializeTest 写 innerHTML → query 家族视图滞后 → **每案都回灌**（非仅首案）→ 未合流态恒存；修复打通派发后多案上下文暴露双收（div-p 案 0→2）。与既有 got=2 342F 集群（isolated 单收、corpus 双收）**机制同源嫌疑高**——打通正确性修复会把更多 got=0 案变成 got=2，**先修双派再落事件丢失刀**。

## 回滚确认

shim `git checkout` 后重建基线二进制：N1 bi=[] input=0（基线丢失态）、N2 bi=1——回滚干净，工作树零净码变。探针文件（zz-probe-b3bi/b3k/b3l/b3m/b3n/b3o，gitignored）留本地备查。

## 下轮刀清单（按收益排序）

1. **双派根因**（got=2 342F + 本轮暴露）：多案上下文 listener/代理叠影定位——b3o O1 isolated bi=1、corpus got=2，加 `__zwCeSyncCount` 回灌计数探针分辨「listener 叠挂」vs「双代理双表」。
2. **元素锚 range 选区删除语义**（≈820F 主簇）：12026/12155 silent return 的 select-delete 全语义（P15a/b/c 形），含 li 全选删除后列表结构维护。
3. **空 range beforeinput 静默臂**（主文件宿主起点/终点 caret，corpus 首案形）：12034/12187 改派 beforeinput（空 range 快照）不派 input——语义已定谳（`checkGetTargetRangesOfInputOnDoNothing`），小刀独立可落。
4. src 非末子保守不并（12040）/ run 中段（12038）按 corpus 案例逐形放刀。
