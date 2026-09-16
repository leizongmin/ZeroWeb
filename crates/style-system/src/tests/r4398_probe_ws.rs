//! R4398：type-selector white-space 声明级联回归守护——`i { white-space: pre-line }`
//! 须产出 WhiteSpaceValue::PreLine（pre-line 断行 identity 的 style 层前提；
//! pre-line-with-space-and-newline 断行修复时序证）。stylesheet 经参数传入
//!（compute_styles 的 `<style>` 提取由 engine pipeline 负责，与 render 路径同构）。
#[test]
fn r4398_probe_ws_cascade() {
    let doc = zero_dom::parse_html(
        r#"<html><head><style>i { white-space: pre-line; font-weight: bold; }</style></head><body><i>a&#10;b</i></body></html>"#,
    );
    let mut sys = crate::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let ss = zero_css_parser::Parser::parse_stylesheet("i { white-space: pre-line; font-weight: bold; }");
    let styles = sys.compute_styles(&doc, &[ss]);
    let mut found = None;
    for (id, s) in &styles {
        if let Some(n) = doc.get(*id) {
            if let zero_dom::NodeKind::Element(e) = &n.kind {
                if e.local_name() == "i" {
                    found = Some((format!("{:?}", s.white_space), format!("{:?}", s.font_weight)));
                }
            }
        }
    }
    eprintln!("[R4398] i computed = {:?}", found);
    let (ws, fw) = found.expect("i element styled");
    assert!(ws.contains("PreLine"), "white-space should be PreLine, got {ws}");
    assert!(
        fw.to_lowercase().contains("bold"),
        "font-weight should be bold, got {fw}"
    );
}
