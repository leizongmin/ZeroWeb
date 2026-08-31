# R284 Evidence — doc 容器 covered children + frag 域归一（extract 186P / clone 184P）

**日期**: 2026-08-26
**切片**: M4——R284(a) 51,x 同节点 doc 域 + (b) 53,x fragment 域
**改动面**: `part06.js`（`_coveredChildren` 纳入 nodeType 9 + extract/clone frag 归 ownerDocument 域）+ `part23.rs`（+1 单测）
**commit**: `51818851f`

## 一、51,x：`_coveredChildren` 的 doc 容器拒绝

R194 的容器白名单（nodeType 1/11/tagName）漏了 **Document（9）**——
同容器 doc 区间的 contained 子语义与元素一致，旧版返 null 使
extract/delete/clone 三侧的同容器 doc 形态**全空转**。修：白名单加 9
（sandbox 断言：`[idoc,1,idoc,2]` → html 子 move 入 frag + docKids 1 +
塌缩 (doc,1)）。

## 二、53,x frag：frag 恒主 document 域

extract/clone 的 frag 恒 `globalThis.document.createDocumentFragment()`
——跨域 append 对 iframe 域子被 flat。修：frag 归 **start 节点的
ownerDocument**（spec `dom-range-extract-contents` 步骤 1；
myExtractContents 同款）。probe 实证修后 frag 持 P#e 包裹（nodeType 1）。

## 三、53,x 残余：comparator walk 域（非 frag 构造）

真实文件注入 probe：`fragKids=3[P#e:1, P:1, #comment:8] od=other`——
**引擎 frag 结构正确**（P#e 包裹在位）。断言仍失败于 isEqualNode 的
first-differing walk（读 actual 侧 P#e 槽位为裸 Text）——遍历域问题
（R276 comparator 家族），下一个独立切片。

## 四、验证（A/B vs R283 基线，全 ranges sweep）

| 项 | R283 | R284 | Δ |
|---|---|---|---|
| Range-extractContents | 183P/4F | **186P/1F** | +3（51,x 全解） |
| Range-cloneContents | 180P/7F | **184P/3F** | +4（25/26/51 + 域连带的 54/55 之一） |
| Range-deleteContents / insertNode / surround | 125P / 1840P / 1840P 全 0F | 同 | 持平（100%） |
| ranges 全量 | 37846P | **37853P** | +7，set-diff 0 新 fail |
| engine 单测 | 2417 | **2418** | +1（r284 doc 容器单测） |
| fmt / clippy | 干净 | 干净 | — |

**残余**：extract 53,x（1）+ clone 29/31/53,x（3）——全部「Returned
fragment」comparator walk 域簇 + Range.detach() 预存。

## 五、教训

- **frag 的 ownerDocument 是 spec 步骤 1 而非实现细节**：域错位的 frag
  在 append 时静默 flat（不抛错）——「期望包裹 got 裸 text」先查 frag
  的创建域，再查 append 的 move 语义。
- **probe 先行断言归因**：53,x 投入修复前先 probe frag 实际结构（P#e
  已包裹）——避免在正确的 frag 构造上继续「修复」，把归因让给真正的
  comparator walk 域。

## 六、R285 靶点

- **(a) comparator walk 域**（extract 53,x + clone 29/31/53,x 共 4F）：
  isEqualNode/nextNode 遍历对跨域 frag 的读取——R276 家族的独立切片。
- **(b) Range.detach() 预存 1F**。
- **(c) deleteContents ShadowRoot 一例**。
