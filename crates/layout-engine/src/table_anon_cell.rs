//! R5168（table.rs 减负：2553 行已超 2000 行限）。CSS2 §17.2.1 匿名 table-cell 合成。
//!
//! **规范**：CSS2.1 §17.2.1——table-row 的子元素中，非 table-cell 的连续 inline 级内容
//!（inline 元素 + 裸文本）应包裹进一个匿名 table-cell 盒，作为该行的单个 cell 参与
//! grid（占一列），而非各自成为独立 cell / 被跳过。
//!
//! **现状缺陷**（table-anonymous-objects-088/091/092，R5167 字号外露后 1.05-1.73%）：
//! `build_row`（table_types.rs）无 display 过滤，把行内每个 inline 子当独立 cell——
//! 4 个裸 span 占 4 列（ref 为 [匿名 cell, td, td] 3 列）；且裸 span 盒几何是 taffy
//! 交换帧 junk（x 堆叠/h=37/ch=0），文本按 junk 位渲染。
//!
//! **机制**：在 `adjust_table_layout` 入口（一切 `layout_table` 之前）全树走一遍，
//! 对每个 display:table-row 盒，把连续 inline 级子（display:Inline 元素 +
//! `is_anonymous_text_item` 文本项）drain 进一个合成 cell 盒插回原位：
//! - `node_id` = 宿主行的（R109 block-mixed 片段盒同款「node_id=宿主」先例）——
//!   compute_final/paint 的样式、strut、Path B 锚全走行节点（继承属性与 cell 同源）；
//! - `fragment_node_ids` = 被吞并子的 DOM id（R109 §9.2.1.1 片段收集机制现成接线：
//!   inline/collect_items.rs 按 fragments 圈定收集范围，paint Path B text.rs:1508 同源）；
//! - `is_block_level` = true（compute_final 早退门放行，extract 对 anon 片段的强制
//!   同款，engine.rs）；`is_anon_table_cell` = true（语义旗标，下游一切判定只认旗标
//!   不认 node_id 的行样式）。
//!
//! 行高由显式 cell 撑起时（088 全部三行）合成 cell 高度经 `row_height.max(...)`
//! 拉齐；纯匿名 cell 行取子盒 taffy 度量（max 底），精度挂账。
//!
//! kill-switch：`ZW_ANON_CELL_WRAP=0` 完整回退本行为集（合成 pass + build_row DOM
//! 遍历的网格列预留一并关闭；与存储臂 `ZW_CELL_IFC_STORE` 分立，供 ON/OFF 归因）。

use std::collections::HashMap;

use zero_css_parser::values::{DisplayValue, FloatValue};

use zero_dom::{Document, NodeId};
use zero_style_system::ComputedStyle;

use crate::types::LayoutBox;

/// 合成开关（default-on；`ZW_ANON_CELL_WRAP=0` 回退 pre-R5168 行为）。
/// LazyLock 构造期单读（R3858 教训：进程级快照，进程内并行测试不可 set_var 翻转，
/// kill-switch 判别经外部 env 运行验证）。
static ANON_CELL_WRAP_ON: std::sync::LazyLock<bool> =
    std::sync::LazyLock::new(|| std::env::var("ZW_ANON_CELL_WRAP").as_deref() != Ok("0"));

pub(crate) fn anon_cell_wrap_enabled() -> bool {
    *ANON_CELL_WRAP_ON
}

/// 全树合成：对每个 display:table-row 盒，把连续 inline 级子包裹为匿名 cell。
pub(crate) fn synthesize_anon_table_cells(
    root: &mut LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    // 本盒是 table-row：先合成其子。合成 cell 借宿主行 node_id（样式查表会返回行
    // 的 TableRow）——必须按 is_anon_table_cell 旗标排除，否则把 cell 的 inline 子
    // 再合成为 cell → 无限递归（首跑 stack overflow 实证）。
    let is_row = !root.is_anon_table_cell
        && root
            .node_id
            .and_then(|id| styles.get(&id))
            .is_some_and(|s| matches!(s.display, DisplayValue::TableRow));
    if is_row {
        // R5168 水平域门：vertical 行的合成/网格预留未对接（R4470 bare-row 臂自有
        // tuned 语义；首轮 corpus inline-table-alignment-002..005 7.9-10.1% 翻红实证）
        // —— vertical 维持 pre-R5168 行为。
        // FIXME(R5168 后继)：vertical 表匿名 cell 合成与网格预留专项。
        if !root.writing_mode.is_vertical_block_flow() {
            wrap_inline_runs_in_row(root, doc, styles);
        }
    }
    for child in &mut root.children {
        synthesize_anon_table_cells(child, doc, styles);
    }
}

