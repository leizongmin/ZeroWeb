//! Path 模块测试 - 覆盖率提升测试
//!
//! 这个测试文件专门针对 path.rs 中覆盖率较低的函数

use crate::path::{Path2D, PathCommand};

/// R3236：path-based `fill()` 须消费 `globalCompositeOperation`——旧 `blit_path_to_pixels` 覆盖写，
/// 致 destination-out/lighter/copy 等经 fill() 的路径填充失效（仅 rect-blit/stroke 经 composite_pixel）。
#[test]
fn fill_path_consumes_composite_operation_r3236() {
    use crate::context::CanvasContext;
    use zero_render_foundation::color::Color;

    // 底层：fill_rect 铺不透明红（blit_rect_to_pixels，已消费 composite——source-over 覆盖透明底）。
    let mut ctx = CanvasContext::new(10, 10);
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
    let base = ctx.get_image_data(5, 5, 1, 1);
    assert_eq!(&base.data[..4], &[255, 0, 0, 255], "底层须为不透明红");

    // destination-out 经**路径 fill** → 须擦除（alpha→0）。旧覆盖写 bug 留 alpha=255。
    ctx.set_composite_operation(crate::CompositeOperation::DestinationOut);
    ctx.set_fill_color(Color::WHITE); // destination-out 仅看 src alpha，颜色无关
    ctx.begin_path();
    ctx.move_to(0.0, 0.0);
    ctx.line_to(10.0, 0.0);
    ctx.line_to(10.0, 10.0);
    ctx.line_to(0.0, 10.0);
    ctx.close_path();
    ctx.fill();
    let erased = ctx.get_image_data(5, 5, 1, 1);
    assert_eq!(erased.data[3], 0, "R3236：destination-out path-fill 须擦除（alpha→0）");

    // 对照：source-over 路径 fill 仍覆盖（不擦除）——防回归到「path-fill 完全不改 dst」。
    let mut ctx2 = CanvasContext::new(10, 10);
    ctx2.set_fill_color(Color::RED);
    ctx2.fill_rect(0.0, 0.0, 10.0, 10.0);
    ctx2.set_fill_color(Color::GREEN);
    ctx2.begin_path();
    ctx2.move_to(0.0, 0.0);
    ctx2.line_to(10.0, 0.0);
    ctx2.line_to(10.0, 10.0);
    ctx2.line_to(0.0, 10.0);
    ctx2.close_path();
    ctx2.fill();
    let over = ctx2.get_image_data(5, 5, 1, 1);
    assert_eq!(&over.data[..4], &[0, 255, 0, 255], "source-over path-fill 须覆盖为绿");
}

/// R3237：`composite_pixel` Lighter（plus）公式——`out_a` 须 clamp 到 [0,1] 再 un-premultiply。
/// 旧实现除以未 clamp 的 `out_a=2` 致红+绿得 `[128,128,0]`（spec 加法应 `[255,255,0]`）。
/// 覆盖 path-fill + rect-fill 两条经 composite_pixel 的路径（pre-existing bug 影响 rect-blit/stroke/path）。
#[test]
fn composite_lighter_additive_r3237() {
    use crate::context::CanvasContext;
    use zero_render_foundation::color::Color;

    // path-fill + lighter：红底 + 绿 path-fill = 黄。
    let mut ctx = CanvasContext::new(10, 10);
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
    ctx.set_composite_operation(crate::CompositeOperation::Lighter);
    ctx.set_fill_color(Color::GREEN);
    ctx.begin_path();
    ctx.move_to(0.0, 0.0);
    ctx.line_to(10.0, 0.0);
    ctx.line_to(10.0, 10.0);
    ctx.line_to(0.0, 10.0);
    ctx.close_path();
    ctx.fill();
    let added = ctx.get_image_data(5, 5, 1, 1);
    assert_eq!(
        &added.data[..4],
        &[255, 255, 0, 255],
        "R3237：lighter path-fill 须加法合成（红+绿=黄），旧 [128,128,0] bug"
    );

    // rect-fill + lighter（pre-existing 路径，亦须修）。
    let mut ctx2 = CanvasContext::new(10, 10);
    ctx2.set_fill_color(Color::RED);
    ctx2.fill_rect(0.0, 0.0, 10.0, 10.0);
    ctx2.set_composite_operation(crate::CompositeOperation::Lighter);
    ctx2.set_fill_color(Color::GREEN);
    ctx2.fill_rect(0.0, 0.0, 10.0, 10.0);
    let added2 = ctx2.get_image_data(5, 5, 1, 1);
    assert_eq!(
        &added2.data[..4],
        &[255, 255, 0, 255],
        "R3237：lighter rect-fill 同须加法（红+绿=黄）"
    );
}

