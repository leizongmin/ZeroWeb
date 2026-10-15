//! multicol 子元素列分配算法（balanced / breaking / multirow / sequential）与
//! 片段定位（`position_multicol_children`）、子树列宽约束。自 multicol.rs 按职责
//! 拆出（文件 ≤2000 行债，R5141）。

use std::collections::HashMap;

use zero_css_parser::values::LengthValue;
use zero_dom::NodeId;
use zero_style_system::ComputedStyle;

use crate::types::LayoutBox;

use super::{ColumnFragment, ColumnInfo};
/// 计算列高限制（用于 column breaking 判断）。
///
/// R1820：forced-break overflow column + auto-height recompute kill-switch（LANDED default-on）。
///
/// 承接 R1817/R1818（code-trace 诊断未经验证属假阳性，A/B 零效果已 revert）。R1820 经
/// REFTEST_DEBUG 实证确证真根因：multicol-fill-auto-005 ZW 输出容器高 160px（= 自然高度和），
/// chromium 100px（forced breaks > column-count 时创建溢出列，容器高 = max 列高）。两层：
/// (1) 主路径 `let _ = position_multicol_children`（multicol.rs:927）丢弃 region_height →
/// 容器高永不按列分配结果重算；(2) 末列 forced break 不创建溢出列（`current_col+1<col_count`
/// 守卫）。本开关同时启用两层；全 multicol corpus A/B net +1 零 pass 回归后 LANDED default-on。
/// `ZW_MULTICOL_FORCED_OVERFLOW=0` 可紧急关闭。
pub(super) fn forced_overflow_enabled() -> bool {
    // R1820 LANDED default-on（全 multicol corpus A/B net +1 零 pass 回归；
    // multicol-fill-auto-005 1.87%→0.62% flip，multicol-nested-027 0.62→0.87 仍 pass）。
    // ZW_MULTICOL_FORCED_OVERFLOW=0 可紧急关闭。
    std::env::var("ZW_MULTICOL_FORCED_OVERFLOW").as_deref() != Ok("0")
}