/// 判子盒是否 run 成员：inline 元素或匿名文本项。OOF（float/abs/fixed）不是——
/// 脱离正常流，留在行盒（build_row 已跳过它们，行为不变）。
fn is_run_member(child: &LayoutBox, doc: &Document, styles: &HashMap<NodeId, ComputedStyle>) -> bool {
    if !matches!(child.float, FloatValue::None) || child.is_absolute || child.is_fixed {
        return false;
    }
    if child.is_anonymous_text_item {
        // 纯空白文本项不入 run（表上下文空白序列不生成匿名 cell，§17.2.1）。
        let Some(nid) = child.node_id else {
            return false;
        };
        return doc
            .get(nid)
            .is_some_and(|n| matches!(&n.kind, zero_dom::NodeKind::Text(t) if !t.content.trim().is_empty()));
    }
    // inline-block/inline-table/inline-flex 等原子 inline 按 §17.2.1 应进匿名 cell，
    // 但其盒尺寸/绘制路径（原子 inline 度量注入、margin 盒）与本切片未对接——
    // FIXME(R5168 后继)：作为 run 断开器留在行盒（现状：被 build_row 误当 cell，不较前更差）。
    child
        .node_id
        .and_then(|id| styles.get(&id))
        .is_some_and(|s| matches!(s.display, DisplayValue::Inline))
}

/// 对单个行盒执行 run 包裹。run 长度 ≥1 即合成（§17.2.1：单个 inline 子也属匿名 cell）。
fn wrap_inline_runs_in_row(row: &mut LayoutBox, doc: &Document, styles: &HashMap<NodeId, ComputedStyle>) {
    let Some(row_node_id) = row.node_id else {
        return; // 无 DOM 身份的行（merge_orphan 包装内的行均为 node-ful，防御）
    };
    let writing_mode = row.writing_mode.clone();

    let mut idx = 0usize;
    while idx < row.children.len() {
        if !is_run_member(&row.children[idx], doc, styles) {
            idx += 1;
            continue;
        }
        let run_start = idx;
        while idx < row.children.len() && is_run_member(&row.children[idx], doc, styles) {
            idx += 1;
        }
        let run_len = idx - run_start;
        let drained: Vec<LayoutBox> = row.children.drain(run_start..run_start + run_len).collect();
        // 片段成员表：被吞并子的 DOM id（元素或文本节点），供 IFC 收集/Path B 圈定。
        let fragment_ids: Vec<NodeId> = drained.iter().filter_map(|c| c.node_id).collect();
        // cell 高度取子盒 taffy 度量的 max 底（子 y 归零后即 max 高度）。
        // 行高若由同 row 的显式 cell 撑起，position_cells 的 row_height.max(...) 会
        // 把本 cell 拉到行高，此估值仅影响纯匿名 cell 行。
        let content_bottom = drained
            .iter()
            .filter(|c| !c.is_absolute && !c.is_fixed)
            .map(|c| c.height + c.margin_top + c.margin_bottom)
            .fold(0.0f32, f32::max);
        let cell = LayoutBox {
            node_id: Some(row_node_id),
            fragment_node_ids: Some(fragment_ids),
            is_block_level: true,
            is_anon_table_cell: true,
            writing_mode: writing_mode.clone(),
            // 子盒几何归零：taffy 交换帧 junk（x 列状堆叠/y 散布）不携带进 cell；
            // 文本位由 cell 的 IFC（stored 或 Path B）重排，盒位仅余 content 高度
            // 走查（cell_float_aware_content_height）消费。
            children: drained
                .into_iter()
                .map(|mut c| {
                    c.x = 0.0;
                    c.y = 0.0;
                    c
                })
                .collect(),
            height: content_bottom,
            content_height: content_bottom,
            ..Default::default()
        };
        row.children.insert(run_start, cell);
        idx = run_start + 1;
    }

    // R5169：盒级 drain 收不掉 R2160 跳过的无盒 run（shape A）——DOM 级补空合成 cell。
    wrap_boxless_runs_in_row(row, doc, styles);
}

