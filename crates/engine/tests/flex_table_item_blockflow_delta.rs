//! 空 display:table 子项在 row 方向 flex 容器中的块流 delta 记账回归（trending
//! live 判别实验收敛的常驻 fixture）。
//!
//! 根因：Primer clearfix 伪元素（`::before/::after { content: ""; display: table }`）
//! 盒化为空表 flex item 后，字体收敛触发表收缩（CSS2 §17.1 空表无行 → 0×0），
//! 块流增量记账（兄弟 y += delta、容器高 += delta）被错误应用到 row flex 语境——
//! main 轴水平，item 不参与块流垂直堆叠（css-flexbox-1 §4），auto 交叉尺寸应取
//! max(in-flow item bottom)。修复面：
//! - `layout-engine/src/table.rs::reflow_siblings_after_table_height_change`
//!   row flex 守卫（跳过兄弟位移 + auto 交叉尺寸重算）
//! - `layout-engine/src/engine/postprocess.rs::clamp_percentage_max_height`
//!   递归内同族记账守卫（`is_row_flex_container` + `recompute_row_flex_cross_size`）
//!
//! 断言全部用字体无关的结构判据（兄弟 main 轴起点 y_rel、表 0×0、显式高尊显），
//! 不冻结文本 advance 数字，跨字体环境稳健。

use zero_engine::RenderPipeline;
use zero_layout_engine::LayoutBox;

fn walk_abs(box_node: &LayoutBox, target: zero_dom::NodeId, ox: f32, oy: f32) -> Option<(&LayoutBox, f32, f32)> {
    let ax = ox + box_node.x;
    let ay = oy + box_node.y;
    if box_node.node_id == Some(target) {
        return Some((box_node, ax, ay));
    }
    box_node.children.iter().find_map(|c| walk_abs(c, target, ax, ay))
}

fn class_box<'a>(root: &'a LayoutBox, doc: &zero_dom::Document, selector: &str) -> Option<(&'a LayoutBox, f32, f32)> {
    let id = doc.query_selector(doc.root(), selector)?;
    walk_abs(root, id, 0.0, 0.0)
}

/// 渲染单 fixture，在 borrow 作用域内跑断言（`Document` 无 Clone，Rc 借用内查询）。
/// CSS 经 <style> 内联，等价 render_html 全量路径。
fn with_render(html: &str, css: &str, f: impl FnOnce(&zero_dom::Document, &LayoutBox)) {
    let full = format!("<!doctype html><html><head><style>{css}</style></head><body>{html}</body></html>");
    let mut p = RenderPipeline::new(1280.0, 800.0);
    let r = p.render_html(&full, "");
    let doc_rc = p.cached_doc_shared().expect("cached doc after render_html");
    let doc = doc_rc.borrow();
    f(&doc, &r.layout.root);
}

const TOL: f32 = 0.5;

/// F1 形：row flex + 空 display:table 子 + 后继 inline 文本兄弟（BtnGroup clearfix
/// 同构）。表收缩 0×0 不得位移后继兄弟（y_rel=0），auto 交叉尺寸 = 文本行高。
#[test]
fn row_flex_empty_table_no_sibling_shift() {
    with_render(
        "<div class=\"c\"><span class=\"t\"></span><span class=\"b\">BTN</span></div>",
        ".c{display:flex}.t{display:table}",
        |doc, root| {
            let (_, _cx, cy) = class_box(root, doc, ".c").expect(".c box");
            let (tb, _, _) = class_box(root, doc, ".t").expect(".t box");
            let (bb, _, by) = class_box(root, doc, ".b").expect(".b box");
            // 空表收缩为 0 贡献（css-flexbox-1 §4.4 auto 交叉尺寸 = max item bottom）。
            assert!(
                tb.width.abs() < TOL && tb.height.abs() < TOL,
                "空 display:table flex item 应收缩 0x0，实际 {}x{}",
                tb.width,
                tb.height
            );
            // 后继兄弟不得被块流 delta 记账位移（trending BTN y −18 实证的回归面）。
            let y_rel = by - cy;
            assert!(
                y_rel.abs() < TOL,
                "row flex 后继兄弟应在 main 轴起点（y_rel≈0），实际 {y_rel}"
            );
            // 容器 auto 交叉尺寸 = max(in-flow item bottom)，不因表收缩塌缩（18→0 实证）。
            let c_height = by + bb.height - cy;
            assert!(
                (c_height - bb.height).abs() < TOL,
                "容器交叉高应=max item bottom（{}），实际 {c_height}",
                bb.height
            );
            assert!(c_height > TOL, "容器交叉高不应塌缩为 0，实际 {c_height}");
        },
    );
}

