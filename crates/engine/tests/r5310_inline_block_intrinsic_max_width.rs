//! R5310：非 replaced 原子 inline（inline-block 族）max 模式固有贡献 DOM 递归真测量
//! + 自身 definite max/min-width 钳制回归钉（css-sizing-3 §5.1/§5.2）。
//!
//! 根因：intrinsic 测量发生在 taffy 首趟前（R4355 同序），内容定尺寸 inline-block
//! 的 child.width 彼时为 0——`width:max-content` 容器（block_intrinsic_width_ex 实际
//! 解析路径，R1018 分发）对 inline-block 子只算到 margin（outline-028 ref 页取证：
//! `margin-left:5px` 的 `XX` 子测 5 应 25）。修复：max 模式改 `box_content_max_width`
//! DOM 递归真测量（kill-switch `ZW_IB_INTRINSIC_RECURSE=0` 回退 child.width 口径），
//! 贡献再被自身 definite max/min-width 钳制（CSS2 §10.3.7 shrink-to-fit：max-width
//! 约束长度定义父 preferred 宽——absolute-non-replaced-width-017/018/021 实证：未钳
//! 时 R5236 D2 只扩不缩写侧把 abspos 父 120 放大到内容裸值）。
//!
//! 诊断链记档：
//! docs/goal/rendering-compat/evidence/r5310-inline-block-intrinsic-2026-10-28.md
//!
//! https://drafts.csswg.org/css-sizing-3/#intrinsic-sizes
//! https://www.w3.org/TR/CSS21/visudet.html#shrink-to-fit-float

use zero_engine::RenderPipeline;

const MAX_CONTENT_CASE: &str = r#"<!DOCTYPE html>
<style>
body { margin: 0; }
div { width: max-content; }
span { display: inline-block; margin-left: 5px; }
</style>
<div><span>XX</span></div>"#;

const ABSPOS_CLAMP_CASE: &str = r#"<!DOCTYPE html>
<style>
body { margin: 0; }
div { position: absolute; font-size: 30px; }
span { display: inline-block; max-width: 120px; }
</style>
<div><span>123456789012345678901234567890</span></div>"#;

const MAX_CONTENT_CLAMP_CASE: &str = r#"<!DOCTYPE html>
<style>
body { margin: 0; }
div { width: max-content; }
span { display: inline-block; max-width: 120px; }
</style>
<div><span>123456789012345678901234567890</span></div>"#;

/// `width:max-content` 容器：内容定尺寸 inline-block 子（无 definite 宽）按 DOM 递归
/// 真测量贡献 = 文本宽 + margin（旧口径 child.width=0 只算 margin，容器塌 5px）。
#[test]
fn max_content_container_measures_content_sized_inline_block() {
    let result = RenderPipeline::new(800.0, 600.0).render_html(MAX_CONTENT_CASE, "");
    let snapshot = result.layout.snapshot();
    // 默认字体 "XX" ≈ 17.6px + margin-left 5px → 容器 ceil 23。旧口径全树无 23 宽盒
    //（容器 5、span 0）。
    assert!(
        snapshot.contains("size=(23.00,"),
        "max-content 容器须测到 inline-block 子内容宽（23 ≈ 文本 17.6 + margin 5）；\
         旧口径 child.width=0 只算 margin 塌 5。实际快照：\n{snapshot}"
    );
}

/// abspos shrink-to-fit 父：inline-block 子带 `max-width:120px` 时父 preferred 宽 =
/// 钳制后贡献 120，而非递归文本 max-content 裸值（post-taffy D2 重测 child.width 已
/// 解析走旧口径，taffy 首趟 120 不被 R5236 只扩不缩写侧放大）。
#[test]
fn abspos_shrink_to_fit_clamped_by_child_max_width() {
    let result = RenderPipeline::new(800.0, 600.0).render_html(ABSPOS_CLAMP_CASE, "");
    let line = result
        .layout
        .snapshot()
        .lines()
        .rev()
        .find(|l| l.contains(" abs"))
        .unwrap_or_default()
        .trim()
        .to_string();
    assert!(
        line.contains("size=(120.00,"),
        "abspos 父 shrink-to-fit 宽须为子 max-width 钳制值 120（内容裸值 ~600 未钳）；实际：{line}"
    );
}

/// `width:max-content` 容器（pre-taffy 塌 0 递归域）：inline-block 子带 definite
/// `max-width:120px` 时贡献 = 钳制值 120，而非递归文本 max-content 裸值（未钳时
/// 容器测 ~450）。断言取容器 div 行（倒数第二个元素盒；span 自身布局期 max-width
/// 也是 120，contains 全文匹配会假阳性）。
#[test]
fn max_content_container_clamped_by_child_max_width() {
    let result = RenderPipeline::new(800.0, 600.0).render_html(MAX_CONTENT_CLAMP_CASE, "");
    let div_line = result
        .layout
        .snapshot()
        .lines()
        .filter(|l| l.contains("NodeId("))
        .rev()
        .nth(1)
        .unwrap_or_default()
        .trim()
        .to_string();
    assert!(
        div_line.contains("size=(120.00,"),
        "max-content 容器须取子 max-width 钳制贡献 120（文本裸值 ~240 未钳）；实际：{div_line}"
    );
}
