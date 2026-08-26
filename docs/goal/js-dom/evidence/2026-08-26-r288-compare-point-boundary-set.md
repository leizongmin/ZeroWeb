# R288 Evidence — comparePoint 顺序 + compareBoundaryPoints 重写 + setStart/End 精确校验 + iframe load 微任务延迟（三大域 956F 全解）

**日期**: 2026-08-26
**切片**: M4——R288(a) comparePoint root-先于-doctype 顺序 + (b) compareBoundaryPoints 边界点对选取重写 + (c) setStart/setEnd 元素容器超长 offset 恢复精确校验 + (d) iframe load 事件 microtask 延迟派发
**改动面**: `part06.js`（comparePoint / compareBoundaryPoints / setStart / setEnd）+ `part01.js`（iframe load `_defer`）+ `part21.rs`（R214 期望更新）+ `part23.rs`（r259 fixture 修正 + 3 个 r288 单测）

## 一、修复内容（四个独立根因）

### (a) comparePoint 步骤序（124F 簇全解）

spec `dom-range-compare-point` 步骤序：root 不同 → WrongDocumentError **先于**
DocumentType 检查。旧序先抛 InvalidNodeTypeError 使 cross-root doctype 用例
（88/89,x）报错类型错。同根 doctype 仍 InvalidNodeTypeError（步骤 3 在 root
检查之后）。isPointInRange 同款序但 root 不同返 false 不抛（无需改，行为已对）。

### (b) compareBoundaryPoints 边界点对选取 + 位置比较重写（592F 簇全解）

旧版两缺陷：
1. **how=2（END_TO_END）落入默认分支成了 START_TO_START 语义**——旧代码
   `howN === 1 ? end : (howN === 3 ? start : start)` 对 how=2 取 start 对，
   同容器 offset 差恒等 → "expected -1 got 0" 12F + 跨容器符号对调 282F。
2. **跨容器走 compareDocumentPosition 位**——FOLLOWING/PRECEDING 只有树序，
   祖先/后代容器对（point 在祖先 offset 与后代树序之间）无 offset-vs-
   childIndex 比较（WPT 1,17,x `[pf,0] vs [body,4]` 族 56F 符号反）。

修法：按 spec 表选取 (this, source) 边界点对——`this` 侧 how∈{1,2} 取 end
（START_TO_END 比 this.end vs src.start；END_TO_END 比 this.end vs src.end），
source 侧 how∈{0,1} 取 start。同容器按 offset 差；跨容器复用 R203
`_zwRangeBpAfter`（祖先 offset-vs-childIndex + 深度感知双 climb，与
comparePoint/isPointInRange/setStart crossing 同源）。

### (c) setStart/setEnd 元素容器超长 offset 精确校验（240F 簇全解）

spec `range-set-start/end` 步骤 2：offset > node length 对**元素容器同样抛
IndexSizeError**。旧版对 `nodeType !== 1` 才校验（历史动机：handle proxy
childNodes 视图缺失时 length 不可判定）——R286 起 registry 事实源使长度可判
定，放宽不再必要。WPT Range-set "too-large offset must throw" 30/39/40
（`[documentElement,7]`/`[paras[0],2]`/`[paras[1],2]`）240F 全解。

### (d) iframe load 事件 microtask 延迟派发（测试基建正确性）

spec（HTML「the end」+ event-loop-processing-model）iframe 加载完成是异步
任务：`.src = ...` 赋值语句本身须先完成，load 才触发。旧同步派发使 onload
handler 在同一赋值表达式的语句序列中间执行（WPT Range mega-case 的
`expectedIframe.src = ...; referenceDoc.appendChild(...)` 链——旧序 onload 先
跑，referenceDoc 尚空，restoreIframe 克隆出空 BODY 使 16,x `[body,4]` 超长
IndexSizeError）。`_defer`（microtask）在当前脚本任务末尾派发，此时 onload
handler 已赋值、referenceDoc 已填充。附带使 insertNode/surroundContents
16,x 46F×2 全解（R255/R258 时代的时序深项收口）。

## 二、验证（A/B：stash 前后同命令对比）

| 套件 | R287 基线 | R288 | Δ |
|---|---|---|---|
| Range-compareBoundaryPoints | 8722P/**592F** | **9314P/0F（100%）** | +592 |
| Range-set | 10680P/**240F** | **10920P/0F（100%）** | +240 |
| Range-comparePoint | 5459P/**124F** | **5583P/0F（100%）** | +124 |
| Range-insertNode / surround | 1840P/0F 各 | 同 | 持平（16,x 连带已在前轮） |
| Range-delete/extract/clone | 125/187/187P 0F | 同 | 持平（A/B 复核） |
| engine 单测 | 2420 | **2423** | +3（r288 三单测） |

## 三、dom 全量（单跑，含 23 Timeout 环境慢用例）

| 域 | R287 | R288 | Δ |
|---|---|---|---|
| dom 全量 | 52778P | **53733P** | +955（≈ 592+240+124 三簇） |
| dom/ranges | 37885P | **38841P** | +956（nodes -1 已知 flaky 抵消） |
| dom/nodes / events / traversal / collections | 12662/579/1603/49P | 12662/578/1603/49P | 持平（events -1 = webkit-animation Timeout 环境慢） |

ranges 域剩余 Fail（A/B 证实全部 pre-existing，非本轮回归）：
- `Range-constructor`（1F：startContainer expected Document got null——构造器
  域，独立切片）
- `Range-selectNode`（1F：setup 报 length undefined——testNodeInput 域）
- `Range-intersectsNode-2`（1F）、`Range-in-shadow-after-the-shadow-removed`
  （2F：shadow 域）
- `Range-mutations-{insert,delete,append,replace}Data/dataChange`（5F：执行
  超时 90s 族，环境慢低 ROI 备档——R261 已归因）
- 本地诊断探针（zz-r54*/R222-probe）：非上游用例，不计。

## 四、测试基建修正

- `r259_leaf_hre_extract_first` fixture 修正：body 须 ≥5 子使 `[body,4]/[body,5]`
  合法（原 4 子使 setEnd(body,5) 超长抛 IndexSizeError——WPT 真形态 6 子）。
  期望同步更新（HRE 抛出 + 边界保持 [4,5)）。删除遗留 R259PROBE 调试行。
- `test_iframe_docelement_structure_r214` 期望更新：工厂期 docEl.childNodes
  恒空（R220 评估），`[docEl,1]` 超长按 spec 抛 IndexSizeError（旧 lax 校验吞
  掉）。真实 harness 的 12,x 走 restoreIframe 后的克隆 docEl（含 head/body 两
  子），[docEl,1] 合法——WPT 12,x 1840P/0F 保持。

## 五、R289 靶点

- **(a) Range-constructor 1F**（startContainer got null——`new Range()` 的
  document 关联域，小簇独立可修）。
- **(b) Range-selectNode 1F**（setup 阶段 length undefined）。
- **(c) Range-intersectsNode-2 + in-shadow-after-removed**（shadow 域 3F）。
- **(d) mutations-data 5F 超时族**（环境慢，低 ROI 备档）。
