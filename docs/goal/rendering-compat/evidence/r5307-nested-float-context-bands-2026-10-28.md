# R5307 — 同 BFC 上下文嵌套 float 带集透传 + §9.5.1 墙模型——position-absolute-dynamic-relayout-004 ✗1.04→✓0.00 翻绿；6 案 ZW-vs-ZW 镜像假阳性定谳，4 案挂账化（float 根 margin 折叠）

**日期**: 2026-10-28
**驱动案**: `css/css-position/position-absolute-dynamic-relayout-004.html`（✗1.04%）
**corpus**: 见「交付」（wall model + 探针删除后全量复跑定版）。

## 定性（R5306 下一步①846F 择轻命中）

驱动案家族（position-absolute-dynamic-relayout-001..006）red 主因 = 同 BFC 上下文内
**非 BFC 兄弟块内的嵌套 float 互相叠压**：float1 在早序 wrapper 块内、float2 在后继
兄弟块内，float2 落位时看不到 float1 → 从 y=0 落位全叠（chromium 须落到 float1 底
下方）。三层缺陷叠加：

1. **透传**：`float_geometries` 只收直接 float 子，嵌套在非 BFC wrapper 内的
   float 带不透传给后继兄弟子树（`nested_float_bottoms` 只透传底边 y，无几何带）。
2. **消费**：inherited 带集此前不参与 earliest-fit 落位试探（`placed_bands` 从空
   Vec 起步，只记录本 walk 已放置 float）——即使透传了也放不进旁位。
3. **文档序**（实现期发现）：R1619 `all_floats` 无序透传若直接升级为带集约束，
   后序直接 float 的带会约束**前序**子树内 float（驱动结构 wrapper 内 float1 被
   后序 float2 带顶到 y=50 实证）——违 R5160 文档顺序原则。

## 实现（float_positioning.rs 五处，无新 kill-switch）

1. **`nested_float_geoms`**（新 helper，与 `nested_float_bottoms` 同帧约定与 BFC
   边界）：递归收集非 BFC 后代 float 几何（`FloatGeom` 7 元组，x/y 外层 content-box
   相对，y 为 border-box 顶、高含 margin_bottom）；BFC 后代独立上下文停止递归。
   注：`FloatGeom` 无 margin_top，带顶以 border-box 顶近似（mt>0 时带略窄；x 区间
   margin-box 精确）。
2. **`placed_bands` 带集 seeding**：以 inherited float 带初始化（本容器 border-box
   帧 → content 帧，减 `content_y_offset`），earliest-fit 试探从「只见本 walk 已放
   置」扩为「见同上下文全部前序带」。
3. **文档序过滤透传**：递归循环内 `child_inherited` = inherited ∪ 文档序在前的直接
   float 带（`float_child_indices[i] < idx` 过滤）∪ 前序兄弟嵌套 float 带
   （`nested_geoms_acc`，递归返回后收集恒文档序在前）。累加器边递归边收集
   （`nested_left_acc/right_acc` 底边 + 带集），只并入**后继**兄弟。
4. **墙模型**（earliest-fit 块内，§9.5.1 规则 1/2/3）：带可来自 inherited 透传
   （含外层 containing block 外的 float，x 区间可负/超容器），区间墙不再默认钳
   容器宽——左系 float 左墙锚容器左缘（规则 1）、右墙仅当存在相交左系带（规则 2
   「another left-floating box to its left」）时取容器右缘，否则无墙、仅受相交右系
   带左缘约束（规则 3，可溢出容器右缘）；右系 float 镜像。旧行为（双墙恒容器宽）
   下 floats-rule3/7-outside-* 家族被误判放不下而误下移。
5. **孤儿清理**：R1619 `all_floats` 死绑定删除（唯一消费者被文档序过滤链替代）+
   失实注释更新。

kill-switch 沿用既有三开关：`ZW_NESTED_BFC_FLOAT_AVOID`（透传总门）、
`ZW_FLOAT_EARLIEST_FIT`（带试探域）、`ZW_FLOAT_CLAMP_CONTENT_REL`（帧约定）。

## 取证关键发现：ZW-vs-ZW reftest 镜像假阳性（方法论）

ZW runner 用 ZW 引擎渲染 test 与 ref 两侧。此前 floats-rule3/7-outside-* 家族 6 案
pass 是**两侧镜像 bug 的巧合相等**：test 页 float 该下移不下移（墙模型误钳）+
ref 页 float 根内块子 margin-top 该保留被丢弃（§8.3.1 违例，见下）——两侧各错各的
恰好渲染相同。修复 test 侧后 PNG bbox 逐案比对：test 页全部 Chromium-correct
（bbox x 与 ref 一致、y=308 正确下移），残余 mismatch 全部来自 ref 页的 float 根
margin 折叠 bug。教训：**修好一侧后两侧镜像页翻红 ≠ 回归，须逐侧 bbox 定位残差来源**。

