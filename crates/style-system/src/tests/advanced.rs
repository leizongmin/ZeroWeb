// Auto-generated test file — split from style-system/lib.rs
use super::super::*;
use super::helpers::*;

// ── @layer 端到端测试 ──

#[test]
fn test_layer_unlayered_beats_layered() {
    // 未分层的 div { color: red; } 应该胜过 @layer base { div { color: blue; } }
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![
            Rule::Layer(zero_css_parser::ast::LayerRule {
                name: "base".to_string(),
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "blue".to_string(),
                        important: false,
                    }],
                })],
            }),
            Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            }),
        ],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    // 未分层胜过分层
    assert_eq!(div_style.color, ColorValue::Rgba(255, 0, 0, 255)); // red
}

#[test]
fn test_layer_later_beats_earlier() {
    // @layer base { div { color: red; } } @layer theme { div { color: green; } }
    // 后面的层胜过前面的
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![
            Rule::Layer(zero_css_parser::ast::LayerRule {
                name: "base".to_string(),
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    }],
                })],
            }),
            Rule::Layer(zero_css_parser::ast::LayerRule {
                name: "theme".to_string(),
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "green".to_string(),
                        important: false,
                    }],
                })],
            }),
        ],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    // 后面的层（theme=green）胜过前面的层（base=red）
    assert_eq!(div_style.color, ColorValue::Rgba(0, 128, 0, 255)); // green
}

#[test]
fn test_layer_specificity_within_same_layer() {
    // 同层内，高特异性仍然胜出
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Layer(zero_css_parser::ast::LayerRule {
            name: "base".to_string(),
            rules: vec![
                // div { color: red; } — 低特异性
                Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    }],
                }),
                // #main { color: blue; } — 高特异性
                Rule::Style(StyleRule {
                    selectors: vec![Selector {
                        complex: ComplexSelector {
                            parts: vec![(
                                CompoundSelector {
                                    type_selector: None,
                                    subclass_selectors: vec![SubclassSelector::Id("main".to_string())],
                                },
                                None,
                            )],
                        },
                    }],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "blue".to_string(),
                        important: false,
                    }],
                }),
            ],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    // 同层内高特异性胜出
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255)); // blue
}

#[test]
fn test_layer_important_beats_normal() {
    // 分层内的 !important 仍然胜出（按 normal < important 规则）
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![
            Rule::Layer(zero_css_parser::ast::LayerRule {
                name: "base".to_string(),
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "blue".to_string(),
                        important: true,
                    }],
                })],
            }),
            Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            }),
        ],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    // !important 总是胜过 normal（即使分层 vs 未分层）
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255)); // blue
}

// ═══════════════════════════════════════════════════════════════════
// 新增端到端测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// grid-column-start/end 端到端
fn test_grid_column_start_end_end_to_end() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![
                Declaration {
                    property: "grid-column-start".to_string(),
                    value: "2".to_string(),
                    important: false,
                },
                Declaration {
                    property: "grid-column-end".to_string(),
                    value: "5".to_string(),
                    important: false,
                },
            ],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.grid_column_start, property::GridLineValue::Line(2));
    assert_eq!(div_style.grid_column_end, property::GridLineValue::Line(5));
}

#[test]
/// grid-row-start/end 端到端
fn test_grid_row_start_end_end_to_end() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![
                Declaration {
                    property: "grid-row-start".to_string(),
                    value: "1".to_string(),
                    important: false,
                },
                Declaration {
                    property: "grid-row-end".to_string(),
                    value: "3".to_string(),
                    important: false,
                },
            ],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.grid_row_start, property::GridLineValue::Line(1));
    assert_eq!(div_style.grid_row_end, property::GridLineValue::Line(3));
}

#[test]
/// grid-area 简写端到端
fn test_grid_area_shorthand_end_to_end() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "grid-area".to_string(),
                value: "1 / 2 / 3 / 4".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.grid_row_start, property::GridLineValue::Line(1));
    assert_eq!(div_style.grid_row_end, property::GridLineValue::Line(3));
    assert_eq!(div_style.grid_column_start, property::GridLineValue::Line(2));
    assert_eq!(div_style.grid_column_end, property::GridLineValue::Line(4));
}

