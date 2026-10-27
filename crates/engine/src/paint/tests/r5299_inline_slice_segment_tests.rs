//! R5299（css-writing-modes-3 #bidi-box-model）：单行整盒路径的裂解段表
//! [owner_line_segment_extent_map]——行连续域在「盒裂解缝」处分段。
//!
//! bidi 重排可把 owner 的两个视觉片段拉开、其间楔入外来内容（bidi-001：RLO
//! 把 `<p>` 直辖文本搬进 span 裂解缝），行连续域会跨缝桥接、横条/bg 盖住缝内
//! 外来文本。分段判据：缝内楔入**非子孙** owner 的片段即切段；缝内仅有子孙
//! 片段（子 inline 穴）或空缝（词 run 间隙）保持连续；祖先楔入同样切段
//! （祖先带连续性由其自身表项维护）。段附 (has_first, has_last) 逻辑首/末
//! 标记（边归属按逻辑序，非 x 序）。

use crate::paint::painter::text::{owner_line_segment_extent_map, r5299_is_owner_descendant};

/// 裂解缝被非子孙（祖先 p 直辖文本，bidi-001 拓扑）楔入 → 切两段；首段
/// has_first、末段 has_last（标记按逻辑迭代序，非 x 序）。
#[test]
fn segment_map_splits_at_foreign_interleave() {
    let mut doc = zero_dom::Document::new();
    let p = doc.create_element("p");
    let span = doc.create_element("span");
    let t1 = doc.create_text_node("AAABBBCCC");
    let t2 = doc.create_text_node("IIIHHHGGG");
    let tp = doc.create_text_node("FFFEEEDDD");
    doc.append_child(p, span).unwrap();
    doc.append_child(span, t1).unwrap();
    doc.append_child(span, t2).unwrap();
    doc.append_child(p, tp).unwrap();
    // 迭代序 = 逻辑序：t1（16..200）、p 直辖 tp（216..389，重排后视觉楔入缝内）、
    // t2（400..485）。
    let frags = [
        (t1, 0.0_f32, 16.0_f32, 184.0_f32),
        (tp, 0.0, 216.0, 173.0),
        (t2, 0.0, 400.0, 85.0),
    ];
    let segs = owner_line_segment_extent_map(frags.into_iter(), &doc, None);
    let span_segs = segs
        .iter()
        .find(|((owner, _), _)| *owner == span)
        .map(|(_, s)| s)
        .expect("span 表项");
    assert_eq!(
        span_segs,
        &vec![(16.0, 200.0, true, false), (400.0, 485.0, false, true)],
        "外来楔入应切段且首/末标记按逻辑序"
    );
    // 祖先 p 自身表项：缝内楔入的 span 片段是 p 的子孙 → p 带保持连续
    //（祖先带连续性由自身表项维护，bidi-001 的 <p> 无装饰故无害）。
    let p_segs = segs
        .iter()
        .find(|((owner, _), _)| *owner == p)
        .map(|(_, s)| s)
        .expect("p 表项");
    assert_eq!(p_segs, &vec![(16.0, 485.0, true, true)], "p 视角缝内皆子孙");
}

/// 缝内仅有子孙 inline（b > text，b 为 span 子）→ 保持连续一段（子 inline 穴
/// 属 owner 领土）；b 自身表项单独分段。
#[test]
fn segment_map_merges_descendant_hole() {
    let mut doc = zero_dom::Document::new();
    let span = doc.create_element("span");
    let d1 = doc.create_text_node("aa ");
    let b = doc.create_element("b");
    let tb = doc.create_text_node("bb");
    let d2 = doc.create_text_node(" cc");
    doc.append_child(span, d1).unwrap();
    doc.append_child(span, b).unwrap();
    doc.append_child(b, tb).unwrap();
    doc.append_child(span, d2).unwrap();
    let frags = [
        (d1, 0.0_f32, 0.0_f32, 50.0_f32),
        (tb, 0.0, 60.0, 20.0), // 子 inline 穴（owner b = span 子孙）
        (d2, 0.0, 90.0, 30.0),
    ];
    let segs = owner_line_segment_extent_map(frags.into_iter(), &doc, None);
    let span_segs = segs
        .iter()
        .find(|((owner, _), _)| *owner == span)
        .map(|(_, s)| s)
        .expect("span 表项");
    assert_eq!(span_segs, &vec![(0.0, 120.0, true, true)], "子孙穴应并入连续段");
}

/// 空缝（词 run 间隙，无任何片段楔入）→ 合并；单段 owner 首末同段双真。
#[test]
fn segment_map_merges_empty_word_gap() {
    let mut doc = zero_dom::Document::new();
    let p = doc.create_element("p");
    let t1 = doc.create_text_node("aa");
    let t2 = doc.create_text_node("bb");
    doc.append_child(p, t1).unwrap();
    doc.append_child(p, t2).unwrap();
    let frags = [(t1, 0.0_f32, 0.0_f32, 40.0_f32), (t2, 0.0, 48.0, 40.0)];
    let segs = owner_line_segment_extent_map(frags.into_iter(), &doc, None);
    let p_segs = segs
        .iter()
        .find(|((owner, _), _)| *owner == p)
        .map(|(_, s)| s)
        .expect("p 表项");
    assert_eq!(p_segs, &vec![(0.0, 88.0, true, true)], "空缝合并、首末同段");
}

/// 子孙判定：自身真、直接子真、祖先链跨级真、祖先反向假。
#[test]
fn owner_descendant_walk() {
    let mut doc = zero_dom::Document::new();
    let p = doc.create_element("p");
    let span = doc.create_element("span");
    let b = doc.create_element("b");
    doc.append_child(p, span).unwrap();
    doc.append_child(span, b).unwrap();
    assert!(r5299_is_owner_descendant(&doc, b, span), "直接子");
    assert!(r5299_is_owner_descendant(&doc, b, p), "跨级子孙");
    assert!(r5299_is_owner_descendant(&doc, span, span), "自身");
    assert!(!r5299_is_owner_descendant(&doc, p, span), "祖先非子孙");
}
