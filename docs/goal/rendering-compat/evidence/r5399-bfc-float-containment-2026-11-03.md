# R5399：BFC root 终末 float 包含补扩 + unhoist 兄弟间距保留（2026-11-03）

案面：`css/CSS2/floats-clear/floats-145.xht`（✗2.06%，floats-clear 族余红）。
「Bottom margins on floats」：float 蓝 div（Ahem 30/90）内唯一子 float 白 span
（mb 1em）；assert 蓝条可见。TEST 页**全无蓝**；REF 页（img 330×30 mt:90）蓝带
y[124,153)。

## 归因（临时几何探针 + 阶段 dump，用后即删）

- **蓝带量化**（PNG 解码）：REF 蓝带 330×30 @ [124,153)；TEST 零蓝像素——float 父
  div 整体缺失而非位移。
- **探针 145 主形态**：`outer float div w=330 h=0`（shrink-to-fit 宽正确含 float 子，
  高度 0）；inner float span h=90。
- **阶段时间线**（S5/FIN 树 dump + BFCGROW 臂内打点）：S0（taffy 原生）inner h=0
  （R3787 在案：float 叶子内容高欠计、子树未终化）；step 5 内 §10.6.7 臂
  （`recompute_bfc_auto_height`，R4069 fold 语义）跑时 inner 未测量 → 被空块子臂
  折叠 mb → outer ch=0；FIN 时 inner h=90 物化（step 6+ inline 终化）但**无人重访
  BFC 祖先**——step 5 重算是唯一 §10.6.7 承担臂且时序早于 float 高物化。
- **探针 145 REF 形态**（第一轮修复后 145 仍 ✗2.20% 引出）：ZW 渲染 div y=18
  （=p_bb+0），chromium oracle（真实 ref 页 img 几何 + 截图双证）div y=50
  （=p_bb+16）、img 内容顶 140；ZW test 页 float 蓝 140 已与 oracle 同位——剩余
  16px = REF 页 p.mb 在兄弟间距中丢失。
- **S0 定谳 16px 链**：taffy 把 img（UA display InlineBlock 近似，spec 值 inline，
  style-system/src/lib.rs 在案「族级两面裁决留待后续任务同规翻转」）当块级子 →
  mt 90 塌穿 div（S0 div.mt=90，div@108=max(p.mb 16, 90)）；R4140
  `unhoist_inline_first_atom_mt` 把容器 y 全额上提 over_collapse（108→18）修正
  IFC 双计，但 taffy 塌进链的**前驱块 mb 份额**（16）随之上提丢弃（gap 0）。
  R4140 驱动案（body 首子）无前驱块兄弟，缺口从未暴露。

## 修复（两个单点）

1. **`bfc_float_containment_late`**（engine/postprocess.rs，engine.rs step 终末
   `synthesize_inline_wrapper_boxes_final` 后调用）：后序遍历，flow-root/float 的
   auto 高容器（豁免面镜像 step 5 重算：abspos/fixed、replaced、flex item、
   contain:size、multicol）**grow-only** max 补扩 content_height 至 float 子
   margin-box 底边。只增不缩：非 BFC 排除收缩归 step 5.2 既有域（谓词互补不重叠）。
   kill-switch `ZW_BFC_FLOAT_CONTAIN_LATE=0`（default-on）。
2. **unhoist 兄弟间距保留**（float_positioning.rs）：walk 签名加
   `prev_sibling_mb`（父帧子循环单趟跟踪最近流内前驱块 mb，float/abspos 脱流不破坏
   §8.3.1 邻接故跳过回溯；根传 0）；unhoist 上提量改 `over_collapse − prev_gap`，
   `prev_gap = clamp(prev.mb − 声明 mt, 0, over_collapse)`（恰为被 taffy 折进链又
   被 unhoist 丢弃的前驱 mb 份）。`adjust_no_float_page` 同参传递。kill-switch
   `ZW_UNHOIST_SIBLING_GAP=0`（default-on）。