#[test]
/// span-based grid placement 端到端
fn test_grid_span_placement_end_to_end() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![
                Declaration {
                    property: "grid-column-start".to_string(),
                    value: "span 2".to_string(),
                    important: false,
                },
                Declaration {
                    property: "grid-column-end".to_string(),
                    value: "5".to_string(),
                    important: false,
                },
            ],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.grid_column_start, property::GridLineValue::Span(2));
    assert_eq!(div_style.grid_column_end, property::GridLineValue::Line(5));
}

#[test]
/// negative grid line numbers 端到端
fn test_grid_negative_line_numbers_end_to_end() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "grid-column-start".to_string(),
                value: "-1".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.grid_column_start, property::GridLineValue::Line(-1));
}

#[test]
/// transition-duration 端到端
fn test_transition_duration_end_to_end() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "transition".to_string(),
                value: "opacity 0.5s ease 0.1s".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.transition_property, vec!["opacity"]);
    assert_eq!(div_style.transition_duration, vec![0.5]);
    assert_eq!(div_style.transition_delay, vec![0.1]);
}

#[test]
/// animation-direction values 端到端
fn test_animation_direction_values_end_to_end() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "animation".to_string(),
                value: "fadeIn 1s linear infinite alternate".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.animation_name, vec!["fadeIn"]);
    assert_eq!(div_style.animation_direction.len(), 1);
}

#[test]
/// animation-fill-mode forwards 端到端
fn test_animation_fill_mode_forwards_end_to_end() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "animation".to_string(),
                value: "slideUp 0.3s ease forwards".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.animation_fill_mode.len(), 1);
}

#[test]
/// animation-play-state paused 端到端
fn test_animation_play_state_paused_end_to_end() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "animation".to_string(),
                value: "spin 2s linear paused".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.animation_play_state.len(), 1);
}

#[test]
/// flex shorthand 端到端
fn test_flex_shorthand_end_to_end() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "flex".to_string(),
                value: "2 1 100px".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.flex_grow, 2.0);
    assert_eq!(div_style.flex_shrink, 1.0);
}

#[test]
/// transform 端到端
fn test_transform_end_to_end() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "transform".to_string(),
                value: "translateX(10px)".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert!(!matches!(
        div_style.transform,
        zero_css_parser::values::TransformValue::None
    ));
}

#[test]
/// 自定义属性与颜色端到端
fn test_custom_property_with_color_end_to_end() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![
                Declaration {
                    property: "--main-color".to_string(),
                    value: "red".to_string(),
                    important: false,
                },
                Declaration {
                    property: "color".to_string(),
                    value: "blue".to_string(),
                    important: false,
                },
            ],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255));
}

/// var() 引用在样式计算管线中正确解析。
#[test]
fn test_var_resolution_in_pipeline() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![
                Declaration {
                    property: "--main-color".to_string(),
                    value: "red".to_string(),
                    important: false,
                },
                Declaration {
                    property: "color".to_string(),
                    value: "var(--main-color)".to_string(),
                    important: false,
                },
            ],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    // var(--main-color) 应该被解析为 "red"
    assert_eq!(div_style.color, ColorValue::Rgba(255, 0, 0, 255));
}

/// var() 带回退值时，变量不存在则使用回退。
#[test]
fn test_var_fallback_in_pipeline() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "color".to_string(),
                value: "var(--undefined, blue)".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255));
}

/// var() 解析 width 长度值。
#[test]
fn test_var_resolution_width_length() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![
                Declaration {
                    property: "--my-width".to_string(),
                    value: "100px".to_string(),
                    important: false,
                },
                Declaration {
                    property: "width".to_string(),
                    value: "var(--my-width)".to_string(),
                    important: false,
                },
            ],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.width, LengthValue::Px(100.0));
}