/// R3238：`drawImage` 须消费 `globalCompositeOperation`——旧 `draw_image_sized` 固定 source-over
/// 内联 alpha 混合，无视 composite_operation（destination-out 不擦除）。
#[test]
fn draw_image_consumes_composite_operation_r3238() {
    use crate::context::CanvasContext;
    use zero_render_foundation::color::Color;

    // 底层：fill_rect 铺不透明红。
    let mut ctx = CanvasContext::new(10, 10);
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(0.0, 0.0, 10.0, 10.0);

    // 构造不透明白 ImageData（destination-out 仅看 src alpha，颜色无关）。
    let mut img = ctx.create_image_data(10, 10);
    for px in 0..100 {
        let i = px * 4;
        img.data[i] = 255;
        img.data[i + 1] = 255;
        img.data[i + 2] = 255;
        img.data[i + 3] = 255;
    }

    // destination-out + drawImage → 须擦除（alpha→0）。旧固定 source-over 覆盖为白。
    ctx.set_composite_operation(crate::CompositeOperation::DestinationOut);
    ctx.draw_image(&img, 0.0, 0.0);
    let erased = ctx.get_image_data(5, 5, 1, 1);
    assert_eq!(erased.data[3], 0, "R3238：destination-out drawImage 须擦除（alpha→0）");

    // 对照：source-over drawImage 覆盖（不擦除）——绿 ImageData 覆盖红底。
    let mut ctx2 = CanvasContext::new(10, 10);
    ctx2.set_fill_color(Color::RED);
    ctx2.fill_rect(0.0, 0.0, 10.0, 10.0);
    let mut green = ctx2.create_image_data(10, 10);
    for px in 0..100 {
        let i = px * 4;
        green.data[i + 1] = 255;
        green.data[i + 3] = 255;
    }
    ctx2.draw_image(&green, 0.0, 0.0);
    let over = ctx2.get_image_data(5, 5, 1, 1);
    assert_eq!(&over.data[..4], &[0, 255, 0, 255], "source-over drawImage 须覆盖为绿");
}

