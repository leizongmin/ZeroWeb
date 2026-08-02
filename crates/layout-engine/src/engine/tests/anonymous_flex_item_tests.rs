//! flex 匿名项 / inline-block shrink-to-fit 回归测试（从 engine.rs 抽出，保持 2000 行约束）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use std::collections::HashMap;
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

/// R1480（R109 inline-box-model 增量 2）：`display:inline` 且带 **border** 的元素也应
/// shrink-to-fit（R372 此前仅对带 background 的 inline 触发）。旧 bug：inline→taffy::Block
/// 拉满宽，带 border 的 inline（如 WPT border-width-applies-to-008）border 画在满宽 box
/// （应 content-width = 内容 + 左右 border）。修：is_shrinkable 对 inline 触发条件加 border。
#[test]
fn test_inline_with_border_shrinks_to_content() {
    // `<span display:inline border:20px>XX</span>` Ahem 40px → 文本宽 80 + 左右 border 各 20
    // = 120，应收缩到 ~120 而非填满 800px 视口（满宽 border）。
    let html = r#"<html><body style="margin:0;font:40px/1 Ahem">
      <span id="s" style="display:inline;border:20px solid black">XX</span>
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
    let s = find("s", &doc, &result.root).expect("inline #s");
    assert!(
        s.width < 300.0,
        "display:inline with border should shrink-to-fit (content ~120px), not fill available with full-width border (got w={})",
        s.width
    );
}

/// 回归（R372）：`display:inline` 且带非默认 background 的元素（如 morning.work
/// `.item-tag` 徽章 span）也应 shrink-to-fit。旧 bug：ZeroWeb 把 inline 映射为 taffy
/// Block，拉到容器满宽（满宽色条）；此处按 intrinsic 内容宽收缩（仅 width 维度，
/// 完整 inline-box 模型属 Phase A）。纯文本 inline span（无 background）不受影响。
#[test]
fn test_inline_element_with_background_shrink_to_fit() {
    // inline span `.tag`（display:inline 默认 + background-color）含文本 "Fedora"，
    // 应收缩到 ~文本宽+padding，而非填满 800px 视口。
    let html = r#"<html><body style="margin:0">
      <span class="tag" id="t" style="background-color:#607cd2;color:#fff;padding:0 6px">Fedora</span>
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
    let t = find("t", &doc, &result.root).expect("inline span #t");
    assert!(
        t.width < 200.0,
        "inline+background span should shrink-to-fit to content, not fill available (got w={})",
        t.width
    );
}

/// 回归（R526）：abspos（position:absolute/fixed）的 flex 子元素**不**受 `order`
/// 属性重排，其绘制顺序遵循 DOM 顺序（CSS Flexbox §8.1 + CSS Appendix E step 6；
/// flexbox-paint-ordering-003）。旧实现把 abspos 也纳入 `order` 排序 → abspos 按
/// order 值重排 → 破坏 tree-order 绘制顺序。修复：tree.rs 建树 + engine.rs
/// sort_children_by_css_order 两站点对 abspos 用 0 作排序键（stable sort 保持 DOM 顺序）。
///
/// 本测试断言 in-flow 子元素被 `order` 正确重排（#a order:3 排在 #b order:1 之后），
/// 而 abspos 子元素保持 DOM 顺序（#abs1 在 DOM 第 1 → 仍在 #abs2 之前），与 order 值无关。
#[test]
fn test_abspos_flex_children_not_reordered_by_order() {
    // flex 容器含 4 个子元素：abspos #abs1(order:9)、in-flow #a(order:3)、
    // in-flow #b(order:1)、abspos #abs2(order:7)。DOM 顺序 = abs1, a, b, abs2。
    let html = r#"<html><body style="margin:0">
      <div id="container" style="display:flex;width:400px">
        <div id="abs1" style="position:absolute;order:9"></div>
        <div id="a" style="order:3"></div>
        <div id="b" style="order:1"></div>
        <div id="abs2" style="position:absolute;order:7"></div>
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

    let container = find("container", &doc, &result.root).expect("flex container");
    // 收集子元素在 LayoutBox 中的出现顺序（按 node_id 对应元素的 id 属性）。
    let order: Vec<String> = container
        .children
        .iter()
        .filter_map(|c| {
            let nid = c.node_id?;
            let n = doc.get(nid)?;
            match &n.kind {
                zero_dom::NodeKind::Element(elem) => elem.get_attribute("id"),
                _ => None,
            }
        })
        .collect();

    // in-flow 子元素须按 order 重排：b(order:1) 在 a(order:3) 之前。
    let pos_b = order
        .iter()
        .position(|x| x.as_str() == "b")
        .expect("in-flow #b present");
    let pos_a = order
        .iter()
        .position(|x| x.as_str() == "a")
        .expect("in-flow #a present");
    assert!(
        pos_b < pos_a,
        "in-flow flex items must be reordered by `order` (b@order:1 before a@order:3), got {:?}",
        order
    );
    // abspos 子元素须保持 DOM 顺序：abs1（DOM 第 1）在 abs2（DOM 第 4）之前，
    // 不受 order 值（9 vs 7）影响。
    let pos_abs1 = order
        .iter()
        .position(|x| x.as_str() == "abs1")
        .expect("abspos #abs1 present");
    let pos_abs2 = order
        .iter()
        .position(|x| x.as_str() == "abs2")
        .expect("abspos #abs2 present");
    assert!(
        pos_abs1 < pos_abs2,
        "abspos flex children must NOT be reordered by `order`; keep DOM order (abs1 before abs2), got {:?}",
        order
    );
}

