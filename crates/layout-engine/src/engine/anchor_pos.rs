//! CSS Anchor Positioning 布局期（wptfyi-css 组F 布局刀2 + 刀3）：position-area
//! 区域定位 pass——锚注册收集（`anchor-name` → 视口 border-box 几何）+
//! anchored 元素 3×3 网格区域定位（htb 写作模式第一刀）；刀3 增 `anchor()`
//! inset 函数求值（物理边 + center + 百分比 + fallback，物理轴第一刀）。
//!
//! 与 abspos.rs 修正 pass 同架构：taffy 主布局后的 LayoutBox 树自递归 walk。
//! 两段式：先全树收集锚几何（树序后者覆盖——`position-anchor` 默认锚取
//! preceding 树序最后匹配，css-anchor-position-1 §position-anchor），再自顶
//! 向下应用（先外后内——嵌套 anchored 链外层先定型）。区域格解析复用
//! `css_parser::values::anchor_area::position_area_cells_wm`（CSSOM 声明面与
//! 布局共用单实现；矩阵下沉 css-parser——layout-engine 不依赖 zero-engine；
//! CB/anchored 双坐标系写作模式/方向逻辑轴映射，g35 wm 面）。
//!
//! 第一刀边界（挂账）：
//! - anchor() 逻辑边关键字（start/end/self-*，需锚元素写作模式映射）不介入
//!   （回落 fallback/槽位既有行为）；anchor-size() 尺寸槽/inset 槽求值已接
//!   （§4 轴映射，见 `resolve_anchor_size_raw`；margin 槽承载挂账——gCS 双
//!   通道布线另刀）；
//! - 无名 anchor() 默认锚取 `position-anchor` 值；缺省（spec 树序 preceding
//!   默认锚规则，§position-anchor）不介入；
//! - `position-anchor` 缺失（无显式锚名）区域定位不介入（默认锚树序规则待
//!   后续刀）；
//! - anchored 元素需直挂 positioned CB 盒（CB 链非直挂场景不介入）；
//! - 纯 inline 流锚盒（display:inline，span 等）静态形/片段化几何不可靠，
//!   pass3 求值不介入（回落 fallback/槽位既有行为）；inline-block 原子盒
//!   border box 可靠不受此门；position-area pass 不受此门（语料无回归证据，
//!   第一刀不扩面）；
//! - 引擎不建模 scroll offset（盒坐标是布局静态位，滚动是 paint 层变换）：
//!   锚滚动上下文（overflow scroll/auto 祖先）与目标不一致时不做 inset 求值
//!   （锚随滚移而目标不动 → 坐标系失配；同滚上下文内两侧坐标同基可求值）。
//!   FIXME: 锚几何按滚动上下文折算（scroll offset 感知）后撤门；
//! - 对齐面（g35）：区域默认表（§4.1——仅 start 轨→end、仅 end 轨→start、
//!   仅 center 轨→center、span-all→anchor-center）+ 显式 align/justify-self
//!   （stretch 满填、物理与逻辑关键字折算）；区域即对齐容器（inset 属性不
//!   介入——A/B 语料实证单独建模 inset-IMCB 破 in-position-try 64P 等值页：
//!   等值断言要求 fallback 声明下 target/ref 同几何，需 fallback×inset 联动
//!   建模才可撤，挂账）；挂账：§4.1 单 auto inset 例外、baseline 族对齐按
//!   区域默认表（FIXME）、safe/unsafe 溢出安全折让与 anchor-center 溢出安全
//!   折让（css-align overflow-shift）未建模（FIXME）；
//! - CSSOM 声明面挂账：`element.style.positionArea` 不通（runner/宿主未注册
//!   position-area CSSOM 属性），position-area-basic/align-justify/wm-dir 族
//!   ~161F 在声明层即全红（expected offsetWidth 400 but got 0——元素未被区域
//!   定位拦截），布局面对齐/网格改动无法翻绿这些页，声明面支持另刀；
//! - anchored 显式 width/height 或 inset 声明时不覆写尺寸（仅定位）；
//! - `position-try-fallbacks`（keyword tactic / @position-try）与 last-successful
//!   position 未建模：任一 fallback 声明不介入（静态位保持基线行为——语料实证
//!   介入净 -5：内置 flip 需布局级区域溢出检查 + last-successful 状态机，custom
//!   `--pt` 需 shim 期 fallback 应用引擎；fallback 面另刀）；
//! - 区域格 wm 面（g35）：plain logical 关键字按 CB 写作模式、self-* 按
//!   anchored 元素自身写作模式（css-parser AreaWmDir 双坐标系单实现）；
//!   物理 anchor() 边求值与写作模式正交（pass3 语料实证面）。
//!
//! env `ZW_ANCHOR_AREA=0` kill-switch（LazyLock 进程级，递归 pass 热路径不逐
//! 节点读 env；R5465 同型）。
//!
//! https://drafts.csswg.org/css-anchor-position-1/#position-area
//! https://drafts.csswg.org/css-anchor-position-1/#anchor-pos

use std::collections::HashMap;
use std::sync::LazyLock;

use zero_dom::NodeId;
use zero_style_system::ComputedStyle;

use crate::types::LayoutBox;

static ANCHOR_AREA_ENABLED: LazyLock<bool> = LazyLock::new(|| std::env::var("ZW_ANCHOR_AREA").as_deref() != Ok("0"));

