//! R5415：OOF 锚空行 strut 取容器 used line-height——行盒唯一流内容是 OofAnchor
//!（零宽，css-position-3 §4.1 OOF 不参与行内布局）+ Br 时，行高 = strut
//!（css-inline-3 §5.2.2 线盒高不计 OOF 盒 / CSS2 §10.8.1）而非 20px legacy。
//!
//! 案形（css/CSS2/positioning/abspos-block-level-001.html ref 页 div1：
//! `<div>\n    <span class="absolute green">…</span>\n    <br>\n  </div>`）——
//! 修复前锚行 20px、test 页文本行 18px，ref 全页 +2px 错位（✗1.44%）；chromium
//! oracle 双页均 18px 单行（截图带扫描定谳）。修复后 ✓0.39%。
//!
//! ★形约束：div1 内须含源空白文本节点——布局探针实证：带空白节点 div1 成
//! blk=true 真块盒（IFC 行高计入 extent）；无空白节点 div1 走匿名 0×0 盒
//!（行高不进 extent，本修复前后渲染不变）——故测试形必须复刻 corpus 真文件
//! 的换行缩进。
//!
//! 门收窄：非锚 br 空行维持 20px legacy（R5322 判例：全量换 strut →
//! ruby-align-001/001a 0.77→1.02 越阈翻红，已回退另案入账）。
//! kill-switch `ZW_OOF_ANCHOR_LINE_STRUT=0` 回退 20px（OFF 精确回退由 corpus
//! A/B 双窗覆盖；OnceLock 进程级缓存不支撑同进程双极性测试）。

use zero_engine::RenderPipeline;

/// corpus ref 页 div1 形态（含源空白节点）：锚空行 + 后随在流蓝块。
/// 返回蓝块 y（= body margin 8 + div1 锚行高）。
fn render_blue_y(html: &str) -> f32 {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(html, "");
    result
        .primitives()
        .fills
        .iter()
        .find(|f| f.color.r == 0 && f.color.g == 0 && f.color.b == 255 && f.color.a == 255)
        .unwrap_or_else(|| panic!("后随蓝块填充缺失"))
        .rect
        .origin
        .y
}

/// ★load-bearing（abspos-block-level-001 ref 页 div1 形态）：锚空行 strut = 容器
/// used line-height（16px 字体 normal 行高 18）→ body margin 8 + 18 = 后随块 y=26。
/// 修复前 20px legacy → y=28（与 test 页 18px 文本行错位 2px，全页累积）。
#[test]
fn r5415_oof_anchor_empty_line_strut() {
    let html = "<body><div>\n    <span style=\"position:absolute;background:lime\">Block-level abspos before inline content</span>\n    <br>\n  </div><div style=\"background:blue;height:10px\"></div></body>";
    let blue_y = render_blue_y(html);
    assert!(
        (blue_y - 26.0).abs() < 0.5,
        "后随块 y = {blue_y} 应 26（body margin 8 + 锚行 strut 18；28 = 20px legacy 判据越界即红）"
    );
}

/// 守卫（R5322 判例保护）：无锚 br 空行维持 20px legacy strut——门未收窄到非锚
/// 空行，ruby-align-001/001a 的承重近似不被扰动。
#[test]
fn r5415_plain_br_empty_line_keeps_legacy_strut() {
    let html = "<body><div><br></div><div style=\"background:blue;height:10px\"></div></body>";
    let blue_y = render_blue_y(html);
    assert!(
        (blue_y - 28.0).abs() < 0.5,
        "无锚 br 空行后随块 y = {blue_y} 应 28（body margin 8 + legacy 20；判例不变）"
    );
}
