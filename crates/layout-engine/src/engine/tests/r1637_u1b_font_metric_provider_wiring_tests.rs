//! R1637 U1b-wiring 切片 A 集成测试：证明 `LayoutEngine::set_font_metric_provider`
//! 注入的 provider 经 `compute_final_inline_layouts`（stored IFC 路径）真触达 IFC，
//! 使 line-height:normal 走 per-font 真实度量。
//!
//! 切片 A 仅 thread stored 路径（measure 路径 `measure_text_content` 同源注入为切片 B）。
//! 默认（不调 `set_font_metric_provider`）= `None` = 逐字节等价旧路径 = 零回归（单测覆盖）。

use super::*;
use crate::inline::{AdvanceSource, FontMetricProvider, LineMetrics};
use std::cell::Cell;
use std::rc::Rc;

/// 带调用计数的桩 provider：每次 `line_metrics` 被咨询时计数 +1，并返回固定的 per-em
/// 度量（ratio 0.9，≠ Ahem 1.0、≠ 常数 1.164，证 provider 路径被走）。`resolve: false`
/// 模拟字体未加载（返回 None）。
struct CountingMetricProvider {
    count: Rc<Cell<u32>>,
    resolve: bool,
}

impl FontMetricProvider for CountingMetricProvider {
    fn line_metrics(&self, _font_family: &[String], size: f32) -> Option<LineMetrics> {
        self.count.set(self.count.get() + 1);
        if !self.resolve {
            return None;
        }
        // ascent=0.6em / descent=-0.2em / line_gap=0.1em → ratio = 0.9
        Some(LineMetrics {
            ascent: 0.6 * size,
            descent: -0.2 * size,
            line_gap: 0.1 * size,
        })
    }
}

/// 构造 html > body > div(TestFont,20px,block) > "hello" 文本节点。
fn build_div_with_text() -> (Document, HashMap<NodeId, ComputedStyle>, NodeId) {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();
    let text = doc.create_text_node("hello");
    doc.append_child(div, text).unwrap();

    let mut styles = HashMap::new();
    let mut s = ComputedStyle::default();
    s.display = DisplayValue::Block;
    s.font_family = vec!["TestFont".to_string()];
    s.font_size = LengthValue::Px(20.0);
    s.line_height = zero_style_system::LineHeightValue::Normal;
    styles.insert(div, s);
    (doc, styles, div)
}

/// 注入 provider 后 `compute` 期间 stored IFC 路径真咨询了 provider（count > 0）。
/// 证明切片 A 接线非 dead：provider 从 LayoutEngine → compute_final_inline_layouts →
/// IFC → resolve_font_metrics_with_provider 触达。
#[test]
fn r1637_injected_provider_is_consulted_during_compute() {
    let (doc, styles, _div) = build_div_with_text();
    let count = Rc::new(Cell::new(0u32));
    let provider = CountingMetricProvider {
        count: count.clone(),
        resolve: true,
    };
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.set_font_metric_provider(Rc::new(provider));
    let _ = engine.compute(&doc, &styles);
    assert!(
        count.get() > 0,
        "stored IFC path must consult the injected provider (count={})",
        count.get()
    );
}

/// 默认（不注入 provider）compute 正常完成（零回归 dormant 保证：字段 None）。
/// provider 字段为 None 时 IFC 回退常数度量，行为与未引入 wiring 前逐字节一致。
#[test]
fn r1637_no_provider_compute_is_zero_regression() {
    let (doc, styles, _div) = build_div_with_text();
    let mut engine = LayoutEngine::new(800.0, 600.0);
    // 不调 set_font_metric_provider → font_metric_provider = None → 零回归。
    let result = engine.compute(&doc, &styles);
    // div 存在且布局完成（无 panic）。
    assert!(result.root.children.len() >= 1, "layout must complete");
}

/// provider 注入但无法解析字体（resolve: false → None）时回退常数度量，
/// 不 panic（与未注入等价，单测层已证 resolve_font_metrics_with_provider 行为）。
#[test]
fn r1637_unresolved_provider_falls_back_safely() {
    let (doc, styles, _div) = build_div_with_text();
    let count = Rc::new(Cell::new(0u32));
    let provider = CountingMetricProvider {
        count: count.clone(),
        resolve: false,
    };
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.set_font_metric_provider(Rc::new(provider));
    let _ = engine.compute(&doc, &styles);
    // 被咨询但返回 None → 回退常数，无 panic。
    assert!(count.get() > 0, "provider consulted even when unresolved");
}

/// resolver 与 advance source 必须沿真实 LayoutEngine 路径触达整串测量。
#[test]
fn contextual_advance_source_is_consulted_during_compute() {
    struct CountingAdvance(Rc<Cell<u32>>);
    impl AdvanceSource for CountingAdvance {
        fn measure(&self, _ch: char, _font_id: Option<u32>, font_size: f32, _is_ahem: bool) -> f32 {
            font_size * 0.5
        }

        fn measure_text(&self, text: &str, font_id: Option<u32>, font_size: f32, _is_ahem: bool) -> f32 {
            assert_eq!(font_id, Some(7));
            self.0.set(self.0.get() + 1);
            text.chars().count() as f32 * font_size * 0.5
        }
    }

    let (doc, styles, _div) = build_div_with_text();
    let count = Rc::new(Cell::new(0));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.set_font_resolver(HashMap::from([("TestFont".to_string(), 7)]));
    engine.set_advance_source(Rc::new(CountingAdvance(count.clone())));
    let _ = engine.compute(&doc, &styles);

    assert!(count.get() > 0, "layout must consult contextual text measurement");
}
