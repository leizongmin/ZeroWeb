//! 边框绘制 — 常规边框、border-image、outline。
//!
//! 包含 BorderEdgeSpec、paint_borders、paint_border_image、paint_border_edge、
//! border_fill_rect、paint_3d_border、paint_outline。

use zero_css_parser::values::{ColorValue, CornerShapeKind, LengthValue};
use zero_dom::Document;
use zero_layout_engine::LayoutBox;
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::image_cache::ImageKey;
use zero_render_foundation::primitive::{FilterPrimitive, ImagePrimitive, LineCap, LineStyle, StrokePrimitive};
use zero_style_system::{
    AppearanceComputedValue, BorderCollapseValue, BorderImageOutsetComputedComponent, BorderImageRepeatComputedMode,
    BorderImageSliceComputedComponent, BorderImageSourceComputedValue, BorderImageWidthComputedComponent,
    BorderStyleValue, ComputedStyle, DisplayValue, FilterComputedValue,
};

use super::super::color::{color_value_to_render, resolve_color_current};
use super::super::helpers::image_resource_key;

/// 边框边缘规格 — 描述一条边框的几何位置和方向。
pub(super) struct BorderEdgeSpec {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub thickness: f32,
    pub is_horizontal: bool,
    /// 垂直边框时，填充区域是否向左延伸（右侧边框为 true）。
    pub extend_left: bool,
}

/// R3906：border-image 9-slice 源子矩形表（CSS Backgrounds 3 §6.2）——源图按
/// border-image-slice 偏移切出的九片在源图中的**归一化 [0,1]** 矩形。
pub(super) struct NineSliceSources {
    pub top_left: Rect,
    pub top_right: Rect,
    pub bottom_right: Rect,
    pub bottom_left: Rect,
    pub top: Rect,
    pub right: Rect,
    pub bottom: Rect,
    pub left: Rect,
    pub center: Rect,
}

impl super::Painter {
    /// 绘制带首个 `<legend>` 的 fieldset 边框。
    ///
    /// https://html.spec.whatwg.org/multipage/rendering.html#the-fieldset-and-legend-elements
    pub(super) fn paint_fieldset_borders(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
        doc: Option<&Document>,
        legend_x: f32,
        legend_width: f32,
        legend_height: f32,
    ) {
        let top_y = abs_y + legend_height / 2.0;
        let mut lower_box = box_node.clone();
        lower_box.height = (lower_box.height - legend_height / 2.0).max(0.0);
        lower_box.border_top = 0.0;
        self.paint_borders(&lower_box, abs_x, top_y, style, doc);

        if box_node.border_top <= 0.0
            || matches!(
                style.border_top_style,
                BorderStyleValue::None | BorderStyleValue::Hidden
            )
        {
            return;
        }
        let color = resolve_border_current_color(style.border_top_color.clone(), &style.color);
        let left_end = legend_x.clamp(abs_x, abs_x + box_node.width);
        let right_start = (legend_x + legend_width).clamp(abs_x, abs_x + box_node.width);
        for (x1, x2) in [(abs_x, left_end), (right_start, abs_x + box_node.width)] {
            if x2 > x1 {
                self.paint_border_edge(
                    &BorderEdgeSpec {
                        x1,
                        y1: top_y,
                        x2,
                        y2: top_y,
                        thickness: box_node.border_top,
                        is_horizontal: true,
                        extend_left: false,
                    },
                    &style.border_top_style,
                    &color,
                );
            }
        }
    }

