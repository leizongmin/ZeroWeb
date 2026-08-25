# R275 Evidence — expected iframe 的扁平文本 body 定位（诊断轮：R274 假设再修正）

**日期**: 2026-08-26
**切片**: M4——R275(a) 克隆域元素 nextSibling 归因（假设再修正）
**改动面**: 无生产代码（诊断轮）

## 一、R274 假设修正

R274 归因「克隆域 P#a.nextSibling 断链」——**再修正**：注入 probe 检查
expected 侧 startContainer 的真实父链：

- `sc.parentNode` = **BODY**（nt=1，非 P#a！）+ `BODY.kids=1`（唯一子 =
  sc 本身——一个扁平文本节点）；
- 即 **expected iframe 的 body 从未建成结构化树**（无 DIV#test/paras），
  其内容是一个扁平文本 blob（完整序列化串）；
- expected range = (BODY-text, 3)-(?, 1)——「expected "Äb"」由 BODY-text
  尾段 deleteData(3) 得出（与 actual 的 paras[0].firstChild 尾段修剪值
  巧合一致，掩盖了树分歧）。

## 二、根因定性

**restoreIframe 的克隆链在 expected iframe 域产生扁平 body**：
- R272 probe 里 `contentWindow.paras=undefined` 的线索同源——expected
  iframe 的 setupRangeTests 建 paras 但 **append/insertBefore 系列静默
  no-op**（testDiv 不入 body → paras 不入 testDiv → body 内容以文本
  形态残留）；
- 候选缺口：克隆 docEl append 到 iframe doc 时 body 绑定（R221 的
  `_zwMarkup` 印记域 / fresh-doc 系列在 restoreIframe 双 iframe 轮转下的
  第二 iframe 形态）——actual iframe 的 body 绑定成功（actual 树正确），
  expected iframe（后建）失败。

**R276 修复方向**：restoreIframe 双 iframe 轮转的 body 绑定时序（第二个
iframe 的 `contentDocument.appendChild(clone)` 后 body/head 重指）——
R221 的 fresh-doc 重绑对**每次 appendChild 到不同 iframe doc** 的轮转
形态覆盖缺口。

## 三、方法论沉淀（三轮假设演进教训）

- R272「wrapper identity churn」→ R273 修正为 CDATA 兄弟槽缺失（真修复 +2P）
- R274「克隆域元素 nextSibling 断链」→ R275 修正为 expected iframe 扁平 body
- **教训**：deleteContents 系断言失败时先 dump **双侧 range 的 sc 父链**
  （`sc.parentNode.nodeType/tag/childNodes.length` 三件）——比 walk 追踪
  更早暴露「树根本不同构」的形态（walk 断链是果不是因）。

## 四、验证

| 项 | R273 | R275（诊断轮） |
|---|---|---|
| Range-deleteContents | 115P/14F | 115P/14F（文件已 restore 零残留） |

## 五、R276 靶点

- **(a) restoreIframe 双 iframe 轮转的 body 绑定**：expected iframe（后
  append 克隆者）的 body/head 重指缺口（R221 fresh-doc 系列邻域）——
  修复后 22/48/52/53,x 的 expected 树同构，断言才有对齐基础。
- (b) 28,x / 49/50,x；extract/clone 重聚类。
