//! R5396：正 clearance × 自塌缩空块塌穿组的差量落位 + 祖先高度传播回归测试。
//!
//! 谱系：WPT `css/CSS2/floats-clear/margin-collapse-125.xht`（CSS 2.1 §8.3.1）。
//! 结构：非 BFC 容器含 float + clear（空块，mt=mb=2em）+ 塌穿延续兄弟（mt=1em、
//! border-top=2em）。clearance 打在自塌缩空块上时，塌穿组 margin（自身 mt∪mb ∪
//! 后继 mt）作为单一边距被 clear 空隙吸收：
//!  - 元素 border 边落 clear_bottom（clear.y == float 底边）；
//!  - 紧随的塌穿延续兄弟不再重放整条链下值（旧语义 = clear_bottom + collapse(
//!    clear.mb, 后继.mt)），只落 clear_bottom + max(0, 链下值 − 链上值)（chromium
//!    oracle 实证：125 mt=mb=32 链上=链下 → 差量 0 → 兄弟贴 clear_bottom）。
//!  - 容器 auto 高随之收缩（float 排除 + 差量），且收缩量经 clearance_extent_
//!    reduction 向祖先传播（R5109 对称臂）——td 按 taffy 期 extent 计高，不传播
//!    则残留泄漏段（125 的 td 高 134 应 102 露红底）。
//!
//! kill-switch 敏感性（外部 env 实跑取证，2026-11-03，test-guard 包裹 runner）：
//! - `ZW_CLEAR_EMPTY_CHAIN_ABSORB=0`（旧整链重放）：125 隔离单跑 ✗（0/1，真通过-
//!   可信 0%）——差量落位与传播链整体回退，本文件断言转红；
//! - `=0` 下 table-anonymous-objects-146/148 隔离单跑仍 ✓（关断态与干净 HEAD
//!   行为一致，吸收旗标不产生关断态旁路效应）。
//!
//! 配套正差量形态（r1316 探针族，clear mt=40/mb=80）：链上值 40 < 链下值 80 →
//! 后继差量 40（旧语义 collapse(80,0)=80）——区分新旧语义的 load-bearing 断言。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::{Document, NodeKind};
use zero_style_system::StyleSystem;

fn find_box<'a>(id: &str, doc: &Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
    if let Some(nid) = b.node_id
        && let Some(n) = doc.get(nid)
        && let NodeKind::Element(elem) = &n.kind
        && elem.get_attribute("id").as_deref() == Some(id)
    {
        return Some(b);
    }
    b.children.iter().find_map(|c| find_box(id, doc, c))
}

/// 125 主形态（差量 0）：clear 贴 float 底边，塌穿延续兄弟不再重放链下值，
/// 落 clear_bottom + 0；容器高 96（clear_bottom 64 + 兄弟 border-top 32）。
/// 旧整链重放语义：兄弟 @ 64 + collapse(32,16) = 96、容器 128（taffy extent）。
#[test]
fn test_mc125_empty_clear_absorbs_chain_diff_zero() {
    let html = r#"<html><body style="margin:0">
      <div id="container" style="margin:0;width:128px">
        <div id="float-left" style="float:left;height:64px;width:64px"></div>
        <div id="clear-left" style="clear:left;margin-top:32px;margin-bottom:32px"></div>
        <div id="block" style="margin-top:16px;border-top:32px solid yellow"></div>
      </div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let container = find_box("container", &doc, &result.root).expect("should find #container");
    let float_left = find_box("float-left", &doc, &result.root).expect("should find #float-left");
    let clear_left = find_box("clear-left", &doc, &result.root).expect("should find #clear-left");
    let block = find_box("block", &doc, &result.root).expect("should find #block");

    // 前置：clearance 生效，clear 元素 border 边落 float 底边（塌穿组 margin 被吸收）。
    let float_bottom = float_left.y + float_left.height;
    assert!(
        (clear_left.y - float_bottom).abs() <= 0.5,
        "cleared empty block border edge must sit at float bottom (chain absorbed); \
         clear.y={} float_bottom={}",
        clear_left.y,
        float_bottom
    );

    // ★ R5396 load-bearing（差量 0）：紧随塌穿延续兄弟贴 clear_bottom——
    // 旧整链重放落 collapse(32,16)=32 → 96，此处必须为 64。
    assert!(
        (block.y - clear_left.y).abs() <= 0.5,
        "collapse-through sibling must land at clear_bottom (delta 0, not full chain replay); \
         block.y={} clear.y={}",
        block.y,
        clear_left.y
    );

    // 容器 auto 高：float 排除（§10.5.1）+ 差量落位 → clear_bottom + 兄弟 border 32 = 96
    //（旧语义 128：链下值整段计入 taffy extent）。
    assert!(
        (container.height - 96.0).abs() <= 0.5,
        "container auto height must be 96 (float excluded + diff-zero landing); h={}",
        container.height
    );
}

