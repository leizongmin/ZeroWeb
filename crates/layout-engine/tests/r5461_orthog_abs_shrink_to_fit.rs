//! R5461 主判例：正交 htb abs 叶 auto 物理宽按真 CB（视口 ICB）shrink-to-fit 重解。
//!
//! 塌陷机制（taffy 交换帧 absolute sizing 两趟，上游 float-lft-orthog-htb-in-vrl-002
//! ref 臂 abs 89 vs chromium 211.48）：pass1 以静态父 content 区测得 (taffy w=物理块轴
//! extent, taffy h=物理行内 advance)（R5456 正交测量臂转置返回）；随后 R5293 纯块级
//! 定高回灌臂（taffy-local block.rs height_content_derived，文本叶恒真）以
//! known_width=Some(已测 taffy w) 重排——该值是物理高（行高 40），被测量闭包当水平
//! IFC 约束消费（交换帧下 taffy width 槽 = 物理块轴，槽语义劈叉）→ 文本按 40px 断行，
//! 行内 extent 塌最宽词。**内容无关实证**：单词「floated」与 16 字符全文塌后同宽
//! 123.2（estimate 域）＝宽度不是内容 shrink-to-fit。
//!
//! 修法＝11.4b pass（ZW_ABSPOS_ORTHO_STF default-on）：viewport-CB abs（镜像 11.5
//! 谓词）按 §10.3.7 available = ICB − 定长 inset − 定长 margin，水平 IFC 断行 →
//! width = ceil(单行 advance)（R5104 paint 复排口径）/最宽行。谱系：R5455（同族
//! x 双计回退）、R5456（正交测量臂）、R5293（回灌臂）；同构 kill-switch 回退锚见
//! r5461_orthog_abs_shrink_to_fit_kill_switch.rs（独立进程，R3858 教训）。

use zero_layout_engine::{LayoutEngine, LayoutResult};

/// 上游 002-ref 臂精确形态（vertical-rl 根 + 正交 htb abs right:16px）。
const REF_HTML: &str = r#"<html><body>
<div class="vp">
<div class="abs">HTB floated left</div>
<span>VRL parent</span>
</div>
</body></html>"#;

const REF_CSS: &str = "html { writing-mode: vertical-rl; } \
div.vp { border: orange solid 8px; font-size: 32px; line-height: 1.25; } \
div.abs { background-color: blue; color: white; right: 16px; position: absolute; writing-mode: horizontal-tb; } \
span { position: relative; top: 40px; }";

fn compute(html: &str, css: &str) -> LayoutResult {
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles)
}

fn find_abs(root: &zero_layout_engine::types::LayoutBox) -> &zero_layout_engine::types::LayoutBox {
    fn walk(b: &zero_layout_engine::types::LayoutBox) -> Option<&zero_layout_engine::types::LayoutBox> {
        if b.is_absolute {
            return Some(b);
        }
        for c in &b.children {
            if let Some(f) = walk(c) {
                return Some(f);
            }
        }
        None
    }
    walk(root).expect("abs 盒在树")
}

#[test]
fn orthog_abs_single_line_advance() {
    let r = compute(REF_HTML, REF_CSS);
    let abs = find_abs(&r.root);
    // 旧版本失败证据：「应 263（单行 advance 262.4 ceil）实得 123」（塌最宽词，
    // 与内容长度无关——taffy R5293 回灌臂把物理高 40 反馈成水平约束）。
    assert!(
        (abs.width - 263.0).abs() < 0.5,
        "abs 宽应单行 advance ceil=263，实得 {}",
        abs.width
    );
    // 行高 1.25 × 32 = 40；单行 → 块轴 extent 不变。
    assert!((abs.height - 40.0).abs() < 0.5, "abs 高应行高 40，实得 {}", abs.height);
    assert!((abs.content_height - 40.0).abs() < 0.5, "abs content_height 应 40");
    // 无边框盒：width = content_width（frame 0，ceil 后同步）。
    assert!(
        (abs.content_width - abs.width).abs() < 0.5,
        "无框盒 content_width 应同步 width"
    );
}

#[test]
fn orthog_abs_shrink_to_fit_responds_to_content() {
    // shrink-to-fit 语义：宽度随内容 advance 变化（塌陷域两文本同宽 123.2 恒假）。
    let r = compute(
        r#"<html><body>
<div class="vp">
<div class="abs">floated</div>
<span>VRL parent</span>
</div>
</body></html>"#,
        REF_CSS,
    );
    let abs = find_abs(&r.root);
    // 单词 advance 123.2 ceil = 124（旧域同塌 123.2 无内容响应）。
    assert!((abs.width - 124.0).abs() < 0.5, "单词 abs 宽应 124，实得 {}", abs.width);
    let full = compute(REF_HTML, REF_CSS);
    let abs_full = find_abs(&full.root);
    assert!(
        abs_full.width > abs.width + 50.0,
        "16 字符全文宽 {} 应显著大于单词宽 {}（shrink-to-fit 内容响应）",
        abs_full.width,
        abs.width
    );
}

#[test]
fn nested_cb_and_explicit_width_untouched() {
    // 嵌套 CB 域哨戒：.vp position:relative → CB=.vp，非 viewport-CB，本 pass 不触
    //（塌 123 维持 = taffy 域既有态；chromium available=40 < min-content 同塌最宽词）。
    let css_rel = REF_CSS.replace("div.vp {", "div.vp { position: relative;");
    let r = compute(REF_HTML, &css_rel);
    let abs = find_abs(&r.root);
    assert!(
        (abs.width - 123.0).abs() < 0.5,
        "嵌套 CB 域不应被本 pass 触碰（维持 taffy 既有 123），实得 {}",
        abs.width
    );
    // 显式宽哨戒：width:300 非 auto，不越权。
    let r = compute(
        r#"<html><body>
<div class="vp">
<div class="abs" style="width: 300px;">HTB floated left</div>
<span>VRL parent</span>
</div>
</body></html>"#,
        REF_CSS,
    );
    let abs = find_abs(&r.root);
    assert!((abs.width - 300.0).abs() < 0.5, "显式宽 300 不越权，实得 {}", abs.width);
}

#[test]
fn stretch_domain_gives_way() {
    // stretch 域哨戒：双水平 inset 全定长（left:0 + 既有 right:16）→ 11.5 stretch 臂按
    // ICB 解析 width = 800 − 0 − 16 = 784，本 pass 不得收缩回内容宽。
    let css = REF_CSS.replace("div.abs {", "div.abs { left: 0px;");
    let r = compute(REF_HTML, &css);
    let abs = find_abs(&r.root);
    assert!(
        (abs.width - 784.0).abs() < 0.5,
        "双 inset stretch 域应维持 ICB 解析 784，实得 {}",
        abs.width
    );
}