## ref 侧阻塞根因：float BFC 根不与子折叠 margin（挂账定谳）

floats-rule*-ref 页结构 = root float 500×500 → 块子 `margin-top:300`。CSS2 §8.3.1：
float 建立 BFC，**BFC 根不与子折叠 margin** → 子应渲染在内容系 (ml, mt)。布局探针
实证（`probe_ref_margin_in_float_root`，引擎级）：ZW 渲染 child pos=(100,0)——mt 字段
完好（300）但位置为 0，html 高 500 无 300 痕迹 → margin 消失于 taffy 折叠链
（html→body→float-root→child 链折叠值折进链顶后，R5210 重解析门只对 float 子/clear
子开启，普通块子的重解析无人接）。修复属 margin-collapse 雷区
（R1316-R1319/R3808/R5109/R5210 谱系），**不在本轮**，为下轮目标；修复后 4 案
（rule3-outside-left-002、rule3-outside-right-002、rule7-outside-left-001、
rule7-outside-right-001）预期免改 test 侧直接翻绿。

**接受 +4 翻绿论证**（对比回退方案）：回退保 846F 假绿 = 7 页（驱动 + 6 rule）
继续渲染错误且驱动案修复丢失；fix-forward 下 7 页 test 侧全部 Chromium-correct，
4 案红是已定谳的独立 ref 侧 bug（与 R5304 002.tentative 同类「基线 pass 系两侧
事故」记账逻辑），且下轮修复即可回收。失败集 diff 逐案对账见「交付」。

## 交付

- 驱动案 **✗1.04→✓0.00**（单案 FILTER 真通过-可信）；家族
  position-absolute-dynamic-relayout-001..006 **6/6 PASS**（改前 4 败）。
- 墙模型按预测精确回收 2 案：floats-rule3-outside-left-001、
  floats-rule3-outside-right-001 ✗→✓（425 宽目标 float 不下移贴边/越缘，
  PNG bbox 与 ref 一致）。
- 全量 corpus（wall model + 探针删除 + 旗标门后终版，两次独立复跑）：**16592 /
  15743P / 849F**；两次失败集逐案一致；归一化 diff vs R5306 基线（846F）恰 =
  {−position-absolute-dynamic-relayout-004, +floats-rule3-outside-left-002,
  +floats-rule3-outside-right-002, +floats-rule7-outside-left-001,
  +floats-rule7-outside-right-001}（4 案 ref 侧挂账暴露，test 侧均已
  Chromium-correct），零意外翻转。
- 回归测试 +5（`r5307_nested_float_context_bands.rs`，引擎级 LayoutEngine API）：
  ①驱动结构 f1@(0,0)/f2@(0,50) ②flat 形态现状钉 f1@(0,0)/f2@(0,0)（挂账，防意外
  回归）③rule3 425 宽无左邻带溢出容器 (0,0) ④rule3 475 宽右系带阻塞下移 (0,300)
  ⑤rule7 425 宽相交左系带下移 (0,300)。探针页 ×8 与临时探针测试已删（corpus
  Total 回 16592）。

## 门禁

### bench O(n·depth) 收集器开销与旗标预扫门（实现迭代记录）

首版实现把嵌套 float 收集器挂在递归循环内无条件运行（每子下探其全子树找
float）——纯块级文档付出 O(n·depth) 重遍历，深度敏感基准（block_layout_1000、
deep_nesting、incremental_layout、page/medium layout）存在结构性回退风险。bench-gate
首轮 8 指标超预算（block_layout_1000 4.70ms vs 预算 4.35ms 等），但当窗 loadavg
6→15 剧烈摆动（并行工作区两条外部 wpt-runner 流 100%+ CPU），clean 树 stash A/B
复跑被守卫判 INCONCLUSIVE（loadavg 15 尖峰）——无法归因。**处置不赌空闲窗**：加
`collect_float_subtree_flags` 入口单趟 O(n) 预扫（HashMap<NodeId, bool> 子树含
float 旗标），收集器按旗标 O(1) 跳过无 float 子树——**跳过与收集逐值等价**（收集器
在无 float 子树本就返回空），行为保持由构造保证 + 全套测试/corpus 失败集逐案复核。
无 id 匿名盒查不到旗标保守回退为收集；`table_float_fix` 等子树级入口按各自子树预扫。
clippy 按惯例 `#[allow(too_many_arguments)]`（本文件 1387 先例）+ `is_none_or`。
总开销收敛为入口一次 O(n)，float 存在时收集本身 O(floats 所在子树)。