/// R3239：blend 模式算术（W3C §9 separable + §10 non-separable）。旧 composite_pixel 对 blend
/// 模式回落 source-over 因子（仅 Porter-Duff 真实合成）；R3239 计算 blended source Cs' 后再合成。
/// 不透源 over 不透背景（da=sa=1）→ out = B(Cb,Cs)（blend 纯结果）。
#[test]
fn blend_modes_r3239() {
    use crate::context::CanvasContext;
    use zero_render_foundation::color::Color;
    let gray = Color {
        r: 128,
        g: 128,
        b: 128,
        a: 255,
    };

    // 灰背景 + 指定 composite 的红 fillRect → 读中心像素。
    let blend = |op: crate::CompositeOperation| -> [u8; 4] {
        let mut ctx = CanvasContext::new(10, 10);
        ctx.set_fill_color(gray);
        ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
        ctx.set_composite_operation(op);
        ctx.set_fill_color(Color::RED);
        ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
        let px = ctx.get_image_data(5, 5, 1, 1);
        [px.data[0], px.data[1], px.data[2], px.data[3]]
    };

    // separable（整数精确）：Cb=gray(128), Cs=red(255,0,0)。
    // multiply: R=128*255/255=128, G=B=0。
    assert_eq!(blend(crate::CompositeOperation::Multiply), [128, 0, 0, 255], "multiply");
    // screen: R=255（cb+cs-cb·cs=1）, G=B=128。
    assert_eq!(blend(crate::CompositeOperation::Screen), [255, 128, 128, 255], "screen");
    // difference: R=|128-255|=127, G=B=|128-0|=128。
    assert_eq!(
        blend(crate::CompositeOperation::Difference),
        [127, 128, 128, 255],
        "difference"
    );

    // non-separable luminosity：取源亮度（Lum(red)=0.3）+ 背景 hue/sat → 纯灰（三通道相等），≠ 红。
    let lum_px = blend(crate::CompositeOperation::Luminosity);
    assert_eq!(lum_px[0], lum_px[1], "luminosity: 三通道相等（纯灰）");
    assert_eq!(lum_px[1], lum_px[2], "luminosity: 三通道相等（纯灰）");
    assert_ne!(&lum_px[..3], &[255, 0, 0][..], "luminosity 须 ≠ source-over 红");

    // 对照 + blend≠source-over：确认 blend 算术生效（非旧 source-over 回落）。
    let source_over = blend(crate::CompositeOperation::SourceOver);
    assert_eq!(&source_over[..], &[255, 0, 0, 255], "source-over: 红覆盖灰");
    assert_ne!(
        blend(crate::CompositeOperation::Overlay),
        source_over,
        "overlay 须 ≠ source-over"
    );
    assert_ne!(
        blend(crate::CompositeOperation::Exclusion),
        source_over,
        "exclusion 须 ≠ source-over"
    );
    assert_ne!(
        blend(crate::CompositeOperation::Hue),
        source_over,
        "hue 须 ≠ source-over"
    );
}

/// R3240：`shadowBlur` 经 region alpha mask + box blur 真实模糊阴影边缘。
/// 旧实现仅画偏移硬边矩形、alpha 衰减（无 blur）——blur>0 与 blur=0 边缘同为硬切。
/// 区分断言：silhouette 外 1px 的像素，blur=0 须无阴影（硬边），blur=4 须有阴影（软扩散）。
#[test]
fn shadow_blur_softens_edge_r3240() {
    use crate::context::CanvasContext;
    use zero_render_foundation::color::Color;

    // 红矩形 (5,5)-(15,15) + 黑阴影（offset 5,5 → silhouette (10,10)-(20,20)），读 alpha。
    let shadow_alpha = |blur: f32, use_path: bool, px: u32, py: u32| -> u8 {
        let mut ctx = CanvasContext::new(30, 30);
        ctx.set_shadow_color(Color::rgba(0, 0, 0, 200));
        ctx.set_shadow_offset_x(5.0);
        ctx.set_shadow_offset_y(5.0);
        ctx.set_shadow_blur(blur);
        ctx.set_fill_color(Color::RED);
        if use_path {
            ctx.begin_path();
            ctx.move_to(5.0, 5.0);
            ctx.line_to(15.0, 5.0);
            ctx.line_to(15.0, 15.0);
            ctx.line_to(5.0, 15.0);
            ctx.close_path();
            ctx.fill();
        } else {
            ctx.fill_rect(5.0, 5.0, 10.0, 10.0);
        }
        ctx.get_image_data(px, py, 1, 1).data[3]
    };

    // (21,15)：silhouette 右边沿（20）外 1px，不被红矩形覆盖（到 15）。
    // blur=0 硬边 → alpha 0；blur=4 软扩散 → alpha > 0。rect + path 两条路径同测。
    assert_eq!(shadow_alpha(0.0, false, 21, 15), 0, "rect blur=0：(21,15) 硬边无阴影");
    assert!(
        shadow_alpha(4.0, false, 21, 15) > 0,
        "R3240 rect blur=4：(21,15) 软扩散有阴影"
    );
    assert_eq!(shadow_alpha(0.0, true, 21, 15), 0, "path blur=0：(21,15) 硬边无阴影");
    assert!(
        shadow_alpha(4.0, true, 21, 15) > 0,
        "R3240 path blur=4：(21,15) 软扩散有阴影"
    );

    // silhouette 内部 (18,18)：两种 blur 均有阴影。
    assert!(shadow_alpha(0.0, false, 18, 18) > 0, "rect silhouette 内部须有阴影");
    assert!(shadow_alpha(4.0, false, 18, 18) > 0, "rect silhouette 内部须有阴影");

    // blur=0 远处 (25,15) 无阴影（防回归）。
    assert_eq!(shadow_alpha(0.0, false, 25, 15), 0, "blur=0 远处无阴影");
}

