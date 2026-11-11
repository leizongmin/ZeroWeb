//! R5498 主判例：根帧转置——根元素书写模式为 vertical 时根节点亦交换进 taffy
//! （css-writing-modes-3 §7.1 根元素书写模式传播至 ICB：根的 CB（ICB）同为 vertical
//! 帧）。此前根节点按父=视口 htb 不交换（tree.rs 轴交换 gate=父书写模式），
//! html→body 形成**混合帧对**：taffy 把根 auto 物理宽拉伸到 ICB 宽（800），body
//! （交换帧）auto 行内消费该物理宽而非 ICB 行内轴（物理高）——viv 根链行内 fill
//! 全走错轴（R5497 缺陷 4 定谳：探针 V1 body=784=800−16 随 ICB 宽、V3 随视口宽
//! 不随高；chromium fresh oracle vrl-007 全链 fill ICB 高链 div/p=497=513−16）。
//!
//! 修复＝根帧转置三件：根节点亦交换进 taffy ＋ 根 available space 转置
//!（taffy 宽=ICB 物理高 / 高=ICB 物理宽）＋ 根 extract/后处理 walk 根父书写模式
//! 同源（engine.rs R4473 交换版 extract 同族补 root 特例）。
//! 效果（800×600）：html 物理高 fill 600、物理宽 content shrink；body/div 级联
//! 584、p 552——与 chromium 全链 fill 一致（R5497 fresh oracle 证级联下游自洽，
//! 单点修复）。
//!
//! 同源 kill-switch 回退锚见 r5498_root_frame_transpose_kill_switch.rs
//!（独立进程，R3858 教训）。谱系：R5497 归因（四变探针＋fresh oracle）、
//! R5460 §4「html root sizing 链」（同一缺陷升格）、R4473（交换版 extract）、
//! R4446（正交域同形错位已修，viv 根对为根特例）。

use zero_layout_engine::types::LayoutBox;
use zero_layout_engine::{LayoutEngine, LayoutResult};
use zero_style_system::StyleSystem;

const BASE_CSS: &str = "html { writing-mode: vertical-rl; } body { margin: 8px; }";
const PROBE_HTML: &str = "<html><head><style>{CSS}</style></head>\
<body><div><p>Sentence before.</p></div></body></html>";

fn compute(css: &str, html: &str, vw: f32, vh: f32) -> LayoutResult {
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = StyleSystem::new();
    sys.set_viewport(f64::from(vw), f64::from(vh));
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(vw, vh);
    engine.compute(&doc, &styles)
}

/// 按标签先序找盒（head 无布局盒自然 miss）。
fn find_by_tag<'a>(node: &'a LayoutBox, tag: &str, doc: &zero_dom::Document) -> Option<&'a LayoutBox> {
    let hit = node
        .node_id
        .and_then(|id| doc.get(id))
        .and_then(|n| match &n.kind {
            zero_dom::NodeKind::Element(e) => Some(e.local_name() == tag),
            _ => None,
        })
        .unwrap_or(false);
    if hit {
        return Some(node);
    }
    node.children.iter().find_map(|c| find_by_tag(c, tag, doc))
}

/// 断言 html/body/div/p 物理高（行内轴）级联 fill ICB 高链。
fn assert_inline_fill_chain(result: &LayoutResult, vw: f32, vh: f32) {
    let doc = zero_dom::parse_html(&PROBE_HTML.replace("{CSS}", "html {} body {}"));
    let html = find_by_tag(&result.root, "html", &doc).expect("html 盒");
    let body = find_by_tag(&result.root, "body", &doc).expect("body 盒");
    let div = find_by_tag(&result.root, "div", &doc).expect("div 盒");
    let p = find_by_tag(&result.root, "p", &doc).expect("p 盒");
    // 根帧转置核心断言：html 物理高（行内轴）= fill ICB 物理高，不再随 ICB 宽或
    // 内容块轴 extent（旧态 34/316 junk，R5460 §4 html root sizing 链）。
    assert!(
        (html.height - vh).abs() <= 0.5,
        "html 物理高 fill ICB 高 {vh}（错轴态随 ICB 宽/内容 extent），实得 {}",
        html.height
    );
    // body margin-box fill html content → border-box = vh − 16（行内 margin 8×2）。
    assert!(
        (body.height - (vh - 16.0)).abs() <= 0.5,
        "body 行内 fill html content（{v}−16），实得 {}",
        body.height,
        v = vh - 16.0
    );
    // div 行内 fill body content；p 减 UA 行内 margin 16×2。
    assert!(
        (div.height - (vh - 16.0)).abs() <= 0.5,
        "div 行内 fill body content（{v}−16），实得 {}",
        div.height,
        v = vh - 16.0
    );
    let _ = vw;
    assert!(
        (p.height - (vh - 16.0 - 32.0)).abs() <= 0.5,
        "p 行内 fill div content 减 UA margin 32（{v}−48），实得 {}",
        p.height,
        v = vh - 16.0
    );
}