    /// 绘制边框（4 条边，支持多种 border-style）。
    ///
    /// 分别绘制上、右、下、左四条边框。根据 border-style 生成不同类型的图元：
    /// - Solid/None/Hidden：填充矩形（原有行为）
    /// - Dotted：圆头点线描边
    /// - Dashed：方头虚线描边
    /// - Double：双线填充矩形（中间留空隙）
    /// - Groove/Ridge/Inset/Outset：3D 效果双色填充
    pub(super) fn paint_borders(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
        doc: Option<&Document>,
    ) {
        // R4534（css-borders-4 §7.5.1）：border-shape 活跃时抑制常规 4 边绘制与形角
        // 环带（border-radius 视为 0、corner-shape 隐式忽略），改发射形状描边
        //（stroke mode，路径居中描边）或内外形状环带填充（fill mode，even-odd）。
        // R5184：等价域（stroke inset(0 round R) + half-border-box）跳过形状描边，
        // 落 R5180 弧带/条带路径（spec 注入等价半径）——与 border-radius 渲染同图元。
        let shape_equiv_radii = crate::paint::helpers::border_shape_round_inset_equivalent_radii(style, box_node);
        if shape_equiv_radii.is_none()
            && let Some(plan) = super::super::helpers::border_shape_plan(style, box_node, abs_x, abs_y)
        {
            match plan {
                crate::paint::helpers::BorderShapePlan::Stroke { vertices, width, color } => {
                    if width > 0.0 {
                        let flat: Vec<f32> = vertices.iter().flat_map(|(x, y)| [*x, *y]).collect();
                        self.primitives.add_path_stroke(flat, color, width, true);
                        // R5186：miter join 楔形补齐逐段粗线无 join 的顶点缺口
                        //（尖角 miter 尖刺，polygon-miter-limit 判例）。
                        for wedge in crate::paint::helpers::stroke_miter_wedges(&vertices, width) {
                            let flat: Vec<f32> = wedge.into_iter().flat_map(|(x, y)| [x, y]).collect();
                            self.primitives.add_path_fill(flat, color);
                        }
                    }
                }
                crate::paint::helpers::BorderShapePlan::Fill { outer, inner, color } => {
                    // even-odd：外+内经 keyhole 切点串联（R5182）→ 扫描线仅填环带且
                    // 无撕裂（朴素 extend 的两条桥边不同 x 交叉，带内撕出背景色隙）。
                    let ring = crate::paint::helpers::concat_ring_vertices(outer, inner);
                    let flat: Vec<f32> = ring.into_iter().flat_map(|(x, y)| [x, y]).collect();
                    self.primitives.add_path_fill(flat, color);
                }
            }
            return;
        }
        let w = box_node.width;
        let h = box_node.height;
        // R5304：Button 族原生边框消费绘制侧 used 值——auto 与 legacy 别名在
        // button 上均解析为 Button（css-ui-4 两态模型），appearance-auto-ref 类
        // 页面与声明 button 同渲染。
        let eff_appearance = super::effective_appearance(style, box_node, doc);
        if matches!(
            eff_appearance,
            AppearanceComputedValue::Button
                | AppearanceComputedValue::PushButton
                | AppearanceComputedValue::SquareButton
        ) {
            // https://html.spec.whatwg.org/multipage/rendering.html#widgets
            // Chromium keeps the 2px native button border in layout geometry but
            // rasterizes a single outer device-pixel edge.
            let color = resolve_color_current(&style.border_top_color, &style.color);
            let corner_outer = Color::rgba(162, 162, 162, 255);
            let corner_inner = Color::rgba(152, 152, 152, 255);
            self.primitives
                .add_fill(Rect::new(abs_x + 3.0, abs_y, (w - 6.0).max(0.0), 1.0), color);
            self.primitives
                .add_fill(Rect::new(abs_x + 3.0, abs_y + h - 1.0, (w - 6.0).max(0.0), 1.0), color);
            self.primitives
                .add_fill(Rect::new(abs_x, abs_y + 3.0, 1.0, (h - 6.0).max(0.0)), color);
            self.primitives
                .add_fill(Rect::new(abs_x + w - 1.0, abs_y + 3.0, 1.0, (h - 6.0).max(0.0)), color);
            for (offset, shade) in [(1.0, corner_outer), (2.0, corner_inner)] {
                self.primitives
                    .add_fill(Rect::new(abs_x + offset, abs_y, 1.0, 1.0), shade);
                self.primitives
                    .add_fill(Rect::new(abs_x + w - offset - 1.0, abs_y, 1.0, 1.0), shade);
                self.primitives
                    .add_fill(Rect::new(abs_x + offset, abs_y + h - 1.0, 1.0, 1.0), shade);
                self.primitives
                    .add_fill(Rect::new(abs_x + w - offset - 1.0, abs_y + h - 1.0, 1.0, 1.0), shade);
                self.primitives
                    .add_fill(Rect::new(abs_x, abs_y + offset, 1.0, 1.0), shade);
                self.primitives
                    .add_fill(Rect::new(abs_x + w - 1.0, abs_y + offset, 1.0, 1.0), shade);
                self.primitives
                    .add_fill(Rect::new(abs_x, abs_y + h - offset - 1.0, 1.0, 1.0), shade);
                self.primitives
                    .add_fill(Rect::new(abs_x + w - 1.0, abs_y + h - offset - 1.0, 1.0, 1.0), shade);
            }
            let corner_center = Color::rgba(172, 172, 172, 255);
            for (x, y) in [
                (abs_x + 1.0, abs_y + 1.0),
                (abs_x + w - 2.0, abs_y + 1.0),
                (abs_x + 1.0, abs_y + h - 2.0),
                (abs_x + w - 2.0, abs_y + h - 2.0),
            ] {
                self.primitives.add_fill(Rect::new(x, y, 1.0, 1.0), corner_center);
            }
            return;
        }

        // border-collapse:collapse 时，内共享边（有邻居）厚度减半，两邻居各画一半合成
        // 居中的完整边框（CSS 2.1 §17.6.2）；外边缘（表格周边，无邻居共享）须绘制
        // 完整厚度。`collapsed_border_outer_edge` 由 resolve_collapsed_borders 阶段 4 标记。
        // kill-switch `ZW_COLLAPSE_OUTER_FULL=0` 关闭（default-on，恢复旧行为：全部减半）。
        let collapse = matches!(style.border_collapse, BorderCollapseValue::Collapse);
        let outer_full = collapse && std::env::var("ZW_COLLAPSE_OUTER_FULL").as_deref() != Ok("0");
        let outer = &box_node.collapsed_border_outer_edge;
        // R5282：winner-center 尺寸模型下 cell 盒缘贴格线（layout 端槽宽含胜者半带），
        // 外缘带一律以格线（盒缘）为中心——含行/行组/表/列源胜出（color override 臂，
        // legacy 内绘 full 的前提「盒缘未在带中心」不再成立）。与 layout 端
        // ZW_TABLE_COLLAPSE_WINNER_CENTER 同名同默认（kill-switch 成对）：默认回退
        // border-center（自源 ref 对 −3px 挂账），`=1` 启用。
        // R5283：与 layout 端 collapse_winner_center_table 同门——vertical 表（cell
        // writing_mode 继承自表）格线模型未接，保持 border-center 带绘制。
        // R5285：随 layout 端翻正默认 ON（`!= "0"` 成对；只翻 layout 侧会成混合态
        // ——corpus 15720→15706 十四案回归实证），回退 kill-switch 显式 `=0`。
        let winner_center = collapse
            && std::env::var("ZW_TABLE_COLLAPSE_WINNER_CENTER").as_deref() != Ok("0")
            && !matches!(
                box_node.writing_mode,
                zero_style_system::WritingModeValue::VerticalRl | zero_style_system::WritingModeValue::VerticalLr
            );
        let winner_center_outer_centered = |side: usize, box_node: &LayoutBox| -> bool {
            winner_center
                || box_node
                    .collapsed_border_color_overrides
                    .as_deref()
                    .is_none_or(|ovr| ovr[side].is_none())
        };
        // R4248（CSS Borders 4 §corner-shaping）：形角盒的边框环——外圈形角多边形
        //（边框色）+ 内圈形角多边形（背景色）叠出环带。前提：四边同色（含 collapse
        // 覆盖差异为零）、背景非透明（内圈须遮出环带孔）、非 collapse 表格。
        // 其余（杂色边框等）回退条带绘制（条带未经形状裁剪，近似旧渲染）。
        let shaped_uniform = !style.corner_shape.is_all_round()
            && !collapse
            && style.background_color != ColorValue::Transparent
            && style.border_top_color == style.border_right_color
            && style.border_right_color == style.border_bottom_color
            && style.border_bottom_color == style.border_left_color;
        if std::env::var("ZW_CORNER_DBG").is_ok() {
            eprintln!(
                "[corner-ring] shaped_uniform={} colors t={:?} r={:?} bg={:?}",
                shaped_uniform, style.border_top_color, style.border_right_color, style.background_color
            );
        }
        if shaped_uniform {
            let border_color = resolve_color_current(&style.border_top_color, &style.color);
            if let Some(outer) = crate::paint::helpers::shaped_corner_polygon(style, w, h, abs_x, abs_y) {
                if std::env::var("ZW_CORNER_DBG").is_ok() {
                    eprintln!("[corner-ring] EMIT outer verts={}", outer.len());
                }
                self.primitives
                    .add_path_fill(outer.into_iter().flat_map(|(x, y)| [x, y]).collect(), border_color);
                let bt = box_node.border_top;
                let br_ = box_node.border_right;
                let bb = box_node.border_bottom;
                let bl = box_node.border_left;
                let clamp0 = |v: f32| v.max(0.0);
                // 内圈全半径与外圈同口径（R5183：shaped_radius_spec 规范解析）。
                let full_r = crate::paint::helpers::shaped_radius_spec(style, w, h);
                let inset_bw = [bl.max(bt), bt.max(br_), br_.max(bb), bb.max(bl)];
                let radii4 = std::array::from_fn(|i: usize| {
                    // R5179：凹侧内圈保留全半径——chromium 实渲染的边框带 = 沿外轮廓
                    // 的 bw 均匀内描边：凹侧（notch 阶梯 / scoop 与深 K 凹弧）的内圈
                    // 腿位 = 内盒角点 + 全半径（notch/深 K 精确；scoop 的真描边弧心
                    // 在外盒角、半径 r+bw，全半径内弧是 builder 可表达下的最近近似，
                    // 偏差 ≤ ~1px——FIXME 如需像素级对齐再引入逐角真描边）。
                    // §3.9.4.2 adjusted 公式在凹侧腿与外圈重合（带宽归零），与
                    // chromium/WPT ref 不符，此处以 chromium 为准。凸侧同心内缩
                    // （r − bw，圆心不变）维持 §3.9.4.2 语义。
                    let kind_i = style.corner_shape.corner(i);
                    let concave = match kind_i {
                        CornerShapeKind::Notch | CornerShapeKind::Scoop => true,
                        CornerShapeKind::Superellipse(k) => k <= -1.0,
                        _ => false,
                    };
                    if concave {
                        full_r[i].max(0.0)
                    } else {
                        clamp0(full_r[i] - inset_bw[i])
                    }
                });
                let bg = resolve_color_current(&style.background_color, &style.color);
                if let Some(inner) = crate::paint::helpers::shaped_corner_polygon_in_box(
                    style,
                    abs_x + bl,
                    abs_y + bt,
                    w - bl - br_,
                    h - bt - bb,
                    radii4,
                ) {
                    self.primitives
                        .add_path_fill(inner.into_iter().flat_map(|(x, y)| [x, y]).collect(), bg);
                }
                return;
            }
        }

        // R5180（css-backgrounds-3 §5.5 弧带）：plain border-radius 的边框带按弧绘制——
        // 外圈（BorderRadiusSpec::from_style_with_box 规范口径：百分比/§5.5 重叠
        // 收缩，与背景裁剪同形）+ 内圈（padding edge，max(0, r−bw) 同心）两多边形串联
        // 单次 even-odd path_fill（cpu/stroke.rs 扫描线 even-odd），孔区不涂改背景，
        // 透明背景亦适用。适用判定（非 collapse、四边 Solid 同色等宽）见 helpers
        // `round_band_uniform_border`；其余（杂色/异宽/非 Solid/零半径）维持条带路径。
        // corpus self-source 下 radius+border 案两侧同变（learnings
        // 「border-radius-band-sharp-strips」）。内盒非正（边框厚过半盒）回退条带。
        if let Some(band_b) = crate::paint::helpers::round_band_uniform_border(style, box_node) {
            // R5184：border-shape 等价域注入等价半径（否则 border-radius 视为 0）。
            let spec = shape_equiv_radii
                .unwrap_or_else(|| crate::paint::helpers::BorderRadiusSpec::from_style_with_box(style, w, h));
            if !spec.is_zero() && w - 2.0 * band_b > 0.0 && h - 2.0 * band_b > 0.0 {
                let border_color = resolve_color_current(&style.border_top_color, &style.color);
                let full_r = [spec.top_left, spec.top_right, spec.bottom_right, spec.bottom_left];
                // 内圈 = padding edge（css-backgrounds-3 §5.5：内半径 max(0, r − bw)；
                // 等宽时圆心不变，与外圈同心）。R5341：垂直轴同式（等宽 → 同 band_b）。
                let inner_radii = full_r.map(|r| (r - band_b).max(0.0));
                let inner_ry = spec.ry.map(|r| (r - band_b).max(0.0));
                let ring = crate::paint::helpers::concat_ring_vertices(
                    crate::paint::helpers::rounded_corner_polygon_in_box(abs_x, abs_y, w, h, full_r, spec.ry),
                    crate::paint::helpers::rounded_corner_polygon_in_box(
                        abs_x + band_b,
                        abs_y + band_b,
                        w - 2.0 * band_b,
                        h - 2.0 * band_b,
                        inner_radii,
                        inner_ry,
                    ),
                );
                // even-odd：外+内顶点串联 → 扫描线仅填环带（同 BorderShapePlan::Fill）。
                let flat: Vec<f32> = ring.into_iter().flat_map(|(x, y)| [x, y]).collect();
                self.primitives.add_path_fill(flat, border_color);
                return;
            }
        }

        // 每条边的有效厚度：collapse 时外边缘=full，内共享边=half；非 collapse=原值。
        let eff = |side: usize, v: f32| -> f32 {
            if collapse {
                if outer_full && outer[side] { v } else { v / 2.0 }
            } else {
                v
            }
        };
        let top_t = eff(0, box_node.border_top);
        let right_t = eff(1, box_node.border_right);
        let bottom_t = eff(2, box_node.border_bottom);
        let left_t = eff(3, box_node.border_left);

        // R5262（CSS2 §17.6.2 collapse 带几何，layout 端格线原点平移的 paint 侧
        // 配对）：cell 自身边框胜出的外缘带以格线为中心——起点外展半厚，带跨
        // [edge−t/2, edge+t/2]（chromium 001e：td 盒缘 = 表盒+10，顶带
        // [表盒, 表盒+20]）。门 = 该边**无 color override**：阶段 3 仅在非 cell
        // 源（行/行组/表/列）胜出时写 override 并把胜出宽度入账 cell border——
        // 此时表盒几何未含该带膨胀（sizing 只读 cell 自身边框，先于 resolve），
        // 盒缘尚未移到带中心，legacy 内绘 full 恰与 chromium 带绝对位对齐
        //（border-top-width-applies-to-001：行组 96px 胜出、表盒高 0，内绘
        // [表盒, 表盒+96] ≈ ref img 位 [53,149]；外展反而上溢 48px 盖段落，
        // 1.92% 实证）。cell 自胜时盒缘已在带中心（R5262 格线原点平移），
        // 内绘 full 会下偏半带（001e 3.92% 根因之一），外展居中才对位。
        let band_outset = |side: usize, t: f32| -> f32 {
            if collapse && outer_full && outer[side] && winner_center_outer_centered(side, box_node) {
                t / 2.0
            } else {
                0.0
            }
        };
        let band_far = |side: usize, t: f32| -> f32 {
            if collapse && outer_full && outer[side] && winner_center_outer_centered(side, box_node) {
                t / 2.0
            } else {
                t
            }
        };
        let top_far = band_far(0, top_t);
        let bottom_near = band_far(2, bottom_t);

        // R5263（CSS2 §17.6.2 collapse 带端归属）：横带端部随相邻竖格线胜者伸缩。
        // chromium 001e 实测：横带跨 [首格线−win/2, 末格线+win/2]（蓝 cell 顶带
        // [48,108] = 盒 [58,98] 左外延 10 + 右跨格线外延 10）——自身边框胜该格线
        // （该边无 override）→ 端点外延 win/2（eff 口径：外缘带 full → t/2，内缘
        // 两半带 → t）；败于邻 cell（阶段 3/R1626 把胜者宽色写入本边 override，
        // [g−win/2, g+win/2] 已由两侧半带按胜者色合成）→ 端点收进同量，让位邻
        // cell 顶带外延；本边无带（win 后仍 0 宽）或外缘带被行/表/列源夺色的
        // legacy 域 → 维持盒缘。
        // R5264：端偏移改判「邻边」归属，不问本带自身 override——001d 取证证
        // loser 带（阶段 3 override 半带）在表 margin 列同样按邻边伸缩（chromium
        // cell five 顶带 override 蓝 [48,108)：左端 = 外缘自胜外延 10、右端 =
        // 右缘自胜外延 10），own-override 门曾封死该外延 → margin 列共享带下半
        // 露白。bwat001 域（行组/表源横带 override + cell 无竖边框）ext 取
        // adj_t = 0 自然不动，原整体门冗余部分撤销。另限 TableCell 盒：表盒自身
        // 红框同样无 override 全自胜，会越出表 border-box 外延（实测表框左右各溢
        // 10px），带端伸缩是格线 cell 专属语义。
        let band_end_off = |adj: usize, adj_w: f32, adj_t: f32, outward: f32| -> f32 {
            if !collapse || !outer_full || !matches!(style.display, DisplayValue::TableCell) {
                return 0.0;
            }
            let ext = if outer[adj] { adj_t / 2.0 } else { adj_t };
            if box_node
                .collapsed_border_color_overrides
                .as_deref()
                .is_none_or(|ovr| ovr[adj].is_none())
            {
                outward * ext
            } else if !outer[adj] && adj_w > 0.0 {
                -outward * ext
            } else {
                0.0
            }
        };
        let top_l = band_end_off(3, box_node.border_left, left_t, -1.0);
        let top_r = band_end_off(1, box_node.border_right, right_t, 1.0);
        let bot_l = band_end_off(3, box_node.border_left, left_t, -1.0);
        let bot_r = band_end_off(1, box_node.border_right, right_t, 1.0);

        // border-collapse 颜色覆盖：当表格边框胜出时使用表格的颜色
        let color_override = box_node.collapsed_border_color_overrides.as_deref();
        let top_color = collapsed_border_color(&color_override.and_then(|ovr| ovr[0]), &style.border_top_color);
        let right_color = collapsed_border_color(&color_override.and_then(|ovr| ovr[1]), &style.border_right_color);
        let bottom_color = collapsed_border_color(&color_override.and_then(|ovr| ovr[2]), &style.border_bottom_color);
        let left_color = collapsed_border_color(&color_override.and_then(|ovr| ovr[3]), &style.border_left_color);

        // currentColor 解析为元素自身计算 `color`（CSS-Color §resolving）。border-color 初始值
        // 为 currentColor 关键字，经层叠/继承保留，paint 时替换为元素 color 的使用值。
        let current = &style.color;
        let top_color = resolve_border_current_color(top_color, current);
        let right_color = resolve_border_current_color(right_color, current);
        let bottom_color = resolve_border_current_color(bottom_color, current);
        let left_color = resolve_border_current_color(left_color, current);

        // border-collapse 样式覆盖：冲突解决后使用获胜方的样式
        let top_style = box_node.collapsed_border_style_overrides[0]
            .as_ref()
            .unwrap_or(&style.border_top_style);
        let right_style = box_node.collapsed_border_style_overrides[1]
            .as_ref()
            .unwrap_or(&style.border_right_style);
        let bottom_style = box_node.collapsed_border_style_overrides[2]
            .as_ref()
            .unwrap_or(&style.border_bottom_style);
        let left_style = box_node.collapsed_border_style_overrides[3]
            .as_ref()
            .unwrap_or(&style.border_left_style);

        // R5083（css-backgrounds-3 §4.4 corner transitions）：相邻两边均非零宽时，角部
        // 过渡区由两边分摊；chromium 取「外角点 → 内角点（padding edge 角）」对角线
        // 分色（半径 0 退化为 mitre 直线）。仅 Solid×Solid、解析色不同、双侧有效厚度
        // ≥1 时启用（更细的角部过渡在像素上不可见，保持整幅矩形避免 AA 噪声）；等色
        // 过渡不可见，保留整幅矩形省两次路径填充。零宽/隐藏边不适用——§4.4「另一边
        // 占据整个过渡区」，即现有整幅矩形行为。collapse 表格走冲突解决半厚绘制，
        // 不在本域。kill-switch ZW_BORDER_DIAGONAL_JOIN=0 恢复整幅矩形（旧行为）。
        let diagonal_joins = !collapse && std::env::var("ZW_BORDER_DIAGONAL_JOIN").as_deref() != Ok("0");
        let is_solid_style = |s: &BorderStyleValue| matches!(s, BorderStyleValue::Solid);
        let corner_join = |a: bool, b: bool, ca: &ColorValue, cb: &ColorValue, ta: f32, tb: f32| {
            a && b && ca != cb && ta >= 1.0 && tb >= 1.0
        };
        let (diag_tl, diag_tr, diag_br, diag_bl) = if diagonal_joins {
            (
                corner_join(
                    box_node.border_top > 0.0 && is_solid_style(top_style),
                    box_node.border_left > 0.0 && is_solid_style(left_style),
                    &top_color,
                    &left_color,
                    top_t,
                    left_t,
                ),
                corner_join(
                    box_node.border_top > 0.0 && is_solid_style(top_style),
                    box_node.border_right > 0.0 && is_solid_style(right_style),
                    &top_color,
                    &right_color,
                    top_t,
                    right_t,
                ),
                corner_join(
                    box_node.border_bottom > 0.0 && is_solid_style(bottom_style),
                    box_node.border_right > 0.0 && is_solid_style(right_style),
                    &bottom_color,
                    &right_color,
                    bottom_t,
                    right_t,
                ),
                corner_join(
                    box_node.border_bottom > 0.0 && is_solid_style(bottom_style),
                    box_node.border_left > 0.0 && is_solid_style(left_style),
                    &bottom_color,
                    &left_color,
                    bottom_t,
                    left_t,
                ),
            )
        } else {
            (false, false, false, false)
        };

        // 上边框（diag 时左右两端各退让出角部三角区，由下方三角形补足）
        if box_node.border_top > 0.0 && *top_style != BorderStyleValue::None && *top_style != BorderStyleValue::Hidden {
            self.paint_border_edge(
                &BorderEdgeSpec {
                    // R5263：collapse 横带端部随相邻竖格线胜者伸缩（band_end_off）
                    x1: if diag_tl { abs_x + left_t } else { abs_x + top_l },
                    // R5262：collapse cell 外缘带外展半厚（带以格线为中心）
                    y1: abs_y - band_outset(0, top_t),
                    x2: if diag_tr {
                        abs_x + w - right_t
                    } else {
                        abs_x + w + top_r
                    },
                    y2: abs_y,
                    thickness: top_t,
                    is_horizontal: true,
                    extend_left: false,
                },
                top_style,
                &top_color,
            );
            if diag_tl {
                // 左上角 top 分摊侧：对角线上方三角（外角点→顶边内点→内角点）
                self.add_corner_triangle(
                    [(abs_x, abs_y), (abs_x + left_t, abs_y), (abs_x + left_t, abs_y + top_t)],
                    &top_color,
                );
            }
            if diag_tr {
                // 右上角 top 分摊侧
                self.add_corner_triangle(
                    [
                        (abs_x + w, abs_y),
                        (abs_x + w - right_t, abs_y),
                        (abs_x + w - right_t, abs_y + top_t),
                    ],
                    &top_color,
                );
            }
        }

        // 右边框
        if box_node.border_right > 0.0
            && *right_style != BorderStyleValue::None
            && *right_style != BorderStyleValue::Hidden
        {
            self.paint_border_edge(
                &BorderEdgeSpec {
                    // R5262：collapse cell 外缘带外展半厚 + 带端收边到相邻横带的远/近缘
                    x1: abs_x + w + band_outset(1, right_t),
                    y1: abs_y + top_far,
                    x2: abs_x + w,
                    y2: abs_y + h - bottom_near,
                    thickness: right_t,
                    is_horizontal: false,
                    extend_left: true,
                },
                right_style,
                &right_color,
            );
            if diag_tr {
                // 右上角 right 分摊侧：对角线下方三角
                self.add_corner_triangle(
                    [
                        (abs_x + w, abs_y),
                        (abs_x + w, abs_y + top_t),
                        (abs_x + w - right_t, abs_y + top_t),
                    ],
                    &right_color,
                );
            }
            if diag_br {
                // 右下角 right 分摊侧：对角线上方三角
                self.add_corner_triangle(
                    [
                        (abs_x + w, abs_y + h),
                        (abs_x + w, abs_y + h - bottom_t),
                        (abs_x + w - right_t, abs_y + h - bottom_t),
                    ],
                    &right_color,
                );
            }
        }

        // 下边框 — y1 在边框盒底边向上 border_bottom 处（边框区域内侧）
        if box_node.border_bottom > 0.0
            && *bottom_style != BorderStyleValue::None
            && *bottom_style != BorderStyleValue::Hidden
        {
            self.paint_border_edge(
                &BorderEdgeSpec {
                    // R5263：collapse 横带端部随相邻竖格线胜者伸缩（band_end_off）
                    x1: if diag_bl { abs_x + left_t } else { abs_x + bot_l },
                    // R5262：collapse cell 外缘带外展半厚（带以格线为中心）
                    y1: abs_y + h - bottom_t + band_outset(2, bottom_t),
                    x2: if diag_br {
                        abs_x + w - right_t
                    } else {
                        abs_x + w + bot_r
                    },
                    y2: abs_y + h - bottom_t + band_outset(2, bottom_t),
                    thickness: bottom_t,
                    is_horizontal: true,
                    extend_left: false,
                },
                bottom_style,
                &bottom_color,
            );
            if diag_bl {
                // 左下角 bottom 分摊侧：对角线下方三角
                self.add_corner_triangle(
                    [
                        (abs_x, abs_y + h),
                        (abs_x + left_t, abs_y + h),
                        (abs_x + left_t, abs_y + h - bottom_t),
                    ],
                    &bottom_color,
                );
            }
            if diag_br {
                // 右下角 bottom 分摊侧
                self.add_corner_triangle(
                    [
                        (abs_x + w, abs_y + h),
                        (abs_x + w - right_t, abs_y + h),
                        (abs_x + w - right_t, abs_y + h - bottom_t),
                    ],
                    &bottom_color,
                );
            }
        }

        // 左边框
        if box_node.border_left > 0.0
            && *left_style != BorderStyleValue::None
            && *left_style != BorderStyleValue::Hidden
        {
            self.paint_border_edge(
                &BorderEdgeSpec {
                    // R5262：collapse cell 外缘带外展半厚 + 带端收边到相邻横带的远/近缘
                    x1: abs_x - band_outset(3, left_t),
                    y1: abs_y + top_far,
                    x2: abs_x,
                    y2: abs_y + h - bottom_near,
                    thickness: left_t,
                    is_horizontal: false,
                    extend_left: false,
                },
                left_style,
                &left_color,
            );
            if diag_tl {
                // 左上角 left 分摊侧：对角线下方三角
                self.add_corner_triangle(
                    [(abs_x, abs_y), (abs_x, abs_y + top_t), (abs_x + left_t, abs_y + top_t)],
                    &left_color,
                );
            }
            if diag_bl {
                // 左下角 left 分摊侧：对角线上方三角
                self.add_corner_triangle(
                    [
                        (abs_x, abs_y + h),
                        (abs_x, abs_y + h - bottom_t),
                        (abs_x + left_t, abs_y + h - bottom_t),
                    ],
                    &left_color,
                );
            }
        }
    }

