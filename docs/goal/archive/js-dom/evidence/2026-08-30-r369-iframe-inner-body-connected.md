# R369 — iframe 子文档 body 的 handle 子记账（Node-isConnected "Test with iframes" 收口）

**日期**: 2026-08-30
**切片**: 已知 Fail 集合 10→9——`Node-isConnected` 整文件转绿（R368 下一步 (a) 的
直接领取；R360 转档的「iframe 专项三层链路」域，实际缺口比预判窄）
**改动面**: `js_dom_shim/part03.js`（detached-doc body 串行合并分支记账 +
removeChild 登记剔除）+ `part04.js`（isConnected innerBody 包含判定 + remove
trap 委托）+ part24 单测

## 1. 根因（探针实证）

WPT 用例链：`frames[0].contentDocument.body.appendChild(frames[1])` 后
`frames[1].isConnected` 期望 **true**（spec `dom-node-isconnected`：connected =
shadow-including root 是 Document——iframe 子文档本身即 Document，不要求是主
文档）。探针实证三层缺口：

1. iframe 工厂 doc 的 body（part03 `_makeDetachedDocument` 的 body 视图）其
   `appendChild` 对 handle 子（iframe 元素是 handle proxy）走 **R112 串行合并
   分支**（序列化并入查询源）→ early-return **不设 `c.parentNode` 也不写
   `_zwNodeParent` 反链** → 子的 parentNode 恒 null；
2. `isConnected`（part04 proxy get trap）沿反链爬升无记录 → 回落
   `__zw_getBoundingClientRect` 布局探测 → false；
3. `frames[3].remove()` 后 nodes[3] 期望**断开**——host 侧
   `__zw_remove_handle` 只作用于主文档，innerBody 视图不反映。

**范围勘误**：R360 时代定性「三层独立链路缺失」为深结构专项；实际缺口收敛为
「串行合并分支的父链记账」一个点（R368 已建 `_zwSerialKids` 登记数组形态，
本轮补记账 + 消费面）。

## 2. 修复三件

1. **记账**（part03 串行合并分支）：early-return 前重接 `c.parentNode = body` +
   `body._zwSerialKids` 登记数组 push + `_zwNodeParent[handle]` 落
   `{ parentSel: null, parentHandle: null, innerBody: body, plainParent: body }`
   ——`plainParent` 槽复用 R180 的 `_parentNodeFor` plain 父直返路径
   （parentNode getter 立即正确；R256 兄弟导航同款消费）。
2. **isConnected 包含判定**（part04 爬升循环首站）：遇 `innerBody` 记录时做
   容器身份包含判定——`_zwSerialKids` 含本节点（handle 串匹配）⇒ root 是
   该 body 所属 Document ⇒ connected；remove 后数组失含 ⇒ false（无额外清除
   钩子，包含判定天然反映当前挂载态）。**过程勘误**：trap 形参是 `_t` 非
   `target`（首版 identity 比较抛 ReferenceError 被 catch 吞 → 恒 false；
   单测 dbg 探针当场抓回）——改纯 handle 串匹配。
3. **remove 委托**（part04 remove trap）：`_zwNodeParent[handle].innerBody`
   命中时委托 `innerBody.removeChild(ceSelf)`（part03 body.removeChild 补
   登记数组 splice + 断父链 + 反链清理分支）——`frames[3].remove()` 后
   nodes[3] 断开、nodes[4]（同 body 内 plain div）保持 connected。

## 3. 验证（landing 门）

| 门 | 结果 |
|----|------|
| 目标件 | `Node-isConnected` 4/4 subtest 全 Pass（含 iframes 变体）——整文件自导入以来首次全绿 |
| 全量 dom sweep（polyfill，TIME_LIMIT=2400） | **55493P/11F/15T——Fail 文件集合 10→9（Node-isConnected 转绿）零新增**；Timeout 集合为并发噪声轮转族（`remove-from-shadow-host-and-adopt-into-iframe-ref` 既存[clean HEAD 同 Timeout]、`insertBefore-iframe-crash` 本轮翻转 Timeout→Pass[串行合并重接父链解堵 pending 链]） |
| 哨兵套件 | MO 族 134P/4F 恒等、Event-dispatch 221P/2T、Node-parentNode 5P/1T、iframe 族 4P/1T、Range-surroundContents 1840P |
| engine 单测 | v8 **2498**（+1：`test_iframe_inner_body_connected_chain_r369`——记账/isConnected=true/removeChild=false/remove()=false 四态断言）/ quickjs 1473 全绿 |
| webview / integration | 657P + 1F（`navigator_controller_tracks` SW 流既存 flake，clean HEAD 同败，run-rules §10）；integration 784P |
| clippy / fmt | v8 + quickjs 双矩阵 `-D warnings` 零警告 / 无 diff |

**过程教训**：① Proxy trap 形参名不统一（`_t`/`t`/`target` 混用）——跨 trap
搬代码时 identity 比较引用形参名，先核对本 trap 签名（ReferenceError 被
catch 吞掉后表现为静默 false，单测 dbg 探针是唯一可靠定位手段）；② 负结果
定性要随基建演进复核——R360 的「深结构专项」定性在 R368 盖章基建落地后
收窄为单点记账。

## 4. 后续

- 已知 Fail 集合余 **9**（全部深结构/基建域定性）：MutationObserver-document
  3F（parse-time 基建）、cross-realm 1F（工厂 body 可观察 id）、
  remove-and-adopt-thcrash（window.open 无 popup 通道）、
  querySelector-mixed-case / remove-next-sibling（R220 identity 双源域）、
  events 2F（onerror 跨 realm / Chromium 专有 pseudo）、ranges
  dataChange/replaceData 2F（文件级 Timeout 尾批）。
- 主线剩余：M5/M7 default-on（待用户点名，改 Mission 级单向门）；M3 已达成；
  M4 基线持续维护；M2 已收口；M8/DC-8 已收敛。
