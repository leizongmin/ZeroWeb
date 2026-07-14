//! Phase 2a multicol 列碎片化**算法** —— 把 IFC 行盒按列高预算切片到列。
//!
//! 本模块是 multicol Phase 2a step-2 的**纯算法**切片（设计文档 commit 1）：
//! 消费 [`ColumnFragmentationContext`]（列几何 + 列高预算 + 每列已占高度）+
//! IFC 宽度换行后的 `&[LineBox]`，产出每行的列分配（哪一列 + 列内 y 位置）。
//!
//! **CSS Multicol §2 碎片化契约**：
//! - **整行不裁断**：块级内容的行盒不在中间断裂（区别 inline 跨列 = Phase 2c）。
//!   当前列放不下整行且还有列 → 整行移至下列。
//! - **列高 respected**：每列累计高度受 `available_height` 约束（避免 R897 probe A
//!   的 overfill：每列 64px 超出 60px 列高）。
//! - **余量 overflow**：超出 `col_count × budget` 的行留在末列（CSS：fixed
//!   column-count + 明确高度时，余量 overflow multicol 盒外，由 paint/容器
//!   overflow:visible 处理；本函数仅产出列分配，不渲染）。
//!
//! **本模块是纯函数 + 单测，零生产调用方**（net 0，零回归）。step-2 commit 2
//! （下会话）在 layout 侧为目标结构（单层 multicol + `column-fill:auto` + 明确高度 +
//! 单一 block 子元素）构造 ctx、调本函数、把分配结果存入 LayoutBox 新字段供 paint 消费。
//!
//! **R897 probe A2 已确认可行性**：`LineBox`（`inline_types.rs:155`）携带每行 `y` +
//! `height`，IFC `layout()` 产出 `self.lines: Vec<LineBox>`，本函数可直接消费。
//!
//! 详见 [`multicol-phase2-column-fragmentation-context.md`] §8.4。

use super::{ColumnFragmentationContext, LineBox};

/// 单行行盒的列分配结果（`fragment_lines_into_columns` 输出）。
///
/// 记录某行（`line_idx` 指向输入 `lines`）应落入哪一列（`column`），以及该行在
/// 该列内容区顶部的 y 偏移（`y_in_column`，已含该列此前行盒的累计高度 +
/// `col_filled_heights` 起始偏移）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColumnLineAssignment {
    /// 输入 `lines` 中的行索引。
    pub line_idx: usize,
    /// 目标列索引（`0..col_count`）。
    pub column: usize,
    /// 该行在该列内容区内的 y 偏移（px，列内容顶相对）。
    pub y_in_column: f32,
}

/// 列高预算比较容差（px）。亚像素行高/累加误差不应触发不必要的换列。
const HEIGHT_EPS: f32 = 0.01;

/// 把 IFC 宽度换行后的行盒按列高预算切片到列（Phase 2a step-2 纯算法）。
///
/// # 算法
///
/// 按文档顺序遍历行盒，逐行累加到当前列。**整行不裁断**：若当前列加上该行超过
/// 预算且还有下一列，则推进到下一列（while 循环可连推多列，跳过已被 block 子
/// 占满的列）。末列无法换列时，余量行留在末列（overflow 由上层处理）。
///
/// # 回退（保守，零回归）
///
/// 以下情况返回空 `Vec`，调用方须回退到非碎片化路径（当前 multicol 行为）：
/// - `col_count == 0`；
/// - `col_filled_heights.len() != col_count`（调用方构造错）；
/// - `available_height` 为 `None`（balance / height:auto，非本函数职责——balance
///   经 `assign_children_to_columns_balanced`，不走行盒切片）；
/// - `available_height <= 0`（无效预算）。
///
/// fill_mode 当前仅 `Auto` 有意义（balance 已被 `None` budget 回退拦截）；字段保留
/// 供 step-2 commit 2 的调用方校验（构造 ctx 时应确保 `Auto`）。
pub fn fragment_lines_into_columns(lines: &[LineBox], ctx: &ColumnFragmentationContext) -> Vec<ColumnLineAssignment> {
    // 回退：列数 / 已占高度长度不匹配 → 空分配（调用方回退非碎片化）。
    if ctx.col_count == 0 || ctx.col_filled_heights.len() != ctx.col_count {
        return Vec::new();
    }
    // 回退：仅 column-fill:auto + 明确正高度走本路径（balance 由上层 balanced 分配处理）。
    let budget = match ctx.available_height {
        Some(h) if h > 0.0 => h,
        _ => return Vec::new(),
    };

    let mut col_heights = ctx.col_filled_heights.clone();
    let mut col_idx = 0usize;
    let mut assignments = Vec::with_capacity(lines.len());

    for (i, line) in lines.iter().enumerate() {
        // 整行不裁断：当前列放不下整行且还有下一列 → 推进（可连推多列，跳过已满列）。
        while col_idx + 1 < ctx.col_count && col_heights[col_idx] + line.height > budget + HEIGHT_EPS {
            col_idx += 1;
        }
        let y_in_column = col_heights[col_idx];
        assignments.push(ColumnLineAssignment {
            line_idx: i,
            column: col_idx,
            y_in_column,
        });
        col_heights[col_idx] += line.height;
    }

    assignments
}

