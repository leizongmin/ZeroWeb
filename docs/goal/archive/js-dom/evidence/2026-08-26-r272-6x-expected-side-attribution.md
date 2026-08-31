# R272 Evidence — 6,x expected 侧归因收口（诊断轮：wrapper-identity churn 定位，无代码 land）

**日期**: 2026-08-26
**切片**: M4——R272(a) 6,x expected 侧克隆树归因
**改动面**: 无生产代码（诊断轮）；probe 方法论沉淀

## 一、诊断链（dual-iframe probe 复刻）

R271 probe 只验证了 actual 侧（顶层文档域）。本轮构建 **dual-iframe probe**
（复刻 Range-deleteContents 的 iframe src=Range-test-iframe.html + onload
链 + restoreIframe 克隆重建 + oracle 内联），dump 双侧 paras[5] 树：

- PRE-A === PRE-E === [CDATA(1234), CDATA(5678), text(9012)]（克隆保真无差异）
- POST-A = [CDATA(12), text()]——**actual 侧完全正确**（R271 在真实测试
  上下文生效）
- POST-E = [CDATA(12), **CDATA(5678), text(9012)**]——**oracle 侧中段 CDATA
  未删 + ec 头段未削**

## 二、根因定位（关键探针）

在 oracle 之前**直接**调 `pE.removeChild(midE)`（midE = pE.childNodes[1]）：
**成功**（post=2）且 oracle 之后 POST-E === POST-A。

结论：oracle 的 `nodesToRemove[i].parentNode.removeChild(nodesToRemove[i])`
失败不是 removeChild 的问题，而是 **nodesToRemove 里的节点对象 ≠
pE.childNodes[1] 的对象**——oracle 树遍历（nextNode 从 startContainer
出发）产生的 wrapper 与 childNodes 视图的 wrapper 是**不同对象**（wrapper
identity churn，R252「缓存中间态快照」家族）→ identity-based removeChild
miss → oracle 异常/静默 → 中段未删 + 后续 deleteData 未执行（aborted）。

## 三、方法论沉淀

1. dual-iframe probe 模板（src=Range-test-iframe.html + onload 链 +
   restoreIframe 克隆重建 + oracle 内联）——此后 delete/extract/clone 的
   expected 侧归因都可复用（R222-probe 的 onload 链模式 + oracle 抽取）。
2. **expected 侧 ≠ actual 侧的归因顺序**：先 dual-dump PRE（克隆保真），
   再 POST 双侧，最后用「直接调用 oracle 内部步骤」定位哪一步分歧
   （本轮 `removeChild` 直调成功 = identity 而非 API 面）。
3. wpt-data 是 gitignored（fetch 脚本填充）——probe 文件可自由创建删除，
   不污染账本。

## 四、R273 靶点（修复方向已明确）

- **6,x wrapper identity**：oracle 遍历（nextNode：firstChild/childNodes
  链）与 childNodes 视图的 wrapper 必须同一对象（`_zwLocalChildNodes`/
  textEl 缓存/fused view 的单一对象源）——具体是哪个域的 churn 需要
  identity 标记探针（给两处 wrapper 打 tag 后对比）。
- element 端点跨容器（22/48/52/53,x）/ extract 32F / clone 29F。
