//! 多列布局的列平衡算法（CSS multicol §8 / chromium LayoutNG 经验模型）。
//!
//! 本模块是 R1348c/R1350 建立的 **empirical column-balancing 模型**，经 12 个受控
//! chromium ground-truth 变体（`docs/goal/rendering-compat/empirical/multicol-section-height/`）
//! 验证：**11/12 匹配**（仅 1-span 末区域 L 型 binary-search split 未解，见测试）。
//!
//! **状态：dormant（Phase 2 wiring 待定）**。当前生产路径 `multicol.rs::try_layout_nested_spanner`
//! 用 R1035 的 region_available + multirow 机制；本模块编码的模型供后续 wiring / 重写时复用，
//! 确保 region 列高分布与 chromium 一致。类 R885 font-bridge 的 de-risking 基础设施。
//!
//! ## 模型（per-region）
//!
//! 对一个 spanner 分隔的 region（内容为单个 block，高度 `content`，列数 `num_cols`）：
//!
//! - **非末 region**（后随 spanner）：列高 = `ceil(content / num_cols)`，内容均匀分片。
//!   实测：variant A/D/F 的 a/b region 全部 content/N（100/100、50/50、67/67/66）。
//!
//! - **末 region**（最后一个，无后随 spanner）：可用高 `available = 容器预算 − 前序 region 高 − spans`，
//!   - 若 `content > available`（内容超预算）：forced balance，列高 = `ceil(content / num_cols)`，
//!     末区域内容溢出容器（variant A/K/E、004a）。
//!   - 否则：列高 = `available / num_cols`，按 col0=min(content,h)、col1=min(rem,h) 顺序填充
//!     （variant D/J/O）。
//!
//! ## 未解：容器总高（overflow 案）
//!
//! 容器渲染高度 = Σ region 高 + spans，但 **overflow 案**（末 region forced balance）的容器高
//! 是 chromium LayoutNG row-height 算法的输出（非 closed-form）：简单 overflow 案近似
//! `H − spans`（variant A=350、B=250），但复杂案（variant E，a/b 自身溢出）不符。此部分待
//! LayoutNG 源码访问或更多变体后补（见 roadmap R1348d/R1349）。

/// 单个 region 的列平衡结果。
#[derive(Clone, Debug, PartialEq)]
pub struct RegionBalance {
    /// 该 region 的列高（= 各列的最大片段高；region 视觉高度）。
    pub column_height: f32,
    /// 各列分到的内容片段高（长度 = num_cols）。
    pub fragments: Vec<f32>,
}

/// 计算**非末 region**的列平衡：内容均匀分片到 num_cols 列。
///
/// 列高 = ceil(content / num_cols)；前若干列各承载 ceil，余列承载 floor（CSS 平衡）。
pub fn balance_nonlast_region(content_height: f32, num_cols: usize) -> RegionBalance {
    let n_cols = num_cols.max(1);
    let n = n_cols as f32;
    let per = (content_height / n).ceil();
    let mut fragments = Vec::with_capacity(n_cols);
    let mut rem = content_height;
    for _ in 0..n_cols {
        let f = rem.min(per);
        fragments.push(f);
        rem -= f;
    }
    RegionBalance {
        column_height: per,
        fragments,
    }
}

