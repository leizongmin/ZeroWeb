//! multicol.rs 多列布局回归测试（从 multicol.rs 抽出，保持 2000 行约束）。

use super::*;

/// R904：em 单位按 element font-size 解析（非 root 16）。column-width:2em 在
/// font-size 20px 容器内 = 40px（旧实现误为 32px = 2×16）。
#[test]
fn test_length_to_px_em_uses_element_font_size() {
    use zero_css_parser::values::LengthValue;
    // 2em @ font-size 20px → 40px（非 32px）。
    assert!(
        (length_to_px(&LengthValue::Em(2.0), 800.0, 20.0) - 40.0).abs() < 0.01,
        "em must resolve against element font-size (2em@20px=40), not root 16"
    );
    // 1em @ font-size 16px（默认）→ 16px（不变，零回归）。
    assert!((length_to_px(&LengthValue::Em(1.0), 800.0, 16.0) - 16.0).abs() < 0.01);
    // Px/Percentage 不受 font_size_px 影响。
    assert!((length_to_px(&LengthValue::Px(50.0), 800.0, 20.0) - 50.0).abs() < 0.01);
    assert!((length_to_px(&LengthValue::Percentage(10.0), 800.0, 20.0) - 80.0).abs() < 0.01);
}

#[test]
fn test_compute_column_count_basic() {
    // 800px 容器, 200px 最小列宽, 0 gap → 4 列
    assert_eq!(compute_column_count(800.0, 200.0, 0.0), 4);
}

#[test]
fn test_compute_column_count_with_gap() {
    // 800px 容器, 200px 最小列宽, 20px gap
    // n <= (800 + 20) / (200 + 20) = 820 / 220 = 3.72 → 3
    assert_eq!(compute_column_count(800.0, 200.0, 20.0), 3);
}

#[test]
fn test_compute_single_column_width() {
    // 800px / 3 列, 20px gap
    // total_gap = 2 * 20 = 40
    // column_width = (800 - 40) / 3 = 253.33
    let w = compute_single_column_width(800.0, 3, 20.0);
    assert!((w - 253.333).abs() < 1.0);
}

#[test]
fn test_assign_children_balanced_sequential() {
    // 5 children, each 100px high, 3 columns
    // Total = 500, target = 166.67
    // Sequential:
    // child0(100): col0=100 < 166.67 → col0=[0]
    // child1(100): col0=200 >= 166.67 → col1=[1]
    // child2(100): col1=100 < 166.67 → col1=[1,2]
    // Wait, col1=100+100=200 >= 166.67, so child2 goes to col1
    // Actually: child1 in col1, child2: col1=100 < 166.67 → col1=[1,2]
    // No: after child1 fills col1 to 100, child2: col1=100 < 166.67, so add child2.
    // col1 now = 200 >= 166.67
    // child3: col1=200 >= 166.67 → col2=[3]
    // child4: col2=100 < 166.67 → col2=[3,4]
    let children = vec![(0, 100.0), (1, 100.0), (2, 100.0), (3, 100.0), (4, 100.0)];
    let cols = assign_children_to_columns_balanced(&children, 3, &[false; 5], &[false; 5], &[]);
    assert_eq!(cols.len(), 3);
    assert_eq!(cols[0].len(), 2); // [0, 1]
    assert_eq!(cols[1].len(), 2); // [2, 3]
    assert_eq!(cols[2].len(), 1); // [4]
}

#[test]
fn test_assign_children_balanced_uneven() {
    // 3 children: 200, 100, 200; 2 columns
    // Total = 500, target = 250
    // child0(200): col0=200 < 250 → col0=[0]
    // child1(100): col0=300 >= 250 → col1=[1]
    // child2(200): col1=100 < 250 → col1=[1,2]
    // Wait: child1(100): col0=200 < 250, so it's added to col0! col0=[0,1], height=300
    // child2(200): 300 >= 250 → col1=[2], height=200
    let children = vec![(0, 200.0), (1, 100.0), (2, 200.0)];
    let cols = assign_children_to_columns_balanced(&children, 2, &[false; 5], &[false; 5], &[]);
    assert_eq!(cols.len(), 2);
    assert_eq!(cols[0].len(), 2); // [0, 1]
    assert_eq!(cols[1].len(), 1); // [2]
}