/// 嵌套 var() 正确解析。
#[test]
fn test_var_nested_resolution() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![
                Declaration {
                    property: "--base".to_string(),
                    value: "red".to_string(),
                    important: false,
                },
                Declaration {
                    property: "--accent".to_string(),
                    value: "var(--base)".to_string(),
                    important: false,
                },
                Declaration {
                    property: "color".to_string(),
                    value: "var(--accent)".to_string(),
                    important: false,
                },
            ],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.color, ColorValue::Rgba(255, 0, 0, 255));
}

// ── CSS 数学函数端到端测试 ──

/// 测试 calc() 在宽度属性中的端到端应用。
#[test]
fn test_calc_width_e2e() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "width".to_string(),
                value: "calc(100px + 50px)".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    // calc(100px + 50px) = 150px
    assert_eq!(div_style.width, LengthValue::Px(150.0));
}

/// 测试 min() 在宽度属性中的端到端应用。
#[test]
fn test_min_width_e2e() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "width".to_string(),
                value: "min(200px, 100px)".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.width, LengthValue::Px(100.0));
}

/// 测试 max() 在高度属性中的端到端应用。
#[test]
fn test_max_height_e2e() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "height".to_string(),
                value: "max(50px, 120px)".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.height, LengthValue::Px(120.0));
}

/// 测试 clamp() 在边距属性中的端到端应用。
#[test]
fn test_clamp_margin_e2e() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "margin-top".to_string(),
                value: "clamp(10px, 50px, 100px)".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    // clamp(10, 50, 100) — 50 在范围内，结果为 50
    assert_eq!(div_style.margin_top, LengthValue::Px(50.0));
}

/// 测试 calc() 嵌套 min() 在内边距中的端到端应用。
#[test]
fn test_calc_nested_min_padding_e2e() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "padding-left".to_string(),
                value: "calc(min(30px, 20px) + 10px)".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    // min(30,20)=20, 20+10=30
    assert_eq!(div_style.padding_left, LengthValue::Px(30.0));
}

/// 测试 calc() 与 em 单位混合在宽度中的端到端应用。
#[test]
fn test_calc_em_width_e2e() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![
                Declaration {
                    property: "font-size".to_string(),
                    value: "20px".to_string(),
                    important: false,
                },
                Declaration {
                    property: "width".to_string(),
                    value: "calc(2em + 10px)".to_string(),
                    important: false,
                },
            ],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    // 2em = 2*20 = 40px, 40+10=50px
    assert_eq!(div_style.width, LengthValue::Px(50.0));
}

// ── aspect-ratio 端到端测试 ──

/// 测试 aspect-ratio 数值解析。
#[test]
fn test_aspect_ratio_number() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "aspect-ratio".to_string(),
                value: "1.5".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.aspect_ratio, Some(1.5));
}

/// 测试 aspect-ratio 斜杠语法（16 / 9）。
#[test]
fn test_aspect_ratio_slash_syntax() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "aspect-ratio".to_string(),
                value: "16 / 9".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    let ratio = div_style.aspect_ratio.expect("should have aspect-ratio");
    assert!((ratio - 16.0 / 9.0).abs() < 0.01);
}

/// 测试 aspect-ratio: auto 重置为 None。
#[test]
fn test_aspect_ratio_auto() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "aspect-ratio".to_string(),
                value: "auto".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.aspect_ratio, None);
}

/// 测试 aspect-ratio 默认值为 None。
#[test]
fn test_aspect_ratio_default() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    assert_eq!(div_style.aspect_ratio, None);
}

