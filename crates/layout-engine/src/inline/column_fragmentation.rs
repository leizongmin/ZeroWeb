//! Phase 2a multicol 列碎片化上下文 —— IFC 把行盒碎片化到列所需的输入。
//!
//! multicol 是 oracle 一致率最低的目录（master.md R893 实测 **23.0%**），剩余
//! wpt-runner-reachable 杠杆全部进入多会话硬核（multicol Phase 2 / R109 §9.2.1.1 /
//! baseline-export）。本模块为 layout 侧 column-aware IFC 的**接口基石**：定义
//! `ColumnFragmentationContext`（IFC 碎片化的**输入**数据）+ `ColumnFillMode` 枚举，
//! 并以 `InlineFormattingContext.column_fragmentation`（默认 `None`）持有。
//!
//! **Phase 2a step-1（本期）**：仅定义 + IFC dormant 字段，**零生产读取**（grep 证）。
//! 默认 `None` 时 IFC 行为完全不变（行盒不碎片化，当前行为），保证零回归。
//!
//! **Phase 2a step-2（下一会话，多会话接力）**：IFC `break_items_into_lines` 产宽度
//! 换行的全部行盒后，按本上下文把行盒分配到列（respected 列高 budget，整行不裁断，
//! 余量 overflow multicol 盒外），env `MULTICOL_COLUMN_FRAG` 门控渐进启用。
//!
//! **设计参照**：font-metric 桥接（`inline/font_metrics.rs`，R885，commit `d5b7e3ae`）
//! 的 dormant 字段模式。区别：本模块是**纯数据结构**（IFC 输入），非依赖反转，故
//! 无需 trait / 无需 handle newtype（纯数据可 `derive(Debug)`）。
//!
//! **R897 probe 经验证据**（[`evidence/r897-multicol-phase2-probe-2026-06-30.txt`]）：
//! 单层 multicol + `column-fill:auto` + 明确高度 + **单一 block 子元素 breaking 跨列**
//! （R109-independent，区别 mixed-content 被 Phase A 阻塞、区别 nested 是 Phase 3）
//! 的真实缺口 = assignment 阶段静默丢余量 + 非整数高度 overfill（非「文本只在 col0」，
//! 后者是 nested case R201）。即本 slice 的 A1 假设已实证存在（区别 Phase 1 的
//! 0-case 停止）。
//!
//! 详见 [`multicol-phase2-column-fragmentation-context.md`]（rally-pattern 设计文档）。

/// column-fill 模式（IFC 碎片化用，本地枚举避免 IFC 耦合 style-system 类型）。
///
/// 与 IFC 既有 `TextAlign` / `WordBreakMode` 本地枚举风格一致——保持
/// `layout-engine` IFC 自包含。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnFillMode {
    /// `column-fill:balance`（默认）/ `height:auto`：内容在列间均衡，无单列高度约束。
    Balance,
    /// `column-fill:auto` + 明确高度：内容顺序填列，每列受 `available_height` 约束。
    Auto,
}

/// multicol 列碎片化上下文 —— IFC 把行盒碎片化到列所需的输入。
///
/// Phase 2a step-1（本期）：仅定义 + IFC dormant 字段，零生产读取。
/// step-2：IFC 产宽度换行行盒后，消费本上下文把行盒分配到列。
///
/// # 字段语义
///
/// - `col_count`：列数（`column-count` 或由 `column-width` 推得）。
/// - `col_width`：单列内容宽度（px，已扣 `column-gap`）。
/// - `col_gap`：列间距（px）。
/// - `available_height`：每列可用高度预算（px）。`column-fill:auto` + 明确高度 =
///   容器内容高度；`column-fill:balance` / `height:auto` = `None`（无单列高度约束）。
/// - `col_filled_heights`：每列已被 block 子元素占用的高度（px，长度须 = `col_count`）。
///   单一 block 子 slice：全 0（block 子自身即被碎片化的内容）。mixed-content（Phase 2b）：
///   block 子占部分列高，IFC 行盒须避开。
/// - `fill_mode`：column-fill 模式（见 `ColumnFillMode`）。
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnFragmentationContext {
    /// 列数。
    pub col_count: usize,
    /// 单列内容宽度（px）。
    pub col_width: f32,
    /// 列间距（px）。
    pub col_gap: f32,
    /// 每列可用高度预算（px）。`None` = 无高度约束（balance / height:auto）。
    pub available_height: Option<f32>,
    /// 每列已被 block 子占用的高度（px，长度须 = `col_count`）。
    pub col_filled_heights: Vec<f32>,
    /// column-fill 模式。
    pub fill_mode: ColumnFillMode,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inline::InlineFormattingContext;

    /// `ColumnFragmentationContext` 字段可构造且保留全部数据（column-fill:auto + 明确
    /// 高度 + 单一 block 子 slice 的典型输入：`col_filled_heights` 全 0）。
    #[test]
    fn column_fragmentation_context_holds_all_fields() {
        let ctx = ColumnFragmentationContext {
            col_count: 3,
            col_width: 250.0,
            col_gap: 16.0,
            available_height: Some(60.0),
            col_filled_heights: vec![0.0, 0.0, 0.0],
            fill_mode: ColumnFillMode::Auto,
        };
        assert_eq!(ctx.col_count, 3);
        assert!((ctx.col_width - 250.0).abs() < f32::EPSILON);
        assert!((ctx.col_gap - 16.0).abs() < f32::EPSILON);
        assert_eq!(ctx.available_height, Some(60.0));
        assert_eq!(ctx.col_filled_heights, vec![0.0, 0.0, 0.0]);
        assert_eq!(ctx.fill_mode, ColumnFillMode::Auto);
    }

    /// balance / height:auto 输入：`available_height = None`（无单列高度约束）。
    #[test]
    fn column_fragmentation_context_balance_mode_has_no_height_budget() {
        let ctx = ColumnFragmentationContext {
            col_count: 2,
            col_width: 300.0,
            col_gap: 20.0,
            available_height: None,
            col_filled_heights: vec![0.0, 0.0],
            fill_mode: ColumnFillMode::Balance,
        };
        assert_eq!(ctx.available_height, None);
        assert_eq!(ctx.fill_mode, ColumnFillMode::Balance);
    }

    /// Zero-regression 默认：`InlineFormattingContext::new()` 的 `column_fragmentation`
    /// 为 `None`（行盒不碎片化，当前行为不变）——证明 step-1 仅添加 dormant 字段，
    /// 未触及任何 layout / paint 路径。
    #[test]
    fn ifc_column_fragmentation_defaults_none() {
        let ctx = InlineFormattingContext::new(800.0);
        assert!(
            ctx.column_fragmentation.is_none(),
            "IFC must default to no column fragmentation context (current behavior = zero regression)"
        );
    }

    /// `with_column_fragmentation` 注入后字段为 `Some` 且数据正确（builder 工作）。
    #[test]
    fn ifc_with_column_fragmentation_injects_context() {
        let frag = ColumnFragmentationContext {
            col_count: 3,
            col_width: 250.0,
            col_gap: 16.0,
            available_height: Some(48.0),
            col_filled_heights: vec![0.0, 0.0, 0.0],
            fill_mode: ColumnFillMode::Auto,
        };
        let ctx = InlineFormattingContext::new(800.0).with_column_fragmentation(frag.clone());
        let got = ctx
            .column_fragmentation
            .as_ref()
            .expect("column_fragmentation should be set after with_column_fragmentation");
        assert_eq!(got, &frag, "injected context data must be preserved exactly");
    }
}