/// 锚几何记录（视口 border-box + pass3 介入门标记）。
#[derive(Clone, Copy)]
struct AnchorGeo {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    /// 锚为纯 inline 流盒（display:inline——静态形/片段化几何不可靠，pass3
    /// 不介入；inline-block 原子盒 border box 可靠不受此门——见模块头挂账）。
    inline_flow: bool,
    /// 锚处于滚动容器内（overflow scroll/auto 严格祖先；锚自身是滚动容器不
    /// 改自身 border box 位置，不计入）——与目标上下文失配时 pass3 不介入。
    in_scroller: bool,
}

/// 锚名 → 锚几何。树序后者覆盖（默认锚语义）。pass2 区域定位只读坐标字段
///（不行内/滚上下文门——无回归证据不扩面）；pass3 经 lookup 门消费。
type AnchorMap = HashMap<String, AnchorGeo>;

/// 盒自身是否滚动容器（overflow scroll/auto——可滚移祖先改变子树渲染坐标；
/// hidden/clip 无用户滚移且语料无 JS 滚移证据，第一刀不计入）。
fn is_scroller(s: &ComputedStyle) -> bool {
    use zero_css_parser::values::OverflowValue;
    matches!(s.overflow_x, OverflowValue::Scroll | OverflowValue::Auto)
        || matches!(s.overflow_y, OverflowValue::Scroll | OverflowValue::Auto)
}

/// pass3 锚查找（介入门统一口径：纯 inline 流锚与滚动上下文失配过滤为无锚
/// → 回落 fallback/槽位既有行为；显式名与 position-anchor 默认锚同消费）。
fn gated_lookup(anchors: &AnchorMap, n: &str, target_sc: bool) -> Option<AnchorGeo> {
    anchors
        .get(n)
        .filter(|g| !g.inline_flow && g.in_scroller == target_sc)
        .copied()
}

/// position-area 区域定位 pass（engine.rs 11.7c 挂点）。
pub(super) fn apply_anchor_position_area(root: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
    if !*ANCHOR_AREA_ENABLED {
        return;
    }
    // pass1：锚注册收集。无锚且无 anchor() inset 声明页零成本跳过（区域
    // pass 依赖锚几何；inset pass 无锚时仍需走 fallback 求值）。
    let mut anchors: AnchorMap = HashMap::new();
    let mut has_anchor_insets = false;
    collect_anchors(root, 0.0, 0.0, false, styles, &mut anchors, &mut has_anchor_insets);
    if anchors.is_empty() && !has_anchor_insets {
        return;
    }
    // pass2：自顶向下应用（外层 anchored 先定型，嵌套链内层用已定几何）。
    apply_walk(root, 0.0, 0.0, styles, &anchors);
    // pass3：anchor() inset 求值（刀3）——同 walk 形，区域定位与 inset 求值
    // 互斥门（position_area 已介入的元素不再覆写几何）。
    apply_inset_walk(root, 0.0, 0.0, false, styles, &anchors);
}

// ── pass3：anchor() inset 求值（css-anchor-position-1 §anchor-pos）───────

/// 锚名 → 锚几何查找（含 position-anchor 默认锚兜底的调用侧闭包形态；行内/
/// 滚上下文介入门在闭包内统一生效——显式名与默认锚同口径）。
type AnchorLookup<'a> = dyn Fn(&str) -> Option<AnchorGeo> + 'a;

/// anchor-size() 求值上下文（clippy too_many_arguments 收参）：默认锚几何、
/// 轴映射写作模式基（inline/block 按 CB、self-* 按目标）与 fallback 长度
/// 基字号。
struct AnchorSizeCtx<'a> {
    default_geo: Option<AnchorGeo>,
    cb_vertical: bool,
    self_vertical: bool,
    font_size: &'a zero_css_parser::values::LengthValue,
}