/// 均衡分配子元素到各列（顺序流 + 目标高度策略）。
///
/// CSS Multi-column Layout §3.3：在 column-fill: balance（默认）模式下，
/// 内容应尽可能均衡地分布在各列中。内容按文档顺序依次填入各列。
///
/// 算法：
/// 1. 计算所有子元素的总高度
/// 2. 目标列高 = 总高度 / 列数
/// 3. 按文档顺序将子元素填入当前列，当列高超过目标时移至下一列
/// 4. **R1037 balance-mode column-breaking**：单个子元素超过目标列高时跨列拆分
///    （每片填满 target 边界），避免超大子元素整体留单列破坏均衡。**仅对 explicit-height
///    子元素启用**（`explicit_height[i]=true`）——CSS Fragmentation monolithic 元素
///    （overflow≠visible 滚动容器 / 替换元素 / auto 高度）不可分。
///
/// 这比 shortest-column-first 更符合规范行为：内容按顺序流过各列，
/// 而非被任意分配到最短列。
pub(super) fn assign_children_to_columns_balanced(
    children: &[(usize, f32)],
    col_count: usize,
    forced_breaks: &[bool],
    forced_breaks_after: &[bool],
    explicit_height: &[bool],
    unbreakable: &[bool],
    min_target: f32,
) -> Vec<Vec<ColumnFragment>> {
    if children.is_empty() || col_count == 0 {
        return vec![Vec::new(); col_count.max(1)];
    }

    // R4514b：零预算区域（total≈0，如 004b-ref article3 的 container height:0）——
    // chromium 语义：fragmentainer 无空间 → block 不可见。发 0 高片段，paint 按 cso
    // slice 裁空；否则子盒以全高单片段落 col1（target=0 的退化 advance）泄漏渲染。
    let total_probe: f32 = children.iter().map(|&(_, h)| h).sum();
    if total_probe <= 0.5 {
        let mut columns: Vec<Vec<ColumnFragment>> = vec![Vec::new(); col_count];
        for &(child_idx, _) in children {
            columns[0].push(ColumnFragment {
                child_idx,
                fragment_y_offset: 0.0,
                visual_height: 0.0,
            });
        }
        return columns;
    }

    // 计算总高度和目标列高（R4509：不可分约束下限，见 layout_multicol 调用处）
    let total_height: f32 = children.iter().map(|&(_, h)| h).sum();
    let target_height = (total_height / col_count as f32).max(min_target);

    let mut columns: Vec<Vec<ColumnFragment>> = vec![Vec::new(); col_count];
    let mut current_col = 0usize;
    let mut current_col_height = 0.0f32;

    for (i, &(child_idx, child_height)) in children.iter().enumerate() {
        // break-before:column：当前列已有内容时强制推进到下一列（R903 消费死值 break_before）。
        if forced_breaks.get(i).copied().unwrap_or(false) && current_col_height > 0.0 && current_col + 1 < col_count {
            current_col += 1;
            current_col_height = 0.0;
        }
        // 如果当前列已超过目标高度且还有更多列可用，移到下一列
        if current_col_height >= target_height && current_col + 1 < col_count {
            current_col += 1;
            current_col_height = 0.0;
        }

        // R5132：不可分（嵌套 multicol）子**先推后放**——当前列已有内容且放置将超
        // target 时先推进新列。旧序「放置后查达标才推进」把不等高的不可分子整体
        // 堆进已达标列（nested-005：[80,100,100]/target 93.3 → [[0,1],[2],[]] col2
        // 空置；ref 为一子一列）。等高子与旧序数学等价；可分子不入本臂（既有已调
        // 簇 R1037/R4522 行为不变）。仅顶层容器启用（调用处传 &[]，同 R4509 floor
        // 的 !nested gate，nested-030 教训）。spanner region balance 调用处传 &[]，
        // 不受影响（R5131 全局先推后放 fieldset×3 误伤教训）。
        if unbreakable.get(i).copied().unwrap_or(false)
            && current_col_height > 0.0
            && current_col_height + child_height > target_height
            && current_col + 1 < col_count
        {
            current_col += 1;
            current_col_height = 0.0;
        }

        // R1037：balance-mode column-breaking，仅对 explicit-height 子元素启用。
        // CSS Fragmentation：monolithic 元素（overflow≠visible 滚动容器 / 替换元素 / auto 高度）
        // 不可分。explicit-height（length/percentage）子元素可跨列拆分以均衡分布。
        // gate 排除 overflow-unsplittable（overflow:scroll + auto height → 不拆）等回归案，
        // 捕获 span-all-children-height（height:200px/100% explicit → 拆）目标簇。
        // R1036 通用应用 net -12（18 回归），本 gate 避免回归。
        let is_explicit = explicit_height.get(i).copied().unwrap_or(false);
        if is_explicit && child_height > target_height && target_height > 0.0 {
            // 先消耗当前列剩余空间。
            let available = (target_height - current_col_height).max(0.0);
            if available > 0.0 {
                columns[current_col].push(ColumnFragment {
                    child_idx,
                    fragment_y_offset: 0.0,
                    visual_height: available,
                });
            }
            // 后续片段按 target_height 填充连续列；末列之外 clip（与 with_breaking 一致）。
            let mut offset = available;
            while offset < child_height {
                if current_col + 1 < col_count {
                    current_col += 1;
                } else {
                    break;
                }
                let remaining = child_height - offset;
                let frag_height = remaining.min(target_height);
                columns[current_col].push(ColumnFragment {
                    child_idx,
                    fragment_y_offset: offset,
                    visual_height: frag_height,
                });
                offset += frag_height;
                current_col_height = frag_height;
            }
            if current_col + 1 >= col_count {
                current_col_height = target_height;
            }
        } else {
            columns[current_col].push(ColumnFragment {
                child_idx,
                fragment_y_offset: 0.0,
                visual_height: child_height,
            });
            current_col_height += child_height;
        }
        // break-after:column：放置完子元素后强制推进到下一列（R1027 消费死值 break_after，mirror R903 break-before）。
        if forced_breaks_after.get(i).copied().unwrap_or(false) && current_col + 1 < col_count {
            current_col += 1;
            current_col_height = 0.0;
        }
    }

    columns
}