/// R5169：无盒 inline run 的空合成 cell（shape A 补绘）。
///
/// 裸 inline 子经 R2160 multi-inline 跳过在 step 8 **无盒**（088 形态），盒级 drain
/// 看不见它们 → 行只有占位 entry（usize::MAX），网格列已预留但 **run 文本无盒可绘**
/// （行级 paint_text 被 has_direct_paintable_text 的 TableCell 块级子判据早退；行级
/// IFC 即便放行也是整行串联，非逐列定位）。本 pass 在 DOM 级按 build_row 同款分组
/// 语义（非空白文本 + display:Inline 元素为成员；OOF 断 run；其余元素断 run 并以其
/// 盒为插入锚）补出**空合成 cell 盒**：`node_id` 借宿主行 + `fragment_node_ids`
/// 圈定 run 成员——paint 侧 fragment 圈定收集（R109 §9.2.1.1 机制，collect_items）
/// 与 position_cells 真盒定位（x/width=列几何）全部现成，run 文本即按列绘制
///（088/092 直接受益）。
///
/// 幂等：row 子已存在 fragment 相交的合成 cell（shape B drain 产物或既往 pass）
/// 即跳过，不重复建盒。
fn wrap_boxless_runs_in_row(row: &mut LayoutBox, doc: &Document, styles: &HashMap<NodeId, ComputedStyle>) {
    let Some(row_node_id) = row.node_id else {
        return;
    };
    // 分组语义与 build_row（table_types.rs DOM 序遍历）一致，保证 fragment 相交
    // 匹配成立：text 一律入 run_ids（has_content 只计非空白）；display:Inline 元素
    // 入 run；OOF 断 run（不入）；其余元素断 run，其盒为插入锚。
    let mut runs: Vec<(Option<NodeId>, Vec<NodeId>)> = Vec::new();
    let mut run_ids: Vec<NodeId> = Vec::new();
    let mut run_has_content = false;
    let flush = |runs: &mut Vec<(Option<NodeId>, Vec<NodeId>)>,
                 run_ids: &mut Vec<NodeId>,
                 has_content: bool,
                 anchor: Option<NodeId>| {
        if has_content {
            runs.push((anchor, std::mem::take(run_ids)));
        }
        run_ids.clear();
    };
    for dom_child in doc.child_nodes(row_node_id) {
        let Some(node) = doc.get(dom_child) else {
            continue;
        };
        match &node.kind {
            zero_dom::NodeKind::Text(t) => {
                run_ids.push(dom_child);
                run_has_content |= !t.content.trim().is_empty();
            }
            zero_dom::NodeKind::Element(_) => {
                let style = styles.get(&dom_child);
                let oof = style.is_some_and(|s| {
                    !matches!(s.float, FloatValue::None)
                        || matches!(
                            s.position,
                            zero_style_system::property::types::PositionValue::Absolute
                                | zero_style_system::property::types::PositionValue::Fixed
                        )
                });
                if oof {
                    flush(&mut runs, &mut run_ids, run_has_content, None);
                    run_has_content = false;
                    continue;
                }
                if matches!(style.map(|s| &s.display), Some(DisplayValue::Inline)) {
                    run_ids.push(dom_child);
                    run_has_content = true; // inline 元素（即使空）也构成匿名 cell 内容
                    continue;
                }
                flush(&mut runs, &mut run_ids, run_has_content, Some(dom_child));
                run_has_content = false;
            }
            _ => {}
        }
    }
    flush(&mut runs, &mut run_ids, run_has_content, None);

    let writing_mode = row.writing_mode.clone();
    for (anchor, run) in runs {
        // R5169：仅合成员含 inline **元素**的 run——纯文本 run（孤儿 table-row 直系裸
        // 文本、flex 容器 table-row 伪元素内容）旧有绘制路径（行级直绘）本就正确，空
        // 合成 cell 以 0 高/未解析列宽接管绘制反而错位（run-in-table-row-between-001/
        // 002 3.17%、flexbox-with-pseudo-elements-003 2.26% 实证）。boxless 合成只修
        // R2160 无盒 inline 元素（088/092 形态）。
        // FIXME(R5169 后继)：纯文本 run 的匿名 cell 合成须连同行级直绘让位一起对接。
        if !run.iter().any(|id| {
            styles
                .get(id)
                .is_some_and(|s| matches!(s.display, DisplayValue::Inline))
        }) {
            continue;
        }
        // 幂等 + shape B 让位：已有合成 cell 圈定本 run（任一成员相交）即跳过。
        if row.children.iter().any(|b| {
            b.is_anon_table_cell
                && b.fragment_node_ids
                    .as_ref()
                    .is_some_and(|frags| run.iter().any(|id| frags.contains(id)))
        }) {
            continue;
        }
        let cell = LayoutBox {
            node_id: Some(row_node_id),
            fragment_node_ids: Some(run),
            is_block_level: true,
            is_anon_table_cell: true,
            writing_mode: writing_mode.clone(),
            // 无子盒（run 成员本就无盒）：高度 0，行高由显式 cell 撑起
            //（position_cells row_height.max）；纯匿名 cell 行行高仍挂账。
            height: 0.0,
            content_height: 0.0,
            ..Default::default()
        };
        // 插入锚：run 之后的首个非 inline DOM 兄弟的盒位（保 DOM 序）；无锚（行尾
        // run）追加行末。锚盒缺失（未盒化）时退化为追加。
        let insert_at = anchor.and_then(|a| row.children.iter().position(|b| b.node_id == Some(a)));
        match insert_at {
            Some(i) => row.children.insert(i, cell),
            None => row.children.push(cell),
        }
    }
}
