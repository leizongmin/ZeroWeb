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
    // clamp 容器 = 声明 line-clamp 的节点对应盒（结构唯一性由用例保证）。
    let clamp_ids: std::collections::HashSet<_> = styles
        .iter()
        .filter(|(_, s)| {
            !matches!(
                s.line_clamp,
                zero_style_system::property::types::LineClampComputedValue::None
            )
        })
        .map(|(id, _)| *id)
        .collect();
    fn find_clamp<'a>(b: &'a LayoutBox, ids: &std::collections::HashSet<zero_dom::NodeId>) -> Option<&'a LayoutBox> {
        if b.node_id.is_some_and(|id| ids.contains(&id)) {
            return Some(b);
        }
        b.children.iter().find_map(|c| find_clamp(c, ids))
    }
    find_clamp(&result.root, &clamp_ids)
        .map(|b| b.height)
        .expect("clamp container box")
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

/// R3781：clamp 点不可落入异 IFC/定高盒内部 → 回退到盒前（css-overflow-4 auto 语义）。
/// driving: line-clamp-auto-033（flow-root 子 = 异 IFC）、line-clamp-auto-035（定高子）。
/// 容器 6lh 预算，[4 行文本片段][3 行 flow-root][Line 8]——偏移落入 flow-root 行内 →
/// 回退到其前：flow-root 整体隐藏、Line 8 隐藏、4 行片段为 ellipsis host → 容器 128。
#[test]
fn r3781_auto_retreat_before_foreign_ifc_child() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"line-clamp: auto; max-height: 192px; font: 16px/32px serif;\">\
<div style=\"white-space: pre;\">Line 1\nLine 2\nLine 3\nLine 4</div>\
<div style=\"display: flow-root; white-space: pre;\">Line 5\nLine 6\nLine 7</div>\
<div style=\"white-space: pre;\">Line 8</div></div>\
</body></html>";
    assert_eq!(
        compute_clamp_container_height(html),
        128.0,
        "max-height 偏移落入 flow-root IFC 内 → clamp 回退到盒前：容器 128（旧 192，.ifc 被错误 mid-cap 到 2 行）"
    );
}

/// R3781 对照：定高子盒同理——cap 行数不收缩盒（height definite 保持），盒内任何
/// clamp 点都令容器超约束 → 回退到盒前。
#[test]
fn r3781_auto_retreat_before_definite_height_child() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"line-clamp: auto; max-height: 192px; font: 16px/32px serif;\">\
<div style=\"white-space: pre;\">Line 1\nLine 2\nLine 3\nLine 4</div>\
<div style=\"height: 96px; white-space: pre;\">Line 5\nLine 6</div>\
<div style=\"white-space: pre;\">Line 7</div></div>\
</body></html>";
    assert_eq!(
        compute_clamp_container_height(html),
        128.0,
        "偏移落入定高盒 → 回退：容器 128（旧 192）"
    );
}

/// R3781 对照（034 不回退）：偏移恰在 flow-root IFC 边界（盒行数 ≤ 余量）→ 正常
/// 完整消耗，不触发回退。
#[test]
fn r3781_flow_root_child_at_boundary_consumes_normally() {
    // 预算 7 行：4（片段）+ 3（flow-root 全部）= 7 → 无溢出无回退，容器 7×32 = 224。
    let html = "<html><body style=\"margin:0\">\
<div style=\"line-clamp: auto; max-height: 224px; font: 16px/32px serif;\">\
<div style=\"white-space: pre;\">Line 1\nLine 2\nLine 3\nLine 4</div>\
<div style=\"display: flow-root; white-space: pre;\">Line 5\nLine 6\nLine 7</div>\
</body></html>";
    assert_eq!(
        compute_clamp_container_height(html),
        224.0,
        "偏移恰在 flow-root 边界：完整消耗，容器 224"
    );
}

/// R3782：multicol 容器中 line-clamp 无效（css-overflow-4 line-clamp-039：collapse 值
/// 在 multicol 中同 auto 初始值，多列分片接管行盒分布）。columns:3 + line-clamp:2 的
/// 9 行内容应 3 列完整渲染、不裁行。
#[test]
fn r3782_multicol_container_line_clamp_disabled() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"line-clamp: 2; columns: 3;\">\
<p>Line 1</p><p>Line 2</p><p>Line 3</p>\
<p>Line 4</p><p>Line 5</p><p>Line 6</p>\
<p>Line 7</p><p>Line 8</p><p>Line 9</p></div>\
</body></html>";
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    // 9 个 p 全可见（无 line_clamp_hidden）：旧实现跨块 clamp 把 p3+ 裁到 0 高。
    fn count_hidden(b: &LayoutBox) -> usize {
        b.children
            .iter()
            .map(|c| usize::from(c.line_clamp_hidden) + count_hidden(c))
            .sum()
    }
    assert_eq!(
        count_hidden(&result.root),
        0,
        "multicol 容器内 line-clamp 无效：9 个 p 全可见（旧 clamp 裁掉 7 个）"
    );
}
