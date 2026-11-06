//! R5432（css-overflow-4 §block-ellipsis × CSS2 §10.3.8）：line-clamp 整行置换链
//! 收口——驱动案 block-ellipsis-018（R5342 slice 1 行盒 strut 后的 slice 2）。
//!
//! 三臂：
//! 1. abspos 几何同步 pass（`sync_clamped_abspos_geometry`，kill-switch
//!    `ZW_CLAMP_GEOMETRY_SYNC=0`）——clamp 容器是 taffy branch abspos 盒（无 measure
//!    函数）时盒高/`top:auto+bottom` 锚定不感知 clamp cap，compute_final 之后读存储
//!    行盒收口（018：存储行 strut 25、盒高 43.65、锚 y=26 三源不一致 → 盒高 25/y=75）。
//! 2. extent 臂（apply_cross_block_line_clamp）——容器自身 stored IFC 已 cap 时，经
//!    容器 IFC 绘制的 inline 包装子盒（无自身 stored IFC）extent 贡献钳到 stored 可见
//!    底边；跨块容器（无 stored IFC）不入臂（line-clamp-007 独立 BFC 子案回归卫）。
//! 3. 旗标镜像——`clamp_strut_applied` 镜像见 inline/tests/r5342 同文件新增针。
//!
//! kill-switch OFF 轮不设单测（进程内 set_var 不可靠，R3858 教训），由 corpus A/B
//! 覆盖（018 OFF ✓0.52% ×3 / ON ✓0.00% ×3）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

fn compute(html: &str, css: &str) -> crate::LayoutResult {
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles)
}

/// 找到携带存树 IFC 的首个盒（R1576 嵌套形态落在 inline 子盒；直存形态在自身）。
fn find_stored_ifc_box(b: &LayoutBox) -> Option<&LayoutBox> {
    if b.inline_layout.is_some() {
        return Some(b);
    }
    for c in &b.children {
        if let Some(found) = find_stored_ifc_box(c) {
            return Some(found);
        }
    }
    None
}

/// 主判例（block-ellipsis-018 同构，018 真案形）：abspos clamp 盒（`bottom:0` 贴底、
/// 无作者高、`top:auto`）内 1.5em 溢出长词整行被置换。修复前盒高 43.65（taffy
/// MaxContent 子盒 generic 比率）、y=26（截断前两行总高锚定）；修复后盒高 = 存储
/// strut 行底 25、y = CB 高 100 − bottom 0 − 25 = 75（§10.3.8 方程）。
#[test]
fn r5432_clamped_abspos_bottom_anchor_syncs_to_stored_ifc() {
    let css = ".cb { position: relative; height: 100px; width: 100px; font: 25px Ahem; } \
               .clamp { position: absolute; bottom: 0; left: 0; width: 100px; line-clamp: 1; color: transparent; } \
               .large { font-size: 1.5em; }";
    let html = format!(
        r#"<html><head><style>{css}</style></head><body style="margin:0">
<div class="cb"><div class="clamp"><span class="large">XXXXXXXXX XXX</span></div></div>
</body></html>"#
    );
    let result = compute(&html, css);
    let clamp_box = find_stored_ifc_box(&result.root).expect("clamp 盒应有存树 IFC");
    assert!(
        (clamp_box.height - 25.0).abs() < 0.01,
        "clamp 盒高须同步到存储 strut 行底 25，got {}",
        clamp_box.height
    );
    assert!(
        (clamp_box.y - 75.0).abs() < 0.01,
        "clamp 盒 y 须按 §10.3.8 重锚定到 75（CB 100 − bottom 0 − 高 25），got {}",
        clamp_box.y
    );
    assert!(
        clamp_box.line_clamp_strut_displaced,
        "整行置换旗标须镜像到盒（paint stored 路径清零被置换 glyph 的依据）"
    );
}

