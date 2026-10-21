//! R1518d/R1518 V2：table-among-floats scoped iterative fix。
//!
//! 背景（见 docs/goal/rendering-compat/master.md R1513–R1518d）：`table-among-floats-001`
//! 失败源于 pass 顺序——`adjust_float_positions`（step5）早于 `adjust_table_layout`（step8）。
//! step5 时 table 是全宽匿名包装盒（无法 §9.5 推开），step8 shrink-to-fit 到真实窄宽后
//! **不重跑 §9.5**，故窄 table 仍堆在 float 下方（y=200），其高度经
//! `reflow_siblings_after_table_height_change`（table.rs）扩容器 +100（200→300）。
//!
//! R1518 用全树 `adjust_float_positions` 重跑解几何但 net -2（margin-collapse 簇回归）；
//! R1518c 用 step5 BFC-shrink 解容器高度但 net-0（table 未布局，shrink 基于不完整信息，
//! 被 step8 reflow_siblings 覆盖）。
//!
//! 本模块（V2）**scoped**：仅对「同时含 float 子 + table 子」的容器介入，
//! (A) table 子树内 re-wrap 内层 float（收窄 td 内堆叠）→ (B) `layout_table` 重算 table 高 →
//! (C) 手动 §9.5 push（仅 table，不重跑全树 → margin-collapse 案无此结构故不受影响）→
//! (D) 重算容器高度 = MAX(float 底边, in-flow 底边)（覆盖 step8 reflow_siblings 的 +delta）。
//!
//! 关键 scoping：margin-collapse-121/122/125/142 等回归案无「float+table 同容器」结构，
//! 本 pass 不介入；只有 table-among-floats 结构的容器才跑——这是 V2 相对 R1518 全树重跑
//! 的核心差异，预期消除 margin-collapse 回归。
//!
//! env `ZW_TABLE_FLOAT_ITER_V2=0` 关闭（kill-switch，default-on）；`ZW_TABLE_FLOAT_DBG`
//! 打印每步几何供 A/B 诊断。

use std::collections::HashMap;

use zero_css_parser::values::{ClearValue, DisplayValue, FloatValue, LengthValue};
use zero_dom::{Document, NodeId};
use zero_style_system::ComputedStyle;

use crate::float_positioning::adjust_float_positions;
use crate::table::{layout_table, layout_table_with_width_constraint};
use crate::types::LayoutBox;

/// table-among-floats scoped iterative fix 入口。post-order 遍历，对匹配结构的容器执行
/// A+B+C+D。env `ZW_TABLE_FLOAT_ITER_V2=0` 关闭。
pub(crate) fn fix_table_among_floats(
    root: &mut LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    inline_fonts: crate::inline_finalization::InlineFontContext<'_>,
) {
    if std::env::var("ZW_TABLE_FLOAT_ITER_V2").as_deref() == Ok("0") {
        return;
    }
    let mut grown_cells: HashMap<NodeId, f32> = HashMap::new();
    fix_inner(root, doc, styles, &mut grown_cells, inline_fonts);
    // R5213（CSS2 §17.5.2 行高 = max(cell 底边) + §10.6.3 auto 高）：D 步扩高的 cell 高度
    // 须上传行/行组/表并位移块域后继兄弟——此前 R1723 搁置（bfc-004 ref definite 20 被
    // 误增长回归）。bfc-006 推下形态 cell 76/68/76/69 须上传（外层表高原停 60，chromium
    // 76/68/76/69 CDP 实测）；上传基线取 root.height 非 content_height（tr/tbody 的
    // content_height 为 0，逐层放大会把 60 放大到 196，见 propagate 内注释）。
    if !grown_cells.is_empty() {
        propagate_grown_cell_heights(root, &grown_cells, styles);
    }
}

