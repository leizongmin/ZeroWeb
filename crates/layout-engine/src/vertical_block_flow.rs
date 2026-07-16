//! 垂直书写模式（vertical-rl / vertical-lr）的 native block-flow 布局算法。
//!
//! 本模块是 R1541（vertical empirical ground truth）+ R1544（设计文档）建立的
//! **vertical block-flow 模型**，经 2 个受控 chromium ground-truth 变体（V2/V3，
//! `docs/goal/rendering-compat/empirical/vertical-mode/`）验证。
//!
//! **状态：dormant（Phase 2 wiring 待定）**。当前生产路径对 vertical-rl/lr 容器的
//! block-level 子仍用 taffy 默认（horizontal-tb 垂直堆叠，R1542 实证 sizing-entangled，
//! postprocess 路三证 net-negative）。本模块编码 native block-flow 的**位置 + 容器
//! content-size 一致计算**，供 Phase 2 wiring（engine.rs block-layout 主循环的 vertical
//! 分支 + parent propagation）复用。类 R885/R1350 的 de-risking 基础设施。
//!
//! ## 模型（R1541 PIL measured）
//!
//! 对一个 `writing-mode: vertical-rl/lr` 容器的 block-level 子（DOM 序，已知各自 outer
//! width/height）：
//!
//! - **container content-width（inline-size）** = Σ child outer-width + (n−1)×gap。
//! - **container content-height（block-size）** = max child outer-height。
//! - **child 位置**（相对 container content-box top-left）：
//!   - vertical-rl：DOM 序子从右到左。child[i].x = content_width − Σ_{0..=i} width[i]
//!     − i×gap；child[i].y = 0（block-start = top）。
//!   - vertical-lr：DOM 序子从左到右。child[i].x = Σ_{0..i} width[i] + i×gap；y = 0。
//!
//! 关键：layout 期算 container content-size = Σ/max（非 taffy 的 max/Σ），父用此值 sizing，
//! 消除 postprocess 的 sizing 矛盾（R1043/R1047/R1542 net-negative 根因）。

#![allow(dead_code)]

use zero_style_system::WritingModeValue;

/// vertical block-flow 布局结果（container content-box 系）。
#[derive(Clone, Debug, PartialEq)]
pub struct VerticalBlockFlowLayout {
    /// 各子（DOM 序）相对 container content-box top-left 的 (x, y) 偏移。
    pub child_offsets: Vec<(f32, f32)>,
    /// container content-width（inline-size）= Σ child width + gaps。
    pub content_width: f32,
    /// container content-height（block-size）= max child height。
    pub content_height: f32,
}

