//! R5462 主判例：vertical 父下的块级正交 htb 子 auto 物理宽按 §7.1 约束
//! （CB indefinite → ICB）shrink-to-fit 重解并重排内容。
//!
//! 塌陷机制（上游 sizing-orthog-htb-in-vrl-016 ✗20.95%，正交块级旧测量路径 FIXME 域，
//! R4437/R5456 收窄先例同债）：taffy 交换帧对块级正交子的测量闭包以父当前收缩宽
//! （循环首趟值）当子行内约束 → 文本按父宽断行，子盒宽 = 断行 extent + frame，
//! 块轴 extent 塌 ICB 高 fill。WPT ref 臂（width:100vw + box-sizing:border-box）
//! 定谳约束落 **border-box**：正交盒 border-box = 视口宽、content = 730。
//!
//! 谱系：R4437（orthogonal inline-block 测量臂）、R5456（正交 float/abs 叶）、
//! R5461（正交 abs 叶 shrink-to-fit）、R5460（viv 容器块轴 extent）——本钉为
//! 块级正交子域收口。R5463 扩展：definite CB（父 Px）constraint = min(父 content
//! 宽, ICB) ＋ stf intrinsic 基校正（intrinsic_sizing 返回 border-box 基，消费前
//! 剥 frame——SHORT 钉 336→266 勘误），该域主判例见
//! r5463_definite_cb_orthog_stf.rs。kill-switch 回退锚见
//! r5462_orthog_block_stf_kill_switch.rs（独立进程，R3858 教训）。

use zero_layout_engine::{LayoutEngine, LayoutResult};

const FULL_NUMS: &str = "01 02 03 04 05 06 07 08 09 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63 64 65 66 67 68 69 70 71 72 73 74 75 76 77 78 79 80 81 82 83 84 85 86 87 88 89 90 91 92 93 94 95 96 97 98 99 00";

const CSS: &str = "html { writing-mode: vertical-rl; } \
body { font-size: 16px; line-height: 1.25; margin-left: 0px; margin-right: 0px; } \
div#cb { width: auto; } \
div#ob { border: blue solid 35px; width: auto; writing-mode: horizontal-tb; }";

fn compute(html: &str, css: &str) -> LayoutResult {
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles)
}

/// 找正交块盒并返回 (绝对 x, 盒引用)——LayoutBox.x 为父相对坐标（paint walk 按
/// `abs = parent_abs + padding + border + b.x` 累加），R5498 根帧转置链右缘锚定后
/// 根盒 x 非 0，块流位断言须按绝对坐标（paint 同式累加）。
fn find_ortho(root: &zero_layout_engine::types::LayoutBox) -> (f32, &zero_layout_engine::types::LayoutBox) {
    fn walk(
        b: &zero_layout_engine::types::LayoutBox,
        parent_abs_x: f32,
    ) -> Option<(f32, &zero_layout_engine::types::LayoutBox)> {
        // 本盒 border-box 绝对 x = 父绝对原点（已含父 frame）+ 本盒父相对 x。
        let abs_x = parent_abs_x + b.x;
        if !b.writing_mode.is_vertical_block_flow() && b.node_id.is_some() && b.is_block_level && b.border_left > 30.0 {
            return Some((abs_x, b));
        }
        let child_origin = abs_x + b.padding_left + b.border_left;
        for c in &b.children {
            if let Some(hit) = walk(c, child_origin) {
                return Some(hit);
            }
        }
        None
    }
    walk(root, 0.0).expect("正交块盒在树")
}

