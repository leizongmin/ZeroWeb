# s424b：selector 臂结构性放行实验——双门实验定谳与回退

日期：2026-11-14。承接 s424a 挂账（evidence
2026-11-14-s424a-class-gate-widening-experiment.md §3「清偿需 selector 臂
结构性放行」假设）。**本刀全程未提交**：engine 已回 HEAD 零残留，A/B 工件
存 `.acceptance/wptfyi-css/results-s424b-structural/`。

## 1. 刀口

假设：s423 selector rect 臂撤 family 谓词（对齐 s421 handle 臂无门全重放
语义），anchor-name-mutation 15F 的结构性队列（template clone Append +
class）即可进视图重放翻绿。

实施：callbacks.rs s423 臂 gate 改 `style_version > applied` 单窗口判
（9+/12−，已回退）。

## 2. 受控 A/B（同 peer g38 提交态基座，对照 = s424a 轮 postrevert 跑）

基座：peer g38 五文件 19:02:37 内容 = 已提交 b0e5ac2da（本轮期间 peer 落
远端）；两轮二进制同基座受控可比（s424b 二进制 19:19:04，基座 mtime 全程
未动）。

**实验组（结构性放行）**：265 页 **10,996P/1,857F/4T**——对对照
（12,061P/792F）净 **−1,065**；对 s424a class 拓宽面（11,199P）再 −203
= 结构性额外暴露。

逐案归因（vs postrevert）：

- **−864** writing-modes-001 + −1 金丝雀：class 拓宽已量面（窗口判开臂
  无族过滤，全数复现）；
- **−203 结构性新暴露**：try-tactic-position-area **131** 案 +
  position-area-in-position-try **64** 案 + position-area-value 7 +
  position-try-tree-scoped 1 + zz-g6probe-mut6 1——正是 s423 收窄注释
  在册「position-try fallback 状态机 ~131F」池的实测全形（快照陈旧读
  伪过被视图真读暴露，新失败形 "expected 95 but got 200"）；
- **+4**：dynamic-001/004 + fallback-invalidation（class 面同回）+
  zz-g6probe-mut5 1；
- **anchor-name-mutation 纹丝不动 15F（假设证伪）**：结构性放行后门开、
  视图槽重放照跑。失败形态逐案对账（results JSON 双侧消息）：基座 15 案
  全为 `got 0`（快照陈旧读）；实验组 15 案全数变值 `expected X got X+1`
  （0→1 / 100→101 / 200→201 三轴恒 +1），且部分案断言链前移（基座
  `expected 100 got 0` 的前置断言实验组已过、败在后段 mutation 断言）——
  **变异重放已通到布局层、活值读已生效，残余缺口 = 重布局几何/读链恒
  +1px**（舍入口径或参照盒基准，归 layout-engine/view 读链域；「视图重放
  物化链上游未达」假设被同批 JSON 反证——读值若仍陈旧不应随门变值，
  挂账更正）。

## 3. 回退决策

净 −1,065 且目标面零交付；双门实验（class 族 −862 / 结构性 −1,065）
定谳：s423 收窄的家族门是伪过池（writing-mode 865 + try-tactic 203+）的
门面依赖，暴露面与 153,000 指标方向冲突；结构性放行亦不服务目标页。
engine `git checkout` 回 HEAD；s423 收窄约定维持。

## 4. 挂账更新

- anchor-name-mutation 15F 清偿前提更正：门与重放链已通（读值随门变值
  实证），残账 = **重布局几何/读链恒 +1px**（三轴 15 案无一例外）——独立
  挂账，probe 首选：同页无变异对照读数是否亦 +1（分离「布局几何漂移」与
  「读链口径」）；
- try-tactic/position-try 伪过池实测 203+（131+64+7+1）：清偿 =
  position-try fallback 状态机建模（master 在册挂账域）落地后同批转真；
- 双门补丁形态存档：s424a `.acceptance/wptfyi-css/s424a-class-gate-
  widening.patch`；s424b 形（gate 单窗口判）见本 evidence §1 描述 +
  results-s424b-structural 工件。

## 5. 双 session 写账竞态收敛

本实验两路 session 各写一份 evidence（本文件先手 19:24 入库；并行件
`…-gate-release.md` 补丁形态推断有误、后删）。逐案归因表（下 1069 案分页
明细）与断言链前移证据取自并行件分析（diff-pages.py results-s424a-
postrevert vs results-s424b-structural）：DOWN = writing-modes-001 864 +
金丝雀 1 + try-tactic-position-area 131 + position-area-in-position-try 64
+ position-area-value 7 + position-try-tree-scoped 1 + zz-g6probe-mut6 1；
UP = dynamic-001/004 + fallback-invalidation + zz-g6probe-mut5（§2 已并）。

