//! R5204（CSS Text Decoration 4 #text-decoration-skip-spaces-property）：①
//! span 级 letter-spacing 元素键在 paint Path B 直通（override map 元素键整条
//! 丢弃的缺陷与 R4133 word-spacing 同构）；② text-decoration-skip-spaces
//! `none | all | [ start || end ]` 两端裁剪（all 追加行尾 letter-spacing）；
//! ③ owner 自声明装饰补绘（CSS 2.1 §16.3.1，容器无装饰而 span{text-decoration}
//! 此前整条丢失）。driving: text-decoration-skip-spaces-001~009。

use crate::pipeline::RenderPipeline;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::RenderPrimitives;

/// 蓝色下划线填充矩形（text-decoration-color:blue）按 x 升序。
fn blue_underline_rects(p: &RenderPrimitives) -> Vec<Rect> {
    let mut rects: Vec<Rect> = p
        .fills
        .iter()
        .filter(|f| f.color.r == 0 && f.color.g == 0 && f.color.b == 255)
        .map(|f| f.rect)
        .collect();
    rects.sort_by(|a, b| a.origin.x.partial_cmp(&b.origin.x).unwrap());
    rects
}

/// 字形 x 序列（文档序）。
fn glyph_xs(p: &RenderPrimitives) -> Vec<f32> {
    p.glyphs.iter().map(|g| g.x).collect()
}

/// Fix A 锚点：span 级 letter-spacing（继承或自声明）在 paint 侧与容器直系文本
/// 同几何——修前 override map 元素键整条丢弃，span 分支片段 ls 落 0，glyph 间距
/// 丢失。span-zero-tail 的 F（ls:0）原点不受自身 ls 影响，与全 20px 的
/// "ABCDEF" 直系文本同位（advance 计入口径：每字符 glyph+ls）。
#[test]
fn r5204_span_letter_spacing_reaches_paint_glyphs() {
    let fs32 = "font-size:32px; white-space:pre";
    let cases: [(&str, String); 4] = [
        (
            "div-direct",
            format!("<div style=\"letter-spacing:20px; {fs32}\">ABCDE</div>"),
        ),
        (
            "span-inherit",
            format!("<div style=\"letter-spacing:20px; {fs32}\"><span>ABCDE</span></div>"),
        ),
        (
            "span-direct-ls",
            format!("<div style=\"{fs32}\"><span style=\"letter-spacing:20px\">ABCDE</span></div>"),
        ),
        (
            "span-zero-tail-vs-full",
            format!("<div style=\"letter-spacing:20px; {fs32}\">ABCDEF</div>"),
        ),
    ];
    let render = |body: &str| {
        let html = format!("<html><body style=\"margin:8px\">{body}</body></html>");
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        pipeline.render_html(&html, "")
    };
    let abcde = [
        glyph_xs(render(&cases[0].1).primitives()),
        glyph_xs(render(&cases[1].1).primitives()),
        glyph_xs(render(&cases[2].1).primitives()),
    ];
    assert_eq!(abcde[0].len(), 5, "ABCDE 应发 5 个字形");
    assert!(
        abcde[0] == abcde[1] && abcde[1] == abcde[2],
        "span 继承/自声明 ls 的字形位应与容器直系文本一致:\n direct={:?}\n inherit={:?}\n span-ls={:?}",
        abcde[0],
        abcde[1],
        abcde[2]
    );
    // F（ls:0）原点 = 前 5 字符（glyph+20）推进，与全 20px 的 F 同位。
    let f_xs = glyph_xs(render(&cases[3].1).primitives());
    assert_eq!(f_xs.len(), 6, "ABCDEF 应发 6 个字形");
    assert_eq!(f_xs[..5], abcde[0][..], "F 前的 ABCDE 字形位不应受 F 段 ls:0 影响");
}

/// Fix B `all`：行尾 letter-spacing 从装饰线裁除（spec：all 于 start/end 语义外
/// 追加裁 trailing spacing）——无 skip-spaces 时下划线右缘多出一个 ls（20px）。
#[test]
fn r5204_skip_spaces_all_trims_trailing_letter_spacing() {
    let render = |skip: &str| {
        let html = format!(
            "<html><body style=\"margin:8px\">\
<div style=\"letter-spacing:20px; font-size:32px; white-space:pre; \
text-decoration:underline; text-decoration-color:blue;{skip}\">ABCDEF</div>\
</body></html>"
        );
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        pipeline.render_html(&html, "")
    };
    let none = blue_underline_rects(render("").primitives());
    let all = blue_underline_rects(render(" text-decoration-skip-spaces:all").primitives());
    assert_eq!(none.len(), 1, "无 skip 应有一条下划线, got {none:?}");
    assert_eq!(all.len(), 1, "all 应有一条下划线, got {all:?}");
    assert!(
        (none[0].origin.x - all[0].origin.x).abs() < 0.5,
        "all 不裁行首，起点应一致: none={} all={}",
        none[0].origin.x,
        all[0].origin.x
    );
    let right = |r: &Rect| r.origin.x + r.size.width;
    assert!(
        ((right(&none[0]) - right(&all[0])) - 20.0).abs() < 0.5,
        "all 应裁除行尾 20px letter-spacing: none 右缘 {} all 右缘 {}",
        right(&none[0]),
        right(&all[0])
    );
}