/// 带列高限制的顺序分配（column breaking 实现）。
///
/// 按文档顺序将子元素填入当前列，当子元素超出列高限制时：
/// - 如果子元素可以整体放入下一列，则移动到下一列
/// - 如果子元素本身超过列高（oversized），则拆分为多个片段，
///   每个片段放入连续的列中
///
/// CSS Multi-column Layout §2 "column breaking"：
/// 当一个块级子元素高度超过列高时，内容应自动延续到后续列中。
pub(super) fn assign_children_to_columns_with_breaking(
    children: &[(usize, f32)],
    col_count: usize,
    max_col_height: f32,
    forced_breaks: &[bool],
    forced_breaks_after: &[bool],
    // R5135/R5136：嵌套 mc 子的首不可断单元高（0 = 可断，照常切片/整放）。
    nested_unbreakable: &[f32],
    // R5140：嵌套 mc 子 + break-before:avoid（avoid-stay 候选；其余路径传空）。
    nested_avoid: &[bool],
) -> Vec<Vec<ColumnFragment>> {
    let mut columns: Vec<Vec<ColumnFragment>> = vec![Vec::new(); col_count];
    let mut current_col = 0usize;
    let mut current_col_height = 0.0f32;
    // R1820：forced break 命中末列时创建溢出列（chromium parity，env-gated default-off）。
    let forced_overflow = forced_overflow_enabled();

    for (i, &(child_idx, child_height)) in children.iter().enumerate() {
        // break-before:column：当前列已有内容时强制推进到下一列（R903 消费死值 break_before）。
        if forced_breaks.get(i).copied().unwrap_or(false) && current_col_height > 0.0 {
            if current_col + 1 < col_count {
                current_col += 1;
                current_col_height = 0.0;
            } else if forced_overflow {
                // R1820：末列 forced break → 创建溢出列（chromium 对「forced breaks 多于
                // column-count」创建额外 column box 于 inline 方向溢出，非堆末列）。
                columns.push(Vec::new());
                current_col = columns.len() - 1;
                current_col_height = 0.0;
            }
        }
        let available = max_col_height - current_col_height;

        if child_height <= available {
            // 子元素完全适应当前列剩余空间
            columns[current_col].push(ColumnFragment {
                child_idx,
                fragment_y_offset: 0.0,
                visual_height: child_height,
            });
            current_col_height += child_height;
        } else if nested_avoid.get(i).copied().unwrap_or(false) && current_col_height > 0.0 && available > 0.0 {
            // R5140：嵌套 mc 子 break-before:avoid + 当前列部分填充 + 整子放不下 →
            // 留在当前列（avoid 禁止盒前非受迫断点，css-break-3 §possible-breaks
            // Class A），行 1 限高 = 剩余空间、列消耗到底——碎片化语境 auto 高
            // sequential mc 盒高 = min(堆叠高, fragmentainer 余量)（Chrome probe
            // 定谳：nested-015/019/020 全一致）。nested_row_limit 由调用方写回，
            // 驱动子自身 adjust pass 重分布内层列；盒高不截断（兄弟由列满自然接排）。
            columns[current_col].push(ColumnFragment {
                child_idx,
                fragment_y_offset: 0.0,
                visual_height: available,
            });
            current_col_height = max_col_height;
        } else if child_height <= max_col_height {
            // 子元素可以整体放入下一列（当列剩余不够但列高足够）
            if current_col + 1 < col_count {
                current_col += 1;
                current_col_height = 0.0;
            }
            // 如果没有更多列，保留在当前列（clip 处理）
            columns[current_col].push(ColumnFragment {
                child_idx,
                fragment_y_offset: 0.0,
                visual_height: child_height.min(max_col_height),
            });
            current_col_height += child_height.min(max_col_height);
        } else if nested_unbreakable.get(i).copied().unwrap_or(0.0) > available
            && current_col_height > 0.0
            && current_col + 1 < col_count
        {
            // R5136：嵌套 multicol 子首不可断单元放不进当前列**剩余空间**（部分填充
            // 列）→ 先推进下一列再整子放置（css-break-3 §possible-breaks Class A2：
            // unforced break 落在盒前；chromium 024 实证：内层 mc 首子 contain:size
            // 100 > 剩余 50 → 内层整体落外层 col1，col0 底部留空）。整子单片段
            //（不可断单元内部无断点机会，不跨外层列切片）。
            current_col += 1;
            current_col_height = 0.0;
            columns[current_col].push(ColumnFragment {
                child_idx,
                fragment_y_offset: 0.0,
                visual_height: child_height,
            });
            current_col_height += child_height;
        } else if nested_unbreakable.get(i).copied().unwrap_or(0.0) > 0.0 {
            // R5135：嵌套 multicol 子（自身为碎片化上下文）超高（> 列高）→ 整子单片段
            // 原地放置，不跨外层列切片。chromium headless 实测（2026-10-15，probeA/011）：
            // 外层不对嵌套碎片化上下文做 block 方向切片续排（011 内层 mc 单片段整放，
            // 其子纵向溢出绘制——fresh 列 Class C3：不可断单元高于 fragmentainer 原地
            // 溢出，不推进）。visual_height 取全高：flow 上盒子占据全高（后续子接排），
            // paint 层非 breaking 路径（cso len=1）纵向不裁剪，子树溢出列底可见。
            columns[current_col].push(ColumnFragment {
                child_idx,
                fragment_y_offset: 0.0,
                visual_height: child_height,
            });
            current_col_height += child_height;
        } else {
            // 子元素超高（> max_col_height）— 需要 column breaking
            // 先消耗当前列剩余空间
            if available > 0.0 {
                columns[current_col].push(ColumnFragment {
                    child_idx,
                    fragment_y_offset: 0.0,
                    visual_height: available,
                });
                // 仅当还有更多列时才推进；单列或末列时保留在当前列（clip 处理），
                // 否则 current_col 越界使后续子元素 columns[current_col].push panic。
                if current_col + 1 < col_count {
                    current_col += 1;
                }
            }

            // 后续片段填满整列。
            // max_col_height > 0.0 守卫：若列高为 0（height:0 multicol 或计算得 0），
            // offset += max_col_height(0) 永不前进会无限循环——此时无法细分，clip 跳出。
            let mut offset = available;
            while offset < child_height && current_col < col_count && max_col_height > 0.0 {
                let remaining = child_height - offset;
                let frag_height = remaining.min(max_col_height);
                columns[current_col].push(ColumnFragment {
                    child_idx,
                    fragment_y_offset: offset,
                    visual_height: frag_height,
                });
                offset += max_col_height;
                current_col_height = frag_height;
                if frag_height >= max_col_height && current_col + 1 < col_count {
                    current_col += 1;
                    current_col_height = 0.0;
                }
            }
        }
        // break-after:column：放置完子元素（含其全部 breaking 片段）后强制推进到下一列
        //（R1027 消费死值 break_after，mirror R903 break-before）。
        if forced_breaks_after.get(i).copied().unwrap_or(false) && current_col + 1 < col_count {
            current_col += 1;
            current_col_height = 0.0;
        }
    }

    columns
}

