//! intrinsic sizing 两趟布局回归测试（从 engine.rs 抽出，保持 2000 行约束）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::Document;
use zero_style_system::StyleSystem;

/// 辅助：按 id 在布局树中查找 LayoutBox。
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

/// 回归：CSS intrinsic sizing — `width:max-content` 的 grid 容器应收缩到其
/// max-content 宽度（2 item × (50 content + 40 padding) = 180），而非塌缩为 ~0
/// （converter MaxContent→length(0)）或填满视口（旧 Auto→fill）。
/// 验证两趟固有宽度布局（apply_intrinsic_content_sizing）把 grid 提升到 intrinsic。
#[test]
fn test_grid_width_max_content_sized_to_intrinsic() {
    // 复刻 child-border-box-and-max-content-001 结构：
    // grid(width:max-content, grid-auto-columns:1fr, column flow) > 2 item >
    // .content(width:50px)。grid intrinsic = 2×(50+40 padding) + 2 border ≈ 182。
    let html = r#"<html><body style="margin:0">
          <div id="g" style="display:grid;grid-auto-columns:1fr;grid-auto-flow:column;border:1px solid red;width:max-content">
            <div style="max-width:max-content;box-sizing:border-box;padding:10px 20px">
              <div style="width:50px;height:50px"></div>
            </div>
            <div style="max-width:max-content;box-sizing:border-box;padding:10px 20px">
              <div style="width:50px;height:50px"></div>
            </div>
          </div>
        </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let g = find("g", &doc, &result.root).expect("grid #g");
    // 不应塌缩（~2px）也不应填满（~784），应在 ~180px。
    assert!(
        g.width > 100.0,
        "width:max-content grid should be sized to intrinsic (~182px), not collapsed (got w={})",
        g.width
    );
    assert!(
        g.width < 400.0,
        "width:max-content grid should shrink-to-fit (~182px), not fill viewport (got w={})",
        g.width
    );
    assert!(
        (g.width - 182.0).abs() < 5.0,
        "expected grid width ~182px (2×(50+40)+border), got w={}",
        g.width
    );
}

/// 回归：intrinsic 不可测的 max-content 容器（纯文本 item）保持塌缩，
/// 不应被填满（验证不可测回退不会引入旧 Auto→fill 的 net -5 回归）。
#[test]
fn test_unmeasurable_max_content_does_not_fill() {
    // 纯文本 flex item 无显式宽度 → intrinsic 测量返回 None（Round C IFC 文本测量未就绪）
    // → 容器应保持塌缩（length(0)），而非填满视口。
    let html = r#"<html><body style="margin:0">
          <div id="f" style="display:flex;width:max-content"><div>text</div></div>
        </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let f = find("f", &doc, &result.root).expect("flex #f");
    // 不应填满视口（<700），证明不可测容器未被 auto-fill。
    assert!(
        f.width < 700.0,
        "unmeasurable width:max-content flex must not fill viewport (got w={}); \
         would regress 5 cases like R181c",
        f.width
    );
}