/// R3241：描边阴影须用 stroke 足迹（thick rect + 连接点），非 centerline——
/// 旧 stroke() 传 centerline 致粗描边阴影过细（仅 centerline offset 的细线）。
#[test]
fn stroke_shadow_uses_footprint_r3241() {
    use crate::context::CanvasContext;
    use zero_render_foundation::color::Color;

    // 水平粗线 line_width=8 在 y=5（footprint y 1-9），阴影 offset (0,10)（band y 11-19），blur=0 硬边。
    let mut ctx = CanvasContext::new(30, 30);
    ctx.set_shadow_color(Color::rgba(0, 0, 0, 200));
    ctx.set_shadow_offset_y(10.0);
    ctx.set_shadow_blur(0.0);
    ctx.set_line_width(8.0);
    ctx.set_stroke_color(Color::RED);
    ctx.begin_path();
    ctx.move_to(5.0, 5.0);
    ctx.line_to(25.0, 5.0);
    ctx.stroke();

    // (15,13)：shadow band（11-19）内，但非 centerline-offset 阴影（y=15）独占 → 旧 bug 无阴影。
    let px = ctx.get_image_data(15, 13, 1, 1);
    assert!(
        px.data[3] > 0,
        "R3241：粗描边阴影须覆盖 band 宽度（非 centerline 细线）——(15,13) 须有阴影"
    );
    // band 中心 (15,15)：须有阴影。
    let center = ctx.get_image_data(15, 15, 1, 1);
    assert!(center.data[3] > 0, "band 中心须有阴影");
    // band 外 (15,20)：blur=0 硬边，无阴影。
    let outside = ctx.get_image_data(15, 20, 1, 1);
    assert_eq!(outside.data[3], 0, "blur=0 band 外无阴影（硬边）");
}

/// R3242：shadowBlur 3 遍 box blur ≈ gaussian——边缘衰减**凸**（near-edge 差 > far-edge 差），
/// 区别于单遍 box 的线性衰减（差值恒定）。验证阴影软度质量提升（R3240 限制①收尾）。
#[test]
fn shadow_blur_multipass_convex_falloff_r3242() {
    use crate::context::CanvasContext;
    use zero_render_foundation::color::Color;

    // 不透明黑阴影（a=255 → 像素 alpha = coverage×255 直接读），blur=8（radius 4，3 遍），offset (0,0)。
    // 红矩形 (10,10)-(20,20)，阴影 silhouette 即矩形，blur 向外扩散。
    let mut ctx = CanvasContext::new(40, 40);
    ctx.set_shadow_color(Color::rgba(0, 0, 0, 255));
    ctx.set_shadow_blur(8.0);
    ctx.set_fill_color(Color::RED);
    ctx.fill_rect(10.0, 10.0, 10.0, 10.0);

    // 读右边沿外 d=1,2,3,4 的阴影 alpha（y=15 矩形内部，x=21..24）。
    let alpha_at = |x: u32| ctx.get_image_data(x, 15, 1, 1).data[3] as i32;
    let a1 = alpha_at(21);
    let a2 = alpha_at(22);
    let a3 = alpha_at(23);
    let a4 = alpha_at(24);
    // 近边有阴影、随距离递减。
    assert!(a1 > a2, "随距离递减：a21({a1}) > a22({a2})");
    assert!(a2 > a4, "随距离递减：a22({a2}) > a24({a4})");
    // 凸衰减（gaussian 特征）：近边差 > 远边差。单遍 box 线性衰减则差值相等 → 本断言区分。
    let near_diff = a1 - a2;
    let far_diff = a3 - a4;
    assert!(
        near_diff > far_diff,
        "R3242：3 遍 gaussian 衰减须凸（near diff {near_diff} > far diff {far_diff}），单遍线性则相等"
    );
}