/// R982 Phase 0 探针：csswg #5663 — flex item min-width:auto 的 transferred-size-suggestion
/// 应从「明确拉伸的 cross size」推导（非固有内容尺寸）。
///
/// 场景（同 flex-minimum-width-flex-items-013.html）：
///   `<div style="display:flex; width:0; height:50px"><img style="width:999px">`
///   img 固有 300×150（ratio 2:1）。
///
/// 预期（csswg #5663）：flex 容器 width:0 → img 主尺寸被 min-width:auto clamp。
///   min-width:auto = transferred-size-suggestion = 拉伸 cross-size 50 × 固有比 300/150 = 100px。
///   故 img 最终 width ≈ 100px（绿方块填满 100px 正方形，无红）。
///
/// 本探针断言正确行为；当前 ZW 若未实现 #5663，img width 会是 999（未 clamp）或 0（塌缩），
/// 本测试 FAIL，揭示精确 gap。fix 落地后此测试转 PASS（作回归守卫）。
#[test]
fn test_flex_transferred_min_size_from_stretched_cross() {
    let html = r#"<html><body style="margin:0">
<div style="display:flex; width:0px; height:50px;"><img src="g.png" style="width:999px;"></div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let img_id = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
    let mut img_sizes = HashMap::new();
    img_sizes.insert(img_id, (300.0, 150.0)); // 固有 300×150，ratio 2:1
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, img_sizes, std::collections::HashMap::new());
    let (w, h) = find_box(&result.root, img_id).expect("img box found");
    // csswg #5663：min-width:auto = stretched cross(50) × ratio(2) = 100px
    assert!(
        (w - 100.0).abs() < 2.0,
        "R982: flex item img width should be clamped to transferred min-size ~100px (stretched cross 50 × ratio 2), got width={w}, height={h}"
    );
}

/// R982 回归守卫：transferred-size-suggestion 仅对 flex 容器生效，非 flex 父（block）
/// 不应改 img 尺寸——img 仍按其显式 width 渲染。
#[test]
fn test_flex_transferred_min_size_not_applied_to_non_flex_parent() {
    let html = r#"<html><body style="margin:0">
<div style="width:0px; height:50px;"><img src="g.png" style="width:999px;"></div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let img_id = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
    let mut img_sizes = HashMap::new();
    img_sizes.insert(img_id, (300.0, 150.0));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, img_sizes, std::collections::HashMap::new());
    let (w, _h) = find_box(&result.root, img_id).expect("img box found");
    // 非 flex 父：img 不被 transferred clamp，保持显式 width 999
    assert!(
        (w - 999.0).abs() < 2.0,
        "R982 guard: img in non-flex block parent should keep width 999 (no transferred clamp), got {w}"
    );
}

