//! Media query coverage improvement tests
//!
//! This file contains additional tests to improve test coverage for the media_query module.

use zero_css_parser::media_query::*;

#[test]
fn test_parse_conditions_from_inner_invalid() {
    // Test invalid feature names
    assert!(parse_conditions_from_inner("invalid-feature").is_empty());

    // Test malformed range syntax
    assert!(parse_conditions_from_inner("width <").is_empty());

    // Test invalid range syntax
    assert!(parse_conditions_from_inner("width < < 600px").is_empty());
}

#[test]
fn test_parse_colon_syntax_invalid() {
    // Test invalid orientation value
    assert_eq!(parse_colon_syntax("orientation: invalid", 11), None);

    // Test invalid prefers-color-scheme value
    assert_eq!(parse_colon_syntax("prefers-color-scheme: invalid", 23), None);

    // Test invalid prefers-reduced-motion value
    assert_eq!(parse_colon_syntax("prefers-reduced-motion: invalid", 24), None);

    // Test invalid pointer value
    assert_eq!(parse_colon_syntax("pointer: invalid", 8), None);

    // Test invalid resolution unit
    assert_eq!(parse_colon_syntax("resolution: invalid", 11), None);
}

#[test]
fn test_contains_range_op() {
    assert!(contains_range_op("width > 600px"));
    assert!(contains_range_op("width >= 600px"));
    assert!(contains_range_op("width < 1000px"));
    assert!(contains_range_op("width <= 1000px"));
    // CSS MQ L4 `=` 精确相等也算范围运算符（路由到范围语法分支）
    assert!(contains_range_op("width = 800px"));
    assert!(!contains_range_op("width: 600px"));
    assert!(!contains_range_op("orientation: portrait"));
}

#[test]
fn test_parse_range_syntax_vec_combined() {
    // Test combined range syntax
    let conds = parse_range_syntax_vec("600px <= width <= 1000px");
    assert_eq!(conds.len(), 2);
    assert_eq!(conds[0], MediaCondition::Width(MediaFeatureOp::GreaterEqual, 600.0));
    assert_eq!(conds[1], MediaCondition::Width(MediaFeatureOp::LessEqual, 1000.0));
}

#[test]
fn test_parse_range_syntax_vec_simple() {
    // Test simple range syntax
    let conds = parse_range_syntax_vec("width > 600px");
    assert_eq!(conds.len(), 1);
    assert_eq!(conds[0], MediaCondition::Width(MediaFeatureOp::GreaterThan, 600.0));
}

#[test]
fn test_parse_range_syntax_vec_invalid() {
    // Test invalid range syntax
    assert!(parse_range_syntax_vec("invalid syntax").is_empty());
    assert!(parse_range_syntax_vec("width <").is_empty());
}

#[test]
fn test_try_parse_combined_range_invalid() {
    // Test invalid combined ranges
    assert_eq!(try_parse_combined_range("invalid"), None);
    assert_eq!(try_parse_combined_range("600px <"), None);
    assert_eq!(try_parse_combined_range("600px <= width"), None);
    assert_eq!(try_parse_combined_range("600px <= width <"), None);
}

#[test]
fn test_flip_op() {
    assert_eq!(flip_op(MediaFeatureOp::LessThan), MediaFeatureOp::GreaterThan);
    assert_eq!(flip_op(MediaFeatureOp::LessEqual), MediaFeatureOp::GreaterEqual);
    assert_eq!(flip_op(MediaFeatureOp::GreaterThan), MediaFeatureOp::LessThan);
    assert_eq!(flip_op(MediaFeatureOp::GreaterEqual), MediaFeatureOp::LessEqual);
    assert_eq!(flip_op(MediaFeatureOp::Exact), MediaFeatureOp::Exact);
}

#[test]
fn test_parse_leading_value() {
    assert_eq!(parse_leading_value("600px"), Some((600.0, "")));
    assert_eq!(parse_leading_value("600.5px"), Some((600.5, "")));
    assert_eq!(parse_leading_value("600 px"), Some((600.0, "px")));
    assert_eq!(parse_leading_value("600"), Some((600.0, "")));
    assert_eq!(parse_leading_value("abc"), None);
}