// ═══════════════════════════════════════════════════════════════════
// 属性边界条件测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// cursor 是继承属性：父元素设置 cursor:pointer，子元素无显式 cursor 时继承 pointer
fn test_cursor_inheritance() {
    let (doc, _html, _body, _div, p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "cursor".to_string(),
                value: "pointer".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let p_style = styles.get(&p).expect("p 应该有样式");
    // cursor 是继承属性，p 应从 div 继承 pointer
    assert_eq!(p_style.cursor, property::CursorValue::Pointer);
}

#[test]
/// opacity 不是继承属性：父元素设置 opacity:0.5，子元素默认 opacity 为 1.0
fn test_opacity_inheritance() {
    let (doc, _html, _body, _div, p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "opacity".to_string(),
                value: "0.5".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let p_style = styles.get(&p).expect("p 应该有样式");
    // opacity 不继承，子元素默认 1.0
    assert_eq!(p_style.opacity, 1.0);
}

#[test]
/// transition-property: none → 保留 ["none"]（R2756：区分 `transition: none` 与未设置，对齐 Chromium）
fn test_transition_property_none() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "transition".to_string(),
                value: "none".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    // transition: none → transition-property = ["none"]（引擎在 transition.rs 跳过 "none" 名）
    assert_eq!(div_style.transition_property, vec!["none".to_string()]);
}

#[test]
/// animation-name: none → 保留 ["none"]（R2756：动画管线过滤 `n != "none"`，不入动画系统）
fn test_animation_name_none() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "animation".to_string(),
                value: "none".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    // animation: none → animation-name = ["none"]
    assert_eq!(div_style.animation_name, vec!["none".to_string()]);
}

#[test]
/// box-sizing: border-box 时，border 宽度从总宽度中扣除，
/// 内容区域宽度 = 指定宽度 - border 宽度
fn test_box_sizing_effect_on_width() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![
                Declaration {
                    property: "box-sizing".to_string(),
                    value: "border-box".to_string(),
                    important: false,
                },
                Declaration {
                    property: "width".to_string(),
                    value: "100px".to_string(),
                    important: false,
                },
                Declaration {
                    property: "border".to_string(),
                    value: "10px solid black".to_string(),
                    important: false,
                },
            ],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    // box-sizing 应用为 border-box
    assert_eq!(div_style.box_sizing, BoxSizingValue::BorderBox);
    // width 仍为 100px（内容宽度计算由布局引擎完成）
    assert_eq!(div_style.width, LengthValue::Px(100.0));
    // border 各边宽度为 10px
    assert_eq!(div_style.border_top_width, LengthValue::Px(10.0));
    assert_eq!(div_style.border_left_width, LengthValue::Px(10.0));
}

#[test]
/// transform 支持多个变换函数组合：translateX(10px) rotate(45deg) 两个函数都被应用
fn test_multiple_transform_functions() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "transform".to_string(),
                value: "translateX(10px) rotate(45deg)".to_string(),
                important: false,
            }],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    // 应该解析为 TransformValue::List 且包含两个函数
    match &div_style.transform {
        zero_css_parser::values::TransformValue::List(funcs) => {
            assert_eq!(funcs.len(), 2, "应包含两个变换函数");
            // 验证第一个函数是 translateX(10px)
            assert!(
                matches!(&funcs[0], zero_css_parser::values::TransformFunction::TranslateX(v) if (*v - 10.0).abs() < 0.01),
                "第一个函数应为 translateX(10px)"
            );
            // 验证第二个函数是 rotate(45deg)
            assert!(
                matches!(&funcs[1], zero_css_parser::values::TransformFunction::Rotate(v) if (*v - 45.0).abs() < 0.01),
                "第二个函数应为 rotate(45deg)"
            );
        }
        other => panic!("transform 应为 List，实际为 {:?}", other),
    }
}

// ═══════════════════════════════════════════════════════════════════
// @layer 排序与级联验证端到端测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// 后 @layer 的声明在特异性相等时覆盖前 @layer 的声明。
///
/// 场景：@layer base { div { color: red } } @layer theme { div { color: green } }
/// 两个选择器特异性都是 (0,0,1)，theme 层索引更大，应胜出。
fn test_layer_ordering_specificity() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![
            // @layer base — color: red
            Rule::Layer(zero_css_parser::ast::LayerRule {
                name: "base".to_string(),
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "red".to_string(),
                        important: false,
                    }],
                })],
            }),
            // @layer theme — color: green（同特异性，后层胜出）
            Rule::Layer(zero_css_parser::ast::LayerRule {
                name: "theme".to_string(),
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "green".to_string(),
                        important: false,
                    }],
                })],
            }),
        ],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div 应该有样式");
    // 后层（theme=green）在特异性相等时胜过前层（base=red）
    assert_eq!(div_style.color, ColorValue::Rgba(0, 128, 0, 255)); // green
}

