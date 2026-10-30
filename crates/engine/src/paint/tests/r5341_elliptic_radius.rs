//! R5341 回归：椭圆圆角（css-backgrounds-3 §5.1/§5.5）。
//!
//! 旧行为：`BorderRadiusSpec` 单轴——`border_radius_y` 槽位（R4574）存储后无任何
//! paint 消费端，百分比半径整体按 border-box **宽** 解析。`border-radius: 50%`
//! 于非方盒（40×100）渲染为胶囊（ry 恒 = rx = 20），规范要求椭圆（rx=20、ry=50）。
//! corpus 判例 border-shape-outline-with-border-double-ellipse：ref 侧
//! `border-radius: 50%` 自源渲染即胶囊，test/ref 双侧同错。

use super::super::helpers::{BorderRadiusSpec, PrimitiveCounts, rounded_corner_polygon_in_box};
use zero_css_parser::values::LengthValue;
use zero_render_foundation::geometry::Rect;
use zero_style_system::ComputedStyle;

/// 顶点扁平数组中是否存在 ≈(px,py)（容差 0.6px，R5183 同款）。
fn has_vert(verts: &[(f32, f32)], px: f32, py: f32) -> bool {
    verts
        .iter()
        .any(|(x, y)| (*x - px).abs() < 0.6 && (*y - py).abs() < 0.6)
}

/// 进程级 env 互斥：from_style_with_box 读 `ZW_ELLIPTIC_RADIUS`，本文件全部用例
/// 抢同一把锁（kill-switch 用例 set/remove 期间其余用例不得解析半径）。
static R5341_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// 40×100 `border-radius: 50%`（纯单值，槽位缺省）：水平按宽解析 rx=20、垂直按高
/// 重解析 ry=50（§5.1 两轴各自相对对应边）——非方盒为椭圆角，非胶囊。
#[test]
fn test_r5341_percent_ellipse_two_axis_resolution() {
    let _guard = R5341_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let mut style = ComputedStyle::default();
    for r in [
        &mut style.border_top_left_radius,
        &mut style.border_top_right_radius,
        &mut style.border_bottom_right_radius,
        &mut style.border_bottom_left_radius,
    ] {
        *r = LengthValue::Percentage(50.0);
    }
    let spec = BorderRadiusSpec::from_style_with_box(&style, 40.0, 100.0);
    assert_eq!(spec.rx4(), [20.0; 4], "rx = 50% 宽 = 20");
    assert_eq!(spec.ry, [50.0; 4], "ry = 50% 高 = 50（旧单轴误 20）");

    // TL 椭圆弧 (0,50)→(20,0) 心 (20,50)：弧起点 (0,50) 在场、45° 点
    // (20−20cos45°, 50−50sin45°) ≈ (5.86, 14.64) 在场；旧胶囊左缘顶点 (0,20) 不在场。
    let pts = rounded_corner_polygon_in_box(0.0, 0.0, 40.0, 100.0, spec.rx4(), spec.ry);
    assert!(has_vert(&pts, 0.0, 50.0), "TL 椭圆弧起点 (0,50) 应在场，got {pts:?}");
    assert!(
        has_vert(
            &pts,
            20.0 - 20.0 * std::f64::consts::FRAC_1_SQRT_2 as f32,
            50.0 - 50.0 * std::f64::consts::FRAC_1_SQRT_2 as f32
        ),
        "TL 椭圆 45° 点应在场，got {pts:?}"
    );
    assert!(
        !has_vert(&pts, 0.0, 20.0),
        "旧胶囊左缘弧起点 (0,20) 不应在场（ry=50 椭圆无此顶点）"
    );
}