/// 测试 arc_to 函数的各种角度和半径组合
#[test]
fn test_arc_to_various_angles_and_radii() {
    let mut path = Path2D::new();

    // 测试不同半径
    path.arc_to(50.0, 0.0, 50.0, 50.0, 10.0);
    path.arc_to(50.0, 0.0, 50.0, 50.0, 0.0); // 零半径
    path.arc_to(50.0, 0.0, 50.0, 50.0, 100.0); // 大半径

    // 测试不同角度配置
    path.move_to(0.0, 0.0);
    path.arc_to(100.0, 100.0, 200.0, 0.0, 20.0); // 控制点在不同象限

    // 测试共线点
    path.move_to(0.0, 0.0);
    path.arc_to(50.0, 0.0, 100.0, 0.0, 15.0); // 共线点

    assert!(path.len() > 0);
}

/// 测试 round_rect 函数的不同半径配置
#[test]
fn test_round_rect_different_radii() {
    let mut path = Path2D::new();

    // 单个半径
    path.round_rect(10.0, 10.0, 100.0, 80.0, vec![5.0]);

    // 四个不同半径
    path.round_rect(10.0, 10.0, 100.0, 80.0, vec![5.0, 10.0, 15.0, 20.0]);

    // 两个半径
    path.round_rect(10.0, 10.0, 100.0, 80.0, vec![5.0, 10.0]);

    // 零半径
    path.round_rect(10.0, 10.0, 100.0, 80.0, vec![0.0]);

    // 空半径列表
    path.round_rect(10.0, 10.0, 100.0, 80.0, vec![]);

    // 超大半径（应被限制）
    path.round_rect(0.0, 0.0, 40.0, 20.0, vec![50.0]);

    assert!(path.len() > 0);
}

/// 测试 add_path 函数的路径组合
#[test]
fn test_add_path_composition() {
    let mut path1 = Path2D::new();
    path1.move_to(10.0, 10.0);
    path1.line_to(50.0, 10.0);
    path1.arc(30.0, 30.0, 20.0, 0.0, std::f32::consts::PI);

    let mut path2 = Path2D::new();
    path2.rect(60.0, 60.0, 30.0, 30.0);
    path2.ellipse(100.0, 100.0, 15.0, 10.0, 0.0, 0.0, std::f32::consts::PI);

    // 组合路径
    let mut combined = Path2D::new();
    combined.add_path(&path1);
    combined.add_path(&path2);

    // 验证命令数量相加
    assert_eq!(combined.len(), path1.len() + path2.len());

    // 验证命令顺序
    let mut iter1 = path1.commands().iter();
    let mut iter2 = path2.commands().iter();
    let mut iter_combined = combined.commands().iter();

    // 第一个路径的命令应该在组合路径的前面
    while let Some(cmd1) = iter1.next() {
        assert_eq!(Some(cmd1), iter_combined.next());
    }

    // 第二个路径的命令应该在组合路径的后面
    while let Some(cmd2) = iter2.next() {
        assert_eq!(Some(cmd2), iter_combined.next());
    }

    // 注意：Path2D 不支持添加自己（会导致无限循环），测试添加另一个路径
    let mut path3 = Path2D::new();
    path3.add_path(&path1);
    path3.add_path(&path2);
    assert_eq!(path3.len(), path1.len() + path2.len());

    // 测试创建一个新的路径并添加它
    let mut path4 = Path2D::new();
    path4.add_path(&path3);
    assert_eq!(path4.len(), path3.len());
}

