//! R1771 §8.3.1 no-clearance containment（margin-collapse-clear-016）测试。
//!
//! 根因：`clear:both` 空块（无前置 float → clearance=0）的 margin-top 应 collapse-through
//! 到父底（§8.3.1），不建立父 content 高。但 taffy 对 `clear` 空块不 collapse-through 其 mt
//!（clear 特判）→ 子定位 y=flow_bottom+mt；R1743 父高回填 fold `max(child.y+child.height)`
//! 含该空子底（y+0）→ 父高被误扩（016：sibling h:100 + empty clear mt:100 → 父 200 露红，
//! 应 100）。修复 = R1743 fold（postprocess.rs `shift_siblings_after_ifc_grow`）排除
//! `is_empty_block` 子（env `ZW_CLEARANCE_NO_FLOAT_CONTAINMENT=1`，default-off）。
//!
//! 镜像 WPT margin-collapse-clear-016：`#parent-block`(mb:0) 含 `#sibling`(h:100) +
//! `#element-without-clearance`(clear:both mt:100 空)。parent content_height 应 ≈ 100（仅
//! green sibling），非 200（不含 collapsed-through mt）。
use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use std::collections::HashMap;
use zero_dom::{Document, NodeId, NodeKind};
use zero_style_system::StyleSystem;

fn compute(html: &str) -> (Document, LayoutBox, HashMap<NodeId, zero_style_system::ComputedStyle>) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let root = engine.compute(&doc, &styles).root;
    (doc, root, styles)
}

/// 找指定 id 的 LayoutBox（深度优先，按 `id` 属性）。
fn find_by_id<'a>(doc: &'a Document, b: &'a LayoutBox, id: &str) -> Option<&'a LayoutBox> {
    if let Some(nid) = b.node_id
        && let Some(node) = doc.get(nid)
        && let NodeKind::Element(e) = &node.kind
        && e.get_attribute("id")
            .map(|s| s.eq_ignore_ascii_case(id))
            .unwrap_or(false)
    {
        return Some(b);
    }
    for c in &b.children {
        if let Some(f) = find_by_id(doc, c, id) {
            return Some(f);
        }
    }
    None
}

/// 016 结构：parent-block(mb:0) + sibling(h:100) + empty clear:both(mt:100)。
/// 开启 `ZW_CLEARANCE_NO_FLOAT_CONTAINMENT` 后 parent content_height 应 ≈ 100（仅 sibling），
/// 不含 collapsed-through mt:100（旧 bug 父 200 露红）。
#[test]
fn r1771_clear_empty_no_clearance_containment() {
    // 测试与并行用例共享进程 env；本 gate 仅排除 is_empty_block 子，对无 trailing 空块容器
    // 字节等价（零影响），故临时 set_var 安全。
    // Safety：单测环境，无其他线程并发读该 env（test单线程跑此 module 的串行子集）。
    unsafe { std::env::set_var("ZW_CLEARANCE_NO_FLOAT_CONTAINMENT", "1") };
    let html = r#"<html><body>
      <div id="parent-block" style="background-color:red;margin-bottom:0">
        <div id="sibling" style="background-color:green;height:100px"></div>
        <div id="element-without-clearance" style="clear:both;margin-top:100px"></div>
      </div>
    </body></html>"#;
    let (doc, root, _) = compute(html);
    let parent = find_by_id(&doc, &root, "parent-block").expect("parent-block");
    // sibling h:100 → parent content 应 ≈ 100（empty clear element mt collapse-through 出父）。
    // 旧 bug：taffy 不折叠 clear 空块 mt + R1743 fold 含空子底 → parent ≈ 200。
    assert!(
        parent.content_height < 130.0,
        "parent content_height={:.1} 应 ≈ 100（仅 sibling；empty clear mt collapse-through 出父，旧 bug 200 露红）",
        parent.content_height
    );
    assert!(
        parent.content_height >= 99.0,
        "parent content_height={:.1} 应 ≥ 99（含 sibling h:100）",
        parent.content_height
    );
}

/// 回归守卫：trailing 空块**无 clear**（普通 collapse-through 空块）父高亦不含其 mt。
/// 镜像 R1771 修复对普通空块的同等处理（is_empty_block 不分 clear/非 clear）。
#[test]
fn r1771_empty_block_no_clear_also_excluded() {
    // Safety：同上，单测环境无并发线程读该 env。
    unsafe { std::env::set_var("ZW_CLEARANCE_NO_FLOAT_CONTAINMENT", "1") };
    let html = r#"<html><body>
      <div id="parent" style="background-color:red">
        <div id="sibling" style="background-color:green;height:100px"></div>
        <div id="empty-trailing" style="margin-top:100px"></div>
      </div>
    </body></html>"#;
    let (doc, root, _) = compute(html);
    let parent = find_by_id(&doc, &root, "parent").expect("parent");
    assert!(
        parent.content_height < 130.0,
        "parent content_height={:.1} 应 ≈ 100（普通空块 trailing 亦 collapse-through，mt 不进 content）",
        parent.content_height
    );
}