/// R983 回归：flex-direction:column 下 transferred-size-suggestion 作用于主轴（height）。
/// 容器 flex-direction:column width:80px height:0，img height:999 固有 300×150（ratio 2:1）。
/// 主轴=height，cross=width=80（明确）。transferred main = cross_w / ratio = 80 / 2 = 40px。
/// img height 应被 min-height:auto=40 floor（从 999 收缩到 40）。
/// 关键：auto_min = transferred（40），非 min(intrinsic_h=150, transferred=40)=40——
/// 此处 intrinsic>transferred 故两种算法一致；真正区别在 intrinsic<transferred 的案
/// （flex-minimum-height-flex-items-007：固有 60×60，cross 100，transferred 100，intrinsic 60）。
#[test]
fn test_flex_transferred_min_size_column_direction() {
    let html = r#"<html><body style="margin:0">
<div style="display:flex; flex-direction:column; width:80px; height:0px;"><img src="g.png" style="height:999px;"></div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let img_id = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
    let mut img_sizes = HashMap::new();
    img_sizes.insert(img_id, (300.0, 150.0));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, img_sizes, std::collections::HashMap::new());
    let (w, h) = find_box(&result.root, img_id).expect("img box found");
    assert!(
        (h - 40.0).abs() < 2.0,
        "R983: column flex item img height should be clamped to transferred min-size ~40px (cross 80 / ratio 2), got width={w}, height={h}"
    );
}

/// R983 回归：column flex item，intrinsic < transferred 时 auto_min = transferred（非 intrinsic）。
/// 同 flex-minimum-height-flex-items-007：img 固有 60×60（ratio 1），column 容器 width:100。
/// transferred = cross_w / ratio = 100 / 1 = 100。旧 min(intrinsic_h=60, 100)=60 错→应 100。
#[test]
fn test_flex_transferred_min_size_column_intrinsic_smaller_than_transferred() {
    let html = r#"<html><body style="margin:0">
<div style="display:flex; flex-direction:column; width:100px; height:10px;"><img src="g.png" style="width:100px;"></div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let img_id = doc.get_elements_by_tag_name("img").into_iter().next().expect("img");
    let mut img_sizes = HashMap::new();
    img_sizes.insert(img_id, (60.0, 60.0)); // 固有 60×60，ratio 1
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, img_sizes, std::collections::HashMap::new());
    let (w, h) = find_box(&result.root, img_id).expect("img box found");
    assert!(
        (h - 100.0).abs() < 2.0,
        "R983: column flex item img height should be transferred min-size ~100px (cross 100 / ratio 1), NOT raw intrinsic 60; got width={w}, height={h}"
    );
}

/// R1750：bare-text 匿名 flex item 的 min-width:auto 应 = min-content（最宽词），
/// 非 max-content（全文本）。`measure_text_content` 文本节点分支（inline_finalization.rs）
/// 旧实现恒返 measured_width（max-content），忽略 available_space MinContent → taffy 把
/// min-size:auto 算成 max-content → flex item 无法收缩到最宽词以下（flex-minimum-width-
/// flex-items 谱系）。R1750 fix：MinContent 时按空白分词取最宽词宽。
///
/// 场景：flex 容器 width:10px + Ahem 50px，匿名文本 item "IT E"：
/// 正确 min-content = 最宽词 "IT" = 2 字 × 50 = 100px；
/// 旧 bug max-content = "IT E" 全文本（含空格）= 200px。
/// 容器仅 10px → item 被 min-width:auto floor。正确 ≈100，旧 bug = 200。
#[test]
fn test_r1750_anonymous_text_flex_item_min_content() {
    let html = r#"<html><body style="margin:0"><div style="display:flex;width:10px;font:50px/1 Ahem">IT E</div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 找匿名文本 flex item。
    let mut anon: Option<&LayoutBox> = None;
    let mut stack = vec![&result.root];
    while let Some(b) = stack.pop() {
        if b.is_anonymous_text_item {
            anon = Some(b);
            break;
        }
        stack.extend(b.children.iter());
    }
    let item = anon.expect("anonymous text flex item");
    // 最宽词 "IT" = 2 × 50 = 100px（min-content）。旧 bug 返 200（max-content 全文本）。
    assert!(
        (item.width - 100.0).abs() < 3.0,
        "R1750: bare-text flex item min-width:auto 应 = min-content 最宽词 ~100px（'IT'），\
         旧 bug = max-content 200px（'IT E' 全文本）；实际 {:.1}",
        item.width
    );
}

