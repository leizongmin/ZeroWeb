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
use crate::table::layout_table;
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
    let mut grown_cell_ids: Vec<NodeId> = Vec::new();
    fix_inner(root, doc, styles, &mut grown_cell_ids, inline_fonts);
    // R1723 eval：D 步扩高的 td（cell）需把增量传到外层 table 行高，否则下推的 table 溢出
    // 被后续块覆盖（floats-wrap-bfc-005）。曾尝试 height-only reflow（reflow_tables_with_grown_cells），
    // 但 bfc-004 是「假通过」案——test table 与 ref purple 经 R1612 line-advance 都落在 y=20，
    // 但 ref 容器 height:20（definite，不增长）vs test 外层 table auto-height；reflow 让 test
    // 外层 table 长高 → 与 ref 不再匹配 → bfc-004 0.42%→8.31% 回归（bfc-005 0.09% flip，net 0）。
    // bfc-005（ref height:40，须长高）与 bfc-004（ref height:20，不须长高）对外层 table 高度
    // 需求相反，R1612 line-advance 又把两者 table 都放到 y=20 → reflow 无法区分 → 暂搁置，
    // 待 R1612 line-advance 改 min-bottom 步进（须重核 floats-placement 簇）或外层 table 高度
    // 传播与 line-advance 解耦后再做。grown_cell_ids 收集保留供未来 reflow 复用。
    let _ = grown_cell_ids;
}