/// anchor() 原文 → CB 坐标系 px 求值。`slot`：求值 inset 属性名（left/right/
/// top/bottom）——物理轴与缺省边（bare `anchor(--a1)` 缺 side 时按槽属性取
/// 对应边，§anchor-pos "If the <anchor-side> is omitted..."）由此派生；
/// `cb_size` = 槽轴向 CB padding box 尺寸（纯百分比/fallback 百分比基）。
/// 物理边 + center + 纯百分比 + fallback 第一刀；逻辑边（start/end/self-*，
/// 需锚写作模式映射）/表外形/锚缺失且无 fallback → None（槽位保持 Auto 既有
/// 布局行为）。
/// https://drafts.csswg.org/css-anchor-position-1/#anchor-pos
fn resolve_anchor_inset(
    raw: &str,
    slot: &str,
    cb_size: f32,
    lookup: &AnchorLookup,
    default_geo: Option<AnchorGeo>,
    cb_origin: (f32, f32),
    font_size: &zero_css_parser::values::LengthValue,
) -> Option<f32> {
    let axis = matches!(slot, "top" | "bottom");
    let inner = raw.strip_prefix("anchor(")?.strip_suffix(')')?;
    let (head, fallback) = split_top_level_comma(inner);
    let mut tokens = head.split_whitespace();
    let mut name: Option<&str> = None;
    let mut side: Option<&str> = None;
    let mut pct: Option<f32> = None;
    for tok in tokens.by_ref() {
        if tok.starts_with("--") {
            if name.is_some() {
                return None;
            }
            name = Some(tok);
        } else if tok.ends_with('%') && pct.is_none() {
            pct = tok.strip_suffix('%')?.parse::<f32>().ok();
        } else if matches!(tok, "top" | "bottom" | "left" | "right" | "center") && side.is_none() {
            side = Some(tok);
        } else {
            // 逻辑边（start/end/self-*）/表外 token/超量 token：第一刀不介入。
            return None;
        }
    }
    if name.is_none() && side.is_none() && pct.is_none() {
        return None;
    }
    // 锚缺失 → 回落 fallback（fallback 段仍在尾部求值，不因锚缺失短路）。
    let geo = match name {
        Some(n) => lookup(n),
        None => default_geo,
    };
    let coord = geo.map(|g| {
        let (ax, ay, aw, ah) = (g.x, g.y, g.w, g.h);
        // 物理/center 边坐标（CB 相对，自左/上缘量）；pct-only 沿槽轴在对面
        // 边对间插值；物理边与槽轴错配（left 于 top 槽）取该物理边坐标但
        // 不做槽向 inset 换算（跨轴坐标语义未定义面，第一刀原样）。
        let (e0, e1, span) = if axis {
            (ay - cb_origin.1, ay + ah - cb_origin.1, ah)
        } else {
            (ax - cb_origin.0, ax + aw - cb_origin.0, aw)
        };
        let (v, along_axis) = match (side, pct) {
            (Some(s), _) => match s {
                "left" => (if axis { ax - cb_origin.0 } else { e0 }, !axis),
                "right" => (if axis { ax + aw - cb_origin.0 } else { e1 }, !axis),
                "top" => (if axis { e0 } else { ay - cb_origin.1 }, axis),
                "bottom" => (if axis { e1 } else { ay + ah - cb_origin.1 }, axis),
                _ => (e0 + span / 2.0, true), // center
            },
            (None, Some(p)) => (e0 + p / 100.0 * span, true),
            // bare 形（只有锚名）：缺省边 = 槽属性同名边。
            (None, None) => (
                match slot {
                    "right" | "bottom" => e1,
                    _ => e0,
                },
                true,
            ),
        };
        // right/bottom 槽 inset 从对面 CB 边缘量起：轴内坐标 → inset 换算。
        if along_axis && matches!(slot, "right" | "bottom") {
            cb_size - v
        } else {
            v
        }
    });
    if let Some(v) = coord {
        return Some(v);
    }
    // fallback：<length> | <percentage>（槽轴向 CB 基）。auto/表外 → None。
    let fb = fallback?.trim();
    if let Some(p) = fb.strip_suffix('%') {
        return p.parse::<f32>().ok().map(|p| p / 100.0 * cb_size);
    }
    match zero_css_parser::values::parse_length(fb) {
        Some(zero_css_parser::values::LengthValue::Auto) | None => None,
        Some(v) => Some(resolve_anchor_len(&v, font_size)),
    }
}

/// 顶层逗号拆分（忽略嵌套函数内逗号——fallback 可含 calc()）。
fn split_top_level_comma(s: &str) -> (&str, Option<&str>) {
    let mut depth = 0i32;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => return (&s[..i], Some(&s[i + 1..])),
            _ => {}
        }
    }
    (s, None)
}