#[test]
fn test_assign_children_balanced_equal() {
    // 4 children, each 100px high, 2 columns
    // Total = 400, target = 200
    // child0(100): col0=100 < 200 → col0=[0]
    // child1(100): col0=200 >= 200 → col1=[1]
    // Wait: 100 < 200, so child1 is added! col0=[0,1], h=200
    // child2(100): 200 >= 200 → col1=[2], h=100
    // child3(100): 100 < 200 → col1=[2,3], h=200
    let children = vec![(0, 100.0), (1, 100.0), (2, 100.0), (3, 100.0)];
    let cols = assign_children_to_columns_balanced(&children, 2, &[false; 5], &[false; 5], &[]);
    assert_eq!(cols.len(), 2);
    assert_eq!(cols[0].len(), 2); // [0, 1]
    assert_eq!(cols[1].len(), 2); // [2, 3]
}

/// R1037：balance-mode column-breaking 对 explicit-height 子元素跨列拆分。
/// 单个 200px explicit 子 + 2 列 → target=100，child>target → 拆 2 片各 100px。
#[test]
fn test_assign_children_balanced_explicit_height_breaks() {
    let children = vec![(0, 200.0)];
    let cols = assign_children_to_columns_balanced(&children, 2, &[false; 1], &[false; 1], &[true]);
    assert_eq!(cols.len(), 2);
    assert_eq!(cols[0].len(), 1);
    assert_eq!(cols[0][0].fragment_y_offset, 0.0);
    assert!((cols[0][0].visual_height - 100.0).abs() < 0.01);
    assert_eq!(cols[1].len(), 1);
    assert!((cols[1][0].fragment_y_offset - 100.0).abs() < 0.01);
}

/// R1037：auto-height（非 explicit）子元素不拆分（CSS Fragmentation monolithic）。
/// 同 200px 子但 explicit_height=[false] → 整体留 col0，col1 空。
#[test]
fn test_assign_children_balanced_auto_height_no_break() {
    let children = vec![(0, 200.0)];
    let cols = assign_children_to_columns_balanced(&children, 2, &[false; 1], &[false; 1], &[false]);
    assert_eq!(cols.len(), 2);
    assert_eq!(cols[0].len(), 1); // 整体留 col0
    assert!((cols[0][0].visual_height - 200.0).abs() < 0.01);
    assert_eq!(cols[1].len(), 0); // col1 空，未拆分
}