/// 把行盒按列高预算顺序填列，**超出 `col_count` 时创建溢出列**（column-fill:auto + 明确高度）。
///
/// 与 [`fragment_lines_into_columns`]（余量留末列）互补：当内容填满全部 `col_count` 列后仍
/// 溢出时，本函数**继续创建新列**（`col_count+1`, `col_count+2`, …），每列同样受 budget
/// 约束。CSS Multicol §8.2：column-fill:auto + 固定 column-count + 内容溢出 → 溢出列在
/// multicol 容器内容边外水平延伸（overflow），column-rule 在每个间隙（含溢出间隙）绘制。
///
/// 返回 `(assignments, total_column_count)`。`total_column_count` ≥ `col_count`；仅当某行
/// 落入 `col_count` 或更后的列时才 > `col_count`。回退条件同 `fragment_lines_into_columns`
///（`col_count == 0` / `col_filled_heights.len() != col_count` / budget `None` 或 ≤ 0）
/// → 返回 `(空 Vec, 0)`。
///
/// **整行不裁断 + 不无限新建**：单行高于 budget（无法放入任何空列）时，留在当前列（与
/// `fragment_lines_into_columns` 单行超高留末列语义一致），不无限创建空溢出列。
pub fn fragment_lines_into_columns_overflow(
    lines: &[LineBox],
    ctx: &ColumnFragmentationContext,
) -> (Vec<ColumnLineAssignment>, usize) {
    // 回退：列数 / 已占高度长度不匹配 → 空分配（调用方回退非碎片化）。
    if ctx.col_count == 0 || ctx.col_filled_heights.len() != ctx.col_count {
        return (Vec::new(), 0);
    }
    // 回退：仅 column-fill:auto + 明确正高度走本路径。
    let budget = match ctx.available_height {
        Some(h) if h > 0.0 => h,
        _ => return (Vec::new(), 0),
    };

    let mut col_heights = ctx.col_filled_heights.clone();
    let mut col_idx = 0usize;
    let mut assignments = Vec::with_capacity(lines.len());

    for (i, line) in lines.iter().enumerate() {
        // 整行不裁断：当前列放不下整行 → 推进下一列。首 col_count 列内跳过已满列；已超出
        // col_count 时，当前（溢出）列满即新建一列。前提：该行能在空列放下（line.height ≤
        // budget）——否则单行超高会无限新建空列，故留在当前列。
        while col_heights[col_idx] + line.height > budget + HEIGHT_EPS && line.height <= budget + HEIGHT_EPS {
            if col_idx + 1 >= col_heights.len() {
                // 当前列是末列（含已建溢出列）且放不下 → 新建一列。
                col_heights.push(0.0);
            }
            col_idx += 1;
        }
        let y_in_column = col_heights[col_idx];
        assignments.push(ColumnLineAssignment {
            line_idx: i,
            column: col_idx,
            y_in_column,
        });
        col_heights[col_idx] += line.height;
    }

    (assignments, col_heights.len())
}

