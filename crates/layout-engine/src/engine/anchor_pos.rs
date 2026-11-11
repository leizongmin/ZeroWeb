//! CSS Anchor Positioning 布局期（wptfyi-css 组F 布局刀2）：position-area
//! 区域定位 pass——锚注册收集（`anchor-name` → 视口 border-box 几何）+
//! anchored 元素 3×3 网格区域定位（htb 写作模式第一刀）。
//!
//! 与 abspos.rs 修正 pass 同架构：taffy 主布局后的 LayoutBox 树自递归 walk。
//! 两段式：先全树收集锚几何（树序后者覆盖——`position-anchor` 默认锚取
//! preceding 树序最后匹配，css-anchor-position-1 §position-anchor），再自顶
//! 向下应用（先外后内——嵌套 anchored 链外层先定型）。区域格解析复用
//! `css_parser::values::anchor_area::position_area_cells_dir`（CSSOM 声明面与
//! 布局共用单实现；矩阵下沉 css-parser——layout-engine 不依赖 zero-engine）。
//!
//! 第一刀边界（挂账）：
//! - `position-anchor` 缺失（无显式锚名）不介入（默认锚树序规则待后续刀）；
//! - anchored 元素需直挂 positioned CB 盒（CB 链非直挂场景不介入）；
//! - align/justify-self 非 stretch 的区域对齐蕴含不建模（格满填语义）；
//! - anchored 显式 width/height 或 inset 声明时不覆写尺寸（仅定位）；
//! - `position-try-fallbacks`（keyword tactic / @position-try）与 last-successful
//!   position 未建模：任一 fallback 声明不介入（静态位保持基线行为——语料实证
//!   介入净 -5：内置 flip 需布局级区域溢出检查 + last-successful 状态机，custom
//!   `--pt` 需 shim 期 fallback 应用引擎；fallback 面另刀）；
//! - self 轴按 anchored 元素自身 direction 解析（rtl 镜像 inline 格）；writing-
//!   mode 非 htb 元素与 CB 侧逻辑轴映射（anchor-position-writing-modes 族）另刀。
//!
//! env `ZW_ANCHOR_AREA=0` kill-switch（LazyLock 进程级，递归 pass 热路径不逐
//! 节点读 env；R5465 同型）。
//!
//! https://drafts.csswg.org/css-anchor-position-1/#position-area

use std::collections::HashMap;
use std::sync::LazyLock;

use zero_dom::NodeId;
use zero_style_system::ComputedStyle;

use crate::types::LayoutBox;

static ANCHOR_AREA_ENABLED: LazyLock<bool> = LazyLock::new(|| std::env::var("ZW_ANCHOR_AREA").as_deref() != Ok("0"));

/// 锚名 → 视口 border-box 几何 (x, y, w, h)。树序后者覆盖（默认锚语义）。
type AnchorMap = HashMap<String, (f32, f32, f32, f32)>;

/// position-area 区域定位 pass（engine.rs 11.7c 挂点）。
pub(super) fn apply_anchor_position_area(root: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
    if !*ANCHOR_AREA_ENABLED {
        return;
    }
    // pass1：锚注册收集。无锚页零成本跳过（区域 pass 依赖锚几何）。
    let mut anchors: AnchorMap = HashMap::new();
    collect_anchors(root, 0.0, 0.0, styles, &mut anchors);
    if anchors.is_empty() {
        return;
    }
    // pass2：自顶向下应用（外层 anchored 先定型，嵌套链内层用已定几何）。
    apply_walk(root, 0.0, 0.0, styles, &anchors);
}

fn collect_anchors(
    b: &LayoutBox,
    origin_x: f32,
    origin_y: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
    out: &mut AnchorMap,
) {
    let vx = origin_x + b.x;
    let vy = origin_y + b.y;
    if let Some(nid) = b.node_id
        && let Some(s) = styles.get(&nid)
        && let Some(names) = s.anchor_name.as_deref()
    {
        for name in names.split(',') {
            let name = name.trim();
            if !name.is_empty() {
                out.insert(name.to_string(), (vx, vy, b.width, b.height));
            }
        }
    }
    for c in &b.children {
        collect_anchors(c, vx, vy, styles, out);
    }
}