/// Fix B `start end`：行首/行尾空白自装饰线裁除——裁进量 = 空白 advance
/// （从无 skip 渲染的起头空格字形距实测），不依赖字体度量魔数。
#[test]
fn r5204_skip_spaces_start_end_trims_whitespace() {
    let render = |skip: &str| {
        let html = format!(
            "<html><body style=\"margin:8px\">\
<div style=\"font-size:32px; white-space:pre; text-decoration:underline; \
text-decoration-color:blue;{skip}\">   ABCDEF   </div>\
</body></html>"
        );
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        pipeline.render_html(&html, "")
    };
    let none_result = render("");
    let none_prims = none_result.primitives();
    let none = blue_underline_rects(none_prims);
    // 无 skip 时装饰按词片段分段（空白各自成段），取整线极值。
    assert!(none.len() >= 7, "无 skip 应含 3 前导 + 文本 + 3 尾随段, got {none:?}");
    let right = |r: &Rect| r.origin.x + r.size.width;
    let none_left = none.iter().map(|r| r.origin.x).fold(f32::INFINITY, f32::min);
    let none_right = none.iter().map(right).fold(f32::NEG_INFINITY, f32::max);
    // 起头三个空格的字形 x 间距 = 空格 advance。
    let xs = glyph_xs(none_prims);
    assert!(xs.len() >= 4, "应至少 4 个字形, got {}", xs.len());
    let sw = xs[1] - xs[0];
    assert!(sw > 0.0, "空格 advance 应为正, got {sw}");
    let se = blue_underline_rects(render(" text-decoration-skip-spaces:start end").primitives());
    assert_eq!(se.len(), 1, "start end 应只剩文本段一条下划线, got {se:?}");
    assert!(
        (se[0].origin.x - (none_left + 3.0 * sw)).abs() < 0.5,
        "start 应裁 3 个前导空格: se 起点 {} 期望 {}",
        se[0].origin.x,
        none_left + 3.0 * sw
    );
    assert!(
        (right(&se[0]) - (none_right - 3.0 * sw)).abs() < 0.5,
        "end 应裁 3 个尾随空格: se 右缘 {} 期望 {}",
        right(&se[0]),
        none_right - 3.0 * sw
    );
}

/// Fix B 默认 `none`：显式 none 与缺省同几何——不引入意外裁剪。
#[test]
fn r5204_skip_spaces_none_matches_absent() {
    let render = |skip: &str| {
        let html = format!(
            "<html><body style=\"margin:8px\">\
<div style=\"letter-spacing:20px; font-size:32px; white-space:pre; \
text-decoration:underline; text-decoration-color:blue;{skip}\">ABCDEF</div>\
</body></html>"
        );
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        pipeline.render_html(&html, "")
    };
    let absent = blue_underline_rects(render("").primitives());
    let explicit = blue_underline_rects(render(" text-decoration-skip-spaces:none").primitives());
    assert_eq!(absent.len(), 1, "缺省应有一条下划线");
    assert_eq!(explicit.len(), 1, "显式 none 应有一条下划线");
    assert!(
        (absent[0].origin.x - explicit[0].origin.x).abs() < 0.5
            && (absent[0].size.width - explicit[0].size.width).abs() < 0.5,
        "显式 none 应与缺省同几何: absent={:?} explicit={:?}",
        absent[0],
        explicit[0]
    );
}

/// Fix C 锚点：容器无装饰、span 自声明 underline——装饰线覆盖 span 文本延伸
///（修前宏内装饰调用只读容器 style，span 声明整条丢失，零蓝色填充）。
#[test]
fn r5204_owner_declared_underline_paints() {
    let render = |body: &str| {
        let html = format!("<html><body style=\"margin:8px\">{body}</body></html>");
        let mut pipeline = RenderPipeline::new(800.0, 600.0);
        pipeline.render_html(&html, "")
    };
    let with_result = render(
        "<div style=\"font-size:32px\"><span style=\"text-decoration:underline; text-decoration-color:blue\">ABCDEF</span></div>",
    );
    let with = with_result.primitives();
    let rects = blue_underline_rects(with);
    assert_eq!(rects.len(), 1, "span 自声明下划线应发一条蓝色填充, got {rects:?}");
    // 下划线左缘 = 首字形起点（span 文本延伸内缩为 0）。
    let gxs = glyph_xs(with);
    assert_eq!(gxs.len(), 6);
    assert!(
        (rects[0].origin.x - gxs[0]).abs() < 0.5,
        "下划线应自首字形起: rect.x={} glyph0.x={}",
        rects[0].origin.x,
        gxs[0]
    );
    // 反例：span 无装饰时不得出现蓝色填充。
    let without_result = render("<div style=\"font-size:32px\"><span>ABCDEF</span></div>");
    let without = without_result.primitives();
    assert!(blue_underline_rects(without).is_empty(), "无装饰 span 不应发蓝色填充");
}
