# M2 s425 收口账——seltouch 重归因轮（勘根下探修正 s424 单面定性：渲染流
# listbox 行渲染硬前置 + engine 激活臂双面，M3 touch 注入面排除，施工候选
# 除名转挂账）+ 表格池跨流复验（渲染流 R5516 到货）

- 日期: 2026-11-14；基线: HEAD 2957c3e0a（s424 收口，含渲染流 R5516
  d2c3897da 到货），驱动: CONTINUE 指令按既定流程（勘根→施工→验证→落账）
  续下一个施工候选。本轮两段产出：①seltouch（select-listbox-touch.optional）
  施工候选勘根下探 → 重归因（**零源码改动**，施工候选除名转挂账）；
  ②表格池 colspan/rowspan-0 跨流复验（渲染流 R5516 匿名 cell margin 面
  到货，solo 实测核对漂移）。

## 一、seltouch 重归因（勘根下探，静态证据链）

### 勘根链（file:line）

1. 红形（s424 solo）：首触断言 `option one should be selected after
   touching it. expected true got false`——首触选中未发生；二/三触断言
   因此同文件连锁（promise_test 首断言败后续仍在）。
2. 语料形态复核：`<select multiple>` 两 option（optionOne/optionTwo），
   `touch()` = testdriver Actions `addPointer('finger','touch')` +
   `pointerMove(0,0,{origin:target}).pointerDown().pointerUp()`；三断言
   **只查 `option.selected` 状态**（首触选中 / 二触不清他人 / 三触自清除
   ——multiple 面语义）。
3. **M3 touch 注入面排除**（推翻 s64/s386/s424 一脉「touch 注入通道缺席」
   归因）：runner Actions touch 指针是标签透传——`_createSource('pointer',
   name, pointerType||'mouse')`（testharness.rs:13228）→ pointer_down/up
   命令 → `script_pointer_down_sequence`/`script_pointer_up_sequence`；
   part06.js:19710 touch 仅加隐式捕获（`st.pending[pid]` 登记臂），
   composed click 组合**无 pointerType 门**（pointerup→mouseup→click 主键
   照常组合，part06.js:19722 起）——selection 状态断言不依赖
   touchstart/touchend 派发，runner M3 挂账面不构成本案阻塞。
4. **几何硬前置（渲染流面）**：select 内 option 布局盒抑制谓词
   `is_suppressed_select_control_child`（layout-engine/tree.rs:2818）——
   自身为 option/optgroup 且祖先链含 `<select>` 即抑制，**谓词不分
   single/multiple 形态**；消费位 tree.rs:3388/5156（R1679 UA display:none
   + s387 盒面抑制同 gate，kill-switch `ZW_SELECT_SUPPRESS_OPTIONS`）。
   option 无布局盒 → rect 桥快照无键（rect_bridge.rs:121
   `fill_layout_rect_snapshot` 逐有 node_id 盒填表，无盒无键）→ shim
   gBCR 回落零 rect → runner `resolve_pointer_target` best-scan 要求
   `r.width>0&&r.height>0`（testharness.rs:11639 起）跳过 option → 命中
   回落 SELECT/其它元素 → click 派发在 select 而非 option——首触自然不
   选中。hit_test.rs 无 OPTION 引用（option 无专属命中盒，同证）。
5. **engine 第二面**：shim 激活链无 OPTION——`_zwHasOwnActivationBehavior`
   （part03.js:17455）无 OPTION 停靠位（现有 A/AREA-with-href/LABEL/
   DETAILS/SUMMARY/INPUT submit 族/BUTTON submit 族），post-activation 段
   分支链 act:label/act:details/act:button/act:a/act:area（part03.js
   :18470 起）无 act:option——即使几何面修复使命中落到 option，click 亦不
   触发选中。
6. **语义参照在场（host 管线已具正确语义，runner 指针链不经由）**：
   `plan_option`（page-runtime/html_actions.rs:440）——`!multiple &&
   selected → Err(AlreadySelected)`；multiple → toggle 不清他人（
   `clear_others: !state.multiple`）；single → select + clear_others；
   宿主写面 `script_set_option_selected`（js_dom_bridge/script_gen.rs:439）
   以 content attribute 落位——与语料三断言逐条吻合。engine 面施工方案
   已定型在案（OPTION 停靠位 + post-activation act:option 分支 gate
   `_zwUaDispatch===true`——UA 通道限定镜像 act:button 先例，合成
   click()/dispatchEvent 路径不激活 option 与真实浏览器一致；toggle 镜像
   plan_option；input/change 派发 SELECT 镜像 part06 键盘选路
   `__zw_select_key_action` 形），**渲染面到货后即可施工**。

