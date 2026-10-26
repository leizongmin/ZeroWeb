//! inline 测试模块。

mod advanced;
mod advanced_float_tab;
mod basic;
mod display_none_leak;
mod edge_cases;
mod plaintext;
mod r1975_vertical_ifc_inf_probe;
mod r2156_inline_box_model_coherence;
mod r3701_inline_replaced_attr_finite_tests;
mod r3836_rtl_override_line_reversal;
mod r5101_path_b_oof_skip_tests;
mod r5102_oof_walk_prune_tests;
mod r5165_atomic_gl_no_break_tests;
mod r5229_word_separator_ws_tests;
mod r5235_nested_atomic_walk_signal_tests;
mod r5239_bidi_control_metering_probe;
mod r5240_first_word_float_fit_probe;
/// R5274 preserve 模式纯空白行盒计 line-height（CSS2 §9.2.1.1 preserved white space）。
mod r5274_preserved_ws_line_height_tests;
/// R5297 跨 inline 边界空白折叠（css-text-3 §4.1）：词尾随空白吞并后继前导空白。
mod r5297_ws_collapse_inline_boundary_tests;
mod text_align;
mod text_indent;
mod vertical_bidi;