fn body_html(content: &str) -> String {
    format!(r#"<html><body><div id="cb"><div id="ob">{content}</div></div></body></html>"#)
}

#[test]
fn full_text_fills_icb_constraint_border_box() {
    let r = compute(&body_html(FULL_NUMS), CSS);
    let (ob_abs_x, ob) = find_ortho(&r.root);
    // 旧版本失败证据：「应 800（border-box = ICB 约束）实得 130」（塌父收缩宽）。
    assert!(
        (ob.width - 800.0).abs() < 0.5,
        "正交块 border-box 应 800（ICB 约束），实得 {}",
        ob.width
    );
    assert!(
        (ob.content_width - 730.0).abs() < 0.5,
        "content 应 730，实得 {}",
        ob.content_width
    );
    // 块轴 extent = Σ 行高（3×20 = 60）+ frame 70 = 130（旧塌 ICB fill 784）。
    assert!(
        (ob.height - 130.0).abs() < 0.5 && (ob.content_height - 60.0).abs() < 0.5,
        "块轴应 Σ 行高 60+frame=130，实得 h={} ch={}",
        ob.height,
        ob.content_height
    );
    // vertical-rl 块流起始 = 右缘：R5498 根帧转置链收口后祖先链 block extent = Σ 终值
    // 子宽（html/body/#cb = ob 800，旧延迟债「父祖先链宽度不回写……R5460 html root
    // sizing 链同账」随转置销账），ob 右缘贴链右缘 = ICB 宽 800（chromium 语义；
    // 旧引擎祖先链滞留预解 junk，右缘 130 = 旧 #cb 塌宽）。
    assert!(
        (ob_abs_x + ob.width - 800.0).abs() < 0.5,
        "右缘应保持 800（块流位贴 ICB 右缘），实得 abs_x={} w={}",
        ob_abs_x,
        ob.width
    );
}

#[test]
fn short_text_shrinks_to_max_content() {
    let r = compute(&body_html("01 02 03 04 05 06 07 08 09 10"), CSS);
    let (ob_abs_x, ob) = find_ortho(&r.root);
    // shrink-to-fit 内容响应：短文本 max-content 196（10 数单行，content 基）→
    // border-box 266。R5463 修正：intrinsic_sizing 返回 border-box 基（含 frame），
    // stf 公式消费前剥 frame——旧钉锁的双计值 336（=196+70+70）已勘误。
    assert!(
        (ob.width - 266.0).abs() < 0.5,
        "短文本应收缩到 max-content+frame=266，实得 {}",
        ob.width
    );
    assert!(
        (ob.height - 90.0).abs() < 0.5 && (ob.content_height - 20.0).abs() < 0.5,
        "单行块轴 20+frame=90，实得 h={} ch={}",
        ob.height,
        ob.content_height
    );
    // 右缘贴链右缘 = ICB 宽 800（R5498 链收口；旧祖先链 junk 右缘 90）。
    assert!(
        (ob_abs_x + ob.width - 800.0).abs() < 0.5,
        "右缘应保持 800，实得 abs_x={} w={}",
        ob_abs_x,
        ob.width
    );
}

#[test]
fn vlr_block_start_is_left_edge_x_unchanged() {
    // vertical-lr 父：块流起始 = 左缘，宽 grow 右缘自由伸展，x 不动。
    let css_vlr = CSS.replace("vertical-rl", "vertical-lr");
    let r = compute(&body_html(FULL_NUMS), &css_vlr);
    let (ob_abs_x, ob) = find_ortho(&r.root);
    assert!(
        (ob.width - 800.0).abs() < 0.5 && (ob.height - 130.0).abs() < 0.5,
        "vlr 域同规则 800×130，实得 {}×{}",
        ob.width,
        ob.height
    );
    assert!(ob_abs_x.abs() < 0.5, "vlr 块流起始 = 左缘 x 不动，实得 {}", ob_abs_x);
}

#[test]
fn explicit_width_and_definite_cb_untouched() {
    // 显式宽（width:auto 门）：不越权——正交子维持 taffy 既有 470。
    let css_400 = CSS.replace(
        "div#ob { border: blue solid 35px; width: auto;",
        "div#ob { border: blue solid 35px; width: 400px;",
    );
    let r = compute(&body_html("01 02 03 04 05 06 07 08 09 10"), &css_400);
    let (_ob_abs_x, ob) = find_ortho(&r.root);
    assert!(
        (ob.width - 470.0).abs() < 0.5,
        "显式宽 400+frame=470 不越权，实得 {}",
        ob.width
    );
    // definite CB（父 style 宽 Px）：R5463 起 constraint = min(父 content 宽, ICB)
    // ＝400 → available 330 ≥ max-content 196 → stf 收缩 196+70=266（不再断言旧
    // 不触态 90；该域主判例移 r5463_definite_cb_orthog_stf.rs）。
    let css_cb = CSS.replace("div#cb { width: auto; }", "div#cb { width: 400px; }");
    let r = compute(&body_html("01 02 03 04 05 06 07 08 09 10"), &css_cb);
    let (_ob_abs_x, ob) = find_ortho(&r.root);
    assert!(
        (ob.width - 266.0).abs() < 0.5,
        "definite CB 400 域 stf 收缩 196+frame=266，实得 {}",
        ob.width
    );
}

#[test]
fn float_orthog_child_untouched() {
    // float 正交子（R5456 域）：本臂不触（float 域 shrink-to-fit 归 R1015/R5368 旧臂）。
    let css_float = CSS.replace(
        "div#ob { border: blue solid 35px; width: auto;",
        "div#ob { float: left; border: blue solid 35px; width: auto;",
    );
    let r = compute(&body_html("01 02 03"), &css_float);
    let (_ob_abs_x, ob) = find_ortho(&r.root);
    assert!(
        (ob.width - 126.0).abs() < 0.5,
        "float 正交子不触（维持 R5456 域 126），实得 {}",
        ob.width
    );
}