## 常驻测试

`engine/tests/r5399_bfc_float_containment_tests.rs` 5 针：145 主形态（outer
h=120 load-bearing）、145 REF 形态（div.y=p_bb+16 load-bearing）、R4140 原语义
守卫（body 首子原子 mt32 → y=8 不变）、非 BFC auto 高 float 排除守卫（§10.5.1
不反转）、overflow:hidden BFC 含 float 守卫（step 5 既有臂与终末补扩幂等）。

## 验证

- 新针 5/5 绿；layout-engine 全腿 2378+14 P/0F（含 5 新针）。
- kill-switch 关断敏感性外科级：`ZW_BFC_FLOAT_CONTAIN_LATE=0` 案面精确回 ✗2.06%
  （蓝条缺失态）、单针精确转红 `test_float_parent_auto_height_contains_float_child`
  （overflow:hidden 守卫保持绿——step 5 臂独立承担）；`ZW_UNHOIST_SIBLING_GAP=0`
  案面精确回 ✗2.20%（ref 页 16px 偏移态）、单针精确转红 sibling gap 针；双关
  ✗2.06%；双开 ✓0.00%。
- 案面：floats-145 ✗2.06% → **✓0.00%（真通过-可信）**。
- corpus 全量 A/B（双侧重跑二进制，基线 = 当前 HEAD 9a98b47ab）：基线 731F →
  修复 **730F**。FIXED={floats-145 2.06%→0.00%}；全量窗口 mover 候选 9 案
  （middle-003 修/tao-149,150,127,128,129,151 等翻红）**双侧各 3×3 隔离全
  ✓0.00%**——全为全量窗口并发抖动（R5395/R5396/R5398 在册同族），真态
  **REGRESSED=∅**。MOVED>0.4%：∅。
- 门禁：fmt ✓ / guarded-clippy 全工作区 ✓ / 本地集 **725/725 全绿** / make test
  全腿零新增失败（失败全为在册对端账：js_dom_bridge CSSOM 5 案〔652a78e00 域〕+
  browser default_actions、gpu_compositor_path〔R5377/R5378 在册 flaky〕+
  image_batch_e2e + mutation_arm）/ product-smoke 6 腿 PASS / bench-gate 定向
  zero-layout-engine **GATE PASS 25 指标（NEW=0）**（首查 loadavg 10.6 对端流
  负载，静窗 2.76 复跑）。

## 挂账 / 下一步

- **floats-031（3.89%）深结构在案**：float 落入当前行后行内内容不避让 float 带
  （7 个既有 kill-switch 全阴性——float-exclusions 重测既有臂均不承担）；oracle
  几何在案（chromium：float 落行 2 顶/左，行内容避让至 float 右缘 x=119、img
  x=256；ZW：文本 x=23 被 float 覆盖、img x≈168）。「源序后置 float 推挤当前行
  重排」是 IFC 行模型结构能力（R5213 float 摆放模型挂账同宗），深结构挂账。
- **img/replaced 族 UA display InlineBlock→inline 翻转**：源注释在案（style-system
  src/lib.rs「族级两面裁决留待后续任务同规翻转」）；R4140 unhoist 兄弟间距保留为
  该翻转前的补偿级修复，翻转落地后本针 REF 形态语义应复核（img inline 后 taffy
  不再塌穿，unhoist 五重门不触发，行为保持）。
- R5397 深结构挂账（negative-clearance 2.55% / margin-collapse-104 4.17%）、
  floats-clear-multicol 7 案特征缺口（R5394）、R5242/R5392/R5391/R5390/R5389
  挂账全维持。
- 对端账维持：js_dom_bridge CSSOM 5 案、image_batch_e2e、mutation_arm、
  default_actions、gpu_compositor_path_dispatches（R5377/R5378 在册 flaky）。
