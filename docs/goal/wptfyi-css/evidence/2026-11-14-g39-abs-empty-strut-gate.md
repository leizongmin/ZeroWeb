# g39：空绝位 auto 高 strut 门——abspos shrink-to-fit 塌 0 补测收窄

日期：2026-11-14。承接 g36 挂账桶「'none'/空绝位元素高 18」（evidence
2026-11-14-g36-border-coord-s423.md §7）。

## 1. 根因追猎（三步法证链）

g36 组合态账遗留：`position-area: none` 形态页 anchored 空盒
offsetHeight=18（语料期望 0）。单测探针链定谳：

1. **taffy 树 0×0**：`ZW_TAFFY_TREE_DUMP=1`（R5011 dump 工具走
   stderr，首查 grep stdout 漏捕）dump 显示 abs 空盒 taffy 布局
   `size=(0.0x0.0)`——taffy 自身解析正确（OOF 子不入 content_size）。
2. **提取盒 0×18**：全树 LayoutBox 打印（node_id/尺寸/旗标）显示同一
   盒提取后 `size=(0x18)`——注入发生在 compute 内 extract 与 dump 之间
   的后处理带。
3. **写入点定谳**：`apply_abspos_shrink_to_fit_width`（sizing.rs）
   `height_fix` 臂——abs 盒 taffy 高塌 0（`b.height <= 0.5`）时对
   **所有**非 replaced 非 grid 盒无条件写行高 strut
   （`resolve_font_metrics` → `lh.max(fs).max(1.0)` ≈18）进 taffy
   style 后重跑布局。该近似本意服务**文本叶**（taffy 对该形态无
   measure，strut 即行盒高近似），但门上没查行内内容——空子树盒同吃
   strut。CSS2 §10.6.4：非替换 abs 盒 height auto = 内容高，空内容
   = 0。

排除项：`recompute_bfc_auto_height`（`!is_absolute` 门不吃 abs 盒）；
`remeasure_text_with_float_exclusions`（`has_floats` 门，空盒不进）；
orthog 垂直域 `resolve_orthog_abspos_shrink_to_fit`（元素子早退 +
空行盒不写，同查无此病）。

## 2. 修复

sizing.rs `height_fix` 臂三分重构：

- replaced_collapsed / grid_tracks_fix 两臂语义不变；
- strut 臂加 `crate::inline_content::has_inline_content(doc, styles, id)`
  门——只有有行内内容（直接文本或 inline/inline-block 元素子）的塌 0
  盒才写 strut；无行内内容不写，维持 taffy 解析（塌 0 = §10.6.4
  内容高 0）；
- 终写改 `style != style_before` 判——无写入不再 set+dirty（旧径对
  塌 0 盒恒 `changed=true` 触发空转重布局）。

同 crate 共享树并行流说明：本轮 style-system/anchor_pos 有他流 g38
anchor-size 槽切片同树在途，release 二进制（18:28 构建）按 provenance
含其**接线态**（anchor-size 语页面 +73 实证非惰性，g37 轮「共享树构建
编译他流在途半成品」教训复现）——§4 为组合态账，本刀面经内容机制逐案
分解（见 §4），发射前工作树非干净以逐案归因补偿。

## 3. 验证

- 常驻钉测 `g1f_abs_empty_auto_height`（探针转正，g1f_anchor_pos_
  tests.rs）：abs 空 Inline（§9.7 块化）/空 Block 0×0 + abs 含文本
  盒行盒高保持 + 普通流空块 0 对照 + 显式高 400 容器对照。
- layout-engine lib **2510 过 0 挂**（+1 = 新钉测）；fmt 净；clippy
  本刀净（5 告警均他流在途）。

## 4. corpus A/B（组合态账，vs comb3-x7 基线）

探针快验（release 二进制，先验链路）：

- zz-g36probe-none-geometry：span-all `0,0,400,400` ✓ / **none
  `0,0,0,0`（修复前 h=18）** / top-left `0,0,100,150` ✓——'none' 形态
  修复兑现，两定位形态零扰动。
- position-area-basic 0F；last-successful-basic 维持 g36 揭真挂账形
  （expected 200 got 250 等，本刀无关）。

全量重录（results-g39-strutgate，90 秒空闲机串行 ~0.17s/案，transcript
/tmp/zw-g39-strutgate.log）：**265 页 12,061P/796F/4T**。对 comb3-x7
JSON 净 +76（up=16 same=248 down=1）；对 g37-final 真态基线（11,993P
/864F/4T）净 **+68**——差 8 = comb3-x7 已定谳 skew（伪绿 12 − 伪红 4，
见 g37 轮更正块），本二进制含 g37 已提交代码故 skew 两项互消。

归因分解（组合态：本刀 + 他流 g38 anchor-size 回填同树在账）：

- **本刀 +3，零回归**：
  - position-area-basic `Offsets for: none`（空 anchored 盒 h 18→0
    对齐期望 0）；`Offsets for: span-all top`（'span-all top' 写面
    文法缺口被拒落 none 态，同空盒路径 h 18→0 对齐 {0,0,0,0}）；
  - anchor-position-multicol-001 `.target 4`（inset bottom
    `anchor(--a1 bottom)` 的高依赖式 y = cb_h − bv − h，strut 剔除后
    对齐 data-offset-y=110）。
- **基线修复项 +4**：anchor-invalid-fallback 29→33——comb3-x7 伪红 4
  （build2 缺 parse_length_or_math guard）在本二进制（含 g37 已提交
  guard）复正，非新增增益。
- **他流 g38 anchor-size 回填面（净 ~+65，含 −1 REG 与页内混翻）**：
  anchor-size-writing-modes-001 +24 / anchor-size-001 +17 /
  anchor-size-replaced-001 +8 / anchor-size-minmax-001 +3 /
  anchor-name-001~004 +2~4（003 页内 8F→P/4P→F 混翻，anchor 选择序
  问题）/ anchor-query-fallback +2 / anchor-position-dynamic-002 +2 /
  multicol-006 +1 / colspan-001 +1（上述页均含 anchor-size 声明，
  grep 计数实证）；REG：anchor-ident-function
  `Property width value 'anchor-size(--a1)'` P→F——全 anchor-size
  声明域，挂账归 g38 流处置。