/// 测试 is_point_in_path 函数的各种路径形状
#[test]
fn test_is_point_in_path_various_shapes() {
    let mut ctx = crate::context::CanvasContext::new(200, 200);

    // 三角形路径
    ctx.begin_path();
    ctx.move_to(50.0, 50.0);
    ctx.line_to(150.0, 50.0);
    ctx.line_to(100.0, 150.0);
    ctx.close_path();

    // 内部点
    assert!(ctx.is_point_in_path(100.0, 90.0));
    // 外部点
    assert!(!ctx.is_point_in_path(200.0, 200.0));
    // 边界上的点（行为不确定，但不应 panic）
    let _ = ctx.is_point_in_path(50.0, 50.0);
    let _ = ctx.is_point_in_path(100.0, 50.0);

    // 矩形路径
    ctx.begin_path();
    ctx.move_to(10.0, 10.0);
    ctx.line_to(90.0, 10.0);
    ctx.line_to(90.0, 90.0);
    ctx.line_to(10.0, 90.0);
    ctx.close_path();

    // 中心点在内部
    assert!(ctx.is_point_in_path(50.0, 50.0));
    // 角落外的点
    assert!(!ctx.is_point_in_path(5.0, 5.0));

    // 圆形路径（使用多边形近似）
    ctx.begin_path();
    ctx.arc(100.0, 100.0, 50.0, 0.0, std::f32::consts::PI * 2.0);
    ctx.fill();

    // 圆心在内部
    assert!(ctx.is_point_in_path(100.0, 100.0));
    // 圆外的点
    assert!(!ctx.is_point_in_path(200.0, 100.0));

    // 空路径
    ctx.begin_path();
    assert!(!ctx.is_point_in_path(50.0, 50.0));
}

/// 测试 ellipse 函数的各种参数和退化情况
#[test]
fn test_ellipse_various_parameters() {
    let mut path = Path2D::new();

    // 标准椭圆
    path.ellipse(50.0, 50.0, 30.0, 20.0, 0.0, 0.0, std::f32::consts::PI);

    // 旋转椭圆
    path.ellipse(
        50.0,
        50.0,
        30.0,
        20.0,
        std::f32::consts::FRAC_PI_4,
        0.0,
        std::f32::consts::PI,
    );

    // 圆（rx == ry）
    path.ellipse(100.0, 100.0, 25.0, 25.0, 0.0, 0.0, std::f32::consts::TAU);

    // 线性椭圆（一个半径为0）
    path.ellipse(150.0, 150.0, 0.0, 20.0, 0.0, 0.0, std::f32::consts::PI);
    path.ellipse(150.0, 150.0, 20.0, 0.0, 0.0, 0.0, std::f32::consts::PI);

    // 点椭圆（两个半径都为0）
    path.ellipse(200.0, 200.0, 0.0, 0.0, 0.0, 0.0, std::f32::consts::PI);

    // 负半径（应被处理）
    path.ellipse(0.0, 0.0, -10.0, -10.0, 0.0, 0.0, std::f32::consts::PI);

    // 零角度跨度
    path.ellipse(50.0, 50.0, 10.0, 10.0, 0.0, 0.0, 0.0);

    assert!(path.len() > 0);
}