/// 125 表格形态（祖先传播）：td 按 taffy 期 extent 计高，容器的 float 排除收缩
/// 须经 clearance_extent_reduction 传播收缩 td（R5109 对称臂）——否则 td 高 134
/// 残留 2em 泄漏段露红底（案面像素差 1.95% 的第二层残差根因）。
#[test]
fn test_mc125_td_propagates_container_shrink() {
    let html = r#"<html><head><style>
      table, td { padding: 0; border-spacing: 0; }
      .comparator { border-spacing: 1em; }
      td { border: solid; }
    </style></head><body style="margin:0">
      <table class="comparator"><tr>
        <td id="test">
          <div id="container" style="margin:0;width:128px">
            <div style="float:left;height:64px;width:64px"></div>
            <div style="clear:left;margin-top:32px;margin-bottom:32px"></div>
            <div style="margin-top:16px;border-top:32px solid yellow"></div>
          </div>
        </td>
      </tr></table>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(
        "table, td { padding: 0; border-spacing: 0; } \
         .comparator { border-spacing: 1em; } \
         td { border: solid; }",
    );
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let container = find_box("container", &doc, &result.root).expect("should find #container");
    let td = find_box("test", &doc, &result.root).expect("should find td#test");

    assert!(
        (container.height - 96.0).abs() <= 0.5,
        "container must shrink to 96; h={}",
        container.height
    );
    // ★ R5396 load-bearing（R5109 对称臂）：td = 容器 96 + td 自身边框 6 = 102
    //（不传播则 134 = taffy 期 extent 128 + 边框 6）。
    assert!(
        (td.height - 102.0).abs() <= 0.5,
        "td must follow container shrink via clearance_extent_reduction (R5109 mirror); \
         td.height={} (stale extent leaks red strip)",
        td.height
    );
}

/// 正差量形态（r1316 探针族）：clear mt=40/mb=80、后继空块 mt=0/mb=140。
/// 链上值 = collapse(0, 40) = 40，链下值 = collapse(80, 0) = 80 → 后继差量 40；
/// 旧整链重放语义落 collapse(80,0)=80。差量断言区分新旧语义。
#[test]
fn test_positive_clearance_delta_offset_following_sibling() {
    let html = r#"<html><body style="margin:0">
      <div id="container" style="margin:0;width:128px">
        <div id="float-left" style="float:left;height:100px;width:100px"></div>
        <div id="clear-left" style="clear:left;margin-top:40px;margin-bottom:80px"></div>
        <div id="following" style="margin-bottom:140px"></div>
      </div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let clear_left = find_box("clear-left", &doc, &result.root).expect("should find #clear-left");
    let following = find_box("following", &doc, &result.root).expect("should find #following");

    // ★ R5396 load-bearing（正差量）：following 落 clear_bottom + (80 − 40) = +40。
    // 旧整链重放为 +80（collapse(80,0)）；R1316 前缺陷为负值（陈旧位）。
    let delta = following.y - clear_left.y;
    assert!(
        (delta - 40.0).abs() <= 0.5,
        "following sibling offset must be chain diff (80-40=40); delta={} clear.y={} following.y={}",
        delta,
        clear_left.y,
        following.y
    );
    // DOM 顺序不回退（R1316 既有口径）。
    assert!(
        delta >= -0.5,
        "following must not precede cleared block (R1316); delta={}",
        delta
    );
}

/// 兄弟自持 clear 的链重启：吸收旗标在重启前复位（后继自持 clear 自成塌穿链，
/// 不吃前一组的差量），锚定 ClearValue::Left 复位点。
#[test]
fn test_absorb_flag_resets_on_self_clearing_sibling() {
    let html = r#"<html><body style="margin:0">
      <div id="container" style="margin:0;width:128px">
        <div id="float-left" style="float:left;height:64px;width:64px"></div>
        <div id="clear-left" style="clear:left;margin-top:32px;margin-bottom:32px"></div>
        <div id="next-clear" style="clear:left;height:20px"></div>
      </div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let float_left = find_box("float-left", &doc, &result.root).expect("should find #float-left");
    let next_clear = find_box("next-clear", &doc, &result.root).expect("should find #next-clear");

    // 自持 clear 的后继须仍避开 float（链重启语义：clearance 独立生效），
    // 不因吸收旗标复位缺失而吃到错误差量。
    let float_bottom = float_left.y + float_left.height;
    assert!(
        next_clear.y + 0.5 >= float_bottom,
        "self-clearing sibling must still clear the float; y={} float_bottom={}",
        next_clear.y,
        float_bottom
    );
}
