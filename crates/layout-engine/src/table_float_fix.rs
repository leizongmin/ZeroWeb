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

use zero_css_parser::values::{ClearValue, DisplayValue, FloatValue};
use zero_dom::{Document, NodeId};
use zero_style_system::ComputedStyle;

use crate::float_positioning::adjust_float_positions;
use crate::table::layout_table;
use crate::types::LayoutBox;

/// table-among-floats scoped iterative fix 入口。post-order 遍历，对匹配结构的容器执行
/// A+B+C+D。env `ZW_TABLE_FLOAT_ITER_V2=0` 关闭。
pub(crate) fn fix_table_among_floats(root: &mut LayoutBox, doc: &Document, styles: &HashMap<NodeId, ComputedStyle>) {
    if std::env::var("ZW_TABLE_FLOAT_ITER_V2").as_deref() == Ok("0") {
        return;
    }
    fix_inner(root, doc, styles);
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

fn fix_inner(root: &mut LayoutBox, doc: &Document, styles: &HashMap<NodeId, ComputedStyle>) {
    // post-order：先修子容器（嵌套结构）
    for child in &mut root.children {
        fix_inner(child, doc, styles);
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
    layout_table(&mut root.children[tidx], doc, styles);

    // 先用不可变读取计算 natural_y / avoidance_x / is_cleared（避免与下方可变借用冲突）
    let table_h = root.children[tidx].height;
    let table_w = root.children[tidx].width;
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
    let max_w = (content_width - avoidance_x).max(0.0);

    // === C: 手动 §9.5 push（仅当 table 非 cleared、能放进 float 右侧空间、且确有 float 须避开）===
    // §9.5 触发条件 = avoidance_x > table.x（左方有须避开的 float）。仅此时才把 table 推到
    // float 右侧并重置 Y 到 natural_y（table 此前被 taffy 当全宽堆到 float 下方）。
    // 无 float 须避开（avoidance_x==0，如 blocks-025 table 在正常流、float 不重叠）则不动 table。
    let table = &mut root.children[tidx];
    let fits = table_w <= max_w + 0.5;
    let need_push = !is_cleared && fits && avoidance_x > table.x + 0.5;
    let mut pushed = false;
    if need_push {
        let (old_x, old_y) = (table.x, table.y);
        table.x = avoidance_x;
        table.y = natural_y;
        if table.width > max_w {
            table.width = max_w;
        }
        pushed = true;
        if dbg {
            eprintln!(
                "ZW_TABLE_FLOAT_DBG C push table: ({},{}) -> ({},{}) w={} avoid={} naty={} maxw={}",
                old_x, old_y, table.x, table.y, table.width, avoidance_x, natural_y, max_w
            );
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
        }
    }
}
