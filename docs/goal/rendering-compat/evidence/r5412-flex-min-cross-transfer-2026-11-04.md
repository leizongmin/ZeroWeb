# R5412 证据：replaced AR flex item definite min-cross 传递——flex-minimum-width-flex-items-015 ✗6.25%→✓0.00%

日期：2026-11-04。驱动案：`css/css-flexbox/flex-minimum-width-flex-items-015.html`
（column 100×444 > row（flex-basis:100px + min-height:0px）> img（解码固有 200×200 +
`min-height:100%`），ref = ref-filled-green-100px-square → img 应精确 100×100）。
meta assert 原文：「Column flexbox's imposed definite size on row flexbox is honored
when resolving percentages in automatic minimum size algorithm」。

## 1. 归因：% min-cross 传递缺口 + R3794 设备阻断交叉轴

series 基线 14/15 ✓（仅 015 ✗）——009（min-height:100px，60px img）经 floor 语义
（传递值 100 ≥ 内容建议 60）过、011/012/013/016（显式 cross/specified main）经既有
transferred 臂过。015 独缺两环：

1. **% min-cross 静态不可解析**：`apply_flex_transferred_min_size`（tree.rs:2345）
   cross 源仅 (a) 容器 CSS definite cross + stretch、(b) item specified/max cross——
   min-cross（min-height:100%）无源；且 % 解析器 `resolve_tree_definite_real_length`
   对 Percentage 恒 None——**父 row 的 definite 高 100 来自外层 column flex-basis
   求解，CSS style 无声明**，静态树构建期原理性不可见。
2. **floor 语义方向错**：R3794「只抬不降」floor max(内容建议 200, 传递 100) = 200，
   而规范 automatic minimum = min(specified suggestion〔=transferred〕, content
   suggestion) = min(100, 200) = 100——收缩应穿透内容地板（009 assert：「content
   suggestion accounts for minimum cross size」同源语义，其 60px img 恰使 floor≥content
   掩盖方向差）。

探针（probe_r5412a，用后即删）四形态：V1（min-height:100%）200×200 ✗（应 100×100）、
V2（无 min）200×200 ✓（内容建议 floor，classic flex img no-shrink）、V3（min-height:
100px）200×200（同缺口 px 形态，corpus 无驱动案）、V4（min-width:100px）100×200 ✓
（显式 main min，automatic minimum 语义域外）。BOX walk：父 row 首趟 laid 高 100 ✓
（taffy 已把 flex 求解 definite 写回布局）→ % 基准在修复层可达。

## 2. 修复（sizing.rs `apply_flex_aspect_ratio_item_size` walk 内单臂，kill-switch `ZW_FLEX_MIN_CROSS_TRANSFER=0` default-on）

插入点：`main_is_auto && (!main_has_definite_min || b.is_replaced)` 臂（R1364 链后）。
门：仅 replaced（非替换维持 R3859 静态 floor 臂）+ ratio>0（外层既有）+ min.main 属性
auto（automatic minimum 语义域，R4988 同门）+ cross 轴 padding 零（R3859 同门）。

1. **definite min-cross 解析**：px 走 `resolve_sizing_definite_real_length`；%
   基准 = taffy 首趟 laid 父 content cross（`taffy_tree.parent(tid)` → `layout(ptid)`
   size − border − padding，css-flexbox §4.1 flex 求解 definite）。
2. **automatic minimum cap**：transferred = min-cross × ratio；仅当 < 当前地板 −0.5
   时**降** `min_size.main`（cap 非 floor；≥ 现值 no-op，R3794 floor 形态不变）。
3. **交叉轴收口（§9.4）**：R3794 both-auto 设备把 raw 固有 cross 钉成 taffy
   definite、阻断 stretch/min 解析（尾差：宽 100 ✓ 高仍 200）。CSS cross auto 且
   设备值 > min-cross +0.5（设备指纹）时释放 size.cross = auto + 写 min-cross 为
   `min_size.cross` 地板（R3794 记录的 row-003 塌缩顾虑由地板兜住；R1364 已写的
   stretch cross ≤ min-cross 不触碰）。

## 3. 探针 A/B（修复后）

| 形态 | 修复前 | 修复后 | 应 |
|---|---|---|---|
| V1 min-height:100%（驱动案） | 200×200 | (8,8,100,100) ✓ | 100×100 |
| V2 无 min-height | 200×200 | 200×200 ✓（不变） | 200×200 |
| V3 min-height:100px | 200×200 | 100×100 ✓ | 100×100（009 assert 语义） |
| V4 min-width:100px | 100×200 | 100×200 ✓（不变） | 语义域外 |
| kill-switch OFF | — | V1/V3 精确回 200×200 | R3858 口径 |

## 4. 常驻测试

