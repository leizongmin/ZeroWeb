//! CSS float 排除区域 + tab-size 行内布局测试。
//!
//! 从 `advanced.rs` 拆分以控制单文件体积（2000 行规则）。这两个主题同属
//! 行内排版（IFC）的浮动排除 / 制表符扩展聚类。复用 `advanced.rs` 相同的
//! `super::super::*` 与 `VA` 导入。

use super::super::*;
use zero_css_parser::values::VerticalAlignValue as VA;

// ── CSS float 排除区域测试 ──────────────────────────────────────────

/// 测试无浮动排除区域时文本布局不受影响。
#[test]
fn test_float_exclusion_none() {
    let mut ctx = InlineFormattingContext::new(800.0);
    let runs = vec![TextRun::simple(
        "Hello World".to_string(),
        NodeId::default(),
        16.0,
        20.0,
        VA::Baseline,
    )];
    ctx.break_into_lines(runs);

    assert_eq!(ctx.lines.len(), 1);
    // 无排除区域时，首片段 x 应为 0.0
    assert!(
        ctx.lines[0].runs[0].x.abs() < 0.01,
        "无浮动排除时首片段 x 应为 0，实际 {}",
        ctx.lines[0].runs[0].x
    );
}

/// 测试左浮动排除区域 — 文本向右偏移。
#[test]
fn test_float_exclusion_left_float() {
    let mut ctx = InlineFormattingContext::new(200.0).with_float_exclusions(vec![FloatExclusion {
        y: 0.0,
        height: 100.0,
        width: 80.0,
        is_left: true,
    }]);
    let runs = vec![TextRun::simple(
        "Hello World".to_string(),
        NodeId::default(),
        16.0,
        20.0,
        VA::Baseline,
    )];
    ctx.break_into_lines(runs);

    assert!(!ctx.lines.is_empty());
    // 首片段 x 应该被左浮动偏移到 80.0
    let first_x = ctx.lines[0].runs[0].x;
    assert!(
        (first_x - 80.0).abs() < 0.01,
        "左浮动排除时首片段 x 应为 80.0，实际 {}",
        first_x
    );
}

/// 测试右浮动排除区域 — 文本可用宽度减小。
#[test]
fn test_float_exclusion_right_float() {
    // 容器 200px，右浮动占 80px，可用宽度 120px
    let mut ctx = InlineFormattingContext::new(200.0).with_float_exclusions(vec![FloatExclusion {
        y: 0.0,
        height: 100.0,
        width: 80.0,
        is_left: false,
    }]);
    // 使用一个会换行的长文本
    let runs = vec![TextRun::simple(
        "AAAAAA BBBBBB CCCCCC DDDDDD EEEEEE FFFFFF".to_string(),
        NodeId::default(),
        16.0,
        20.0,
        VA::Baseline,
    )];
    ctx.break_into_lines(runs);

    assert!(!ctx.lines.is_empty());
    // 首片段 x 应该从 0.0 开始（右浮动不偏移左侧）
    let first_x = ctx.lines[0].runs[0].x;
    assert!(first_x.abs() < 0.01, "右浮动排除时首片段 x 应为 0，实际 {}", first_x);
}

/// 测试左右浮动同时存在 — 文本排列在中间缝隙中。
#[test]
fn test_float_exclusion_both_sides() {
    // 容器 200px，左浮动 60px，右浮动 60px，可用 80px
    let mut ctx = InlineFormattingContext::new(200.0).with_float_exclusions(vec![
        FloatExclusion {
            y: 0.0,
            height: 100.0,
            width: 60.0,
            is_left: true,
        },
        FloatExclusion {
            y: 0.0,
            height: 100.0,
            width: 60.0,
            is_left: false,
        },
    ]);
    let runs = vec![TextRun::simple(
        "AAAAAA BBBBBB CCCCCC DDDDDD EEEEEE FFFFFF".to_string(),
        NodeId::default(),
        16.0,
        20.0,
        VA::Baseline,
    )];
    ctx.break_into_lines(runs);

    assert!(!ctx.lines.is_empty());
    // 首片段应被左浮动偏移到 60.0
    let first_x = ctx.lines[0].runs[0].x;
    assert!(
        (first_x - 60.0).abs() < 0.01,
        "左右浮动同时存在时首片段 x 应为 60.0，实际 {}",
        first_x
    );
}

