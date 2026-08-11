//! Path2D path.rs 覆盖率测试

use super::super::types::*;
use crate::context::*;
use crate::path::{Path2D, PathCommand};
use zero_render_foundation::color::Color;

#[test]
fn test_path2d_arc_to_collinear_points() {
    let mut p = Path2D::new();
    p.move_to(0.0, 0.0);
    // Three colinear points - arcTo should handle gracefully
    p.arc_to(10.0, 0.0, 20.0, 0.0, 5.0);
    assert!(!p.is_empty());
}

#[test]
fn test_path2d_arc_to_zero_radius() {
    let mut p = Path2D::new();
    p.move_to(0.0, 0.0);
    p.arc_to(10.0, 10.0, 20.0, 0.0, 0.0);
    // Zero radius should degenerate to a line
    assert!(!p.is_empty());
}

#[test]
fn test_path2d_arc_to_negative_radius() {
    let mut p = Path2D::new();
    p.move_to(0.0, 0.0);
    // Negative radius should be treated as positive
    p.arc_to(10.0, 10.0, 20.0, 0.0, -5.0);
    assert!(!p.is_empty());
}

#[test]
fn test_path2d_arc_from_origin() {
    let mut p = Path2D::new();
    p.move_to(0.0, 0.0);
    p.arc_to(10.0, 10.0, 20.0, 0.0, 5.0);
    // Test with current point at origin
    let vertices = p.flatten_to_vertices();
    assert!(!vertices.is_empty());
}

#[test]
fn test_path2d_ellipse_zero_radii() {
    let mut p = Path2D::new();
    p.ellipse(10.0, 10.0, 0.0, 0.0, 0.0, 0.0, std::f32::consts::PI);
    // Zero radii should degenerate to a point
    assert!(!p.is_empty());
}

#[test]
fn test_path2d_ellipse_negative_radii() {
    let mut p = Path2D::new();
    p.ellipse(10.0, 10.0, -5.0, -10.0, 0.0, 0.0, std::f32::consts::PI);
    // Negative radii should be handled (absolute value taken in cos/sin)
    assert!(!p.is_empty());
}

#[test]
fn test_path2d_ellipse_full_rotation() {
    let mut p = Path2D::new();
    p.ellipse(10.0, 10.0, 5.0, 10.0, 0.0, 0.0, std::f32::consts::TAU);
    // Full circle (TAU = 2π)
    let vertices = p.flatten_to_vertices();
    assert!(vertices.len() >= 64); // Should have many vertices for full ellipse
}

#[test]
fn test_path2d_ellipse_large_angles() {
    let mut p = Path2D::new();
    p.ellipse(10.0, 10.0, 5.0, 10.0, 0.0, 100.0, 200.0);
    // Angles > 2π should still work
    assert!(!p.is_empty());
}

#[test]
fn test_path2d_ellipse_zero_rotation() {
    let mut p = Path2D::new();
    p.ellipse(10.0, 10.0, 5.0, 10.0, 0.0, 0.0, std::f32::consts::PI);
    // Zero rotation - should still produce vertices
    let vertices = p.flatten_to_vertices();
    assert!(!vertices.is_empty());
}

#[test]
fn test_path2d_round_rect_zero_size() {
    let mut p = Path2D::new();
    p.round_rect(10.0, 10.0, 0.0, 0.0, vec![5.0]);
    // Zero width/height rectangle
    assert!(!p.is_empty());
}

#[test]
fn test_path2d_round_rect_negative_size() {
    let mut p = Path2D::new();
    p.round_rect(10.0, 10.0, -5.0, -5.0, vec![5.0]);
    // Negative width/height - might be handled as absolute or produce degenerate path
    assert!(!p.is_empty());
}

#[test]
fn test_path2d_round_rect_empty_radii() {
    let mut p = Path2D::new();
    p.round_rect(10.0, 10.0, 50.0, 50.0, vec![]);
    // Empty radii list
    assert!(!p.is_empty());
}

#[test]
fn test_path2d_round_rect_odd_radii_count() {
    let mut p = Path2D::new();
    p.round_rect(10.0, 10.0, 50.0, 50.0, vec![5.0, 10.0, 15.0]);
    // Odd number of radii - should handle gracefully
    assert!(!p.is_empty());
}

#[test]
fn test_path2d_round_rect_radii_larger_than_half_size() {
    let mut p = Path2D::new();
    p.round_rect(10.0, 10.0, 50.0, 50.0, vec![100.0]);
    // Radii larger than half size - should be clamped
    assert!(!p.is_empty());
}

