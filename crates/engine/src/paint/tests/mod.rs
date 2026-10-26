//! paint 模块的单元测试。

mod advanced;
mod advanced_part2;
mod background_repeat;
mod basic;
mod border_image_repeat;
mod clearfix;
mod content_objectfit;
mod counters;
mod css_remaining;
mod css_table_3d;
mod css_unpainted;
mod effects;
mod effects_visual;
mod helpers;
mod interaction_indicators;
mod r2311_background_multilayer_tests;
mod r3818_canvas_bg_anchor_tests;
mod r3997_ifc_ar_transfer;
mod r4058_cv_hidden_replaced_tests;
mod r4059_shadow_clip_tests;
mod r4068_bfc_empty_child_margin_tests;
mod r4073_abspos_intrinsic_pct_box_side_tests;
mod r4074_ar_transfer_ife_reclaim_tests;
mod r5150_contain_size_multicol_paint;
mod r5151_zeroh_fragmentainer_oof_flush;
mod r5155_contain_paint_corner_clip_tests;
mod r5171_inline_ca_paint;
mod r5173_transform_3d_axis_direct;
mod r5174_parent_perspective;
mod r5175_perspective_used_value;
mod r5176_transformed_paint_phase;
mod r5177_inline_text_color;
mod r5179_corner_shape_geometry;
mod r5180_round_border_band;
mod r5182_border_shape_fill_ring;
mod r5183_shaped_percent_radius;
mod r5185_border_area_shape_band;
mod r5186_shape_join_outline;
mod r5187_svg_css_presentation;
mod r5188_border_shape_shadow;
mod r5201_overflow_clip_radius_tests;
mod r5202_anon_table_caption_tests;
mod r5203_bg_clip_radius_tests;
mod r5204_skip_spaces_decoration_tests;
/// R5205 paint Path B 分带窗口绝对锚定（line-clamp serif 簇省略号行位）。
mod r5205_clamp_ellipsis_anchor_tests;
/// R5255 迟延 positioned 后代的 shaped clip edge（R5201 圆角改写覆盖 R3922 补裁窗口）。
mod r5255_deferred_positioned_rounded_clip_tests;
/// R5256 整contents 选区的后代 owner 行（R5054 后代 owner FIXME 首片）。
mod r5256_selection_descendant_owner_tests;
/// R5258 ::first-line 首格式行字形 color（切片 1，CSS2 §5.12.1）。
mod r5258_first_line_color_tests;
/// R5259 appearance widget 纳入 per-node 效果窗口（transform 作用于全部渲染输出）。
mod r5259_appearance_transform_window_tests;
/// R5260 clip:text 恒色染色接入 flex/grid 匿名文本项（css-backgrounds-4 §background-clip:text）。
mod r5260_clip_text_anonymous_item_tests;
/// R5263 collapse 横带端部随相邻竖格线胜者伸缩（CSS2 §17.6.2 带端归属）。
mod r5263_collapse_band_end_tests;
mod r5264_collapse_content_half_shift_tests;
/// R5265 collapse 带 z 序相位间插（CSS2.1 Appendix E / Blink kCollapsedTableBorders）。
mod r5265_collapse_band_phase_tests;
/// R5267 clip:text 恒色层间混合（css-compositing-1 §background-blend-mode）。
mod r5267_bg_clip_text_layer_blend_tests;
/// R5268 clip:text 逐层 clip（css-backgrounds-4 §background-clip:text）。
mod r5268_bg_clip_text_per_layer_tests;
/// R5269 inline owner 自持选区行高亮（css-pseudo-4 §highlight-cascading）。
mod r5269_inline_owner_selection_tests;
/// R5270 inline owner 选区行高亮 fill 同站同几何（css-pseudo-4 §highlight-painting）。
mod r5270_inline_owner_selection_fill_tests;
/// R5271 选区高亮 UA 默认回落面（css-pseudo-4 §highlight-styling）。
mod r5271_selection_ua_default_tests;
/// R5272 祖先链整 contents 选区 UA 默认高亮（css-pseudo-4 §highlight-painting）。
mod r5272_selection_ancestor_ua_default_tests;
/// R5273 图片替换元素作者 ::selection 底 tint（css-pseudo-4 §highlight-painting）。
mod r5273_selection_img_bg_tests;
mod r5296_inline_bg_ml_gate_tests;
/// R5297 多片段 slice inline 背景行连续域 + 内容区锚（css-break-3 §5.3）。
mod r5297_inline_ml_band_extents_tests;
mod s10_hotlist_paint_tests;
mod ui_indicators;
mod visual;
mod visual_inline;
mod visual_part2;
mod visual_resize_scroll;
