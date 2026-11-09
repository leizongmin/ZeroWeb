//! `ZW_IB_DESTRETCH=0` kill-switch 文档语义钉（tv-2 复核 F-1 闭环）：「回退 outer_w
//! 旧行为」= intrinsic_sizing 两臂（destretch 递归臂 + 纯文本臂）整体让位，原子回落
//! generic outer_w + 裸文本 walk 旧口径——R54xx 前真实基线，非混合态。区分力语境：
//! 简单 max-content 语境两路径数值收敛（裸 walk 计原子内文本），须 child.width 被
//! taffy 同步拉伸的反馈语境（summary 形）才分流——kill-switch 关闭后 summary 塌回
//! Block 堆叠 max(span,caret) 腐化形（R54xx default-on 的定谳依据，r1 诊断在案：
//! laid 26px、真实 168px）。默认路径（destretch 开）由 flex_item_base_authority
//! 独立进程钉住。单文件单测：进程内 set_var 会与并行单测互踩（R3858 教训）。

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

/// flex_item_base_authority 同形单条目（BASE_CSS/FRAG 子集，见该文件）。
const HTML: &str = "<!doctype html><html><head><style>\
body{margin:0}\
.rgt{display:flex;align-items:center;justify-content:flex-end}\
summary.btn-link{font-size:inherit;white-space:nowrap;background-color:#0000;border:0;padding:0;display:inline-block}\
.details-reset>summary{list-style:none}.details-reset>summary:before{display:none}\
.text-bold{font-weight:600}\
.select-menu-button:after{vertical-align:-2px;content:\"\";border:2px solid;border-bottom-color:#0000;border-left-color:#0000;border-right-color:#0000;width:0;height:0;display:inline-block}\
</style></head><body>\
<div class=\"rgt\"><div><details class=\"details-reset\"><summary class=\"select-menu-button btn-link\">Spoken Language: <span class=\"text-bold\">Any</span></summary></details></div></div>\
</body></html>";

/// 旧形断言 = flex_item_base_authority 健康判据（summary 右缘盖过 span 右缘+caret）
/// 的取反：两臂让位后匿名文本/伪元素不计入 basis，summary 塌 Block 堆叠，caret 越界。
#[test]
fn destretch_zero_restores_pre_r54xx_collapsed_form() {
    // Rust 2024：set_var/remove_var 为 unsafe（进程全局可变）。ZW_IB_DESTRETCH 在
    // intrinsic_sizing 逐臂 inline 读取（非 LazyLock），测量前设置即生效。
    unsafe { std::env::set_var("ZW_IB_DESTRETCH", "0") };
    let mut p = RenderPipeline::new(1280.0, 800.0);
    let r = p.render_html(HTML, "");
    let doc_rc = p.cached_doc_shared().expect("cached doc after render_html");
    let doc = doc_rc.borrow();
    let sid = doc.query_selector(doc.root(), "summary").expect("summary 存在");
    let (sb, sx, _) = class_box(&r.layout.root, &doc, "summary").expect("summary box");
    let span_id = doc.query_selector(sid, "span").expect("span 子");
    let (spb, spx, _) = walk_abs(&r.layout.root, span_id, 0.0, 0.0).expect("span box");
    let span_right = spx + spb.width;
    let summary_right = sx + sb.width;
    assert!(
        summary_right + TOL < span_right + 2.0,
        "destretch=0 须回退 R54xx 前旧形：summary 塌 Block 堆叠（右缘 {} 应窄于 span 右缘 \
         {} + caret，即匿名文本/伪元素不计入 basis）；健康形说明 kill-switch 失效",
        summary_right,
        span_right
    );
}
