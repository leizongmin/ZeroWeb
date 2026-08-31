# R368 — 无身份 plain 元素 append 的 host 身份盖章（adopted 工厂子树的文档级查询可见性）

**日期**: 2026-08-30
**切片**: 已知 Fail 集合 11→10——`node-realm-adoption-after-frame-removal` 整文件转绿
（R367 下一步 (a) 的直接领取）
**改动面**: `js_dom_shim/part04.js`（appendChild trap 盖章分支）+ `part03.js`
（detached-doc body 串行合并 early-return 的 adopt 传播补口）+ part24 单测

## 1. 根因（探针链实证）

WPT test 3 与 test 1/2 的唯一差异是**访问路径**：test 1/2 经容器自身
`querySelector` 访问（本地 childNodes walk），test 3 经主文档
`document.getElementById` 访问。探针序列（临时 fixture，跑后即删）：

1. 工厂 plain 容器 adopt 后：`container.querySelector('p')=hit`、
   `document.querySelector(#id)=null`、`gebi=null`、`getEBTN('p')=0`、
   `document.querySelectorAll('p')=0` —— **文档级查询面整体 miss**；
2. `container.__zwSelector=null, __zwHandle=null`——iframe 工厂 createElement
   产物是纯 plain 对象，host 快照与 pending 索引（按 sel/handle 寻址）均无从
   命中；
3. 脚本侧手工盖 `__zwHandle = __zw_create_element('div')` 后 re-append →
   `gebi=hit`（落入 part04 handle-child 分支的 `__zw_append_child(sel,handle)`
   wire + R51c `_zwHCCollectSubtree` childNodes 展开进同步索引）——**根因
   确证 = 缺身份，而非 creation-realm 印记缺失**（R366/R367 的原假设不成立：
   realm 原型链在创建时已正确挂接，`instanceof InnerParagraph` 天然为真）。

## 2. 修复三件

1. **盖章分支**（part04 appendChild trap，R177 之前）：无身份（无 sel 无
   handle）元素子 append 到 sel/handle 父时分配 host 句柄并盖 `__zwHandle`
   章 + R368 adopt 传播（子树 ownerDocument 重指主 document，configurable
   getter——后续重 adopt 可覆盖；**不动原型链**——spec `concept-node-adopt`
   只重指 node document，与 creation realm 正交）。
2. **容器 gate**：目标是 shadow/fragment 容器 handle（`_isContainerHandle`）
   时不盖章——容器子树是本地 registry 语义（R97 fragment 展平/R195 plain 子
   剔除）。**过程回归**：首版无此 gate，全量 sweep 中 Range-*
   {clone,extract}Contents-in-ShadowRoot 3+3F（span 被盖章后改道 handle wire，
   其文本子注册错位使 extractContents 序列化丢文本 `<span>ABC</span>` →
   `<span></span>`）；stash 对照确认为本轮引入后加 gate 收口。
3. **串行合并 adopt 补口**（part03 detached-doc body.appendChild 的 R112
   串行合并分支）：early-return 此前跳过下方 R191 的 adopt 子树传播——盖章
   使容器落入 handle 分支，node-realm-mixed "moved into realm B's document"
   随之回归（B 段 re-adopt 后 ownerDocument 停留主文档）；按 R327 测绘结论
   （本分支确证 iframe doc body 域）补同构传播。

## 3. 验证（landing 门）

| 门 | 结果 |
|----|------|
| 目标件 | `node-realm-adoption-after-frame-removal` 3 subtest 全 Pass（polyfill + native 双路径；native node-realm 族 16P 恒等） |
| 全量 dom sweep（polyfill，TIME_LIMIT=2400） | **55493P/12F/14T——Fail 文件集合 11→10（frame-removal 转绿、node-realm-mixed 收口）零新增**（Timeout 15→14 为并发噪声轮转族） |
| 哨兵套件 | MO 族 134P/4F 恒等、ParentNode-querySelector 2055P、Element-matches 675P、Node-appendChild 11P、Node-removeChild 28P、Range-insertNode 1841P、Document-getElementById 18P、in-ShadowRoot 12P |
| engine 单测 | v8 **2497**（+1：`test_identity_stamp_plain_child_adoption_r368`——盖章/主文档重指/子树传播/sel 子不盖章四态断言）/ quickjs 1473 全绿 |
| webview / integration | 657P + 1F（`navigator_controller_tracks` SW 流既存 flake，clean HEAD 同败，run-rules §10）；integration 784P |
| clippy / fmt | v8 + quickjs 双矩阵 `-D warnings` 零警告 / 无 diff |

**过程教训**（R143 教训第九次 + 两处当场抓回）：单测 JS 串内 `//` 注释被
Rust 续行 `\` 吞掉致 SyntaxError（改注释前置到 Rust 侧）；首版容器缺 gate
与串行合并 adopt 缺口均被全量 sweep / 目标族复核当场暴露，stash 对照归因
后同轮收口——全量 sweep 是盖章类跨域改动的必要门。

## 4. 后续

- 已知 Fail 集合余 **10**（全部深结构/基建域定性）：Node-isConnected iframe
  专项、MutationObserver-document 3F（parse-time）、cross-realm 1F（工厂
  body 可观察 id）、remove-and-adopt-thcrash（window.open 无 popup 通道）、
  querySelector-mixed-case / remove-next-sibling（R220 identity 双源域）、
  events 2F（onerror 跨 realm / Chromium 专有 pseudo）、ranges
  dataChange/replaceData 2F（文件级 Timeout 尾批）。
- 主线剩余：M5/M7 default-on（待用户点名，改 Mission 级单向门）；M3 已达成；
  M4 基线持续维护；M2 已收口；M8/DC-8 已收敛（202P/0F/3 NotRun）。
