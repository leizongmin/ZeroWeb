//! R5487：物化 inline 层 float 序列级联（CSS2 §9.5.1 左 float 横向排布）。
//!
//! 蓝本＝R5486 chromium headless oracle 四形（400 容器两 left float；断言用
//! ABS 坐标＝LayoutBox 相对父编码沿父链累加 (x+bl+pl, y+bt+pt)，口径对齐
//! chromium gBCR oracle）：①原形（inline 层 ml=40，两 50%=200 float）chromium
//! 堆叠 (0,0)+(0,18)；②v_c（ml=0）并排 (0,0)+(200,0)；③v_a（ml=40，40%=160）
//! 并排 (0,0)+(160,0)——fit 放**前 float 右缘**非光标位（40+160=200）；④v_d
//! （两独立 inline 层各一 float，ml=40）堆叠 (0,0)+(0,18)。fit 判定用 inline
//! 光标位＝前 float 右缘＋wrapper 链偏移（原形 40+200=240，可用 160<200 →
//! 降带 y=max(锚,前 float 底)、x=own ml）。pre-fix 旧径（R5446 逐 float 独立
//! 目标）①②③全重叠 (0,0)×2 红证据谱系。
//!
//! R5492 容器高勘误：R5486 oracle 把 **float 带底误读为块容器高**（「带至 y
//! 35/17」＝cyan float 带延伸，非 .block 背景底）。R5492 标准（DOCTYPE）模式
//! marker-div 复测（in-flow 红标 y 位＝blk 底）：①③④ blk=**18**（ml40＝非零
//! mpb 行内部分 → strut 计入，CSS2 §9.4.2×§10.6.3×§10.8.1，float extent 36 不
//! 计）、② blk=**0**（ml0 全零 mpb ＝零高行盒）。float extent 一律不计入普通块
//! 高（R5026/R5470 立场维持）。float **位形**断言（堆叠/并排/fit 右缘）不受
//! 影响——①的堆叠 (0,0)+(0,18) 复测仍真（cyan 列 x 0..199 跨 y 0..35 双带）。
//! blk 高值随 R5492 strut/0 二分更正为 chromium 真值。
//! kill-switch：`ZW_NESTED_FLOAT_CASCADE=0`（R3858 独立进程锚见
//! r5487_nested_float_cascade_kill_switch.rs）。

use zero_layout_engine::{LayoutBox, LayoutEngine};
use zero_style_system::StyleSystem;

fn sheets(html: &str) -> Vec<zero_css_parser::Stylesheet> {
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find("<style") {
        let Some(tail_start) = rest[start..].find('>') else {
            break;
        };
        let after = &rest[start + tail_start + 1..];
        let Some(end) = after.find("</style>") else { break };
        out.push(zero_css_parser::Parser::parse_stylesheet(&after[..end]));
        rest = &after[end + 8..];
    }
    out
}

fn abs_rect(root: &LayoutBox, id: zero_dom::NodeId) -> (f32, f32, f32, f32) {
    // LayoutBox.x/y 为相对父级内容区域坐标（types/mod.rs 文档语义）；ABS = 沿
    // 父链累加 (x+bl+pl, y+bt+pt)。
    fn walk(b: &LayoutBox, id: zero_dom::NodeId, x: f32, y: f32) -> Option<(f32, f32, f32, f32)> {
        let ax = x + b.x + b.border_left + b.padding_left;
        let ay = y + b.y + b.border_top + b.padding_top;
        if b.node_id == Some(id) {
            return Some((ax, ay, b.width, b.height));
        }
        for c in &b.children {
            if let Some(r) = walk(c, id, ax, ay) {
                return Some(r);
            }
        }
        None
    }
    walk(root, id, 0.0, 0.0).expect("目标盒存在")
}

fn compute(html: &str) -> zero_layout_engine::LayoutResult {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles)
}

fn span_id(doc: &zero_dom::Document, name: &str) -> zero_dom::NodeId {
    doc.get_element_by_id(name).expect("span 存在")
}

const TPL: &str = r#"<html><head><style>
body { margin: 0; font: 16px/18px Ahem, sans-serif; }
.block { display: block; width: 400px; }
.inline { display: inline; MARGIN }
span { float: left; SPANW }
</style></head><body>
<div class="block" id="blk"><div class="inline" id="inl">
  <span id="s1">A</span><span id="s2">B</span>
</div></div>
</body></html>"#;

/// ①原形（oracle 堆叠形）：inline 层 ml=40，两 50%=200 float——cursor=40+200=
/// 240，可用 160<200 → float2 降带 (0,18)；pre-fix 重叠 (0,0)×2 红证据谱系。
#[test]
fn r5487_cascade_orig_stacks_second_float() {
    let html = TPL
        .replace("MARGIN", "margin-left: 40px;")
        .replace("SPANW", "width: 50%");
    let result = compute(&html);
    let doc = zero_dom::parse_html(&html);
    let f1 = abs_rect(&result.root, span_id(&doc, "s1"));
    assert!(
        f1.0.abs() <= 0.5 && f1.1.abs() <= 0.5,
        "float1 应在容器内容原点 (0,0)，实际 {:?}",
        f1
    );
    let f2 = abs_rect(&result.root, span_id(&doc, "s2"));
    assert!(
        f2.0.abs() <= 0.5 && (f2.1 - 18.0).abs() <= 0.5,
        "float2 应降带堆叠 (0,18)（chromium oracle；pre-fix 重叠 (0,0)），实际 {:?}",
        f2
    );
    let blk = doc.get_element_by_id("blk").expect("blk 存在");
    let b = abs_rect(&result.root, blk);
    assert!(
        (b.3 - 18.0).abs() <= 0.5,
        "容器高应 18（chromium marker 复测：ml40 strut 计入，float 堆叠 extent 36 \
         不计——R5492 勘误 pre-fix「带至 y 35」系 float 带底误读），实际 {}",
        b.3
    );
}

