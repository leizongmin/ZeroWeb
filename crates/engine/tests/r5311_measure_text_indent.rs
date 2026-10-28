//! R5311：taffy 测量 IFC 同源 text-indent 回归钉（CSS2 §10.3.1）。
//!
//! 根因：`measure_text_content` 构造测量 IFC 漏 `.with_text_indent()`——auto 高度块的
//! taffy 内容高按「无首行缩进」的行数计；布局 IFC（inline_finalization）与 paint Path B
//! 均带缩进多折一行，末行溢出盒外（c547-indent-001 丢词签名：196px 容器 + 50% 缩进，
//! 测量 3 行 54 高，布局 4 行，第 4 行 "block." 超出盒高被裁）。
//!
//! Ahem 16px 确定性断言：同文两 div，width:196px + text-indent:98px 者首行余 98px
//!（3 词/行 → 17 词 4 行），width:197px 无缩进者 6 词/行（3 行）——修复前测量面无视
//! 缩进，两 div 等高（3 行），断言红；修复后缩进版高一整行。
//!
//! 诊断链记档：
//! docs/goal/rendering-compat/evidence/r5311-measure-text-indent-2026-10-28.md
//!
//! https://www.w3.org/TR/CSS21/text.html#indentation-prop

use zero_engine::RenderPipeline;

const CASE: &str = r#"<!DOCTYPE html>
<style>
body { margin: 0; font-family: Ahem; font-size: 16px; }
div { width: 197px; }
#ind { width: 196px; text-indent: 98px; }
</style>
<div id="ind">a a a a a a a a a a a a a a a a a</div>
<div id="plain">a a a a a a a a a a a a a a a a a</div>"#;

/// 测量 IFC 须消费 text-indent：带缩进容器（首行余 98px 装 3 词）测高比同文无缩进
/// 容器（6 词/行）多一个行盒。修复前测量 IFC 无缩进，两 div 同为 3 行等高。
#[test]
fn measured_height_accounts_for_text_indent() {
    let result = RenderPipeline::new(800.0, 600.0).render_html(CASE, "");
    let snapshot = result.layout.snapshot();
    let height_of = |marker: &str| {
        snapshot
            .lines()
            .find(|l| l.contains(marker))
            .and_then(|l| {
                let idx = l.find("size=(")? + "size=(".len();
                l[idx..]
                    .split(',')
                    .nth(1)?
                    .split(')')
                    .next()?
                    .trim()
                    .parse::<f32>()
                    .ok()
            })
            .unwrap_or_else(|| panic!("快照缺少 {marker} 对应盒行；实际快照：\n{snapshot}"))
    };
    let ind_h = height_of("size=(196.00,");
    let plain_h = height_of("size=(197.00,");
    assert!(
        ind_h - plain_h >= 1.0,
        "带 text-indent 容器的测量高须比同文无缩进容器多一个行盒\
         （缩进 4 行 vs 无缩进 3 行；测量 IFC 漏 text-indent 时两者等高）。\
         实际 ind={ind_h} plain={plain_h}；快照：\n{snapshot}"
    );
}