/// R324：position:fixed 须视口相对，即使位于有偏移的 positioned 祖先内。
///
/// taffy 0.7 把 fixed 当 absolute 处理（containing block = 最近 positioned 祖先），
/// 故 fixed 的 left/top 被解析为相对该祖先。`adjust_fixed_to_viewport` 须从累积
/// 祖先偏移中**扣除**（而非旧实现的「加上」），使其最终绝对坐标 = (left, top) 视口相对。
/// 旧「加上」实现仅在 parent_offset==0 时碰巧正确，对有 margin-offset 的 relative
/// 祖先内的 fixed 会 over-correct。本测试构造该场景，断言 fixed 视口相对、absolute
/// 兄弟仍祖先相对。
#[test]
fn test_fixed_is_viewport_relative_inside_offset_positioned_ancestor() {
    let html = r#"<html><body style="margin:0">
      <div style="position:relative; margin-top:100px; margin-left:50px; width:400px; height:300px">
        <div style="position:absolute; top:20px; left:20px; width:50px; height:50px"></div>
        <div style="position:fixed; top:20px; left:20px; width:50px; height:50px"></div>
      </div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 与 painter 一致的绝对坐标累积：abs = parent_offset + box.x/y，
    // 子元素 offset = abs + padding + border（本测试盒均无 padding/border，故相等）。
    fn collect(b: &crate::types::LayoutBox, ox: f32, oy: f32, out: &mut Vec<(bool, bool, f32, f32)>) {
        let ax = ox + b.x;
        let ay = oy + b.y;
        out.push((b.is_fixed, b.is_absolute, ax, ay));
        let child_ox = ax + b.padding_left + b.border_left;
        let child_oy = ay + b.padding_top + b.border_top;
        for c in &b.children {
            collect(c, child_ox, child_oy, out);
        }
    }
    let mut positions = Vec::new();
    collect(&result.root, 0.0, 0.0, &mut positions);

    let fixed = positions
        .iter()
        .find(|(f, _, _, _)| *f)
        .expect("should have a position:fixed box");
    let absolute = positions
        .iter()
        .find(|(f, a, _, _)| !*f && *a)
        .expect("should have a position:absolute box");

    // R324：fixed 视口相对 = (left 20, top 20)，不受 relative 祖先 margin(50,100) 影响
    assert!(
        (fixed.2 - 20.0).abs() < 1.0,
        "fixed x should be viewport-relative ~20, got {}",
        fixed.2
    );
    assert!(
        (fixed.3 - 20.0).abs() < 1.0,
        "fixed y should be viewport-relative ~20, got {}",
        fixed.3
    );
    // absolute 仍祖先相对 = 祖先(50,100) + (20,20) = (70,120)
    assert!(
        (absolute.2 - 70.0).abs() < 1.0,
        "absolute x should be ancestor-relative ~70, got {}",
        absolute.2
    );
    assert!(
        (absolute.3 - 120.0).abs() < 1.0,
        "absolute y should be ancestor-relative ~120, got {}",
        absolute.3
    );
}

