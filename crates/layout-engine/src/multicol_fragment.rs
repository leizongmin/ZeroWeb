//! multicol 列感知碎片化工具（CSS Multicol §8 balance + §6 fragmentation）。
//!
//! 为 multicol 列分配（见 `docs/goal/rendering-compat/multicol-fragmentation-design.md`）
//! 提供碎片化算法基础。本模块**仅做纯计算**（不参与布局/paint，compute() 不调用其
//! 改变布局的函数），提供 shortest-column balance 分配 + 单元测试，分轮渐进接线。
//! 同 `intrinsic_sizing` 的「测量先行」方法学。

/// CSS Multicol §8 balance：shortest-column-first 把行盒分配到各列。
///
/// 对每个行盒（按 DOM 顺序），分配到当前**最矮**的列（含起始 `col_filled` 已占高度）。
/// 比纯均高分配（`total/col_count`）更接近 chromium（实测 multicol-columns-001 ref 即
/// shortest-column 模式），且天然消除 fractional target_h 致的列内偏移。
///
/// # 参数
/// - `line_heights`：行盒高度序列（按 DOM 顺序）。
/// - `col_count`：列数（≥1）。
/// - `col_filled`：每列起始已占高度（block 子元素累积），长度须 == col_count；空则全 0。
///
/// # 返回
/// `Vec<Vec<usize>>`，外层 = 列索引，内层 = 该列的行索引序列（按分配顺序）。
/// 行的列内 y 由调用方按累积高度计算。
#[allow(dead_code)] // Round 1：测量工具，未接线（下轮 Round 2 接入 paint text.rs 列分配）
pub(crate) fn balance_lines_to_columns(line_heights: &[f32], col_count: usize, col_filled: &[f32]) -> Vec<Vec<usize>> {
    if col_count == 0 {
        return Vec::new();
    }
    let n = col_count;
    let mut heights: Vec<f32> = if col_filled.len() == n {
        col_filled.to_vec()
    } else {
        vec![0.0; n]
    };
    let mut columns: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (line_idx, &h) in line_heights.iter().enumerate() {
        // 分配到当前最矮列（并列取最左）
        let col = heights
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);
        columns[col].push(line_idx);
        heights[col] += h;
    }
    columns
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_even_split_4_lines_2_cols() {
        // 4 行（高 10）/ 2 列 → shortest-column 轮转：col0=[0,2], col1=[1,3]
        // line0→col0(h10); line1→col1(0最矮,h10); line2→col0(10==10 取左,h20); line3→col1(10<20,h20)
        let cols = balance_lines_to_columns(&[10.0, 10.0, 10.0, 10.0], 2, &[]);
        assert_eq!(cols.len(), 2);
        assert_eq!(cols[0], vec![0, 2]);
        assert_eq!(cols[1], vec![1, 3]);
    }

    #[test]
    fn test_non_divisible_11_lines_6_cols() {
        // 11 行（高 10）/ 6 列 → shortest-column 分布 ~2,2,2,2,2,1
        let lines = vec![10.0; 11];
        let cols = balance_lines_to_columns(&lines, 6, &[]);
        assert_eq!(cols.len(), 6);
        let total: usize = cols.iter().map(|c| c.len()).sum();
        assert_eq!(total, 11);
        // 前 5 列各 2 行，末列 1 行（shortest-column 轮转）
        let counts: Vec<usize> = cols.iter().map(|c| c.len()).collect();
        assert_eq!(counts, vec![2, 2, 2, 2, 2, 1]);
    }

    #[test]
    fn test_with_col_filled_block_occupied() {
        // col0 已被 block 占 30px，3 行（高 10）/ 2 列
        // col0 起始 30，col1 起始 0：line0→col1(0), line1→col1(10), line2→col1(20) → 全归 col1
        let cols = balance_lines_to_columns(&[10.0, 10.0, 10.0], 2, &[30.0, 0.0]);
        assert_eq!(cols[0], Vec::<usize>::new()); // col0 已满（30），不再分配
        assert_eq!(cols[1], vec![0, 1, 2]);
    }

    #[test]
    fn test_single_column_all_lines() {
        let cols = balance_lines_to_columns(&[5.0, 7.0, 3.0], 1, &[]);
        assert_eq!(cols, vec![vec![0, 1, 2]]);
    }

    #[test]
    fn test_zero_cols_empty() {
        assert!(balance_lines_to_columns(&[10.0], 0, &[]).is_empty());
    }

    #[test]
    fn test_empty_lines() {
        let cols = balance_lines_to_columns(&[], 3, &[]);
        assert_eq!(cols.len(), 3);
        assert!(cols.iter().all(|c| c.is_empty()));
    }
}
