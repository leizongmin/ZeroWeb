//! R4037：abspos `height: fit-content` 内容底改子树递归（css-position-3 §3 abspos
//! auto-size + css-sizing-3 fit-content）。
//!
//! 中间层百分比高子（height:100%）在 taffy 首趟因父高塌 0 而自身塌 0，其固定高孙溢出
//! 在外——旧实现 `fix_abspos_height_content_keyword` 只看直接子 bottom → content_h=0
//! 不抬升（abspos-auto-sizing-fit-content-percentage ×17 @2.08% 同值簇 + intrinsic-
//! height-abspos-percentage-child-002 连带）。fit-content/max-content 按全部内容度量，
//! 固定高后代照样贡献。

use std::sync::Arc;

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

fn layout(html: &str) -> (zero_dom::Document, LayoutBox) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut eng = LayoutEngine::new(800.0, 600.0);
    let r = eng.compute(&doc, &styles);
    let root = Arc::try_unwrap(r.root).unwrap_or_else(|arc| (*arc).clone());
    (doc, root)
}

fn find_div<'a>(root: &'a LayoutBox, doc: &zero_dom::Document, class: &str) -> &'a LayoutBox {
    fn walk<'a>(b: &'a LayoutBox, doc: &zero_dom::Document, class: &str) -> Option<&'a LayoutBox> {
        if let Some(nid) = b.node_id
            && let Some(n) = doc.get(nid)
            && let zero_dom::NodeKind::Element(e) = &n.kind
            && e.get_attribute("class").is_some_and(|c| c == class)
        {
            return Some(b);
        }
        b.children.iter().find_map(|c| walk(c, doc, class))
    }
    walk(root, doc, class).expect("div with class")
}

/// driving: abspos-auto-sizing-fit-content-percentage-005——abspos height:fit-content，
/// 中间层 height:100% 子 + 固定 100px 孙 → fit-content 应取到孙的 100px（百分比子按
/// indefinite 解析），而非塌 0。
#[test]
fn r4037_abspos_fitcontent_height_lifts_to_subtree_content() {
    let (doc, root) = layout(
        r#"<html><body>
        <div style="position:relative;width:100px;height:50px;">
          <div class="abs" style="position:absolute;top:0;left:0;height:fit-content;width:100px;background:green;">
            <div style="height:100%;">
              <div style="height:100px;"></div>
            </div>
          </div>
        </div></body></html>"#,
    );
    let abs = find_div(&root, &doc, "abs");
    assert!(
        (abs.height - 100.0).abs() < 1.0,
        "fit-content 应经子树递归取到固定高孙 100px，got {}",
        abs.height
    );
}

/// 直接子已够高时行为不变（R4012 既有绿面回归锚）。
#[test]
fn r4037_abspos_fitcontent_direct_child_still_lifts() {
    let (doc, root) = layout(
        r#"<html><body>
        <div style="position:relative;width:100px;height:50px;">
          <div class="abs" style="position:absolute;top:0;left:0;height:fit-content;width:100px;">
            <div style="height:80px;"></div>
          </div>
        </div></body></html>"#,
    );
    let abs = find_div(&root, &doc, "abs");
    assert!(
        (abs.height - 80.0).abs() < 1.0,
        "直接子 80px 仍应抬升（R4012 基线），got {}",
        abs.height
    );
}
