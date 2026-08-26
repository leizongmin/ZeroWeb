# R282 Evidence — extract doctype 抛移植 + doc-sc 父守卫修正（extract 168P→178P）

**日期**: 2026-08-26
**切片**: M4——R282(a) extract 25/26,x doctype 抛 + (b) 29/31,x doc-sc 守卫
**改动面**: `part06.js`（extractContents R282 doctype 抛 IIFE + R280 父守卫 doc 豁免）+ `part23.rs`（+1 单测）
**commit**: `fdf9d23e6`

## 一、25/26,x：doctype 抛对称移植

R281b 的 cloneContents doctype 抛移植到 extractContents（spec
`concept-range-extract` 同款步骤）：同容器 doc 查 [so,eo) 子区间、跨容器
doc-sc 查 [so, ecPathIdx) 尾段——nodeType 10 命中即抛
HierarchyRequestError。

## 二、29/31,x：R280 doc-sc 尾段是死代码（probe 定位）

sandbox 单测复现 29,x 形态（`implementation.createHTMLDocument` + [dt,
html, comment]）：frag=0、树未动但 **collapse 却发生**——⓪ 段的 debug
标记 `unset` 实证**从未执行**。根因：R280 的 `!sc.parentNode` 守卫对
Document 恒真（doc 无父是**合法形态**非 detached 信号）——doc sc 在
cac 计算前就被拒。修：sc 非 doc 时才要求有父（`scParOk`）。

修后 sandbox 断言：`post=2[html,#comment("mmenter tail")] frag=2[HTML,
#comment("Co")] col=(fdoc,1)`——HTML 本体 move 入 frag + comment 头段
切片克隆 + 源削头 + 塌缩 (doc, so)，与 oracle 语义一致。

## 三、验证（A/B vs R281 基线，全 ranges sweep）

| 项 | R281 | R282 | Δ |
|---|---|---|---|
| Range-extractContents | 168P/19F | **178P/9F** | +10P（25/26/29/31 全解） |
| Range-deleteContents / insertNode / surround | 125P / 1840P / 1840P 全 0F | 同 | 持平（100%） |
| Range-cloneContents | 180P/7F | 同 | 持平 |
| ranges 全量 | 37831P | **37841P** | +10，set-diff 0 新 fail / 10 消失 |
| engine 单测 | 2415 | **2416** | +1（r282 doc-sc 单测） |
| fmt / clippy | 干净 | 干净 | — |

**残余 9F**：48/51/53,x element-sc 递归组树簇（extract 三条 + cursor 三
条 + frag 三条）——spec 的 firstPartial.clone + subfrag 递归结构，下轮。

## 四、教训

- **「无父」对 doc 不是 detached 信号**：跨容器分支的父守卫须按
  nodeType 分流——Document 的 parentNode 恒 null 是形态本身。probe
  「collapse 发生了但内容段没跑」= 守卫在内容段之前拦截的经典签名。
- **clone 侧的 spec 步骤移植到 extract 是低成本对称件**（R281b →
  R282 一小时内 +4P）——两侧 spec 同构时先查「对侧已有什么」。

## 五、R283 靶点

- **(a) element-sc 递归组树**（extract 48/51/53,x + clone 53,x）：
  firstPartial.clone + 子区间递归提取/克隆——R280 ①' 的 sc 侧已对
  CD-sc 递归组树，element-sc 需要「sc 尾段子 [so, ecPathIdx) 中 partially
  contained 的孙层递归」。
- **(b) clone 残余域簇**：54/55,x collapsed foreign/xml 域克隆 +
  Range.detach() 预存。
- **(c) deleteContents ShadowRoot 一例**。
