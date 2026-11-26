//! Style-system 测试模块。

mod advanced;
mod apply_coverage;
mod apply_coverage_extra;
mod basic;
mod color_scheme_synthesis;
mod coverage_round3;
/// k121：dashed-function 产物元素级 CSS 宽关键字落地钉测。
mod custom_keyword_landing;
mod custom_properties_sharing;
mod edge;
mod first_letter;
mod font_feature_values;
/// k126：函数求值端 attr() 替换＋attr-security taint 钉测。
mod function_attr_substitution;
/// k124：`@function` 体条件规则组钉测。
mod function_conditionals;
/// k125：标准属性值域 `<dashed-function>` 替换钉测。
mod function_standard_property;
mod helpers;
mod incremental_oracle;
/// k135：伪元素存储槽 background 判据扩面（CSS Shadow Parts §3.2.2）。
mod kc5r135_slot_gate;
/// k123：会话内 inherit() 记法父栈帧替换钉测。
mod inherit_substitution;
mod inline_style;
mod matcher_coverage;
mod matcher_coverage2;
mod matcher_coverage3;
mod matcher_coverage4;
mod matcher_coverage5;
mod matcher_extra;
mod pseudo_clearfix;
mod r4398_probe_ws;
mod registered_property;
mod review_r3345;
mod root_font_units;
mod shadow_host_segment;
/// k133：`::part(<ident-list>)` 段级联测试（CSS Shadow Parts §3.2.2）。
mod shadow_part_segment;
/// 45 号 M2：`::slotted` 段级联测试。
mod shadow_slotted_segment;
mod shorthand_coverage;
/// g19：sibling-index()/sibling-count() 兄弟序位置 + 端到端。
mod sibling_tree_counting;
mod style_system_coverage;
/// k122：style() 冒号等值类型感知比较钉测。
mod typed_style_query;
mod ua_dialog_overlay;
mod ua_dialog_visibility;