/// R1750 守卫：单行无空格文本 min-content == max-content（无换行机会），
/// fix 不应改变此类 case（split 仍得唯一词 = 全文本宽）。
#[test]
fn test_r1750_single_word_min_content_equals_max() {
    let html = r#"<html><body style="margin:0"><div style="display:flex;width:10px;font:50px/1 Ahem">XXXX</div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let mut anon: Option<&LayoutBox> = None;
    let mut stack = vec![&result.root];
    while let Some(b) = stack.pop() {
        if b.is_anonymous_text_item {
            anon = Some(b);
            break;
        }
        stack.extend(b.children.iter());
    }
    let item = anon.expect("anonymous text flex item");
    // 单词 "XXXX" 无换行机会：min-content = max-content = 4 × 50 = 200。
    assert!(
        (item.width - 200.0).abs() < 3.0,
        "R1750 guard: 单词文本 min-content 应 == max-content ~200px（无换行机会），实际 {:.1}",
        item.width
    );
}

/// 在 LayoutResult 树中按 node_id 查找盒的 (width, height)。
fn find_box(root: &LayoutBox, node_id: zero_dom::NodeId) -> Option<(f32, f32)> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some((b.width, b.height));
        }
        stack.extend(b.children.iter());
    }
    None
}

/// R1024：flex item（block 容器）含文本 + inline Element 子（`<br>`）时不应塌缩 w=0。
///
/// `<div style="display:flex"><div class=item>The quick brown fox<br>jumps</div></div>`
/// 此前：默认 block build 路径只收 Element 子 → item 成 new_with_children([br]) 非 leaf →
/// measure 不触发 → intrinsic 宽 = br(0) = 0 → 文本 wrap 到 ~0 宽垂直堆叠。
/// 修复：flex/grid item 的全 inline 子 block 作 leaf（context=dom_id），measure 经
/// has_inline_content 把文本作一个 IFC 单位测量 → item 宽 = 文本宽（非零）。
#[test]
fn test_r1024_flex_item_with_text_and_br_not_collapse() {
    let html = r#"<html><body style="margin:0"><div style="display:flex">
        <div id="item">The quick brown fox jumps<br>over the lazy dog</div>
    </div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let item_id = doc
        .query_selector(doc.root(), "#item")
        .or_else(|| {
            // 回退：按 id 属性手动查找
            let mut stack = vec![doc.root()];
            while let Some(nid) = stack.pop() {
                if let Some(n) = doc.get(nid) {
                    if let zero_dom::NodeKind::Element(e) = &n.kind {
                        if e.get_attribute("id").as_deref() == Some("item") {
                            return Some(nid);
                        }
                    }
                    stack.extend(doc.child_nodes(nid));
                }
            }
            None
        })
        .expect("item element found");
    let (w, _h) = find_box(&result.root, item_id).expect("item box found");
    assert!(
        w > 100.0,
        "R1024: flex item with text+br should have non-zero width (text width), got w={w}; \
         was 0 before fix (block build path skipped text children → new_with_children non-leaf → measure not fired)"
    );
}

