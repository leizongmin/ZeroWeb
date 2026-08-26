# R278 Evidence — iframe 域元素兄弟 getter + deleteContents sc-CharData/ec-element 分支（+5P，诊断链收口）

**日期**: 2026-08-26
**切片**: M4——R278(a) 22,x 复现 probe → 真根因修复
**改动面**: `part05.js`（`_zwIframeCreateElement` 兄弟 getter + insertBefore 自插入重定向）+ `part06.js`（deleteContents R278 分支）+ `part23.rs`（+1 单测）
**commit**: `c050908d0`

## 一、诊断链（R277 假设推翻：分歧无需多轮累积）

R277 三域 probe（顶层/单 iframe/单轮克隆）全部 clean，结论倾向「多轮克隆
轮转累积」。本轮 **真实文件注入 + identity 标记**（range 端点 PRE 打
`__r278tag`，POST 比对）一步复现：

1. **actual 侧 no-op 实证**：POST dump `nvE=full`（端点 text 未修剪）、
   `P#a:kids=1`（中段未删）——引擎 deleteContents 对 `[text,3,element,1]`
   全分支 miss（R213 同父 ✗ / R267 ancestor ✗ / R268 双 CharData ✗ /
   R269 同容器 ✗ / `_coveredChildren` sc≠ec 恒 null）→ 整体空转。
2. **expected 侧 oracle walk 饿死**：`oracle[n=0]`，爬升链 dump 实证
   **克隆域每一层 nextSibling=null**（P#a→null、DIV→null、BODY→null…）
   + `nextNode(sc)=null` 且 `stop=null` → `node != stop`（null!=null 为
   false）→ 遍历立即终止。
3. **R276「平行 wrapper」结论修正**：`firstChildSame=true`（firstChild
   **就是**被标记的端点对象，identity 单一）——比较遍历拿完整数据不是
   因为「新 wrapper」，而是因为 delete 从未发生。R274 的「克隆域
   nextSibling 断链」诊断才是真根因（R275/R276/R277 三轮修正后回归）。

## 二、修复（两处引擎 + 一处规范对齐）

### 修复 1：`_zwIframeCreateElement` 补 `_zwMDefineSiblings`（expected 侧）

根因：iframe 工厂元素是 plain 对象（无 sel/handle，不走 part04 proxy 的
sibling trap），原型链 `Element.prototype.nextSibling`（R3019
`_zwProtoOwnGetter`）对无 own 槽恒 null。修：R273 CDATA 同款动态 getter
（`parentNode.childNodes.indexOf` 现算，detached/边界 null）——与
`_zwMText`/`_zwMComment`/`_zwMEl` 工厂对齐，尾簇补元素域。

### 修复 2：deleteContents R278 分支（actual 侧，sc CharData + ec 元素）

spec partially-contained 语义：**ec 元素本体保留，仅其 [0, eo) 直接子
删除**（R268 首版教训整体排除 element 端点过宽）。sc 侧爬升/中段/塌缩
复用 R268 已验证形态。

### 修复 3（R279 前置，A/B 抓到的回归）：insertBefore 自插入重定向

oracle 解锁后暴露 Range-insertNode 28,0 回归（`[testDiv,0,comment,5]` +
node paras[0]）：旧「先摘再 indexOf(ref)」使 c===ref 自插入形态 push 到
尾部。修：spec `concept-node-pre-insert` 步骤 2「referenceChild is child
→ next sibling」重定向（读值在摘除前）+ 摘除后按 identity 重定位。

## 三、验证（A/B 全量 dom sweep，patched vs baseline 同日双跑）

| 项 | baseline | R278 | Δ |
|---|---|---|---|
| Range-deleteContents | 112P/13F | **117P/8F** | +5P（22/28/52,x DOM+cursor 六条 fail 消失） |
| Range-insertNode | 1840P/0F | 1840P/0F | 持平（首版回归被修复 3 修复） |
| Range-extractContents / cloneContents / surround | 31F/29F/0F | 同 | 持平 |
| mutations 五套件 | 超时 ×5 | 超时 ×5 | 持平（baseline 同超时，环境慢，预存） |
| dom/nodes | 12662P | 12661P | -1 = flaky 超时（crash 用例单跑 Pass） |
| dom/events / traversal / collections | 579/1603/49 | 同 | 持平 |
| engine 单测 | 2411 | **2412** | +1（r278 单测：克隆域链 b/c/cm/null） |
| fmt / clippy | — | 干净 | — |

**set-diff**：仅 mine 有 `Fail 24,x Resulting DOM`——非回归而是
**oracle-broken false-pass 翻真**：baseline 里 24,x DOM 靠双侧一致空转
侥幸相等；oracle 修复后暴露引擎 sc=element 跨容器缺口（记 R279 靶点）。

## 四、方法论沉淀

- **identity 标记 probe**：PRE 给 range 端点打 tag，POST 从两个导航面读
  回比对——一次跑出「identity 是否单一 + data 是否修剪」双结论，比纯
  dump 快（本轮 3 次跑完成 R276/R277 三轮未完成的定位）。
- **harness 只报首条断言消息**：同一 step 内多条 assert_true(false) 只有
  第一条的 message 进结果——多段 dump 须合并单条消息。
- **oracle 解锁的连带效应预期管理**：修 expected 侧基建会使「双侧一致
  错」的 false-pass 翻真（本轮 24,x、上轮 insertNode 28,0）——A/B
  set-diff 时先判「baseline pass 是否 oracle-broken 侥幸」再定性回归。

## 五、R279 靶点（残余 8F）

- **(a) sc=element 跨容器族**（24,x `[testDiv,2,paras[4],1]`、48,x
  `[testDiv,1,paras[2].firstChild,5]`）：R278 分支的对称缺口——sc 元素
  的 [so,end) 右侧子删 + 方向分支 contained 递归（R268 首版教训的另一半）。
- **(b) 49/50,x cursor-only**（docEl/body 容器）+ **53,x**（`[paras[3],1,
  comment,8]` element sc + comment ec）。
- **(c) extractContents 31F / cloneContents 29F 重聚类**（oracle 现在可
  遍历，expected 侧数据真实化——值得重新取样）。