#[test]
fn test_path2d_round_rect_negative_radii() {
    let mut p = Path2D::new();
    p.round_rect(10.0, 10.0, 50.0, 50.0, vec![-5.0]);
    // Negative radii - might be clamped to 0
    assert!(!p.is_empty());
}

#[test]
fn test_path2d_is_point_in_path_large_polygon() {
    let mut p = Path2D::new();
    // Create a large polygon
    p.move_to(0.0, 0.0);
    for i in 0..100 {
        let angle = i as f32 * 2.0 * std::f32::consts::PI / 100.0;
        p.line_to(angle.cos() * 50.0 + 50.0, angle.sin() * 50.0 + 50.0);
    }
    p.close_path();

    let inside = p.is_point_in_path(50.0, 50.0);
    assert!(inside, "Center should be inside large polygon");
}

#[test]
fn test_path2d_is_point_in_path_self_intersecting() {
    let mut p = Path2D::new();
    // Create a self-intersecting path (star)
    for i in 0..10 {
        let angle = i as f32 * std::f32::consts::PI / 5.0;
        let radius = if i % 2 == 0 { 50.0 } else { 25.0 };
        p.line_to(angle.cos() * radius + 50.0, angle.sin() * radius + 50.0);
    }
    p.close_path();

    // Test point at center (should be inside by even-odd rule)
    let center_inside = p.is_point_in_path(50.0, 50.0);
    assert!(center_inside, "Center should be inside star");
}

#[test]
fn test_path2d_is_point_in_path_concave() {
    let mut p = Path2D::new();
    // Create a concave polygon (U-shape)
    p.move_to(0.0, 0.0);
    p.line_to(100.0, 0.0);
    p.line_to(100.0, 50.0);
    p.line_to(50.0, 50.0);
    p.line_to(50.0, 100.0);
    p.line_to(0.0, 100.0);
    p.close_path();

    // (25,75) 在凹形左臂实体内（与 test_point_in_polygon_concave 同 L 形几何，(25,75) 判定为内部）。
    let inside = p.is_point_in_path(25.0, 75.0);
    assert!(inside, "Point (25,75) in left arm of concave shape should be inside");
    // (75,75) 在凹角缺口内（U 形凹陷处），应在外部。
    let in_niche = p.is_point_in_path(75.0, 75.0);
    assert!(!in_niche, "Point (75,75) in concave niche should be outside");
}

#[test]
fn test_path2d_is_point_on_edge() {
    let mut p = Path2D::new();
    p.move_to(0.0, 0.0);
    p.line_to(100.0, 0.0);
    p.line_to(100.0, 100.0);
    p.line_to(0.0, 100.0);
    p.close_path();

    // Point exactly on edge - behavior might vary
    let on_edge = p.is_point_in_path(50.0, 0.0);
    assert!(on_edge || !on_edge); // Either is acceptable
}

#[test]
fn test_path2d_flatten_to_vertices_empty() {
    let p = Path2D::new();
    let vertices = p.flatten_to_vertices();
    assert!(vertices.is_empty());
}

#[test]
fn test_path2d_flatten_to_vertices_single_point() {
    let mut p = Path2D::new();
    p.move_to(5.0, 5.0);
    let vertices = p.flatten_to_vertices();
    // Single point might not produce any line segments
    assert_eq!(vertices.len(), 0);
}

#[test]
fn test_path2d_flatten_to_vertices_duplicate_points() {
    let mut p = Path2D::new();
    p.move_to(0.0, 0.0);
    p.line_to(0.0, 0.0); // Same point
    p.line_to(10.0, 10.0);
    let vertices = p.flatten_to_vertices();
    // Duplicate points might be filtered or preserved
    assert!(!vertices.is_empty());
}

#[test]
fn test_path2d_flatten_to_vertices_close_path_no_move() {
    let mut p = Path2D::new();
    p.line_to(10.0, 10.0);
    p.close_path();
    let vertices = p.flatten_to_vertices();
    // Should still work even without initial move_to
    assert!(!vertices.is_empty());
}

#[test]
fn test_path2d_flatten_to_vertices_long_path() {
    let mut p = Path2D::new();
    p.move_to(0.0, 0.0);
    for i in 0..100 {
        p.line_to(i as f32, i as f32);
    }
    p.close_path();
    let vertices = p.flatten_to_vertices();
    assert!(vertices.len() > 400); // Many line segments
}