#[test]
/// 未分层样式覆盖分层样式，无论特异性高低。
///
/// 场景：@layer base { #main { color: blue } } div { color: red }
/// 分层内用 ID 选择器 (1,0,0)，未分层用标签选择器 (0,0,1)。
/// 未分层仍应胜出。
fn test_unlayered_overrides_layered() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let id_sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Id("main".to_string())],
                },
                None,
            )],
        },
    };

    let stylesheets = vec![Stylesheet {
        rules: vec![
            // @layer base — #main { color: blue }，高特异性但在层内
            Rule::Layer(zero_css_parser::ast::LayerRule {
                name: "base".to_string(),
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![id_sel],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "blue".to_string(),
                        important: false,
                    }],
                })],
            }),
            // 未分层 — div { color: red }，低特异性但未分层
            Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            }),
        ],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div 应该有样式");
    // 未分层声明胜过分层声明（无论特异性）
    assert_eq!(div_style.color, ColorValue::Rgba(255, 0, 0, 255)); // red
}

#[test]
/// !important 声明在级联中胜过 normal 声明，即使前者在更早的 @layer。
///
/// 场景：@layer base { div { color: blue !important } }
///        @layer theme { div { color: green } }
/// blue 的 !important 使其胜过 green 的 normal。
fn test_important_overrides_layer_order() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![
            // @layer base — color: blue !important
            Rule::Layer(zero_css_parser::ast::LayerRule {
                name: "base".to_string(),
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "blue".to_string(),
                        important: true,
                    }],
                })],
            }),
            // @layer theme — color: green（后层但 normal）
            Rule::Layer(zero_css_parser::ast::LayerRule {
                name: "theme".to_string(),
                rules: vec![Rule::Style(StyleRule {
                    selectors: vec![make_tag_selector("div")],
                    declarations: vec![Declaration {
                        property: "color".to_string(),
                        value: "green".to_string(),
                        important: false,
                    }],
                })],
            }),
        ],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div 应该有样式");
    // !important 胜过后层的 normal 声明
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255)); // blue
}

#[test]
/// 验证特异性优先级：内联 > ID > class > element > universal。
///
/// 为 div#main 同时应用多个不同特异性的 color 声明，
/// 级联应选择特异性最高的胜出者。
/// 注意：本测试不使用内联样式（引擎不支持 style 属性），
/// 只验证 ID > class > element > universal。
fn test_cascade_specificity_order() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let universal_sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Universal),
                    subclass_selectors: vec![],
                },
                None,
            )],
        },
    };
    let class_sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Class("nonexistent".to_string())],
                },
                None,
            )],
        },
    };
    let id_sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Id("main".to_string())],
                },
                None,
            )],
        },
    };

    // 通用选择器 (0,0,0) → purple
    // 标签选择器 (0,0,1) → red
    // class 选择器 (0,1,0) → yellow（不匹配 div，仅用于对比）
    // ID 选择器 (1,0,0) → blue
    let stylesheets = vec![Stylesheet {
        rules: vec![
            Rule::Style(StyleRule {
                selectors: vec![universal_sel],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "purple".to_string(),
                    important: false,
                }],
            }),
            Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            }),
            Rule::Style(StyleRule {
                selectors: vec![class_sel],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "yellow".to_string(),
                    important: false,
                }],
            }),
            Rule::Style(StyleRule {
                selectors: vec![id_sel],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "blue".to_string(),
                    important: false,
                }],
            }),
        ],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div 应该有样式");
    // ID 选择器 (1,0,0) 特异性最高，blue 胜出
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255)); // blue

    // 额外验证：去掉 ID 选择器后，标签选择器应胜过通用选择器
    let stylesheets_no_id = vec![Stylesheet {
        rules: vec![
            Rule::Style(StyleRule {
                selectors: vec![Selector {
                    complex: ComplexSelector {
                        parts: vec![(
                            CompoundSelector {
                                type_selector: Some(TypeSelector::Universal),
                                subclass_selectors: vec![],
                            },
                            None,
                        )],
                    },
                }],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "purple".to_string(),
                    important: false,
                }],
            }),
            Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            }),
        ],
    }];

    let mut sys2 = StyleSystem::new();
    let styles2 = sys2.compute_styles(&doc, &stylesheets_no_id);
    let div_style2 = styles2.get(&div).expect("div 应该有样式");
    // 标签选择器 (0,0,1) > 通用选择器 (0,0,0)
    assert_eq!(div_style2.color, ColorValue::Rgba(255, 0, 0, 255)); // red
}

