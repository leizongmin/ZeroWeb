//! CSS Anchor Positioning 布局期（wptfyi-css 组F 布局刀2 + 刀3）：position-area
//! 区域定位 pass——锚注册收集（`anchor-name` → 视口 border-box 几何）+
//! anchored 元素 3×3 网格区域定位（htb 写作模式第一刀）；刀3 增 `anchor()`
//! inset 函数求值（物理边 + center + 百分比 + fallback，物理轴第一刀）。
//!
//! 与 abspos.rs 修正 pass 同架构：taffy 主布局后的 LayoutBox 树自递归 walk。
//! 两段式：先全树收集锚几何（树序后者覆盖——`position-anchor` 默认锚取
//! preceding 树序最后匹配，css-anchor-position-1 §position-anchor），再自顶
//! 向下应用（先外后内——嵌套 anchored 链外层先定型）。区域格解析复用
//! `css_parser::values::anchor_area::position_area_cells_dir`（CSSOM 声明面与
//! 布局共用单实现；矩阵下沉 css-parser——layout-engine 不依赖 zero-engine）。
//!
//! 第一刀边界（挂账）：
//! - anchor() 逻辑边关键字（start/end/self-*，需锚元素写作模式映射）不介入
//!   （回落 fallback/槽位既有行为）；anchor-size() 尺寸槽求值另刀；
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
//! - align/justify-self 非 stretch 的区域对齐蕴含不建模（格满填语义）；
//! - anchored 显式 width/height 或 inset 声明时不覆写尺寸（仅定位）；
//! - `position-try-fallbacks`（keyword tactic / @position-try）与 last-successful
//!   position 未建模：任一 fallback 声明不介入（静态位保持基线行为——语料实证
//!   介入净 -5：内置 flip 需布局级区域溢出检查 + last-successful 状态机，custom
//!   `--pt` 需 shim 期 fallback 应用引擎；fallback 面另刀）；
//! - self 轴按 anchored 元素自身 direction 解析（rtl 镜像 inline 格）；writing-
//!   mode 非 htb 元素与 CB 侧逻辑轴映射（anchor-position-writing-modes 族）
//!   另刀——物理边 anchor() 求值与写作模式正交（本刀语料实证面）。
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
    let vx = origin_x + b.x;
    let vy = origin_y + b.y;
    // 子树滚动上下文（目标侧 = 盒树祖先链含本盒；锚侧注册口径一致）。
    let child_sc = within_scroller || b.node_id.and_then(|nid| styles.get(&nid)).is_some_and(is_scroller);
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
            let resolve = |raw: &str, slot: &str, cb_size: f32| {
                resolve_anchor_inset(raw, slot, cb_size, &lookup, default_geo, (cb_x, cb_y), &s.font_size)
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
            // 拉伸定尺寸，单侧定位（尺寸取 taffy shrink-to-fit 结果）。
            if let Some(lv) = l {
                child.x = cb_x + lv - vx + ml;
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
                child.x = cb_x + cb_w - rv - mr - child.width;
            }
            if let Some(tv) = t {
                child.y = cb_y + tv - vy + mt;
                if let Some(bv) = bt {
                    let h = (cb_h - tv - bv - mt - mb).max(0.0);
                    child.height = h;
                    child.content_height =
                        (h - child.border_top - child.border_bottom - child.padding_top - child.padding_bottom)
                            .max(0.0);
                }
            } else if let Some(bv) = bt {
                child.y = cb_y + cb_h - bv - mb - child.height;
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
    let vx = origin_x + b.x;
    let vy = origin_y + b.y;
    if let Some(nid) = b.node_id
        && let Some(s) = styles.get(&nid)
    {
        if let Some(names) = s.anchor_name.as_deref() {
            // 全锚注册（pass2 区域定位几何不受介入门影响）；门标记随几何存档，
            // pass3 lookup 处统一消费。inline_flow 按 computed display 精确切：
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
        // anchor() inset 声明存在性（无锚页 fallback 求值仍需走 pass3）。
        if !*found_insets
            && (s.anchor_inset_top.is_some()
                || s.anchor_inset_right.is_some()
                || s.anchor_inset_bottom.is_some()
                || s.anchor_inset_left.is_some())
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