/// R5213：把 D 步扩高的 cell（`grown_cells`: node_id → 高度增量）沿
/// cell → row → row-group → table 上传（祖先高 = max(in-flow 子盒底边)，只增不减），
/// 并在纵向堆叠语境把后继兄弟整体下移（row 内 cell 横排，不位移同级 cell）。
/// 返回 root 自身高度增量。
fn propagate_grown_cell_heights(
    root: &mut LayoutBox,
    grown_cells: &HashMap<NodeId, f32>,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> f32 {
    let mut max_child_delta = 0.0f32;
    // row 的子（cell）横排——cell 增高不位移同级 cell，仅上传行高。
    let root_is_row = root
        .node_id
        .and_then(|id| styles.get(&id))
        .is_some_and(|s| s.display == DisplayValue::TableRow);
    for i in 0..root.children.len() {
        let d = propagate_grown_cell_heights(&mut root.children[i], grown_cells, styles);
        if d > max_child_delta {
            max_child_delta = d;
        }
        if d > 0.01 && !root_is_row {
            for c in &mut root.children[i + 1..] {
                c.y += d;
            }
        }
    }
    if max_child_delta > 0.01 {
        // 有子增长 → 按 in-flow 子底边最大值只增不减（float/abs/fixed 子不计入
        // CSS auto 高；原高更大时维持原值，不收缩）。
        let extent = root
            .children
            .iter()
            .filter(|c| !c.is_absolute && !c.is_fixed && matches!(c.float, FloatValue::None))
            .map(|c| c.y + c.height)
            .fold(0.0f32, f32::max);
        // 以 height 为基线：中间层 tr/tbody 的 content_height 可能为 0（表管线
        // 只填 leaf 内容高），用 content_height 会把 extent 全额当增量逐层放大
        // （bfc-006 外层表 60→196）。
        let g = (extent - root.height).max(0.0);
        if g > 0.01 {
            root.content_height += g;
            root.height += g;
            return g;
        }
        return 0.0;
    }
    // 基例：自身是 D 步扩高的 cell，返回记录增量。
    if let Some(id) = root.node_id
        && let Some(d) = grown_cells.get(&id)
    {
        return *d;
    }
    0.0
}

fn is_table_box(b: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) -> bool {
    // 仅 in-flow table（非 float）才算 §9.5 推开候选。float 的 table（如
    // float-page-break-inside-avoid-1-print 的 <table class="test" float:left>）本身是
    // 浮动元素，由 float 定位处理，不应被当作 in-flow BFC 做 §9.5 推开。
    if is_float(b) {
        return false;
    }
    // R5207：孤立 table-internal 元素经 mark_anonymous_table_roots / merge_orphan_table_run
    // 形成的匿名 table 根（CSS2 §17.2.1.1）与真表同为块级 BFC，同属 §9.5 推开候选——
    // step8 layout_table 收窄后 x 回落流位 0，step5 的 BFC 推开无重跑（R1518 同构），
    // 不识别则压在前置 float 上（clear-applies-to-005/006：colgroup/col + row 群包装盒）。
    if b.is_anon_table_root {
        return true;
    }
    b.node_id
        .and_then(|id| styles.get(&id))
        .is_some_and(|s| matches!(s.display, DisplayValue::Table | DisplayValue::InlineTable))
}

fn is_float(b: &LayoutBox) -> bool {
    !matches!(b.float, FloatValue::None)
}

fn is_in_flow(b: &LayoutBox) -> bool {
    !is_float(b) && !b.is_absolute && !b.is_fixed
}

fn resolve_declared_table_width(style: &ComputedStyle, content_width: f32) -> Option<f32> {
    match &style.width {
        LengthValue::Auto | LengthValue::MinContent | LengthValue::MaxContent | LengthValue::FitContent(_) => None,
        LengthValue::Percentage(p) => Some((*p as f32 / 100.0) * content_width),
        LengthValue::Px(v) if *v == f64::INFINITY => None,
        LengthValue::Px(v) => Some(*v as f32),
        other => {
            let font_size_px = zero_style_system::computed::resolve_length(&style.font_size, 16.0, None, None);
            let px = zero_style_system::computed::resolve_length(other, font_size_px, None, None);
            px.is_finite().then_some(px as f32)
        }
    }
}

fn fix_inner(
    root: &mut LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    grown_cells: &mut HashMap<NodeId, f32>,
    inline_fonts: crate::inline_finalization::InlineFontContext<'_>,
) {
    // post-order：先修子容器（嵌套结构）
    for child in &mut root.children {
        fix_inner(child, doc, styles, grown_cells, inline_fonts);
    }
    // 仅「同时含 float 子 + table 子」的容器介入（scoping 关键）
    let has_float = root.children.iter().any(is_float);
    if !has_float {
        return;
    }
    let Some(tidx) = root.children.iter().position(|c| is_table_box(c, styles)) else {
        return;
    };
    let dbg = std::env::var("ZW_TABLE_FLOAT_DBG").is_ok();
    let content_width = root.content_width;

    // === A: re-wrap table 子树内层 float（收窄 td 内 inner float 堆叠）===
    adjust_float_positions(&mut root.children[tidx], doc, styles);

    // R5207：匿名 table 根的几何在初始布局已定稿（shrink-to-fit 宽含内），本 pass 仅做
    // C 步 §9.5 推位，不重排——layout_table 重排按 fill-available 度量，会把已 shrink
    // 的匿名表根撑宽（clear-applies-to-001：96 → 480，cell 背景横跨整行）。
    let is_anon_root = root.children[tidx].is_anon_table_root;

    // === B: 重算 table 高度（inner float 堆叠后 table h 增长，如 100→200）===
    if !is_anon_root {
        layout_table(&mut root.children[tidx], doc, styles, inline_fonts);
    }

    // 先用不可变读取计算 natural_y / avoidance_x / is_cleared（避免与下方可变借用冲突）
    let mut table_h = root.children[tidx].height;
    let mut table_w = root.children[tidx].width;
    // R1723：definite-width table 的「声明宽」（Px 或 Percentage 解析到容器 content_width）。
    // step5 `adjust_float_positions` 会把旁 float 的 BFC table **shrink** 到可用宽（如 150→100），
    // 故此处读到的是 shrink 后宽。但 CSS §9.5：definite-width table 应保持声明宽，放不下 float 旁
    // 可用空间时**推到 float 下方**（非 shrink beside）。floats-wrap-bfc-005 子案 1/2：
    // `<table width="50%">`（=150）旁 200px float（300 容器，可用 100）→ 应推下保宽 150，旧实测
    // shrink 到 100 beside。用 effective_w（声明宽 if definite，否则当前 table_w）做 fit 决策，
    // below 时恢复声明宽。
    let declared_w: Option<f32> = root.children[tidx]
        .node_id
        .and_then(|id| styles.get(&id))
        .and_then(|s| resolve_declared_table_width(s, content_width));
    let mut effective_w = declared_w.unwrap_or(table_w);
    // clear != None 的 table 应由 clear 逻辑定位（推到 float 下方），不做 §9.5 推开
    //（clear-applies-to-013：display:table + clear:both 应清到 float 下，非推到 float 右）。
    let is_cleared = !matches!(root.children[tidx].clear, ClearValue::None);
    // natural_y = 前置 in-flow 兄弟底边最大值（child.y 相对父内容盒顶）；无则 0
    let natural_y = root
        .children
        .iter()
        .take(tidx)
        .filter(|c| is_in_flow(c))
        .fold(0.0f32, |my, c| my.max(c.y + c.height));
    // R1721：float:right avoidance 预读（提前到 shrink 臂前——两者共用重叠判定）。
    let right_float_left: Option<f32> = {
        let mut rl: Option<f32> = None;
        for f in &root.children {
            if matches!(f.float, FloatValue::Right) && natural_y < f.y + f.height && natural_y + table_h > f.y {
                rl = Some(rl.map_or(f.x, |m: f32| m.min(f.x)));
            }
        }
        rl
    };
    // === B2（R4227，CSS2 §9.5「may even shrink」）：auto-width 表右 float 避让收缩——
    // effective_w > 右 float 群左侧可用宽（right_float_left = 与 table y 域重叠的右 float
    // 最小 x）时，表按约束宽重排（列宽比例收缩，layout_table_with_width_constraint）后
    // beside 放置，非推到 float 下方。table-sizing-with-adjacent-floats：auto 表 100
    //（td width:100 列）+ 右 float 群 min-x=50 → 缩到 50 beside（chromium ref 绿 50×100）。
    // 仅 auto-width（declared_w None）：definite-width 表维持 R1723 推下保声明宽
    //（floats-wrap-bfc-005 子案：150 应推下非缩 100）。
    // **cell 语境门**（empirical chromium 差异）：floats-wrap-bfc-003-right-table（同构形态
    // 但父为 td）chromium **不收缩**——250 表在 200 可用旁仍推到 float 下方保全宽（ref
    // 像素实证），故 td 内 float 表不适用 shrink 臂。kill-switch `ZW_TABLE_FLOAT_SHRINK=0`。
    let root_is_cell = root
        .node_id
        .and_then(|id| styles.get(&id))
        .is_some_and(|s| matches!(s.display, DisplayValue::TableCell));
    if std::env::var("ZW_TABLE_FLOAT_SHRINK").as_deref() != Ok("0")
        && declared_w.is_none()
        && !is_cleared
        && !root_is_cell
        && !is_anon_root
        && let Some(rl) = right_float_left
        && rl > 0.5
        && effective_w > rl + 0.5
    {
        layout_table_with_width_constraint(&mut root.children[tidx], doc, styles, inline_fonts, Some(rl));
        table_h = root.children[tidx].height;
        table_w = root.children[tidx].width;
        effective_w = table_w;
        if dbg {
            eprintln!("ZW_TABLE_FLOAT_DBG B2 shrink table: w -> {} (avail {rl})", table_w);
        }
    }
    // avoidance_x = 与 table 垂直范围 [natural_y, natural_y+h] 重叠的 float 的右 margin-box 边最大值
    let avoidance_x = root
        .children
        .iter()
        .filter(|c| is_float(c))
        .map(|f| {
            let overlap = natural_y < f.y + f.height && natural_y + table_h > f.y;
            if overlap { f.x + f.width + f.margin_right } else { 0.0 }
        })
        .fold(0.0f32, |mx, v| mx.max(v));
    // === C: §9.5 BFC float-avoidance（per-y；R1609 泛化原 fits-only push）===
    // 原逻辑仅当 table 在 natural_y 能放进 float 右侧（fits）时推右；放不下（fits=false）时
    // 不动 → table 卡顶部重叠 float（floats-wrap-bfc-006：230px table 放不进 142px 剩余空间）。
    // 新逻辑：从 natural_y 起按 float bottom 递进，找首个 table 整高 [y,y+h] 不与任何 float
    // 重叠的位置——放得下则 x=max_right、y；放不下则 y 推过当前最大右边缘（最宽）float 的
    // bottom，逐 float 收敛。clear 的 table 由 clear 逻辑定位，不介入。
    // 触发条件 = avoidance_x > 0.5（natural_y 处有重叠 float 须避开）；table 已在正确位置则
    // diff 检查不移动（守 blocks-025 等 float 不重叠案）。
    let mut pushed = false;
    // R5213：区间化 band-step（左右 float 分离，吸收 R1721 纯右 float right_target 臂）——
    // 左 float 群右缘 max 给左界 left_edge，右 float 群左缘 min 给右界 right_edge；table
    // 整高 [y,y+h] 放得进 [left_edge, right_edge] 即 beside 于 x=left_edge、填 gap 宽。
    // 无右 float 时 right_edge=content_width，退化为既有单侧 max_right 算法；纯右 float
    // 时 left_edge=0，等价旧 right_target。右 float 不再以右缘充当左界阻挡物（旧
    // max_right 语义会把混合 float 场景的可用区间误判为 0）。
    // kill-switch ZW_TABLE_FLOAT_RIGHT_AVOID=0：右 float 退回旧 max_right 语义（仅作
    // 阻挡物参与推下，不给区间右界）。R4227：right_float_left 预读仍供 B2 shrink 臂。
    // target = (nx, ny, fill_w)：fill_w = beside 可用区间宽（填），below 时 = 全宽（仅 clamp）。
    let right_avoid_off = std::env::var("ZW_TABLE_FLOAT_RIGHT_AVOID").as_deref() == Ok("0");
    let target: Option<(f32, f32, f32)> = if !is_cleared && avoidance_x > 0.5 {
        // 左 float 群 (右 margin 缘, 顶, 底)；右 float 群 (左缘, 顶, 底)
        let mut lefts: Vec<(f32, f32, f32)> = Vec::new();
        let mut rights: Vec<(f32, f32, f32)> = Vec::new();
        for f in root.children.iter().filter(|c| is_float(c)) {
            if matches!(f.float, FloatValue::Right) && !right_avoid_off {
                rights.push((f.x, f.y, f.y + f.height));
            } else {
                lefts.push((f.x + f.width + f.margin_right, f.y, f.y + f.height));
            }
        }
        let mut y = natural_y;
        let mut placed_x = 0.0f32;
        let mut avail_w = content_width;
        let mut found = false;
        // band-step：每轮 y 严格推进到下一 float 底边（单调收敛），上界 = float 数 + 2。
        for _ in 0..(lefts.len() + rights.len() + 2) {
            let left_edge = lefts
                .iter()
                .filter(|(_, ft, fb)| *ft < y + table_h && y < *fb)
                .map(|(r, _, _)| *r)
                .fold(0.0f32, f32::max);
            let right_edge = rights
                .iter()
                .filter(|(_, ft, fb)| *ft < y + table_h && y < *fb)
                .map(|(l, _, _)| *l)
                .fold(content_width, f32::min);
            let gap = (right_edge - left_edge).max(0.0);
            if effective_w <= gap + 0.5 {
                placed_x = left_edge;
                avail_w = gap;
                found = true;
                break;
            }
            // 放不下 → y 推进到仍与 [y, y+h] 重叠的 float 中**最近**的底边（per-y band，
            // earliest-fit）。R5213 定谳（chromium CDP 探针，取代 R1612 line-advance）：
            // chromium 的 float 与 BFC 盒均落最近 float 底旁（bfc-004 S1 表 (100,6) 实测），
            // 非行底 line-advance；且 D 步扩高上传（propagate_grown_cell_heights）以
            // earliest-fit 落位为前提——line-advance 把表推到行底后扩高上传使外层表
            // 20→30 节节下移级联爆炸（004 8.19%/006 16.17% 实测回退），两臂必须配套。
            // 已知代价：ref 侧由 ZW float 模型渲染（150 宽 float 掉行底被后节背景覆盖，
            // 扫描线实证），表单边 earliest-fit 使 004 test 可见而 ref 不可见（0.92%→2.17%）
            // ——根因在 float 摆放仍 line-advance，earliest-fit 化为下一轮独立切片，到位后
            // 004 摘红（chromium 两侧均 earliest-fit，test/ref 将同模型一致）。
            let next_band = lefts
                .iter()
                .chain(rights.iter())
                .filter(|(_, ft, fb)| *ft < y + table_h && y < *fb && *fb > y + 0.5)
                .map(|(_, _, fb)| *fb)
                .fold(f32::INFINITY, f32::min);
            if next_band.is_finite() {
                y = next_band;
            } else {
                placed_x = 0.0;
                avail_w = content_width;
                found = true;
                break;
            }
        }
        if found { Some((placed_x, y, avail_w)) } else { None }
    } else {
        None
    };
    if let Some((nx, ny, fill_w)) = target {
        let table = &mut root.children[tidx];
        // beside float：auto-width BFC table 填可用 avoidance 宽度（非 shrink-to-fit 内容宽）
        // ——floats-wrap-bfc-002-left-table（float:left，nx>0，mw=content_width-nx）/ R1721 -right-table
        //（float:right，nx=0，mw=right_float.left）auto table 应 200 非 150，匹配 ref 显式 width:200。
        // R1721：beside 判定加 `fill_w < content_width` 捕获 float:right（nx=0 但填部分宽）。
        let beside = nx > 0.5 || fill_w < content_width - 0.5;
        let pos_change = (nx - table.x).abs() > 0.5 || (ny - table.y).abs() > 0.5;
        // R5213：beside 宽语义待 §10.3.5 shrink-to-fit pref 信号（cell intrinsic 现读
        // 子盒 laid-out 宽、多 inline-block 兄弟行的 max-content 未建模，克隆探针实测
        // 回读窄化值 150 非真 pref 300，独立切片）。本沿 R1613 beside 填 gap 语义。
        // 须在「位置变 OR 宽度变」时介入（即使位置已正确，beside 宽度仍须填，故
        // width_change 独立于 pos_change）。
        let mw = fill_w;
        let width_change = if is_anon_root {
            !beside && table.width > mw + 0.5
        } else if beside {
            (table.width - mw).abs() > 0.5
        } else {
            table.width > mw + 0.5
        };
        if pos_change || width_change {
            let (old_x, old_y) = (table.x, table.y);
            table.x = nx;
            table.y = ny;
            // R1723：definite-width table 推到 float 下方时，恢复声明宽（step5 shrink 到可用宽，
            // below 应保声明宽，floats-wrap-bfc-005 子案 1/2：150 非 shrink 100）。先恢复再 clamp，
            // 防 declared>container 时溢出（width:120% 等 edge case）。
            if !beside {
                if let Some(dw) = declared_w {
                    table.width = dw;
                }
            }
            // beside float（nx>0）：auto-width BFC table 填可用宽（非 shrink-to-fit）。
            // below floats（nx=0）：仅当溢出时 clamp。合并为 `beside || 溢出` → 设 mw。
            // R5207：匿名表根 beside 不填（保 shrink 宽），溢出 clamp 照旧。
            if (!is_anon_root && beside) || table.width > mw {
                table.width = mw;
            }
            pushed = true;
            if dbg {
                eprintln!(
                    "ZW_TABLE_FLOAT_DBG C bfc-avoid table: ({},{}) -> ({},{}) w={} naty={}",
                    old_x, old_y, table.x, table.y, table.width, natural_y
                );
            }
        }
    }

    // === D: 仅当 C 实际推开了 table 且容器是 auto-height 时才重算容器高度 ===
    //（覆盖 step8 reflow_siblings 把堆叠 table 高度 +delta 到容器的过扩）。
    // 两个守卫：(1) C 未推（cleared / 不放得下 / 已正确）的容器不跑——step8 高度已正确，
    // MAX 公式过简（忽略 margin/line-height）会误改（clear-applies-to-013 body 回归）；
    // (2) definite-height 容器不跑——其高度由声明决定，不应按内容重算（floats-038 #div1
    // height:2in=192 被 D 误缩到 68）。
    if pushed && root.declared_height_auto {
        let float_bottom = root
            .children
            .iter()
            .filter(|c| is_float(c))
            .fold(0.0f32, |my, f| my.max(f.y + f.height));
        let in_flow_bottom = root
            .children
            .iter()
            .filter(|c| is_in_flow(c))
            .fold(0.0f32, |my, c| my.max(c.y + c.height));
        let new_content_h = float_bottom.max(in_flow_bottom);
        let pb = root.padding_top + root.padding_bottom + root.border_top + root.border_bottom;
        if (new_content_h - root.content_height).abs() > 0.5 {
            if dbg {
                eprintln!(
                    "ZW_TABLE_FLOAT_DBG D container content_h: {} -> {} (fb={} ifb={})",
                    root.content_height, new_content_h, float_bottom, in_flow_bottom
                );
            }
            let old_content_height = root.content_height;
            root.content_height = new_content_h;
            root.height = new_content_h + pb;
            // R5213：记录被 D 步扩高的容器（常为 td/cell）高度增量，供
            // propagate_grown_cell_heights 上传行/行组/表高并位移后继兄弟。
            if let Some(id) = root.node_id {
                grown_cells.insert(id, root.content_height - old_content_height);
            }
        }
    }
}
