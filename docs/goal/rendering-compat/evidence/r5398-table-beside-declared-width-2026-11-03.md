# R5398：table-among-floats beside 填 gap 臂 declared-width 分流（2026-11-03）

案面：`css/CSS2/floats-clear/clear-applies-to-014.xht`（✗7.36%，floats-clear 族最大单案红）。

案面 assert「The 'clear' property does not apply to elements with a display of
'inline-table'」——inline-table 旁置 float 右（x=float 右缘、y=流顶）、保持声明宽
96×96（ref = clear-applies-to-001-ref 蓝块 96×96 @ (328,8)）。

## 归因（临时探针变体 + 阶段边界插桩，用后即删）

- **变体探针**（A 裸/B float 后/C px96/D 去 table-layout:fixed/E auto/F 块兄弟/G
  float 无 clear）：C/D/E/G 全 464——与 `1in` 单位、`table-layout:fixed`、`clear`
  全无关；**只要前面有 float 兄弟，inline-table 宽即被写成 float 旁可用宽** 464
  （=784−320，定宽被无视，与 auto 同解）；F 证明单纯块兄弟无害。
- **阶段边界插桩**（S0 pre→S1 adjust_float→S2 shrink-inline-blocks→S3 remeasure
  float-excl→S4 remeasure-inline-only→S4b step8 表布局→S4c step8.5→S5）：
  B/C/D 全部 **S4b w=96 h=96 正确 → S4c w=464**；E（auto）S4b w=17.6 → S4c 464。
  写入点锁定 **step 8.5 `fix_table_among_floats`（table_float_fix.rs，R1518d V2）**。
- **C 步铁证**（自带 `ZW_TABLE_FLOAT_DBG=1`）：`C bfc-avoid table: (320,0) ->
  (320,0) w=464 naty=0`。
- **机制**：C 步（§9.5 BFC float-avoidance）beside 分支 `(!is_anon_root && beside)`
  无条件 `table.width = mw`（gap 宽）——R1613 语义为 **auto-width** BFC 表按
  floats-wrap-bfc-002 ref 显式 width:200 调优；declared-width 表被同臂改写。014
  的 `clear:both` 不入 `LayoutBox.clear`（inline-table 不适用 clear，恰是案面
  assert），`!is_cleared` 门放行。step 8 表布局本身输出正确（96×96），被 8.5 撤回。
  注：R1723 below 恢复声明宽臂已存在且正确（放不下 gap 时 band-step 推下走 `!beside`
  分支）；缺口仅在 beside 且放得下的形态。

## 修复（table_float_fix.rs C 步单点）

beside 目标宽按 declared 分流：`beside_target_w = declared_w.map_or(mw, |dw|
dw.min(mw))`（band-step 用 effective_w=declared 判 beside，能 beside 即 dw ≤ gap，
min 仅防 step5 收缩残余越界）；`width_change` beside 分支与终写同用 `beside_target_w`。
auto-width（declared_w=None）逐字节维持填 gap。014 修复后 pos_change/width_change
双假 → C 步零触碰（step 8 的正确几何直通终局）。kill-switch
`ZW_TABLE_FLOAT_BESIDE_KEEP_DECLARED=0` 回退 beside 一律填 gap（default-on）。

## 常驻测试

`engine/tests/r5398_table_beside_declared_width_tests.rs` 4 针：

- `test_inline_table_beside_float_keeps_declared_width`：014 主形态（inline-table
  width:96 + clear:both beside float）——w=96（旧 464 load-bearing）+ x=float 右缘
  + 不下推 float 底（clear 不适用 inline-table）。
- `test_declared_block_table_beside_float_keeps_declared_width`：定宽 display:table
  beside 保 150（分流谓词按 declared 判，与 display 形态无关）。
- `test_auto_table_beside_float_still_fills_gap`：R1613 既有语义守卫——auto 表
  beside 仍填 gap 684（分流只动 declared 臂）。
- `test_declared_table_pushed_below_float_restores_declared_width`：R1723 既有
  语义守卫——float 700 使 gap 84 < 150 → 下推 float 底、x=0、恢复声明宽。

## 验证

- 新针 4/4 绿；kill-switch 关断（`ZW_TABLE_FLOAT_BESIDE_KEEP_DECLARED=0`）精确
  转红两根 beside 保宽针、below/auto 守卫保持绿（开关敏感性外科级）。
- 案面：014 ✓1/1 **0.00%（真通过-可信）**；关断态精确回 ✗7.36%（修复前基线值，
  35328/480000 px）。
- corpus 全量 A/B（双侧重跑二进制，基线 = 当前 HEAD 32ebe9f98）：
  基线 732F → 修复 731F 账面。FIXED 候选 {014 7.36% + position-absolute-dynamic-
  static-position-inline}、REGRESSED 候选 {insert-block-in-blocks-middle-003}——
  三案隔离复跑：014 BASE 3×3 稳定 ✗7.36% / FIX 3×3 稳定 ✓0.00%（真修复）；
  position-absolute 与 middle-003 两侧各 3×3 全 ✓0.00%（全量窗口并发抖动洗清，
  R5395/R5396 在册同族）。真态 **731F，FIXED={clear-applies-to-014}，REGRESSED=∅**。
- 门禁：fmt ✓ / guarded-clippy 全工作区 ✓ / layout-engine 全腿 2373 绿（含 4 新针）/
  本地集 **725/725 全绿** / make test 除 **HEAD 既有对端账**（js_dom_bridge CSSOM
  序列化 5 案：style.color 读回 'rgb(255,0,0)' 应 'red' 等，stash 后干净 HEAD
  复跑同败实证——652a78e00 CSSOM 颜色归一同域，非本轮引入，记对端账待对端流收口）/
  product-smoke 6 腿全 PASS / bench-gate 定向 zero-layout-engine **GATE PASS 25
  指标（NEW=0）**（首查 loadavg 13.3 对端流负载，静窗 3.78 复跑，R5392/R5396 同型流程）。

## 挂账 / 下一步

- floats-clear 族余红：floats-031（3.89%）、floats-145（2.06%）侦察待排；
  negative-clearance-after-bottom-margin（2.55%）/ margin-collapse-104（4.17%）
  维持 R5397 深结构挂账（五规则语义链在案）。
- **对端账新增**：js_dom_bridge CSSOM 序列化 5 案（HEAD 既有，652a78e00 同域）——
  本轮 make test 唯一红面，待对端流确认收口。
- R5242 kill-switch 关断态既有回归维持；R5394/R5392/R5391/R5390/R5389 挂账全维持；
  既有对端账（image_batch_e2e 环境前置/mutation_arm/default_actions/paste_and_go）维持。