fn apply_inset_walk(
    b: &mut LayoutBox,
    origin_x: f32,
    origin_y: f32,
    within_scroller: bool,
    styles: &HashMap<NodeId, ComputedStyle>,
    anchors: &AnchorMap,
) {
    let vx = origin_x + b.border_left + b.x;
    let vy = origin_y + b.border_top + b.y;
    // 子树滚动上下文（目标侧 = 盒树祖先链含本盒；锚侧注册口径一致）。
    let child_sc = within_scroller || b.node_id.and_then(|nid| styles.get(&nid)).is_some_and(is_scroller);
    let cb_is_positioned = b.is_abspos_cb;
    // vx/vy 已是 V 系 border box 原点（rect 提取/渲染递归口径：V(child) =
    // V(parent) + parent border + child.x）→ CB padding box 原点只加 padding。
    let cb_x = vx + b.padding_left;
    let cb_y = vy + b.padding_top;
    let cb_w = (b.width - b.border_left - b.border_right - b.padding_left - b.padding_right).max(0.0);
    let cb_h = (b.height - b.border_top - b.border_bottom - b.padding_top - b.padding_bottom).max(0.0);
    for child in &mut b.children {
        let style = child.node_id.and_then(|nid| styles.get(&nid));
        if cb_is_positioned
            && child.is_absolute
            && let Some(s) = style
            && s.position_area.is_none()
            && s.position_try_fallbacks.is_none()
        {
            // 介入门统一在 lookup（gated_lookup）：纯 inline 流锚与滚动上下文
            // 失配（锚滚移而目标不动 → 坐标系失配）过滤为无锚 → 回落 fallback/
            // 槽位既有行为（显式名与 position-anchor 默认锚同口径）。
            let lookup = |n: &str| gated_lookup(anchors, n, child_sc);
            let default_geo = s
                .position_anchor
                .as_deref()
                .and_then(|n| gated_lookup(anchors, n, child_sc));
            // anchor-size() 轴映射写作模式基：CB = 本盒（inset/尺寸槽
            // inline/block 关键字），目标 = 子盒 style（self-* 关键字）。
            let cb_vertical = b
                .node_id
                .and_then(|nid| styles.get(&nid))
                .is_some_and(|cb| cb.writing_mode.is_vertical_block_flow());
            let self_vertical = s.writing_mode.is_vertical_block_flow();
            let size_ctx = AnchorSizeCtx {
                default_geo,
                cb_vertical,
                self_vertical,
                font_size: &s.font_size,
            };
            let resolve = |raw: &str, slot: &str, cb_size: f32| {
                if raw.starts_with("anchor-size(") {
                    // anchor-size() inset 槽：求值结果即 inset 量（无 anchor()
                    // 坐标/对边换算语义——§4 求值产出 <length> 直接入槽）。
                    resolve_anchor_size_raw(raw, slot, cb_size, &lookup, &size_ctx)
                } else {
                    resolve_anchor_inset(raw, slot, cb_size, &lookup, default_geo, (cb_x, cb_y), &s.font_size)
                }
            };
            let l = s.anchor_inset_left.as_deref().and_then(|r| resolve(r, "left", cb_w));
            let rr = s.anchor_inset_right.as_deref().and_then(|r| resolve(r, "right", cb_w));
            let t = s.anchor_inset_top.as_deref().and_then(|r| resolve(r, "top", cb_h));
            let bt = s
                .anchor_inset_bottom
                .as_deref()
                .and_then(|r| resolve(r, "bottom", cb_h));
            let ml = resolve_anchor_len(&s.margin_left, &s.font_size);
            let mr = resolve_anchor_len(&s.margin_right, &s.font_size);
            let mt = resolve_anchor_len(&s.margin_top, &s.font_size);
            let mb = resolve_anchor_len(&s.margin_bottom, &s.font_size);
            // inset→几何：inset 定 margin edge（CSS2 §10.3.7）；双侧非 auto
            // 拉伸定尺寸，单侧定位（尺寸取 taffy shrink-to-fit 结果）。lv 已是
            // V 系 CB padding box 相对量 → child.x（rect 口径同基准）直写。
            if let Some(lv) = l {
                child.x = lv + ml;
                if let Some(rv) = rr {
                    // 双侧非 auto：宽 = CB − 两 inset − 水平 margin（inset 各自
                    // 从对面 CB 边缘量起，CSS2 §10.3.7 约束式）。
                    let w = (cb_w - lv - rv - ml - mr).max(0.0);
                    child.width = w;
                    child.content_width =
                        (w - child.border_left - child.border_right - child.padding_left - child.padding_right)
                            .max(0.0);
                }
            } else if let Some(rv) = rr {
                child.x = cb_w - rv - mr - child.width;
            }
            if let Some(tv) = t {
                child.y = tv + mt;
                if let Some(bv) = bt {
                    let h = (cb_h - tv - bv - mt - mb).max(0.0);
                    child.height = h;
                    child.content_height =
                        (h - child.border_top - child.border_bottom - child.padding_top - child.padding_bottom)
                            .max(0.0);
                }
            } else if let Some(bv) = bt {
                child.y = cb_h - bv - mb - child.height;
            }
            // ── anchor-size() 尺寸槽求值（css-anchor-position-1 §4 used 值面）──
            // 解析面无锚几何 fail-closed（槽位存 Auto），此处按锚几何回填 used
            // 宽高。显式尺寸与 inset 双侧拉伸并存时显式尺寸胜（后置覆盖）；
            // replaced 元素按固有比/aspect-ratio 均匀缩放（css-anchor-1 §4
            // "Specifying the width or the height should scale uniformly"——
            // anchor-size-replaced-001 钉面）；min/max 先 min 后 max（CSS2
            // §10.4 约束表次序）。margin 槽承载挂账（FIXME: gCS 读出静态路径
            // GENERATION_CACHE 级联无布局几何，见 ComputedStyle::anchor_size_slots）。
            if let Some(slots) = s.anchor_size_slots.as_ref() {
                let resolve_size = |raw: &str, slot: &str, cb_size: f32| {
                    resolve_anchor_size_raw(raw, slot, cb_size, &lookup, &size_ctx)
                };
                let spec_w = slots.width.as_deref().and_then(|r| resolve_size(r, "width", cb_w));
                let spec_h = slots.height.as_deref().and_then(|r| resolve_size(r, "height", cb_h));
                let spec_min_w = slots
                    .min_width
                    .as_deref()
                    .and_then(|r| resolve_size(r, "min-width", cb_w));
                let spec_min_h = slots
                    .min_height
                    .as_deref()
                    .and_then(|r| resolve_size(r, "min-height", cb_h));
                let spec_max_w = slots
                    .max_width
                    .as_deref()
                    .and_then(|r| resolve_size(r, "max-width", cb_w));
                let spec_max_h = slots
                    .max_height
                    .as_deref()
                    .and_then(|r| resolve_size(r, "max-height", cb_h));
                let set_w = |c: &mut LayoutBox, v: f32| {
                    c.width = v;
                    c.content_width = (v - c.border_left - c.border_right - c.padding_left - c.padding_right).max(0.0);
                };
                let set_h = |c: &mut LayoutBox, v: f32| {
                    c.height = v;
                    c.content_height = (v - c.border_top - c.border_bottom - c.padding_top - c.padding_bottom).max(0.0);
                };
                if spec_w.is_none()
                    && spec_h.is_none()
                    && spec_min_w.is_none()
                    && spec_min_h.is_none()
                    && spec_max_w.is_none()
                    && spec_max_h.is_none()
                {
                    // 无任何槽解析成功（锚缺失全页/未介入形）→ 保持既有布局行为。
                } else if child.is_replaced {
                    let cur_w = child.width;
                    let cur_h = child.height;
                    // 有效比：aspect-ratio 属性优先（语料 t11/t12），否则当前
                    // used 尺寸固有比（natural ratio——img 16×16 = 1）。
                    let ratio = s
                        .aspect_ratio
                        .filter(|r| *r > 0.0)
                        .unwrap_or_else(|| if cur_h > 0.0 { cur_w / cur_h } else { 1.0 });
                    let mut w = spec_w.unwrap_or_else(|| spec_h.map(|h| h * ratio).unwrap_or(cur_w));
                    let mut h = spec_h.unwrap_or_else(|| spec_w.map(|ww| ww / ratio).unwrap_or(cur_h));
                    if let Some(m) = spec_min_w.filter(|m| *m > w) {
                        let scale = m / w;
                        w = m;
                        h *= scale;
                    }
                    if let Some(m) = spec_min_h.filter(|m| *m > h) {
                        let scale = m / h;
                        h = m;
                        w *= scale;
                    }
                    if let Some(m) = spec_max_w.filter(|m| *m < w) {
                        let scale = m / w;
                        w = m;
                        h *= scale;
                    }
                    if let Some(m) = spec_max_h.filter(|m| *m < h) {
                        let scale = m / h;
                        h = m;
                        w *= scale;
                    }
                    set_w(child, w);
                    set_h(child, h);
                } else {
                    if let Some(v) = spec_w {
                        set_w(child, v);
                    }
                    if let Some(v) = spec_h {
                        set_h(child, v);
                    }
                    if let Some(m) = spec_min_w {
                        set_w(child, child.width.max(m));
                    }
                    if let Some(m) = spec_min_h {
                        set_h(child, child.height.max(m));
                    }
                    if let Some(m) = spec_max_w {
                        set_w(child, child.width.min(m));
                    }
                    if let Some(m) = spec_max_h {
                        set_h(child, child.height.min(m));
                    }
                }
            }
        }
        apply_inset_walk(child, vx, vy, child_sc, styles, anchors);
    }
}

