//! row/nowrap flex auto 条目的内容基准权威回归（trending 头部右下拉区 live 判别
//! 实验收敛的常驻 fixture）。
//!
//! 根因：converter 把 inline 级元素映射为 taffy Block（R109），summary 因原子 caret
//! 伪元素盒化成**非叶** taffy 节点 → taffy 首趟内容计量按 Block 堆叠取 max(子)
//! （span 26、caret 4 → 26），匿名文本（"Spoken Language: " 138px）不在 taffy 树
//! 中不可见 → row flex auto item basis 被解成 span 宽，summary 塌缩为 span 大小、
//! caret 伪元素画到盒外。修复面：
//! - `layout-engine/src/engine/sizing.rs::apply_flex_item_base_authority`（R54xx）：
//!   混合内容条目以 `flex_item_base_size` ZW 基准写回 taffy flex_basis，
//!   kill-switch `ZW_FLEX_BASE_AUTHORITY=0`
//! - `layout-engine/src/intrinsic_sizing.rs` R4032 destretch 臂 + inline-block 族
//!   else 臂段核算（`ZW_IB_DESTRETCH=0` 仅回退递归取值，段核算独立生效）
//!
//! 断言全部用结构判据（summary 右缘盖过 span 右缘 + caret；V12 无 caret 形
//! fresh==heal 逐盒相等），不冻结文本 advance 数字，跨字体环境稳健。

use zero_engine::RenderPipeline;
use zero_layout_engine::LayoutBox;

fn walk_abs(box_node: &LayoutBox, target: zero_dom::NodeId, ox: f32, oy: f32) -> Option<(&LayoutBox, f32, f32)> {
    let ax = ox + box_node.x;
    let ay = oy + box_node.y;
    if box_node.node_id == Some(target) {
        return Some((box_node, ax, ay));
    }
    box_node.children.iter().find_map(|c| walk_abs(c, target, ax, ay))
}

fn class_box<'a>(root: &'a LayoutBox, doc: &zero_dom::Document, selector: &str) -> Option<(&'a LayoutBox, f32, f32)> {
    let id = doc.query_selector(doc.root(), selector)?;
    walk_abs(root, id, 0.0, 0.0)
}

const TOL: f32 = 0.5;

const BASE_CSS: &str = "body{margin:0}\
    .hdr{display:flex;justify-content:space-between;align-items:center}\
    .rgt{display:flex;align-items:center;justify-content:flex-end}\
    summary.btn-link{font-size:inherit;white-space:nowrap;background-color:#0000;border:0;padding:0;display:inline-block}\
    .details-reset>summary{list-style:none}.details-reset>summary:before{display:none}\
    .text-bold{font-weight:600}\
    .select-menu-button:after{vertical-align:-2px;content:\"\";border:2px solid;border-bottom-color:#0000;border-left-color:#0000;border-right-color:#0000;width:0;height:0;display:inline-block}";

const FRAG: &str = "<div class=\"hdr\"><div class=\"rgt\">\
     <div><details class=\"details-reset\"><summary class=\"select-menu-button btn-link\">Spoken Language: <span class=\"text-bold\">Any</span></summary></details></div>\
     <div><details class=\"details-reset\"><summary class=\"select-menu-button btn-link\">Language: <span class=\"text-bold\">Any</span></summary></details></div>\
     <div><details class=\"details-reset\"><summary class=\"select-menu-button btn-link\">Stars: <span class=\"text-bold\">100+</span></summary></details></div>\
     </div></div>";