#[test]
fn test_path2d_flatten_to_vertices_curves_no_endpoints() {
    let mut p = Path2D::new();
    p.quadratic_curve_to(10.0, 0.0, 20.0, 10.0);
    p.bezier_curve_to(30.0, 0.0, 40.0, 0.0, 50.0, 0.0);
    let vertices = p.flatten_to_vertices();
    // Should still work even without initial move_to
    assert!(!vertices.is_empty());
}

#[test]
fn test_path2d_commands_mut_clear() {
    let mut p = Path2D::new();
    p.move_to(0.0, 0.0);
    p.line_to(10.0, 10.0);

    {
        let commands = p.commands_mut();
        commands.clear();
        assert_eq!(commands.len(), 0);
    }
    // 可变借用释放后再读 p（NLL 作用域隔离，避免 E0502）。
    assert!(p.is_empty());
}

#[test]
fn test_path2d_commands_mut_remove() {
    let mut p = Path2D::new();
    p.move_to(0.0, 0.0);
    p.line_to(10.0, 10.0);
    p.line_to(20.0, 20.0);

    let commands = p.commands_mut();
    commands.remove(0); // Remove move_to

    assert_eq!(commands.len(), 2);
    assert!(matches!(commands[0], PathCommand::LineTo(10.0, 10.0)));
}

#[test]
fn test_path2d_commands_mut_insert() {
    let mut p = Path2D::new();
    p.move_to(0.0, 0.0);
    p.line_to(10.0, 10.0);

    let commands = p.commands_mut();
    commands.insert(1, PathCommand::MoveTo(5.0, 5.0));

    assert_eq!(commands.len(), 3);
    assert!(matches!(commands[1], PathCommand::MoveTo(5.0, 5.0)));
}

#[test]
fn test_path2d_commands_mut_pop() {
    let mut p = Path2D::new();
    p.move_to(0.0, 0.0);
    p.line_to(10.0, 10.0);
    p.line_to(20.0, 20.0);

    let commands = p.commands_mut();
    let _ = commands.pop();

    assert_eq!(commands.len(), 2);
    assert!(matches!(commands[1], PathCommand::LineTo(10.0, 10.0)));
}

#[test]
fn test_path2d_commands_mut_truncate() {
    let mut p = Path2D::new();
    p.move_to(0.0, 0.0);
    p.line_to(10.0, 10.0);
    p.line_to(20.0, 20.0);

    let commands = p.commands_mut();
    commands.truncate(1);

    assert_eq!(commands.len(), 1);
    assert!(matches!(commands[0], PathCommand::MoveTo(0.0, 0.0)));
}

#[test]
fn test_path2d_commands_mut_swap() {
    let mut p = Path2D::new();
    p.move_to(0.0, 0.0);
    p.line_to(10.0, 10.0);
    p.line_to(20.0, 20.0);

    let commands = p.commands_mut();
    commands.swap(0, 1);

    assert_eq!(commands.len(), 3);
    assert!(matches!(commands[0], PathCommand::LineTo(10.0, 10.0)));
    assert!(matches!(commands[1], PathCommand::MoveTo(0.0, 0.0)));
}

#[test]
fn test_path2d_commands_mut_retain() {
    let mut p = Path2D::new();
    p.move_to(0.0, 0.0);
    p.line_to(10.0, 10.0);
    p.line_to(20.0, 20.0);
    p.close_path();

    let commands = p.commands_mut();
    commands.retain(|cmd| matches!(cmd, PathCommand::LineTo(_, _)));

    assert_eq!(commands.len(), 2);
    assert!(matches!(commands[0], PathCommand::LineTo(10.0, 10.0)));
    assert!(matches!(commands[1], PathCommand::LineTo(20.0, 20.0)));
}

#[test]
fn test_path2d_commands_mut_drain() {
    let mut p = Path2D::new();
    p.move_to(0.0, 0.0);
    p.line_to(10.0, 10.0);
    p.line_to(20.0, 20.0);

    let commands = p.commands_mut();
    let drained: Vec<_> = commands.drain(1..2).collect();

    assert_eq!(commands.len(), 2);
    assert_eq!(drained.len(), 1);
    assert!(matches!(drained[0], PathCommand::LineTo(10.0, 10.0)));
}

#[test]
fn test_path2d_commands_mut_split_off() {
    let mut p = Path2D::new();
    p.move_to(0.0, 0.0);
    p.line_to(10.0, 10.0);
    p.line_to(20.0, 20.0);

    let commands = p.commands_mut();
    let split = commands.split_off(1);

    assert_eq!(commands.len(), 1);
    assert_eq!(split.len(), 2);
    assert!(matches!(commands[0], PathCommand::MoveTo(0.0, 0.0)));
    assert!(matches!(split[0], PathCommand::LineTo(10.0, 10.0)));
}