/// 多行列模型分配（CSS Multi-column Layout §3 multi-row）。
///
/// 内容溢出 `col_count` 列后创建额外行（row 2, 3...）。当 `current_col` 超过
/// `col_count` 时**不截断到末列**，而是换行到 row+1 col 0（动态 `push` 新列）。
/// 返回扁平 `Vec`（`len = rows × col_count`），由 `position_multicol_children` 据
/// `row_height` 还原 row/col。
///
/// R1035：用于 spanner 路径每个区域的 overflow 处理（region 内容超过 region 可用高度时）。
pub(super) fn assign_children_to_columns_multirow(
    children: &[(usize, f32)],
    col_count: usize,
    max_col_height: f32,
    cap_nested: &[bool],
) -> Vec<Vec<ColumnFragment>> {
    if children.is_empty() || col_count == 0 || max_col_height <= 0.0 {
        return vec![Vec::new(); col_count.max(1)];
    }
    let mut columns: Vec<Vec<ColumnFragment>> = vec![Vec::new(); col_count];
    let mut current_col = 0usize;
    let mut current_col_height = 0.0f32;
    macro_rules! advance_col {
        () => {{
            current_col += 1;
            current_col_height = 0.0;
            if current_col >= columns.len() {
                columns.push(Vec::new());
            }
        }};
    }

    for (ci, &(child_idx, child_height)) in children.iter().enumerate() {
        // R4522：嵌套 multicol 子（自身为碎片化上下文）在 definite-height balance 容器
        // 溢出时，超出 col_count 的行片段**剪裁不渲染**（children-height-007：chromium
        // 对嵌套 multicol 子无内联溢出列；与平铺子的 R1075 溢出列模型分叉）。cap 标志
        // 由调用方按子样式判定。
        let cap_at_count = cap_nested.get(ci).copied().unwrap_or(false);
        let available = max_col_height - current_col_height;
        if child_height <= available {
            columns[current_col].push(ColumnFragment {
                child_idx,
                fragment_y_offset: 0.0,
                visual_height: child_height,
            });
            current_col_height += child_height;
        } else if child_height <= max_col_height {
            // cap：无列可推进 → 整子剪裁（零高片段 → painter 零高 clip 全裁子树图元；
            // R4524：004-ref inner#2 整体落在 outer fragmentainer 之外，chromium 不渲染）。
            if cap_at_count && current_col + 1 >= col_count {
                columns[current_col].push(ColumnFragment {
                    child_idx,
                    fragment_y_offset: 0.0,
                    visual_height: 0.0,
                });
                continue;
            }
            advance_col!();
            columns[current_col].push(ColumnFragment {
                child_idx,
                fragment_y_offset: 0.0,
                visual_height: child_height,
            });
            current_col_height += child_height;
        } else {
            // 子元素超高（> max_col_height）— column breaking 跨多列（含跨行）。
            if available > 0.0 {
                columns[current_col].push(ColumnFragment {
                    child_idx,
                    fragment_y_offset: 0.0,
                    visual_height: available,
                });
                // cap：首片已占满最后一列（col_count−1）→ 剩余行剪裁，不创建溢出列
                //（R4523：count:1 嵌套 multicol 子 500>300 整段截断）。
                if cap_at_count && current_col + 1 >= col_count {
                    continue;
                }
                advance_col!();
            }
            let mut offset = available;
            while offset < child_height && max_col_height > 0.0 {
                // cap：游标已在 col_count 之外（不应发生，防御）→ 剪裁。
                if cap_at_count && current_col >= col_count {
                    break;
                }
                let remaining = child_height - offset;
                let frag_height = remaining.min(max_col_height);
                columns[current_col].push(ColumnFragment {
                    child_idx,
                    fragment_y_offset: offset,
                    visual_height: frag_height,
                });
                offset += max_col_height;
                current_col_height = frag_height;
                if frag_height >= max_col_height && offset < child_height {
                    // cap：已占满最后一列（col_count−1）且仍有剩余行 → 剪裁，不再推进
                    // 创建溢出列。
                    if cap_at_count && current_col + 1 >= col_count {
                        break;
                    }
                    advance_col!();
                }
            }
        }
    }

    columns
}