## 6. +1px 定位（分析定谳：offsetParent walk 对 runtime 元素失明，修复缓议）

### 6.1 判别面（s424b results JSON 逐案消息 + 页面结构）

- 15 案失败消息（三定位法 explicit anchor()/implicit anchor()/
  position-area **逐字相同**）：expected 0/100/100/100/200 → got
  1/101/101/101/201——恒 +1 且跨定位法一致 → 非 anchor() 解析专有。
- 页面结构（anchor-name-mutation.html）：基座 body 仅空 `<main id="main">`，
  **全部元素（含 bordered CB）经 `<template>.content.cloneNode(true)` +
  append 运行时创建**——CB `.containing-block{position:relative;
  border:1px solid black}` 亦 runtime 产物（handle 链身份，无 id）。
- 真浏览器口径（CSSOM View offsetTop/offsetLeft，
  https://drafts.csswg.org/cssom-view/#dom-htmlelement-offsettop）：元素
  border edge − **offsetParent padding edge**——期望 0/100/200 = CB padding
  原点口径（静态位 padding 原点 delta=1、减 CB border 1 → 0；anchor 底缘
  delta=101 → 100）。

### 6.2 机制定谳：offsetTop 基准回落 body（+1 = CB border）

offsetTop 全链（shim part05 `_zwOffsetOffset`/`_zwOffsetParentSel` +
part04 trap）跨**三层视图文档**，runtime handle 链元素在其中两层半失明：

1. **`__zw_parent` 走 query view doc**（callbacks.rs:2163
   `with_query_view_doc`）——该 doc 只重放 R57 结构级子集，「属性级/handle
   链/Remove/SetInnerHtml 不应用」（callbacks.rs:205-215 R57 文档注）；
   live doc 为应用前基座——runtime 元素三层全缺席 → `__zw_parent` miss
   → shim walk `if (!p) break`（part05.js:16202-16210）→ **offsetParent
   恒回落 `'body'`**；
2. 元素自身 rect 经 pending view slot（全量 applier 重放，handle 链可见，
   s424b 门开臂）→ t.y = 8(body margin) + 1(CB border) + X 正确；
3. `_zwOffsetOffset` 以 body rect（y=8）为减数、body border 0 为减项 →
   offsetTop = (8+1+X) − 8 = **X+1**：默认 X=0 → got 1 ✓、anchor1 X=100 →
   got 101 ✓、anchor2 X=200 → got 201 ✓——三轴逐字吻合。

判别史（三分解）：(b) shim opBorder 丢失——**判死**（要求 CB 是
offsetParent，walk 根本到不了 CB；shim part05.js:16231 本已正确减
offsetParent border）；(a) 视图 rect 链 border 双计——待探针复核
（`probe_s424d_bordered_cb_view_rect_basis` 已置 part115 待跑，见 §6.3；
若 rect delta 干净 1/101 即终证 (a) 证伪）；(c) offsetParent walk 失明
——全链代码落地 + 数值逐字吻合，定谳。

副证：基座态（门闭）同批 JSON 恒 `got 0`——runtime 元素 rect 快照缺席 →
`_zwLayoutRectTrap` null → `if (!r) return 0`，与本模型一致无矛盾；
try-tactic 池 "expected 95 got 200" 属另一缺口（fallback 状态机未建模）。

### 6.3 修复位（缓议）与探针状态

修复方向：`__zw_parent` 补 handle 身份通道（`__n` 前缀 → pending view
slot doc 内 `parent_selector_for_doc` 等价解析，镜像 rect 回调双臂形），
落 `callbacks.rs`（本流域文件）；shim walk 无需动。修复后钉测：bordered
CB + template clone 常驻单测（无门直读）+ anchor-name-mutation 定向探针
（需 s424b 门态 patch 配合复用）双证。

缓议理由：① `crates/css-parser` 五文件 peer WIP（CalcExpr::Resolution
新变体 11 处 match 未覆盖）当前工作树**不编译**——engine 探针/修复构建
全被阻断；② shim part01/part01f peer WIP 在途（共享树构建互相扫入）。
判别探针 `probe_s424d_bordered_cb_view_rect_basis`（part115 临时件，跑后
即撤）已就位待 peer 构建窗解除后补跑，终证 (a) 证伪。