/// 测试路径命令生成的正确性
#[test]
fn test_path_command_generation() {
    let mut path = Path2D::new();

    // 验证命令序列
    path.move_to(10.0, 20.0);
    path.line_to(30.0, 40.0);
    path.quadratic_curve_to(50.0, 60.0, 70.0, 80.0);
    path.bezier_curve_to(90.0, 100.0, 110.0, 120.0, 130.0, 140.0);
    path.arc(150.0, 160.0, 17.0, 0.0, std::f32::consts::PI);
    path.arc_to(180.0, 170.0, 190.0, 180.0, 19.0);
    path.ellipse(200.0, 210.0, 20.0, 15.0, 0.5, 0.0, std::f32::consts::PI);
    path.round_rect(220.0, 230.0, 40.0, 30.0, vec![5.0, 10.0, 15.0, 20.0]);
    path.close_path();

    let commands = path.commands();
    assert_eq!(commands.len(), 9);

    // 验证每个命令的类型和参数
    match &commands[0] {
        PathCommand::MoveTo(x, y) => {
            assert_eq!(*x, 10.0);
            assert_eq!(*y, 20.0);
        }
        _ => panic!("Expected MoveTo command"),
    }

    match &commands[1] {
        PathCommand::LineTo(x, y) => {
            assert_eq!(*x, 30.0);
            assert_eq!(*y, 40.0);
        }
        _ => panic!("Expected LineTo command"),
    }

    match &commands[2] {
        PathCommand::QuadraticCurveTo(cpx, cpy, x, y) => {
            assert_eq!(*cpx, 50.0);
            assert_eq!(*cpy, 60.0);
            assert_eq!(*x, 70.0);
            assert_eq!(*y, 80.0);
        }
        _ => panic!("Expected QuadraticCurveTo command"),
    }

    match &commands[3] {
        PathCommand::BezierCurveTo(cp1x, cp1y, cp2x, cp2y, x, y) => {
            assert_eq!(*cp1x, 90.0);
            assert_eq!(*cp1y, 100.0);
            assert_eq!(*cp2x, 110.0);
            assert_eq!(*cp2y, 120.0);
            assert_eq!(*x, 130.0);
            assert_eq!(*y, 140.0);
        }
        _ => panic!("Expected BezierCurveTo command"),
    }

    match &commands[4] {
        PathCommand::Arc(x, y, radius, start, end) => {
            assert_eq!(*x, 150.0);
            assert_eq!(*y, 160.0);
            assert_eq!(*radius, 17.0);
            assert_eq!(*start, 0.0);
            assert_eq!(*end, std::f32::consts::PI);
        }
        _ => panic!("Expected Arc command"),
    }

    match &commands[5] {
        PathCommand::ArcTo(x1, y1, _x2, _y2, _radius) => {
            assert_eq!(*x1, 180.0);
            assert_eq!(*y1, 170.0);
        }
        _ => panic!("Expected ArcTo command"),
    }

    match &commands[6] {
        PathCommand::Ellipse(x, y, rx, ry, rotation, start, end) => {
            assert_eq!(*x, 200.0);
            assert_eq!(*y, 210.0);
            assert_eq!(*rx, 20.0);
            assert_eq!(*ry, 15.0);
            assert_eq!(*rotation, 0.5);
            assert_eq!(*start, 0.0);
            assert_eq!(*end, std::f32::consts::PI);
        }
        _ => panic!("Expected Ellipse command"),
    }

    match &commands[7] {
        PathCommand::RoundRect(x, y, w, h, radii) => {
            assert_eq!(*x, 220.0);
            assert_eq!(*y, 230.0);
            assert_eq!(*w, 40.0);
            assert_eq!(*h, 30.0);
            assert_eq!(radii, &vec![5.0, 10.0, 15.0, 20.0]);
        }
        _ => panic!("Expected RoundRect command"),
    }

    match &commands[8] {
        PathCommand::ClosePath => {}
        _ => panic!("Expected ClosePath command"),
    }
}