/// 把 IFC 行盒按**文档序均衡**分配到列（column-fill:balance 的 inline 分布）。
///
/// 与 `fragment_lines_into_columns`（auto，按列高 budget 顺序填）互补：balance 无单列
/// 高度约束，每列分 `ceil(剩余行/剩余列)` 行（文档序），使各列行数尽量均等。用于
/// R901 之后的 inline-only balance multicol 扩展（同 use_stored 重定位机制）。
///
/// **当前无生产 caller**：R902 A/B 测 inline-only balance 扩展零 oracle-pass yield（已回退
/// wiring）。函数 + 单测保留作 banked infra，待 balance 路径有具体 yielding 案时重新接线。
///
/// 回退：`col_count == 0` 或 `lines` 空 → 空 `Vec`（调用方回退非碎片化）。
#[allow(dead_code)]
pub fn distribute_lines_balanced(lines: &[LineBox], col_count: usize) -> Vec<ColumnLineAssignment> {
    if col_count == 0 || lines.is_empty() {
        return Vec::new();
    }
    let n = lines.len();
    let mut assignments = Vec::with_capacity(n);
    let mut idx = 0usize;
    let mut remaining_cols = col_count;
    for col in 0..col_count {
        let lines_in_col = (n - idx).div_ceil(remaining_cols);
        let mut y_in_column = 0.0f32;
        for _ in 0..lines_in_col {
            if idx >= n {
                break;
            }
            assignments.push(ColumnLineAssignment {
                line_idx: idx,
                column: col,
                y_in_column,
            });
            y_in_column += lines[idx].height;
            idx += 1;
        }
        remaining_cols -= 1;
    }
    assignments
}

#[cfg(test)]
mod tests {
    use super::super::ColumnFillMode;
    use super::*;

    /// 构造最小行盒（仅高度参与切片逻辑，其余置零/空）。
    fn line(height: f32) -> LineBox {
        LineBox {
            y: 0.0,
            height,
            runs: Vec::new(),
            baseline_y: 0.0,
            ascent: 0.0,
            descent: 0.0,
        }
    }

    /// 构造 column-fill:auto + 明确高度 + 单一 block 子 slice 的典型 ctx
    ///（`col_filled_heights` 全 0）。
    fn ctx_auto(col_count: usize, budget: f32) -> ColumnFragmentationContext {
        ColumnFragmentationContext {
            col_count,
            col_width: 250.0,
            col_gap: 16.0,
            available_height: Some(budget),
            col_filled_heights: vec![0.0; col_count],
            fill_mode: ColumnFillMode::Auto,
        }
    }

    /// 整行不裁断 + 列满续列：3 行 h=20，budget=50，2 列 → col0 容纳 2 行（40），
    /// 第 3 行（40+20=60 > 50）整行移至 col1。
    #[test]
    fn whole_line_never_splits_advances_column_when_full() {
        let lines = vec![line(20.0), line(20.0), line(20.0)];
        let ctx = ctx_auto(2, 50.0);
        let a = fragment_lines_into_columns(&lines, &ctx);
        assert_eq!(a.len(), 3);
        assert_eq!(
            a[0],
            ColumnLineAssignment {
                line_idx: 0,
                column: 0,
                y_in_column: 0.0
            }
        );
        assert_eq!(
            a[1],
            ColumnLineAssignment {
                line_idx: 1,
                column: 0,
                y_in_column: 20.0
            }
        );
        // 第 3 行 col0 累计 40+20=60 > 50 → 推进 col1。
        assert_eq!(
            a[2],
            ColumnLineAssignment {
                line_idx: 2,
                column: 1,
                y_in_column: 0.0
            }
        );
    }

