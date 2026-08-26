# R281 Evidence — cloneContents 路径克隆分支（clone 158P→180P +22）

**日期**: 2026-08-26
**切片**: M4——R281(a) cloneContents 跨容器分支（R280 模式移植）
**改动面**: `part06.js`（cloneContents R281 段：路径克隆组树 + 同节点 CD 切片 + doctype 抛 + 空切片守卫）+ `part23.rs`（+1 单测）
**commit**: `b06e9d8a1`

## 一、重聚类（29F 分布）

| 簇 | 形态 | 归因 |
|---|---|---|
| 20-24/28/30/48/49/50/52/53,x | 跨容器族 | `_coveredChildren` sc≠ec null → toString 文本回落 |
| 27/35/36/37/39,x | 同节点 comment/PI | 同上（文本回落，frag 期望 Comment 克隆切片） |
| 25/26,x | `[document,0,document,1/2]` | assert_throws（doctype contained 须抛 HRE） |
| 54/55,x | collapsed foreignDoc/xmlDoc | 域克隆问题（[object Object] vs __n） |
| Range.detach() | 基础 | 预存 |

## 二、实现（R281 分支，cloneContents 内）

R280 路径克隆组树的**纯 clone 版**（无 move、无删源）：

1. **跨容器**：frag = [firstPartial.clone(sc 侧路径层 + 尾段文本切片/
   尾部子区间), contained 中段 deep clone, lastPartial.clone(ec 侧)]；
   element-sc/doc-sc 尾部用 R279 的 ecPathIdx 规则。
2. **同节点 CD 切片**：clone + substringData [so,eo)——27/35-39,x 的
   comment/PI 簇（旧版 toString 文本回落）。
3. **doctype contained 抛 HRE**（spec 步骤）——25/26,x。
4. **空切片守卫**：collapsed 返空 frag——**首版教训**：无守卫时空 #text
   克隆使 0/4/8/56-59,x collapsed 文本族 +8 翻红（spec 返回空 frag）。

## 三、验证（A/B vs R280 基线，全 ranges sweep）

| 项 | R280 | R281 | Δ |
|---|---|---|---|
| Range-cloneContents | 158P/29F | **180P/7F** | +22P |
| Range-deleteContents | 125P/0F | 125P/0F | 持平（100%） |
| Range-insertNode / surround | 1840P/0F | 同 | 持平（100%） |
| Range-extractContents | 168P/19F | 同 | 持平 |
| ranges 全量 | 37809P | **37831P** | +22，set-diff 0 新 fail / 22 消失 |
| engine 单测 | 2414 | **2415** | +1（r281 四形态单测） |
| fmt / clippy | 干净 | 干净 | — |

**残余 7F**（下轮靶点）：29/31/51,x document 容器 frag 域簇 +
53,x sc-el 递归 + 54/55,x collapsed foreign/xml 域克隆 + Range.detach()
预存。

## 四、教训

- **collapsed 空切片 = 空 frag**：clone 的 same-node 分支须零宽守卫
  （空 #text 克隆节点是非法形态——真浏览器返回无子 frag）。
- **R280 模式的移植成本极低**：路径克隆组树骨架（cac 定位 + 双侧
  clone 栈 + 爬升同层挂载）跨 clone/extract 复用，只换「内容操作」
  （clone vs move+prune）——验证了 delete 侧十三轮分支模式作为
  「模板库」的杠杆。

## 五、R282 靶点

- **(a) clone 残余 7F 的域簇归因**：29/31/51,x doc 容器 frag（doc 的
  cloneNode 域）+ 54/55,x collapsed foreign/xml。
- **(b) extract 残余 19F**：25/26/51,x doc-doctype throws（extract 侧
  对称缺口——R281b 的 doctype 抛只接了 clone）+ 29/31,x comment 克隆域 +
  48/53,x element-sc 递归组树。
- **(c) deleteContents ShadowRoot 一例**。
