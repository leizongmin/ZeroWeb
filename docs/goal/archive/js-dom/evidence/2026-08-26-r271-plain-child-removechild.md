# R271 Evidence — removeChild plain 子 registry 剔除放宽（引擎修复，WPT 净 0）

**日期**: 2026-08-26
**切片**: M4——R271(a) 6,x CDATA 族的引擎侧修复（WPT 残余归因 expected 侧）
**改动面**: part04 removeChild 的 R195 分支容器限定放宽 + part23.rs（+1 单测）

## 一、定位与修复

**6,x** `[paras[5].firstChild,2,paras[5].lastChild,4]` probe 链：
1. 直接 deleteContents 探针：POST = [CDATA:12, CDATA:5678, text:'']——
   **中段 CDATA 未移除**（首尾 deleteData 都对）。
2. 直接 removeChild(mid CDATA) 探针：`no-h`（无 handle）+ post=3 +
   stillIn——**静默穿透**。
3. 根因：part04 removeChild 的 R195 plain-子分支限定
   `_isContainerHandle(handle)`（容器父）——paras[5] 是普通元素父，
   createCDATASection 的轻量包装 append 后在 `_handleChildren` 但非容器
   父形态 → 分支 miss。

**修**：容器限定放宽——`_isContainerHandle || child 在 _handleChildren[handle]`
（普通元素父承载 plain 子的通用形态）。

**验证 probe**：POST = [CDATA:12, text:''] + 塌缩 (p5,1)——6,x 算法
（R268 四段）完全正确。

## 二、WPT 净 0 的归因（expected 侧）

deleteContents 仍 113P/16F——6,x 两 subtest 仍 Fail（assert_unreached:
DOMs not equal）。actual 侧算法已正确（probe 实证）→ 分歧在 **expected
侧**：expected iframe 经 restoreIframe 的克隆树（referenceDoc.docEl.
cloneNode 重建）——克隆域的 CDATA/文本子形态与主域可能仍有差异
（R221 fresh-doc 系列的克隆保真域）。R272 靶点：expected 侧克隆树的
CDATA 保真 dump 对比。

## 三、验证（vs R270 基线）

| 项 | R270 | R271 | Δ |
|---|---|---|---|
| Range-deleteContents | 113P/16F | 113P/16F | 持平（6,x 归因 expected 侧） |
| Range-mutations-removeChild | 20P/0F | 20P/0F | 持平（100%） |
| Range-mutations-insertBefore | 76P/0F | 76P/0F | 持平（100%） |
| Range-insertNode / surround / extract / clone | 100%/基线 | 同 | 全持平 |
| engine 单测 | 2409 | **2410** | +1（r271 单测）全绿 |
| fmt / clippy | 干净 | 干净 | — |

**land 依据**：引擎 removeChild 语义 bug 真实（probe 三轮实证——普通
元素父的 plain 子移除静默穿透），为 6,x 及后续形态扫清 actual 侧；WPT
净 0 但 expected 侧归因明确记档。

## 四、R272 靶点

- **(a) 6,x expected 侧**：restoreIframe 克隆树的 CDATA 子保真 dump
  对比（克隆域 vs 主域形态差异）。
- **(b) element 端点跨容器**（22/48/52/53,x）：方向分支 contained 递归。
- **(c) 28,x 深形态 + 49/50,x cursor-only**。
- **(d) extractContents 32F / cloneContents 29F 重聚类**。