/// R5497 探针 V1（800×600）：全链 fill ICB 高（错轴态 body/div=784、p=752 随 ICB 宽）。
#[test]
fn v1_root_chain_fills_icb_height() {
    let result = compute(BASE_CSS, &PROBE_HTML.replace("{CSS}", BASE_CSS), 800.0, 600.0);
    assert_inline_fill_chain(&result, 800.0, 600.0);
    // 块轴 extent 链（物理宽）不随本修复变化：html 物理宽维持 content shrink
    //（body 块 extent 18 + margin 16 = 34；错轴态由 shrink pass 补写，转置后 taffy
    // 交换帧原生 content sizing，同值）。
    let doc = zero_dom::parse_html(&PROBE_HTML.replace("{CSS}", "html {} body {}"));
    let html = find_by_tag(&result.root, "html", &doc).expect("html 盒");
    assert!(
        (html.width - 34.0).abs() <= 1.0,
        "html 物理宽维持 content shrink 34（块轴链不变），实得 {}",
        html.width
    );
}

/// R5497 探针 V3（500×700）：视口换轴——fill 源=ICB 物理高（700）而非物理宽
///（错轴态 484=500−16 实锤轴源错误）。
#[test]
fn v3_axis_source_follows_viewport_height() {
    let result = compute(BASE_CSS, &PROBE_HTML.replace("{CSS}", BASE_CSS), 500.0, 700.0);
    assert_inline_fill_chain(&result, 500.0, 700.0);
}

/// R5497 探针 V2（div width:300px）：块轴 definite extent（物理宽 300）不进行内
/// 轴级联（错轴态判别 html w=316 照走块轴 shrink、行内链仍 784 域）——转置后
/// 行内链维持 ICB 高 fill，块轴 extent 链原样。
#[test]
fn v2_definite_block_extent_not_inline_source() {
    let css = "html { writing-mode: vertical-rl; } body { margin: 8px; } div { width: 300px; }";
    let result = compute(css, &PROBE_HTML.replace("{CSS}", css), 800.0, 600.0);
    assert_inline_fill_chain(&result, 800.0, 600.0);
    let doc = zero_dom::parse_html(&PROBE_HTML.replace("{CSS}", "html {} body {}"));
    let html = find_by_tag(&result.root, "html", &doc).expect("html 盒");
    assert!(
        (html.width - 316.0).abs() <= 1.0,
        "html 物理宽随 div 块 extent（300+16=316，块轴链不变），实得 {}",
        html.width
    );
}

/// R5497 探针 V4（body height:200px definite 行内轴）：definite 链本就正确
///（错轴态 p=168 已对）——转置后 definite 臂不受影响（html 仍 fill ICB 高）。
#[test]
fn v4_definite_inline_cb_chain_unchanged() {
    let css = "html { writing-mode: vertical-rl; } body { margin: 8px; height: 200px; }";
    let result = compute(css, &PROBE_HTML.replace("{CSS}", css), 800.0, 600.0);
    let doc = zero_dom::parse_html(&PROBE_HTML.replace("{CSS}", "html {} body {}"));
    let html = find_by_tag(&result.root, "html", &doc).expect("html 盒");
    let body = find_by_tag(&result.root, "body", &doc).expect("body 盒");
    let div = find_by_tag(&result.root, "div", &doc).expect("div 盒");
    let p = find_by_tag(&result.root, "p", &doc).expect("p 盒");
    assert!(
        (html.height - 600.0).abs() <= 0.5,
        "html 物理高 fill ICB 高 600（definite body 不影响根行内 fill），实得 {}",
        html.height
    );
    assert!(
        (body.height - 200.0).abs() <= 0.5,
        "body definite 行内 200 保持，实得 {}",
        body.height
    );
    assert!(
        (div.height - 200.0).abs() <= 0.5,
        "div 行内 fill definite body 200，实得 {}",
        div.height
    );
    assert!(
        (p.height - 168.0).abs() <= 0.5,
        "p 行内 fill 200−32（R5497 V4 值），实得 {}",
        p.height
    );
}
