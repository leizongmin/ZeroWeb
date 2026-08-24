# R233 Evidence — sim 全源注入复刻（HRE 簇复刻一致性实证，负结果轮）

**日期**: 2026-08-25
**切片**: M4——R233(a) mySurroundContents 全源注入探针（HRE 37 + INVALID_STATE 30 + startOffset 11）
**改动面**: 无 land 代码（探针已清理，工作树零 diff）

## 一、方法论执行

R231 方法论复用成功：注入 case-local `mySurroundContents`（Range-surroundContents.html 自带副本）+ common.js 依赖链（myExtractContents/myInsertNode/ensurePreInsertionValidity/isPartiallyContained/isAncestorContainer/nextNode/nextNodeDescendants/getDomExceptionName 等 13 函数）到沙箱，对 24,x 复刻形态（factory div + 3 p + `[td,2,p2.firstChild,1]` + Text newParent）直接执行。

## 二、关键实证：复刻形态上 sim 与 host 一致

| 引擎 | 结果 |
|---|---|
| sim（注入源） | `INVALID_STATE_ERR`（partial check 命中 p2） |
| host | `InvalidStateError`（R210 partial check 同命中） |

**两引擎在复刻形态上行为一致**——但 WPT 24,x 期望 HIERARCHY_REQUEST_ERR。
推断：真实页面的分歧不在 surround 逻辑层，而在 **sim partial-check 的
nextNode 遍历爬出 testDiv 后进入 body 的兄弟（含 harness iframe）再上到
合成 docEl**——爬升链上的 cDP 调用（R219 关闭态缺失）使 sim 的 partial 检查
中途 TypeError 被 catch，走不到 INVALID_STATE 分支，后续 insertNode validity
才产出 HRE。即 **HRE 37F 簇与 cDP 108F 簇同根**（都绑定在 R219 开关 +
fresh-doc 深项上）——这解释了为何 R232 的 cDP-only 实验同时影响两簇。

复刻差异点：本地复刻的 div 未挂在 body 下（爬升链短），两引擎都在 partial
检查命中；真实页面 testDiv 在 body 内且 body 有 harness iframe 兄弟。

## 三、结论与 R234 靶点

- HRE 37F / INVALID_STATE 30F / cDP 108F **三簇同根**：R219 开关（cDP 方法面）
  × fresh-doc 残余（跨轮树形态分歧）。单独解任何一簇都会被另外两簇钳制
  （R227/R232 两次 -28P 实测）。
- **R234 首选靶点转向 fresh-doc 深项本体**：restoreIframe 残留形态 dump
  （每轮 doc 首末子清理循环后 head 内残留是什么）——这是解锁 ~250F
  （assert_unreached 133 + cDP 108 + HRE 37）的钥匙。次选：startOffset 11F
  （16,x harness-iframe index 算术，独立小簇）。
- 深项清单不变。

## 四、commit

无代码 land（探针已清理，工作树零 diff；engine 2380 全绿确认）。