/// R695：CSS §10.5 — 百分比 `height` 仅当包含块高度**明确指定**时解析，否则
/// compute-to-auto。
///
/// 复刻 height-percentage-005：`grandparent{height:0} > parent{auto} >
/// child{height:100%} > img{height:100%}`。parent 高度 auto（不明确），故
/// child 的 height:100% 应 compute-to-auto；进而 img 的 height:100%（CB=child
/// 亦不明确）→ auto → 固有 96px。旧实现：taffy 0.7 把 %height 回退到 CB **宽度**，
/// child/img 被拉到 ~784（满宽）。验证 `apply_indefinite_percent_height_to_auto`
/// 第二趟把 img 恢复到固有尺寸、child 恢复到内容高度。
#[test]
fn test_r695_percent_height_indefinite_cb_computes_to_auto() {
    let html = r#"<html><body style="margin:0">
        <div id="grandparent" style="height:0px">
          <div id="parent">
            <div id="child" style="height:100%">
              <img id="img" style="height:100%" />
            </div>
          </div>
        </div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    // 第一趟（空 img 尺寸）仅用于拿到 img 的 DOM NodeId（稳定）。
    let probe = engine.compute_with_img_sizes(
        &doc,
        &styles,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    let img_id = find("img", &doc, &probe.root)
        .and_then(|b| b.node_id)
        .expect("img node_id");

    // 第二趟注入 img 固有尺寸 96×96（模拟解码 black96x96.png）。
    let mut sizes: std::collections::HashMap<zero_dom::NodeId, (f32, f32)> = std::collections::HashMap::new();
    sizes.insert(img_id, (96.0, 96.0));
    let result = engine.compute_with_img_sizes(&doc, &styles, sizes, std::collections::HashMap::new());

    let img_box = find("img", &doc, &result.root).expect("img box");
    assert!(
        (img_box.height - 96.0).abs() < 2.0,
        "img height:100% on indefinite-CB should compute-to-auto → intrinsic 96px, got {} \
         (old taffy width-fallback gave ~784)",
        img_box.height
    );

    // child（height:100% on indefinite CB）→ auto → 内容（img）≈ 96px。
    let child_box = find("child", &doc, &result.root).expect("child box");
    assert!(
        (child_box.height - 96.0).abs() < 3.0,
        "child height:100% on indefinite-CB should compute-to-auto → content(img) ≈96px, got {}",
        child_box.height
    );
}

/// R695 反向回归：包含块高度**明确**时，百分比 height 必须照常解析（不被误改 auto）。
/// `parent{height:200px} > child{height:50%}` → child 应为 100px（明确 CB 解析）。
#[test]
fn test_r695_percent_height_definite_cb_still_resolves() {
    let html = r#"<html><body style="margin:0">
        <div id="parent" style="height:200px">
          <div id="child" style="height:50%"></div>
        </div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let child_box = find("child", &doc, &result.root).expect("child box");
    assert!(
        (child_box.height - 100.0).abs() < 2.0,
        "height:50% of definite-CB(200px) must resolve to 100px, got {}",
        child_box.height
    );
}

/// R699：CSS §10.5.1 — 非 BFC 块级元素 `height:auto` 且 `overflow` 计算为 `visible`
/// 时，高度只计入 in-flow 子元素，浮动子元素被显式忽略。
///
/// `#parent{height:auto;overflow:visible} > div{float:left;height:96px}`：parent 的
/// 唯一子元素是 float → 应被忽略 → parent height ≈ 0（float 溢出但本例无背景）。
/// 旧实现：taffy 把 float 当 in-flow block 计入父 content height → parent=96。
#[test]
fn test_r699_non_bfc_auto_height_ignores_float_child() {
    let html = r#"<html><body style="margin:0">
        <div id="parent" style="height:auto;overflow:visible">
          <div id="f" style="float:left;height:96px;width:96px"></div>
        </div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let parent = find("parent", &doc, &result.root).expect("parent box");
    assert!(
        parent.height < 1.0,
        "non-BFC height:auto parent should ignore float child (height ~0), got {} \
         (old taffy behavior gave ~96 from float)",
        parent.height
    );
    // float 子元素自身高度仍正确（96px），只是不贡献给父高度。
    let f = find("f", &doc, &result.root).expect("float box");
    assert!(
        (f.height - 96.0).abs() < 2.0,
        "float child's own height should still be 96px, got {}",
        f.height
    );
}

/// R699 反向回归：BFC 父（`overflow:hidden`）应**包含** float，高度不被本规则收缩到 0。
/// 防止 R699 误把 BFC 父也塌缩（establishes_bfc 守卫）。
#[test]
fn test_r699_bfc_parent_not_collapsed_by_float_exclusion() {
    let html = r#"<html><body style="margin:0">
        <div id="parent" style="overflow:hidden">
          <div style="float:left;height:96px;width:96px"></div>
        </div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let parent = find("parent", &doc, &result.root).expect("parent box");
    // overflow:hidden → BFC → 不应被 R699 塌缩到 0；应包含 float（≈96）。
    assert!(
        parent.height > 50.0,
        "BFC (overflow:hidden) parent must NOT be collapsed by R699 float-exclusion; \
         should contain float (height ~96), got {}",
        parent.height
    );
}

/// R711：block-level `position:relative` 的**百分比** top/bottom inset 被 taffy 0.7 丢弃。
///
/// 复刻 bottom-113：`#parent{height:100px} > #child{position:relative;bottom:100%}`。
/// CSS §9.4.3：bottom:100% → 向上偏移 CB 高度（100px）。旧实现 taffy 丢弃 % → 不偏移。
/// 验证 apply_block_relative_percent_insets 把 child 上移（child.y 相对 parent 内容盒 ≈ -100）。
#[test]
fn test_r711_relative_percent_bottom_inset_applied() {
    let html = r#"<html><body style="margin:0">
        <div id="parent" style="height:100px">
          <div id="child" style="position:relative;bottom:100%;height:20px"></div>
        </div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let child = find("child", &doc, &result.root).expect("child");
    // bottom:100% of parent height(100) → 上移 100。child.y（相对 parent 内容盒）≈ -100，
    // 而非 0（taffy 0.7 丢弃 percent inset 时 child.y ≈ 0）。
    assert!(
        child.y < -50.0,
        "relative bottom:100% should shift child up by ~100px (child.y ≈ -100), got {} \
         (taffy 0.7 drops percent inset → child.y would be ~0)",
        child.y
    );
}

/// R1044：inline 元素不为其 block 后代建立 containing block——block 后代的 CB 跳过 inline
/// 继承祖父级 block container（CSS §9.2.1.1 / §10.1）。故 inline span（auto height）包
/// block-level relative 子时，子的 top/bottom % 应解析到祖父 definite-CB 的高度，而非被
/// inline 的 auto height 截断为 None。复刻 position-relative-002：
/// `div(h:100) > span(position:relative) > div(position:relative;top:-100%;h:100)`。
/// green div 的 top:-100% 应解析到祖父 div 的 100px → 向上偏移 100（覆盖 red div）。
#[test]
fn test_r1044_inline_passes_through_cb_height_for_relative_percent() {
    let html = r#"<html><body style="margin:0">
        <div id="red" style="width:100px;height:100px;background:red">
          <span id="span" style="position:relative;top:100px;left:100px">
            <div id="green" style="width:100px;height:100px;background:green;position:relative;top:-100%;left:-100%"></div>
          </span>
        </div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let green = find("green", &doc, &result.root).expect("green box");
    // green top:-100% 应解析到祖父 red 的 100px → green.y（相对父 span 内容盒）≈ -100。
    // 旧实现 inline span 的 auto height 截断 cb_h → green.top:-100% 不解析 → green.y ≈ 0。
    assert!(
        green.y < -50.0,
        "green div top:-100% should resolve against grandparent red div height (100px), shifting \
         green.y by ~-100; got green.y={} (inline span was breaking the CB-height chain, leaving \
         top:-100% unresolved → green.y would be ~0)",
        green.y
    );
}

/// R1293：grid/flex item（style.height==Auto）经 stretch 拉伸到定值 track 后，其
/// relative 后代的 top/bottom % 应解析到该 **post-layout content_height**（非 None）。
/// 复刻 relative-grandchild：`grid(h:100) > div(auto-h grid item) > div(relative;top:-100%;h:100)`。
/// green 的 top:-100% 应解析到 grid item 的最终 content_height（100px）→ 向上偏移 100
/// 覆盖 red div。旧 R711 严格 gate（仅 style.height==Px 视为明确）把 auto-h grid item
/// 判 None → green top:-100% 不解析 → green.y ≈ 0（red 未被覆盖）。
/// kill-switch `ZW_RELPOS_PCT_AUTO_CB=0` 回退 R711 严格 gate（证 load-bearing）。
#[test]
fn test_r1293_relative_percent_resolves_against_stretched_grid_item_cb() {
    let html = r#"<html><body style="margin:0">
        <div id="red" style="width:100px;height:100px;background:red"></div>
        <div id="grid" style="display:grid;width:100px;height:100px">
          <div id="item">
            <div id="green" style="position:relative;height:100px;background:green;top:-100%"></div>
          </div>
        </div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let green = find("green", &doc, &result.root).expect("green box");
    // grid item auto-h 经 stretch 拉到 100px → green top:-100% 解析到 -100 → green.y（相对
    // 父 item 内容盒）≈ -100。旧 R711 严格 gate（auto-h→None）→ green.y ≈ 0（不偏移）。
    assert!(
        green.y < -50.0,
        "green top:-100% should resolve against the stretched grid item's content_height (100px), \
         shifting green.y by ~-100; got green.y={} (old R711 strict gate treated the auto-height \
         grid item as indefinite CB → top:-100% unresolved → green.y would be ~0)",
        green.y
    );
}

/// R1293 反例（css-position-3 §relpos-insets）：auto-height **block** 容器（无 height，
/// 仅 min-height 或纯 content-derived）的 relative 后代 top/bottom % **不应**解析——CB 高
/// indefinite。复刻 position-relative-006：`div(min-height:100px; 无 height) > green(top:-10000%;h:100)`。
/// green top:-10000% 不应解析（若解析则 green 飞出视口、red 露出）。R1293 精确 gate 区分
/// grid-stretch-definite（解析）vs auto-block-indefinite（不解析）；naive「用 content_height」
/// 会让此案错误解析 → position-relative-006 回归（0.63→2.71%）。default-on green.y 不偏移。
#[test]
fn test_r1293_relative_percent_not_resolved_against_indefinite_block_cb() {
    let html = r#"<html><body style="margin:0">
        <div id="red" style="width:100px;min-height:100px;background:red">
          <div id="green" style="width:100px;height:100px;background:green;top:-10000%;position:relative"></div>
        </div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let green = find("green", &doc, &result.root).expect("green box");
    // indefinite CB（auto+min-height，非 grid/flex stretch）→ top:-10000% 不解析 → green.y ≈ 0
    //（覆盖 red）。naive content_height gate 会解析 → green.y ≈ -10000（飞出）。
    assert!(
        green.y.abs() < 50.0,
        "green top:-10000% must NOT resolve against indefinite (auto+min-height) block CB; \
         green.y should stay ~0 (covering red); got green.y={} (naive content_height gate would \
         resolve → green.y ≈ -10000, regressing position-relative-006)",
        green.y
    );
}

/// R1044b：inline-level `position:relative` 的 top/bottom % 同样须解析（taffy 0.7 丢弃，
/// R850 原仅门控 block-level）。复刻 position-relative-001：
/// `div(h:100) > span(relative;top:100%;left:100%) > div(relative;top:-100px)`。
/// span 的 top:100% 应解析到 CB（red div）100px → span abs_y 下移 100；green（top:-100px）
/// 落回 red 位置（覆盖）。旧实现 inline 跳过 → span top:100% 不应用 → green 在 red 上方。
#[test]
fn test_r1044b_inline_relative_percent_inset_applied() {
    let html = r#"<html><body style="margin:0">
        <div id="red" style="width:100px;height:100px;background:red">
          <span id="span" style="position:relative;top:100%;left:100%">
            <div id="green" style="width:100px;height:100px;background:green;position:relative;top:-100px;left:-100px"></div>
          </span>
        </div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let span = find("span", &doc, &result.root).expect("span box");
    // span top:100% 应解析到 red div 100px → span.y（相对父 red 内容盒）≈ 100。
    // 旧实现 inline 跳过 R850 → span.top:100% 不应用 → span.y ≈ 0。
    assert!(
        span.y > 50.0,
        "inline span top:100% should resolve against red div height (100px), shifting span.y by \
         ~100; got span.y={} (R850 was skipping inline-level relative, leaving top:100% \
         unresolved → span.y would be ~0)",
        span.y
    );
}

/// CSS §8.3.1：min-height 溢出型块阻止末子 margin collapse-through 穿透父底部。
///
/// 复刻 margin-collapse-min-height-001 结构。规范：min-height 把 parent 撑到
/// 100px（高于内容 30px），child 的 550px margin-bottom 不应穿透 parent，footer
/// 应紧随 parent。旧实现 taffy CollapsibleMarginSet 让 550px 穿透。
#[test]
fn test_min_height_prevents_collapse_through() {
    let html = r#"<html><body style="margin:0">
        <div id="parent" style="min-height:100px">
          <div id="child" style="height:30px;margin-bottom:550px"></div>
        </div>
        <div id="footer" style="height:50px"></div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let parent = find("parent", &doc, &result.root).expect("parent box");
    let footer = find("footer", &doc, &result.root).expect("footer box");
    // parent 受 min-height 撑到 ~100px。
    assert!(
        (parent.height - 100.0).abs() < 2.0,
        "parent should be raised to min-height 100px, got {}",
        parent.height
    );
    // parent 的 margin_bottom 不应含穿透的 550px（应回到自身声明值 0）。
    assert!(
        parent.margin_bottom < 10.0,
        "parent margin_bottom should NOT include collapse-through child margin \
         (should be ~0, not 550), got {}",
        parent.margin_bottom
    );
    // footer 应紧随 parent：footer.y（相对 body 内容盒）≈ parent.y + parent.height，
    // 而非 parent.y + parent.height + 550。
    let expected_footer_y = parent.y + parent.height;
    assert!(
        (footer.y - expected_footer_y).abs() < 5.0,
        "footer should follow parent immediately (y ≈ {}, not {}+550), got footer.y={}",
        expected_footer_y,
        expected_footer_y,
        footer.y
    );
}

/// CSS §8.3.1 反向回归：min-height **小于**内容时不阻止 collapse-through。
///
/// 复刻 margin-collapse-min-height-003 结构。min-height 不生效（内容 30 > 5），故
/// child 的 margin-bottom 仍合法穿透 parent，footer 应在 parent_bottom + 50。
/// 防止本规则在 min-height 未溢出时误剥离合法 margin。
#[test]
fn test_min_height_below_content_still_collapses_through() {
    let html = r#"<html><body style="margin:0">
        <div id="parent" style="min-height:5px">
          <div id="child" style="height:30px;margin-bottom:50px"></div>
        </div>
        <div id="footer" style="height:50px"></div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let parent = find("parent", &doc, &result.root).expect("parent box");
    let footer = find("footer", &doc, &result.root).expect("footer box");
    // parent 高度由内容决定（~30），min-height:5px 不生效。
    assert!(
        (parent.height - 30.0).abs() < 3.0,
        "parent height should be driven by content (~30, min-height 5 inactive), got {}",
        parent.height
    );
    // footer 应被 child 的 50px margin 推下（合法穿透）：footer.y ≈ parent.y + 30 + 50。
    let expected_footer_y = parent.y + 30.0 + 50.0;
    assert!(
        (footer.y - expected_footer_y).abs() < 5.0,
        "footer should still be pushed by collapse-through margin (y ≈ {}, not parent_bottom), \
         got footer.y={}",
        expected_footer_y,
        footer.y
    );
}

/// CSS §8.3/§8.4：百分比 padding 相对**包含块内容宽度**解析（与元素自身宽度无关）。
///
/// taffy 0.7 的 LengthPercentage::Percent padding 解析为 0。`#cb{width:300} > #box{padding:20%}`
/// 的 box padding 应为 60px（20% of 300 CB），而非 0。验证 resolve_percentage_padding
/// 两趟预解析把百分比 padding 改写为绝对 px。
#[test]
fn test_percentage_padding_resolved_against_cb_width() {
    let html = r#"<html><body style="margin:0">
        <div id="cb" style="width:300px">
          <div id="box" style="padding:20%"></div>
        </div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let box_ = find("box", &doc, &result.root).expect("box");
    // 20% of CB content width (300) = 60px on each side.
    assert!(
        (box_.padding_top - 60.0).abs() < 1.5,
        "percentage padding-top should resolve to 60px (20% of 300 CB), got {} \
         (taffy 0.7 Percent padding bug gives 0)",
        box_.padding_top
    );
    assert!(
        (box_.padding_left - 60.0).abs() < 1.5,
        "percentage padding-left should resolve to 60px, got {}",
        box_.padding_left
    );
}

/// 百分比 padding 应相对**父级**内容宽（非元素自身宽）。`#cb{width:300} > #box{width:150;padding:20%}`
/// 的 box padding 仍为 60px（20% of CB 300），而非 30px（20% of own 150）。
#[test]
fn test_percentage_padding_uses_cb_width_not_own_width() {
    let html = r#"<html><body style="margin:0">
        <div id="cb" style="width:300px">
          <div id="box" style="width:150px;padding:20%"></div>
        </div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let box_ = find("box", &doc, &result.root).expect("box");
    assert!(
        (box_.padding_top - 60.0).abs() < 1.5,
        "percentage padding must use CB width (300) not own width (150): 60px, got {}",
        box_.padding_top
    );
}

/// CSS §10.3.3/§10.6.3：根元素（html）的固定 margin 相对初始包含块定位 border-box。
///
/// taffy 把根节点固定在 (0,0)，根的声明 margin 不被应用，致
/// `<html style="margin:50px">` 的边框盒落在视口原点而非 (50,50)
/// （abspos-containing-block-initial-009a 簇）。验证根固定 margin 位置偏移。
#[test]
fn test_root_element_margin_offsets_border_box() {
    let html = r#"<html style="margin:50px;border:10px solid black"><body></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    // 根 html 边框盒应偏移到 (50,50)（margin 50），而非 (0,0)。
    assert!(
        (result.root.x - 50.0).abs() < 1.0,
        "root border-box x should be offset by margin-left 50, got {}",
        result.root.x
    );
    assert!(
        (result.root.y - 50.0).abs() < 1.0,
        "root border-box y should be offset by margin-top 50, got {}",
        result.root.y
    );
}

/// CSS §10.1/§9.3.2：根元素 position:absolute/fixed（无 positioned 祖先）的包含块是
/// 初始包含块（视口），其 left/top Length inset 定位根 border-box。taffy 把根固定在
/// (0,0) 不解析根的 position:absolute，致 `<html style="position:absolute;left:50px;
/// top:50px">` 落 (0,0) 而非 (50,50)（abspos-containing-block-initial-009b/004a-d 簇）。
#[test]
fn test_root_abspos_inset_positions_border_box() {
    let html = r#"<html style="position:absolute;left:50px;top:50px;width:100px;height:100px"><body></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    assert!(
        (result.root.x - 50.0).abs() < 1.0,
        "abspos root border-box x should be left inset 50, got {}",
        result.root.x
    );
    assert!(
        (result.root.y - 50.0).abs() < 1.0,
        "abspos root border-box y should be top inset 50, got {}",
        result.root.y
    );
}