#[test]
/// 验证 !important 声明对同一属性的优先级高于 normal 声明。
///
/// 场景：div { color: red !important } div { color: blue }
/// 即使两个声明同源同特异性，!important 胜出。
fn test_cascade_importance_order() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![
            // normal 声明 — color: blue
            Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "blue".to_string(),
                    important: false,
                }],
            }),
            // !important 声明 — color: red
            Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: true,
                }],
            }),
        ],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div 应该有样式");
    // !important 的 red 胜过 normal 的 blue
    assert_eq!(div_style.color, ColorValue::Rgba(255, 0, 0, 255)); // red

    // 额外验证：即使 !important 声明在前，仍然胜出
    let stylesheets_important_first = vec![Stylesheet {
        rules: vec![
            // !important 在前 — color: green
            Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "green".to_string(),
                    important: true,
                }],
            }),
            // normal 在后 — color: blue
            Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "blue".to_string(),
                    important: false,
                }],
            }),
        ],
    }];

    let mut sys2 = StyleSystem::new();
    let styles2 = sys2.compute_styles(&doc, &stylesheets_important_first);
    let div_style2 = styles2.get(&div).expect("div 应该有样式");
    // !important 在前仍然胜过后面的 normal
    assert_eq!(div_style2.color, ColorValue::Rgba(0, 128, 0, 255)); // green
}

#[test]
/// 验证在特异性和重要性相等时，后出现的声明胜出。
///
/// 场景：div { color: red } div { color: green } div { color: blue }
/// 三个声明同源、同重要性、同特异性，位置靠后的 blue 胜出。
fn test_cascade_origin_order() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();

    let stylesheets = vec![Stylesheet {
        rules: vec![
            // 第一个声明 — color: red
            Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            }),
            // 第二个声明 — color: green
            Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "green".to_string(),
                    important: false,
                }],
            }),
            // 第三个声明 — color: blue（最后出现）
            Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "blue".to_string(),
                    important: false,
                }],
            }),
        ],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div 应该有样式");
    // 同特异性同重要性时，最后出现的声明胜出
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255)); // blue

    // 额外验证：不同样式表中同样遵循后出现胜出规则
    let mut sys2 = StyleSystem::new();
    let stylesheets_multi = vec![
        Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            })],
        },
        Stylesheet {
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "green".to_string(),
                    important: false,
                }],
            })],
        },
    ];

    let styles2 = sys2.compute_styles(&doc, &stylesheets_multi);
    let div_style2 = styles2.get(&div).expect("div 应该有样式");
    // 第二个样式表的 green 胜过第一个的 red
    assert_eq!(div_style2.color, ColorValue::Rgba(0, 128, 0, 255)); // green
}

// ═══════════════════════════════════════════════════════════════════
// 容器查询端到端测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// 容器宽度 500px，@container (min-width: 400px) → 条件满足，样式应用。
fn test_container_query_min_width_applies() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();
    sys.set_viewport(500.0, 600.0);

    // @container (min-width: 400px) { div { color: red; } }
    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Container(zero_css_parser::ast::ContainerRule {
            name: None,
            condition: zero_css_parser::ast::ContainerCondition::Size(zero_css_parser::ast::ContainerSizeCondition {
                feature: "min-width".to_string(),
                value: "400px".to_string(),
                operator: None,
                range_min: None,
                range_max: None,
            }),
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            })],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    // 容器宽度 500px >= 400px，条件满足，color 应为红色
    assert_eq!(div_style.color, ColorValue::Rgba(255, 0, 0, 255));
}