/// R1025：inline-block 含文本 + inline Element 子（`<br>`）应 shrink-to-fit，不应填满父宽。
///
/// `<span style="display:inline-block">text<br>text</span>` 此前：默认 block build 路径只收
/// Element 子 → inline-block 成 new_with_children([br]) 非 leaf → measure 不触发 → 误填满父宽
///（w=800）。修复：inline-block（content-sized）的全 inline 子作 leaf，measure 经 has_inline_content
/// 测文本宽 → shrink-to-fit（w=文本宽）。
#[test]
fn test_r1025_inline_block_with_text_and_br_shrink_to_fit() {
    let html = r#"<html><body style="margin:0"><span id="ib" style="display:inline-block">The quick brown fox<br>jumps over the lazy dog</span></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let ib_id = doc
        .query_selector(doc.root(), "#ib")
        .or_else(|| {
            let mut stack = vec![doc.root()];
            while let Some(nid) = stack.pop() {
                if let Some(n) = doc.get(nid) {
                    if let zero_dom::NodeKind::Element(e) = &n.kind {
                        if e.get_attribute("id").as_deref() == Some("ib") {
                            return Some(nid);
                        }
                    }
                    stack.extend(doc.child_nodes(nid));
                }
            }
            None
        })
        .expect("ib element found");
    let (w, _h) = find_box(&result.root, ib_id).expect("ib box found");
    assert!(
        w < 400.0,
        "R1025: inline-block with text+br should shrink-to-fit (w<400, text width), got w={w}; \
         was 800 before fix (filled parent — block build path skipped text → new_with_children non-leaf → measure not fired)"
    );
}

/// R1025：float 含文本 + inline Element 子（`<br>`）应 shrink-to-fit，不应填满父宽。
/// 同 inline-block bug 形态，R1024 leaf pattern 扩展到 float（content-sized）。
#[test]
fn test_r1025_float_with_text_and_br_shrink_to_fit() {
    let html = r#"<html><body style="margin:0"><div id="fl" style="float:left">The quick brown fox<br>jumps over the lazy dog</div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let fl_id = doc
        .query_selector(doc.root(), "#fl")
        .or_else(|| {
            let mut stack = vec![doc.root()];
            while let Some(nid) = stack.pop() {
                if let Some(n) = doc.get(nid) {
                    if let zero_dom::NodeKind::Element(e) = &n.kind {
                        if e.get_attribute("id").as_deref() == Some("fl") {
                            return Some(nid);
                        }
                    }
                    stack.extend(doc.child_nodes(nid));
                }
            }
            None
        })
        .expect("fl element found");
    let (w, _h) = find_box(&result.root, fl_id).expect("fl box found");
    assert!(
        w < 400.0,
        "R1025: float with text+br should shrink-to-fit (w<400), got w={w}; was 800 before fix"
    );
}

/// R1495：plain block（`<p>`）含文本 + inline 元素子（`<a>`）时，taffy 把 `<a>` 作 block 子
/// 致 `<p>` non-leaf，taffy 仅按 `<a>` 定 `<p>` 高（丢多行文本高度）→ remeasure 长高后后续
/// 兄弟 `<div>` 仍定位在旧 taffy 高处 → 重叠。`shift_siblings_after_ifc_grow` post-process
/// 下移后续兄弟。本测试验证后续 `<div>` 不再与 `<p>` 重叠（div.y >= p.y + p.height）。
#[test]
fn test_shift_siblings_after_ifc_grow_no_overlap() {
    let html = r##"<html><body style="margin:0;font:20px/1 Ahem">
      <p id="p">XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX <a href="#">link</a> YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY</p>
      <div id="f">follow</div>
    </body></html>"##;
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
    let p = find("p", &doc, &result.root).expect("p");
    let f = find("f", &doc, &result.root).expect("follow div");
    // 同父（body）下，后续兄弟 div 的 y 须 >= p 底边（y+height），不重叠。
    assert!(
        f.y + 0.5 >= p.y + p.height,
        "R1495: follow div (y={}) must be below <p> bottom ({} = y {} + h {}), \
         not overlapping (shift_siblings_after_ifc_grow regression)",
        f.y,
        p.y + p.height,
        p.y,
        p.height
    );
}