/// 测试浮动排除区域仅影响 y 范围重叠的行。
#[test]
fn test_float_exclusion_only_affects_overlapping_lines() {
    // 浮动区域在 y=0 到 y=20，高度 20px
    let mut ctx = InlineFormattingContext::new(200.0).with_float_exclusions(vec![FloatExclusion {
        y: 0.0,
        height: 20.0,
        width: 80.0,
        is_left: true,
    }]);
    // 使用足够多的文本产生多行（每行约 20px 高）
    let runs = vec![TextRun::simple(
        "AAAAAA BBBBBB CCCCCC DDDDDD EEEEEE FFFFFF GGGGGG HHHHHH IIIIII".to_string(),
        NodeId::default(),
        16.0,
        20.0,
        VA::Baseline,
    )];
    ctx.break_into_lines(runs);

    assert!(ctx.lines.len() >= 2, "应至少产生 2 行");

    // 第一行在浮动区域 y 范围内，应被偏移
    let first_x = ctx.lines[0].runs[0].x;
    assert!(
        (first_x - 80.0).abs() < 0.01,
        "首行在浮动区域内，x 应为 80.0，实际 {}",
        first_x
    );

    // 第二行应不在浮动区域 y 范围内（y=20 超出浮动区域 y=0..20），
    // 或如果与浮动区域相邻，可能仍有偏移
    let second_line_y = ctx.lines[1].y;
    if second_line_y >= 20.0 {
        let second_x = ctx.lines[1].runs[0].x;
        assert!(
            second_x.abs() < 0.01,
            "第二行在浮动区域外，x 应为 0，实际 {}（y={})",
            second_x,
            second_line_y
        );
    }
}

/// 测试 effective_content_area 辅助函数。
#[test]
fn test_effective_content_area() {
    let ctx = InlineFormattingContext::new(300.0).with_float_exclusions(vec![
        FloatExclusion {
            y: 0.0,
            height: 50.0,
            width: 100.0,
            is_left: true,
        },
        FloatExclusion {
            y: 0.0,
            height: 50.0,
            width: 80.0,
            is_left: false,
        },
    ]);

    // y 范围重叠
    let (left, avail) = ctx.effective_content_area(10.0, 20.0);
    assert!((left - 100.0).abs() < 0.01, "左偏移应为 100，实际 {}", left);
    assert!((avail - 120.0).abs() < 0.01, "可用宽度应为 120，实际 {}", avail);

    // y 范围不重叠
    let (left2, avail2) = ctx.effective_content_area(60.0, 20.0);
    assert!(left2.abs() < 0.01, "不重叠时左偏移应为 0，实际 {}", left2);
    assert!(
        (avail2 - 300.0).abs() < 0.01,
        "不重叠时可用宽度应为 300，实际 {}",
        avail2
    );
}

// ── CSS tab-size 行内布局测试 ──────────────────────────────────────────

/// 测试默认 tab-size（8 个空格宽度）。
#[test]
fn test_tab_size_default() {
    let ctx = InlineFormattingContext::new(800.0);
    assert!(
        (ctx.tab_size - 8.0).abs() < 0.01,
        "默认 tab-size 应为 8.0，实际 {}",
        ctx.tab_size
    );
}

/// 测试 preserve_whitespace 模式下制表符展开为空格。
#[test]
fn test_tab_expansion_in_preserve_mode() {
    let mut ctx = InlineFormattingContext::new(800.0)
        .with_preserve_whitespace(true)
        .with_tab_size(4.0);
    let runs = vec![TextRun::simple(
        "A\tB".to_string(),
        NodeId::default(),
        16.0,
        20.0,
        VA::Baseline,
    )];
    ctx.break_into_lines(runs);

    assert!(!ctx.lines.is_empty());
    // 制表符应展开为 4 个空格，作为独立片段
    // 总行应有 3 个片段：A + 4个空格 + B
    assert!(
        ctx.lines[0].runs.len() >= 2,
        "制表符展开后应至少 2 个片段，实际 {}",
        ctx.lines[0].runs.len()
    );
}

/// 测试自定义 tab-size 值影响制表符展开宽度。
#[test]
fn test_tab_size_custom_width() {
    let mut ctx = InlineFormattingContext::new(800.0)
        .with_preserve_whitespace(true)
        .with_tab_size(2.0);
    let runs = vec![TextRun::simple(
        "A\tB".to_string(),
        NodeId::default(),
        16.0,
        20.0,
        VA::Baseline,
    )];
    ctx.break_into_lines(runs);

    assert!(!ctx.lines.is_empty());
    // 查找空格片段（assert that tab expanded to 2 spaces, not 8）
    let space_fragments: Vec<_> = ctx.lines[0].runs.iter().filter(|r| r.text.trim().is_empty()).collect();
    assert!(!space_fragments.is_empty(), "应有空格片段");

    // 2 个空格 * font_size * 0.25 + letter_spacing(0) ≈ 2 * 4 = 8px 宽
    let space_width = space_fragments[0].width;
    assert!(
        space_width < 20.0,
        "tab-size=2 时空格片段宽度应较小（<20px），实际 {}",
        space_width
    );
}

/// 测试非 preserve_whitespace 模式下制表符被折叠为普通空白。
#[test]
fn test_tab_collapsed_in_normal_mode() {
    let mut ctx = InlineFormattingContext::new(800.0)
        .with_preserve_whitespace(false) // 默认模式
        .with_tab_size(4.0);
    let runs = vec![TextRun::simple(
        "A\tB".to_string(),
        NodeId::default(),
        16.0,
        20.0,
        VA::Baseline,
    )];
    ctx.break_into_lines(runs);

    assert!(!ctx.lines.is_empty());
    // 在非 preserve 模式下，split_whitespace 将制表符视为普通空白
    // "A\tB" 应被视为两个单词 "A" 和 "B"，各带尾部空格
    assert!(
        ctx.lines[0].runs.len() == 2,
        "非 preserve 模式下 'A\\tB' 应为 2 个单词片段，实际 {}",
        ctx.lines[0].runs.len()
    );
}