fn collect_anchors(
    b: &LayoutBox,
    origin_x: f32,
    origin_y: f32,
    within_scroller: bool,
    styles: &HashMap<NodeId, ComputedStyle>,
    out: &mut AnchorMap,
    found_insets: &mut bool,
) {
    let vx = origin_x + b.border_left + b.x;
    let vy = origin_y + b.border_top + b.y;
    if let Some(nid) = b.node_id
        && let Some(s) = styles.get(&nid)
    {
        if let Some(names) = s.anchor_name.as_deref() {
            // 全锚注册（pass2 区域定位几何不受介入门影响）；门标记随几何存档，
            // pass3 lookup 处统一消费。V 系 border box 原点（rect 提取/渲染
            // 递归口径，见 apply_inset_walk 注）——曾缺 border 累计致有边框
            // CB 的 inset/格坐标少 border（g36 定谳）。inline_flow 按 computed display 精确切：
            // 纯 inline 流盒（静态形/片段化，几何不可靠——anchor-position-
            // inline-002 介入致 2P→0P 的语料回归门；css-anchor-position-1
            // §anchor-pos 锚定盒取首片段语义未建模）；inline-block 原子盒
            // border box 可靠不受此门。
            for name in names.split(',') {
                let name = name.trim();
                if !name.is_empty() {
                    out.insert(
                        name.to_string(),
                        AnchorGeo {
                            x: vx,
                            y: vy,
                            w: b.width,
                            h: b.height,
                            inline_flow: matches!(s.display, zero_css_parser::values::DisplayValue::Inline),
                            in_scroller: within_scroller,
                        },
                    );
                }
            }
        }
        // anchor()/anchor-size() 函数声明存在性（无锚页 fallback 求值仍需走
        // pass3——尺寸槽 fallback 形可无锚名）。
        if !*found_insets
            && (s.anchor_inset_top.is_some()
                || s.anchor_inset_right.is_some()
                || s.anchor_inset_bottom.is_some()
                || s.anchor_inset_left.is_some()
                || s.anchor_size_slots.is_some())
        {
            *found_insets = true;
        }
    }
    for c in &b.children {
        // 子树滚动上下文（锚侧注册口径 = 严格祖先链；锚自身是滚动容器不
        // 改自身 border box 位置不计入——与 AnchorGeo.in_scroller 文档一致）。
        let child_sc = within_scroller || c.node_id.and_then(|nid| styles.get(&nid)).is_some_and(is_scroller);
        collect_anchors(c, vx, vy, child_sc, styles, out, found_insets);
    }
}

/// ComputedStyle 写作模式/方向 → 区域格坐标系描述（css-parser 单实现入参）。
fn area_wm_dir(s: &ComputedStyle) -> zero_css_parser::values::anchor_area::AreaWmDir {
    use zero_style_system::{DirectionValue, WritingModeValue};
    zero_css_parser::values::anchor_area::AreaWmDir {
        vertical: !matches!(s.writing_mode, WritingModeValue::HorizontalTb),
        block_rl: matches!(s.writing_mode, WritingModeValue::VerticalRl),
        rtl: matches!(s.direction, DirectionValue::Rtl),
    }
}