/// 负向守卫：作者 definite 高压过 clamp 内容（height:50）时几何同步不入（盒高保持
/// 作者值、锚定按作者高走既有臂）；`top` definite 时锚定是静态位域不入。既有簇语义
/// 逐字节不变护栏。
#[test]
fn r5432_clamped_abspos_sync_negative_guards() {
    // (a) height:50px definite —— 盒高保持 50，y = 100 − 0 − 50 = 50。
    let css = ".cb { position: relative; height: 100px; width: 100px; font: 25px Ahem; } \
               .clamp { position: absolute; bottom: 0; left: 0; width: 100px; height: 50px; line-clamp: 1; } \
               .large { font-size: 1.5em; }";
    let html = format!(
        r#"<html><head><style>{css}</style></head><body style="margin:0">
<div class="cb"><div class="clamp"><span class="large">XXXXXXXXX XXX</span></div></div>
</body></html>"#
    );
    let result = compute(&html, css);
    let clamp_box = find_stored_ifc_box(&result.root).expect("clamp 盒应有存树 IFC");
    assert!(
        (clamp_box.height - 50.0).abs() < 0.01,
        "作者 definite 高不入同步臂（保持 50），got {}",
        clamp_box.height
    );
    // (b) top:0 definite + bottom:auto —— 锚定走既有 top 臂，y 保持 0。
    let css = ".cb { position: relative; height: 100px; width: 100px; font: 25px Ahem; } \
               .clamp { position: absolute; top: 0; left: 0; width: 100px; line-clamp: 1; } \
               .large { font-size: 1.5em; }";
    let html = format!(
        r#"<html><head><style>{css}</style></head><body style="margin:0">
<div class="cb"><div class="clamp"><span class="large">XXXXXXXXX XXX</span></div></div>
</body></html>"#
    );
    let result = compute(&html, css);
    let clamp_box = find_stored_ifc_box(&result.root).expect("clamp 盒应有存树 IFC");
    assert!(
        (clamp_box.y - 0.0).abs() < 0.01 && (clamp_box.height - 25.0).abs() < 0.01,
        "top definite 不入重锚定臂（y=0），盒高仍可被 extent 收口到 25，got y={} h={}",
        clamp_box.y,
        clamp_box.height
    );
}

/// extent 臂回归卫（line-clamp-007 同构，独立 BFC 子案）：跨块 clamp 容器（serif 无
/// stored IFC）不入臂——独立 BFC 子行不计预算（R3771），clamp 点后的合法子（Line 5，
/// 末保留行带 ellipsis）extent 不得被 cap×lh 误钳。修复误伤形态：容器 192→96（Line 5
/// 随背景一起被裁掉，corpus ✗15.74%）。容器高 = 32×2 + 48×2 + 32 = 192。
#[test]
fn r5432_cross_block_extent_not_clamped_without_stored_ifc() {
    let css = ".clamp { line-clamp: 3; font: 16px / 32px serif; white-space: pre; padding: 0 4px; background-color: yellow; } \
               .child { overflow: auto; font: 24px / 48px serif; color: blue; padding: 0 4px; }";
    let html = format!(
        r#"<html><head><style>{css}</style></head><body style="margin:0">
<div class="clamp">Line 1
Line 2<div class="child">Line 3
Line 4</div>Line 5
Line 6</div>
</body></html>"#
    );
    let result = compute(&html, css);
    // clamp 容器 = 黄底盒（serif 不存 IFC，按高 192 全树搜：2×32 + 2×48 + 32）。
    fn find_h192(b: &LayoutBox) -> Option<&LayoutBox> {
        if (b.height - 192.0).abs() < 0.6 {
            return Some(b);
        }
        b.children.iter().find_map(find_h192)
    }
    let clamp_div = find_h192(&result.root).expect("clamp 容器高须保持 192（Line 5 extent 未被钳掉）");
    let last = clamp_div.children.last().expect("末子（Line 5/6 包装盒）");
    assert!(
        (last.y + last.height - 192.0).abs() < 0.6,
        "末包装盒底须到 192（Line 5 extent 未被钳到 cap×lh=96），got {}",
        last.y + last.height
    );
}
