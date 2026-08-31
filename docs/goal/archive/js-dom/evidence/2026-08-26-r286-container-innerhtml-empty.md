# R286 Evidence — 容器 handle innerHTML 空态（ShadowRoot deleteContents 100%）

**日期**: 2026-08-26
**切片**: M4——R286(c) deleteContents ShadowRoot + (b) detach 连带
**改动面**: `part04.js`（innerHTML 容器空态）+ `part23.rs`（+1 单测）
**commit**: `475205df8`

## 一、ShadowRoot 一例：innerHTML 空态

sandbox 复现：`{<span>ABC</span>}` 全删后 `childNodes=0` **但 innerHTML
仍 "<span>ABC</span>"**。根因：shadow/fragment handle 的内容只存 **JS
registry**（host 无对应 mutation 域），innerHTML getter 的融合视图空时
回落 `__zw_get_inner_html_handle` 的**缓存序列化**——删除后的空态被旧
缓存掩盖。修：容器 handle 以 registry 为唯一事实源，空 registry → ''
（`_isContainerHandle(handle)` 分支）。

修后：Range-deleteContents-in-ShadowRoot **4P/0F（100%）**。

## 二、Range.detach() 连带翻绿

R281 时代记录的 cloneContents「Range.detach()」1F 本轮实测 **Pass**
（该用例断言 detached range 的 cloneContents 返空 frag——先前失败
推测为容器 innerHTML 空态的同族掩盖）。

## 三、验证（A/B vs R285 基线，全 ranges sweep）

| 项 | R285 | R286 | Δ |
|---|---|---|---|
| Range-deleteContents-in-ShadowRoot | 3P/1F | **4P/0F（100%）** | +1 |
| Range-cloneContents | 185P/2F | 185P/2F | 持平（detach 翻绿计入基线复测） |
| 其余 ranges 套件 | delete 125P/extract 187P/insert 1840P/surround 1840P 全 0F | 同 | 持平（100%） |
| ranges 全量 | 37855P | **37857P** | +2，set-diff 0 新 fail |
| engine 单测 | 2418 | **2419** | +1（r286 shadow 全删单测） |
| fmt / clippy | 干净 | 干净 | — |

**ranges 域现状**：五套件 100%（delete / delete-ShadowRoot / insert /
surround / extract）+ clone 185P/2F（29/31,x docEl clone 的 handle vs
plain 域——最后的 2F）。

## 四、R287 靶点

- **(a) clone 29/31,x**（docEl clone 域：expected plain [object Object]
  vs got handle __n——cloneContents 对 doc sc 的 docElement 深克隆走
  host handle 域，oracle 走 _zwDeepCloneEl plain 域）。
- **(b) mutations 超时族**（环境慢，低 ROI 备档）+ dom 全量 sweep 的
  nodes/events 域复核（本轮只跑了 ranges-）。
