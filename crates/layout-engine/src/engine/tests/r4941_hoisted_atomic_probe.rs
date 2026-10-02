//! R4941 修复探针：visufx/overflow-applies-to-001 结构复刻（inline style 形态——本
//! harness 不加载 <style> 表）——提升原子（img in inline）盒几何 + IFC 片段对齐验证。
//! #[ignore]（诊断用，不入常规门禁）。

use crate::engine::LayoutEngine;
use zero_style_system::StyleSystem;

#[test]
#[ignore]
fn r4941_probe_hoisted_img_geometry() {
    let html = r#"<html><body style="margin:0"><div style="width:320px;background:red;text-align:center">
  <div id="child1" style="width:160px;height:19px;background:green"><div id="gc1" style="width:320px;height:19px;background:green"></div></div>
  <div id="child2" style="width:320px;height:19px;background:green;overflow:hidden"><div id="gc2" style="width:320px;height:19px"></div></div>
  <span id="child3" style="background:red"><img id="gc3" style="width:320px;height:19px;vertical-align:bottom"></span>
  <div id="child4" style="height:19px">Block 4</div>
  </div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    fn find(b: &crate::types::LayoutBox, doc: &zero_dom::Document, id: &str) -> Option<(f32, f32, f32, f32)> {
        if let Some(nid) = b.node_id
            && doc.get_attribute(nid, "id").is_some_and(|v| v == id)
        {
            return Some((b.x, b.y, b.width, b.height));
        }
        for c in &b.children {
            if let Some(hit) = find(c, doc, id) {
                return Some(hit);
            }
        }
        None
    }
    for id in ["child1", "child2", "gc3", "child4"] {
        println!("{} = {:?}", id, find(&result.root, &doc, id));
    }
}
