# R260 Evidence — CharacterData 变更的 live-range 边界调整（mutations 三套件 100%，全量 +196P）

**日期**: 2026-08-26
**切片**: M4——R260(a) 套件重聚类 → Range-mutations-insertData 的
「Wrong end offset」簇 → spec replace-data 末段实施
**改动面**: part03（`_zwAdjustRangesForData` + plain 域接线 + globalThis 导出）、
part04（proxy 域接线）、part05（parsed sel 域接线）、part06（注册表 + textEl
域接线）+ part23.rs（r231 期望更新 + r260 回归单测）
**commit**: 9f40ba074

## 一、诊断链（重聚类 → 算法缺口）

1. **重聚类**（R256–R259 四轮行为面变化后）：insertNode/surround 均 100%
   保持；extractContents 51F 中的「startOffset and endOffset must always be
   the same after extractContents()」13F 簇 + Range-mutations-insertData 的
   122F「Wrong end offset」簇。
2. **harness PRE/POST 探针**：sim 侧同节点 Text 区间 extract 后 range 从
   (2→8) **折叠到 (2→2)**——机制 = 真浏览器 deleteData 的 spec
   `concept-node-replace-data` 末段「for each live range whose boundary
   point is in node」边界调整；shim 的 deleteData 无此机制使双侧恒不折叠。
3. **spec 公式**（WPT Range-mutations.js 引用原文）：① off ≤ offset 不动；
   ② offset < off ≤ offset+count → **收到 offset**（首版误用
   offset+insertLen，Range-mutations-insertData 实测纠正）；③
   off > offset+count → + insertLen − count。

## 二、实施（四域接线 + 跨域身份匹配）

- **注册表**：`_makeRange` 每 Range 登记入 `globalThis.__zwLiveRanges`
  （环形 8192——长测试序列旧 range 不可达，调整 dead range 无副作用）。
- **`_zwAdjustRangesForData`**（part03）：遍历注册表按三键匹配逻辑同一
  文本节点——① identity（plain/factory 域对象稳定）；② `__zwHandle`
  字符串（proxy 域单次 get trap 产物 identity 不稳）；③ `__zwIsText` +
  父 sel/handle + childIndex（harness paras 域——probe R260ID 实证
  pSel=N 需父 handle 键）。写 `_startOffsetBase/_endOffsetBase`
  （offset accessor 恒读 _base 槽）。
- **四域接线**：part03 `_zwAttachCharacterDataMethods`（factory/detached）、
  part04 proxy trap（handle）、part05 parsed sel 域 + appendData=
  insertData(len)、part06 `_zwRegisterTextEl`（textContent= 建的 textEl）。
- **过程坑**：part06 闭包对 part03 函数声明的直接引用不可靠（probe
  entry=0 实证调用未达）——全站改经 `globalThis.__zwAdjustRangesForData`
  路由后生效（entry=40/hitEc=40）。

## 三、验证（vs R259 基线）

| 套件 | R259 | R260 | Δ |
|---|---|---|---|
| Range-mutations-insertData | 260P/122F | **382P/0F** | +122（100%） |
| Range-mutations-deleteData | — | **564P/0F** | 100% |
| Range-mutations-appendData | — | **384P/0F** | 100% |
| Range-extractContents | 141P/51F | 160P/32F | +19 |
| Range-surroundContents | 1840P/0F | 1840P/0F | 持平（100%） |
| Range-insertNode | 1841P/0F | 1841P/0F | 持平（100%） |
| ranges 全量 set-diff | — | — | **+196 F2P / 0 P2F** |
| engine 单测 | 2399 | 2400 | r231 期望更新 + r260 回归单测全绿 |
| fmt / clippy | — | 干净 | — |

（r231 期望从 2,8 更新为 2,2——旧断言记录的是无调整机制的观测而非 spec
行为。）

## 四、R261 靶点（本轮遗留）

- **Range-mutations-replaceData 超时**（871 用例 90s 超时）——adjust 循环
  在超长套件的性能/疑似死循环排查（可能是 `_regWrite` → MO notify →
  observer 回调再变异的环路）。
- **splitText/remove 的边界调整未接线**（spec split text 的 boundary
  retarget 族——Range-mutations-splitText 96P/20F / remove 2P/18F）。
- extractContents 残余 32F / cloneContents 29F 重聚类。