### 判决（修正 s424 归因，施工候选除名转挂账）

- s424 归因汇总把 seltouch 记为「runner M3（seltouch）注入面」——**单面
  定性不完整，本轮修正**：真实阻塞 = **渲染流 listbox 行渲染面（硬前置）
  + engine option 激活臂（第二面）双面**；M3 touch 注入面非本案阻塞。
- 与 s388 select-multiple-resize 判例同位同源：同一抑制谓词不分形态
  （s388 账原文「R1679 UA none+s387 盒面抑制谓词不分 single/multiple
  形态——上游 picker-none 包裹仅 closed 下拉形态，multiple/size>1 真实
  浏览器恒 inline listbox 渲染 option 行；施工需 UA display 按形态分支+
  盒面/IFC 谓词同分支」）——seltouch 与 multiple-resize 共享同一渲染面
  首件；差异 = seltouch 主体是 option 选中（非 resize 拖拽交互），渲染面
  落地后本车道尚需 engine act:option 激活臂第二刀才翻绿。
- **施工候选除名转挂账**：渲染面未落地前 engine 面单独施工零位移（s388
  判例「任一单独施工不能翻绿」），本轮不动源码。挂账协作路径 = 渲染流落
  listbox 形态 carve-out → 本车道续 engine act:option 臂（方案见上 6）。

## 二、表格池跨流复验（渲染流 R5516 到货）

- 渲染流 R5516（匿名 cell margin 抑制＋BFC 隔离＋行高分配 aware floor，
  d2c3897da）落表格 face——在册池表格邻位 solo 复验核对漂移。runner 重建
  自 HEAD 2957c3e0a（test-guard 包裹 build，--per-proc-mem 6 --total-mem
  12 --time-limit 900）。
- solo 实测（ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1，test-guard 包裹，同 s424
  solo 命令面；三文件均 exit 1，红键与 s424 双基线逐键一致）：
  - col-span-limits.html: **2F**（col span of 1000 / 1001 treated as 1000），
    红形逐字同 s424（`assert_equals: expected 53 but got 51.99985`）——
    分数列宽/边框计算面不变。
  - rowspan-0.html: **1F+1P**（clientHeight F `expected a number greater
    than 20 but got 20` / rowSpan P）。
  - rowspan-0-quirks.html: **1F+1P**（同上同形）。
- 判读: **零漂移零翻转**——R5516 匿名 cell margin 面不波及本池三键
  （同表格域不同 face：margin collapsing vs 分数列宽 / rowspan-0
  clientHeight 高度分配）；表格池对渲染流的复验触发面继续维持挂账
  （dialog R-s31 簇 / img density / embed sizing face 到货再核）。

## 口径

- 域 corpus 计数零位移（本轮零源码改动；seltouch 归因修正属账面迁移：
  施工候选→渲染流协作挂账，红键状态不变）。
- 在册红维持 s424 终态 raw 202F+13T（轮1）/ 201F+14T（轮2）口径（簇 D
  闪变与 select-event flake 归一后稳定红面 201F+14T）。
- 用户门维持：GB-20261031（簇 D）、GB-20261114-VIDEO（video P1/P2/P3）、
  s137（form-nameditem）、sizes-auto F5（渲染流移交）。

## 下一步

- 渲染流 listbox 形态 carve-out 到货后：本车道施工 engine act:option 激活
  臂（方案已定型，见一段 6）→ solo seltouch + the-select-element 目录回归
  → 全域 sweep ×2 join 对 s424 双基线（/tmp/s424/pairs-{1,2}.txt，
  extract3.sh 提取约定）。
- 其余候选维持挂账待触发：dialog R-s31 簇 14 行 / img density s416 /
  embed intrinsic sizing ×2（渲染流对应 face 到货复验）；GB-20261114-VIDEO
  决策后 video P1+P2；簇 D 决策后 A2 分诊续。