#[test]
/// 容器宽度 300px，@container (min-width: 400px) → 条件不满足，样式不应用。
fn test_container_query_min_width_not_applies() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();
    sys.set_viewport(300.0, 600.0);

    // @container (min-width: 400px) { div { color: red; } }
    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Container(zero_css_parser::ast::ContainerRule {
            name: None,
            condition: zero_css_parser::ast::ContainerCondition::Size(zero_css_parser::ast::ContainerSizeCondition {
                feature: "min-width".to_string(),
                value: "400px".to_string(),
                operator: None,
                range_min: None,
                range_max: None,
            }),
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            })],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    // 容器宽度 300px < 400px，条件不满足，color 保持默认黑色
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 0, 255));
}

#[test]
/// 容器宽度 500px，@container (max-width: 600px) → 500px <= 600px，条件满足。
fn test_container_query_max_width() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();
    sys.set_viewport(500.0, 600.0);

    // @container (max-width: 600px) { div { color: green; } }
    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Container(zero_css_parser::ast::ContainerRule {
            name: None,
            condition: zero_css_parser::ast::ContainerCondition::Size(zero_css_parser::ast::ContainerSizeCondition {
                feature: "max-width".to_string(),
                value: "600px".to_string(),
                operator: None,
                range_min: None,
                range_max: None,
            }),
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "green".to_string(),
                    important: false,
                }],
            })],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    // 容器宽度 500px <= 600px，max-width 条件满足
    assert_eq!(div_style.color, ColorValue::Rgba(0, 128, 0, 255));
}

#[test]
/// 范围语法：@container (200px <= width <= 500px)，容器宽度 350px → 在范围内，样式应用。
fn test_container_query_range_syntax() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();
    sys.set_viewport(350.0, 600.0);

    // @container (200px <= width <= 500px) { div { color: blue; } }
    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Container(zero_css_parser::ast::ContainerRule {
            name: None,
            condition: zero_css_parser::ast::ContainerCondition::Size(zero_css_parser::ast::ContainerSizeCondition {
                feature: "width".to_string(),
                value: String::new(),
                operator: None,
                range_min: Some("200px".to_string()),
                range_max: Some("500px".to_string()),
            }),
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "blue".to_string(),
                    important: false,
                }],
            })],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    // 200 <= 350 <= 500，范围条件满足
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 255, 255));

    // 额外验证：超出范围时不应用
    let mut sys2 = StyleSystem::new();
    sys2.set_viewport(600.0, 400.0);
    let styles2 = sys2.compute_styles(&doc, &stylesheets);
    let div_style2 = styles2.get(&div).expect("div should have style");
    // 600 > 500，超出上界，不应用
    assert_eq!(div_style2.color, ColorValue::Rgba(0, 0, 0, 255));
}

#[test]
/// @container 无 ContainerContext（未设置视口）→ 不应用容器查询样式。
fn test_container_query_no_context() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let mut sys = StyleSystem::new();
    // 不设置视口，无 ContainerContext

    // @container (min-width: 400px) { div { color: red; } }
    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Container(zero_css_parser::ast::ContainerRule {
            name: None,
            condition: zero_css_parser::ast::ContainerCondition::Size(zero_css_parser::ast::ContainerSizeCondition {
                feature: "min-width".to_string(),
                value: "400px".to_string(),
                operator: None,
                range_min: None,
                range_max: None,
            }),
            rules: vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            })],
        })],
    }];

    let styles = sys.compute_styles(&doc, &stylesheets);
    let div_style = styles.get(&div).expect("div should have style");
    // 无容器上下文，@container 不应用，color 保持默认黑色
    assert_eq!(div_style.color, ColorValue::Rgba(0, 0, 0, 255));
}

// ═══════════════════════════════════════════════════════════════════
// Scroll Snap 端到端测试