    /// R5083：角部对角分色三角（css-backgrounds-3 §4.4，chromium mitre join 语义）。
    fn add_corner_triangle(&mut self, pts: [(f32, f32); 3], color: &ColorValue) {
        self.primitives.add_path_fill(
            pts.iter().flat_map(|p| [p.0, p.1]).collect(),
            color_value_to_render(color),
        );
    }

    /// R5024：border-image 片发射——图像图元 +（filter() 图像函数时）紧随滤镜图元
    ///（draw_order 插入序：阴影只作用于已绘本片像素，「under」合成由 CPU raster
    /// DropShadow 臂的 covered 跳过承担，R5023）。`src` = 本片源子矩形（归一化；
    /// None = 整图映射回退）——阴影形状采样按该片子矩形逆映射（filter() 先滤镜后
    /// 切片，每片阴影内容 = 滤镜结果图对应源子区，见 #funcdef-filter）。
    fn add_border_image_piece(
        &mut self,
        prim: ImagePrimitive,
        src: Option<Rect>,
        filters: &[FilterComputedValue],
        key: u64,
    ) {
        let rect = prim.rect;
        self.primitives.add_image(prim);
        let kinds: Vec<_> = filters
            .iter()
            .filter_map(super::effects::filter_computed_to_kind)
            .collect();
        if !kinds.is_empty() {
            self.primitives.add_filter(FilterPrimitive {
                rect,
                filters: kinds,
                raster_drop_shadow_source: Some((
                    ImageKey::new(key),
                    rect,
                    src.unwrap_or(Rect::new(0.0, 0.0, 1.0, 1.0)),
                )),
                clip_polygon: None,
            });
        }
    }