fn apply_walk(
    b: &mut LayoutBox,
    origin_x: f32,
    origin_y: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
    anchors: &AnchorMap,
) {
    let vx = origin_x + b.x;
    let vy = origin_y + b.y;
    // CB 几何（padding box——abspos containing block 语义，CSS2 §10.1）。
    // 仅当本盒是 positioned CB（直挂场景）才解析区域定位；非 positioned 中间层
    // 的 anchored 子（CB 在上层）第一刀不介入。
    let cb_is_positioned = b.is_abspos_cb;
    let cb_x = vx + b.border_left + b.padding_left;
    let cb_y = vy + b.border_top + b.padding_top;
    let cb_w = (b.width - b.border_left - b.border_right - b.padding_left - b.padding_right).max(0.0);
    let cb_h = (b.height - b.border_top - b.border_bottom - b.padding_top - b.padding_bottom).max(0.0);
    for child in &mut b.children {
        let style = child.node_id.and_then(|nid| styles.get(&nid));
        if cb_is_positioned
            && child.is_absolute
            && let Some(s) = style
            // position-try fallback 机制未建模（§position-try-fallbacks）：任一
            // fallback 声明不介入（见模块头挂账——语料实证介入净 -5）。
            && s.position_try_fallbacks.is_none()
            && let Some(area) = s.position_area.as_deref()
            && let Some(((rs, re), (cs, ce))) =
                zero_css_parser::values::anchor_area::position_area_cells_dir(
                    area,
                    matches!(s.direction, zero_style_system::DirectionValue::Rtl),
                )
        {
            // 锚解析：position-anchor 名原文匹配（anchor-name 列表项，含 -- 前缀）。
            let anchor = s.position_anchor.as_deref().and_then(|pa| anchors.get(pa));
            if let Some(&(ax, ay, aw, ah)) = anchor {
                // 网格四线（相对 CB padding box；锚边越界允许负值——spec 区域
                // 网格线 = 锚 border box 各边在 CB 坐标投影）。
                let col = [0.0f32, ax - cb_x, ax + aw - cb_x, cb_w];
                let row = [0.0f32, ay - cb_y, ay + ah - cb_y, cb_h];
                let rx = col[cs as usize];
                let ry = row[rs as usize];
                let rw = (col[ce as usize] - rx).max(0.0);
                let rh = (row[re as usize] - ry).max(0.0);
                // 目标视口 = cb 原点 + 区域左上；转父相对坐标（减父视口原点）。
                // abspos margin 参与（CSS2 §10.3.7——margin box 对齐区域，
                // margin 非 auto 时 border box 内缩）。
                let font = &s.font_size;
                let ml = resolve_anchor_len(&s.margin_left, font);
                let mr = resolve_anchor_len(&s.margin_right, font);
                let mt = resolve_anchor_len(&s.margin_top, font);
                let mb = resolve_anchor_len(&s.margin_bottom, font);
                child.x = cb_x + rx - vx + ml;
                child.y = cb_y + ry - vy + mt;
                // stretch 尺寸：显式 width/height 声明不覆写（区域只定位）；
                // 未声明时格满填（WPT basic 显式 align/justify-self:stretch 同形）。
                if matches!(s.width, zero_css_parser::values::LengthValue::Auto) {
                    let bw = (rw - ml - mr).max(0.0);
                    child.width = bw;
                    child.content_width =
                        (bw - child.border_left - child.border_right - child.padding_left - child.padding_right)
                            .max(0.0);
                }
                if matches!(s.height, zero_css_parser::values::LengthValue::Auto) {
                    let bh = (rh - mt - mb).max(0.0);
                    child.height = bh;
                    child.content_height =
                        (bh - child.border_top - child.border_bottom - child.padding_top - child.padding_bottom)
                            .max(0.0);
                }
            }
        }
        // 递归（child 几何已定 → 子树 origin 正确；嵌套 anchored 链外先内后）。
        apply_walk(child, vx, vy, styles, anchors);
    }
}

/// 长度/margin 解析（区域定位 margin 内缩用；%/auto margin 记 0——与 abspos
/// 静态位 margin 口径一致）。委托 `resolve_abspos_real_length`（engine 既有
/// 求值口径，calc/em/rem 全支持）。
fn resolve_anchor_len(
    v: &zero_css_parser::values::LengthValue,
    font_size: &zero_css_parser::values::LengthValue,
) -> f32 {
    super::resolve_abspos_real_length(v, font_size, 0.0, 0.0).unwrap_or(0.0)
}