- layout-engine 全套 **2194P/0F**（lib 2185 + r4543 4 + 新 5；旗标门后复跑同值）。
- fmt 0 diff；clippy zero-layout-engine `--all-targets -D warnings` 0 告警（新增
  测试 struct-update 初始化过 field_reassign_with_default；孤儿 all_floats 清理）。
- product-smoke PASS（welcome oracle **15.50% 持平** + struct ×6 全 PASS）。
- **bench-gate 定向 zero-layout-engine**：首版收集器实现 8 指标超预算（环境
  loadavg 6→15 摆动叠加 O(n·depth) 结构风险，clean 树 A/B 被 INCONCLUSIVE，
  见上节）→ 旗标门后同量级负载下 **8→1**（block_layout_1000/wide_tree/
  incremental/deep_nesting/page-medium 全部回预算内，结构性回退消除）；残余
  flex_layout_1000 +3% 超预算（4.089 vs 3.970ms），量级分析 = 旗标预扫至多
  ~50µs（无 float 树单趟 O(n)）vs 差距 1.1ms，20 倍缺口归并行走外部流
  （112% CPU testharness 进程持续在跑）——按 R5305/R5306 判例等空闲窗复跑定版。
- make test 全量未跑（改动面 float_positioning 递归/落位域 + 同 crate 测试，消费面
  corpus 两次全量 + product-smoke + lib 覆盖，R5301-5306 同判例）。

### 交接核验（rollover 会话，提交前对当前未提交树全门复跑）

代码 mtime（06:30）晚于本档（04:52），提交前对当前树全量复验：fmt 0 diff；
clippy zero-layout-engine `--all-targets -D warnings` 0 告警（clippy 缓存即当前码态）；
layout-engine 全套 2200P/0F（2185 lib + 10 定向，含 r5307 ×5）；product-smoke PASS
（welcome **15.50% 持平** + struct 全 PASS + form-perf PASS）；全量 corpus **16592/
15742P/850F**——失败集 = 本轮定版 849F ∪ {size-adjust.tentative}，后者单案隔离 2×
复跑同值 ✗6.47%（R5303 flake 签名逐字同），且 release 二进制（06:32 构建，md5
c3e87ab3）与两次 849F 复跑所用二进制全同——同二进制不同轮次结果分歧 = 运行间 flake
非代码回归，R5303 判例记账维持。bench-gate 空闲窗复跑仍挂账（对端流连轴 sweep，
当窗 loadavg ~10，静窗短期不可得）。

## 挂账

- **float BFC 根 margin 折叠（§8.3.1）——下轮目标**：root float 内块子 mt 被 taffy
  折叠链吞噬（引擎级探针结构已录本档；修复点在 margin-collapse 链重解析域，R5210
  谱系），修复后 rule3/7-outside-{left,right}-002/001 余 4 案翻绿 → 预期 845F。
- **同 walk 直接兄弟 float 叠压（flat 形态）**：line_y=0 时 Phase 1 无 taffy 期带集
  可收集，float2 与 float1 同位叠压（现状钉测试在案）；需 Phase 1 taffy 期 float 带
  收集（结构轮）。
- **earliest-fit kill-switch 回退行模型无 inherited 带集**（FIXME 已注源码）：回退
  路径下嵌套兄弟块内 float 仍叠压；默认配置 earliest-fit 开，不影响 corpus。
- R5306 及既往挂账全维持：taffy vendored clippy 104 处、TP 重采样结构局限、R5007
  AA fringe、002.tentative、types.rs 注释、R5299 三项（bidi 裂解/backdrop 合成/度量
  缓存）、013 ellipsis host、vertical-rl 域、flake 账本。
- 本轮书面否决：无（featureless 家族 ×4 与 drop-shadow-currentcolor-dynamic-003
  否决记录见前轮，维持）。
- float_positioning.rs 3813+ 行超 2000 行规则（既有状态，本轮 +166 行未拆分，
  报告在案）。

## 下一步 R5308

①float BFC 根 margin 折叠修复（§8.3.1，R5210 谱系；回收 rule3/7 家族 4 案）②849F
账本续扫择轻 ③013 ellipsis host 传播评估（结构轮，视账本轻案供给）。