    /// 绘制 border-image。
    ///
    /// 当 border-image-source 不为 none 时，将图片按 slice 分割为
    /// 9 个区域（4 角 + 4 边 + 中心），分别绘制到边框的对应区域。
    /// 支持所有 border-image-repeat 模式：stretch/repeat/round/space。
    pub(super) fn paint_border_image(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
        // R5024：filter() 图像函数剥壳（filter-effects-1 #FilterCSSImageValue）——滤镜
        // 产出 <image> 后整体走 9-slice（切片作用于滤镜结果图）。嵌套 filter 取最外层
        // 滤镜列表 + 最内层图像（嵌套滤镜组合深域，FIXME 按需细化）。
        let mut source = &style.border_image_source;
        let mut layer_filters: &[FilterComputedValue] = &[];
        while let BorderImageSourceComputedValue::Filtered { image, filters } = source {
            if layer_filters.is_empty() {
                layer_filters = filters;
            }
            source = image;
        }
        let url = match source {
            BorderImageSourceComputedValue::None => return,
            BorderImageSourceComputedValue::Url(u) => u.clone(),
            // R3909：gradient 源按同款 9-slice 几何切分绘制（css-backgrounds-3 §6.1）。
            // 渐变无固有尺寸——一次构造覆盖**整个 border-image 绘制区**（含 outset）的
            // 基础图元，随后逐片以 clip 窗口发射（crop 语义，渐变绝对坐标定义不变）。
            BorderImageSourceComputedValue::Gradient(gradient) => {
                self.paint_border_image_gradient(box_node, abs_x, abs_y, style, gradient, layer_filters);
                return;
            }
            // while-let 剥壳后不可达（防御臂保持穷尽）。
            BorderImageSourceComputedValue::Filtered { .. } => return,
        };

        let bt = box_node.border_top;
        let br = box_node.border_right;
        let bb = box_node.border_bottom;
        let bl = box_node.border_left;

        // R3906：border-style:none / border-width:0 **不**禁用 border-image（CSS Backgrounds 3
        // §6.1「the border-style: none will make it disappear — however, border-image-width/outset
        // can alter the border area into which the image is drawn」）。此前的「至少有一条边框才
        // 绘制」整体早退把显式 border-image-width（length/percent）创建的绘制区一并丢掉
        //（driving: border-image-width-005..007 同值 2.10%——`border-width:0 + width:50px` 应
        // 绘出延伸进 padding/margin 的绿方块，ZW 全白）。移除整体早退：默认 width=Number(1.0)
        // 在零边框下厚度 0、下方逐块 `> 0` 守卫照旧不绘（与旧行为逐字节一致）；仅显式
        // width/outset 创建面积时新增绘制，零回归面。

        let key = image_resource_key(&url, self.document_url.as_deref());

        // R3906：9-slice 源矩形（CSS Backgrounds 3 §6.2 border-image-slice）。旧实现忽略
        // slice 偏移——每片把**整张源图**拉伸到目标片（whole-image 拉伸近似），slice 切出的
        // 源区域语义丢失（border-image-slice/repeat 族红）。现按 slice 值把源图切成 9 片，
        // 每片 dest rect 携带对应源子矩形（ImagePrimitive.source），渲染器只重采样该区域。
        // Number = 源图像素；Percent = 源图对应轴百分比。源图尺寸取 image_sizes 解码缓存，
        // 未解码（None）时回退整图映射（旧行为，不因缺尺寸出黑片）。
        // 源图尺寸优先固有尺寸缓存；ratio-only SVG 等无确定固有尺寸的源回退自然位图
        // 尺寸（usvg tree.size = viewBox 尺寸——border-image slice 的参照就是源图）。
        let img_size = self
            .get_image_size(key)
            .or_else(|| self.image_natural_sizes.get(&key).copied());
        let slice_src = |c: &BorderImageSliceComputedComponent, total: f32| -> f32 {
            match c {
                BorderImageSliceComputedComponent::Number(n) => *n,
                BorderImageSliceComputedComponent::Percent(p) => p / 100.0 * total,
            }
        };
        // R5084：css-backgrounds-3 §5.2 切片重叠语义 + §5.6 绘制过程（middle 缩放因子
        // 与 repeat 平铺）。kill-switch ZW_BORDER_IMAGE_DRAW_PROCESS=0 恢复旧近似
        //（重叠等比缩减使边片非空 + middle 恒 stretch 整区域）。
        let draw_process = std::env::var("ZW_BORDER_IMAGE_DRAW_PROCESS").as_deref() != Ok("0");

        // 切片几何 = (iw, ih, st, sr, sb, sl)；nine_slice 与 middle 因子共用。
        let slice_geom: Option<(f32, f32, f32, f32, f32, f32)> = img_size.and_then(|(iw, ih)| {
            if iw <= 0.0 || ih <= 0.0 {
                return None;
            }
            let st = slice_src(&style.border_image_slice.top, ih);
            let sr = slice_src(&style.border_image_slice.right, iw);
            let sb = slice_src(&style.border_image_slice.bottom, ih);
            let sl = slice_src(&style.border_image_slice.left, iw);
            Some((iw, ih, st, sr, sb, sl))
        });
        let nine_slice: Option<NineSliceSources> = slice_geom.map(|(iw, ih, st, sr, sb, sl)| {
            // css-backgrounds-3 §5.2（R5084）：切片区域允许重叠——「if the sum of the
            // right and left widths is equal to or greater than the width of the image,
            // the images for the top and bottom edge and the middle part are empty」
            //（top+bottom ≥ 图像高时对 left/right 边同理）。实现：角部切片保持原值
            //（可整幅重叠），边/middle 源矩形宽高 `.max(0.0)` 归零 → 发射侧按零面积
            // 跳过（驱动案 border-image-006 / border-image-slice-006：slice 全幅 →
            // 仅四角绘制整图，边带透明）。旧路径按比例缩减使边片非空（多绘整带）。
            let (sl, sr) = if !draw_process && sl + sr > iw {
                let k = iw / (sl + sr);
                (sl * k, sr * k)
            } else {
                (sl, sr)
            };
            let (st, sb) = if !draw_process && st + sb > ih {
                let k = ih / (st + sb);
                (st * k, sb * k)
            } else {
                (st, sb)
            };
            // source 矩形用**归一化 [0,1] 坐标**（R3906：SVG 源会被按目标尺寸重栅格化，
            // 位图坐标与源图 px 坐标脱钩；归一化坐标对两类源一致）。
            let nx = |v: f32| v / iw;
            let ny = |v: f32| v / ih;
            NineSliceSources {
                top_left: Rect::new(0.0, 0.0, nx(sl), ny(st)),
                top_right: Rect::new(nx(iw - sr), 0.0, nx(sr), ny(st)),
                bottom_right: Rect::new(nx(iw - sr), ny(ih - sb), nx(sr), ny(sb)),
                bottom_left: Rect::new(0.0, ny(ih - sb), nx(sl), ny(sb)),
                top: Rect::new(nx(sl), 0.0, nx((iw - sl - sr).max(0.0)), ny(st)),
                right: Rect::new(nx(iw - sr), ny(st), nx(sr), ny((ih - st - sb).max(0.0))),
                bottom: Rect::new(nx(sl), ny(ih - sb), nx((iw - sl - sr).max(0.0)), ny(sb)),
                left: Rect::new(0.0, ny(st), nx(sl), ny((ih - st - sb).max(0.0))),
                center: Rect::new(nx(sl), ny(st), nx((iw - sl - sr).max(0.0)), ny((ih - st - sb).max(0.0))),
            }
        });
        // R5084：零面积源片（slice 0 / §5.2 重叠置空）不绘——此前零宽/高源矩形逐片
        // 整图映射回退（fill-002 驱动案八片全绘整图）。kill-switch 保持旧回退。
        let piece_drawable = |src: Option<Rect>| -> bool {
            if !draw_process {
                return true;
            }
            match src {
                None => true,
                Some(r) => r.size.width > 0.0 && r.size.height > 0.0,
            }
        };

        // 辅助：创建 ImagePrimitive（每次创建新的 ImageKey，因为 ImageKey 不是 Copy）。
        // src = 该片对应的源子矩形；None = 整图映射（无解码尺寸时的回退）。
        let make_img = |rect: Rect, src: Option<Rect>| ImagePrimitive {
            rect,
            image_key: ImageKey::new(key),
            clip: None,
            source: src,
            clip_radius: None,
        };
        // 按片取源矩形的便捷闭包（nine_slice 为 None 时返回 None → 整图映射）。
        let src_of = |pick: fn(&NineSliceSources) -> &Rect| -> Option<Rect> { nine_slice.as_ref().map(|s| *pick(s)) };

        // border-image-outset（CSS Backgrounds 3 §7.2）：border-image 绘制区向外扩展。
        // Number = × 对应边框宽度；Length = 已解析 px。各边 outset 把外矩形向外移，
        // 9 宫格（角/边/中心）相对外矩形定位 → 仅调整 bx/by/w/h 即可（其余公式不变）。
        let outset_px = |c: &BorderImageOutsetComputedComponent, bw: f32| -> f32 {
            match c {
                BorderImageOutsetComputedComponent::Number(n) => n * bw,
                BorderImageOutsetComputedComponent::Length(l) => *l,
            }
        };
        let o_top = outset_px(&style.border_image_outset.top, bt);
        let o_right = outset_px(&style.border_image_outset.right, br);
        let o_bottom = outset_px(&style.border_image_outset.bottom, bb);
        let o_left = outset_px(&style.border_image_outset.left, bl);

        // 边框区域坐标（含 outset 外扩；默认 outset=0 → bx=abs_x/by=abs_y，旧行为不变）。
        let bx = abs_x - o_left;
        let by = abs_y - o_top;
        let w = box_node.width + o_left + o_right;
        let h = box_node.height + o_top + o_bottom;

        // border-image-width（CSS Backgrounds 3 §7.3）：覆盖 9 宫格区域厚度（绘制用）。
        // 默认 Number(1.0) = 1×边框宽度 = 边框宽度 → 区域厚度同边框（旧行为）；显式值则
        // 用指定厚度绘制（可大于边框延伸进 padding/content，或小于）。Auto 简化为边框宽度
        //（真 auto 用图像固有尺寸，ZW paint 期无解码尺寸）；Percent 相对边框盒对应轴。
        // shadow bt/br/bb/bl → 下方 9 宫格公式（角/边/中心）自动用新厚度，guard/outset 既用原值不变。
        let width_px = |c: &BorderImageWidthComputedComponent, bw: f32, box_dim: f32| -> f32 {
            match c {
                BorderImageWidthComputedComponent::Auto => bw,
                BorderImageWidthComputedComponent::Number(n) => n * bw,
                BorderImageWidthComputedComponent::Length(l) => *l,
                BorderImageWidthComputedComponent::Percent(p) => (p / 100.0) * box_dim,
            }
        };
        let bt = width_px(&style.border_image_width.top, bt, h);
        let br = width_px(&style.border_image_width.right, br, w);
        let bb = width_px(&style.border_image_width.bottom, bb, h);
        let bl = width_px(&style.border_image_width.left, bl, w);

        // 四条边的尺寸
        let edge_h_w = (w - bl - br).max(0.0);
        let edge_v_h = (h - bt - bb).max(0.0);

        let h_mode = &style.border_image_repeat.horizontal;
        let v_mode = &style.border_image_repeat.vertical;
        let fill = style.border_image_slice.fill;

        // 中心区域（fill 时绘制）。css-backgrounds-3 §5.6 step 1（R5084）：middle 宽的
        // 缩放因子取 top 片（「the middle image's width is scaled by the same factor as
        // the top image」），top 片零切片（因子零/无穷）依次回退 bottom → 仍无则不缩放
        //（自然尺寸）；高同型取 left → right → 不缩放。step 2：stretch 拉伸到区域；
        // repeat/space/round 按自然 tile 平铺（裁剪到区域时 src 同步取子矩形——每 tile
        // 只采样窗口对应源区，fill-001/002/003 驱动案由此把 green 中窗对准 middle 区）。
        // kill-switch 走旧「恒 stretch」。
        let center_src = src_of(|s| &s.center);
        if fill && edge_h_w > 0.0 && edge_v_h > 0.0 && piece_drawable(center_src) {
            let region = Rect::new(bx + bl, by + bt, edge_h_w, edge_v_h);
            let drawn = match (draw_process, slice_geom) {
                (true, Some((iw, ih, st, sr, sb, sl))) => {
                    let mid_w_src = (iw - sl - sr).max(0.0);
                    let mid_h_src = (ih - st - sb).max(0.0);
                    // step 1 缩放因子：切片与区域均须非零（零 = 因子退化，按规格回退）。
                    let factor = |slice: f32, piece: f32| -> Option<f32> {
                        if slice > 0.0 && piece > 0.0 {
                            Some(piece / slice)
                        } else {
                            None
                        }
                    };
                    let fw = factor(st, bt).or_else(|| factor(sb, bb));
                    let fh = factor(sl, bl).or_else(|| factor(sr, br));
                    let tile_w = fw.map_or(mid_w_src, |f| mid_w_src * f);
                    let tile_h = fh.map_or(mid_h_src, |f| mid_h_src * f);
                    let xs = Self::middle_tile_spans(edge_h_w, tile_w, h_mode);
                    let ys = Self::middle_tile_spans(edge_v_h, tile_h, v_mode);
                    for &(dx, tw) in &xs {
                        for &(dy, th) in &ys {
                            // tile 起点平移到绝对坐标（spans 相对 region 原点）。
                            let x = region.origin.x + dx;
                            let y = region.origin.y + dy;
                            // dest tile ∩ middle region（复用边片 clip_tile）；src 按
                            // 窗口比例同步裁剪——每 tile 只采样窗口对应源区。
                            let Some((vx, vy, vw, vh)) = Self::clip_tile(
                                x,
                                y,
                                tw,
                                th,
                                region.origin.x,
                                region.origin.y,
                                region.size.width,
                                region.size.height,
                            ) else {
                                continue;
                            };
                            let src_clip = center_src.map(|s| {
                                Rect::new(
                                    s.origin.x + (vx - x) / tw * s.size.width,
                                    s.origin.y + (vy - y) / th * s.size.height,
                                    vw / tw * s.size.width,
                                    vh / th * s.size.height,
                                )
                            });
                            self.add_border_image_piece(
                                make_img(Rect::new(vx, vy, vw, vh), src_clip),
                                src_clip,
                                layer_filters,
                                key,
                            );
                        }
                    }
                    true
                }
                _ => false,
            };
            if !drawn {
                self.add_border_image_piece(make_img(region, center_src), center_src, layer_filters, key);
            }
        }

        // 四个角（始终 stretch，不受 repeat 模式影响；R5084 零面积源片跳过）
        if bl > 0.0 && bt > 0.0 {
            let src = src_of(|s| &s.top_left);
            if piece_drawable(src) {
                self.add_border_image_piece(make_img(Rect::new(bx, by, bl, bt), src), src, layer_filters, key);
            }
        }
        if br > 0.0 && bt > 0.0 {
            let src = src_of(|s| &s.top_right);
            if piece_drawable(src) {
                self.add_border_image_piece(
                    make_img(Rect::new(bx + w - br, by, br, bt), src),
                    src,
                    layer_filters,
                    key,
                );
            }
        }
        if br > 0.0 && bb > 0.0 {
            let src = src_of(|s| &s.bottom_right);
            if piece_drawable(src) {
                self.add_border_image_piece(
                    make_img(Rect::new(bx + w - br, by + h - bb, br, bb), src),
                    src,
                    layer_filters,
                    key,
                );
            }
        }
        if bl > 0.0 && bb > 0.0 {
            let src = src_of(|s| &s.bottom_left);
            if piece_drawable(src) {
                self.add_border_image_piece(
                    make_img(Rect::new(bx, by + h - bb, bl, bb), src),
                    src,
                    layer_filters,
                    key,
                );
            }
        }

        // 四条边 — 根据 border-image-repeat 模式生成图元

        // 上边（水平 repeat 模式）
        if edge_h_w > 0.0 && bt > 0.0 {
            let src = src_of(|s| &s.top);
            if piece_drawable(src) {
                self.paint_border_image_edge_h(
                    &make_img,
                    src,
                    layer_filters,
                    key,
                    bx + bl,
                    by,
                    edge_h_w,
                    bt,
                    bl, // 自然 tile 宽度 = 左边框宽度
                    h_mode,
                );
            }
        }
        // 右边（垂直 repeat 模式）
        if br > 0.0 && edge_v_h > 0.0 {
            let src = src_of(|s| &s.right);
            if piece_drawable(src) {
                self.paint_border_image_edge_v(
                    &make_img,
                    src,
                    layer_filters,
                    key,
                    bx + w - br,
                    by + bt,
                    br,
                    edge_v_h,
                    bt, // 自然 tile 高度 = 上边框高度
                    v_mode,
                );
            }
        }
        // 下边（水平 repeat 模式）
        if edge_h_w > 0.0 && bb > 0.0 {
            let src = src_of(|s| &s.bottom);
            if piece_drawable(src) {
                self.paint_border_image_edge_h(
                    &make_img,
                    src,
                    layer_filters,
                    key,
                    bx + bl,
                    by + h - bb,
                    edge_h_w,
                    bb,
                    bl, // 自然 tile 宽度 = 左边框宽度
                    h_mode,
                );
            }
        }
        // 左边（垂直 repeat 模式）
        if bl > 0.0 && edge_v_h > 0.0 {
            let src = src_of(|s| &s.left);
            if piece_drawable(src) {
                self.paint_border_image_edge_v(
                    &make_img,
                    src,
                    layer_filters,
                    key,
                    bx,
                    by + bt,
                    bl,
                    edge_v_h,
                    bt, // 自然 tile 高度 = 上边框高度
                    v_mode,
                );
            }
        }
    }

