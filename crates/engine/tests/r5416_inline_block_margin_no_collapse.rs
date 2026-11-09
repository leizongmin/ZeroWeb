//! R5416：行内级原子垂直 margin 不参与块链 margin 折叠——CSS2 §8.3.1 明文
//! 「margins of inline-block boxes do not collapse (not even with their in-flow
//! children)」。旧径 taffy 把 inline-block 族块化为 Block 后，原子 margin 并入
//! 容器首/末塌穿集逐级外溢：
//!
//! 1. **原子自身塌 0 容器**（css/CSS2/normal-flow/inline-block-non-replaced-
//!    height-002：`height:0 + margin:0.5in 0` 的空 inline-block 是 div2 唯一
//!    流内内容）——48px 泄到 div1/div2 margin，div1 被推离静态位 48px，abspos
//!    橙块（top:0 锚 div1）随之错位 ✗3.20%；
//! 2. **常规原子后继兄弟**：`<div><span inline-block h50 m20/></div><div red/>`
//!    ——h50 蓝容 90px 正确，但 mb20 仍外溢推红块到 110（应 90）。
//!
//! 修复两腿：①collect_items definite 零高原子以 margin box 入列（宽有效 +
//! CSS height:0 / taffy used 高 0；auto/未解析高仍降级零宽 Text）——行盒 96px
//! 撑起 div2；②taffy block compute 纯行内级子容器禁用父子塌穿
//!（`item_is_inline_level` 标记 + 全 in-flow item 行内级判定，kill-switch
//! `ZW_INLINE_CHILD_NO_COLLAPSE=0` 回退旧行为）。

use zero_engine::RenderPipeline;

struct Rect {
    y: f32,
    w: f32,
    h: f32,
}

fn fill_of(result: &zero_engine::RenderResult, r: u8, g: u8, b: u8) -> Rect {
    let f = result
        .primitives()
        .fills
        .iter()
        .find(|f| f.color.r == r && f.color.g == g && f.color.b == b && f.color.a == 255)
        .unwrap_or_else(|| panic!("填充 #{r:02x}{g:02x}{b:02x} 缺失"));
    Rect {
        y: f.rect.origin.y,
        w: f.rect.size.width,
        h: f.rect.size.height,
    }
}

/// ★load-bearing（inline-block-non-replaced-height-002 形态，含源空白节点）：
/// 空零高 inline-block（margin 0.5in 0 = 48px）撑起 div2 行盒 96px（§10.8.1
/// margin box），蓝块 y=34（body 8 + p 18 + p mb 8）96×96；div1/div2 margin
/// 回归声明值 0，abspos 橙块（top:0 锚 div1）y=34 96×96——蓝橙同高同位。
/// 修复前：原子降级零宽 Text → div2 塌 0，48px 借塌穿链泄到 div1 → 橙块 y=82。
#[test]
fn r5416_zero_height_inline_block_margin_box_line() {
    let html = "<body>\n  <p>x</p>\n  <div id=\"div1\">\n    <div id=\"div2\">\n      <div></div>\n    </div>\n    <div id=\"div3\"></div>\n  </div>\n</body>";
    let css = "#div1 { line-height: 0; position: relative; } \
               #div2 { background: blue; } \
               #div2 div { display: inline-block; height: 0; margin: 0.5in 0; } \
               div div { width: 1in; } \
               #div3 { background: orange; left: 1in; height: 1in; position: absolute; top: 0; }";
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, css);
    let blue = fill_of(&result, 0, 0, 255);
    let orange = fill_of(&result, 255, 165, 0);
    assert!(
        (blue.h - 96.0).abs() < 0.5 && (blue.w - 96.0).abs() < 0.5,
        "蓝块 = {blue_w}x{blue_h} 应 96×96（行盒 = 原子 margin box 48+0+48）",
        blue_w = blue.w,
        blue_h = blue.h
    );
    assert!(
        (blue.y - orange.y).abs() < 0.5,
        "橙块 y={} 应与蓝块 y={} 同位（原子 margin 不外溢推 div1）",
        orange.y,
        blue.y
    );
    assert!((orange.h - 96.0).abs() < 0.5, "橙块高 {} 应 96", orange.h);
}

/// 守卫（§8.3.1 邻域）：常规 inline-block（h50 + margin 20/0）容器高度含
/// margin box（90），后继红块紧贴容器底 y=90——mb 不再外溢推红块（旧行为 110）。
#[test]
fn r5416_normal_inline_block_margin_kept_in_container() {
    let html = "<body><div style=\"background:blue\"><span style=\"display:inline-block;width:50px;height:50px;margin:20px 0\"></span></div><div style=\"background:red;height:10px\"></div></body>";
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, "");
    let blue = fill_of(&result, 0, 0, 255);
    let red = fill_of(&result, 255, 0, 0);
    assert!(
        (blue.h - 90.0).abs() < 0.5,
        "蓝容器高 {} 应 90（20+50+20 margin box）",
        blue.h
    );
    assert!(
        (red.y - (blue.y + blue.h)).abs() < 0.5,
        "红块 y={} 应紧贴蓝容器底 {}（原子 mb 不外溢；旧径 +20=110 判据越界即红）",
        red.y,
        blue.y + blue.h
    );
}

/// 守卫（R5281 判例保护）：块级子的首/末 margin 仍照常与父折叠（§8.3.1
/// parent-child collapse）——块子 mt20 塌出父顶（蓝块 y=20）；mb20 塌出父底
/// 成父 margin_bottom，与红块 mt0 相邻折叠 = max(20,0)，红块 y = 20+10+20 = 50。
#[test]
fn r5416_block_child_margins_still_collapse() {
    let html = "<body><div style=\"background:blue\"><div style=\"height:10px;margin:20px 0\"></div></div><div style=\"background:red;height:10px\"></div></body>";
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, "");
    let blue = fill_of(&result, 0, 0, 255);
    let red = fill_of(&result, 255, 0, 0);
    assert!(
        (blue.y - 20.0).abs() < 0.5 && (blue.h - 10.0).abs() < 0.5,
        "蓝块 y={} h={} 应 y=20 h=10（块子 mt20 塌出父顶、高度 10）",
        blue.y,
        blue.h
    );
    assert!(
        (red.y - 50.0).abs() < 0.5,
        "红块 y={} 应 50（块子 mb20 塌出父底成父 mb，相邻折叠后仍隔 20）",
        red.y
    );
}
