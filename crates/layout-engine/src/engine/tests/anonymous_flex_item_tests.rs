//! flex 匿名项 / inline-block shrink-to-fit 回归测试（从 engine.rs 抽出，保持 2000 行约束）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_css_parser::values::DisplayValue;
use zero_dom::Document;
use zero_style_system::StyleSystem;

/// 测试 flex 容器中的文本节点生成匿名 flex item。
/// CSS Flexbox §4：flex 容器中每个连续文本运行应生成匿名 flex item。
#[test]
fn test_anonymous_flex_item_created() {
    let html = r#"<html><body style="margin:0"><div style="display:flex">text node</div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 找到 flex 容器
    let found_flex = false;
    let mut found_anonymous_text = false;
    let mut stack = vec![&result.root];
    while let Some(box_node) = stack.pop() {
        // 检查是否为匿名文本项
        if box_node.is_anonymous_text_item {
            found_anonymous_text = true;
            // 匿名文本项应有非零尺寸
            assert!(box_node.width > 0.0, "anonymous flex item should have width > 0");
            assert!(box_node.height > 0.0, "anonymous flex item should have height > 0");
            // node_id 应指向文本节点
            if let Some(nid) = box_node.node_id {
                if let Some(n) = doc.get(nid) {
                    assert!(
                        matches!(&n.kind, zero_dom::NodeKind::Text(_)),
                        "anonymous item node_id should point to a text node"
                    );
                }
            }
        }
        stack.extend(&box_node.children);
    }

    assert!(
        found_anonymous_text,
        "should find at least one anonymous text item in flex container"
    );
    let _ = found_flex;
}

/// 测试多个文本节点和元素混合在 flex 容器中。
/// "a a" <div>x x</div> "b b" 应生成 3 个 flex items（2 个匿名 + 1 个元素）。
#[test]
fn test_mixed_text_and_element_flex_items() {
    let html = r#"<html><body style="margin:0"><div style="display:flex">a a<div>x x</div>b b</div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 找到 flex 容器（display:flex 的 div）
    let mut flex_container: Option<&crate::types::LayoutBox> = None;
    let mut stack = vec![&result.root];
    while let Some(box_node) = stack.pop() {
        if let Some(nid) = box_node.node_id {
            if let Some(style) = styles.get(&nid) {
                if matches!(style.display, DisplayValue::Flex | DisplayValue::InlineFlex) {
                    flex_container = Some(box_node);
                    break;
                }
            }
        }
        stack.extend(&box_node.children);
    }

    let container = flex_container.expect("should find flex container");
    // 应有 3 个子项：2 个匿名文本 + 1 个 div 元素
    assert_eq!(
        container.children.len(),
        3,
        "flex container should have 3 children (2 anonymous text + 1 element)"
    );

    let anonymous_count = container.children.iter().filter(|c| c.is_anonymous_text_item).count();
    assert_eq!(anonymous_count, 2, "should have 2 anonymous text items");

    let element_count = container.children.iter().filter(|c| !c.is_anonymous_text_item).count();
    assert_eq!(element_count, 1, "should have 1 element child");
}

/// 测试非 flex 容器中的文本节点不会生成匿名项。
#[test]
fn test_no_anonymous_items_in_block_container() {
    let html = r#"<html><body style="margin:0"><div>text node</div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 确保整个布局树中没有匿名文本项
    let mut stack = vec![&result.root];
    while let Some(box_node) = stack.pop() {
        assert!(
            !box_node.is_anonymous_text_item,
            "block container should not create anonymous text items"
        );
        stack.extend(&box_node.children);
    }
}

/// 回归：CSS §10.3.9 — width:auto 的 inline-block 应 shrink-to-fit 到内容最大宽度。
/// 旧 bug：taffy 把 width:auto 的 inline-block 拉伸到可用宽度（如同 block），
/// 含显式宽度 block 子元素的 inline-block 被错误填满 784px 而非收缩到子元素宽度。
/// 验证 shrink_inline_blocks_to_content 后处理（baseline-block-with-overflow-001 用例）。
#[test]
fn test_inline_block_width_auto_shrink_to_fit() {
    // inline-block `.outer`（width:auto）含一个显式 width:30px 的 block 子元素，
    // 应收缩到 ~30px，而非填满 800px 视口。
    let html = r#"<html><body style="margin:0">
      <div class="outer" id="o" style="display:inline-block;background:orange;padding:4px">
        <div class="inner" id="i" style="width:30px;height:30px;background:blue"></div>
      </div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn find<'a>(id: &str, doc: &Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
        if let Some(nid) = b.node_id
            && let Some(n) = doc.get(nid)
            && let zero_dom::NodeKind::Element(elem) = &n.kind
            && elem.get_attribute("id").as_deref() == Some(id)
        {
            return Some(b);
        }
        b.children.iter().find_map(|c| find(id, doc, c))
    }
    let o = find("o", &doc, &result.root).expect("inline-block #o");
    // 收缩后 border-box = 30(内容) + 4+4(左右 padding) = 38，远小于 784。
    assert!(
        o.width < 100.0,
        "width:auto inline-block should shrink-to-fit to content (~38px), not fill available (got w={})",
        o.width
    );
    assert!(
        (o.width - 38.0).abs() < 1.0,
        "expected inline-block width ~38px (30 content + 8 padding), got w={}",
        o.width
    );
}

/// 回归（R368）：width:auto 且仅含**文本内容**（无 block/inline 子元素）的 inline-block
/// 也必须 shrink-to-fit。旧 `shrink_inline_blocks_to_content` 仅遍历 LayoutBox 子元素求宽，
/// 而文本经 measure callback 不产生子盒 → content_max_w=0 → 不收缩 → inline-block 被拉到
/// 容器满宽。ifc-011 的 `<span style="display:inline-block;border:20px solid blue;font:50px Ahem">X</span>`
/// 即此情形：应收缩到 ~90（50 文本 + 40 border）而非 784。修复改用 intrinsic_sizing 的
/// `box_content_max_width`（按 DOM text_content + 字体度量累加）。
#[test]
fn test_inline_block_text_content_shrink_to_fit() {
    // inline-block `.s`（width:auto）仅含文本 "XX"，Ahem 等宽 50px → 文本宽 100，
    // + border 20×2 = 140，应收缩到 ~140 而非填满 800px 视口。
    let html = r#"<html><body style="margin:0">
      <span class="s" id="s" style="display:inline-block;border:20px solid blue;font:50px/1 Ahem">XX</span>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn find<'a>(id: &str, doc: &Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
        if let Some(nid) = b.node_id
            && let Some(n) = doc.get(nid)
            && let zero_dom::NodeKind::Element(elem) = &n.kind
            && elem.get_attribute("id").as_deref() == Some(id)
        {
            return Some(b);
        }
        b.children.iter().find_map(|c| find(id, doc, c))
    }
    let s = find("s", &doc, &result.root).expect("inline-block #s");
    assert!(
        s.width < 200.0,
        "width:auto inline-block with text content should shrink-to-fit (~140px), not fill available (got w={})",
        s.width
    );
    // 文本 "XX" Ahem 50px = 100 + 左右 border 各 20 = 140。
    assert!(
        (s.width - 140.0).abs() < 1.5,
        "expected inline-block width ~140px (100 text + 40 border), got w={}",
        s.width
    );
}
