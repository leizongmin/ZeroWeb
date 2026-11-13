//! R5517（CSS2 §10.3.5 × §10.4）：纯文本 float shrink-to-fit 收缩值过自身 definite
//! min/max-width 交叉约束。纯文本测量（text_content_max_width/Σ walk）不带 R5368
//! 自身 min-width 地板/max cap 语义（那内嵌在 block_min/max_content_width 测量臂），
//! 收缩臂把 converter taffy min_size 解出的地板宽覆盖回词宽——驱动案
//! css-sizing/box-sizing-content-box-003 ref 腿（.box-sized min-width:250px;
//! max-width:350px; float:left，双 float 并排应填满 500px 容器）：ZW 旧态两 float
//! 收缩到词宽 ~9.8px 叠红底。修复：收缩写入前经
//! [`crate::intrinsic_sizing::clamp_used_width_cross_constraints`]（先 cap 后 floor
//! ＝ §10.4 min 胳 max 序）。文本度量接固定 10px/字符 advance source（engine 测试
//! 不装 shaping/resolver 恒 Legacy＝锚点空转教训，r1242 同款接线），锚值全确定。

use crate::types::LayoutBox;
use crate::{LayoutResult, inline::AdvanceSource};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use zero_dom::{Document, NodeKind};
use zero_style_system::FontSizeAdjustValue;

/// 固定 10px/字符的 advance source（字数 × 10 = 文本宽，与系统字体无关）。
struct CharAdvance10;

impl AdvanceSource for CharAdvance10 {
    fn measure(&self, _ch: char, _font_id: Option<u32>, _font_size: f32, _is_ahem: bool) -> f32 {
        10.0
    }

    fn measure_text_with_font_context(
        &self,
        text: &str,
        _font_ids: &[u32],
        _font_size: f32,
        _is_ahem: bool,
        _size_adjust: &FontSizeAdjustValue,
        _variations: &zero_style_system::FontVariationSettingsValue,
    ) -> f32 {
        text.chars().count() as f32 * 10.0
    }
}

fn layout(css: &str, html: &str) -> LayoutResult {
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    engine.set_advance_source(Rc::new(CharAdvance10));
    engine.set_font_resolver(HashMap::new());
    engine.compute(&doc, &styles)
}

/// 按 id 找 LayoutBox（深度优先）。
fn find_id<'a>(b: &'a LayoutBox, doc: &'a Document, id: &str) -> Option<&'a LayoutBox> {
    if let Some(nid) = b.node_id
        && let Some(node) = doc.get(nid)
        && let NodeKind::Element(e) = &node.kind
        && e.get_attribute("id").as_deref() == Some(id)
    {
        return Some(b);
    }
    b.children.iter().find_map(|c| find_id(c, doc, id))
}

/// 驱动案最小形（box-sizing-content-box-003 ref 腿）：abspos 容器 min-width:500px，
/// 双纯文本 float min-width:250px + max-width:350px 应各 250 宽并排填满容器。
/// 旧态：收缩臂按词宽写回（"LEFT HALF" 9 字符 = 90px），§10.4 min-width 地板丢失。
#[test]
fn r5517_float_min_width_floor_fills_container() {
    let css = "body { margin: 0; } .container { position: absolute; left: 25px; top: 25px; \
               min-width: 500px; max-width: 700px; height: 90px; background: red; } \
               .box-sized { min-width: 250px; max-width: 350px; height: 100%; float: left; } \
               #one { background: green; } #two { background: blue; }";
    let html = format!(
        r#"<html><head><style>{css}</style></head><body>
<div class="container">
 <div class="box-sized" id="one">LEFT HALF</div>
 <div class="box-sized" id="two">RIGHT HALF</div>
</div>
</body></html>"#
    );
    let doc = zero_dom::parse_html(&html);
    let result = layout(css, &html);
    let root = Arc::try_unwrap(result.root).unwrap_or_else(|arc| (*arc).clone());
    let one = find_id(&root, &doc, "one").expect("float one 应存在");
    assert!(
        (one.width - 250.0).abs() < 0.5,
        "float one 宽应 250（§10.4 min-width 地板胜词宽收缩 90），实得 {}",
        one.width
    );
    let two = find_id(&root, &doc, "two").expect("float two 应存在");
    assert!(
        (two.width - 250.0).abs() < 0.5,
        "float two 宽应 250（同上），实得 {}",
        two.width
    );
    assert!(
        (two.x - one.x - 250.0).abs() < 0.5,
        "两 float 应并排（two.x = one.x + 250），实得 one.x={} two.x={}",
        one.x,
        two.x
    );
}

/// max-width cap 侧（§10.4 先 cap）＋ box-sizing:frame 算术两态：content-box 时
/// cap 入内容盒（border-box cap = 值 + 水平 frame），border-box 时值即 border-box。
/// "lorem ipsum dolor sit amet consectetur" 39 字符 × 10px = 390 > 50（cap 生效前提）。
#[test]
fn r5517_float_max_width_cap_and_box_sizing_frames() {
    let css = "body { margin: 0; } \
               .cap-cb { float: left; max-width: 50px; padding: 5px; border: 2px solid black; } \
               .cap-bb { float: left; max-width: 50px; padding: 5px; border: 2px solid black; box-sizing: border-box; }";
    let html = format!(
        r#"<html><head><style>{css}</style></head><body>
<div class="cap-cb" id="cb">lorem ipsum dolor sit amet consectetur</div>
<div class="cap-bb" id="bb">lorem ipsum dolor sit amet consectetur</div>
</body></html>"#
    );
    let doc = zero_dom::parse_html(&html);
    let result = layout(css, &html);
    let root = Arc::try_unwrap(result.root).unwrap_or_else(|arc| (*arc).clone());
    // content-box：max-width:50 管内容盒 → border-box = 50 + (5+2)×2 = 64。
    let cb = find_id(&root, &doc, "cb").expect("cap-cb 应存在");
    assert!(
        (cb.width - 64.0).abs() < 0.5,
        "content-box float 宽应 64（cap 50 + frame 14），实得 {}",
        cb.width
    );
    assert!(
        (cb.content_width - 50.0).abs() < 0.5,
        "content-box 内容宽应恰 50（cap 直接管内容盒），实得 {}",
        cb.content_width
    );
    // border-box：max-width:50 管 border-box → 宽 50、内容 50 − 14 = 36。
    let bb = find_id(&root, &doc, "bb").expect("cap-bb 应存在");
    assert!(
        (bb.width - 50.0).abs() < 0.5,
        "border-box float 宽应 50（cap 即 border-box，不加 frame），实得 {}",
        bb.width
    );
    assert!(
        (bb.content_width - 36.0).abs() < 0.5,
        "border-box 内容宽应 36（50 − frame 14），实得 {}",
        bb.content_width
    );
}