/// R1304：`width:min-content` 的 block 容器应收缩到其 min-content 宽度，而非塌缩为 0
///（converter MinContent→length(0)；R1018 intrinsic gate 旧仅放行 MaxContent，MinContent
/// block 被跳过 → 塌缩）。R1304 扩 gate 经 block_max_content_width 测（固定宽/原子内容
/// min==max 精确命中 table-intrinsic-size 簇；文本内容 overestimate 最宽词但优于 0）。
/// kill-switch ZW_MINCONTENT_BLOCK=0 回退旧行为（本测试 default-on PASS / kill=0 FAIL）。
#[test]
fn test_block_width_min_content_sized_to_intrinsic() {
    // block(width:min-content) > 固定宽 100px 子。min-content = max-content = 100px。
    let html = r#"<html><body style="margin:0">
          <div id="t" style="width:min-content">
            <div style="width:100px;height:50px"></div>
          </div>
        </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let t = find("t", &doc, &result.root).expect("block #t");
    // 不应塌缩（<10）也不应填满视口（>400）；应在 ~100px（固定宽子 min==max）。
    assert!(
        (t.width - 100.0).abs() < 5.0,
        "width:min-content block should size to intrinsic (~100px, fixed-width child), \
         not collapse to 0 (got w={}); R1304 min-content block gate",
        t.width
    );
}
