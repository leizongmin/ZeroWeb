//! R4499 diagnostic：multicol-span-all-001 spanner 区域 inline 内容的 box 树结构
//! + span-all 参照页 inline-block 幻影高再现探针。
//!
//! 变体 a：multicol 容器 + spanner + 两区域 inline 内容（R3893 匿名块片段盒结构：
//! host node_id + fragment_node_ids，区域列平衡 slice 的处理对象）。
//! 变体 b：参照页近似结构（inline-block 容器 + 3 block 行 + 行间空白文本）——
//! 容器高幻影膨胀（行真和 100 vs 容器 180）再现，span-all 族 ref 侧残差的根因复现体。
//! 变体 c：同 b 但 div 间零空白（容器 140 正确）——界定幻影来自 block 间内容，
//! 非 inline-block 盒模型自身。
//! 注：probe 走 inline style（engine.compute 不收 <style> 块），字体用默认栈
//! （行高与 reftest 管线的 Ahem 度量不同），结构面一致。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

const PAGE: &str = r#"<!DOCTYPE html>
<html><body style="margin:0">
<div style="background-color: yellow; border: 20px solid gray; color: navy;
      font-size: 20px; line-height: 1; width: 220px;
      column-count: 4; column-gap: 20px;">
  <span style="color: blue"> bl ue bl ue </span>
  <span style="color: pink"> Pi nk Pi nk </span>
  <h4 style="background-color: black; color: black; font: inherit; margin: 0;
     column-span: all"> sPana </h4>
  ab cd ef gh
  ij kl mn oq
</div>
</body></html>"#;

fn label(b: &LayoutBox, doc: &zero_dom::Document) -> String {
    match b.node_id.and_then(|id| doc.get(id)) {
        Some(n) => match &n.kind {
            zero_dom::NodeKind::Element(e) => {
                let id_attr = e.get_attribute("id").map(|v| format!("#{v}")).unwrap_or_default();
                format!("<{}{}>", e.local_name(), id_attr)
            }
            zero_dom::NodeKind::Text(t) => format!("#text({:?})", t.content.chars().take(10).collect::<String>()),
            _ => "?".to_string(),
        },
        None => "<anon>".to_string(),
    }
}

fn dump(b: &LayoutBox, doc: &zero_dom::Document, depth: usize) {
    let indent = "  ".repeat(depth);
    let lines_info = b
        .inline_layout
        .as_ref()
        .map(|lines| {
            let hs: Vec<String> = lines
                .iter()
                .map(|l| format!("y={:.0}/h={:.0}", l.y, l.height))
                .collect();
            let ws: Vec<String> = lines
                .iter()
                .map(|l| {
                    l.fragments
                        .iter()
                        .map(|f| format!("{:.0}@{:.0}", f.width, f.x))
                        .collect::<Vec<_>>()
                        .join("|")
                })
                .collect();
            format!(" lines[{}]={} widths[{}]", lines.len(), hs.join(","), ws.join(" ;; "))
        })
        .unwrap_or_default();
    eprintln!(
        "{}{} w={:.0} cw={:.0} h={:.0} x={:.0} y={:.0} nid={:?} r109split={} blockmixed={} fragids={} cso={} ilw={:.0}{}",
        indent,
        label(b, doc),
        b.width,
        b.content_width,
        b.height,
        b.x,
        b.y,
        b.node_id,
        b.is_r109_split,
        b.is_r109_block_mixed,
        b.fragment_node_ids.is_some(),
        b.column_span_offsets.len(),
        b.inline_layout_width,
        lines_info
    );
    for c in &b.children {
        dump(c, doc, depth + 1);
    }
}

#[test]
fn r4499_dump_spanner_region_structure() {
    let doc = zero_dom::parse_html(PAGE);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut eng = LayoutEngine::new(800.0, 600.0);
    let r = eng.compute(&doc, &styles);
    eprintln!("=== R4499 span-all-001 box tree ===");
    dump(&r.root, &doc, 0);

    // R4499b：ref 页 inline-block 幻影高再现探针（multicol-span-all-001-ref 结构近似：
    // inline-block 容器 + 3 个 block 行，行内 inline-block 盒模拟 40px 图）。
    let ref_like = r#"<html><body style="margin:0">
<div style="display:inline-block; background:yellow; border:20px solid gray; font-size:20px;">
  <div style="line-height:1"><span style="display:inline-block;width:40px;height:40px;background:blue;vertical-align:top"></span><span style="display:inline-block;width:40px;height:40px;background:blue;vertical-align:top"></span></div>
  <div style="line-height:1"><span style="display:inline-block;width:220px;height:20px;background:black;vertical-align:top"></span></div>
  <div style="line-height:1"><span style="display:inline-block;width:40px;height:40px;background:navy;vertical-align:top"></span><span style="display:inline-block;width:40px;height:40px;background:navy;vertical-align:top"></span></div>
</div>
</body></html>"#;
    let doc2 = zero_dom::parse_html(ref_like);
    let styles2 = sys.compute_styles(&doc2, &[]);
    let mut eng2 = LayoutEngine::new(800.0, 600.0);
    let r2 = eng2.compute(&doc2, &styles2);
    eprintln!("=== R4499b ref-like inline-block tree ===");
    dump(&r2.root, &doc2, 0);

    // R4499c：无空白变体（div 间零空白）——区分「inter-block 空白行盒」与其它来源。
    let ref_no_ws = r#"<html><body style="margin:0">
<div style="display:inline-block; background:yellow; border:20px solid gray; font-size:20px;"><div style="line-height:1"><span style="display:inline-block;width:40px;height:40px;background:blue;vertical-align:top"></span></div><div style="line-height:1"><span style="display:inline-block;width:220px;height:20px;background:black;vertical-align:top"></span></div><div style="line-height:1"><span style="display:inline-block;width:40px;height:40px;background:navy;vertical-align:top"></span></div></div>
</body></html>"#;
    let doc3 = zero_dom::parse_html(ref_no_ws);
    let styles3 = sys.compute_styles(&doc3, &[]);
    let mut eng3 = LayoutEngine::new(800.0, 600.0);
    let r3 = eng3.compute(&doc3, &styles3);
    eprintln!("=== R4499c no-whitespace inline-block tree ===");
    dump(&r3.root, &doc3, 0);
}
