//! R5483：多行 float 容器行盒落盘（行盒归属主案多行 float 切片，R5475 行数推广）。
//!
//! floats-149 形多行推广：float span 文本折成 ≥2 行时（8 词 / float 宽 50% / 4 行），
//! 旧 store 门 R355 臂 `is_floated && lines > 1` 无条件拒存 → span 无 inline_layout
//! （行盒消费面失据与单行修复前同态）。chromium oracle 探针定谳（google-chrome
//! headless 截图 PIL 扫描，会话临时探针页）：单/多行同模型——float 行盒归属自身
//! IFC 不分行数（chromium 单行形 float w=200 h=18、多行形 w=200 h=72=4×18，均
//! blk h=0 float 溢出普通块）；ZW span IFC 多行已正确产 4 行、盒高 72 正确（float
//! extent 通道），仅 store 门拒存。修复＝放行臂增多行分支（`float_multiline_store`），
//! R355 拒存臂收窄为开关关闭态回退。
//!
//! x=-40/w=180 与 blk h≠0 为 chromium 对照下的另一域缺口（display:inline 中间层
//! 物化坐标记账＋float extent 回退臂，单/多行同缺陷、corpus self-source 绿域），
//! 非本切片域。kill-switch：`ZW_FLOAT_OWN_LINE_STORE=0`（R5475 单行臂）、
//! `ZW_FLOAT_MULTILINE_STORE=0`（本臂，R3858 独立进程锚见
//! r5483_multiline_store_kill_switch.rs）。

use zero_layout_engine::{LayoutBox, LayoutEngine};
use zero_style_system::StyleSystem;

fn sheets(html: &str) -> Vec<zero_css_parser::Stylesheet> {
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find("<style") {
        let Some(tail_start) = rest[start..].find('>') else {
            break;
        };
        let after = &rest[start + tail_start + 1..];
        let Some(end) = after.find("</style>") else { break };
        out.push(zero_css_parser::Parser::parse_stylesheet(&after[..end]));
        rest = &after[end + 8..];
    }
    out
}

fn find_box(root: &LayoutBox, id: zero_dom::NodeId) -> Option<&LayoutBox> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(id) {
            return Some(b);
        }
        stack.extend(b.children.iter());
    }
    None
}

fn span1_lines(html: &str) -> (Option<usize>, f32) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let inline_div = doc.get_element_by_id("inl").expect("div.inline 存在");
    let span_ids: Vec<zero_dom::NodeId> = doc
        .child_nodes(inline_div)
        .into_iter()
        .filter(|&c| {
            doc.get(c)
                .is_some_and(|n| matches!(n.kind, zero_dom::NodeKind::Element(_)))
        })
        .collect();
    let span1 = find_box(&result.root, span_ids[0]).expect("span1 盒存在");
    (span1.inline_layout.as_ref().map(|l| l.len()), span1.height)
}

const MULTILINE_HTML: &str = r#"<html><head><style>
body { margin: 0; font: 16px/18px Ahem, sans-serif; }
.block { display: block; background: green; width: 400px; }
.inline { display: inline; margin-left: 40px; }
span { float: left; width: 50%; background: white; }
</style></head><body>
<div class="block" id="blk">
 <div class="inline" id="inl">
  <span>Ahem Ahem Ahem Ahem Ahem Ahem Ahem Ahem</span>
  <span></span>
 </div>
</div>
<div id="after" style="background: yellow; height: 20px;"></div>
</body></html>"#;

/// 多行 float 行盒落盘：span1 IFC 产 4 行（8 词 / float 宽 50% 折 4 行 × 18px）、
/// 盒高 72，inline_layout = Some(4)（chromium oracle 同模型；pre-fix R355 臂拒存
/// = None 红证据谱系）。
#[test]
fn r5483_multiline_float_stores_its_line_boxes() {
    let (lines, h) = span1_lines(MULTILINE_HTML);
    assert_eq!(
        lines,
        Some(4),
        "多行 float 应自持 4 行行盒（R5483 放行；chromium oracle 同模型），实际 {lines:?}"
    );
    assert!(
        (h - 72.0).abs() <= 0.5,
        "float 盒高应为 4×18=72（float extent 通道 pre-fix 已正确），实际 {h}"
    );
}

/// 单行 float（R5475 域）不因本切片回归：同形 2 词单行 Some(1)。
#[test]
fn r5483_singleline_float_regression_guard() {
    let html = MULTILINE_HTML.replace("Ahem Ahem Ahem Ahem Ahem Ahem Ahem Ahem", "Ahem Ahem");
    let (lines, h) = span1_lines(&html);
    assert_eq!(lines, Some(1), "单行 float 维持 R5475 放行，实际 {lines:?}");
    assert!((h - 18.0).abs() <= 0.5, "单行 float 盒高 18，实际 {h}");
}