`crates/engine/tests/r5412_flex_min_cross_transfer.rs` 2 针：
- `r5412_min_cross_pct_transfers_and_caps_automatic_min` ★load-bearing：015 形态
  (8,8,100,100)——cap + 交叉轴收口。
- `r5412_no_min_cross_keeps_content_suggestion`：无 min 形态 (8,8,200,200)——
  cap 不降、设备不释放。2/2 绿。

## 5. corpus 验证（全量 A/B 双窗，16594 案，BASE=同二进制 `ZW_FLEX_MIN_CROSS_TRANSFER=0`）

- 单案 ON ✓0.00%（修复前 ✗6.25%）；series 全组 14/15 → 15/15。
- 窗1 OFF 15874 / ON 15875：FIXED 候选 {015, insert-inline-middle-003, table-anon-129/148}、
  REGRESSED 候选 {table-anon-128/146/154}。
- 窗2 OFF 15874 / ON 15875：FIXED 候选 {015, insert-block-middle-001, insert-inline-end-002,
  svg-mutation-single-to-multiple-002}、REGRESSED 候选 {table-anon-130/149/153}。
- 交叉定谳：015 两窗均 FIXED＝唯一稳定归属；其余 11 候选（两窗 table-anon 集不同＋
  insert-block/repaint 族）单案隔离 ON/OFF 双侧 3×3 **全绿**＝在册抖动族
  （guidelines #35）。**真态 FIXED={flex-minimum-width-flex-items-015} REGRESSED=∅，
  corpus 真态 715F→714F**（窗口原始计数 OFF 720F/ON 719F 含在册抖动族波动，
  真态自 R5411 收口值 715F 延续）。

## 6. 门禁

fmt ✓；guarded-clippy 全工作区 EXIT 0；zero-layout-engine 全腿绿（lib 2394 ✓）+
r5411/r5412 常驻 4/4 ✓；make test——workspace 腿 3724P/5F（js_dom_bridge CSSOM 对端账
在册 5 案同名单，HEAD 既有）+ renderer-bin/renderer-lib/gpu/gpu-consistency 四腿绿 +
browser-bin/browser-tests 各 1 案红＝`default_actions_work_without_javascript`
（**新 drift 红腿**：`git stash push`（仅 sizing.rs）后 HEAD 同败 2/2＝位元无关，
单测隔离 5/5 败、同 commit R5411 全绿过＝环境漂移（本机 proxy/端口态嫌疑，归对端
碰头账），非本轮引入）；product-smoke 主腿 form-input-perf-gate PASS + struct
全 PASS、legacy struct-check failures: 0；bench-gate 定向 zero-layout-engine：
ON1 腿 4 宏指标超预算（block 9.52ms/flex 12.22ms/incremental 4.15ms/wide_tree
452µs——均 3-4× 于同会话静窗值＝前段编译负载余波），静窗 ON2 GATE PASS 25（NEW=0）
＋同窗 kill-switch OFF 配对腿 GATE PASS 25（NEW=0）——ON/OFF 静窗值 flex
3.227/3.138ms、block 3.586/3.425ms、incremental 3.259/3.207ms、wide_tree
187.3/186.6µs（ON +0.4~4.7% 噪声带内，R5410/R5411 配对口径）。

## 7. 挂账（本轮新增）

1. **px min-cross cap 的非 flex 域外溢未评估**：本臂仅 flex item 域；009 家族
   （200px img + min-height px + 收缩域）cap 语义已并入且 V3 探针 ✓，但块流域
   （replaced-elements-min-height-20 判例 floor 语义）未触碰——不触碰即无回归面，
   留作驱动案出现时的语义统一项。
2. **非替换 % min-cross 缺口**：R3859 臂仅静态解析，% 对 flex 求解父 cross 同理
   不可见（无驱动案，本臂 replaced 限定外）。
3. **css-flexbox 深域归档修订**（侦察结论落账，见控制面 R5412 块）：竖排
   writing-mode flexbox 深域 19→20 案（auto-margins-003 全 10657 差异 px 在
   vertical-lr 半区并入）；scrollbar 族两案特性级归档（ZW 无 classic scrollbar
   geometry，仅 scrollbar-gutter 属性 + 绘制效果）；align-self-016 → column wrap
   multi-line 域；flex-wrap-006 维持动态域。

## 8. 工件

- 修复：`crates/layout-engine/src/engine/sizing.rs`（R5412 单臂：min-cross 解析 +
  automatic minimum cap + 交叉轴设备释放）。
- 常驻测试：`crates/engine/tests/r5412_flex_min_cross_transfer.rs`。
- 探针 `probe_r5412a.rs` 用后即删；A/B 窗口记录 /tmp/r5412_ab_{on,off}_w{1,2}.txt。