    /// R3909：gradient border-image-source 的 9-slice 绘制（css-backgrounds-3 §6.1）。
    ///
    /// 渐变无固有尺寸——slice 参照源图的语义对渐变退化为「源 = 绘制区本身」，与
    /// chromium 行为一致（`border-image: linear-gradient(...) 1 fill` 即整图 9 等分）。
    /// 实现：按 URL 路径同款的 border-image-width/outset/repeat 计算出各片 dest rect，
    /// 基础渐变图元覆盖**整个绘制区**（含 outset 外扩矩形），每片发射一个 clip = 片
    /// rect 的克隆（GradientPrimitive.clip = crop 语义，渲染器只画交集、t 值仍按完整
    /// 绘制区解析——切片不重定义渐变坐标）。
    fn paint_border_image_gradient(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
        gradient: &zero_css_parser::values::GradientValue,
        filters: &[FilterComputedValue],
    ) {
        use super::super::helpers::gradient_to_primitive_with_font_size;

        let bt = box_node.border_top;
        let br = box_node.border_right;
        let bb = box_node.border_bottom;
        let bl = box_node.border_left;

        // 边框区域（含 outset 外扩）——与 URL 路径同公式（CSS Backgrounds 3 §7.2）。
        let outset_px = |c: &BorderImageOutsetComputedComponent, bw: f32| -> f32 {
            match c {
                BorderImageOutsetComputedComponent::Number(n) => n * bw,
                BorderImageOutsetComputedComponent::Length(l) => *l,
            }
        };
        let o_top = outset_px(&style.border_image_outset.top, bt);
        let o_right = outset_px(&style.border_image_outset.right, br);
        let o_bottom = outset_px(&style.border_image_outset.bottom, bb);
        let o_left = outset_px(&style.border_image_outset.left, bl);
        let bx = abs_x - o_left;
        let by = abs_y - o_top;
        let w = box_node.width + o_left + o_right;
        let h = box_node.height + o_top + o_bottom;

        // border-image-width（§7.3）——与 URL 路径同公式。
        let width_px = |c: &BorderImageWidthComputedComponent, bw: f32, box_dim: f32| -> f32 {
            match c {
                BorderImageWidthComputedComponent::Auto => bw,
                BorderImageWidthComputedComponent::Number(n) => n * bw,
                BorderImageWidthComputedComponent::Length(l) => *l,
                BorderImageWidthComputedComponent::Percent(p) => (p / 100.0) * box_dim,
            }
        };
        let bt = width_px(&style.border_image_width.top, bt, h);
        let br = width_px(&style.border_image_width.right, br, w);
        let bb = width_px(&style.border_image_width.bottom, bb, h);
        let bl = width_px(&style.border_image_width.left, bl, w);

        let area = Rect::new(bx, by, w, h);
        let font_size = zero_style_system::computed::resolve_length(&style.font_size, 16.0, None, None);
        let base = match gradient_to_primitive_with_font_size(gradient, &area, &style.color, font_size as f32) {
            Some(g) => g,
            None => return, // conic 等不支持的渐变类型：等同 none（不 panic）
        };
        let emit = |p: &mut Self, rect: Rect| {
            if rect.size.width <= 0.0 || rect.size.height <= 0.0 {
                return;
            }
            let mut piece = base.clone();
            piece.rect = rect;
            piece.clip = Some(rect);
            p.primitives.add_gradient(piece);
            // R5024：filter() 图像函数渐变源——滤镜图元紧随片图元（R5021 同式）。
            // raster 源 = None（渐变不透明源回退既有臂，DropShadow CPU 面跳过——
            // 与背景/内容渐变源一致）。
            let kinds: Vec<_> = filters
                .iter()
                .filter_map(super::effects::filter_computed_to_kind)
                .collect();
            if !kinds.is_empty() {
                p.primitives.add_filter(FilterPrimitive {
                    rect,
                    filters: kinds,
                    raster_drop_shadow_source: None,
                    clip_polygon: None,
                });
            }
        };

        let edge_h_w = (w - bl - br).max(0.0);
        let edge_v_h = (h - bt - bb).max(0.0);
        let h_mode = &style.border_image_repeat.horizontal;
        let v_mode = &style.border_image_repeat.vertical;

        // 四角（始终 stretch；repeat 只作用于边）
        if bl > 0.0 && bt > 0.0 {
            emit(self, Rect::new(bx, by, bl, bt));
        }
        if br > 0.0 && bt > 0.0 {
            emit(self, Rect::new(bx + w - br, by, br, bt));
        }
        if br > 0.0 && bb > 0.0 {
            emit(self, Rect::new(bx + w - br, by + h - bb, br, bb));
        }
        if bl > 0.0 && bb > 0.0 {
            emit(self, Rect::new(bx, by + h - bb, bl, bb));
        }

        // 中心（fill 关键字；始终 stretch 覆盖中央区域）
        if style.border_image_slice.fill && edge_h_w > 0.0 && edge_v_h > 0.0 {
            emit(self, Rect::new(bx + bl, by + bt, edge_h_w, edge_v_h));
        }

        // 四条边：沿边轴按 repeat 模式划分 tile 序列，每 tile 独立 clip 发射。
        // tile 的自然尺寸取对应边框厚度（渐变源无 tile 内在尺寸，与 URL 路径同取法）。
        let tiles_on_axis = |total: f32, tile: f32, mode: &BorderImageRepeatComputedMode| -> Vec<(f32, f32)> {
            match mode {
                BorderImageRepeatComputedMode::Stretch => vec![(total, 0.0)],
                BorderImageRepeatComputedMode::Repeat => {
                    let n = (total / tile).ceil().max(1.0) as usize;
                    let total_tiles = n as f32 * tile;
                    let start = (total - total_tiles) / 2.0;
                    (0..n).map(|i| (tile, start + i as f32 * tile)).collect()
                }
                BorderImageRepeatComputedMode::Round => {
                    let n = (total / tile).round().max(1.0) as usize;
                    let stretched = total / n as f32;
                    (0..n).map(|i| (stretched, i as f32 * stretched)).collect()
                }
                BorderImageRepeatComputedMode::Space => {
                    let n = (total / tile).floor().max(0.0) as usize;
                    if n <= 1 {
                        vec![(total, 0.0)]
                    } else {
                        let gap = (total - n as f32 * tile) / (n + 1) as f32;
                        (0..n).map(|i| (tile, gap + i as f32 * (tile + gap))).collect()
                    }
                }
            }
        };

        // 上/下边（水平轴 tile 划分；tile 间按边框区域裁剪不越界）
        for (edge_y, edge_h) in [(by, bt), (by + h - bb, bb)] {
            if edge_h <= 0.0 || edge_h_w <= 0.0 {
                continue;
            }
            for (tile_w, off) in tiles_on_axis(edge_h_w, bl, h_mode) {
                let tx = (bx + bl + off).max(bx + bl);
                let right = (tx + tile_w).min(bx + bl + edge_h_w);
                if right > tx {
                    emit(self, Rect::new(tx, edge_y, right - tx, edge_h));
                }
            }
        }
        // 左/右边（垂直轴 tile 划分）
        for (edge_x, edge_w) in [(bx, bl), (bx + w - br, br)] {
            if edge_w <= 0.0 || edge_v_h <= 0.0 {
                continue;
            }
            for (tile_h, off) in tiles_on_axis(edge_v_h, bt, v_mode) {
                let ty = (by + bt + off).max(by + bt);
                let bottom = (ty + tile_h).min(by + bt + edge_v_h);
                if bottom > ty {
                    emit(self, Rect::new(edge_x, ty, edge_w, bottom - ty));
                }
            }
        }
    }

    /// 绘制水平方向的 border-image 边（上边/下边）。
    ///
    /// `tile_w` 是单个 tile 的自然宽度（对应边框宽度），
    /// `total_w` 是需要覆盖的总宽度。
    #[allow(clippy::too_many_arguments)]
    fn paint_border_image_edge_h(
        &mut self,
        make_img: &impl Fn(Rect, Option<Rect>) -> ImagePrimitive,
        src: Option<Rect>,
        filters: &[FilterComputedValue],
        key: u64,
        start_x: f32,
        y: f32,
        total_w: f32,
        edge_h: f32,
        tile_w: f32,
        mode: &BorderImageRepeatComputedMode,
    ) {
        match mode {
            BorderImageRepeatComputedMode::Stretch => {
                // 拉伸单个 tile 覆盖整条边
                let rect = Rect::new(start_x, y, total_w, edge_h);
                self.add_border_image_piece(make_img(rect, src), src, filters, key);
            }
            BorderImageRepeatComputedMode::Repeat => {
                // 以自然 tile 大小重复，从中心向两边展开
                let n = (total_w / tile_w).ceil().max(1.0) as usize;
                let total_tiles_w = n as f32 * tile_w;
                let mut x = start_x + (total_w - total_tiles_w) / 2.0;
                for _ in 0..n {
                    let clipped = Self::clip_tile(x, y, tile_w, edge_h, start_x, y, total_w, edge_h);
                    if let Some((cx, cy, cw, ch)) = clipped {
                        let rect = Rect::new(cx, cy, cw, ch);
                        self.add_border_image_piece(make_img(rect, src), src, filters, key);
                    }
                    x += tile_w;
                }
            }
            BorderImageRepeatComputedMode::Round => {
                // 拉伸 tile 使整数个刚好覆盖
                let n = (total_w / tile_w).round().max(1.0) as usize;
                let stretched = total_w / n as f32;
                let mut x = start_x;
                for _ in 0..n {
                    let rect = Rect::new(x, y, stretched, edge_h);
                    self.add_border_image_piece(make_img(rect, src), src, filters, key);
                    x += stretched;
                }
            }
            BorderImageRepeatComputedMode::Space => {
                // 均匀分布 tile，不足 2 个时退化为 stretch
                let n = (total_w / tile_w).floor().max(0.0) as usize;
                if n <= 1 {
                    let rect = Rect::new(start_x, y, total_w, edge_h);
                    self.add_border_image_piece(make_img(rect, src), src, filters, key);
                } else {
                    let gap = (total_w - n as f32 * tile_w) / (n + 1) as f32;
                    let mut x = start_x + gap;
                    for _ in 0..n {
                        let rect = Rect::new(x, y, tile_w, edge_h);
                        self.add_border_image_piece(make_img(rect, src), src, filters, key);
                        x += tile_w + gap;
                    }
                }
            }
        }
    }