#[test]
fn test_parse_op() {
    assert_eq!(parse_op(">="), Some((MediaFeatureOp::GreaterEqual, "")));
    assert_eq!(parse_op("<="), Some((MediaFeatureOp::LessEqual, "")));
    assert_eq!(parse_op(">"), Some((MediaFeatureOp::GreaterThan, "")));
    assert_eq!(parse_op("<"), Some((MediaFeatureOp::LessThan, "")));
    // CSS MQ L4 §7.1：`=` 精确相等（≡ 冒号形式 Exact）
    assert_eq!(parse_op("="), Some((MediaFeatureOp::Exact, "")));
    assert_eq!(parse_op("invalid"), None);
}

#[test]
fn test_parse_feature_name() {
    // Test feature name extraction
    assert_eq!(parse_feature_name("width"), Some(("width", "")));
    assert_eq!(parse_feature_name("height"), Some(("height", "")));
    assert_eq!(parse_feature_name("width abc"), Some(("width", "abc")));
    assert_eq!(parse_feature_name("invalid-feature"), None);
    assert_eq!(parse_feature_name(""), None);
}

#[test]
fn test_make_feature_condition() {
    assert_eq!(
        make_feature_condition("width", MediaFeatureOp::GreaterThan, 600.0),
        Some(MediaCondition::Width(MediaFeatureOp::GreaterThan, 600.0))
    );
    assert_eq!(
        make_feature_condition("height", MediaFeatureOp::LessThan, 400.0),
        Some(MediaCondition::Height(MediaFeatureOp::LessThan, 400.0))
    );
    assert_eq!(make_feature_condition("invalid", MediaFeatureOp::Exact, 100.0), None);
}

#[test]
fn test_parse_simple_range_invalid() {
    // Test invalid simple range syntax
    assert_eq!(parse_simple_range("invalid"), None);
    assert_eq!(parse_simple_range("width <"), None);
    assert_eq!(parse_simple_range("width < invalid"), None);
}

#[test]
fn test_find_range_op_pos() {
    // Test finding range operator position
    assert_eq!(find_range_op_pos("width > 600px"), Some(6));
    assert_eq!(find_range_op_pos("width >= 600px"), Some(6));
    assert_eq!(find_range_op_pos("width < 1000px"), Some(6));
    assert_eq!(find_range_op_pos("width <= 1000px"), Some(6));
    // `=` 命中位置；`>=`/`<=` 中 `<`/`>` 优先命中（见下方断言）
    assert_eq!(find_range_op_pos("width = 800px"), Some(6));
    assert_eq!(find_range_op_pos("width: 600px"), None);
}

#[test]
fn test_parse_px_value() {
    assert_eq!(parse_px_value("600px"), Some(600.0));
    assert_eq!(parse_px_value("600"), Some(600.0));
    assert_eq!(parse_px_value("50.5px"), Some(50.5));
    assert_eq!(parse_px_value("0"), Some(0.0));
    assert_eq!(parse_px_value("invalid"), None);
}

#[test]
fn test_parse_dpi_value() {
    assert_eq!(parse_dpi_value("96dpi"), Some(96.0));
    assert_eq!(parse_dpi_value("150dpi"), Some(150.0));
    assert_eq!(parse_dpi_value("96"), Some(96.0));
    assert_eq!(parse_dpi_value("0dpi"), Some(0.0));
    assert_eq!(parse_dpi_value("invalid"), None);
}

#[test]
fn test_find_matching_paren() {
    // Test finding matching parentheses
    assert_eq!(find_matching_paren("(test)"), Some(5));
    assert_eq!(find_matching_paren("(test) and (more)"), Some(5));
    assert_eq!(find_matching_paren("(nested (parentheses))"), Some(21));
    assert_eq!(find_matching_paren("no paren"), None);
    assert_eq!(find_matching_paren("(unmatched"), None);
}

#[test]
fn test_parse_media_query_error_cases() {
    // Test various error cases
    assert!(parse_media_query("").is_none());
    // Note: "only" is treated as a media type and creates a valid query
    let q = parse_media_query("only").unwrap();
    assert_eq!(q.len(), 1);
    assert_eq!(q[0].media_type, Some(MediaType::All)); // "only" without type defaults to "all"
    assert!(parse_media_query("invalid query").is_none());
    assert!(parse_media_query("(unclosed").is_none());
    assert!(parse_media_query("(invalid: value)").is_none());
}

