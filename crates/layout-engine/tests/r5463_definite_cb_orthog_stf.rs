//! R5463 主判例：definite-CB（父 style 宽 Px）下的块级正交 htb 子 §7.1 shrink-to-fit。
//!
//! 塌陷机制（上游 sizing-orthog-htb-in-vrl-007 ✗5.02%，007..012 批次）：R5462 pass
//! 以「父 style 宽 Auto」gate 排除 definite-CB 域 → taffy 交换帧旧测量路径维持塌陷
//!（探针：正交块 w=66/cw=60 按 3 数断行，h=784 塌 ICB fill；父 cw=400 definite 已
//! 被 taffy 解出，仅子臂未重解）。§7.1 orthogonal-auto：constraint = min(available,
//! ICB)，definite CB → available = CB 行内尺寸（物理水平 = 父块方向 extent）。
//!
//! chromium oracle 定谳（headless=new，.xht XML + title 注入零流内污染）：
//! **OBW=400 OBH=126**＝border-box = constraint 400、内容 6 行 × 20 + border 6
//!（ZW 同字体逐位同）。oracle OBX=260 与 ZW 的 sibling 列位差属 vrl 块流基位另账
//!（self-source 双臂 table 模拟同形，非本臂域）。
//!
//! 同轮勘误（R5462 域内修正）：intrinsic_sizing `block_max/min_content_width` 返回
//! **border-box 基**（inner + 自身 frame，R5368 float 外宽消费口径）；本域 stf 公式
//! 以 content 基消费 → 调用点剥 frame_h。旧 SHORT 域 frame 双计（196+70+70=336 应
//! 266，判别矩阵 A/B vs C/D 定谳）；FULL 域 maxc 被 available 钳掩盖。
//! kill-switch 回退锚见 r5463_definite_cb_orthog_stf_kill_switch.rs（独立进程）。

use zero_layout_engine::{LayoutEngine, LayoutResult};

/// 上游 007 精确形态：vrl + body 水平 margin 100px + CB 400px + sentence 前后 +
/// 正交块 border 3px。
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
fn full_text_fills_definite_cb_constraint() {
    let r = compute(&body_html(&nums100()), CSS_007);
    let ob = find_ortho(&r.root);
    // 旧版本失败证据：「应 400（border-box = constraint = CB definite）实得 66」
    //（塌父收缩宽按 3 数断行）。
    assert!(
        (ob.width - 400.0).abs() < 0.5,
        "正交块 border-box 应 400（constraint = min(CB 400, ICB)），实得 {}",
        ob.width
    );
    assert!(
        (ob.content_width - 394.0).abs() < 0.5,
        "content 应 394，实得 {}",
        ob.content_width
    );
    // chromium oracle OBH=126 逐位同（6 行 × 20 + border 6；同字体度量）。
    assert!(
        (ob.height - 126.0).abs() < 0.5 && (ob.content_height - 120.0).abs() < 0.5,
        "块轴应 Σ 行高 120+frame=126，实得 h={} ch={}",
        ob.height,
        ob.content_height
    );
    // body 水平 margin 不进子约束（constraint 是 CB content 宽非视口可用）。
    assert!(
        (ob.width - (ob.content_width + 6.0)).abs() < 0.5,
        "border-box = content + frame 自洽"
    );
}

#[test]
fn constraint_clamped_to_icb_when_cb_wider() {
    // CB 900 > ICB 800：constraint = min(900, 800) = 800。
    let css_900 = CSS_007.replace("div#cb { width: 400px; }", "div#cb { width: 900px; }");
    let r = compute(&body_html(&nums100()), &css_900);
    let ob = find_ortho(&r.root);
    assert!(
        (ob.width - 800.0).abs() < 0.5 && (ob.content_width - 794.0).abs() < 0.5,
        "constraint 应钳 ICB：border-box 800，实得 {}×{}",
        ob.width,
        ob.content_width
    );
    assert!(
        (ob.height - 66.0).abs() < 0.5,
        "794 约束 3 行 Σ60+6=66，实得 {}",
        ob.height
    );
}

#[test]
fn short_text_stf_shrinks_below_cb() {
    // CB 400 下短文本：max-content 196 < available 394 → stf 收缩 196+6=202
    //（不 fill CB——shrink-to-fit 语义）。
    let r = compute(&body_html("01 02 03 04 05 06 07 08 09 10"), CSS_007);
    let ob = find_ortho(&r.root);
    assert!(
        (ob.width - 202.0).abs() < 0.5,
        "STF 收缩应 196+frame=202（< CB 400），实得 {}",
        ob.width
    );
    assert!(
        (ob.height - 26.0).abs() < 0.5 && (ob.content_height - 20.0).abs() < 0.5,
        "单行 20+frame=26，实得 h={} ch={}",
        ob.height,
        ob.content_height
    );
}

#[test]
fn vlr_mirror_same_constraint() {
    // vertical-lr 父镜像：constraint 同 400，块流起始 = 左缘 x 不动。
    let css_vlr = CSS_007.replace("vertical-rl", "vertical-lr");
    let r = compute(&body_html(&nums100()), &css_vlr);
    let ob = find_ortho(&r.root);
    assert!(
        (ob.width - 400.0).abs() < 0.5 && (ob.height - 126.0).abs() < 0.5,
        "vlr 域同规则 400×126，实得 {}×{}",
        ob.width,
        ob.height
    );
    // vlr 左起第一位是 sentence-before p1（列宽 20），ob 块流位 = p1 右缘 20；
    // 宽度重解不位移 vrl 之外的块流位（vlr 不执行 x −= delta）。
    assert!(
        (ob.x - 20.0).abs() < 0.5,
        "vlr 块流位 = sentence 列 20 之后 x 不动，实得 {}",
        ob.x
    );
}
