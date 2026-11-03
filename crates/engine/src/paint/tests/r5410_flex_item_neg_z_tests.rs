//! R5410：static flex item 负 z-index 下沉绘制（css-flexbox-1 #painting，
//! flex-item-z-ordering-002）。
//!
//! 规范语义：「z-index applies to flex items even if they are position: static」——
//! 负 z-index flex item 下沉到常规流内容之下绘制（Appendix E step 2 类比）。legacy
//! 相位非 positioned 子 z-index 不参与堆叠（child_paint_sort_key 原门），红盒上浮。
//! 正 z-index 建堆叠上下文域无驱动案，另案挂账。kill-switch `ZW_FLEX_ITEM_NEG_Z=0`
//!（OFF 轮由 corpus 002 ✗2.08% 精确回退覆盖，不设进程内单测——R3858 口径）。
use crate::pipeline::{RenderPipeline, RenderResult};

/// fills 按色的绘制序 index（首命中）。
fn fill_index(result: &RenderResult, rgb: [u8; 3]) -> Option<usize> {
    result
        .primitives()
        .fills
        .iter()
        .position(|f| f.color.r == rgb[0] && f.color.g == rgb[1] && f.color.b == rgb[2] && f.color.a == 255)
}

/// ★load-bearing（flex-item-z-ordering-002 形态）：flex 容器两 item，item1 绿子
/// 100×100、item2（z-index:-10）红子 100×100。修复后 item2 下沉先绘——红 index <
/// 绿 index，红被绿覆盖（旧行为红后绘上浮，ref 判红）。
#[test]
fn r5410_negative_z_flex_item_paints_below_inflow() {
    let html = r#"<!DOCTYPE html><style>
body { margin: 0; }
#flex { display: flex; width: 100px; height: 100px; }
#flex > div { width: 0px; height: 0px; }
#flex > div > div { width: 100px; height: 100px; }
</style>
<div id="flex">
  <div><div style="background: green"></div></div>
  <div style="z-index: -10"><div style="background: red;"></div></div>
</div>"#;
    let result = RenderPipeline::new(200.0, 200.0).render_html(html, "");
    let red = fill_index(&result, [255, 0, 0]);
    let green = fill_index(&result, [0, 128, 0]);
    let (red, green) = (red.expect("red fill"), green.expect("green fill"));
    assert!(
        red < green,
        "负 z-index flex item 应下沉先绘：red idx {} 应 < green idx {}",
        red,
        green
    );
}

/// 非 flex 父守卫：块容器内重叠兄弟（负 margin 上拉）z-index:-10 不参与堆叠——
/// legacy 相位保持 tree order（绿先红后，红可见）。flex 门不得外溢到普通块流。
#[test]
fn r5410_block_parent_negative_z_keeps_tree_order() {
    let html = r#"<!DOCTYPE html><style>
body { margin: 0; }
#a { width: 100px; height: 100px; background: green; }
#b { width: 100px; height: 100px; background: red; margin-top: -100px; z-index: -10; }
</style>
<div id="a"></div>
<div id="b"></div>"#;
    let result = RenderPipeline::new(200.0, 200.0).render_html(html, "");
    let red = fill_index(&result, [255, 0, 0]);
    let green = fill_index(&result, [0, 128, 0]);
    let (red, green) = (red.expect("red fill"), green.expect("green fill"));
    assert!(
        green < red,
        "非 flex 父负 z-index 静态子不参与堆叠：green idx {} 应 < red idx {}（legacy tree order）",
        green,
        red
    );
}