/// 单轴区域对齐模式（css-align self-position 物理化归类：区域物理
/// 低/高边 + 居中 + 锚中心 + 满填；inset 不建模 → 区域即对齐容器）。
#[derive(Clone, Copy, PartialEq)]
enum AreaAlign {
    /// 区域低边（左/上）margin-box 起边钉位。
    Low,
    /// 区域高边（右/下）margin-box 终边钉位。
    High,
    /// 区域内 margin-box 居中。
    Center,
    /// margin-box 中心对锚盒中心（§anchor-center；§4.1 span-all 默认）。
    AnchorCenter,
    /// 显式 stretch：auto 尺寸满填区域（显式尺寸只钉起边不覆写——挂账）。
    Fill,
}

/// 逻辑 start 在本轴是否物理低边（css-align self-alignment 逻辑关键字按
/// 自身写作模式：justify 轴取 inline、align 轴取 block——FIXME CB/自身
/// 参照系边界语义另验）。
fn logical_start_low(axis_is_inline: bool, inline_low: bool, block_low: bool) -> bool {
    if axis_is_inline { inline_low } else { block_low }
}

/// justify-self 显式值 → 对齐（物理 Left/Right 直接低/高边；Auto/Normal/
/// baseline 族返 None 走区域默认表——FIXME baseline 对齐未建模）。
fn align_from_justify(
    v: &zero_style_system::JustifySelfValue,
    axis_is_inline: bool,
    inline_low: bool,
    block_low: bool,
) -> Option<AreaAlign> {
    use zero_style_system::JustifySelfValue::*;
    let start_low = logical_start_low(axis_is_inline, inline_low, block_low);
    Some(match v {
        Stretch => AreaAlign::Fill,
        Start => {
            if start_low {
                AreaAlign::Low
            } else {
                AreaAlign::High
            }
        }
        End => {
            if start_low {
                AreaAlign::High
            } else {
                AreaAlign::Low
            }
        }
        Left => AreaAlign::Low,
        Right => AreaAlign::High,
        Center => AreaAlign::Center,
        Auto | Normal | Baseline => return None,
    })
}

/// align-self 显式值 → 对齐（Flex* 视同 start/end——非 flex 容器语义同流
/// 相对；Safe* 按底层对齐，溢出安全折让未建模 FIXME）。
fn align_from_align(
    v: &zero_css_parser::values::AlignmentValue,
    axis_is_inline: bool,
    inline_low: bool,
    block_low: bool,
) -> Option<AreaAlign> {
    use zero_css_parser::values::AlignmentValue::*;
    let start_low = logical_start_low(axis_is_inline, inline_low, block_low);
    Some(match v {
        Stretch => AreaAlign::Fill,
        Start | FlexStart | SafeStart | SafeFlexStart => {
            if start_low {
                AreaAlign::Low
            } else {
                AreaAlign::High
            }
        }
        End | FlexEnd | SafeEnd | SafeFlexEnd => {
            if start_low {
                AreaAlign::High
            } else {
                AreaAlign::Low
            }
        }
        Center | SafeCenter => AreaAlign::Center,
        Auto | Normal | Baseline | LastBaseline | SpaceBetween | SpaceAround | SpaceEvenly => return None,
    })
}

/// 区域对齐解析（css-anchor-position-1 §4.1 Area-specific Default Alignment）：
/// 显式 self-alignment 优先；否则区域默认表——仅 start 轨 → end、仅 end 轨 →
/// start、仅 center 轨 → center、三轨全选 → anchor-center。挂账：单 auto
/// inset 例外（§4.1——inset 面撤除，见模块头）不触发。
fn axis_align(explicit: Option<AreaAlign>, track: (u8, u8)) -> AreaAlign {
    if let Some(a) = explicit {
        return a;
    }
    match track {
        (0, 3) => AreaAlign::AnchorCenter,
        (0, _) => AreaAlign::High,
        (_, 3) => AreaAlign::Low,
        _ => AreaAlign::Center,
    }
}

/// 单轴几何参数（cb 相对：区域 [lo,hi] + 锚中心 + 起终侧 margin + auto 尺寸）。
struct AxisGeom {
    /// 区域低边（低轨格线）。
    lo: f32,
    /// 区域高边。
    hi: f32,
    /// 锚盒中心（§anchor-center 对齐基准）。
    anchor_center: f32,
    /// 起侧 margin（X=left / Y=top）。
    margin_start: f32,
    /// 终侧 margin（X=right / Y=bottom）。
    margin_end: f32,
    /// 尺寸 auto（Fill 时满填区域；显式尺寸不覆写）。
    size_auto: bool,
}

