use super::*;
use crate::inline::{AdvanceSource, FontMetricMap};
use std::rc::Rc;
use zero_render_foundation::font::FontLoader;

struct LoadedAdvance(FontLoader);
impl AdvanceSource for LoadedAdvance {
    fn measure(&self, ch: char, id: Option<u32>, size: f32, _: bool) -> f32 {
        self.0.measure_advance(id.expect("resolved document font"), ch, size)
    }
}

#[test]
fn downloaded_alias_preserves_intrinsic_width_normal_height_and_paint_baseline() {
    // Deliberately do not name the family Ahem: the synthetic font-name shortcut
    // would hide missing font-resource/metric plumbing.
    for (text, size) in [("XXXX", 40.0), ("XXX", 20.0)] {
        let (mut doc, body) = make_doc_with_body();
        let div = doc.create_element("div");
        let tn = doc.create_text_node(text);
        doc.append_child(body, div).unwrap();
        doc.append_child(div, tn).unwrap();
        let styles = HashMap::from([(
            div,
            ComputedStyle {
                display: DisplayValue::InlineBlock,
                font_family: vec!["DocumentFace".into()],
                font_size: LengthValue::Px(size),
                ..ComputedStyle::default()
            },
        )]);
        let mut fonts = FontLoader::new();
        let id = fonts
            .load_font(include_bytes!("../../../../../tests/wpt-runner/fonts/Ahem.ttf"))
            .unwrap();
        fonts.register_family_alias("DocumentFace", id);
        let mut engine = LayoutEngine::new(800.0, 600.0);
        engine.set_font_resolver(fonts.build_font_resolver());
        engine.set_font_metric_provider(Rc::new(FontMetricMap::new(fonts.build_line_metric_map(), false)));
        engine.set_advance_source(Rc::new(LoadedAdvance(fonts)));
        let result = engine.compute(&doc, &styles);
        let box_node = find_child_by_node_id(&result.root, div).unwrap();
        assert!(
            (box_node.width - text.len() as f32 * size as f32).abs() < 0.01,
            "width={}",
            box_node.width
        );
        assert!(
            (box_node.height - size as f32).abs() < 0.01,
            "height={}",
            box_node.height
        );
        assert_eq!(box_node.text_node_ascent_ratios.get(&tn), Some(&0.8));
    }
}