/// 按顺序填充列（column-fill: auto）。
///
/// 子元素按文档顺序依次填入当前列，当列高度达到容器高度时移至下一列。
pub(super) fn assign_children_to_columns_sequential(
    children: &[(usize, f32)],
    col_count: usize,
    container_height: f32,
    forced_breaks: &[bool],
    forced_breaks_after: &[bool],
) -> Vec<Vec<ColumnFragment>> {
    let mut columns: Vec<Vec<ColumnFragment>> = vec![Vec::new(); col_count];
    let mut current_col = 0usize;
    let mut current_col_height = 0.0f32;

    for (i, &(child_idx, child_height)) in children.iter().enumerate() {
        // break-before:column：当前列已有内容时强制推进到下一列（R903 消费死值 break_before）。
        if forced_breaks.get(i).copied().unwrap_or(false) && current_col_height > 0.0 && current_col + 1 < col_count {
            current_col += 1;
            current_col_height = 0.0;
        }
        // 如果当前列放不下，且还有更多列可用，移到下一列
        if current_col_height + child_height > container_height
            && current_col_height > 0.0
            && current_col + 1 < col_count
        {
            current_col += 1;
            current_col_height = 0.0;
        }

        columns[current_col].push(ColumnFragment {
            child_idx,
            fragment_y_offset: 0.0,
            visual_height: child_height,
        });
        current_col_height += child_height;
        // break-after:column：放置完子元素后强制推进到下一列（R1027 消费死值 break_after，mirror R903 break-before）。
        if forced_breaks_after.get(i).copied().unwrap_or(false) && current_col + 1 < col_count {
            current_col += 1;
            current_col_height = 0.0;
        }
    }

    columns
}

