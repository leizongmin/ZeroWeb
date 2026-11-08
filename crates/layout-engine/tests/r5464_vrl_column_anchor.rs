//! R5464 主判例：vrl definite 宽容器列组右缘重排（块流基位域）。
//!
//! 病理（sizing-orthog-htb-in-vrl-007 残余 4.70%，R5463 挂账）：compute_vertical_block_flow
//! 排列基准 = Σ 子 outer 宽，R4566 definite 容器不改写物理宽 → 列组贴 Σ 右缘而非
//! CB content 右缘（本页 body 继承 html vrl，cb 属 viv 嵌套域；restack 在 11.12
//! 宽重解后跑、用新列宽，Σ=440 → sb [420,440] 溢出 CB 400 右缘 40px）。
//!
//! chromium oracle headless 探针逐位定谳（cb=280,8,400,477）：sb [660,680] 右缘
//! 贴 cb content 右缘 680、ob [260,660]、sa [240,260] 溢出向左（sa.left < cb.left）。
//! CSS Writing Modes §7.1：vrl 块流起始 = CB 行内 start 侧 = content 右缘。
//!
//! 修复落点：apply_inner restack 后整组平移 dx = 容器 content_width − Σ（margin-box
//! 基准同移，DOM 序与列距不变；溢出向左允许负 x）。gate：vrl + CSS 宽 definite
//!（同 R4566 判据）。auto 容器（016 域，extent=Σ 改写自洽）与 vlr 镜像（左缘锚定
//! 与容器宽无关）零接触——见 auto_cb_untouched / vlr_mirror_untouched。
//! kill-switch `ZW_VRL_COLUMN_CB_ANCHOR`，回退锚见
//! r5464_vrl_column_anchor_kill_switch.rs（独立进程，R3858 教训）。

use zero_layout_engine::{LayoutEngine, LayoutResult};

const CSS_007: &str = "html { writing-mode: vertical-rl; } \
body { font-size: 16px; line-height: 1.25; margin-left: 100px; margin-right: 100px; } \
div#cb { width: 400px; } \
div#ob { border: blue solid 3px; width: auto; writing-mode: horizontal-tb; } \
p { margin: 1em 0; }";

fn compute(html: &str, css: &str) -> LayoutResult {
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles)
}

fn find_ortho(root: &zero_layout_engine::types::LayoutBox) -> &zero_layout_engine::types::LayoutBox {
    fn walk(b: &zero_layout_engine::types::LayoutBox) -> Option<&zero_layout_engine::types::LayoutBox> {
        if !b.writing_mode.is_vertical_block_flow()
            && b.node_id.is_some()
            && b.is_block_level
            && b.border_left > 2.0
            && b.border_left < 4.0
        {
            return Some(b);
        }
        for c in &b.children {
            if let Some(f) = walk(c) {
                return Some(f);
            }
        }
        None
    }
    walk(root).expect("正交块盒在树")
}

/// ob 父容器（cb）直接子中 20px 单行列（sentence p）的 x 集合——列组锚定的判据面。
/// 修复后期望 {380, −40}（sb 贴 CB 右缘、sa 溢出向左）；旧 restack 域为 {86, 0}。
fn sentence_column_xs(root: &zero_layout_engine::types::LayoutBox) -> Vec<f32> {
    fn find_parent<'a>(
        b: &'a zero_layout_engine::types::LayoutBox,
        target: &zero_layout_engine::types::LayoutBox,
    ) -> Option<&'a zero_layout_engine::types::LayoutBox> {
        if b.children.iter().any(|c| std::ptr::eq(c, target)) {
            return Some(b);
        }
        for c in &b.children {
            if let Some(p) = find_parent(c, target) {
                return Some(p);
            }
        }
        None
    }
    let ob = find_ortho(root);
    let parent = find_parent(root, ob).expect("正交块父容器在树");
    let mut xs: Vec<f32> = parent
        .children
        .iter()
        .filter(|c| (c.width - 20.0).abs() < 1.0)
        .map(|c| c.x)
        .collect();
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    xs
}