#[test]
fn test_parse_single_media_query_edge_cases() {
    // Test edge cases for single media query
    let q = parse_single_media_query("all").unwrap();
    assert_eq!(q.media_type, Some(MediaType::All));

    let q = parse_single_media_query("not print").unwrap();
    assert!(q.negated);
    assert_eq!(q.media_type, Some(MediaType::Print));

    let q = parse_single_media_query("only screen and (min-width: 600px)").unwrap();
    assert_eq!(q.media_type, Some(MediaType::Screen));
    assert_eq!(q.conditions.len(), 1);
}

#[test]
fn test_evaluate_error_conditions() {
    // Test evaluation of error conditions
    let invalid_q = MediaQuery {
        media_type: None,
        negated: false,
        conditions: vec![],
    };
    let ctx = MediaContext::new(800.0, 600.0);
    assert!(evaluate_media_query(&invalid_q, &ctx));
}

#[test]
fn test_parse_media_query_complex_nested() {
    // Test complex nested media queries
    let queries = parse_media_query("screen, print and (min-width: 600px), (orientation: landscape)");
    assert!(queries.is_some());
    assert_eq!(queries.unwrap().len(), 3);
}

#[test]
fn test_parse_invalid_range_combinations() {
    // Test invalid range combinations
    assert!(parse_range_syntax_vec("600px < width > 1000px").is_empty());
    assert!(parse_range_syntax_vec("width < 600px < 1000px").is_empty());
    assert!(parse_range_syntax_vec("600px < < width").is_empty());
}

#[test]
fn test_parse_resolution_with_different_units() {
    // Test resolution with different units
    let q = parse_media_query("(resolution: 150dppx)").unwrap();
    // Note: current implementation only supports dpi, not dppx
    assert_eq!(q.conditions.len(), 0);

    let q = parse_media_query("(min-resolution: 300)").unwrap();
    assert_eq!(q.conditions.len(), 0);
}

#[test]
fn test_parse_boolean_feature_with_invalid_syntax() {
    // Test boolean feature with invalid syntax
    let q = parse_media_query("(prefers-color-scheme:)").unwrap();
    assert_eq!(q.conditions.len(), 0);

    let q = parse_media_query("(prefers-reduced-motion: )").unwrap();
    assert_eq!(q.conditions.len(), 0);
}

#[test]
fn test_parse_pointer_with_boolean_syntax() {
    // Test pointer boolean syntax
    let q = parse_media_query("(pointer)").unwrap();
    assert_eq!(q.conditions.len(), 1);
    assert_eq!(q.conditions[0], MediaCondition::Pointer(PointerValue::Coarse));

    // Test boolean syntax only returns true for coarse and fine
    let mut ctx_none = MediaContext::new(1024.0, 768.0);
    ctx_none.pointer_type = PointerValue::None;
    assert!(!evaluate_media_query(&q, &ctx_none));

    let mut ctx_coarse = MediaContext::new(1024.0, 768.0);
    ctx_coarse.pointer_type = PointerValue::Coarse;
    assert!(evaluate_media_query(&q, &ctx_coarse));

    let mut ctx_fine = MediaContext::new(1024.0, 768.0);
    ctx_fine.pointer_type = PointerValue::Fine;
    assert!(!evaluate_media_query(&q, &ctx_fine));
}

#[test]
fn test_parse_all_type_with_conditions() {
    // Test all type with conditions
    let q = parse_media_query("all and (min-width: 600px)").unwrap();
    assert_eq!(q.media_type, Some(MediaType::All));
    assert_eq!(q.conditions.len(), 1);

    // all type should always match as long as conditions are met
    let ctx = MediaContext::with_type(800.0, 600.0, MediaType::Screen);
    assert!(evaluate_media_query(&q, &ctx));

    let ctx_small = MediaContext::with_type(400.0, 300.0, MediaType::Screen);
    assert!(!evaluate_media_query(&q, &ctx_small));
}