    /// 绘制垂直方向的 border-image 边（左边/右边）。
    #[allow(clippy::too_many_arguments)]
    fn paint_border_image_edge_v(
        &mut self,
        make_img: &impl Fn(Rect, Option<Rect>) -> ImagePrimitive,
        src: Option<Rect>,
        filters: &[FilterComputedValue],
        key: u64,
        x: f32,
        start_y: f32,
        edge_w: f32,
        total_h: f32,
        tile_h: f32,
        mode: &BorderImageRepeatComputedMode,
    ) {
        match mode {
            BorderImageRepeatComputedMode::Stretch => {
                let rect = Rect::new(x, start_y, edge_w, total_h);
                self.add_border_image_piece(make_img(rect, src), src, filters, key);
            }
            BorderImageRepeatComputedMode::Repeat => {
                let n = (total_h / tile_h).ceil().max(1.0) as usize;
                let total_tiles_h = n as f32 * tile_h;
                let mut y = start_y + (total_h - total_tiles_h) / 2.0;
                for _ in 0..n {
                    let clipped = Self::clip_tile(x, y, edge_w, tile_h, x, start_y, edge_w, total_h);
                    if let Some((cx, cy, cw, ch)) = clipped {
                        let rect = Rect::new(cx, cy, cw, ch);
                        self.add_border_image_piece(make_img(rect, src), src, filters, key);
                    }
                    y += tile_h;
                }
            }
            BorderImageRepeatComputedMode::Round => {
                let n = (total_h / tile_h).round().max(1.0) as usize;
                let stretched = total_h / n as f32;
                let mut y = start_y;
                for _ in 0..n {
                    let rect = Rect::new(x, y, edge_w, stretched);
                    self.add_border_image_piece(make_img(rect, src), src, filters, key);
                    y += stretched;
                }
            }
            BorderImageRepeatComputedMode::Space => {
                let n = (total_h / tile_h).floor().max(0.0) as usize;
                if n <= 1 {
                    let rect = Rect::new(x, start_y, edge_w, total_h);
                    self.add_border_image_piece(make_img(rect, src), src, filters, key);
                } else {
                    let gap = (total_h - n as f32 * tile_h) / (n + 1) as f32;
                    let mut y = start_y + gap;
                    for _ in 0..n {
                        let rect = Rect::new(x, y, edge_w, tile_h);
                        self.add_border_image_piece(make_img(rect, src), src, filters, key);
                        y += tile_h + gap;
                    }
                }
            }
        }
    }

    /// 裁剪单个 tile 到边界区域，返回裁剪后的 (x, y, w, h)。
    /// 如果 tile 完全在边界外返回 None。
    #[allow(clippy::too_many_arguments)]
    /// R5084：middle 片单轴 tile 序列（css-backgrounds-3 §5.6 step 2/3/4）。返回
    /// (起点, 宽度) 对——stretch = 单片整区域；repeat = 自然 tile 居中向外重复
    ///（§5.6 step 3「centered」）；round = 重标定 tile 使整数个恰好覆盖；space =
    /// 均匀分布（不足一个 tile 时按边片同款约定退化为 stretch）。
    fn middle_tile_spans(total: f32, tile: f32, mode: &BorderImageRepeatComputedMode) -> Vec<(f32, f32)> {
        if tile <= 0.0 || total <= 0.0 {
            return vec![(0.0, total)];
        }
        match mode {
            BorderImageRepeatComputedMode::Stretch => vec![(0.0, total)],
            BorderImageRepeatComputedMode::Repeat => {
                let n = (total / tile).ceil().max(1.0) as usize;
                let total_tiles = n as f32 * tile;
                let start = (total - total_tiles) / 2.0;
                (0..n).map(|i| (start + i as f32 * tile, tile)).collect()
            }
            BorderImageRepeatComputedMode::Round => {
                let n = ((total / tile).round() as usize).max(1);
                let stretched = total / n as f32;
                (0..n).map(|i| (i as f32 * stretched, stretched)).collect()
            }
            BorderImageRepeatComputedMode::Space => {
                let n = (total / tile).floor().max(0.0) as usize;
                if n <= 1 {
                    vec![(0.0, total)]
                } else {
                    let gap = (total - n as f32 * tile) / (n + 1) as f32;
                    (0..n).map(|i| (gap + i as f32 * (tile + gap), tile)).collect()
                }
            }
        }
    }

    fn clip_tile(
        tile_x: f32,
        tile_y: f32,
        tile_w: f32,
        tile_h: f32,
        clip_x: f32,
        clip_y: f32,
        clip_w: f32,
        clip_h: f32,
    ) -> Option<(f32, f32, f32, f32)> {
        let tile_right = tile_x + tile_w;
        let tile_bottom = tile_y + tile_h;
        let clip_right = clip_x + clip_w;
        let clip_bottom = clip_y + clip_h;

        if tile_right <= clip_x || tile_x >= clip_right || tile_bottom <= clip_y || tile_y >= clip_bottom {
            return None;
        }

        let cx = tile_x.max(clip_x);
        let cy = tile_y.max(clip_y);
        let cw = tile_right.min(clip_right) - cx;
        let ch = tile_bottom.min(clip_bottom) - cy;

        if cw > 0.0 && ch > 0.0 {
            Some((cx, cy, cw, ch))
        } else {
            None
        }
    }

    /// R1141：dashed/dotted border 用 StrokePrimitive（线居中）绘制，但 border 应在 border-box
    /// 内侧（同 Solid 的 fill rect 语义）。返回使 stroke 中心从「边界线」移到「内侧填充区中心」
    /// 的偏移：horizontal（top/bottom）→ y += thickness/2；vertical 左边框 → x += thickness/2；
    /// vertical 右边框（extend_left）→ x -= thickness/2。
    fn stroke_inward_offset(spec: &BorderEdgeSpec) -> (f32, f32) {
        let half = spec.thickness / 2.0;
        if spec.is_horizontal {
            (0.0, half)
        } else if spec.extend_left {
            (-half, 0.0)
        } else {
            (half, 0.0)
        }
    }

    /// 绘制单条边框（根据 border-style 生成合适的图元）。
    pub(super) fn paint_border_edge(
        &mut self,
        spec: &BorderEdgeSpec,
        border_style: &BorderStyleValue,
        color: &ColorValue,
    ) {
        let render_color = color_value_to_render(color);

        match border_style {
            BorderStyleValue::None | BorderStyleValue::Hidden => {}
            BorderStyleValue::Solid => {
                self.primitives.add_fill(self.border_fill_rect(spec), render_color);
            }
            BorderStyleValue::Dotted => {
                // R1141：stroke 默认以线为中心（width 两侧各半），但 border 应在 border-box
                // 内侧（同 Solid 的 border_fill_rect：从 y1/x1 向厚度方向延伸）。offset stroke
                // 中心 inward 使 dashed/dotted 与 solid 定位一致：horizontal → y += thickness/2；
                // vertical extend_left（右边框）→ x -= thickness/2；vertical else（左边框）→ x += thickness/2。
                // 旧未 offset 致 dashed/dotted border 半宽溢出 border-box（top border y=30 w5
                // 居中 y[27.5,32.5] 而非内侧 y[30,35]，position-*-root-element dashed border -3px 偏移）。
                let (dx, dy) = Self::stroke_inward_offset(spec);
                self.primitives.add_stroke(StrokePrimitive {
                    x1: spec.x1 + dx,
                    y1: spec.y1 + dy,
                    x2: spec.x2 + dx,
                    y2: spec.y2 + dy,
                    width: spec.thickness,
                    color: render_color,
                    style: LineStyle::Dotted,
                    cap: LineCap::Round,
                });
            }
            BorderStyleValue::Dashed => {
                let (dx, dy) = Self::stroke_inward_offset(spec);
                self.primitives.add_stroke(StrokePrimitive {
                    x1: spec.x1 + dx,
                    y1: spec.y1 + dy,
                    x2: spec.x2 + dx,
                    y2: spec.y2 + dy,
                    width: spec.thickness,
                    color: render_color,
                    style: LineStyle::Dashed,
                    cap: LineCap::Square,
                });
            }
            BorderStyleValue::Double => {
                let gap = (spec.thickness / 3.0).max(1.0);
                let line_w = ((spec.thickness - gap) / 2.0).max(1.0);
                if spec.is_horizontal {
                    self.primitives
                        .add_fill(Rect::new(spec.x1, spec.y1, spec.x2 - spec.x1, line_w), render_color);
                    self.primitives.add_fill(
                        Rect::new(spec.x1, spec.y1 + line_w + gap, spec.x2 - spec.x1, line_w),
                        render_color,
                    );
                } else {
                    let outer_x = if spec.extend_left {
                        spec.x1 - spec.thickness
                    } else {
                        spec.x1
                    };
                    self.primitives
                        .add_fill(Rect::new(outer_x, spec.y1, line_w, spec.y2 - spec.y1), render_color);
                    self.primitives.add_fill(
                        Rect::new(outer_x + line_w + gap, spec.y1, line_w, spec.y2 - spec.y1),
                        render_color,
                    );
                }
            }
            BorderStyleValue::Groove => {
                let (light, dark) = groove_ridge_colors(&render_color);
                self.paint_3d_border(spec, &light, &dark);
            }
            BorderStyleValue::Ridge => {
                let (light, dark) = groove_ridge_colors(&render_color);
                self.paint_3d_border(spec, &dark, &light);
            }
            BorderStyleValue::Inset => {
                let lighter = lighten(&render_color, 0.3);
                let darker = darken(&render_color, 0.3);
                self.paint_3d_border(spec, &darker, &lighter);
            }
            BorderStyleValue::Outset => {
                let lighter = lighten(&render_color, 0.3);
                let darker = darken(&render_color, 0.3);
                self.paint_3d_border(spec, &lighter, &darker);
            }
        }
    }

    /// 根据 BorderEdgeSpec 计算填充矩形。
    fn border_fill_rect(&self, spec: &BorderEdgeSpec) -> Rect {
        if spec.is_horizontal {
            Rect::new(spec.x1, spec.y1, spec.x2 - spec.x1, spec.thickness)
        } else if spec.extend_left {
            Rect::new(spec.x1 - spec.thickness, spec.y1, spec.thickness, spec.y2 - spec.y1)
        } else {
            Rect::new(spec.x1, spec.y1, spec.thickness, spec.y2 - spec.y1)
        }
    }

    /// 绘制 3D 效果边框（groove/ridge/inset/outset 使用）。
    ///
    /// 将边框分为上下两半（水平边）或左右两半（垂直边），分别使用不同颜色。
    fn paint_3d_border(&mut self, spec: &BorderEdgeSpec, first_color: &Color, second_color: &Color) {
        let half = spec.thickness / 2.0;
        if spec.is_horizontal {
            self.primitives
                .add_fill(Rect::new(spec.x1, spec.y1, spec.x2 - spec.x1, half), *first_color);
            self.primitives.add_fill(
                Rect::new(spec.x1, spec.y1 + half, spec.x2 - spec.x1, half),
                *second_color,
            );
        } else {
            let fill_x = if spec.extend_left {
                spec.x1 - spec.thickness
            } else {
                spec.x1
            };
            self.primitives
                .add_fill(Rect::new(fill_x, spec.y1, half, spec.y2 - spec.y1), *first_color);
            self.primitives.add_fill(
                Rect::new(fill_x + half, spec.y1, half, spec.y2 - spec.y1),
                *second_color,
            );
        }
    }