/// 显式双值声明：`border_radius_y` 槽位逐角按高解析（apply.rs 哨兵存声明副本后
/// 无随 rx 分支）——四角各存各值。
#[test]
fn test_r5341_slot_ry_per_corner_resolution() {
    let _guard = R5341_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let mut style = ComputedStyle::default();
    for r in [
        &mut style.border_top_left_radius,
        &mut style.border_top_right_radius,
        &mut style.border_bottom_right_radius,
        &mut style.border_bottom_left_radius,
    ] {
        *r = LengthValue::Px(5.0);
    }
    style.border_radius_y = Some(Box::new([
        LengthValue::Px(10.0),
        LengthValue::Px(20.0),
        LengthValue::Px(30.0),
        LengthValue::Px(40.0),
    ]));
    let spec = BorderRadiusSpec::from_style_with_box(&style, 40.0, 100.0);
    assert_eq!(spec.rx4(), [5.0; 4]);
    assert_eq!(spec.ry, [10.0, 20.0, 30.0, 40.0]);
}

/// kill-switch `ZW_ELLIPTIC_RADIUS=0`：ry=rx（旧单轴渲染），40×100 50% 回胶囊。
/// env var 用例，抢锁防并行互踩（R5084 同款形态）。
#[test]
fn test_r5341_kill_switch_restores_single_axis() {
    let _guard = R5341_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    unsafe { std::env::set_var("ZW_ELLIPTIC_RADIUS", "0") };

    let mut style = ComputedStyle::default();
    for r in [
        &mut style.border_top_left_radius,
        &mut style.border_top_right_radius,
        &mut style.border_bottom_right_radius,
        &mut style.border_bottom_left_radius,
    ] {
        *r = LengthValue::Percentage(50.0);
    }
    let spec = BorderRadiusSpec::from_style_with_box(&style, 40.0, 100.0);
    unsafe { std::env::remove_var("ZW_ELLIPTIC_RADIUS") };
    assert_eq!(spec.ry, spec.rx4(), "kill-switch 下 ry 随 rx（旧单轴）");
    assert_eq!(spec.rx4(), [20.0; 4]);
}

/// pill 习语守卫（welcome 页 v0.1.0 徽章）：200×30 `border-radius: 999px` 单值——
/// §5.5 收缩 f = min(200/1998, 30/1998, …) = 30/1998，rx=ry=15 均匀胶囊；ry=rx 时
/// 各边和与旧 4 项逐项一致，f 逐位同旧（位兼容）。
#[test]
fn test_r5341_pill_shrink_bit_compatible() {
    let _guard = R5341_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let mut style = ComputedStyle::default();
    for r in [
        &mut style.border_top_left_radius,
        &mut style.border_top_right_radius,
        &mut style.border_bottom_right_radius,
        &mut style.border_bottom_left_radius,
    ] {
        *r = LengthValue::Px(999.0);
    }
    let spec = BorderRadiusSpec::from_style_with_box(&style, 200.0, 30.0);
    for (i, (&rx, &ry)) in spec.rx4().iter().zip(spec.ry.iter()).enumerate() {
        assert!(
            (rx - 15.0).abs() < 1e-3,
            "角{i} rx 应收缩至 15（30/1998·999），got {rx}"
        );
        assert!((ry - 15.0).abs() < 1e-3, "角{i} ry 应收缩至 15（同 f），got {ry}");
    }
}

/// 方盒单值不变量：100×100 `border-radius: 50%` → rx=ry=50（§5.5 f=1 不缩），
/// 与旧单轴逐位一致。
#[test]
fn test_r5341_square_percent_unchanged() {
    let _guard = R5341_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let mut style = ComputedStyle::default();
    for r in [
        &mut style.border_top_left_radius,
        &mut style.border_top_right_radius,
        &mut style.border_bottom_right_radius,
        &mut style.border_bottom_left_radius,
    ] {
        *r = LengthValue::Percentage(50.0);
    }
    let spec = BorderRadiusSpec::from_style_with_box(&style, 100.0, 100.0);
    assert_eq!(spec.rx4(), [50.0; 4]);
    assert_eq!(spec.ry, [50.0; 4]);
}