fn body_html(content: &str) -> String {
    format!(
        r#"<html><body><div id="cb"><p>Sentence before.</p><div id="ob">{content}</div><p>Sentence after.</p></div></body></html>"#
    )
}

fn nums100() -> String {
    (1..=100)
        .map(|i| format!("{:02} ", if i == 100 { 0 } else { i }))
        .collect()
}

#[test]
fn definite_cb_columns_right_anchor_at_block_start() {
    let r = compute(&body_html(&nums100()), CSS_007);
    let ob = find_ortho(&r.root);
    // 列组按当前 outer 尺寸以 CB content 宽 400 右起重排：sb [380,400]（右缘贴
    // CB 右缘）、ob [−20,380]、sa [−40,−20]（溢出向左，oracle sa 实证）。
    // 旧版本失败证据：「ob 应 [−20,380] 实得 [20,420]」（Σ=440 锚定溢出 CB 右缘
    // 40px，sentence 列 {0, 420}）。
    assert!(
        (ob.x + 20.0).abs() < 0.5 && (ob.x + ob.width - 380.0).abs() < 0.5,
        "ob 应重排至 [−20,380]（右缘 = CB 右缘 − sb 列 20），实得 [{},{}]",
        ob.x,
        ob.x + ob.width
    );
    let xs = sentence_column_xs(&r.root);
    assert_eq!(xs.len(), 2, "两单行列在树");
    assert!(
        (xs[0] + 40.0).abs() < 0.5 && (xs[1] - 380.0).abs() < 0.5,
        "sentence 列应 {{sa:−40, sb:380}}（sb 右缘贴 CB 400），实得 {xs:?}"
    );
}

#[test]
fn short_group_no_overflow_right_anchored() {
    // 短文本 ob stf 收缩 202（R5463 SHORT 域）：Σ=242 < CB 400 → 组仍贴右缘、
    // 左侧留空（vrl 块流起始语义，不居中不贴左）。
    let r = compute(&body_html("01 02 03 04 05 06 07 08 09 10"), CSS_007);
    let ob = find_ortho(&r.root);
    assert!(
        (ob.x - 178.0).abs() < 0.5 && (ob.x + ob.width - 380.0).abs() < 0.5,
        "ob 应 [178,380]（右缘贴 sb 列左侧），实得 [{},{}]",
        ob.x,
        ob.x + ob.width
    );
    let xs = sentence_column_xs(&r.root);
    assert!(
        (xs[0] - 158.0).abs() < 0.5 && (xs[1] - 380.0).abs() < 0.5,
        "sentence 列应 {{sa:158, sb:380}}，实得 {xs:?}"
    );
}

#[test]
fn vlr_mirror_untouched() {
    // vlr 左缘锚定与容器宽无关：本臂不触 vlr 域——列组维持 Σ=440 左起累进位
    //（sb x=0、sa x=20+400=420；sa 溢出 cb 右缘为 vlr 宽重解另账 FIXME）。
    let css_vlr = CSS_007.replace("vertical-rl", "vertical-lr");
    let r = compute(&body_html(&nums100()), &css_vlr);
    let xs = sentence_column_xs(&r.root);
    assert!(
        (xs[0] - 0.0).abs() < 0.5 && (xs[1] - 420.0).abs() < 0.5,
        "vlr 域应零接触（{{0, 420}} Σ 左起累进位），实得 {xs:?}"
    );
}

#[test]
fn auto_cb_untouched() {
    // auto 宽 cb（016 域）：extent=Σ 收缩改写自洽，本臂不触——SHORT 域 ob 重解
    // 202 后组 Σ=242，容器宽同步改写 242，列组 Σ 左起累进位 {0, 222}。
    let css_auto = CSS_007.replace("div#cb { width: 400px; }", "div#cb { width: auto; }");
    let r = compute(&body_html("01 02 03 04 05 06 07 08 09 10"), &css_auto);
    let xs = sentence_column_xs(&r.root);
    assert!(
        (xs[0] - 0.0).abs() < 0.5 && (xs[1] - 222.0).abs() < 0.5,
        "auto cb 域应零接触（Σ=242 restack 位 {{0, 222}}），实得 {xs:?}"
    );
}