/// R54xx 主形（V8）：带 caret 伪元素的 summary 条目必须按内容计量——summary 右缘
/// 盖过 span 右缘 + caret（匿名文本参与 basis）。旧缺陷形 summary==span 宽、caret
/// 画到盒外（断言即红）。kill-switch 复红证据：
/// `ZW_FLEX_BASE_AUTHORITY=0 cargo test ... flex_item_base_authority`（summary 塌
/// 回 Block 堆叠 max(span,caret)，caret 越界）。
#[test]
fn summary_row_flex_item_sized_by_content() {
    let html = format!("<!doctype html><html><head><style>{BASE_CSS}</style></head><body>{FRAG}</body></html>");
    let mut p = RenderPipeline::new(1280.0, 800.0);
    let r = p.render_html(&html, "");
    let doc_rc = p.cached_doc_shared().expect("cached doc after render_html");
    let doc = doc_rc.borrow();
    // 逐条目：span 右缘 + caret(≥2) 不得越出 summary 右缘（匿名文本须计入 basis）。
    let summaries = doc.query_selector_all(doc.root(), "summary");
    assert!(summaries.len() >= 3, "应有 3 个 summary 条目，实际 {}", summaries.len());
    for (i, sid) in summaries.iter().enumerate() {
        let (sb, sx, _) = walk_abs(&r.layout.root, *sid, 0.0, 0.0).expect("summary box");
        let span_id = doc
            .query_selector(*sid, "span")
            .unwrap_or_else(|| panic!("summary[{i}] 缺 span 子"));
        let (spb, spx, _) = walk_abs(&r.layout.root, span_id, 0.0, 0.0).expect("span box");
        let span_right = spx + spb.width;
        let summary_right = sx + sb.width;
        assert!(
            summary_right + TOL >= span_right + 2.0,
            "summary[{i}] 右缘 {} 未盖过 span 右缘 {} + caret：匿名文本/伪元素未计入 \
             flex basis（R54xx 回归面）",
            summary_right,
            span_right
        );
    }
}

/// 增量一致性（V12 无 caret 形）：同管线增量重算（`repaint_cached_viewport`，I 臂
/// 口径）后 .rgt 与各 summary 的盒几何须与全量渲染逐盒相等——增量 relayout 与全量
/// 路径同源收敛（trending stale 几何任务的验收判据；caret 伪元素 fresh/incremental
/// 物化不对称属 K4b 挂账，故用无 caret 形钉一致性问题本身）。
#[test]
fn incremental_relayout_matches_full_for_summary_row() {
    // 独立后置规则按级联序压过 base 的 content:""（同声明块内前置会被同块
    // content:"" 覆盖——CSS Cascading §4 声明序）。
    let v12 = format!("{BASE_CSS} .select-menu-button:after{{content:none}}");
    let html = format!("<!doctype html><html><head><style>{v12}</style></head><body>{FRAG}</body></html>");
    let mut p = RenderPipeline::new(1280.0, 800.0);
    let r = p.render_html(&html, "");
    let r2 = p
        .repaint_cached_viewport("")
        .expect("incremental relayout should produce a frame");
    let doc_rc = p.cached_doc_shared().expect("cached doc");
    let doc = doc_rc.borrow();
    for (tag, layout_root) in [("fresh", &r.layout.root), ("heal", &r2.layout.root)] {
        let (b, x, y) = class_box(layout_root, &doc, ".rgt").expect(".rgt box");
        eprintln!("[{tag}] .rgt ({x},{y}) {}x{}", b.width, b.height);
    }
    let (fb, fx, fy) = class_box(&r.layout.root, &doc, ".rgt").expect(".rgt fresh");
    let (hb, hx, hy) = class_box(&r2.layout.root, &doc, ".rgt").expect(".rgt heal");
    assert!(
        (fb.width - hb.width).abs() < TOL
            && (fb.height - hb.height).abs() < TOL
            && (fx - hx).abs() < TOL
            && (fy - hy).abs() < TOL,
        ".rgt 增量重算与全量渲染分歧：fresh {}x{}@({fx},{fy}) vs heal {}x{}@({hx},{hy})",
        fb.width,
        fb.height,
        hb.width,
        hb.height
    );
    let summaries = doc.query_selector_all(doc.root(), "summary");
    for (i, sid) in summaries.iter().enumerate() {
        let (sb1, sx1, sy1) = walk_abs(&r.layout.root, *sid, 0.0, 0.0).expect("fresh summary");
        let (sb2, sx2, sy2) = walk_abs(&r2.layout.root, *sid, 0.0, 0.0).expect("heal summary");
        assert!(
            (sb1.width - sb2.width).abs() < TOL
                && (sb1.height - sb2.height).abs() < TOL
                && (sx1 - sx2).abs() < TOL
                && (sy1 - sy2).abs() < TOL,
            "summary[{i}] 增量重算 stale：fresh {}x{}@({sx1},{sy1}) vs heal {}x{}@({sx2},{sy2})",
            sb1.width,
            sb1.height,
            sb2.width,
            sb2.height
        );
    }
}