/// R5341 clip-path:inset × path_fill：椭圆角背景降级发射的 path_fill 必须纳入
/// 事后矩形裁剪（Sutherland–Hodgman）——旧覆盖面缺口下整块漏出
///（inner-border-non-renderable 回归实证）。部分相交：顶点收窄到交集多边形。
#[test]
fn test_r5341_path_fill_rect_clip_partial_intersection() {
    use zero_render_foundation::primitive::RenderPrimitives;

    let mut prims = RenderPrimitives::new();
    let from = PrimitiveCounts::snapshot(&prims);
    // 三角形 (0,0)-(200,0)-(200,200)，裁剪窗 (50,50)-(150,150)。
    prims.add_path_fill(
        vec![0.0, 0.0, 200.0, 0.0, 200.0, 200.0],
        zero_render_foundation::color::Color::rgb(0, 0, 255),
    );
    let clip_rect = Rect::new(50.0, 50.0, 100.0, 100.0);
    crate::paint::clip::clip_all_primitives_to_rect(&mut prims, &from, &clip_rect);

    let v = &prims.path_fills[0].vertices;
    assert!(v.len() >= 6, "交集多边形应保留顶点，got {v:?}");
    let mut min = (f32::MAX, f32::MAX);
    let mut max = (f32::MIN, f32::MIN);
    for c in v.as_chunks::<2>().0 {
        let (x, y) = (c[0], c[1]);
        assert!(
            (50.0..=150.0).contains(&x) && (50.0..=150.0).contains(&y),
            "顶点应全在窗内，got ({x},{y})"
        );
        min = (min.0.min(x), min.1.min(y));
        max = (max.0.max(x), max.1.max(y));
    }
    assert_eq!((min, max), ((50.0, 50.0), (150.0, 150.0)), "交集包围盒应铺满裁剪窗");
}

/// 完全在窗内的 path_fill 顶点不变；快照之前的既有 path_fill 不受后续裁剪影响。
#[test]
fn test_r5341_path_fill_rect_clip_inside_and_snapshot_scope() {
    use zero_render_foundation::primitive::RenderPrimitives;

    let mut prims = RenderPrimitives::new();
    prims.add_path_fill(
        vec![0.0, 0.0, 10.0, 0.0, 0.0, 10.0],
        zero_render_foundation::color::Color::rgb(255, 0, 0),
    );
    let from = PrimitiveCounts::snapshot(&prims); // 既有 1 个不在裁剪范围
    let verts_before = prims.path_fills[0].vertices.clone();
    prims.add_path_fill(
        vec![60.0, 60.0, 90.0, 60.0, 60.0, 90.0],
        zero_render_foundation::color::Color::rgb(0, 255, 0),
    );
    let clip_rect = Rect::new(50.0, 50.0, 100.0, 100.0);
    crate::paint::clip::clip_all_primitives_to_rect(&mut prims, &from, &clip_rect);

    assert_eq!(prims.path_fills[0].vertices, verts_before, "快照前图元不受影响");
    assert_eq!(
        prims.path_fills[1].vertices,
        vec![60.0, 60.0, 90.0, 60.0, 60.0, 90.0],
        "窗内顶点原样保留"
    );
}

/// 完全在窗外的 path_fill 顶点清空（渲染端见空顶点跳过）。
#[test]
fn test_r5341_path_fill_rect_clip_outside_emptied() {
    use zero_render_foundation::primitive::RenderPrimitives;

    let mut prims = RenderPrimitives::new();
    let from = PrimitiveCounts::snapshot(&prims);
    prims.add_path_fill(
        vec![200.0, 200.0, 300.0, 200.0, 200.0, 300.0],
        zero_render_foundation::color::Color::rgb(0, 0, 255),
    );
    let clip_rect = Rect::new(0.0, 0.0, 100.0, 100.0);
    crate::paint::clip::clip_all_primitives_to_rect(&mut prims, &from, &clip_rect);

    assert!(prims.path_fills[0].vertices.is_empty(), "窗外 path_fill 应清空");
}