/// 自动高度的 balance 多列没有固定列高，内容不得生成容器右侧的溢出列。
#[test]
fn test_auto_height_balanced_multicol_does_not_create_overflow_columns() {
    let mut doc = zero_dom::Document::new();
    let container_id = doc.create_element("div");
    let mut style = ComputedStyle::default();
    style.column_count = ColumnCountComputedValue::Number(3);
    style.column_gap = LengthValue::Px(10.0);
    let styles = HashMap::from([(container_id, style)]);
    let mut container = LayoutBox {
        node_id: Some(container_id),
        content_width: 320.0,
        content_height: 100.0,
        children: (0..5)
            .map(|_| LayoutBox {
                width: 100.0,
                content_width: 100.0,
                height: 100.0,
                content_height: 100.0,
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    };
    let info = compute_column_info(styles.get(&container_id).unwrap(), container.content_width).unwrap();

    layout_multicol(&mut container, &info, &styles);

    assert!(
        container
            .children
            .iter()
            .flat_map(|child| &child.column_span_offsets)
            .all(|(_, _, col_x, col_w, _, _)| { col_x + col_w <= container.content_width + 0.01 }),
        "height:auto balance must use only the configured columns"
    );
}

#[test]
fn test_assign_children_with_breaking() {
    // 4 children, each 100px high, 3 columns, 150px height limit
    // child0(100): col0=100 → col0=[0]
    // child1(100): col0=200 > 150, move to col1 → col1=[1]
    // child2(100): col1=200 > 150, move to col2 → col2=[2]
    // child3(100): col2=200 > 150, no more cols, stays → col2=[2,3]
    let children = vec![(0, 100.0), (1, 100.0), (2, 100.0), (3, 100.0)];
    let cols = assign_children_to_columns_with_breaking(&children, 3, 150.0, &[false; 4], &[false; 4]);
    assert_eq!(cols.len(), 3);
    assert_eq!(cols[0].len(), 1);
    assert_eq!(cols[1].len(), 1);
    assert_eq!(cols[2].len(), 2); // last 2 overflow into col2
}

#[test]
fn test_assign_children_with_breaking_oversized() {
    // Single child larger than column height — stays in current column
    let children = vec![(0, 300.0)];
    let cols = assign_children_to_columns_with_breaking(&children, 3, 100.0, &[false; 4], &[false; 4]);
    assert_eq!(cols.len(), 3);
    assert_eq!(cols[0].len(), 1); // oversized child stays in first column
}

#[test]
fn test_assign_children_with_breaking_single_col_oversized_no_panic() {
    // 回归：col_count=1（column-count:1 或计算为单列）+ column-fill:auto + 明确高度 +
    // oversized 子元素后跟另一个子元素。修复前 line 378 无守卫 current_col+=1 越界，
    // 使后续子元素 columns[current_col].push 在 line 350 panic（index OOB, len 1 idx 1）。
    // 修复后：单列时 oversized 内容 clip 到唯一列，后续子元素也落入唯一列，无 panic。
    let children = vec![(0, 300.0), (1, 50.0)];
    let cols = assign_children_to_columns_with_breaking(&children, 1, 100.0, &[false; 4], &[false; 4]);
    assert_eq!(cols.len(), 1);
    // 两子元素都分配到唯一列（clip），不应 panic
    assert!(cols[0].iter().any(|f| f.child_idx == 0));
    assert!(cols[0].iter().any(|f| f.child_idx == 1));
}

/// R1074：inline 方向列溢出定位。definite 高度 multicol 内容超 col_count×列高时，
/// 额外 column box 应在 inline 方向（向右）溢出，而非堆到下方行。`position_multicol_children`
/// 传 row_height=0.0 时，col_idx ≥ col_count 的溢出列 col_x 单调递增（col_idx×(col_w+gap)），
/// 落到容器右外侧；同 y_base 单行。对应 multicol-span-all-children-height-002（block2 4 列：
/// 2 in-article + 2 右溢出）匹配 chromium（z_vs_chr 3.99%→0.29%）。
#[test]
fn test_position_multicol_inline_overflow_row_height_zero() {
    use crate::types::LayoutBox;
    // 2 列 × 列宽 100 + gap 10；单个超高子元素被 multirow assign 拆成 4 片（4 列）。
    let info = ColumnInfo {
        count: 2,
        column_width: 100.0,
        gap: 10.0,
        sequential_fill: false,
    };
    let mut container = LayoutBox::default();
    container.content_width = 210.0;
    container.children = vec![LayoutBox::default()];
    // 4 列每列一片（child 0 的 4 个 50px 片段），模拟 multirow assign 输出。
    let assignments: Vec<Vec<ColumnFragment>> = (0..4)
        .map(|off| {
            vec![ColumnFragment {
                child_idx: 0,
                fragment_y_offset: off as f32 * 50.0,
                visual_height: 50.0,
            }]
        })
        .collect();

    let region_height = position_multicol_children(&mut container, &assignments, &info, 0.0, 0.0);
    // 行高 50（单行），未向下堆叠。
    assert!(
        (region_height - 50.0).abs() < 0.01,
        "single inline row, region_height=50"
    );

    let offsets = &container.children[0].column_span_offsets;
    assert_eq!(offsets.len(), 4, "4 fragments (one per column incl. 2 overflow)");
    // tuple = (child_x, child_y, col_x, col_w, col_top, col_h)；col_x = col_idx×(col_w+gap)。
    // 溢出列 col2/col3 落到 220/330（容器右外侧），非 wrap 回 0/110。
    let col_xs: Vec<f32> = offsets.iter().map(|t| t.2).collect();
    assert!((col_xs[0] - 0.0).abs() < 0.01, "col0 x=0");
    assert!((col_xs[1] - 110.0).abs() < 0.01, "col1 x=110");
    assert!(
        (col_xs[2] - 220.0).abs() < 0.01,
        "col2 (overflow) x=220 inline, not wrapped"
    );
    assert!((col_xs[3] - 330.0).abs() < 0.01, "col3 (overflow) x=330 inline");
    // 所有片段同 y_base（单行），col_top=0（非跨行下移）。
    for t in offsets {
        assert!((t.4 - 0.0).abs() < 0.01, "col_top=0 (inline, no row stacking)");
    }
}

/// R1035：multi-row 列模型。3 列 × 100px 高度，5 个 100px 子元素 → 总 500 > 3×100=300，
/// 溢出换行：col0=[0], col1=[1], col2=[2], col3(row1)=[3], col4(row1)=[4]。
#[test]
fn test_assign_children_multirow_basic() {
    let children = vec![(0, 100.0), (1, 100.0), (2, 100.0), (3, 100.0), (4, 100.0)];
    let cols = assign_children_to_columns_multirow(&children, 3, 100.0);
    assert_eq!(cols.len(), 5, "overflow creates 2 extra columns (row 1)");
    assert_eq!(cols[0].len(), 1);
    assert_eq!(cols[3][0].child_idx, 3); // row1 col0 = child 3
    assert_eq!(cols[4][0].child_idx, 4);
}

/// R1035：内容恰好填满（4×100=400 = 4×100 容量）不应触发 multi-row。
#[test]
fn test_assign_children_multirow_no_overflow_stays_single_row() {
    let children = vec![(0, 100.0), (1, 100.0), (2, 100.0), (3, 100.0)];
    let cols = assign_children_to_columns_multirow(&children, 4, 100.0);
    assert_eq!(cols.len(), 4, "exact fit = single row");
}

/// R1035：超高子元素跨多列 + 跨行 breaking。
/// 2 列 × 100px，单个 250px 子元素 → col0[0..100], col1[100..200], col2(row1)[200..250]。
#[test]
fn test_assign_children_multirow_oversized_breaks_across_rows() {
    let children = vec![(0, 250.0)];
    let cols = assign_children_to_columns_multirow(&children, 2, 100.0);
    assert_eq!(cols.len(), 3, "oversized child breaks across 2 cols + row1");
    assert_eq!(cols[0][0].fragment_y_offset, 0.0);
    assert_eq!(cols[0][0].visual_height, 100.0);
    assert_eq!(cols[1][0].fragment_y_offset, 100.0);
    assert!((cols[2][0].visual_height - 50.0).abs() < 0.01);
}

/// R903：`break-before:column` 强制换列——3 子元素各带 forced break，3 列 → 每列一个
///（首个子元素的 break 在空列上 no-op，故仍落 col0；后续两个推进到 col1/col2）。
/// 对应 multicol-break-001（A/B/C 各入独立列，chromium Oracle 1.22%→1.06% 改善）。
#[test]
fn test_break_before_column_forces_new_column_balanced() {
    let children = vec![(0, 100.0), (1, 100.0), (2, 100.0)];
    // 全部 forced break（模拟 `div > div { break-before: column }`）
    let cols = assign_children_to_columns_balanced(&children, 3, &[true, true, true], &[false; 3], &[]);
    assert_eq!(cols.len(), 3);
    // 首个子元素 break 在空 col0 上 no-op → col0=[0]；col1=[1]；col2=[2]。
    assert_eq!(cols[0].len(), 1);
    assert_eq!(cols[1].len(), 1);
    assert_eq!(cols[2].len(), 1);
    assert_eq!(cols[0][0].child_idx, 0);
    assert_eq!(cols[1][0].child_idx, 1);
    assert_eq!(cols[2][0].child_idx, 2);
}

/// R903：`break-before:column` 在 column-fill:auto + 明确高度路径也生效。
/// height:3em 容纳多子，但 forced break 使每个强制入新列。
#[test]
fn test_break_before_column_forces_new_column_breaking() {
    // max_col_height=200 容纳全部 3 子（各 50），但 forced break 强制换列。
    let children = vec![(0, 50.0), (1, 50.0), (2, 50.0)];
    let cols = assign_children_to_columns_with_breaking(&children, 3, 200.0, &[false, true, true], &[false; 3]);
    assert_eq!(cols.len(), 3);
    assert_eq!(cols[0].len(), 1); // child0 无 forced break → col0
    assert_eq!(cols[1].len(), 1); // child1 forced → col1
    assert_eq!(cols[2].len(), 1); // child2 forced → col2
}

/// R903：首个子元素的 forced break 在空列上 no-op（不创建前导空列）。
#[test]
fn test_break_before_column_first_child_is_noop() {
    let children = vec![(0, 100.0), (1, 100.0)];
    // 仅首个 forced break → no-op（col0 空，不创建前导空列），child0 仍落 col0。
    let cols = assign_children_to_columns_balanced(&children, 2, &[true, false], &[false; 2], &[]);
    assert_eq!(cols.len(), 2);
    assert!(
        cols[0].iter().any(|f| f.child_idx == 0),
        "first-child break-before must not create a leading empty column"
    );
    // child1 因 target_height（100>=100）推进 col1。
    assert!(cols[1].iter().any(|f| f.child_idx == 1));
}

/// R1027：`break-after:column` 强制换列——mirror R903 break-before，但作用于
/// 「放置完子元素后」。3 子各 100，3 列，target=100：child0 落 col0 后 break-after
/// 推进 col1；child1 落 col1 后推进 col2；child2 落 col2（末列，break-after no-op）。
/// 对应 multicol-break-000（`div > div { break-after: column }`，A/B/C 各入独立列）。
#[test]
fn test_break_after_column_forces_new_column_balanced() {
    let children = vec![(0, 100.0), (1, 100.0), (2, 100.0)];
    // 全部 break-after（模拟 `div > div { break-after: column }`）
    let cols = assign_children_to_columns_balanced(&children, 3, &[false; 3], &[true, true, true], &[]);
    assert_eq!(cols.len(), 3);
    assert_eq!(cols[0].len(), 1);
    assert_eq!(cols[1].len(), 1);
    assert_eq!(cols[2].len(), 1);
    assert_eq!(cols[0][0].child_idx, 0);
    assert_eq!(cols[1][0].child_idx, 1);
    assert_eq!(cols[2][0].child_idx, 2);
}

/// R1027：`break-after:column` 在 column-fill:auto + 明确高度路径也生效。
/// max_col_height=200 容纳全部 3 子（各 50），但 break-after 使每个强制入新列。
#[test]
fn test_break_after_column_forces_new_column_breaking() {
    let children = vec![(0, 50.0), (1, 50.0), (2, 50.0)];
    let cols = assign_children_to_columns_with_breaking(&children, 3, 200.0, &[false; 3], &[true, true, true]);
    assert_eq!(cols.len(), 3);
    assert_eq!(cols[0].len(), 1);
    assert_eq!(cols[1].len(), 1);
    assert_eq!(cols[2].len(), 1);
    assert_eq!(cols[0][0].child_idx, 0);
    assert_eq!(cols[1][0].child_idx, 1);
    assert_eq!(cols[2][0].child_idx, 2);
}

/// R1027：末子元素的 break-after 在末列上 no-op（`current_col + 1 < col_count` 守卫
/// 防止越界，不创建尾随空列）。
#[test]
fn test_break_after_column_last_child_in_last_col_is_noop() {
    let children = vec![(0, 100.0), (1, 100.0)];
    // 仅末子 break-after → child0 落 col0 后 break-after 推进 col1；child1 落 col1（末列），
    // 其 break-after 因 current_col+1 >= col_count no-op，不创建尾随空列。
    let cols = assign_children_to_columns_balanced(&children, 2, &[false; 2], &[false, true], &[]);
    assert_eq!(cols.len(), 2);
    assert!(cols[0].iter().any(|f| f.child_idx == 0));
    assert!(cols[1].iter().any(|f| f.child_idx == 1));
}

/// R1352：`has_descendant_spanner`（DFS 后代）vs `has_direct_spanner_child`（仅直接子）
/// 的关键语义差异——这是 R1352 修复 remove-transform-descendant 回归的 gate 核心。
///
/// - **direct-child spanner**（004a/004b 结构：wrapper > spanner）：两函数都 true，
///   `enable_painter_core` = true，painter 跑列循环 + cso 传播 → block 分布修对。
/// - **grandchild spanner**（remove-transform：wrapper > div > div(spanner)）：descendant
///   = true（仍触发 try_layout_nested_spanner 做 baseline x/y 回填），但 direct-child =
///   false → `enable_painter_core` = false → 不设 flag、不传 cso、painter 不跑列循环 →
///   保留 R1341 baseline 0.63%（避 painter 误处理 deep-nesting spanner 的回归）。
#[test]
fn test_r1352_direct_vs_descendant_spanner_gate() {
    use zero_dom::Document;
    let mut doc = Document::new();
    // 分配 distinct NodeId（slotmap 保证唯一）。
    let wrapper_id = doc.create_element("div");
    let child_id = doc.create_element("div");
    let grandchild_id = doc.create_element("div");
    let spanner_id = doc.create_element("div");

    // 场景 1：grandchild spanner（remove-transform 结构 wrapper > child > grandchild-spanner）。
    let mut style_spanner = ComputedStyle::default();
    style_spanner.column_span = ColumnSpanComputedValue::All;
    let styles_deep = HashMap::from([(grandchild_id, style_spanner)]);
    let mut child = LayoutBox {
        node_id: Some(child_id),
        ..Default::default()
    };
    child.children.push(LayoutBox {
        node_id: Some(grandchild_id),
        ..Default::default()
    });
    let mut wrapper = LayoutBox {
        node_id: Some(wrapper_id),
        ..Default::default()
    };
    wrapper.children.push(child);
    // descendant gate 命中（spanner 是后代）→ try_layout_nested_spanner 仍触发做 x/y 回填。
    assert!(
        has_descendant_spanner(&wrapper, &styles_deep),
        "descendant gate must fire for grandchild spanner (baseline x/y backfill)"
    );
    // direct-child gate 不命中（spanner 非直接子）→ enable_painter_core=false，避 painter 回归。
    assert!(
        !has_direct_spanner_child(&wrapper, &styles_deep),
        "direct-child gate must NOT fire for grandchild spanner (R1352 regression fix)"
    );

    // 场景 2：direct-child spanner（004a 结构 wrapper > spanner）。两 gate 都命中。
    let styles_direct = HashMap::from([(spanner_id, {
        let mut s = ComputedStyle::default();
        s.column_span = ColumnSpanComputedValue::All;
        s
    })]);
    let mut wrapper2 = LayoutBox {
        node_id: Some(wrapper_id),
        ..Default::default()
    };
    wrapper2.children.push(LayoutBox {
        node_id: Some(spanner_id),
        ..Default::default()
    });
    assert!(
        has_descendant_spanner(&wrapper2, &styles_direct),
        "descendant gate must fire for direct-child spanner"
    );
    assert!(
        has_direct_spanner_child(&wrapper2, &styles_direct),
        "direct-child gate must fire for direct-child spanner (004a/004b case)"
    );

    // 场景 3：无 spanner → 两 gate 都 false。
    let wrapper3 = LayoutBox::default();
    let empty_styles = HashMap::new();
    assert!(!has_descendant_spanner(&wrapper3, &empty_styles));
    assert!(!has_direct_spanner_child(&wrapper3, &empty_styles));
}

/// zero-column-width-layout：column-width:0 合法，used value 钳到 ≥1px。
/// 50px 容器 + column-gap:0 + column-width:0 → 50 列 × 1px（旧实现 `<= 0.0` 误判为无多列）。
#[test]
fn test_column_width_zero_clamps_used_to_one_px() {
    use zero_css_parser::values::LengthValue;
    // column-width:0 + column-gap:0（无 column-count）
    let mut style = ComputedStyle::default();
    style.column_width = ColumnWidthComputedValue::Length(LengthValue::Px(0.0));
    style.column_gap = LengthValue::Px(0.0);
    let info = compute_column_info(&style, 50.0).expect("column-width:0 must produce multicol");
    assert_eq!(info.count, 50, "50px / 1px-used = 50 columns");
    assert!(
        (info.column_width - 1.0).abs() < 0.01,
        "used column-width clamped to 1px"
    );

    // column-count:3 + column-width:0 + gap:0 → used hint 1px，count=min(3, floor(50/1))=3，
    // 最终列宽 = container/count = 50/3 ≈ 16.67（1px clamp 仅作用于 hint，非最终列宽）。
    let mut style2 = ComputedStyle::default();
    style2.column_count = ColumnCountComputedValue::Number(3);
    style2.column_width = ColumnWidthComputedValue::Length(LengthValue::Px(0.0));
    style2.column_gap = LengthValue::Px(0.0);
    let info2 = compute_column_info(&style2, 50.0).expect("count+width:0 must produce multicol");
    assert_eq!(info2.count, 3);
    assert!(
        (info2.column_width - 16.667).abs() < 0.1,
        "final column-width = container/count = 50/3"
    );

    // column-width:-100px（负值非法）→ 无多列（None）。
    let mut style_neg = ComputedStyle::default();
    style_neg.column_width = ColumnWidthComputedValue::Length(LengthValue::Px(-100.0));
    style_neg.column_gap = LengthValue::Px(0.0);
    assert!(
        compute_column_info(&style_neg, 50.0).is_none(),
        "negative column-width is invalid → no multicol"
    );
}