/// F4/F7/F9 变体：表在后、::after 伪元素载体（exact Primer 机制）、inline-table。
/// 修复语义对全部同型变体一致。
#[test]
fn row_flex_empty_table_variants_no_sibling_shift() {
    let cases: [(&str, &str, bool); 3] = [
        // F4：表子在后
        (
            "<div class=\"c\"><span class=\"b\">BTN</span><span class=\"t\"></span></div>",
            ".c{display:flex}.t{display:table}",
            true,
        ),
        // F7：::after 伪元素（Primer clearfix exact 机制）
        (
            "<div class=\"c\"><span class=\"b\">BTN</span></div>",
            ".c{display:flex}.c::after{content:\"\";display:table}",
            false,
        ),
        // F9：inline-table 变体
        (
            "<div class=\"c\"><span class=\"t\"></span><span class=\"b\">BTN</span></div>",
            ".c{display:flex}.t{display:inline-table}",
            true,
        ),
    ];
    for (frag, css, has_t) in cases {
        with_render(frag, css, |doc, root| {
            let (_, _cx, cy) = class_box(root, doc, ".c").unwrap_or_else(|| panic!(".c box: {css}"));
            let (bb, _, by) = class_box(root, doc, ".b").unwrap_or_else(|| panic!(".b box: {css}"));
            if has_t {
                let (tb, _, _) = class_box(root, doc, ".t").unwrap_or_else(|| panic!(".t box: {css}"));
                assert!(
                    tb.width.abs() < TOL && tb.height.abs() < TOL,
                    "空表应 0x0（{css}），实际 {}x{}",
                    tb.width,
                    tb.height
                );
            }
            let y_rel = by - cy;
            assert!(y_rel.abs() < TOL, "变体 {css}: 后继兄弟 y_rel 应≈0，实际 {y_rel}");
            let c_height = by + bb.height - cy;
            assert!(c_height > TOL, "变体 {css}: 容器交叉高不应塌缩，实际 {c_height}");
        });
    }
}

/// F6：显式高表尊显——表收缩往返（CSS2 §17.5.3 表高下限）后 height 保持 10px，
/// delta 不再经块流记账加给容器（postprocess clamp 递归腐化的回归面）。
#[test]
fn row_flex_table_explicit_height_honored() {
    with_render(
        "<div class=\"c\"><span class=\"t\" style=\"height:10px\"></span><span class=\"b\">BTN</span></div>",
        ".c{display:flex}.t{display:table}",
        |doc, root| {
            let (_, cy, _) = class_box(root, doc, ".c").expect(".c box");
            let (tb, _, _) = class_box(root, doc, ".t").expect(".t box");
            let (bb, _, by) = class_box(root, doc, ".b").expect(".b box");
            assert!(
                (tb.height - 10.0).abs() < TOL,
                "显式 height:10px 表应尊显，实际 {}",
                tb.height
            );
            let y_rel = by - cy;
            assert!(y_rel.abs() < TOL, "兄弟 y_rel 应≈0，实际 {y_rel}");
            // 容器交叉高 = max(10, 文本行)，不得被 ±delta 记账二次腐化。
            let c_height = by + bb.height - cy;
            assert!(c_height >= 10.0 - TOL, "容器交叉高应 ≥ 显式表高，实际 {c_height}");
        },
    );
}

/// F3：非空 display:table 子正常参与布局（修复不得殃及有内容的表 item）。
#[test]
fn row_flex_nonempty_table_normal() {
    with_render(
        "<div class=\"c\"><span class=\"t\">TX</span><span class=\"b\">BTN</span></div>",
        ".c{display:flex}.t{display:table}",
        |doc, root| {
            let (_, _cx, cy) = class_box(root, doc, ".c").expect(".c box");
            let (tb, _, _) = class_box(root, doc, ".t").expect(".t box");
            let (bb, _, by) = class_box(root, doc, ".b").expect(".b box");
            // 非空表有实际内容高（行盒），不收缩为 0。
            assert!(tb.height > TOL, "非空表应保留内容高，实际 {}", tb.height);
            let y_rel = by - cy;
            assert!(y_rel.abs() < TOL, "兄弟 y_rel 应≈0，实际 {y_rel}");
            let c_height = by + bb.height - cy;
            assert!(
                (c_height - tb.height.max(bb.height)).abs() < TOL,
                "容器交叉高应=max item bottom（{}），实际 {}",
                tb.height.max(bb.height),
                c_height
            );
        },
    );
}

