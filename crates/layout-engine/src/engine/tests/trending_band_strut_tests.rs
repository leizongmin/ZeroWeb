//! t-trending（CSS2 §9.2.1.1 + §10.8.1）：含块级子的 inline 元素 flatten 泄漏块子文本
//! 修复回归——block 容器 [块级 fixed 子][空白文本][inline>block] 形态下，空白段匿名块
//! 行盒曾因载体 span 把块子文本扁平进 IFC（`text="b"` run）获得 line-height strut
//! （14×1.5=21px）= github trending 页顶黑带根因。修复后载体仅产零宽空 run
//!（§10.8 空 inline 语义），孤立行由 R1476/R57 M3 塌零。
//!
//! 探针定谳链（band-bisect2 v54→v55）：SEG_IFC(span2) frags=["text=\"b\" w=8.8"] →
//! 修复后 frags=["text=\"\" w=0.0"]；FRAG_BOX 18→0；渲染器 f1/f2/f5 21→0。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::{Document, NodeKind};
use zero_style_system::StyleSystem;

fn find_box<'a>(id: &str, doc: &Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
    if let Some(nid) = b.node_id
        && let Some(n) = doc.get(nid)
        && let NodeKind::Element(elem) = &n.kind
        && elem.get_attribute("id").as_deref() == Some(id)
    {
        return Some(b);
    }
    b.children.iter().find_map(|c| find_box(id, doc, c))
}

fn find_first_fragment(b: &LayoutBox) -> Option<&LayoutBox> {
    if b.fragment_node_ids.is_some() {
        return Some(b);
    }
    b.children.iter().find_map(find_first_fragment)
}

fn f2_shape_html() -> String {
    // 内联样式（探针阶段实测 <style> 块在本 harness 未生效——B2_STYLE height=Auto；
    // 内联属性确保断言几何确定）。
    r##"<html><head></head><body style="margin:8px">
<div id="f2"><span style="display:flex;position:fixed;height:2px;width:100%"></span>
<span style="display:inline"><div id="b2" style="height:50px;background:#24292f">b</div></span></div>
    </body></html>"##
        .to_string()
}

/// 修复主断言：空白+块级 fixed+inline>block 形态——匿名块片段行盒塌零（strut 消失），
/// banner 块盒顶到容器顶（y=0），容器高 = banner 高（无 21px 带增量）。
#[test]
fn test_trending_block_child_flatten_no_strut_line() {
    let html = f2_shape_html();
    let doc = zero_dom::parse_html(&html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(1280.0, 800.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(1280.0, 800.0);
    let result = engine.compute(&doc, &styles);

    // ① R109 匿名块片段（空白段）行盒高 0——旧行为 18/21px strut（页顶黑带）。
    let frag = find_first_fragment(&result.root).expect("R109 anon fragment");
    assert_eq!(
        frag.height, 0.0,
        "t-trending: whitespace+OOF anonymous fragment line box must collapse to 0 \
         (block-child text leaked into the carrier IFC produced a line-height strut); got {}",
        frag.height
    );

    // ② banner 块盒 y=0（不被 strut 行推下）。
    let b2 = find_box("b2", &doc, &result.root).expect("banner div");
    assert_eq!(b2.y, 0.0, "banner must sit at wrapper top, got y={}", b2.y);

    // ③ 容器 #f2 高 = banner 高（块片段计入，无多余带）。
    let f2 = find_box("f2", &doc, &result.root).expect("wrapper f2");
    assert_eq!(
        f2.height, 50.0,
        "wrapper height must equal banner height (50), got {}",
        f2.height
    );

    // ④ 块级 fixed 子保持离流静态位（y=0, 自身高 2），不占流空间。
    let fixed_span = frag
        .children
        .iter()
        .find(|c| c.is_fixed)
        .expect("fixed span box inside fragment");
    assert_eq!(fixed_span.y, 0.0);
    assert_eq!(fixed_span.height, 2.0);
}

/// 对照组：可见文本的 inline>block 形态行盒保留——inline 前驱文本（"a"）所在匿名块
/// 仍计行高，块子文本只归块盒（双计/全塌都不允许）。
#[test]
fn test_trending_inline_text_around_block_keeps_line() {
    let html = r##"<html><head></head><body style="margin:0">
<div id="w"><span style="display:inline">a<div id="inner" style="height:40px">b</div></span>c</div>
    </body></html>"##;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let inner = find_box("inner", &doc, &result.root).expect("inner block div");
    assert_eq!(inner.height, 40.0, "block child box keeps own height");
    let w = find_box("w", &doc, &result.root).expect("wrapper");
    // 容器高 = 前驱文本行 + 块盒 40 + 后继文本行（行数 ≥1 各 ~16-21px，显著 > 40 即
    // 证明文本行未塌、块盒计入；旧 flatten 双计形态该值会再 +一行）。
    assert!(
        w.height > 40.0 && w.height < 40.0 + 3.0 * 40.0,
        "wrapper height {} should be block child + text lines (no double count, no collapse)",
        w.height
    );
}

/// 在流块级子判定排除离流（fixed/absolute）块子——css-position position-relative-003
/// 形态（relative span>span>fixed 块子，crbug 1058690）：离流子不入块流、不触发
/// §9.2.1.1 拆分，其载体空白文本必须维持旧扁平化口径（置空会移 OOF 静态位锚线）。
/// 几何钉值 = 旧扁平化基线探针（ZW_BLOCK_CHILD_NO_FLATTEN=0）逐值一致。
#[test]
fn test_trending_oof_block_child_carrier_keeps_flatten() {
    let html = r##"<html><head></head><body style="margin:0">
<div id="w" style="width:100px;height:100px;background:red"><span style="position:relative;top:100%;left:100%"><span style="position:relative;top:-50px;left:-50px"><div id="g" style="position:fixed;width:100px;height:100px;background:green"></div></span></span></div>
    </body></html>"##;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    // 离流块子静态位不受载体 flatten 置空影响：绿色方块 (0,0) 100x100 恰好盖住红色
    // 参照（reftest ref-filled-green-100px-square 语义）。
    let g = find_box("g", &doc, &result.root).expect("fixed green div");
    assert_eq!((g.x, g.y, g.width, g.height), (0.0, 0.0, 100.0, 100.0));
    let w = find_box("w", &doc, &result.root).expect("wrapper");
    assert_eq!((w.x, w.y, w.width, w.height), (0.0, 0.0, 100.0, 100.0));
}