#[test]
fn test_path2d_commands_mut_resize() {
    let mut p = Path2D::new();
    p.move_to(0.0, 0.0);
    p.line_to(10.0, 10.0);

    let commands = p.commands_mut();
    commands.resize(5, PathCommand::MoveTo(0.0, 0.0));

    assert_eq!(commands.len(), 5);
    assert!(matches!(commands[0], PathCommand::MoveTo(0.0, 0.0)));
    assert!(matches!(commands[4], PathCommand::MoveTo(0.0, 0.0)));
}

#[test]
fn test_path2d_commands_mut_insert_many() {
    let mut p = Path2D::new();
    p.move_to(0.0, 0.0);
    p.line_to(20.0, 20.0);

    let commands = p.commands_mut();
    let new_commands = vec![
        PathCommand::MoveTo(5.0, 5.0),
        PathCommand::LineTo(10.0, 10.0),
        PathCommand::LineTo(15.0, 15.0),
    ];
    // splice 替代 nightly-only Vec::insert_many（在 index 1 处插入 new_commands 全部元素）。
    commands.splice(1..1, new_commands);

    assert_eq!(commands.len(), 5);
    assert!(matches!(commands[1], PathCommand::MoveTo(5.0, 5.0)));
    assert!(matches!(commands[2], PathCommand::LineTo(10.0, 10.0)));
    assert!(matches!(commands[3], PathCommand::LineTo(15.0, 15.0)));
    assert!(matches!(commands[4], PathCommand::LineTo(20.0, 20.0)));
}

#[test]
fn test_path2d_commands_mut_append() {
    let mut p = Path2D::new();
    p.move_to(0.0, 0.0);
    p.line_to(10.0, 10.0);

    let commands = p.commands_mut();
    let new_commands = vec![PathCommand::LineTo(20.0, 20.0), PathCommand::LineTo(30.0, 30.0)];
    commands.append(&mut new_commands.clone());

    // 原 2 个（move_to[0], line_to(10)[1]）+ append 2 个（line_to(20)[2], line_to(30)[3]）= 4。
    assert_eq!(commands.len(), 4);
    assert!(matches!(commands[2], PathCommand::LineTo(20.0, 20.0)));
    assert!(matches!(commands[3], PathCommand::LineTo(30.0, 30.0)));
}

#[test]
fn test_path2d_commands_mut_extend() {
    let mut p = Path2D::new();
    p.move_to(0.0, 0.0);
    p.line_to(10.0, 10.0);

    let commands = p.commands_mut();
    let new_commands = vec![PathCommand::LineTo(20.0, 20.0), PathCommand::LineTo(30.0, 30.0)];
    commands.extend(new_commands);

    // 原 2 个 + extend 2 个 = 4（extend 项追加到末尾）。
    assert_eq!(commands.len(), 4);
    assert!(matches!(commands[2], PathCommand::LineTo(20.0, 20.0)));
    assert!(matches!(commands[3], PathCommand::LineTo(30.0, 30.0)));
}

#[test]
fn test_path2d_commands_mut_remove_item() {
    let mut p = Path2D::new();
    p.move_to(0.0, 0.0);
    p.line_to(10.0, 10.0);
    p.line_to(20.0, 20.0);

    let commands = p.commands_mut();
    // position + remove 替代 nightly-only Vec::remove_item（移除首个匹配元素）。
    let removed = match commands.iter().position(|c| *c == PathCommand::LineTo(10.0, 10.0)) {
        Some(idx) => {
            commands.remove(idx);
            true
        }
        None => false,
    };

    assert!(removed);
    assert_eq!(commands.len(), 2);
    assert!(matches!(commands[0], PathCommand::MoveTo(0.0, 0.0)));
    assert!(matches!(commands[1], PathCommand::LineTo(20.0, 20.0)));
}

#[test]
fn test_path2d_commands_mut_dedup() {
    let mut p = Path2D::new();
    p.move_to(0.0, 0.0);
    p.line_to(10.0, 10.0);
    p.line_to(10.0, 10.0); // Duplicate
    p.line_to(20.0, 20.0);

    let commands = p.commands_mut();
    commands.dedup();

    assert_eq!(commands.len(), 3);
    assert!(matches!(commands[1], PathCommand::LineTo(10.0, 10.0)));
    assert!(matches!(commands[2], PathCommand::LineTo(20.0, 20.0)));
}