/// F5：块容器对照——同构 markup 在 block 容器中维持块流语义（表收缩塌 0、容器
/// 保住后继文本行高），修复不得殃及块流路径。块流 inline 内容（span.b）以匿名
/// 行结构承载（R4489 fragment 宿主），无独立 LayoutBox，故以容器/表几何为判据。
#[test]
fn block_container_control_unchanged() {
    with_render(
        "<div class=\"c\"><span class=\"t\"></span><span class=\"b\">BTN</span></div>",
        ".c{display:block}.t{display:table}",
        |doc, root| {
            let (cb, cx, cy) = class_box(root, doc, ".c").expect(".c box");
            let (tb, _, _) = class_box(root, doc, ".t").expect(".t box");
            eprintln!(
                "[block-control] .c ({cx},{cy}) {}x{} | .t {}x{}",
                cb.width, cb.height, tb.width, tb.height
            );
            // 块流中空表收缩为 0（css-sizing 空 table 无行），容器仍保住 .b 文本行高
            //——块流记账路径未被 row flex 守卫误伤。
            assert!(
                tb.width.abs() < TOL && tb.height.abs() < TOL,
                "块容器中空表应收缩 0x0，实际 {}x{}",
                tb.width,
                tb.height
            );
            assert!(cb.height > TOL, "容器应保住文本行高，实际 {}", cb.height);
        },
    );
}

/// F8：空 block 子对照——display:block 空 item 语义不受本修复影响。
#[test]
fn row_flex_empty_block_child_control() {
    with_render(
        "<div class=\"c\"><span class=\"t\"></span><span class=\"b\">BTN</span></div>",
        ".c{display:flex}.t{display:block}",
        |doc, root| {
            let (_, _cx, cy) = class_box(root, doc, ".c").expect(".c box");
            let (bb, _, by) = class_box(root, doc, ".b").expect(".b box");
            let y_rel = by - cy;
            assert!(y_rel.abs() < TOL, "兄弟 y_rel 应≈0，实际 {y_rel}");
            assert!(by + bb.height - cy > TOL, "容器交叉高不应塌缩");
        },
    );
}

/// F11：浮动子高度变化不位移 in-flow 兄弟（trending 第三写点：article 首子
/// div.float-right 随内层空表收缩 44→28，旧实现按块流 delta 把 h2/p/footer
/// 整体上移 16 并回填 article 高）。CSS2 §9.4.1（float 盒 out of flow，不参与
/// 块流兄弟堆叠）——修复落在 `reflow_siblings_after_table_height_change` 的
/// float 子早退，与 R2061 abspos 早退同族。
#[test]
fn float_child_height_change_no_sibling_shift() {
    with_render(
        "<div class=\"c\"><div class=\"f\"><span class=\"t\"></span></div><div class=\"b\">TITLE</div></div>",
        ".c{display:block}.f{float:right}.t{display:table}",
        |doc, root| {
            let (_, cx, cy) = class_box(root, doc, ".c").expect(".c box");
            let (fb, _, _) = class_box(root, doc, ".f").expect(".f box");
            let (bb, _, by) = class_box(root, doc, ".b").expect(".b box");
            eprintln!(
                "[float-child] .c ({cx},{cy}) | .f {}x{} | .b y_rel={}",
                fb.width,
                fb.height,
                by - cy
            );
            // 浮动子不占块流空间：无论其高度如何收缩，in-flow 兄弟停在内容顶。
            let y_rel = by - cy;
            assert!(y_rel.abs() < TOL, "浮动子收缩不得位移 in-flow 兄弟，实际 y_rel={y_rel}");
            assert!(by + bb.height - cy > TOL, "容器应保住文本行高");
        },
    );
}

/// F12：浮动子为 display:table 本体（float + display:table 组合）——表收缩同样
/// 不得位移块流兄弟（:272 real-table 调用点经同一 float 早退）。
#[test]
fn floated_table_child_no_sibling_shift() {
    with_render(
        "<div class=\"c\"><span class=\"t\">TX</span><div class=\"b\">TITLE</div></div>",
        ".c{display:block}.t{display:table;float:right}",
        |doc, root| {
            let (_, cy, _) = class_box(root, doc, ".c").expect(".c box");
            let (tb, _, _) = class_box(root, doc, ".t").expect(".t box");
            let (bb, _, by) = class_box(root, doc, ".b").expect(".b box");
            eprintln!("[floated-table] .t {}x{} | .b y_rel={}", tb.width, tb.height, by - cy);
            let y_rel = by - cy;
            assert!(y_rel.abs() < TOL, "浮动表收缩不得位移 in-flow 兄弟，实际 y_rel={y_rel}");
            assert!(by + bb.height - cy > TOL, "容器应保住文本行高");
        },
    );
}
