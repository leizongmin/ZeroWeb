//! R5417：IFC 原子 fit 门按 margin box 计占位——CSS2 §10.3.5「inline-block 的
//! margin box 参与行内格式化」+ §9.4.2 行内格式化横占位口径。旧径 fit 门只计
//! border box（`current_x + box_width`），原子水平 margin 越出行缘不折行、强留
//! 本行横向溢出；同臂 placement（`current_x += m_left` / `+= box_width + m_right`）
//! 与 flex 断行均按 margin box，双口径分裂（flexbox_first-line：test flex 4/4
//! 折行 vs ref block 5/3 溢出 30px 恒红 2.05%，chromium oracle 同形 4/4 定谳）。
//!
//! kill-switch `ZW_ATOMIC_MARGIN_BOX_FIT=0` 回退 border box 口径。
//! 测试显式 CSS 宽度构造判别形，字体度量无关。

use zero_engine::RenderPipeline;

/// 取黄色（#ffcc00）填充矩形，按 y 排序。
fn yellow_boxes(result: &zero_engine::RenderResult) -> Vec<(f32, f32, f32)> {
    let mut boxes: Vec<(f32, f32, f32)> = result
        .primitives()
        .fills
        .iter()
        .filter(|f| f.color.r == 255 && f.color.g == 204 && f.color.b == 0 && f.color.a == 255)
        .map(|f| (f.rect.origin.y, f.rect.origin.x, f.rect.size.width))
        .collect();
    boxes.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    boxes
}

fn render(width_ul: &str, margin: &str) -> zero_engine::RenderResult {
    let html = "<body><div style=\"width:CONT\"><div style=\"display:inline-block;width:100px;height:20px;margin:MAR;background:#ffcc00\"></div><div style=\"display:inline-block;width:100px;height:20px;margin:MAR;background:#ffcc00\"></div></div></body>"
        .replace("CONT", width_ul)
        .replace("MAR", margin);
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    pipeline.render_html(&html, "")
}

/// 负 margin 针专用：两枚原子各自独立 margin（首枚 0、次枚 ml −90）。
fn render_negative(width_ul: &str) -> zero_engine::RenderResult {
    let html = "<body><div style=\"width:CONT\"><div style=\"display:inline-block;width:100px;height:20px;background:#ffcc00\"></div><div style=\"display:inline-block;width:100px;height:20px;margin-left:-90px;background:#ffcc00\"></div></div></body>"
        .replace("CONT", width_ul);
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    pipeline.render_html(&html, "")
}

/// ★load-bearing（flexbox_first-line ref 页判别形）：行尾原子 margin box 越缘
/// 必折行——容器 342px（内容 8..350），原子 border box 100 + mr 90：li1 占位
/// 8..108（advance 至 198）；li2 margin box 占位 198+90+100+90 = 388 > 350 →
/// 折行 2（旧 border box 口径 198+100 = 298 ≤ 350 判「放得下」强留行 1，
/// mr 溢出 38px）。
#[test]
fn r5417_atomic_margin_box_overflow_wraps() {
    let result = render("342px", "0 90px 0 0");
    let boxes = yellow_boxes(&result);
    assert_eq!(boxes.len(), 2, "两个原子盒都应渲染");
    assert!(
        boxes[1].0 - boxes[0].0 > 10.0,
        "li2 y={} 应折到行 2（li1 y={}；旧行为同排 y 差 0）",
        boxes[1].0,
        boxes[0].0
    );
    assert!(
        (boxes[0].1 - 8.0).abs() < 0.5 && (boxes[1].1 - 8.0).abs() < 0.5,
        "两行首原子 x 应为容器左缘 8（实测 {} / {}）",
        boxes[0].1,
        boxes[1].1
    );
}

/// 守卫（margin 计入恰量、不过折）：容器 400px（内容 8..408）时 li2 margin box
/// 占位 388 ≤ 408 恰放得下——同排不折行，li2 x = 198（ml 0）。占位多计（如 ml
/// 重复计入 fit 与 placement）会误折行 2，本针即红。
#[test]
fn r5417_atomic_margin_box_exact_fit_stays() {
    let result = render("400px", "0 90px 0 0");
    let boxes = yellow_boxes(&result);
    assert_eq!(boxes.len(), 2, "两个原子盒都应渲染");
    assert!(
        (boxes[1].0 - boxes[0].0).abs() < 0.5,
        "li2 y={} 应与 li1 y={} 同排（margin box 恰放下，不过折）",
        boxes[1].0,
        boxes[0].0
    );
    assert!(
        (boxes[1].1 - 198.0).abs() < 0.5,
        "li2 x={} 应 198（li1 border box 8..108 + mr 90 advance）",
        boxes[1].1
    );
}

/// 守卫（零 margin 原子旧行为不变）：无横 margin 的原子 fit 门与旧径恒等
///（margin box = border box）——容器 150px（内容 8..158）内两枚 100px 原子，
/// li2 fit 108+100 = 208 > 158 折行 2（旧径同判，回归保护）。
#[test]
fn r5417_zero_margin_atomic_fit_unchanged() {
    let result = render("150px", "0");
    let boxes = yellow_boxes(&result);
    assert_eq!(boxes.len(), 2, "两个原子盒都应渲染");
    assert!(
        boxes[1].0 - boxes[0].0 > 10.0,
        "li2 y={} 应折行 2（108+100 > 158 越缘；旧行为同判）",
        boxes[1].0
    );
}

/// 守卫（负 margin 符号语义，R5068 同源）：fit 占位与 placement advance 同口径
/// 有符号——li2 ml −90 使 fit 108−90+100 = 118 ≤ 350 放得下，x = current_x +
/// ml = 18（负 margin 左移重叠，CSS 声明语义；旧径 border box 口径 208 ≤ 350
/// 同判同排，仅钉住新口径符号一致性）。
#[test]
fn r5417_negative_margin_signed_fit() {
    let result = render_negative("342px");
    let boxes = yellow_boxes(&result);
    assert_eq!(boxes.len(), 2, "两个原子盒都应渲染");
    assert!(
        (boxes[1].0 - boxes[0].0).abs() < 0.5,
        "li2 y={} 应与 li1 同排（fit 118 ≤ 350）",
        boxes[1].0
    );
    assert!(
        (boxes[1].1 - 18.0).abs() < 0.5,
        "li2 x={} 应 18（placement 108 + ml −90，与 fit 同口径）",
        boxes[1].1
    );
}