    /// 余量 overflow 留末列：5 行 h=20，budget=30，2 列 → col0 1 行；col1 收 4 行
    ///（末列无法换列，余量留末列，由上层 overflow 处理）。
    #[test]
    fn overflow_beyond_column_count_stays_in_last_column() {
        let lines = vec![line(20.0); 5];
        let ctx = ctx_auto(2, 30.0);
        let a = fragment_lines_into_columns(&lines, &ctx);
        // 列分布：col0=line0，col1=line1..4。
        assert_eq!(a[0].column, 0);
        for assn in &a[1..] {
            assert_eq!(assn.column, 1, "overflow lines must stay in last column");
        }
        // 末列 y 累计：line1 y=0，line2 y=20，line3 y=40，line4 y=60。
        assert!((a[4].y_in_column - 60.0).abs() < 0.01);
    }

    /// col_filled_heights 预占（mixed-content 预演）：col0 已占 30，budget=50，
    /// 2 行 h=20 → line0（30+20=50 ≤ 50）留 col0（y=30），line1（50+20=70 > 50）→ col1。
    #[test]
    fn prefilled_column_advances_when_new_line_does_not_fit() {
        let ctx = ColumnFragmentationContext {
            col_count: 2,
            col_width: 250.0,
            col_gap: 16.0,
            available_height: Some(50.0),
            col_filled_heights: vec![30.0, 0.0],
            fill_mode: ColumnFillMode::Auto,
        };
        let lines = vec![line(20.0), line(20.0)];
        let a = fragment_lines_into_columns(&lines, &ctx);
        assert_eq!(
            a[0],
            ColumnLineAssignment {
                line_idx: 0,
                column: 0,
                y_in_column: 30.0
            }
        );
        assert_eq!(
            a[1],
            ColumnLineAssignment {
                line_idx: 1,
                column: 1,
                y_in_column: 0.0
            }
        );
    }