/// ②v_c（oracle 并排形）：inline 层 ml=0——cursor=0+200=200，可用 200≥200 →
/// fit 并排放带右缘 (200,0)，单带容器高 18。
#[test]
fn r5487_cascade_ml0_side_by_side() {
    let html = TPL
        .replace("MARGIN", "margin-left: 0px;")
        .replace("SPANW", "width: 50%");
    let result = compute(&html);
    let doc = zero_dom::parse_html(&html);
    let f2 = abs_rect(&result.root, span_id(&doc, "s2"));
    assert!(
        (f2.0 - 200.0).abs() <= 0.5 && f2.1.abs() <= 0.5,
        "float2 应并排放前 float 右缘 (200,0)（chromium oracle；pre-fix 重叠 (0,0)），实际 {:?}",
        f2
    );
    let blk = doc.get_element_by_id("blk").expect("blk 存在");
    let b = abs_rect(&result.root, blk);
    assert!(
        (b.3 - 0.0).abs() <= 0.5,
        "容器高应 0（chromium marker 复测：ml0 全零 mpb ＝零高行盒，float 并排 extent \
         18 不计——R5492 勘误 pre-fix「带至 y 17」系 float 带底误读），实际 {}",
        b.3
    );
}

/// ③v_a（oracle fit 右缘形）：inline 层 ml=40，40%=160——cursor=40+160=200，
/// 可用 200≥160 → fit 并排 x=**前 float 右缘 160**（非光标位 200），y=0。
#[test]
fn r5487_cascade_fit_places_at_prev_right_edge() {
    let html = TPL
        .replace("MARGIN", "margin-left: 40px;")
        .replace("SPANW", "width: 40%");
    let result = compute(&html);
    let doc = zero_dom::parse_html(&html);
    let f2 = abs_rect(&result.root, span_id(&doc, "s2"));
    assert!(
        (f2.0 - 160.0).abs() <= 0.5 && f2.1.abs() <= 0.5,
        "fit float2 应放前 float 右缘 (160,0) 而非光标位 200（chromium oracle 定谳），实际 {:?}",
        f2
    );
    let blk = doc.get_element_by_id("blk").expect("blk 存在");
    let b = abs_rect(&result.root, blk);
    // chromium marker 复测 blk=18＝strut（ml40，§9.4.2）——与单带 extent 18 碰巧
    // 同值；R5492 后由 strut 臂供给（R5486 时代归因 float extent 系误读域）。
    assert!((b.3 - 18.0).abs() <= 0.5, "单带并排容器高应 18（strut），实际 {}", b.3);
}

/// ④v_d（oracle 双层堆叠形）：两独立 inline 层各一 float（各 ml=40，50%）——
/// float2 光标位含自身层链偏移，可用不足 → 降带 (0,18)。
#[test]
fn r5487_cascade_two_layers_stack() {
    let html = r#"<html><head><style>
body { margin: 0; font: 16px/18px Ahem, sans-serif; }
.block { display: block; width: 400px; }
.inline { display: inline; margin-left: 40px; }
span { float: left; width: 50% }
</style></head><body>
<div class="block" id="blk">
  <div class="inline" id="inl"><span id="s1">A</span></div>
  <div class="inline" id="inl2"><span id="s2">B</span></div>
</div>
</body></html>"#;
    let result = compute(html);
    let doc = zero_dom::parse_html(html);
    let f2 = abs_rect(&result.root, span_id(&doc, "s2"));
    assert!(
        f2.0.abs() <= 0.5 && (f2.1 - 18.0).abs() <= 0.5,
        "双层形 float2 应降带堆叠 (0,18)（chromium oracle；pre-fix 重叠 (0,0)），实际 {:?}",
        f2
    );
    let blk = doc.get_element_by_id("blk").expect("blk 存在");
    let b = abs_rect(&result.root, blk);
    assert!(
        (b.3 - 18.0).abs() <= 0.5,
        "双层堆叠容器高应 18（chromium marker 复测：ml40 strut 计入，float extent 36 \
         不计——R5492 勘误 pre-fix「带至 y 35」系 float 带底误读），实际 {}",
        b.3
    );
}

/// ⑤对照形（R5486 已正确路径防回归）：float 块容器直子（无 inline 层）两 50%
/// float 并排 (0,0)+(200,0)——direct 路径既有级联不受嵌套臂改动影响。
#[test]
fn r5487_direct_float_children_stay_side_by_side() {
    let html = r#"<html><head><style>
body { margin: 0; font: 16px/18px Ahem, sans-serif; }
.block { display: block; width: 400px; }
span { float: left; width: 50% }
</style></head><body>
<div class="block" id="blk"><span id="s1">A</span><span id="s2">B</span></div>
</body></html>"#;
    let result = compute(html);
    let doc = zero_dom::parse_html(html);
    let f1 = abs_rect(&result.root, span_id(&doc, "s1"));
    assert!(
        f1.0.abs() <= 0.5 && f1.1.abs() <= 0.5,
        "直子 float1 应 (0,0)，实际 {:?}",
        f1
    );
    let f2 = abs_rect(&result.root, span_id(&doc, "s2"));
    assert!(
        (f2.0 - 200.0).abs() <= 0.5 && f2.1.abs() <= 0.5,
        "直子 float2 应并排 (200,0)（R5486 对照形 oracle＝已正确路径），实际 {:?}",
        f2
    );
}