#[test]
fn test_parse_not_with_conditions() {
    // Test not prefix with conditions
    let q = parse_media_query("not screen and (min-width: 600px)").unwrap();
    assert!(q.negated);
    assert_eq!(q.media_type, Some(MediaType::Screen));
    assert_eq!(q.conditions.len(), 1);

    // This should be interpreted as not (screen and (min-width: 600px))
    let ctx_screen = MediaContext::with_type(800.0, 600.0, MediaType::Screen);
    assert!(!evaluate_media_query(&q, &ctx_screen));

    let ctx_print = MediaContext::with_type(800.0, 600.0, MediaType::Print);
    assert!(evaluate_media_query(&q, &ctx_print));
}

#[test]
fn test_parse_multiple_not_conditions() {
    // Test multiple not conditions
    let q = parse_media_query("not not screen").unwrap();
    assert!(!q.negated); // Double negative equals positive
    assert_eq!(q.media_type, Some(MediaType::Screen));

    assert!(evaluate_media_query(&q, &MediaContext::with_type(800.0, 600.0, MediaType::Screen)));
    assert!(!evaluate_media_query(&q, &MediaContext::with_type(800.0, 600.0, MediaType::Print)));
}

#[test]
fn test_parse_mixed_case_keywords() {
    // Test mixed case keywords
    let q = parse_media_query("ScReEn").unwrap();
    assert_eq!(q.media_type, Some(MediaType::Screen));

    let q = parse_media_query("PrInT").unwrap();
    assert_eq!(q.media_type, Some(MediaType::Print));

    let q = parse_media_query("NoT sCrEeN").unwrap();
    assert!(q.negated);
    assert_eq!(q.media_type, Some(MediaType::Screen));

    let q = parse_media_query("OnLy ScReEn").unwrap();
    assert_eq!(q.media_type, Some(MediaType::Screen));
}

#[test]
fn test_parse_whitespace_variations() {
    // Test different whitespace forms
    let q = parse_media_query("  screen  ").unwrap();
    assert_eq!(q.media_type, Some(MediaType::Screen));

    let q = parse_media_query("screen   and   (min-width: 600px)").unwrap();
    assert_eq!(q.media_type, Some(MediaType::Screen));
    assert_eq!(q.conditions.len(), 1);

    let q = parse_media_query("(  min-width : 600px  )").unwrap();
    assert_eq!(q.conditions.len(), 1);
    assert_eq!(q.conditions[0], MediaCondition::MinWidth(600.0));
}

#[test]
fn test_evaluate_all_condition() {
    // Test evaluation with all media type
    let q = parse_media_query("all").unwrap();
    assert_eq!(q.media_type, Some(MediaType::All));

    // all should match any context
    let ctx_screen = MediaContext::with_type(800.0, 600.0, MediaType::Screen);
    let ctx_print = MediaContext::with_type(800.0, 600.0, MediaType::Print);
    assert!(evaluate_media_query(&q, &ctx_screen));
    assert!(evaluate_media_query(&q, &ctx_print));
}

#[test]
fn test_split_media_queries_edge_cases() {
    // Test edge cases for query splitting
    let parts = split_media_queries("screen");
    assert_eq!(parts, vec!["screen"]);

    let parts = split_media_queries("screen, print");
    assert_eq!(parts, vec!["screen", "print"]);

    let parts = split_media_queries("(min-width: 600px), (max-width: 1024px)");
    assert_eq!(parts, vec!["(min-width: 600px)", "(max-width: 1024px)"]);

    let parts = split_media_queries("screen and (min-width: 600px), print");
    assert_eq!(parts, vec!["screen and (min-width: 600px)", "print"]);
}

#[test]
fn test_parse_conditions_from_inner_boolean() {
    // Test boolean features
    let conds = parse_conditions_from_inner("hover");
    assert_eq!(conds.len(), 1);
    assert_eq!(conds[0], MediaCondition::Hover);

    let conds = parse_conditions_from_inner("color");
    assert_eq!(conds.len(), 1);
    assert_eq!(conds[0], MediaCondition::Color);

    let conds = parse_conditions_from_inner("prefers-reduced-motion");
    assert_eq!(conds.len(), 1);
    assert_eq!(conds[0], MediaCondition::PrefersReducedMotion(ReducedMotionValue::Reduce));

    let conds = parse_conditions_from_inner("pointer");
    assert_eq!(conds.len(), 1);
    assert_eq!(conds[0], MediaCondition::Pointer(PointerValue::Coarse));

    // Test unknown boolean feature
    assert!(parse_conditions_from_inner("unknown").is_empty());
}

