//! CSS 属性值类型。
//!
//! 定义 CSS 属性值的类型化表示，以及解析函数。

pub mod anchor;
pub mod anchor_area;
pub mod attr;
pub mod calc_mix;
pub mod color;
pub mod color_math;
pub mod named_colors;
pub mod parse_extended;
pub mod parse_extended_border;
pub mod parse_extended_visual;
pub mod parse_layout;
pub mod parse_misc;
pub mod parse_transform;
pub mod random;
pub mod serialize_gradient;
pub mod serialize_math;
mod transform_helpers;
pub mod types;

// Re-export all public items.
pub use anchor::*;
pub use attr::{
    attr_calls_valid, attr_target_fetches_url, eval_attr_call, has_attr_call, has_attr_word, substitute_attr_values,
    substitute_attr_values_checked, substitute_attr_values_deferring, substitute_attr_values_readjudicate,
};
pub use calc_mix::{CalcMixItem, CalcMixWeight};
pub use color::*;
pub use color_math::*;
pub use named_colors::*;
pub use parse_extended::*;
pub use parse_extended_border::*;
pub use parse_extended_visual::*;
pub use parse_layout::*;
pub use parse_misc::*;
pub use parse_transform::*;
pub use random::{
    RandomEvalContext, RandomFamily, RandomSpec, canonicalize_random_keys, parse_random_args, random_family_allowed,
    random_item_args_valid, substitute_random_values,
};
pub use serialize_gradient::*;
pub use serialize_math::*;
pub use types::*;

#[cfg(test)]
mod tests;