/// 测试多个连续制表符展开。
#[test]
fn test_multiple_tabs_expansion() {
    let mut ctx = InlineFormattingContext::new(800.0)
        .with_preserve_whitespace(true)
        .with_tab_size(4.0);
    let runs = vec![TextRun::simple(
        "A\t\tB".to_string(),
        NodeId::default(),
        16.0,
        20.0,
        VA::Baseline,
    )];
    ctx.break_into_lines(runs);

    assert!(!ctx.lines.is_empty());
    // 两个制表符各展开为 4 个空格
    // 应该有：A + spaces(4) + spaces(4) + B = 4 片段
    assert!(
        ctx.lines[0].runs.len() >= 3,
        "两个制表符展开后应至少 3 个片段，实际 {}",
        ctx.lines[0].runs.len()
    );
}

/// 测试 tab-size = 0 时制表符仍产生至少一个空格。
#[test]
fn test_tab_size_zero_fallback() {
    let mut ctx = InlineFormattingContext::new(800.0)
        .with_preserve_whitespace(true)
        .with_tab_size(0.0);
    let runs = vec![TextRun::simple(
        "A\tB".to_string(),
        NodeId::default(),
        16.0,
        20.0,
        VA::Baseline,
    )];
    ctx.break_into_lines(runs);

    // tab-size=0 时 max(1) 确保至少 1 个空格
    assert!(!ctx.lines.is_empty(), "tab-size=0 时不应崩溃");
    assert!(
        ctx.lines[0].runs.len() >= 2,
        "tab-size=0 时制表符应至少展开为 1 个空格，实际 {} 个片段",
        ctx.lines[0].runs.len()
    );
}

/// R1447：pre-wrap 制表符按 tab stop 推进（CSS Text 3 §4.1.3），非固定 tab_size 空格。
///
/// 驱动案 css-text/pre-wrap-tab-* + text-align-justify-tabs-*。旧实现把 `\t` 展开为
/// `tab_size` 个空格（固定宽度），仅在光标恰处 tab stop 边界时正确；否则过度推进
///（如 "abc\tx" 在 60px 处，旧实现 tab=8 空格=160px → x @ 220，应 @ 160）。
/// 修复：`\t` 推进到下一个 `tab_size` 倍数（相对内容盒起点）。本测试 load-bearing。
#[test]
fn r1447_tab_advances_to_next_tab_stop() {
    // Ahem 20px：每字符（含空格）= 20px。tab-size:8 → tab stop 每 160px（0/160/320…）。
    // "abc\tx"：abc=60px，tab 推进到下一个 stop 160（advance 100），x 落在 160。
    let mut ctx = InlineFormattingContext::new(800.0)
        .with_preserve_whitespace(true)
        .with_tab_size(8.0);
    let mut run = TextRun::simple("abc\tx".to_string(), NodeId::default(), 20.0, 20.0, VA::Baseline);
    run.is_ahem_font = true;
    ctx.break_into_lines(vec![run]);

    assert!(!ctx.lines.is_empty(), "应产出至少一行");
    let x_frag = ctx.lines[0].runs.iter().find(|r| r.text.contains('x'));
    let x_pos = x_frag.expect("应含 'x' 片段").x;
    assert!(
        (x_pos - 160.0).abs() < 1.0,
        "R1447: pre-wrap tab 应推进到 tab stop（tab-size:8 × 20px = 160），'x' 应在 x≈160，\
         实际 {:.1}（修复前 bug：tab 展开为固定 8 空格 → 'x' @ 220）",
        x_pos
    );
}

/// R1447：光标恰在 tab stop 边界时，tab 推进整整一个 tab_unit（不原地不动）。
#[test]
fn r1447_tab_at_stop_boundary_advances_full_unit() {
    // "aaaaaaaa\t"：8 个 a（Ahem 20px）= 160px = 恰好一个 tab stop。tab 应推进到 320（+160），
    // 而非停留在 160。验证 floor(160/160)+1 = 2 → next_stop = 320。
    let mut ctx = InlineFormattingContext::new(800.0)
        .with_preserve_whitespace(true)
        .with_tab_size(8.0);
    let mut run = TextRun::simple("aaaaaaaa\tx".to_string(), NodeId::default(), 20.0, 20.0, VA::Baseline);
    run.is_ahem_font = true;
    ctx.break_into_lines(vec![run]);

    let x_pos = ctx.lines[0]
        .runs
        .iter()
        .find(|r| r.text.contains('x'))
        .map(|r| r.x)
        .expect("应含 'x' 片段");
    assert!(
        (x_pos - 320.0).abs() < 1.0,
        "R1447: 光标在 tab stop 边界（160）时，tab 应推进整整一个 unit 到 320，实际 {:.1}",
        x_pos
    );
}