/// 测试路径的边界条件和特殊情况
#[test]
fn test_path_edge_cases() {
    let mut path = Path2D::new();

    // 测试移动到负坐标
    path.move_to(-10.0, -20.0);
    path.line_to(10.0, 20.0);

    // 测试负坐标的圆弧
    path.arc(-50.0, -50.0, 25.0, 0.0, std::f32::consts::PI);

    // 测试负坐标的椭圆
    path.ellipse(-100.0, -100.0, 30.0, 20.0, 0.0, 0.0, std::f32::consts::PI);

    // 测试负坐标的圆角矩形
    path.round_rect(-200.0, -200.0, 100.0, 80.0, vec![5.0]);

    // 测试大数值
    path.move_to(10000.0, 10000.0);
    path.arc_to(20000.0, 10000.0, 20000.0, 20000.0, 5000.0);

    // 测试 NaN 值（函数不应 panic）
    // Rust 的 f32 类型会处理 NaN，我们只确保不 panic
    path.move_to(f32::NAN, f32::NAN);
    path.arc_to(f32::NAN, f32::NAN, f32::NAN, f32::NAN, f32::NAN);
    path.ellipse(f32::NAN, f32::NAN, f32::NAN, f32::NAN, f32::NAN, f32::NAN, f32::NAN);
    path.round_rect(f32::NAN, f32::NAN, f32::NAN, f32::NAN, vec![f32::NAN]);

    assert!(path.len() > 0);
}

/// 测试路径的扁平化功能
#[test]
fn test_path_flattening() {
    let mut path = Path2D::new();

    // 测试各种路径类型的扁平化
    path.move_to(0.0, 0.0);
    path.line_to(100.0, 0.0);
    path.quadratic_curve_to(50.0, 100.0, 100.0, 200.0);
    path.bezier_curve_to(150.0, 100.0, 200.0, 300.0, 300.0, 200.0);
    path.arc(150.0, 150.0, 50.0, 0.0, std::f32::consts::PI);
    path.ellipse(250.0, 250.0, 50.0, 30.0, 0.0, 0.0, std::f32::consts::PI);
    path.round_rect(0.0, 300.0, 200.0, 100.0, vec![10.0]);
    path.close_path();

    // 获取扁平化的顶点
    let vertices = path.flatten_to_vertices();

    // 扁平化后的顶点数量应该大于零
    assert!(vertices.len() > 0);

    // 验证顶点的格式（应该是 x, y, x, y, ...）
    for chunk in vertices.chunks_exact(2) {
        // 每个 chunk 应该有两个坐标值
        assert_eq!(chunk.len(), 2);
    }
}

/// 测试射线法判断点是否在多边形内部
#[test]
fn test_point_in_polygon_edge_cases() {
    // 使用 point_in_polygon 函数（私有函数的公开接口）
    // 测试凹多边形
    let concave_polygon = [
        (0.0, 0.0),
        (100.0, 0.0),
        (100.0, 50.0),
        (50.0, 50.0), // 凹角
        (50.0, 100.0),
        (0.0, 100.0),
    ];

    // 凹角内侧的点
    assert!(crate::path::point_in_polygon(25.0, 75.0, &concave_polygon));

    // 凹角外侧的点
    assert!(!crate::path::point_in_polygon(75.0, 75.0, &concave_polygon));

    // 测试边界情况
    let square = [(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)];

    // 正方形中心
    assert!(crate::path::point_in_polygon(50.0, 50.0, &square));

    // 正方形外部
    assert!(!crate::path::point_in_polygon(150.0, 50.0, &square));

    // 测试少于3个点的情况
    let two_points = [(0.0, 0.0), (100.0, 100.0)];
    assert!(!crate::path::point_in_polygon(50.0, 50.0, &two_points));

    let empty: [(f32, f32); 0] = [];
    assert!(!crate::path::point_in_polygon(50.0, 50.0, &empty));
}