    /// UA 默认键盘焦点环宽度（px）。paint() 末尾合并冲刷同用。
    pub(super) const FOCUS_RING_WIDTH: f32 = 2.0;

    /// 键盘焦点环适用判定：作者未声明非 none 的 outline-style 即适用。
    /// outline-width 初始值 = medium（3px，非零），不能作为「作者已声明」信号；
    /// style=None 时 paint_outline 本就不绘制。「未声明」与「显式 none」在 computed
    /// 层不可区分，保守按作者去除环的意图处理（声明非 none 由 paint_outline 绘制，
    /// 不叠加焦点环）。
    pub(super) fn focus_ring_applicable(&self, style: &ComputedStyle) -> bool {
        use zero_style_system::OutlineStyleValue;
        style.outline_style == OutlineStyleValue::None
    }

    /// 绘制 UA 默认键盘焦点环：作者未声明 outline 的聚焦元素外画 2px 实线环。
    ///
    /// 几何复用 paint_outline 的外侧矩形推导（width=2px、offset=0）；作者已声明
    /// `outline-style`（含显式 none）或 `outline-width` 时不叠加——声明非 none 由
    /// paint_outline 正常绘制，显式 none 视为作者去除环的意图（「未声明」与
    /// 「显式 none」在 computed 层不可区分，保守按作者意图处理）。
    pub(super) fn paint_focus_ring_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        if !self.focus_ring_applicable(style) {
            return;
        }
        let width = Self::FOCUS_RING_WIDTH;
        let color = zero_render_foundation::color::Color::rgb(0x1a, 0x73, 0xe8);
        let w = box_node.width;
        let h = box_node.height;
        let ow = (w + 2.0 * width).max(2.0 * width);
        let oh = (h + 2.0 * width).max(2.0 * width);
        let ox = abs_x + w / 2.0 - ow / 2.0;
        let oy = abs_y + h / 2.0 - oh / 2.0;
        self.paint_ring_rects(ox, oy, ow, oh, width, color);
    }

    /// 行内聚焦元素的焦点环（paint_node 盒路径不覆盖 a/span 等无独立盒的 inline
    /// 元素）：render_fragment 逐行片段调用，几何 = 片段行盒外扩 ring width。
    pub(super) fn paint_inline_focus_ring(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let width = Self::FOCUS_RING_WIDTH;
        let color = zero_render_foundation::color::Color::rgb(0x1a, 0x73, 0xe8);
        self.paint_ring_rects(x - width, y - width, w + 2.0 * width, h + 2.0 * width, width, color);
    }

    /// 焦点环四边填充（盒路径与行内片段路径共用）。
    fn paint_ring_rects(
        &mut self,
        ox: f32,
        oy: f32,
        ow: f32,
        oh: f32,
        width: f32,
        color: zero_render_foundation::color::Color,
    ) {
        self.primitives.add_fill(Rect::new(ox, oy, ow, width), color);
        self.primitives
            .add_fill(Rect::new(ox, oy + oh - width, ow, width), color);
        self.primitives
            .add_fill(Rect::new(ox, oy + width, width, oh - 2.0 * width), color);
        self.primitives
            .add_fill(Rect::new(ox + ow - width, oy + width, width, oh - 2.0 * width), color);
    }

    /// 绘制 outline（位于 border 外侧，支持多种 outline-style）。
    ///
    /// outline 绘制为 4 条边框段，根据 outline-style 生成不同图元类型。
    pub(super) fn paint_outline(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
        use zero_style_system::OutlineStyleValue;
        use zero_style_system::property::types::DisplayValue;

        // CSS2 §17.4：table-column / table-column-group 不生成盒（仅参与列宽计算），
        // 故 outline 不应用——同 R2108「margin 不应用 table-column/cell/column-group」
        // 谱系（display-type exclusion）。driving cluster：outline-applies-to-005/006，
        // 4 个 outline 属性（outline/width/color/style）× 2 display = 8 案全应无红边。
        // 其它 table 类型（row-group/header-group/footer-group/row/cell）生成盒，outline
        // 仍应用（outline-applies-to-001~004 PASS）。
        if matches!(
            style.display,
            DisplayValue::TableColumn | DisplayValue::TableColumnGroup
        ) {
            return;
        }

        let outline_width = resolve_outline_length(&style.outline_width, style);

        if outline_width <= 0.0 || style.outline_style == OutlineStyleValue::None {
            return;
        }

        let offset = if style.outline_offset_inset {
            // CSS-UI-4 §4.4: `outline-offset: inset` ≡ 负 outline-width 偏移，
            // 使 outline 绘制在 border-box 内侧（total_offset = outline_width + (-outline_width) = 0，
            // 既有外扩矩形几何退化为贴 border-box 边内侧绘制）。driving: outline-offset-inset-001/003/004。
            -outline_width
        } else {
            resolve_outline_length(&style.outline_offset, style)
        };

        let w = box_node.width;
        let h = box_node.height;
        let total_offset = outline_width + offset;
        // outline-color 初始 = currentColor（CSS UI：invert 无浏览器支持回落 currentColor），
        // 须按元素自身 color 解析（color_value_to_render 无元素色上下文会回落黑色）。
        let color = resolve_color_current(&style.outline_color, &style.color);

        // R5186（css-ui §outline + css-borders-4 §border-shape，判例 outline /
        // outline-double）：形状活跃时 outline 随形状路径——带 = 路径外向
        // [offset, offset + width]（ref 判例内缘 = 形状路径：circle r50 outline 4px
        // → 带 [50,54]）。solid/auto 单带；double 三等分两条子带（两线 + 中隙，
        // w/3 每段）。等价域（inset round R）不在此——落 border-radius 同构既有路径。
        // offset_convex_polygon 拒绝（非凸 / miter>4）或 fill mode / 其他线型 →
        // 回退下方矩形路径（FIXME: 非凸 bevel 拓扑与 fill mode 外形状带）。
        if crate::paint::helpers::border_shape_border_path_active(style)
            && matches!(
                style.outline_style,
                OutlineStyleValue::Solid | OutlineStyleValue::Auto | OutlineStyleValue::Double
            )
            && crate::paint::helpers::border_shape_round_inset_equivalent_radii(style, box_node).is_none()
            && let Some(crate::paint::helpers::BorderShapePlan::Stroke { vertices, .. }) =
                crate::paint::helpers::border_shape_plan(style, box_node, abs_x, abs_y)
        {
            let fracs: &[(f32, f32)] = if matches!(style.outline_style, OutlineStyleValue::Double) {
                &[(0.0, 1.0 / 3.0), (2.0 / 3.0, 1.0)]
            } else {
                &[(0.0, 1.0)]
            };
            let offset_poly = |d: f32| -> Option<Vec<(f32, f32)>> {
                if d == 0.0 {
                    Some(vertices.clone())
                } else {
                    crate::paint::helpers::offset_convex_polygon(&vertices, d)
                }
            };
            let mut rings = Vec::with_capacity(fracs.len());
            let mut shaped = true;
            for &(f0, f1) in fracs {
                match (
                    offset_poly(offset + f1 * outline_width),
                    offset_poly(offset + f0 * outline_width),
                ) {
                    (Some(outer), Some(inner)) => rings.push(crate::paint::helpers::concat_ring_vertices(outer, inner)),
                    _ => {
                        shaped = false;
                        break;
                    }
                }
            }
            if shaped {
                for ring in rings {
                    let flat: Vec<f32> = ring.into_iter().flat_map(|(x, y)| [x, y]).collect();
                    self.primitives.add_path_fill(flat, color);
                }
                return;
            }
        }

        // 计算外侧矩形坐标。
        // R4196（css-ui-3 §outline-props，outline-013）：outline 形状外缘尺寸不得小于
        // 2×outline-width（assert「Negative values on outline-offset should not cause the
        // height and the width of outside of the shape drawn by the outline to become
        // smaller than twice the computed value of the outline-width property」）——
        // 负 offset 过深（total_offset −100）时 ow_raw = w + 2×total 为负，旧实现画负宽
        // 矩形 = outline 消失。钳到下限后 ox/oy 按**盒中心**重锚（替代 abs−total 差分式，
        // offset=0/负浅时与原式逐字节一致：ox = abs_x + w/2 − (w+2t)/2 = abs_x − t）。
        let ow = (w + 2.0 * total_offset).max(2.0 * outline_width);
        let oh = (h + 2.0 * total_offset).max(2.0 * outline_width);
        let ox = abs_x + w / 2.0 - ow / 2.0;
        let oy = abs_y + h / 2.0 - oh / 2.0;

        match style.outline_style {
            OutlineStyleValue::None => {}
            // R2379：CSS UI 4 auto 为 UA-defined 描边，ZW 按 solid 渲染（典型焦点环）。
            OutlineStyleValue::Solid | OutlineStyleValue::Auto => {
                // 实线：4 个填充矩形
                self.primitives.add_fill(Rect::new(ox, oy, ow, outline_width), color);
                self.primitives
                    .add_fill(Rect::new(ox, oy + oh - outline_width, ow, outline_width), color);
                self.primitives.add_fill(
                    Rect::new(ox, oy + outline_width, outline_width, oh - 2.0 * outline_width),
                    color,
                );
                self.primitives.add_fill(
                    Rect::new(
                        ox + ow - outline_width,
                        oy + outline_width,
                        outline_width,
                        oh - 2.0 * outline_width,
                    ),
                    color,
                );
            }
            OutlineStyleValue::Dotted => {
                let mid_y_top = oy + outline_width / 2.0;
                let mid_y_bottom = oy + oh - outline_width / 2.0;
                let mid_x_left = ox + outline_width / 2.0;
                let mid_x_right = ox + ow - outline_width / 2.0;
                for (x1, y1, x2, y2) in [
                    (ox, mid_y_top, ox + ow, mid_y_top),
                    (ox, mid_y_bottom, ox + ow, mid_y_bottom),
                    (mid_x_left, mid_y_top, mid_x_left, mid_y_bottom),
                    (mid_x_right, mid_y_top, mid_x_right, mid_y_bottom),
                ] {
                    self.primitives.add_stroke(StrokePrimitive {
                        x1,
                        y1,
                        x2,
                        y2,
                        width: outline_width,
                        color,
                        style: LineStyle::Dotted,
                        cap: LineCap::Round,
                    });
                }
            }
            OutlineStyleValue::Dashed => {
                let mid_y_top = oy + outline_width / 2.0;
                let mid_y_bottom = oy + oh - outline_width / 2.0;
                let mid_x_left = ox + outline_width / 2.0;
                let mid_x_right = ox + ow - outline_width / 2.0;
                for (x1, y1, x2, y2) in [
                    (ox, mid_y_top, ox + ow, mid_y_top),
                    (ox, mid_y_bottom, ox + ow, mid_y_bottom),
                    (mid_x_left, mid_y_top, mid_x_left, mid_y_bottom),
                    (mid_x_right, mid_y_top, mid_x_right, mid_y_bottom),
                ] {
                    self.primitives.add_stroke(StrokePrimitive {
                        x1,
                        y1,
                        x2,
                        y2,
                        width: outline_width,
                        color,
                        style: LineStyle::Dashed,
                        cap: LineCap::Square,
                    });
                }
            }
            OutlineStyleValue::Double => {
                let gap = (outline_width / 3.0).max(1.0);
                let line_w = ((outline_width - gap) / 2.0).max(1.0);
                // 外线
                self.primitives.add_fill(Rect::new(ox, oy, ow, line_w), color);
                self.primitives
                    .add_fill(Rect::new(ox, oy + oh - line_w, ow, line_w), color);
                self.primitives
                    .add_fill(Rect::new(ox, oy + line_w, line_w, oh - 2.0 * line_w), color);
                self.primitives.add_fill(
                    Rect::new(ox + ow - line_w, oy + line_w, line_w, oh - 2.0 * line_w),
                    color,
                );
                // 内线
                let ix = ox + line_w + gap;
                let iy = oy + line_w + gap;
                let iw = ow - 2.0 * (line_w + gap);
                let ih = oh - 2.0 * (line_w + gap);
                self.primitives.add_fill(Rect::new(ix, iy, iw, line_w), color);
                self.primitives
                    .add_fill(Rect::new(ix, iy + ih - line_w, iw, line_w), color);
                self.primitives
                    .add_fill(Rect::new(ix, iy + line_w, line_w, ih - 2.0 * line_w), color);
                self.primitives.add_fill(
                    Rect::new(ix + iw - line_w, iy + line_w, line_w, ih - 2.0 * line_w),
                    color,
                );
            }
            OutlineStyleValue::Groove | OutlineStyleValue::Ridge => {
                let (first, second) = if matches!(style.outline_style, OutlineStyleValue::Groove) {
                    groove_ridge_colors(&color)
                } else {
                    let (l, d) = groove_ridge_colors(&color);
                    (d, l)
                };
                let half = outline_width / 2.0;
                self.primitives.add_fill(Rect::new(ox, oy, ow, half), first);
                self.primitives.add_fill(Rect::new(ox, oy + half, ow, half), second);
                self.primitives
                    .add_fill(Rect::new(ox, oy + oh - outline_width, ow, half), first);
                self.primitives
                    .add_fill(Rect::new(ox, oy + oh - half, ow, half), second);
                self.primitives
                    .add_fill(Rect::new(ox, oy + outline_width, half, oh - 2.0 * outline_width), first);
                self.primitives.add_fill(
                    Rect::new(ox + half, oy + outline_width, half, oh - 2.0 * outline_width),
                    second,
                );
                self.primitives.add_fill(
                    Rect::new(
                        ox + ow - outline_width,
                        oy + outline_width,
                        half,
                        oh - 2.0 * outline_width,
                    ),
                    first,
                );
                self.primitives.add_fill(
                    Rect::new(ox + ow - half, oy + outline_width, half, oh - 2.0 * outline_width),
                    second,
                );
            }
            OutlineStyleValue::Inset | OutlineStyleValue::Outset => {
                let (first, second) = if matches!(style.outline_style, OutlineStyleValue::Inset) {
                    (darken(&color, 0.3), lighten(&color, 0.3))
                } else {
                    (lighten(&color, 0.3), darken(&color, 0.3))
                };
                let half = outline_width / 2.0;
                self.primitives.add_fill(Rect::new(ox, oy, ow, half), first);
                self.primitives.add_fill(Rect::new(ox, oy + half, ow, half), second);
                self.primitives
                    .add_fill(Rect::new(ox, oy + oh - outline_width, ow, half), first);
                self.primitives
                    .add_fill(Rect::new(ox, oy + oh - half, ow, half), second);
                self.primitives
                    .add_fill(Rect::new(ox, oy + outline_width, half, oh - 2.0 * outline_width), first);
                self.primitives.add_fill(
                    Rect::new(ox + half, oy + outline_width, half, oh - 2.0 * outline_width),
                    second,
                );
                self.primitives.add_fill(
                    Rect::new(
                        ox + ow - outline_width,
                        oy + outline_width,
                        half,
                        oh - 2.0 * outline_width,
                    ),
                    first,
                );
                self.primitives.add_fill(
                    Rect::new(ox + ow - half, oy + outline_width, half, oh - 2.0 * outline_width),
                    second,
                );
            }
        }
    }

    /// R5335（CSS2 §9.2.1.1 block-in-inline）：被块级后代切片的 inline 盒边框按
    /// slice 描绘。box-level 全边绘制把 left/right 竖带画成贯穿全部块的连续竖带
    /// （driving: block-in-inline-insert-009/010 outer blue 全边带）；规范语义是
    /// 边框随切片分段：left 竖边仅首 slice、right 竖边仅末 slice、上下横带每个
    /// slice 独立（slice 间被无框块子隔开，带不相接）。横带外侧锚定各 slice 盒
    /// 区间（嵌套 inline 域横带与内层片段盒横带同位，内层后画覆盖，视觉只剩
    /// 内层带——nosplit-ref slice1 实证）。绘制为实心矩形条带；复杂 border-style
    /// 按实心近似（FIXME：dashed/double 等逐 slice 复刻），background 的切片化
    /// 未实现（本域案例均无背景，挂账）。
    pub(super) fn paint_block_in_inline_slice_borders(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
        slices: &[(f32, f32, f32)],
    ) {
        let bl = box_node.border_left;
        let br = box_node.border_right;
        let bt = box_node.border_top;
        let bb = box_node.border_bottom;
        let current = &style.color;
        let left_color = color_value_to_render(&resolve_border_current_color(style.border_left_color.clone(), current));
        let right_color =
            color_value_to_render(&resolve_border_current_color(style.border_right_color.clone(), current));
        let top_color = color_value_to_render(&resolve_border_current_color(style.border_top_color.clone(), current));
        let bottom_color = color_value_to_render(&resolve_border_current_color(
            style.border_bottom_color.clone(),
            current,
        ));
        let n = slices.len();
        for (i, &(top, bottom, right)) in slices.iter().enumerate() {
            let is_first = i == 0;
            let is_last = i == n - 1;
            let band_right = right + br;
            let y0 = abs_y + top;
            let y1 = abs_y + bottom;
            let span = (y1 - y0).max(0.0);
            // 上下横带：每 slice 独立（外侧锚定 slice 盒区间）。
            if bt > 0.0 {
                self.primitives
                    .add_fill(Rect::new(abs_x, y0 - bt, band_right, bt), top_color);
            }
            if bb > 0.0 {
                self.primitives
                    .add_fill(Rect::new(abs_x, y1, band_right, bb), bottom_color);
            }
            // 左竖带：仅首 slice（含角部全高）。
            if bl > 0.0 && is_first {
                self.primitives
                    .add_fill(Rect::new(abs_x, y0 - bt, bl, span + bt + bb), left_color);
            }
            // 右竖带：仅末 slice。
            if br > 0.0 && is_last {
                self.primitives.add_fill(
                    Rect::new(abs_x + band_right - br, y0 - bt, br, span + bt + bb),
                    right_color,
                );
            }
        }
    }
}