#[test]
fn test_parse_colon_syntax_resolution() {
    // Test resolution with colon syntax
    let cond = parse_colon_syntax("resolution: 96dpi", 11).unwrap();
    assert_eq!(cond, MediaCondition::Resolution(MediaFeatureOp::Exact, 96.0));

    let cond = parse_colon_syntax("min-resolution: 150dpi", 16).unwrap();
    assert_eq!(cond, MediaCondition::Resolution(MediaFeatureOp::GreaterEqual, 150.0));

    let cond = parse_colon_syntax("max-resolution: 300dpi", 16).unwrap();
    assert_eq!(cond, MediaCondition::Resolution(MediaFeatureOp::LessEqual, 300.0));
}

#[test]
fn test_parse_colon_syntax_dimension_values() {
    // Test dimension values with colon syntax
    let cond = parse_colon_syntax("width: 600px", 7).unwrap();
    assert_eq!(cond, MediaCondition::Width(MediaFeatureOp::Exact, 600.0));

    let cond = parse_colon_syntax("min-width: 600px", 12).unwrap();
    assert_eq!(cond, MediaCondition::MinWidth(600.0));

    let cond = parse_colon_syntax("max-width: 1024px", 12).unwrap();
    assert_eq!(cond, MediaCondition::MaxWidth(1024.0));

    let cond = parse_colon_syntax("height: 400px", 8).unwrap();
    assert_eq!(cond, MediaCondition::Height(MediaFeatureOp::Exact, 400.0));

    let cond = parse_colon_syntax("min-height: 300px", 13).unwrap();
    assert_eq!(cond, MediaCondition::MinHeight(300.0));

    let cond = parse_colon_syntax("max-height: 800px", 13).unwrap();
    assert_eq!(cond, MediaCondition::MaxHeight(800.0));
}

#[test]
fn test_evaluate_exact_dimensions() {
    // Test exact dimension evaluation
    let q = parse_media_query("(width: 600px)").unwrap();
    assert_eq!(q.conditions.len(), 1);

    // Test exact equality with tolerance
    let ctx_exact = MediaContext::new(600.0, 400.0);
    assert!(evaluate_media_query(&q, &ctx_exact));

    // Test values slightly outside tolerance
    let ctx_off = MediaContext::new(600.01, 400.0);
    assert!(!evaluate_media_query(&q, &ctx_off));

    let ctx_off = MediaContext::new(599.99, 400.0);
    assert!(!evaluate_media_query(&q, &ctx_off));
}

#[test]
fn test_media_query_with_empty_conditions() {
    // Test media query with empty conditions
    let q = MediaQuery {
        media_type: Some(MediaType::Screen),
        negated: false,
        conditions: vec![],
    };
    let ctx = MediaContext::new(800.0, 600.0);
    assert!(evaluate_media_query(&q, &ctx));

    let q = MediaQuery {
        media_type: Some(MediaType::Print),
        negated: true,
        conditions: vec![],
    };
    let ctx_screen = MediaContext::with_type(800.0, 600.0, MediaType::Screen);
    assert!(!evaluate_media_query(&q, &ctx_screen));
}

#[test]
fn test_parse_single_media_query_empty_remaining() {
    // Test parsing with empty remaining string after media type
    let q = parse_single_media_query("screen").unwrap();
    assert_eq!(q.media_type, Some(MediaType::Screen));
    assert!(q.conditions.is_empty());

    let q = parse_single_media_query("print").unwrap();
    assert_eq!(q.media_type, Some(MediaType::Print));
    assert!(q.conditions.is_empty());

    let q = parse_single_media_query("all").unwrap();
    assert_eq!(q.media_type, Some(MediaType::All));
    assert!(q.conditions.is_empty());
}