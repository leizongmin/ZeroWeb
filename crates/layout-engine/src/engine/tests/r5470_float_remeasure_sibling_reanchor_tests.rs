//! R5470：float 排除感知重测后的兄弟重锚（步 6 传播臂）测试。
//!
//! 谱系：WPT `css/CSS2/css1/c5525-fltmrgn-000.xht`（CSS2 §8.3.1 × §10.6.3）。
//!
//! 根因链：taffy 对「float 子 + inline 内容」容器给 collapse-through（行内内容对
//! taffy 块堆叠不可见 → h=0、mt/mb 折链吃掉兄弟 margin）；真实行盒高由步 6 排除
//! 感知 IFC 重测物化，但后续流内兄弟停驻 taffy 旧帧位——「含 float 前驱之后兄弟
//! 竖直 margin 丢失」签名（c5525 p3-p6 累计上移 15/60/75/90px；chromium 探针
//! min-a/min-f 双形态定谳）。修复两件：①§10.6.3 高度门翻默认（普通块 auto 高 =
//! 行盒底，float 溢出不撑高，`ZW_IFC_AUTOHEIGHT_BFC=0` 回退）；②重测后兄弟重锚
//! （首个后继流内兄弟落生长块 margin-box 堆叠位，正差下推，
//! `ZW_FLOAT_REMEASURE_SIB_SHIFT=0` 回退）。
//!
//! 反证不由单测承担：kill-switch flag 走 selected 快照路径，进程内 set_var 不可靠
//! （R3858 教训）——由 corpus OFF 轮覆盖。

use crate::engine::LayoutEngine;

fn compute(html: &str) -> std::sync::Arc<crate::types::LayoutBox> {
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles).root
}

fn find_by<'a>(
    root: &'a crate::types::LayoutBox,
    pred: &dyn Fn(&crate::types::LayoutBox) -> bool,
) -> Option<&'a crate::types::LayoutBox> {
    if pred(root) {
        return Some(root);
    }
    root.children.iter().find_map(|c| find_by(c, pred))
}

fn child_ps(root: &crate::types::LayoutBox) -> Vec<&crate::types::LayoutBox> {
    let div = find_by(root, &|b: &crate::types::LayoutBox| b.width == 122.0).expect("div 盒（w=120+边框 2）");
    div.children.iter().collect()
}

/// min-a 形态（chromium 探针定谳）：p2 含 45px float + 单行文本（行高 15px）。
/// §10.6.3：float 溢出不撑高普通块 → p2 h=15；p3 落 p2 堆叠位 + collapse(15,15)
/// = p2.y + 30。修复前 p2 h=45（float extent 计入）且 p3 贴 p2.y（margin 被折链吃掉）。
#[test]
fn float_overhang_does_not_grow_plain_block_and_sibling_margin_restored() {
    let root = compute(
        r#"<html><body style="margin:0">
      <div style="width:120px; border:1px solid black; font-size:15px; line-height:15px;">
        <p style="margin:15px 0">x</p>
        <p style="margin:15px 0"><span style="float:left; width:45px; height:45px;"></span>x</p>
        <p style="margin:15px 0">x</p>
      </div>
    </body></html>"#,
    );
    let ps = child_ps(&root);
    assert_eq!(ps.len(), 3, "div 应 3 个 p 子");
    let (p1, p2, p3) = (ps[0], ps[1], ps[2]);
    // §10.6.3：p2 auto 高 = 行盒底（15px 单行），45px float 溢出不撑高
    assert!(
        (p2.height - 15.0).abs() < 0.5,
        "p2 高应 15（float 溢出不撑高普通块），实际 {}",
        p2.height
    );
    // 兄弟 margin 恢复：p3.y = p2.y + p2.h + collapse(15,15)
    let gap = p3.y - (p2.y + p2.height);
    assert!(
        (gap - 15.0).abs() < 0.5,
        "p3 与 p2 间应有 15px 折叠 margin，实际 gap={}（修复前=0）",
        gap
    );
    // 前驱对照（无 float）：p1→p2 间距本就正确
    assert!((p2.y - (p1.y + p1.height) - 15.0).abs() < 0.5, "p1→p2 对照间距应 15",);
}

/// min-f 形态（chromium 探针定谳）：p2 含 30px float + 3 行文本（<br> 控行，免字体
/// 度量依赖）。重测物化 p2 h=45 后 p3 须随动：p3.y = p2.y + 45 + 15。修复前
/// p2 h 物化 45 但 p3 停驻 taffy 折链位 p2.y（gap=0）。
#[test]
fn remeasured_height_pushes_following_sibling() {
    let root = compute(
        r#"<html><body style="margin:0">
      <div style="width:120px; border:1px solid black; font-size:15px; line-height:15px;">
        <p style="margin:15px 0">x</p>
        <p style="margin:15px 0"><span style="float:left; width:45px; height:30px;"></span>x<br>x<br>x</p>
        <p style="margin:15px 0">x</p>
      </div>
    </body></html>"#,
    );
    let ps = child_ps(&root);
    assert_eq!(ps.len(), 3, "div 应 3 个 p 子");
    let (p2, p3) = (ps[1], ps[2]);
    assert!(
        (p2.height - 45.0).abs() < 0.5,
        "p2 高应 45（3 行 × 15px，float 30px 短于文本不撑高），实际 {}",
        p2.height
    );
    let gap = p3.y - (p2.y + p2.height);
    assert!(
        (gap - 15.0).abs() < 0.5,
        "重测撑高后 p3 应落 p2 堆叠位 + 15px margin，实际 gap={}（修复前=0）",
        gap
    );
}