/// 计算**末 region**的列平衡（empirical 模型，11/12 验证）。
///
/// - `content_height`：末 region 内容（单个 block）高。
/// - `available`：末 region 可用高（容器预算 − 前序 region 高 − spans）。
/// - `num_cols`：列数。
///
/// 规则：content > available → forced balance（列高 ceil(content/N)，溢出）；
/// 否则列高 = available/N，按 col0=min(content,h)、col1=min(rem,h) 顺序填。
pub fn balance_last_region(content_height: f32, available: f32, num_cols: usize) -> RegionBalance {
    let n_cols = num_cols.max(1);
    let n = n_cols as f32;
    let column_height = if content_height > available + 0.5 {
        // forced balance：内容超预算，列高由内容决定（溢出容器）。
        (content_height / n).ceil()
    } else {
        // 内容在预算内：列高 = 可用高 / 列数（chromium 实测 D/J/O）。
        (available / n).max(0.0)
    };
    let mut fragments = Vec::with_capacity(n_cols);
    let mut rem = content_height;
    for _ in 0..n_cols {
        let f = rem.min(column_height).max(0.0);
        fragments.push(f);
        rem -= f;
    }
    RegionBalance {
        column_height,
        fragments,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 12 chromium ground-truth 变体（R1348c/R1350，docs/goal/.../empirical/.../measured-data.txt）
    /// 验证模型。
    #[test]
    fn test_last_region_empirical_12_variants() {
        struct Case {
            label: &'static str,
            content: f32,
            available: f32,
            num_cols: usize,
            exp_column_height: f32,
            exp_fragments: &'static [f32],
        }
        let cases = [
            Case {
                label: "A/004a c",
                content: 200.0,
                available: 150.0,
                num_cols: 2,
                exp_column_height: 100.0,
                exp_fragments: &[100.0, 100.0],
            },
            Case {
                label: "D c",
                content: 100.0,
                available: 250.0,
                num_cols: 2,
                exp_column_height: 125.0,
                exp_fragments: &[100.0, 0.0],
            },
            Case {
                label: "J c",
                content: 100.0,
                available: 150.0,
                num_cols: 2,
                exp_column_height: 75.0,
                exp_fragments: &[75.0, 25.0],
            },
            Case {
                label: "K c",
                content: 100.0,
                available: 50.0,
                num_cols: 2,
                exp_column_height: 50.0,
                exp_fragments: &[50.0, 50.0],
            },
            Case {
                label: "O b",
                content: 200.0,
                available: 200.0,
                num_cols: 2,
                exp_column_height: 100.0,
                exp_fragments: &[100.0, 100.0],
            },
            Case {
                label: "E c",
                content: 400.0,
                available: 150.0,
                num_cols: 2,
                exp_column_height: 200.0,
                exp_fragments: &[200.0, 200.0],
            },
            Case {
                label: "F c",
                content: 200.0,
                available: 116.0,
                num_cols: 3,
                exp_column_height: 67.0,
                exp_fragments: &[67.0, 67.0, 66.0],
            },
        ];
        for c in cases {
            let r = balance_last_region(c.content, c.available, c.num_cols);
            assert!(
                (r.column_height - c.exp_column_height).abs() < 1.5,
                "{}: column_height {} expected {}",
                c.label,
                r.column_height,
                c.exp_column_height
            );
            for (i, (got, exp)) in r.fragments.iter().zip(c.exp_fragments.iter()).enumerate() {
                assert!(
                    (got - exp).abs() < 1.5,
                    "{}: fragment[{}] {} expected {}",
                    c.label,
                    i,
                    got,
                    exp
                );
            }
        }
    }

    /// 末 region L 型（1-span，C=200, A=300）= 已知未解 outlier。
    /// chromium 实测 125/75，模型预测 150/50。标记 ignored 待 LayoutNG row-height 解。
    #[test]
    fn test_last_region_l_outlier_documented() {
        let r = balance_last_region(200.0, 300.0, 2);
        // 模型预测（与 chromium 实测 125/75 不符——LayoutNG binary-search split 未解）。
        assert!((r.column_height - 150.0).abs() < 0.5, "model predicts 150");
        assert_eq!(r.fragments, vec![150.0, 50.0]);
        // ★ 已知不符：chromium 实测 column_height=125, fragments=[125,75]。
        // 此 outlier 是 multicol Phase 2 wiring 时须排除 / 进一步研究的 case。
    }

    #[test]
    fn test_nonlast_region_balances_to_content_over_n() {
        // variant A/D/F 的 a/b region（非末）：content/N
        let r = balance_nonlast_region(200.0, 2);
        assert!((r.column_height - 100.0).abs() < 0.5);
        assert_eq!(r.fragments, vec![100.0, 100.0]);

        let r = balance_nonlast_region(100.0, 2);
        assert!((r.column_height - 50.0).abs() < 0.5);
        assert_eq!(r.fragments, vec![50.0, 50.0]);

        // N=3: 200/3 → 67/67/66
        let r = balance_nonlast_region(200.0, 3);
        assert!((r.column_height - 67.0).abs() < 0.5);
        assert_eq!(r.fragments, vec![67.0, 67.0, 66.0]);
    }

    #[test]
    fn test_num_cols_minimum_1() {
        // num_cols=0 不 panic（兜底为 1）。
        let r = balance_last_region(100.0, 200.0, 0);
        assert_eq!(r.fragments.len(), 1);
    }
}
