//! t-trending 伪影 1（github trending 星数竖排碎列）回归——`float:right` 容器含
//! inline replaced svg + 文本的单行 shrink-to-fit（css-sizing-3 §max-content +
//! CSS2 §10.3.5）。修复臂（float_positioning.rs Σ 臂）：① 早退门 Σ 开时放行
//! inline 级 replaced 子；② replaced_w（svg_max_content_contribution）入账扩臂；
//! ③ [`float_text_max_width_adv`] 真实 advance 源测文本分量。
//!
//! 几何语义参照 live 修复后 [1000,442,113,19]（svg 16 + 文本 ≈97，单行 19px）与
//! Chrome 参照 [997.6,499,115.4,18]：宽 = svg + 文本 Σ 收缩、高 = 单行 line-height。
//!
//! kill-switch 敏感性（返修硬要求，外部 env 实跑取证，见返修证据 r1 日志）：
//! - `ZW_FLOAT_INLINE_SUM=0`：早退门恢复（svg is_replaced 触发）→ float 滞留 taffy
//!   max(子盒)=16px 塌缩、文本折行竖排（实測 w=16/h=73 词竖排碎列 = live 修复前形态）→ 红；
//! - `ZW_FLOAT_STF_ADV=0`：文本回 scoped walk——harness advance 源为 EstimateAdvance
//!   启发式（walk/adv 同源），本用例无真实字体度量分裂可复现，实测不改变几何（见
//!   SUMMARY；VF 窄计场景需真实字体，单测面不可达，如实披露）。
//!
//! fixture 用 `white-space:nowrap` 钉单行断言：harness 内 walk/IFC 同为 per-char
//! 启发式度量，auto 收缩宽与 IFC 行需求逐 f32 求和存在 ~0.2px 漂移（实测定宽 65px
//! 单行、auto 收缩 64.8 折行），普通折行语境下单行断言贴边不稳定；nowrap 使 IFC
//! 跳过拟合门，单行成为确定几何，敏感性与伪影钉在塌缩宽度（w=16）上。

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

/// ① svg+文本混合臂：星数 float 单行收缩——高 = 单行 line-height（19px），
/// 宽 = svg 16 + 文本 Σ（replaced_w 入账），盒右缘贴容器右（float:right）。
#[test]
fn test_trending_float_star_svg_text_single_line() {
    let html = r##"<html><head></head><body style="margin:0">
<div><div id="star" style="float:right;white-space:nowrap;font-size:12px;line-height:19px"><svg width="16" height="16" style="display:inline-block"></svg> 1,527 stars today</div></div>
    </body></html>"##;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(1280.0, 800.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(1280.0, 800.0);
    let result = engine.compute(&doc, &styles);
    let star = find_box("star", &doc, &result.root).expect("float star box");

    // 臂前置条件：svg 是 float 盒的 inline 级 replaced 子（早退门放行与 replaced_w
    // 入账的判定对象；svg 误分类块级/丢失盒则本测试失去靶点）。
    let svg_child = star
        .children
        .iter()
        .find(|c| c.is_replaced)
        .expect("svg must be a replaced box child of the star float");
    assert!(
        !svg_child.is_block_level,
        "svg must stay inline-level (early-exit gate arm precondition)"
    );
    assert_eq!(svg_child.width, 16.0, "svg attr intrinsic width must resolve to 16");

    // 单行：高恰为显式 line-height 19px。旧行为（Σ OFF）塌缩 16px 宽后文本逐词
    // 折行竖排（实测 h=73 碎列）。
    assert_eq!(
        star.height, 19.0,
        "star float must be single-line (h=line-height 19); got {} \
         (vertical word-fragmentation = artifact 1 regression)",
        star.height
    );
    // 宽含 svg+文本 Σ（≈116.8）：旧塌缩形态 w=16（taffy max(子盒)，svg 外文本溢出）。
    assert!(
        (100.0..=140.0).contains(&star.width),
        "star float width must shrink to svg 16 + text sum (~117); got {} (collapsed-to-svg = artifact 1)",
        star.width
    );
    // float:right 贴容器右缘（容器 1280，容差 0.5 取整）。
    assert!(
        star.x + star.width <= 1280.5,
        "right float must hug container right edge; got x={} w={}",
        star.x,
        star.width
    );
}

/// ② 纯文本 float 对照（无元素子）：不进 Σ 臂元素子路径，walk/adv 两口径一致——
/// 修复与 kill-switch OFF 均不得使纯文本 float 漂移（守既有口径）。
#[test]
fn test_trending_float_pure_text_control_single_line() {
    let html = r##"<html><head></head><body style="margin:0">
<div><div id="plain" style="float:right;white-space:nowrap;font-size:12px;line-height:19px">1,527 stars today</div></div>
    </body></html>"##;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(1280.0, 800.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(1280.0, 800.0);
    let result = engine.compute(&doc, &styles);
    let plain = find_box("plain", &doc, &result.root).expect("pure-text float box");

    assert_eq!(
        plain.height, 19.0,
        "pure-text float must stay single-line (control); got {}",
        plain.height
    );
    assert!(
        (60.0..=200.0).contains(&plain.width),
        "pure-text float width must shrink to text content (~101); got {}",
        plain.width
    );
}