/// R1498：R1495 的 `is_plain_real_block` 旧 display gate（Block/Flow/FlowRoot/ListItem）排除
/// `display:table`，致 `<p>`（Block，IFC remeasure 长高）后续 `<table>` 兄弟未下移而重叠
///（morning @375 `<p>` vs `<table>` 14px 重叠，struct-check 抓到）。gate 扩 Table/InlineTable
/// 后 table 整盒随前序兄弟长高下移（内部行/列布局与 y 无关，安全）。本测试验证后续
/// `<table>` 不再与 `<p>` 重叠（table.y >= p.y + p.height）。
#[test]
fn test_shift_siblings_after_ifc_grow_table_sibling_no_overlap() {
    // 窄 viewport（200）逼 `<p>` 长文本换行多行（IFC remeasure 长高），inline `<a>` 致
    // `<p>` non-leaf（taffy 仅按 `<a>` 估高，丢多行文本高）→ 后续 `<table>` 重叠。
    let html = r##"<html><body style="margin:0;font:20px/1 Ahem">
      <p id="p">XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX <a href="#">link</a> YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY</p>
      <table id="t"><tr><td>cell</td></tr></table>
    </body></html>"##;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(200.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(200.0, 600.0);
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
    let p = find("p", &doc, &result.root).expect("p");
    let t = find("t", &doc, &result.root).expect("table");
    assert!(
        t.y + 0.5 >= p.y + p.height,
        "R1498: table (y={}) must be below <p> bottom ({} = y {} + h {}), \
         not overlapping (table-sibling shift gate regression)",
        t.y,
        p.y + p.height,
        p.y,
        p.height
    );
}

/// R1502：split gate——Flex 容器作**next 兄弟**（shiftee）应随前序 block 长高下移，但**不作 prev**
///（is_plain_real_block 仍排除 Flex）。R1500 实测把 Flex 加进 is_plain_real_block（prev+next 都含）
/// 致 css-flexbox -7 net-negative（Flex 作 prev 时 height 与 item grow/stretch 交互致 prev_bottom 误估）；
/// split 后 Flex 仅作 shiftee 解 morning @320 `<article>`(Block 长高) 重叠 disqus-side `<div>`(Flex)。
/// 本测试：`<p>`(Block, 含 inline `<a>` → IFC 长高) 后续 `<div style="display:flex">` 须下移不重叠。
#[test]
fn test_shift_siblings_after_ifc_grow_flex_sibling_shifts() {
    let html = r##"<html><body style="margin:0;font:20px/1 Ahem">
      <p id="p">XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX <a href="#">link</a> YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY</p>
      <div id="f" style="display:flex"><span>item</span></div>
    </body></html>"##;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(200.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(200.0, 600.0);
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
    let p = find("p", &doc, &result.root).expect("p");
    let f = find("f", &doc, &result.root).expect("flex div");
    assert!(
        f.y + 0.5 >= p.y + p.height,
        "R1502: flex div (y={}) must be below <p> bottom ({} = y {} + h {}), \
         not overlapping (split-gate flex-as-next regression)",
        f.y,
        p.y + p.height,
        p.y,
        p.height
    );
}

/// R1505：`display:inline-block`（非 floated）的 `is_block_level=false`（engine.rs:1888 仅
/// floated inline-block 标 block_level），故旧 `is_shiftable_next` 首个 guard（`c.is_block_level`）
/// 把它整体排除——`<p>`（Block，含 inline `<a>` → IFC remeasure 长高）后续 inline-block 兄弟
/// 未下移而重叠（inline-block-non-replaced-width-003/004：`<div style="display:inline-block">`
/// 定位 y=36 重叠 `<p>` 16..72，struct-sweep 抓到 4320px²）。shiftee 角色放宽含 InlineBlock
/// （prev 角色仍排除，避 height 误估）后 inline-block 整盒随前序 block 长高下移。本测试验证
/// 后续 inline-block `<div>` 不再与 `<p>` 重叠（ib.y >= p.y + p.height）。
#[test]
fn test_shift_siblings_after_ifc_grow_inline_block_sibling_shifts() {
    let html = r##"<html><body style="margin:0;font:20px/1 Ahem">
      <p id="p">XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX <a href="#">link</a> YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY</p>
      <div id="ib" style="display:inline-block"><span>ib</span></div>
    </body></html>"##;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(200.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(200.0, 600.0);
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
    let p = find("p", &doc, &result.root).expect("p");
    let ib = find("ib", &doc, &result.root).expect("inline-block div");
    assert!(
        ib.y + 0.5 >= p.y + p.height,
        "R1505: inline-block div (y={}) must be below <p> bottom ({} = y {} + h {}), \
         not overlapping (shiftable-next inline-block gate regression)",
        ib.y,
        p.y + p.height,
        p.y,
        p.height
    );
}
