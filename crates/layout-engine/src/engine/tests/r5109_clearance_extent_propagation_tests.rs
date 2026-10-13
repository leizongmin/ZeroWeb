//! R5109：clearance 修正后的 extent 缩减量祖先传播 + 空块末链自折叠剔除测试。
//!
//! 谱系：WPT `css/CSS2/floats-clear/margin-collapse-157.xht`（CSS2 §8.3.1 + §9.5.2 +
//! §10.6.3）、`margin-collapse-142.xht`。
//!
//! 根因：taffy 不建模 clearance——被正 clearance 吸收的 cleared 子 margin（§8.3.1
//! 「child has clearance」阻止与父 top 折叠）与 containment 收缩后的容器高，在 taffy
//! 祖先盒高度里仍按修正前的子 margin-box extent 计算。R3808 撤销（mt/y 归位）与
//! containment 收缩只修本层，祖先残留泄漏段：
//! - 157 row1：.test 内容高多 16/32px（.test 红底在 lime 下方露出）；
//! - 157 row2：.clear 内 .empty{margin:1em} 自折叠组被计入 .clear ch=16（红条纹），
//!   且 .clear 的折叠组（声明 mt 0 + empty 16）未被 R3808 签名覆盖（子 mt 被自身
//!   子树膨胀 → 撤销被跳过）→ container mt=16 lime 下移露红；
//! - 142：.clear 的 margin 链收缩后 td 内容高多 64px 露红底。
//!
//! 修复三件：①R5109 记录臂——containment 收缩臂与 R3808 两撤销点把缩减量记入
//! `clearance_extent_reduction`；②传播臂——`shift_siblings_after_clearance_containment`
//! 消费该值（后续兄弟上移 + 祖先缩高，与 R1319 子树缩减同构）；③剔除臂——
//! `strip_empty_only_child_self_collapse`：唯一 in-flow 子为空块 + block-end 框边的
//! auto 高容器，末链自折叠 margin 不计入容器高（§8.3.1 自折叠组在 top 侧消耗）。

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

/// 157 row1-b 形态：test(border) > container > float + clear(mt:16, border-bottom:64)。
/// 断言：.test/.container 内容高 128（无泄漏段）、.clear 位于 float 底（clearance 吸收 mt）。
#[test]
fn test_r5109_157_row1_cleared_mt_not_stale_in_ancestor() {
    let root = compute(
        r#"<html><body style="margin:0">
      <div id="test" style="border:3px solid black; background:red; width:128px;">
        <div id="container" style="background:lime;">
          <div style="float:left; height:64px; width:64px; background:aqua"></div>
          <div style="clear:left; border-bottom:64px solid yellow; background:red; margin-top:16px;"></div>
        </div>
      </div>
    </body></html>"#,
    );
    let test = find_by(&root, &|b| b.width == 134.0).expect("test 盒");
    let container = &test.children[0];
    let clear = container
        .children
        .iter()
        .find(|c| c.height == 64.0 && c.content_height == 0.0)
        .expect("clear 盒");
    // 修复前：test/ch=150/144（泄漏 16 红段）；修复后 = 128 + 边框 3×2。
    assert!(
        (test.content_height - 128.0).abs() < 0.5,
        "test ch={}",
        test.content_height
    );
    assert!(
        (container.content_height - 128.0).abs() < 0.5,
        "container ch={}",
        container.content_height
    );
    // .clear border-box 位于 float 底（mt 16 被 clearance 吸收，不推自身）。
    assert!((clear.y - 64.0).abs() < 0.5, "clear y={}", clear.y);
}

/// 157 row2-a 形态：test(border) > container > float + clear(border-bottom:64) > empty(margin:16)。
/// 断言：container mt=0（折叠组被 clearance 阻断于父边界，撤销签名容忍子 mt 子树膨胀）、
/// .clear ch=0 h=64（空块末链自折叠不计入）、.test ch=128。
#[test]
fn test_r5109_157_row2_empty_self_collapse_stripped() {
    let root = compute(
        r#"<html><body style="margin:0">
      <div id="test" style="border:3px solid black; background:red; width:128px;">
        <div id="container" style="background:lime;">
          <div style="float:left; height:64px; width:64px; background:aqua"></div>
          <div style="clear:left; border-bottom:64px solid yellow; background:red;">
            <div style="margin:16px; background:red;"></div>
          </div>
        </div>
      </div>
    </body></html>"#,
    );
    let test = find_by(&root, &|b| b.width == 134.0).expect("test 盒");
    let container = &test.children[0];
    let clear = &container.children[1];
    // 修复前：container mt=16（lime 下移 16 露红）+ clear ch=16 h=80（红条纹）。
    assert!(
        container.margin_top.abs() < 0.5,
        "container mt={}",
        container.margin_top
    );
    assert!((clear.content_height).abs() < 0.5, "clear ch={}", clear.content_height);
    assert!((clear.height - 64.0).abs() < 0.5, "clear h={}", clear.height);
    assert!(
        (test.content_height - 128.0).abs() < 0.5,
        "test ch={}",
        test.content_height
    );
}

/// 142 形态：test(border) > container > float + clear(m:64 0, h:64) + block(mt:16, border:32)。
/// 断言：test 内容高 256（修复前 320——clear margin 链收缩后 td 残留 64 红段）、
/// block 位于 collapse(clear mb 64, block mt 16)=64 折叠段之后（y=192）。
#[test]
fn test_r5109_142_margin_chain_shrink_propagates_to_ancestor() {
    let root = compute(
        r#"<html><body style="margin:0">
      <div id="test" style="border:3px solid black; background:red; width:128px;">
        <div id="container" style="background:lime; width:128px;">
          <div style="float:left; height:64px; width:64px; background:aqua"></div>
          <div style="clear:left; background:yellow; margin:64px 0; height:64px;"></div>
          <div style="margin:16px 0 0; border:32px solid #AAFFAA;"></div>
        </div>
      </div>
    </body></html>"#,
    );
    let test = find_by(&root, &|b| b.width == 134.0).expect("test 盒");
    let container = &test.children[0];
    let block = &container.children[2];
    assert!(
        (test.content_height - 256.0).abs() < 0.5,
        "test ch={}",
        test.content_height
    );
    assert!(
        (container.content_height - 256.0).abs() < 0.5,
        "container ch={}",
        container.content_height
    );
    assert!((block.y - 192.0).abs() < 0.5, "block y={}", block.y);
}

/// 负例：无 block-end 框边时剔除臂不触发（margin 折叠穿出属 collapse-through 域，
/// 父内容高保持 0；clearance_extent_reduction 不记录）。
#[test]
fn test_r5109_no_bottom_edge_keeps_collapse_through() {
    let root = compute(
        r#"<html><body style="margin:0">
      <div id="parent" style="background:lime;">
        <div style="margin:16px 0;"></div>
      </div>
    </body></html>"#,
    );
    // #parent：body 下唯一内容盒（width 800、恰一个空块子）。body 自身有 anon+parent
    // 两个子、html 有 3 个子，均被 children.len()==1 排除。
    let parent = find_by(&root, &|b| b.width == 800.0 && b.children.len() == 1).expect("parent 盒");
    assert!(parent.content_height.abs() < 0.5, "ch={}", parent.content_height);
    assert!(
        parent.clearance_extent_reduction.abs() < 0.5,
        "reduction={}",
        parent.clearance_extent_reduction
    );
}