/// 根据列分配结果定位每个子元素。
///
/// 子元素坐标相对于容器 content area（与 taffy/float 后处理一致），
/// 因此列 x 从 0 开始，不需要加 content_x/content_y。
///
/// 对于 column breaking 拆分的片段，使用负 y 偏移（fragment_y_offset）
/// 来显示子元素内容的不同垂直切片。paint 层通过容器的 overflow 裁剪
/// 确保每列只显示对应片段的内容。
///
/// 当一个子元素因 column breaking 出现在多个列中时：
/// - 第一个片段的位置存储在 child.x/y（主位置）
/// - 后续片段存储在 child.column_span_offsets
/// - paint 层对每个额外片段重新绘制子元素，并裁剪到对应列区域
pub(super) fn position_multicol_children(
    container: &mut LayoutBox,
    assignments: &[Vec<ColumnFragment>],
    info: &ColumnInfo,
    y_base: f32,
    row_height: f32,
    truncate_leading_margin: bool,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> f32 {
    // 跟踪每个子元素已出现的片段数（用于区分主片段和额外片段）
    let mut child_fragment_count: HashMap<usize, usize> = HashMap::new();
    let mut region_height = 0.0f32;
    // R1035：multi-row 列模型（CSS Multicol §3）。assignments 为扁平布局
    // （索引 = row × col_count + col_in_row）。row_height=0.0 表示单行（向后兼容，
    // assignments.len() ≤ col_count 时 row 恒为 0）。
    let col_count = info.count.max(1);

    for (col_idx, col_fragments) in assignments.iter().enumerate() {
        let col_in_row = if row_height > 0.0 { col_idx % col_count } else { col_idx };
        let row = if row_height > 0.0 { col_idx / col_count } else { 0 };
        let col_x = col_in_row as f32 * (info.column_width + info.gap);
        let mut y_offset = y_base + (row as f32) * row_height;
        // 本列是否尚在顶端（未放置任何片段）
        let mut at_column_top = true;

        for frag in col_fragments {
            let child = &mut container.children[frag.child_idx];
            let frag_idx = *child_fragment_count
                .entry(frag.child_idx)
                .and_modify(|c| *c += 1)
                .or_insert(0);

            // R5132/css-break-3 §3.1：列顶是 fragmentation 边缘，邻接 block-start
            // margin 截断（nested-005 ref 实证：col1/2 内层 div mt=20 截断，三列
            // 内容同顶对齐、容器高一致）。截断的 margin 同步从列消耗高中扣除。
            // 首列顶端不入截断（首子 margin 走常规塌穿/保留模型，corpus 已调）；
            // 仅 balance 分配路径启用（truncate_leading_margin，multirow/fixed-
            // column-height/sequential 语义未校准不入）。
            // https://drafts.csswg.org/css-break-3/#break-margins
            let skipped_margin_top = if truncate_leading_margin && at_column_top && col_idx > 0 {
                child.margin_top
            } else {
                0.0
            };
            at_column_top = false;

            let child_x = col_x + child.margin_left;
            let child_y = y_offset + child.margin_top - skipped_margin_top - frag.fragment_y_offset;

            // 所有片段（包括主片段）存储到 column_span_offsets。
            // paint 层根据 column_span_offsets 的存在跳过正常渲染，
            // 并对每个片段进行独立的列区域裁剪渲染。
            // R1039：扩存 col_top（片段列顶 y_offset）+ col_h（片段视觉高 visual_height），
            // 供 paint 把 breaking 片段裁到自己的 slice 范围 [col_top, col_top+col_h]，
            // 而非容器全高（修 span-all-children-height-002 block1 全 200px 覆盖 spanner 区）。
            // 格式：(x_in_container, y_in_container, column_x, column_width, col_top, col_h)
            child
                .column_span_offsets
                .push((child_x, child_y, col_x, info.column_width, y_offset, frag.visual_height));

            if frag_idx == 0 {
                // 第一个片段同时设置主位置（用于非 column-breaking 的子元素
                // 和作为后备渲染位置）
                child.x = child_x;
                child.y = child_y;
            }

            y_offset += frag.visual_height - skipped_margin_top;

            // CSS Multi-column Layout：子元素宽度限制到列宽。
            // 仅对第一个片段执行宽度约束（避免重复递归）。
            // R5132：border-box 可用宽 = 列宽 − 水平 margin（css-multicol §4 与
            // BFC 块宽方程同式；nested-005 实证：内层 div ml/mr=20 时 border-box
            // 260 → 内列 73.3px，ref 220 → 60px）。
            // R5135：约束按指定 width 分类——百分比宽以**列宽**为基准重导出
            // （chromium probeI 实测：100px 容器 2 列下子 width:100% 绘 50px = 列宽、
            // 150% 绘 75px，taffy 以容器内容宽解析须换算；011 width:400% 溢出几何
            // 不可灭）；其余（auto/显式长度）维持基线 clamp——span-all 全带
            // div{width:11em} 实证：显式长度保持溢出 +6 红，corpus 校准优先。
            let specified_width = child.node_id.and_then(|id| styles.get(&id)).map(|s| &s.width);
            let avail_border_box = (info.column_width - child.margin_left - child.margin_right).max(0.0);
            if frag_idx == 0 && child.width > avail_border_box {
                if let Some(LengthValue::Percentage(p)) = specified_width {
                    // R5135：百分比宽以**列宽**为基准重导出（chromium probeI 实测：
                    // 100px 容器 2 列下子 width:100% 绘 50px = 列宽、150% 绘 75px；
                    // taffy 以容器内容宽解析且上游 auto clamp 的
                    // constrain_subtree_width 级联已压盒宽——011 实证 400% 盒宽被压
                    // 至 50 后再缩放得 25，故必须由指定百分比直接重导出，不得缩放
                    // 既有盒宽）。显式长度宽（Px/Em 归一）不在此臂——span-all 全带
                    // 实证（div{width:11em} 4 列容器，保持溢出会使 6 案回归），维持
                    // 基线 clamp。
                    child.width = (*p as f32 * info.column_width / 100.0).max(0.0);
                    let new_content_w = (child.width
                        - child.border_left
                        - child.border_right
                        - child.padding_left
                        - child.padding_right)
                        .max(0.0);
                    child.content_width = new_content_w;
                    child.content_x = child.border_left + child.padding_left;
                    constrain_subtree_width(child, new_content_w);
                } else {
                    let _old_width = child.width;
                    child.width = avail_border_box;
                    let new_content_w = (avail_border_box
                        - child.border_left
                        - child.border_right
                        - child.padding_left
                        - child.padding_right)
                        .max(0.0);
                    child.content_width = new_content_w;
                    child.content_x = child.border_left + child.padding_left;
                    constrain_subtree_width(child, new_content_w);
                }
            }
        }
        // region 高度 = 各列达到的最大 y（相对 y_base）。各列 balance 后高度相近。
        if y_offset - y_base > region_height {
            region_height = y_offset - y_base;
        }
    }
    region_height
}

/// 递归约束子树中所有元素的宽度不超过指定最大值。
///
/// 用于 multicol 列宽约束：子元素被 taffy 按容器全宽布局，
/// 但实际需要约束到列宽。此函数递归更新所有后代的 width
/// 和 content_width，确保内部布局不会溢出列边界。
///
/// R5138：OOF（absolute/fixed）子树不入约束链——列宽约束只作用于 in-flow
/// 内容（css-multicol §4 约束的是列内布局；css-position §10.1 OOF 宽按其
/// containing block 解析，与其 DOM 祖先内容宽无关）。此前约束经 OOF 链
/// content 宽传导：OOF 中间层 shrink-to-fit 宽 0（如 auto 宽 absolute 容器）
/// → 其下 fixed 子的显式 width 也被钳 0（fixed-in-multicol-with-transform-
/// container 等三案 fixed 绿方整体消失）。
pub(super) fn constrain_subtree_width(box_node: &mut LayoutBox, max_width: f32) {
    if box_node.width > max_width {
        let new_width = max_width;
        let new_content_w =
            (new_width - box_node.border_left - box_node.border_right - box_node.padding_left - box_node.padding_right)
                .max(0.0);
        box_node.width = new_width;
        box_node.content_width = new_content_w;
    }
    // 递归约束子元素
    let child_max = box_node.content_width;
    for child in &mut box_node.children {
        if child.is_absolute || child.is_fixed {
            continue;
        }
        constrain_subtree_width(child, child_max);
    }
}