/// 单轴定位/尺寸应用（css-align margin-box 对齐口径；inset 属性不建模——
/// 区域即对齐容器，见模块头挂账）。Fill + auto 尺寸 → 满填区域；显式尺寸
/// 不覆写（区域只定位）。区域几何 `g.lo/hi` 与写入 `child.x/y` 同为 V 系
/// CB padding box 相对量（rect 提取/渲染递归口径），无需再换算。
fn apply_axis_align(child: &mut LayoutBox, horizontal: bool, align: AreaAlign, g: AxisGeom) {
    let bw = if horizontal { child.width } else { child.height };
    let pos = match align {
        AreaAlign::Fill => {
            if g.size_auto {
                let full = (g.hi - g.lo - g.margin_start - g.margin_end).max(0.0);
                if horizontal {
                    child.width = full;
                    child.content_width =
                        (full - child.border_left - child.border_right - child.padding_left - child.padding_right)
                            .max(0.0);
                } else {
                    child.height = full;
                    child.content_height =
                        (full - child.border_top - child.border_bottom - child.padding_top - child.padding_bottom)
                            .max(0.0);
                }
            }
            g.lo + g.margin_start
        }
        AreaAlign::Low => g.lo + g.margin_start,
        AreaAlign::High => g.hi - bw - g.margin_end,
        AreaAlign::Center => g.lo + g.margin_start + (g.hi - g.lo - g.margin_start - g.margin_end - bw) / 2.0,
        // margin-box 中心 = 锚中心（§anchor-center；auto margin 已按 0 计入，
        // 溢出安全折让未建模 FIXME）。
        AreaAlign::AnchorCenter => g.anchor_center + g.margin_start - (bw + g.margin_start + g.margin_end) / 2.0,
    };
    if horizontal {
        child.x = pos;
    } else {
        child.y = pos;
    }
}