/// 计算 vertical-rl/lr 容器的 native block-flow（R1541 ground truth 模型）。
///
/// - `children_outer_sizes`：各 block-level 子的 (outer-width, outer-height)，DOM 序。
/// - `writing_mode`：须为 VerticalRl 或 VerticalLr（HorizontalTb 调用方不应进入此函数）。
/// - `gap`：block-flow 方向（水平）的列间距。
pub fn compute_vertical_block_flow(
    children_outer_sizes: &[(f32, f32)],
    writing_mode: WritingModeValue,
    gap: f32,
) -> VerticalBlockFlowLayout {
    let n = children_outer_sizes.len();
    if n == 0 {
        return VerticalBlockFlowLayout {
            child_offsets: Vec::new(),
            content_width: 0.0,
            content_height: 0.0,
        };
    }
    let total_width: f32 = children_outer_sizes.iter().map(|(w, _)| *w).sum();
    let content_width = (total_width + (n as f32 - 1.0) * gap).max(0.0);
    let content_height = children_outer_sizes.iter().map(|(_, h)| *h).fold(0.0_f32, f32::max);

    let mut child_offsets = Vec::with_capacity(n);
    match writing_mode {
        WritingModeValue::VerticalRl => {
            // DOM 序从右到左：child[i].x = content_width − Σ_{0..=i} width − i×gap。
            let mut cum = 0.0_f32;
            for (i, (w, _)) in children_outer_sizes.iter().enumerate() {
                cum += w;
                let x = (content_width - cum - i as f32 * gap).max(0.0);
                child_offsets.push((x, 0.0));
            }
        }
        WritingModeValue::VerticalLr => {
            // DOM 序从左到右：child[i].x = Σ_{0..i} width + i×gap。
            let mut cum = 0.0_f32;
            for (i, (w, _)) in children_outer_sizes.iter().enumerate() {
                let x = cum + i as f32 * gap;
                child_offsets.push((x, 0.0));
                cum += w;
            }
        }
        // HorizontalTb 不应进入此函数；兜底按 vertical-lr（左到右）以避免 panic。
        WritingModeValue::HorizontalTb => {
            let mut cum = 0.0_f32;
            for (i, (w, _)) in children_outer_sizes.iter().enumerate() {
                let x = cum + i as f32 * gap;
                child_offsets.push((x, 0.0));
                cum += w;
            }
        }
    }

    VerticalBlockFlowLayout {
        child_offsets,
        content_width,
        content_height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// R1541 V2：vertical-rl，2 个 50×150 block，gap 0。
    /// chromium 实测：B1 x[52-101]（右），B2 x[2-51]（左），container 100×150（+border）。
    #[test]
    fn test_v2_vertical_rl_two_blocks() {
        let r = compute_vertical_block_flow(&[(50.0, 150.0), (50.0, 150.0)], WritingModeValue::VerticalRl, 0.0);
        assert!(
            (r.content_width - 100.0).abs() < 0.5,
            "content_width {}",
            r.content_width
        );
        assert!(
            (r.content_height - 150.0).abs() < 0.5,
            "content_height {}",
            r.content_height
        );
        // B1 右（x=50），B2 左（x=0）。
        assert!(
            (r.child_offsets[0].0 - 50.0).abs() < 0.5,
            "B1.x {}",
            r.child_offsets[0].0
        );
        assert!((r.child_offsets[1].0).abs() < 0.5, "B2.x {}", r.child_offsets[1].0);
        assert!(
            r.child_offsets[0].1.abs() < 0.5 && r.child_offsets[1].1.abs() < 0.5,
            "y=0"
        );
    }

    /// R1541 V3：vertical-lr，2 个 50×150 block，gap 0。
    /// chromium 实测：B1 x[2-51]（左），B2 x[52-101]（右）。
    #[test]
    fn test_v3_vertical_lr_two_blocks() {
        let r = compute_vertical_block_flow(&[(50.0, 150.0), (50.0, 150.0)], WritingModeValue::VerticalLr, 0.0);
        assert!((r.content_width - 100.0).abs() < 0.5);
        assert!((r.content_height - 150.0).abs() < 0.5);
        // B1 左（x=0），B2 右（x=50）。
        assert!((r.child_offsets[0].0).abs() < 0.5, "B1.x {}", r.child_offsets[0].0);
        assert!(
            (r.child_offsets[1].0 - 50.0).abs() < 0.5,
            "B2.x {}",
            r.child_offsets[1].0
        );
    }

    /// rl 三子：验证 right-to-left 累计（content_width=150，B1 x=100/B2 x=50/B3 x=0）。
    #[test]
    fn test_vertical_rl_three_blocks() {
        let r = compute_vertical_block_flow(
            &[(50.0, 100.0), (50.0, 100.0), (50.0, 100.0)],
            WritingModeValue::VerticalRl,
            0.0,
        );
        assert!((r.content_width - 150.0).abs() < 0.5);
        assert!((r.content_height - 100.0).abs() < 0.5);
        let xs: Vec<f32> = r.child_offsets.iter().map(|(x, _)| *x).collect();
        assert!((xs[0] - 100.0).abs() < 0.5 && (xs[1] - 50.0).abs() < 0.5 && xs[2].abs() < 0.5);
    }

    /// gap 非零：content_width 含 (n−1)×gap，子位置含累计 gap。
    #[test]
    fn test_vertical_lr_with_gap() {
        let r = compute_vertical_block_flow(&[(50.0, 100.0), (50.0, 100.0)], WritingModeValue::VerticalLr, 10.0);
        assert!(
            (r.content_width - 110.0).abs() < 0.5,
            "content_width {}",
            r.content_width
        );
        assert!((r.child_offsets[0].0).abs() < 0.5);
        assert!(
            (r.child_offsets[1].0 - 60.0).abs() < 0.5,
            "B2.x with gap {}",
            r.child_offsets[1].0
        );
    }

    /// content_height = max（非 sum）。
    #[test]
    fn test_content_height_is_max() {
        let r = compute_vertical_block_flow(&[(50.0, 100.0), (50.0, 200.0)], WritingModeValue::VerticalRl, 0.0);
        assert!((r.content_height - 200.0).abs() < 0.5);
    }

    /// 空子：零宽高，无 panic。
    #[test]
    fn test_empty_children() {
        let r = compute_vertical_block_flow(&[], WritingModeValue::VerticalRl, 0.0);
        assert!(r.child_offsets.is_empty());
        assert_eq!(r.content_width, 0.0);
        assert_eq!(r.content_height, 0.0);
    }
}
