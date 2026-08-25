use super::super::*;

use zero_css_parser::values::VerticalAlignValue;
use zero_dom::NodeId;

fn vertical_run(text: &str, node_id: NodeId, is_rtl: bool) -> TextRun {
    let mut run = TextRun::simple(text.to_string(), node_id, 20.0, 20.0, VerticalAlignValue::Baseline);
    run.is_ahem_font = true;
    run.is_rtl = is_rtl;
    run
}

fn visible_fragments(ctx: &InlineFormattingContext) -> Vec<&TextFragment> {
    ctx.all_fragments()
        .into_iter()
        .filter(|fragment| !fragment.text.is_empty())
        .collect()
}

fn visual_text(fragments: &[&TextFragment]) -> String {
    fragments.iter().flat_map(|fragment| fragment.text.chars()).collect()
}

fn visual_char_positions(fragments: &[&TextFragment]) -> Vec<(f32, f32, f32, f32)> {
    fragments
        .iter()
        .flat_map(|fragment| {
            let char_count = fragment.text.chars().count();
            let char_height = fragment.height / char_count as f32;
            (0..char_count).map(move |index| {
                (
                    fragment.x,
                    fragment.y + char_height * index as f32,
                    fragment.width,
                    char_height,
                )
            })
        })
        .collect()
}

fn source_expectations<'a>(fragments: &'a [&'a TextFragment]) -> Vec<(&'a str, &'a str, Option<&'a str>)> {
    fragments
        .iter()
        .map(|fragment| {
            let source = fragment
                .source
                .as_ref()
                .expect("BiDi fragment must keep source mapping");
            (fragment.text.as_str(), source.text.as_ref(), source.logical_slice())
        })
        .collect()
}

/// https://drafts.csswg.org/css-pseudo-4/#generated-content
/// https://drafts.csswg.org/css-writing-modes-3/#bidi-algo
#[test]
fn generated_prefix_and_text_share_vertical_bidi_sequence() {
    let mut doc = zero_dom::Document::new();
    let prefix_node = doc.create_text_node("▴ ");
    let text_node = doc.create_text_node("CR");

    let mut split = InlineFormattingContext::new(200.0).with_vertical(true);
    split.break_into_lines(vec![
        vertical_run("▴ ", prefix_node, true),
        vertical_run("CR", text_node, true),
    ]);

    let mut single = InlineFormattingContext::new(200.0).with_vertical(true);
    single.break_into_lines(vec![vertical_run("▴ CR", NodeId::default(), true)]);

    let split_fragments = visible_fragments(&split);
    let single_fragments = visible_fragments(&single);
    assert_eq!(visual_text(&split_fragments), visual_text(&single_fragments));
    assert_eq!(
        visual_char_positions(&split_fragments),
        visual_char_positions(&single_fragments)
    );

    let split_owners = split_fragments
        .iter()
        .map(|fragment| (fragment.text.as_str(), fragment.node_id))
        .collect::<Vec<_>>();
    assert_eq!(
        split_owners,
        vec![("CR", text_node), (" ", prefix_node), ("▴", prefix_node)]
    );
    assert_eq!(
        source_expectations(&split_fragments),
        vec![("CR", "CR", Some("CR")), (" ", "▴ ", Some(" ")), ("▴", "▴ ", Some("▴"))]
    );
}