fn apply_walk(
    b: &mut LayoutBox,
    origin_x: f32,
    origin_y: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
    anchors: &AnchorMap,
) {
    use zero_css_parser::values::LengthValue;
    use zero_css_parser::values::anchor_area::{AreaWmDir, position_area_cells_wm};

    let vx = origin_x + b.border_left + b.x;
    let vy = origin_y + b.border_top + b.y;
    // CB 几何（padding box——abspos containing block 语义，CSS2 §10.1）。
    // 仅当本盒是 positioned CB（直挂场景）才解析区域定位；非 positioned 中间层
    // 的 anchored 子（CB 在上层）第一刀不介入。vx/vy 已是 V 系 border box 原点
    //（rect 提取/渲染递归口径，见 apply_inset_walk 注）→ 只加 padding。
    let cb_is_positioned = b.is_abspos_cb;
    let cb_x = vx + b.padding_left;
    let cb_y = vy + b.padding_top;
    let cb_w = (b.width - b.border_left - b.border_right - b.padding_left - b.padding_right).max(0.0);
    let cb_h = (b.height - b.border_top - b.border_bottom - b.padding_top - b.padding_bottom).max(0.0);
    // CB 写作模式/方向（plain logical 区域关键字参照系，§position-area §3.1.2；
    // 非 styled 盒缺省 HTB_LTR）。
    let cb_wm = b
        .node_id
        .and_then(|nid| styles.get(&nid))
        .map(area_wm_dir)
        .unwrap_or(AreaWmDir::HTB_LTR);
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
                position_area_cells_wm(area, cb_wm, area_wm_dir(s))
        {
            // 锚解析：position-anchor 名原文匹配（anchor-name 列表项，含 -- 前缀）。
            // 区域定位不受行内/滚上下文门（坐标字段消费，见模块头挂账）。
            let anchor = s.position_anchor.as_deref().and_then(|pa| anchors.get(pa));
            if let Some(&g) = anchor {
                let (ax, ay, aw, ah) = (g.x, g.y, g.w, g.h);
                // 网格四线（相对 CB padding box；锚边越界允许负值——spec 区域
                // 网格线 = 锚 border box 各边在 CB 坐标投影）。
                let col = [0.0f32, ax - cb_x, ax + aw - cb_x, cb_w];
                let row = [0.0f32, ay - cb_y, ay + ah - cb_y, cb_h];
                let rx = col[cs as usize];
                let ry = row[rs as usize];
                let rw = (col[ce as usize] - rx).max(0.0);
                let rh = (row[re as usize] - ry).max(0.0);
                let font = &s.font_size;
                let ml = resolve_anchor_len(&s.margin_left, font);
                let mr = resolve_anchor_len(&s.margin_right, font);
                let mt = resolve_anchor_len(&s.margin_top, font);
                let mb = resolve_anchor_len(&s.margin_bottom, font);
                // 对齐解析（css-align self-alignment：justify → 自身 inline 轴、
                // align → block 轴；逻辑关键字按自身写作模式）。inset 属性不
                // 介入（区域即对齐容器，A/B 实证 inset-IMCB 破等值页——模块头挂账）。
                let self_wm = area_wm_dir(s);
                let inline_is_x = !self_wm.vertical;
                let inline_low = if self_wm.vertical {
                    !self_wm.block_rl
                } else {
                    !self_wm.rtl
                };
                let block_low = !self_wm.block_rl;
                let x_align = axis_align(
                    if inline_is_x {
                        align_from_justify(&s.justify_self, true, inline_low, block_low)
                    } else {
                        align_from_align(&s.align_self, false, inline_low, block_low)
                    },
                    (cs, ce),
                );
                let y_align = axis_align(
                    if inline_is_x {
                        align_from_align(&s.align_self, true, inline_low, block_low)
                    } else {
                        align_from_justify(&s.justify_self, false, inline_low, block_low)
                    },
                    (rs, re),
                );
                // 锚中心（cb 相对）——span-all 默认 anchor-center 对齐基准。
                let acx = ax - cb_x + aw / 2.0;
                let acy = ay - cb_y + ah / 2.0;
                let width_auto = matches!(s.width, LengthValue::Auto);
                let height_auto = matches!(s.height, LengthValue::Auto);
                apply_axis_align(
                    child,
                    true,
                    x_align,
                    AxisGeom {
                        lo: rx,
                        hi: rx + rw,
                        anchor_center: acx,
                        margin_start: ml,
                        margin_end: mr,
                        size_auto: width_auto,
                    },
                );
                apply_axis_align(
                    child,
                    false,
                    y_align,
                    AxisGeom {
                        lo: ry,
                        hi: ry + rh,
                        anchor_center: acy,
                        margin_start: mt,
                        margin_end: mb,
                        size_auto: height_auto,
                    },
                );
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

// ── anchor-size() 尺寸槽/inset 槽求值（css-anchor-position-1 §4）──────────

/// [`zero_style_system::ComputedStyle::anchor_size_slots`] 槽位的隐式缺省
/// <anchor-size> 关键字（§4 "defaults to the axis of the property"）：物理
/// width 族槽 → self-inline（inline 轴经目标自身写作模式映射）、height 族
/// 槽 → self-block；物理 inset 槽 left/right → width、top/bottom → height
///（物理属性槽轴 = 物理维度）。语料钉面：anchor-size-001 implicit 段
///（width/left → 5、height/top → 7，全 HTB 物理 = 逻辑两说同值）。
/// FIXME: 逻辑尺寸槽（inline-size/block-size）承载未接（apply 臂未捕获）。
fn anchor_size_default_keyword(slot: &str) -> &'static str {
    match slot {
        "width" | "min-width" | "max-width" => "self-inline",
        "height" | "min-height" | "max-height" => "self-block",
        "left" | "right" => "width",
        _ => "height",
    }
}

/// <anchor-size> 关键字 → 锚物理维度（css-anchor-position-1 §4 轴映射）：
/// width/height 物理直取；inline/block 按包含块写作模式（锚自身写作模式
/// 无关——anchor-size-writing-modes-001「Anchor's `writing-mode` should not
/// matter」四行断言钉面）；self-inline/self-block 按目标元素自身写作模式
///（同页「self-inline and self-block take target's writing-mode」钉面）。
fn anchor_size_axis_px(kw: &str, g: &AnchorGeo, cb_vertical: bool, self_vertical: bool) -> Option<f32> {
    Some(match kw {
        "width" => g.w,
        "height" => g.h,
        "inline" => {
            if cb_vertical {
                g.h
            } else {
                g.w
            }
        }
        "block" => {
            if cb_vertical {
                g.w
            } else {
                g.h
            }
        }
        "self-inline" => {
            if self_vertical {
                g.h
            } else {
                g.w
            }
        }
        "self-block" => {
            if self_vertical {
                g.w
            } else {
                g.h
            }
        }
        _ => return None,
    })
}

/// anchor-size() 声明原文 → px 求值（css-anchor-position-1 §4 used 值面）。
/// `slot`：消费属性名（隐式形缺省关键字与 fallback % 轴向由此派生）；
/// `cb_size`：槽轴向 CB padding box 尺寸（fallback 百分比基）。
/// 显式名查无 + 默认锚 None → fallback `<length-percentage>` → 仍无 → None
///（槽位保持既有行为 = 初始 Auto；spec invalid → fallback → IACVT 链）。
/// 逻辑关键字写作模式映射：inline/block 按 CB（`cb_vertical`）、self-* 按
/// 目标（`self_vertical`）。
/// https://drafts.csswg.org/css-anchor-position-1/#anchor-size
fn resolve_anchor_size_raw(
    raw: &str,
    slot: &str,
    cb_size: f32,
    lookup: &AnchorLookup,
    ctx: &AnchorSizeCtx,
) -> Option<f32> {
    let inner = raw.strip_prefix("anchor-size(")?.strip_suffix(')')?;
    let (head, fallback) = split_top_level_comma(inner);
    let mut name: Option<&str> = None;
    let mut kw: Option<&str> = None;
    for tok in head.split_whitespace() {
        if tok.starts_with("--") {
            if name.is_some() {
                return None;
            }
            name = Some(tok);
        } else if kw.is_none()
            && matches!(
                tok,
                "width" | "height" | "inline" | "block" | "self-inline" | "self-block"
            )
        {
            kw = Some(tok);
        } else {
            // 表外 token（超量 token/嵌套 calc() 实参形）第一刀不介入。
            return None;
        }
    }
    let geo = match name {
        Some(n) => lookup(n),
        None => ctx.default_geo,
    };
    if let Some(g) = geo {
        let kw = kw.unwrap_or_else(|| anchor_size_default_keyword(slot));
        return anchor_size_axis_px(kw, &g, ctx.cb_vertical, ctx.self_vertical);
    }
    // fallback：<length> | <percentage>（% 按槽轴向 CB 基）。auto/表外 → None。
    let fb = fallback?.trim();
    if let Some(p) = fb.strip_suffix('%') {
        return p.parse::<f32>().ok().map(|p| p / 100.0 * cb_size);
    }
    match zero_css_parser::values::parse_length(fb) {
        Some(zero_css_parser::values::LengthValue::Auto) | None => None,
        Some(v) => Some(resolve_anchor_len(&v, ctx.font_size)),
    }
}
