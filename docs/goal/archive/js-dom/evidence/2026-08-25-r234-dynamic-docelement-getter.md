# R234 Evidence — fresh-doc 深项：动态 documentElement getter（cDP 108F 簇正面修复）

**日期**: 2026-08-25
**切片**: M4——R234(a) fresh-doc 深项本体（restoreIframe 残留形态 dump → 根因 → 修复）
**改动面**: `part05.js`（动态 documentElement getter）+ `part06.js`（extract 两种记账）+ `part23.rs`（r234 单测）
**commit**: `3bafaec06`

## 一、探针 dump（R234-probe.html，已清理）

在 restoreIframe 前后 + 清理循环内三站点 dump doc/head/body 形态（920 轮 × 4 消息）：

- **doc 级跨轮零累积**：round 1 起 920 轮 PRE/POST 形态完全一致
  （`doc=[#10html[0],#1HTML[2]] head{#1TITLE} body{#1DIV}`）——
  **R233 的「跨轮残留累积」假设在 doc 级被证伪**（唯一异常是 round 0
  的 factory 初始态：body 含 fixture 原文 TITLE/LINK/META/SCRIPT×2）。
- distinct POST (head,body) 形态数 = **1**（x920 全同）。

## 二、根因（cDP 108F 簇）

`_zwMakeIframeDoc`（part05）的 `documentElement` 是**固定闭包 getter**
（`get: function () { return docEl; }`）。restoreIframe 语义下：

1. 清理循环摘除 factory docEl（R216 入树的那个）；
2. `appendChild(referenceDoc.documentElement.cloneNode(true))` 挂入克隆；
3. **getter 仍返回已脱离的空壳 factory docEl**——无 cDP/contains。

rows 12–14,x `[documentElement, 0, …]` 的 `range.startContainer` =
该空壳 → sim `isAncestorContainer(node, docEl)` 调
`docEl.compareDocumentPosition` → TypeError → harness 记
`node2.compareDocumentPosition is not a function`（12/13/14,x 全列 +
17,x foreignDoc.docEl + 30,x foreignDoc.body 跨容器族 = 108F）。
克隆子树本身经 `_zwDeepCloneEl` → `_zwMEl`（part03:5576 有 cDP
own-property）方法面全配——只是 getter 从不返回它。

## 三、修复

1. **part05 动态 getter**：`documentElement` = doc.childNodes 首个元素子，
   回落 factory docEl（spec `dom-document-documentelement`「首个元素子」；
   与 `_makeDetachedDocument` 的 R130 惰性 getter 同语义）。restoreIframe
   后读到克隆（方法面全配）。
2. **part06 extractContents 记账两件**（动态 getter 后该形态从空转
   no-op 变真实提取，暴露两处 host 分歧）：
   - plain 子（无 handle）clone+remove 摘除后 `parentNode` 记到
     fragment（spec contained children 是 move 语义；旧版原件变无根
     游离树 → harness「different number of pieces expected 1 got 2」）；
   - 跨容器路径（`_coveredChildren` null）`collapse(true)`（harness
     「startContainer and endContainer must always be the same」断言）。
3. **part23 r234 单测**：dyn/cdp/same/rooted 四断言。

## 四、验证链（vs R233 基线，全量 ranges 目录逐条 diff）

| 项 | R233 | R234 | Δ |
|---|---|---|---|
| Range-surroundContents | 1385P/455F | **1421P/419F** | **+36，0 新失败**（12/13/14,x 全解锁） |
| Range-insertNode | 1841P/0F | 1841P/0F | 0（100% 保持） |
| Range-extractContents | 119P | 118P | −1（49,x vacuous 翻转，见下） |
| Range-deleteContents | 68P | 67P | −1（同上） |
| Range-cloneContents | 156P | 155P | −1（同上） |
| **ranges 目录全量** | **39571P** | **39640P** | **净 +36P（set-diff：fixed 39 / new 3）** |

- 3 个 new failures（extract/delete/clone 的 49,x
  `[documentElement,1,body,0]` 单 subtest）：旧基线经**空壳 docEl 空转
  no-op** 凑合通过（两侧都无操作→DOM 相等）；现在 host 真实执行，
  fragment 深比较仍分歧——**行为暴露非能力回归**，其 DOM + cursor
  主体 subtest 已随塌缩修复转 Pass。
- **native 路径同值**：ZW_NATIVE_DOM=1 surround 1421P 逐计数一致。
- **engine 单测**：**2381 全绿**（+r234）；fmt/clippy 零警告；
  make test 唯一 1F 为既知 XOpenDisplayFailed 环境项（run-rules §10，
  历轮同形态）。
- 探针文件已清理（wpt-data gitignored，零仓库 diff）。

## 五、三簇同根假设修正 + R235 靶点

- **R233 归因修正**：cDP 簇的根因不是「R219 开关 × fresh-doc 残余」
  的耦合，而是**固定闭包 getter 的 stale 指针**——本修复后 108F 簇
  中的 rows 12–14,x（69F）已解；17,x/30,x（foreignDoc 侧 39F）待同
  款诊断（foreignDoc = createHTMLDocument 路径，documentElement 走
  R130 惰性 getter——理论已动态，需探针确认 17,x 的 nodeB 形态）。
- HRE 37 / INVALID_STATE 30 簇未动（本轮未涉及）——**下轮重聚类**
  确认三簇独立性是否成立。
- assert_unreached 133 簇：本轮 dump 证明 doc 级无累积，该簇根因
  需在**克隆子树内部**（head 内节点级残留）继续定位。

R235 首选：17,x/30,x foreignDoc cDP 残余 39F（同根水到渠成）；
次选：455→419F 后的 surround 重聚类。