    /// 单行超高（> budget）：无法放入任何整列，连推至末列留之（CSS：不可拆行 → overflow）。
    #[test]
    fn single_line_taller_than_budget_goes_to_last_column() {
        let lines = vec![line(100.0)];
        let ctx = ctx_auto(3, 30.0);
        let a = fragment_lines_into_columns(&lines, &ctx);
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].column, 2, "oversized line advances to last column");
        assert_eq!(a[0].y_in_column, 0.0);
    }

    /// balance / height:auto（无高度预算）→ 空分配（回退，balance 走上层 balanced 分配）。
    #[test]
    fn balance_mode_no_height_budget_returns_empty() {
        let lines = vec![line(20.0), line(20.0)];
        let ctx = ColumnFragmentationContext {
            col_count: 3,
            col_width: 250.0,
            col_gap: 16.0,
            available_height: None,
            col_filled_heights: vec![0.0, 0.0, 0.0],
            fill_mode: ColumnFillMode::Balance,
        };
        assert!(fragment_lines_into_columns(&lines, &ctx).is_empty());
    }

    /// col_count == 0 → 空分配（回退）。
    #[test]
    fn zero_col_count_returns_empty() {
        let ctx = ColumnFragmentationContext {
            col_count: 0,
            col_width: 0.0,
            col_gap: 0.0,
            available_height: Some(50.0),
            col_filled_heights: Vec::new(),
            fill_mode: ColumnFillMode::Auto,
        };
        assert!(fragment_lines_into_columns(&[line(20.0)], &ctx).is_empty());
    }

    /// col_filled_heights.len() != col_count（调用方构造错）→ 空分配（回退）。
    #[test]
    fn mismatched_filled_heights_returns_empty() {
        let ctx = ColumnFragmentationContext {
            col_count: 2,
            col_width: 250.0,
            col_gap: 16.0,
            available_height: Some(50.0),
            col_filled_heights: vec![0.0], // len 1 != col_count 2
            fill_mode: ColumnFillMode::Auto,
        };
        assert!(fragment_lines_into_columns(&[line(20.0)], &ctx).is_empty());
    }

    /// budget <= 0（无效）→ 空分配（回退）。
    #[test]
    fn non_positive_budget_returns_empty() {
        let ctx = ColumnFragmentationContext {
            col_count: 2,
            col_width: 250.0,
            col_gap: 16.0,
            available_height: Some(0.0),
            col_filled_heights: vec![0.0, 0.0],
            fill_mode: ColumnFillMode::Auto,
        };
        assert!(fragment_lines_into_columns(&[line(20.0)], &ctx).is_empty());
    }

    /// 空行盒列表 → 空分配。
    #[test]
    fn empty_lines_returns_empty() {
        let ctx = ctx_auto(3, 50.0);
        assert!(fragment_lines_into_columns(&[], &ctx).is_empty());
    }

    /// balance 文档序均衡：5 行，3 列 → col0=2 行（ceil(5/3)）、col1=2 行（ceil(3/2)）、col2=1 行。
    #[test]
    fn balance_distributes_document_order_ceil_split() {
        let lines = vec![line(20.0); 5];
        let a = distribute_lines_balanced(&lines, 3);
        assert_eq!(a.len(), 5);
        // col0: line0 (y=0), line1 (y=20)
        assert_eq!(
            a[0],
            ColumnLineAssignment {
                line_idx: 0,
                column: 0,
                y_in_column: 0.0
            }
        );
        assert_eq!(
            a[1],
            ColumnLineAssignment {
                line_idx: 1,
                column: 0,
                y_in_column: 20.0
            }
        );
        // col1: line2 (y=0), line3 (y=20)
        assert_eq!(
            a[2],
            ColumnLineAssignment {
                line_idx: 2,
                column: 1,
                y_in_column: 0.0
            }
        );
        assert_eq!(
            a[3],
            ColumnLineAssignment {
                line_idx: 3,
                column: 1,
                y_in_column: 20.0
            }
        );
        // col2: line4 (y=0)
        assert_eq!(
            a[4],
            ColumnLineAssignment {
                line_idx: 4,
                column: 2,
                y_in_column: 0.0
            }
        );
    }

    /// balance 行数 ≤ 列数 → 每列 ≤1 行。
    #[test]
    fn balance_fewer_lines_than_columns() {
        let lines = vec![line(20.0); 2];
        let a = distribute_lines_balanced(&lines, 3);
        assert_eq!(a.len(), 2);
        assert_eq!(a[0].column, 0);
        assert_eq!(a[1].column, 1);
    }

    /// balance col_count=0 或空 lines → 空分配（回退）。
    #[test]
    fn balance_zero_cols_or_empty_returns_empty() {
        assert!(distribute_lines_balanced(&[line(20.0)], 0).is_empty());
        assert!(distribute_lines_balanced(&[], 3).is_empty());
    }

    // ── R1429 fragment_lines_into_columns_overflow（溢出列创建）──

    /// 内容填满 2 列后溢出 → 创建第 3 列（溢出列）。5 行 h=20，budget=40，col_count=2：
    /// col0 收 2 行（40），col1 收 2 行（40），第 5 行溢出 → 新建 col2 收之。total=3。
    #[test]
    fn overflow_creates_extra_column_when_content_exceeds_column_count() {
        let lines = vec![line(20.0); 5];
        let ctx = ctx_auto(2, 40.0);
        let (a, total) = fragment_lines_into_columns_overflow(&lines, &ctx);
        assert_eq!(total, 3, "5 行 ×20 over 2 列 budget 40 → 3 列（含 1 溢出列）");
        assert_eq!(a.len(), 5);
        // col0: line0(y0) line1(y20)；col1: line2(y0) line3(y20)；col2(溢出): line4(y0)
        assert_eq!(a[0].column, 0);
        assert_eq!(a[0].y_in_column, 0.0);
        assert_eq!(a[1].column, 0);
        assert_eq!(a[1].y_in_column, 20.0);
        assert_eq!(a[2].column, 1);
        assert_eq!(a[2].y_in_column, 0.0);
        assert_eq!(a[3].column, 1);
        assert_eq!(a[3].y_in_column, 20.0);
        assert_eq!(a[4].column, 2, "第 5 行落入溢出列 col2");
        assert_eq!(a[4].y_in_column, 0.0);
    }

    /// 内容恰好填满 col_count 列（无溢出）→ total == col_count（不创建溢出列）。
    /// 4 行 h=20，budget=40，col_count=2 → col0/col1 各 2 行，total=2。
    #[test]
    fn overflow_exact_fit_creates_no_extra_column() {
        let lines = vec![line(20.0); 4];
        let ctx = ctx_auto(2, 40.0);
        let (a, total) = fragment_lines_into_columns_overflow(&lines, &ctx);
        assert_eq!(total, 2, "恰好填满 2 列 → 无溢出列");
        assert_eq!(a[2].column, 1);
        assert_eq!(a[3].column, 1);
    }

    /// 单行高于 budget（无法放入空列）→ 留在当前列，不无限新建空溢出列。
    /// 3 行 [100, 20, 20]，budget=30，col_count=2：line0(100>30) 留 col0；line1/line2
    /// → col0 满（0+20>30? 否，留 col0... 实际 col0=100 后 line1 100+20>30 推进 col1）。
    /// 关键断言：total 有界（不无限增长），line0 不触发新建。
    #[test]
    fn overflow_single_oversized_line_does_not_create_infinite_columns() {
        let lines = vec![line(100.0), line(20.0), line(20.0)];
        let ctx = ctx_auto(2, 30.0);
        let (a, total) = fragment_lines_into_columns_overflow(&lines, &ctx);
        // line0=100 > budget=30 → 留 col0（不新建）。line1: col0 100+20>30 且 20≤30 → 推 col1。
        // line2: col1 20+20>30 且 20≤30 → col1 是末列 → 新建 col2。
        assert_eq!(a[0].column, 0, "单行超高 line0 留 col0");
        assert!(total <= 3, "total 有界（{}），不无限新建", total);
    }

    /// 多重溢出：内容远超 col_count → 创建多个溢出列。9 行 h=20，budget=20，col_count=2
    /// → 每列 1 行，9 行需 9 列（col0..col8，7 个溢出列）。total=9。
    #[test]
    fn overflow_creates_multiple_extra_columns() {
        let lines = vec![line(20.0); 9];
        let ctx = ctx_auto(2, 20.0);
        let (a, total) = fragment_lines_into_columns_overflow(&lines, &ctx);
        assert_eq!(total, 9, "9 行每列 1 行 → 9 列（含 7 溢出列）");
        assert_eq!(a.len(), 9);
        assert_eq!(a[8].column, 8);
        assert_eq!(a[8].y_in_column, 0.0);
    }

    /// 回退（与 fragment_lines_into_columns 一致）：col_count==0 / 长度不符 / budget None 或 ≤0
    /// → 空 Vec + total 0。
    #[test]
    fn overflow_fallbacks_return_empty_and_zero_total() {
        let good = ctx_auto(2, 40.0);
        // col_count 0
        let ctx0 = ColumnFragmentationContext {
            col_count: 0,
            col_width: 0.0,
            col_gap: 0.0,
            available_height: Some(40.0),
            col_filled_heights: Vec::new(),
            fill_mode: ColumnFillMode::Auto,
        };
        let (a, t) = fragment_lines_into_columns_overflow(&[line(20.0)], &ctx0);
        assert!(a.is_empty() && t == 0);
        // 长度不符
        let ctx_bad = ColumnFragmentationContext {
            col_count: 2,
            col_width: 0.0,
            col_gap: 0.0,
            available_height: Some(40.0),
            col_filled_heights: vec![0.0],
            fill_mode: ColumnFillMode::Auto,
        };
        let (a, t) = fragment_lines_into_columns_overflow(&[line(20.0)], &ctx_bad);
        assert!(a.is_empty() && t == 0);
        // budget None
        let ctx_none = ColumnFragmentationContext {
            col_count: 2,
            col_width: 0.0,
            col_gap: 0.0,
            available_height: None,
            col_filled_heights: vec![0.0, 0.0],
            fill_mode: ColumnFillMode::Auto,
        };
        let (a, t) = fragment_lines_into_columns_overflow(&[line(20.0)], &ctx_none);
        assert!(a.is_empty() && t == 0);
        // good ctx 非 0（4 行 h=20，budget=40，2 列恰好填满 → total=2 无溢出）
        let four: Vec<LineBox> = vec![line(20.0), line(20.0), line(20.0), line(20.0)];
        let (_a, t) = fragment_lines_into_columns_overflow(&four, &good);
        assert_eq!(t, 2);
    }
}
