//! R3780：跨块 line-clamp 容器高度收缩——extent 只计 in-flow 子 + 边界上零高自塌盒
//! margin 计入。
//!
//! css-overflow-4 auto clamp：容器收缩到 clamp 边界。两个根因：
//! 1. 收缩 extent fold 含 abspos/fixed 子——oof 脱流不贡献 auto 高（CSS §10.6.3），
//!    clamp 边界处的可见 abspos（静态位 = 边界、本体保留）把 extent 推到自身底边
//!    （auto-032：abspos h=100 → 容器 233 而非 138）。
//! 2. 边界上的零高自塌盒（collapse-through，h=0 + 上下 margin）被 R3775 隐藏后其
//!    bottom margin 从 extent 消失——css-overflow-4 auto-032 assert「bottom margins
//!    end at the clamp boundary」：该 margin 属于边界，容器高须保留（138 = inner 128
//!    + 2×5 margin）。
//!
//! driving: css-overflow/line-clamp/line-clamp-auto-032.tentative.html、
//! line-clamp-auto-025.html（abspos at clamp point）、line-clamp-auto-047/031（不回退）。
use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

/// 返回 line-clamp 容器盒（styles 中 line_clamp ≠ None 的 div）高度。
fn compute_clamp_container_height(html: &str) -> f32 {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn find_clamp(b: &LayoutBox) -> Option<&LayoutBox> {
        // clamp 容器 = 置 clamped/cap 或含 hidden 子的盒（跨块 clamp 消费标记）。
        // 结构唯一性由用例保证（单 clamp 容器）。
        if (b.line_clamp_clamped || b.line_clamp_cap.is_some() || b.children.iter().any(|c| c.line_clamp_hidden))
            && b.height > 1.0
        {
            return Some(b);
        }
        b.children.iter().find_map(find_clamp)
    }
    find_clamp(&result.root).map(|b| b.height).expect("clamp container box")
}

/// 根因 1：clamp 边界上的可见 abspos 不贡献容器 auto 高（in-flow extent）。
/// driving: line-clamp-auto-032（abspos 静态位 = 边界、本体 shown，容器 = 约束 138）。
#[test]
fn r3780_clamp_extent_excludes_abspos() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"line-clamp: auto; max-height: 138px; font: 16px/32px serif;\">\
<div style=\"margin: 5px; white-space: pre;\">Line 1\nLine 2\nLine 3\nLine 4</div>\
<div style=\"margin: 5px;\"></div>\
<div style=\"position: absolute; right: 0; width: 100px; height: 100px; background-color: skyblue;\"></div>\
<div style=\"margin: 5px; white-space: pre;\">Line 5\nLine 6\nLine 7\nLine 8</div></div>\
</body></html>";
    assert_eq!(
        compute_clamp_container_height(html),
        138.0,
        "extent 排除 abspos（h=100 静态位在边界）：容器 = 约束 138（旧 233）"
    );
}

/// 根因 2：边界上的零高自塌盒 bottom margin 计入收缩下限（capped at 约束）。
/// 与上同构但无 abspos——collapse-through（h=0, mb=5）被 R3775 隐藏后其 margin-box
/// 底（133+5=138）= 约束 → 容器 138 而非 extent 133。
#[test]
fn r3780_zero_collapse_margin_extends_boundary_to_constraint() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"line-clamp: auto; max-height: 138px; font: 16px/32px serif;\">\
<div style=\"margin: 5px; white-space: pre;\">Line 1\nLine 2\nLine 3\nLine 4</div>\
<div style=\"margin: 5px;\"></div>\
<div style=\"margin: 5px; white-space: pre;\">Line 5\nLine 6\nLine 7\nLine 8</div></div>\
</body></html>";
    assert_eq!(
        compute_clamp_container_height(html),
        138.0,
        "collapse-through bottom margin 延伸 clamp 边界到约束 138"
    );
}

/// 对照（R3775 不回退）：零高自塌盒真在边界**后**（其 margin-box 超约束）不延伸——
/// 容器收缩到约束（4 行 128），非 143。
#[test]
fn r3780_zero_margin_beyond_constraint_capped() {
    // inner#1 4 行 = 128（= 约束），collapse-through margin-box 133+5=138 > 128。
    // min(138, 128) = 128 → 容器 128（031 语义：边界后的零高盒 margin 不保留）。
    let html = "<html><body style=\"margin:0\">\
<div style=\"line-clamp: auto; max-height: 128px; font: 16px/32px serif;\">\
<div style=\"margin: 0 5px; white-space: pre;\">Line 1\nLine 2\nLine 3\nLine 4</div>\
<div style=\"margin: 5px;\"></div>\
<div style=\"margin: 5px; white-space: pre;\">Line 5\nLine 6</div></div>\
</body></html>";
    assert_eq!(
        compute_clamp_container_height(html),
        128.0,
        "边界后零高盒 margin 经 min(constraint) 截掉：容器 = 128（非 133/138）"
    );
}
