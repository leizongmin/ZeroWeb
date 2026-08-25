# R263 Evidence — insert 侧 live-range 边界调整（mutations-appendChild/replaceChild 双 100%，+58P）

**日期**: 2026-08-26
**切片**: M4——R263 `__zwAdjustRangesForInsert` + appendChild/replaceChild 全域接线
**改动面**: part03（`_zwAdjustRangesForInsert` helper + doc 级 replaceChild 两段
+ Node.prototype 通用 replaceChild 两段）+ part04（appendChild 移动路径两段 +
replaceChild 三路径两段 + replace-with-self 短路调整 + 无 wire 域兜底）+
part23.rs（+1 回归单测）

## 一、spec 语义（WPT modifyForInsert 逐字引用）

> "For each boundary point whose node is the new parent of the affected
> node and whose offset is greater than the new index of the affected
> node, add one to the boundary point's offset."

**调用时机契约 = 插入后**（newParent/newIndex 读插入后形态——与 R262 的
移除前快照对偶）。**移动语义**（appendChild/insertBefore/replaceChild）=
remove 调整（R262）+ insert 调整（R263）两段串联：
- WPT testAppendChild = modifyForRemove + modifyForInsert
- WPT testReplaceChild = modifyForRemove(old) + modifyForRemove(new)
  + modifyForInsert(new)

## 二、实现与接线（八站点）

`_zwAdjustRangesForInsert(inserted)`（part03 挂 globalThis）：读
`inserted.parentNode` + `childNodes.indexOf`，父三键匹配（identity/handle/
sel——与 R262 sameParent 同款），offset > newIndex 则 +1（_base 槽）。

| 域 | 站点 | 段 |
|---|---|---|
| part04 appendChild | `_r51OldLink` 移动路径 | remove 段（registry 剔除前） |
| part04 appendChild | ceAdded 循环后 | insert 段（fragment 逐子） |
| part04 replaceChild | handle-handle 主路径 | remove(old)+remove(new) + insert 段 |
| part04 replaceChild | handle-new + sel-old 路径 | 同上两段 |
| part04 replaceChild | sel-sel 路径 | 同上两段（sel 域跨父近似） |
| part04 replaceChild | **replace-with-self 短路** | 树不动但边界照 WPT 序迁移（remove+insert 净效应） |
| part04 replaceChild | 无 wire 域兜底（text oldChild） | 两段兜底 |
| part03 doc 级 replaceChild（`_makeDetachedDocument` own 方法） | remove(old) 在 splice 前 + insert 段在 splice 后 |
| part03 Node.prototype 通用 replaceChild | 同上两段（兜 _zwMEl/plain 域） |

## 三、诊断方法论（probe HTML 注入 wpt-data 定点复现）

foreignDoc.replaceChild 残余 8F 的定位链（四轮 probe，每轮删临时文件）：
1. **对照探针**：同形 removeChild（part03:7728 own 方法，R262 已挂）单独
   Pass → 排除 remove 侧。
2. **直接调用探针**：页面内 `globalThis.__zwAdjustRangesForRemove(dt)` 直调
   Pass → 排除调整函数自身。
3. **源码指纹探针**：`fd.removeChild.toString()` 确认 R262 在场；
   `Object.getOwnPropertyDescriptor(fd,'replaceChild')` 发现 **own 方法**——
   `_makeDetachedDocument` 的 doc 字面量在 7840 行有自己的 replaceChild
   （此前 grep 范围 6690-7700 漏掉），未走 Node.prototype 通用路径。
4. 挂上两段后 60P/0F。

**教训**：detached doc 域的方法解析须逐方法验证 own-vs-prototype——
`_makeDetachedDocument` 的字面量方法面与 Node.prototype 通用面是**两套并行
实现**，通用路径的修复不会自动到达 own 方法。

## 四、验证（vs R262 基线）

| 项 | R262 | R263 | Δ |
|---|---|---|---|
| Range-mutations-appendChild | 42P/28F | **70P/0F** | **+28（100%）** |
| Range-mutations-replaceChild | 30P/30F | **60P/0F** | **+30（100%）** |
| Range-mutations-removeChild | 20P/0F | 20P/0F | 持平（100%） |
| Range-mutations-splitText | 116P/0F | 116P/0F | 持平（100%） |
| Range-surroundContents | 1840P/0F | 1840P/0F | 持平（100%） |
| Range-insertNode | 1841P/0F | 1841P/0F | 持平（100%） |
| Range-deleteContents | 80P/49F | 80P/49F | 持平（预存） |
| Range-extractContents | 160P/32F | 160P/32F | 持平（预存） |
| Range-cloneContents | 162P/29F | 162P/29F | 持平（预存） |
| Range-mutations-insertBefore | 超时 | 超时 | 持平（累积型慢预存） |
| engine 单测 | 2402 | **2403** | +1（r263 回归单测）全绿 |
| fmt / clippy（workspace） | 干净 | 干净 | — |

**mutations 域状态**：非超时套件全部 100%（removeChild 20/appendData 384/
appendChild 70/replaceChild 60/deleteData 564/insertData 382/splitText 116/
dataChange 部分）；超时族（insertBefore 15/replaceData 437/dataChange 426+
——累积型慢，R261(a) 归因）为独立性能深项。

## 五、R264 靶点

- **insertBefore 超时族**（15 用例 90s）：part04 insertBefore 路径的 remove+
  insert 两段接线（同 appendChild 模式——先接线验证正确性，超时可能是
  adjust 循环常数 × 巨型测试表的累积——若接线后仍超时则按 R261(a) 归因
  为性能深项）。
- extractContents 残余 32F / cloneContents 29F / deleteContents 49F 重聚类。
- replaceData/dataChange 超时（累积型慢，低 ROI）。