fn resolve_outline_length(length: &LengthValue, style: &ComputedStyle) -> f32 {
    match length {
        LengthValue::Px(v) => *v as f32,
        other => {
            let font_size = zero_style_system::computed::resolve_length(&style.font_size, 16.0, None, None);
            zero_style_system::computed::resolve_length(other, font_size, None, None) as f32
        }
    }
}

/// Groove/Ridge 颜色对生成。
fn groove_ridge_colors(color: &Color) -> (Color, Color) {
    (lighten(color, 0.3), darken(color, 0.3))
}

/// 使颜色变亮。
fn lighten(color: &Color, amount: f32) -> Color {
    Color::rgba(
        (color.r as f32 + (255.0 - color.r as f32) * amount).min(255.0) as u8,
        (color.g as f32 + (255.0 - color.g as f32) * amount).min(255.0) as u8,
        (color.b as f32 + (255.0 - color.b as f32) * amount).min(255.0) as u8,
        color.a,
    )
}

/// 使颜色变暗。
fn darken(color: &Color, amount: f32) -> Color {
    Color::rgba(
        (color.r as f32 * (1.0 - amount)).min(255.0) as u8,
        (color.g as f32 * (1.0 - amount)).min(255.0) as u8,
        (color.b as f32 * (1.0 - amount)).min(255.0) as u8,
        color.a,
    )
}

/// 解析 border-collapse 边框颜色覆盖。
///
/// 当 LayoutBox 有 collapsed_border_color_overrides 时，使用覆盖颜色（RGBA u32），
/// 否则回退到 ComputedStyle 中的原始颜色。
fn collapsed_border_color(override_color: &Option<u32>, original: &ColorValue) -> ColorValue {
    match override_color {
        Some(rgba) => {
            let r = ((rgba >> 24) & 0xFF) as u8;
            let g = ((rgba >> 16) & 0xFF) as u8;
            let b = ((rgba >> 8) & 0xFF) as u8;
            let a = (rgba & 0xFF) as u8;
            ColorValue::Rgba(r, g, b, a)
        }
        None => original.clone(),
    }
}

/// 把边框颜色的 `currentColor` 关键字解析为元素自身计算 `color`（CSS-Color §resolving）。
/// 非 currentColor 颜色原样返回。border-color 初始值为 currentColor，故无显式颜色的
/// `border: solid` 会使用元素文本色（与 Chromium 一致）。
fn resolve_border_current_color(color: ColorValue, current: &ColorValue) -> ColorValue {
    if color == ColorValue::CurrentColor {
        current.clone()
    } else if matches!(
        color,
        ColorValue::Mix(_) | ColorValue::RelativeColor(_) | ColorValue::ContrastColor(_)
    ) {
        // R3907：color-mix/RCS 内嵌 currentColor 须按元素 color 解析——下游
        // paint_border_edge 用 color_value_to_render（无元素上下文，currentColor 回落
        // 黑色），keyword 替换覆盖不到嵌套形态（driving:
        // color-mix-currentcolor-border/outline-repaint，黑色方块应红/绿）。
        // 全量解析后以 Rgba 回包（paint_border_edge 的 color_value_to_render 恒等）。
        let c = resolve_color_current(&color, current);
        ColorValue::Rgba(c.r, c.g, c.b, c.a)
    } else {
        color
    }
}

/// R5335：收集 inline 盒子树中匿名块片段盒的 slice 区间（相对盒 border-box
/// 原点：(top, bottom, right)）。CSS2 §9.2.1.1 切片 = inline 内容被块级后代
/// 切开的 inline 段（R109 匿名块片段盒承载）；无框块级子（div 等）只作切分
/// 边界不计 slice（其区域外层 inline 无边框——nosplit-ref div 区无上下带实证）。
/// 中间 inline 层（R109 split 宿主壳）继续深入；原子盒（replaced）不深入。
/// 返回 None = 子树无片段盒（调用方不进切片绘制路径）。
pub(crate) fn collect_block_in_inline_slices(box_node: &LayoutBox) -> Option<Vec<(f32, f32, f32)>> {
    let mut out: Vec<(f32, f32, f32)> = Vec::new();
    collect_slice_regions(box_node, 0.0, 0.0, 0, &mut out);
    if out.is_empty() {
        None
    } else {
        out.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        Some(out)
    }
}

fn collect_slice_regions(box_node: &LayoutBox, base_x: f32, base_y: f32, depth: usize, out: &mut Vec<(f32, f32, f32)>) {
    if depth > 32 {
        return;
    }
    let cx = base_x + box_node.padding_left + box_node.border_left;
    let cy = base_y + box_node.padding_top + box_node.border_top;
    for c in &box_node.children {
        let x = cx + c.x;
        let y = cy + c.y;
        if c.is_absolute || c.is_fixed {
            continue;
        }
        if c.fragment_node_ids.is_some() {
            out.push((y, y + c.height, x + c.width));
            continue;
        }
        if c.is_block_level {
            continue;
        }
        if !c.is_replaced {
            collect_slice_regions(c, x, y, depth + 1, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_border_current_color_substitutes_keyword() {
        // currentColor 解析为元素计算 color
        let green = ColorValue::Named("green".to_string());
        assert_eq!(resolve_border_current_color(ColorValue::CurrentColor, &green), green);
        // 非 currentColor 颜色原样返回，不受影响
        let red = ColorValue::Rgba(255, 0, 0, 255);
        assert_eq!(resolve_border_current_color(red.clone(), &green), red);
    }

    #[test]
    /// R3907：color-mix 内嵌 currentColor 按元素 color 全量解析（下游
    /// paint_border_edge 的 color_value_to_render 无元素上下文会回落黑色）。
    fn test_resolve_border_current_color_resolves_nested_mix() {
        let red = ColorValue::Rgba(255, 0, 0, 255);
        let mix = zero_css_parser::values::parse_color("color-mix(in hsl, transparent 0%, currentColor 100%)")
            .expect("in hsl 应可解析");
        let resolved = resolve_border_current_color(mix, &red);
        assert_eq!(
            resolved,
            ColorValue::Rgba(255, 0, 0, 255),
            "嵌套 currentColor 须按元素红解析为不透明红，非黑"
        );
    }
}
