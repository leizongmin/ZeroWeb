//! t-p125 常驻回归：github.com/trending select-menu 锚点页在引擎管线内的最小复现。
//!
//! 真站形状（2026-10-30 探针链）：github.com/trending 的语言/主题筛选 `<details-menu>`
//! 直挂 846 个 `role=menuitemradio` 锚点（各 ~276B，`aria-checked`/`data-pjax`/
//! `class="select-menu-item Link"` + `<span class="select-menu-item-text">` 子元素）。
//! ZeroWeb release renderer：资源加载完成后 ~22s 起内存爆炸（实测 renderer RSS 2.93GB）
//! → 进程自灭（分配失败 abort），CDP 全程失联，零 JS 错误零异常。剥离全部 `<script>`
//! 仍复现（解析/样式/布局路径，非 JS）；单卡/全卡 article、`<main>` 前后段全部健康；
//! 合成梯：同款锚点 ×400 健康（元素数 818，complete 1.16s）、×846 死亡；
//! 无属性纯锚点 ×846 健康（complete 1.14s）——内容+数量双驱动。
//!
//! 本文件三测试：前两个为对照（须绿），第三个为回归（p125 修复——dom 查询层廉价
//! 成分先行 + 每查询位置索引——之前挂死/abort，修复后须完成渲染；复现边界在
//! 400–846 锚点之间，元素总数 818–1651）。

use zero_engine::RenderPipeline;

fn fixture(n: u32) -> String {
    match n {
        846 => include_str!("fixtures/p125-select-menu-846.html").to_string(),
        400 => include_str!("fixtures/p125-select-menu-400.html").to_string(),
        _ => panic!("未登记的 fixture 档位"),
    }
}

fn plain_846() -> String {
    include_str!("fixtures/p125-select-menu-plain-846.html").to_string()
}

/// 对照 1：无属性纯锚点 ×846 须正常渲染（排除「仅数量多」归因）。
#[test]
fn plain_846_anchors_render() {
    let result = RenderPipeline::new(800.0, 600.0).render_html(&plain_846(), "");
    let snap = result.layout.snapshot();
    assert!(snap.contains("NodeId("), "纯锚点对照页须产出布局快照");
}

/// 对照 2：同款锚点 ×400（元素 818）须正常渲染（复现下界以下）。
#[test]
fn syn_400_anchors_render() {
    let result = RenderPipeline::new(800.0, 600.0).render_html(&fixture(400), "");
    let snap = result.layout.snapshot();
    assert!(snap.contains("NodeId("), "400 锚点对照页须产出布局快照");
}

/// 回归：同款锚点 ×846（元素 ~1651）——p125 修复前渲染管线挂死或 abort（真站
/// renderer 死亡形状）；修复后须可完成渲染并可查询布局快照。
#[test]
fn syn_846_anchors_render() {
    let result = RenderPipeline::new(800.0, 600.0).render_html(&fixture(846), "");
    let snap = result.layout.snapshot();
    assert!(snap.contains("NodeId("), "846 锚点页须可完成渲染并产出布局快照");
}