fn is_table_box(b: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) -> bool {
    // 仅 in-flow table（非 float）才算 §9.5 推开候选。float 的 table（如
    // float-page-break-inside-avoid-1-print 的 <table class="test" float:left>）本身是
    // 浮动元素，由 float 定位处理，不应被当作 in-flow BFC 做 §9.5 推开。
    if is_float(b) {
        return false;
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

fn fix_inner(
    root: &mut LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    grown_cell_ids: &mut Vec<NodeId>,
    inline_fonts: crate::inline_finalization::InlineFontContext<'_>,
) {
    // post-order：先修子容器（嵌套结构）
    for child in &mut root.children {
        fix_inner(child, doc, styles, grown_cell_ids, inline_fonts);
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
    adjust_float_positions(&mut root.children[tidx]);

    // === B: 重算 table 高度（inner float 堆叠后 table h 增长，如 100→200）===
    layout_table(&mut root.children[tidx], doc, styles, inline_fonts);

    // 先用不可变读取计算 natural_y / avoidance_x / is_cleared（避免与下方可变借用冲突）
    let table_h = root.children[tidx].height;
    let table_w = root.children[tidx].width;
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
        .and_then(|s| match &s.width {
            LengthValue::Px(v) => Some(*v as f32),
            LengthValue::Percentage(p) => Some((*p as f32 / 100.0) * content_width),
            _ => None,
        });
    let effective_w = declared_w.unwrap_or(table_w);
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
    // R1721：float:right avoidance —— table 避到 float 左侧（mirror of 既有 float:left 右避）。
    // 既有 C 算法仅 float:left（table 放 float 右侧 content_width-max_right）；float:right 的
    // right_edge≈content_width → 右侧无空间 → table 错误推 below（应 beside 左 x=0 w=float.left）。
    // 仅纯右 float（natural_y 处有重叠右 float 且无重叠左 float）触发；混合 fall through 左 float 逻辑。
    // kill-switch ZW_TABLE_FLOAT_RIGHT_AVOID=0 关闭（default-on）。
    let right_float_left: Option<f32> = {
        let mut rl: Option<f32> = None;
        for f in &root.children {
            if matches!(f.float, FloatValue::Right) && natural_y < f.y + f.height && natural_y + table_h > f.y {
                rl = Some(rl.map_or(f.x, |m: f32| m.min(f.x)));
            }
        }
        rl
    };
    let has_left_overlap = root
        .children
        .iter()
        .any(|c| matches!(c.float, FloatValue::Left) && natural_y < c.y + c.height && natural_y + table_h > c.y);
    // target = (nx, ny, fill_w)：fill_w = table 应填的 avoidance 宽度（beside 时填，below 时仅 clamp）
    let right_target: Option<(f32, f32, f32)> =
        if std::env::var("ZW_TABLE_FLOAT_RIGHT_AVOID").as_deref() != Ok("0") && !is_cleared && !has_left_overlap {
            // 纯右 float：table beside 左侧 x=0 y=natural_y，填到右 float 左边。
            // R1723：仅当 table 放得进左可用宽（right_float_left）时 beside；definite-width table
            // 声明宽 > 可用宽（floats-wrap-bfc-005 子案 2：150 > 100）→ 不 beside，fall through 到
            // 下方 left-float 算法推下（mirror 子案 1）。
            right_float_left.and_then(|rl| {
                if effective_w <= rl + 0.5 {
                    Some((0.0, natural_y, rl))
                } else {
                    None
                }
            })
        } else {
            None
        };
    let target: Option<(f32, f32, f32)> = if right_target.is_some() {
        right_target
    } else if !is_cleared && avoidance_x > 0.5 {
        // 既有 float:left 算法（target (placed_x, y)），fill_w = content_width - placed_x。
        let frects: Vec<(f32, f32, f32)> = root
            .children
            .iter()
            .filter(|c| is_float(c))
            .map(|f| (f.x + f.width + f.margin_right, f.y, f.y + f.height))
            .collect();
        let mut y = natural_y;
        let mut placed_x = 0.0;
        let mut found = false;
        for _ in 0..64 {
            let max_right = frects
                .iter()
                .filter(|(_, ft, fb)| *ft < y + table_h && y < *fb)
                .map(|(r, _, _)| *r)
                .fold(0.0f32, |mx, r| mx.max(r));
            if effective_w <= (content_width - max_right).max(0.0) + 0.5 {
                placed_x = max_right;
                found = true;
                break;
            }
            // 放不下 → 推到「同行」float 中最晚结束的 bottom（MAX-bottom，仅 top<=y 的 float）。
            // R1612：匹配 float_positioning 的 line-advance——float 不 fit 当前行时整行下移到
            // 当前行 max float bottom（非 strict-CSS 最早 beside-fit y）。R1611 root-cause：ref 用
            // float_positioning（line-advance），block1 purple 在 blue(底20)+silver(底6) 同行
            // → max-bottom 20，故 purple y=20（abs 28）非 silver 底 6。「同行」filter（top<=y）
            // 排除 clear 致的后继行 float（如 float2 clear:left top=10 在 y=0 行不算）→ table
            // 推到 float1 底后在 float2 行 beside-fit，匹配 float_positioning。
            let max_bottom = frects
                .iter()
                .filter(|(_, ft, fb)| *ft <= y + 0.5 && *ft < y + table_h && y < *fb)
                .map(|(_, _, fb)| *fb)
                .fold(0.0f32, |mx, fb| mx.max(fb));
            if max_bottom > y {
                y = max_bottom;
            } else {
                placed_x = 0.0;
                found = true;
                break;
            }
        }
        if found {
            Some((placed_x, y, (content_width - placed_x).max(0.0)))
        } else {
            None
        }
    } else {
        None
    };
    if let Some((nx, ny, fill_w)) = target {
        let table = &mut root.children[tidx];
        let mw = fill_w;
        // beside float：auto-width BFC table 填可用 avoidance 宽度（非 shrink-to-fit 内容宽）
        // ——floats-wrap-bfc-002-left-table（float:left，nx>0，mw=content_width-nx）/ R1721 -right-table
        //（float:right，nx=0，mw=right_float.left）auto table 应 200 非 150，匹配 ref 显式 width:200。
        // below floats（nx=0 且 fill_w≈content_width）：保持 shrink-to-fit，仅 clamp 不溢出。须在
        // 「位置变 OR 宽度变」时介入（即使位置已正确，beside 宽度仍须填，故 width_change 独立于 pos_change）。
        // R1721：beside 判定加 `fill_w < content_width` 捕获 float:right（nx=0 但填部分宽）。
        let beside = nx > 0.5 || fill_w < content_width - 0.5;
        let pos_change = (nx - table.x).abs() > 0.5 || (ny - table.y).abs() > 0.5;
        let width_change = if beside {
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
            if beside || table.width > mw {
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
            root.content_height = new_content_h;
            root.height = new_content_h + pb;
            // R1723：记录被 D 步扩高的容器 node_id（常为 td/cell），供后续
            // reflow_tables_with_grown_cells 重算其外层 table 行高。
            if let Some(id) = root.node_id {
                grown_cell_ids.push(id);
            }
        }
    }
}
