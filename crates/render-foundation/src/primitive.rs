//! 渲染图元 — 填充矩形、圆角矩形、路径填充、路径描边、裁剪区域、渐变、阴影、图片、Glyph 图元等

use crate::color::Color;
use crate::geometry::Rect;
use crate::image_cache::ImageKey;

/// 填充图元 — 纯色矩形
#[derive(Debug, Clone)]
pub struct FillPrimitive {
    /// 矩形区域
    pub rect: Rect,
    /// 填充颜色
    pub color: Color,
}

/// 圆角矩形图元 — 支持 border-radius 的填充矩形
#[derive(Debug, Clone)]
pub struct RoundedRectPrimitive {
    /// 矩形区域
    pub rect: Rect,
    /// 填充颜色
    pub color: Color,
    /// 左上角圆角半径
    pub top_left_radius: f32,
    /// 右上角圆角半径
    pub top_right_radius: f32,
    /// 右下角圆角半径
    pub bottom_right_radius: f32,
    /// 左下角圆角半径
    pub bottom_left_radius: f32,
}

impl RoundedRectPrimitive {
    /// 创建四个圆角相同的圆角矩形
    pub fn uniform(rect: Rect, color: Color, radius: f32) -> Self {
        Self {
            rect,
            color,
            top_left_radius: radius,
            top_right_radius: radius,
            bottom_right_radius: radius,
            bottom_left_radius: radius,
        }
    }
}

/// 路径填充图元 — 使用路径命令填充任意形状。
#[derive(Debug, Clone)]
pub struct PathFillPrimitive {
    /// 路径命令列表（扁平化的线段序列）。
    /// 每对 f32 表示一个顶点 (x, y)，构成闭合多边形。
    pub vertices: Vec<f32>,
    /// 填充颜色。
    pub color: Color,
}

/// 线段端点样式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineCap {
    /// 平头
    Butt,
    /// 圆头
    Round,
    /// 方头
    Square,
}

/// 描边线型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineStyle {
    /// 实线
    Solid,
    /// 虚线（线段和间隔交替）
    Dashed,
    /// 点线
    Dotted,
}

/// 路径描边图元 — 使用路径命令描边任意形状。
#[derive(Debug, Clone)]
pub struct PathStrokePrimitive {
    /// 路径命令列表（扁平化的线段序列）。
    /// 每对 f32 表示一个顶点 (x, y)，构成折线/多边形。
    pub vertices: Vec<f32>,
    /// 描边颜色。
    pub color: Color,
    /// 线宽。
    pub line_width: f32,
    /// 是否闭合路径。
    pub closed: bool,
}

/// 描边线段图元 — 用于 border-style dashed/dotted 等单条线段
#[derive(Debug, Clone)]
pub struct StrokePrimitive {
    /// 线段起点
    pub x1: f32,
    /// 线段起点
    pub y1: f32,
    /// 线段终点
    pub x2: f32,
    /// 线段终点
    pub y2: f32,
    /// 线宽
    pub width: f32,
    /// 线条颜色
    pub color: Color,
    /// 线型
    pub style: LineStyle,
    /// 端点样式
    pub cap: LineCap,
}

/// 裁剪图元 — 限制后续绘制到指定矩形区域内
#[derive(Debug, Clone)]
pub struct ClipPrimitive {
    /// 裁剪矩形区域
    pub rect: Rect,
}

/// 渐变停止点
#[derive(Debug, Clone)]
pub struct GradientStop {
    /// 偏移量 [0.0, 1.0]
    pub offset: f32,
    /// 颜色
    pub color: Color,
}

/// 渐变类型
#[derive(Debug, Clone)]
pub enum GradientKind {
    /// 线性渐变：从起点到终点
    Linear {
        /// 起点 X
        x0: f32,
        /// 起点 Y
        y0: f32,
        /// 终点 X
        x1: f32,
        /// 终点 Y
        y1: f32,
    },
    /// 径向渐变：从内圆到外圆
    Radial {
        /// 内圆圆心 X
        cx: f32,
        /// 内圆圆心 Y
        cy: f32,
        /// 内圆半径
        inner_radius: f32,
        /// 外圆半径
        outer_radius: f32,
    },
}

/// 渐变图元 — 线性/径向渐变填充矩形
#[derive(Debug, Clone)]
pub struct GradientPrimitive {
    /// 渐变覆盖的矩形区域
    pub rect: Rect,
    /// 渐变类型
    pub kind: GradientKind,
    /// 颜色停止点列表
    pub stops: Vec<GradientStop>,
}

/// 阴影图元 — box-shadow 效果
#[derive(Debug, Clone)]
pub struct ShadowPrimitive {
    /// 阴影对应的矩形区域
    pub rect: Rect,
    /// 阴影颜色
    pub color: Color,
    /// 水平偏移
    pub offset_x: f32,
    /// 垂直偏移
    pub offset_y: f32,
    /// 模糊半径
    pub blur_radius: f32,
    /// 扩展半径
    pub spread_radius: f32,
}

/// 图片图元 — 在指定矩形区域内绘制图片
#[derive(Debug, Clone)]
pub struct ImagePrimitive {
    /// 目标绘制区域
    pub rect: Rect,
    /// 图片缓存键
    pub image_key: ImageKey,
}

/// Glyph 图元 — 字符渲染
#[derive(Debug, Clone)]
pub struct GlyphPrimitive {
    /// 在表面上的位置（左上角）
    pub x: f32,
    /// 在表面上的位置（基线）
    pub y: f32,
    /// 字体大小（像素）
    pub font_size: f32,
    /// 前景色
    pub color: Color,
    /// Glyph 索引
    pub glyph_id: u32,
    /// 字体 ID
    pub font_id: FontId,
    /// 预缓存的位图宽度（如果已缓存）
    pub bitmap_width: Option<u16>,
    /// 预缓存的位图高度
    pub bitmap_height: Option<u16>,
}

/// 字体 ID 标识
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FontId(pub u32);

/// 渲染统计 — 追踪图元数量、估算 draw call 数量和批处理效率。
#[derive(Debug, Clone, Default)]
pub struct RenderStats {
    /// 填充矩形数量
    pub fill_count: usize,
    /// 圆角矩形数量
    pub rounded_rect_count: usize,
    /// 路径填充数量
    pub path_fill_count: usize,
    /// 路径描边数量
    pub path_stroke_count: usize,
    /// 描边线段数量
    pub stroke_count: usize,
    /// 渐变数量
    pub gradient_count: usize,
    /// 阴影数量
    pub shadow_count: usize,
    /// 图片数量
    pub image_count: usize,
    /// Glyph 数量
    pub glyph_count: usize,
    /// 裁剪区域数量
    pub clip_count: usize,
    /// 估算 draw call 数量（按材质/状态分组后的最少调用次数）
    pub estimated_draw_calls: usize,
    /// 因 viewport culling 被剔除的图元数量
    pub culled_count: usize,
}

impl RenderStats {
    /// 图元总数
    pub fn total_primitives(&self) -> usize {
        self.fill_count
            + self.rounded_rect_count
            + self.path_fill_count
            + self.path_stroke_count
            + self.stroke_count
            + self.gradient_count
            + self.shadow_count
            + self.image_count
            + self.glyph_count
            + self.clip_count
    }
}

/// 渲染图元列表 — 由渲染管线生成，供 Backend 消费
#[derive(Debug, Clone, Default)]
pub struct RenderPrimitives {
    /// 裁剪区域列表（绘制其他图元前应应用裁剪）
    pub clips: Vec<ClipPrimitive>,
    /// 填充矩形列表
    pub fills: Vec<FillPrimitive>,
    /// 圆角矩形列表
    pub rounded_rects: Vec<RoundedRectPrimitive>,
    /// 路径填充列表
    pub path_fills: Vec<PathFillPrimitive>,
    /// 路径描边列表
    pub path_strokes: Vec<PathStrokePrimitive>,
    /// 描边线段列表
    pub strokes: Vec<StrokePrimitive>,
    /// 渐变列表
    pub gradients: Vec<GradientPrimitive>,
    /// 阴影列表
    pub shadows: Vec<ShadowPrimitive>,
    /// 图片列表
    pub images: Vec<ImagePrimitive>,
    /// Glyph 列表
    pub glyphs: Vec<GlyphPrimitive>,
}

impl RenderPrimitives {
    /// 创建空的图元列表
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加一个填充矩形
    pub fn add_fill(&mut self, rect: Rect, color: Color) {
        self.fills.push(FillPrimitive { rect, color });
    }

    /// 添加一个圆角矩形
    pub fn add_rounded_rect(&mut self, rounded: RoundedRectPrimitive) {
        self.rounded_rects.push(rounded);
    }

    /// 添加一个路径填充图元。
    pub fn add_path_fill(&mut self, vertices: Vec<f32>, color: Color) {
        self.path_fills.push(PathFillPrimitive { vertices, color });
    }

    /// 添加一个路径描边图元。
    pub fn add_path_stroke(&mut self, vertices: Vec<f32>, color: Color, line_width: f32, closed: bool) {
        self.path_strokes.push(PathStrokePrimitive {
            vertices,
            color,
            line_width,
            closed,
        });
    }

    /// 添加一个描边线段
    pub fn add_stroke(&mut self, stroke: StrokePrimitive) {
        self.strokes.push(stroke);
    }

    /// 添加一个裁剪区域
    pub fn add_clip(&mut self, rect: Rect) {
        self.clips.push(ClipPrimitive { rect });
    }

    /// 添加一个渐变
    pub fn add_gradient(&mut self, gradient: GradientPrimitive) {
        self.gradients.push(gradient);
    }

    /// 添加一个阴影
    pub fn add_shadow(&mut self, shadow: ShadowPrimitive) {
        self.shadows.push(shadow);
    }

    /// 添加一个图片图元
    pub fn add_image(&mut self, image: ImagePrimitive) {
        self.images.push(image);
    }

    /// 添加一个 Glyph
    pub fn add_glyph(&mut self, glyph: GlyphPrimitive) {
        self.glyphs.push(glyph);
    }

    /// 图元总数
    pub fn len(&self) -> usize {
        self.clips.len()
            + self.fills.len()
            + self.rounded_rects.len()
            + self.path_fills.len()
            + self.path_strokes.len()
            + self.strokes.len()
            + self.gradients.len()
            + self.shadows.len()
            + self.images.len()
            + self.glyphs.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.clips.is_empty()
            && self.fills.is_empty()
            && self.rounded_rects.is_empty()
            && self.path_fills.is_empty()
            && self.path_strokes.is_empty()
            && self.strokes.is_empty()
            && self.gradients.is_empty()
            && self.shadows.is_empty()
            && self.images.is_empty()
            && self.glyphs.is_empty()
    }

    /// 获取所有图元的包围盒
    pub fn bounding_box(&self) -> Option<Rect> {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        let mut expand = |left: f32, top: f32, right: f32, bottom: f32| {
            min_x = min_x.min(left);
            min_y = min_y.min(top);
            max_x = max_x.max(right);
            max_y = max_y.max(bottom);
        };

        for fill in &self.fills {
            expand(fill.rect.left(), fill.rect.top(), fill.rect.right(), fill.rect.bottom());
        }

        for rr in &self.rounded_rects {
            expand(rr.rect.left(), rr.rect.top(), rr.rect.right(), rr.rect.bottom());
        }

        for path_fill in &self.path_fills {
            for chunk in path_fill.vertices.chunks_exact(2) {
                expand(chunk[0], chunk[1], chunk[0], chunk[1]);
            }
        }

        for path_stroke in &self.path_strokes {
            for chunk in path_stroke.vertices.chunks_exact(2) {
                expand(chunk[0], chunk[1], chunk[0], chunk[1]);
            }
        }

        for stroke in &self.strokes {
            let half_w = stroke.width / 2.0;
            expand(
                stroke.x1.min(stroke.x2) - half_w,
                stroke.y1.min(stroke.y2) - half_w,
                stroke.x1.max(stroke.x2) + half_w,
                stroke.y1.max(stroke.y2) + half_w,
            );
        }

        for grad in &self.gradients {
            expand(grad.rect.left(), grad.rect.top(), grad.rect.right(), grad.rect.bottom());
        }

        for shadow in &self.shadows {
            let left = shadow.rect.left() + shadow.offset_x - shadow.spread_radius - shadow.blur_radius;
            let top = shadow.rect.top() + shadow.offset_y - shadow.spread_radius - shadow.blur_radius;
            let right = shadow.rect.right() + shadow.offset_x + shadow.spread_radius + shadow.blur_radius;
            let bottom = shadow.rect.bottom() + shadow.offset_y + shadow.spread_radius + shadow.blur_radius;
            expand(left, top, right, bottom);
        }

        for img in &self.images {
            expand(img.rect.left(), img.rect.top(), img.rect.right(), img.rect.bottom());
        }

        for glyph in &self.glyphs {
            expand(glyph.x, glyph.y, glyph.x + glyph.font_size, glyph.y + glyph.font_size);
        }

        if min_x < max_x && min_y < max_y {
            Some(Rect::new(min_x, min_y, max_x - min_x, max_y - min_y))
        } else {
            None
        }
    }

    /// 生成稳定的文本快照，用于测试对比。
    ///
    /// 输出每行一个图元，坐标精度固定为 2 位小数。
    /// 格式: `<类型>: <关键几何属性> <颜色>`
    pub fn snapshot(&self) -> String {
        let mut buf = String::new();
        for (i, clip) in self.clips.iter().enumerate() {
            buf.push_str(&format!(
                "clip[{}]: ({:.2},{:.2} {:.2}x{:.2})\n",
                i, clip.rect.origin.x, clip.rect.origin.y, clip.rect.size.width, clip.rect.size.height,
            ));
        }
        for (i, fill) in self.fills.iter().enumerate() {
            buf.push_str(&format!(
                "fill[{}]: ({:.2},{:.2} {:.2}x{:.2}) #{:02x}{:02x}{:02x}{:02x}\n",
                i,
                fill.rect.origin.x,
                fill.rect.origin.y,
                fill.rect.size.width,
                fill.rect.size.height,
                fill.color.r,
                fill.color.g,
                fill.color.b,
                fill.color.a,
            ));
        }
        for (i, rr) in self.rounded_rects.iter().enumerate() {
            buf.push_str(&format!(
                "rounded[{}]: ({:.2},{:.2} {:.2}x{:.2}) r=({:.2},{:.2},{:.2},{:.2}) #{:02x}{:02x}{:02x}{:02x}\n",
                i,
                rr.rect.origin.x,
                rr.rect.origin.y,
                rr.rect.size.width,
                rr.rect.size.height,
                rr.top_left_radius,
                rr.top_right_radius,
                rr.bottom_right_radius,
                rr.bottom_left_radius,
                rr.color.r,
                rr.color.g,
                rr.color.b,
                rr.color.a,
            ));
        }
        for (i, stroke) in self.strokes.iter().enumerate() {
            buf.push_str(&format!(
                "stroke[{}]: ({:.2},{:.2})->({:.2},{:.2}) w={:.2} #{:02x}{:02x}{:02x}{:02x}\n",
                i,
                stroke.x1,
                stroke.y1,
                stroke.x2,
                stroke.y2,
                stroke.width,
                stroke.color.r,
                stroke.color.g,
                stroke.color.b,
                stroke.color.a,
            ));
        }
        for (i, grad) in self.gradients.iter().enumerate() {
            buf.push_str(&format!(
                "gradient[{}]: ({:.2},{:.2} {:.2}x{:.2}) stops={}\n",
                i,
                grad.rect.origin.x,
                grad.rect.origin.y,
                grad.rect.size.width,
                grad.rect.size.height,
                grad.stops.len(),
            ));
        }
        for (i, shadow) in self.shadows.iter().enumerate() {
            buf.push_str(&format!(
                "shadow[{}]: ({:.2},{:.2} {:.2}x{:.2}) offset=({:.2},{:.2}) blur={:.2} spread={:.2}\n",
                i,
                shadow.rect.origin.x,
                shadow.rect.origin.y,
                shadow.rect.size.width,
                shadow.rect.size.height,
                shadow.offset_x,
                shadow.offset_y,
                shadow.blur_radius,
                shadow.spread_radius,
            ));
        }
        for (i, img) in self.images.iter().enumerate() {
            buf.push_str(&format!(
                "image[{}]: ({:.2},{:.2} {:.2}x{:.2}) key={}\n",
                i, img.rect.origin.x, img.rect.origin.y, img.rect.size.width, img.rect.size.height, img.image_key.0,
            ));
        }
        for (i, glyph) in self.glyphs.iter().enumerate() {
            buf.push_str(&format!(
                "glyph[{}]: ({:.2},{:.2}) size={:.2}\n",
                i, glyph.x, glyph.y, glyph.font_size,
            ));
        }
        buf
    }

    /// 计算渲染统计信息，包括估算的 draw call 数量。
    ///
    /// draw call 估算规则：
    /// - 每种不同颜色/材质的 fill 算一次 draw call
    /// - 每个 rounded_rect 算一次 draw call（通常圆角不同）
    /// - 每种颜色的 path_fill 算一次 draw call
    /// - 每个 gradient 算一次 draw call
    /// - 每个 image 算一次 draw call（纹理不同）
    /// - 每种字体+颜色组合的 glyph 算一次 draw call
    /// - 每个 shadow 算一次 draw call
    pub fn stats(&self) -> RenderStats {
        use std::collections::HashSet;

        // 计算不同颜色 fill 的 draw call 数量
        let fill_colors: HashSet<[u8; 4]> = self
            .fills
            .iter()
            .map(|f| [f.color.r, f.color.g, f.color.b, f.color.a])
            .collect();

        // 计算不同颜色 path_fill 的 draw call 数量
        let path_fill_colors: HashSet<[u8; 4]> = self
            .path_fills
            .iter()
            .map(|pf| [pf.color.r, pf.color.g, pf.color.b, pf.color.a])
            .collect();

        // 计算不同字体+颜色 glyph 的 draw call 数量
        let glyph_keys: HashSet<(u32, [u8; 4])> = self
            .glyphs
            .iter()
            .map(|g| (g.font_id.0, [g.color.r, g.color.g, g.color.b, g.color.a]))
            .collect();

        // 计算不同颜色 path_stroke 的 draw call 数量
        let stroke_colors: HashSet<[u8; 4]> = self
            .path_strokes
            .iter()
            .map(|ps| [ps.color.r, ps.color.g, ps.color.b, ps.color.a])
            .collect();

        let estimated_draw_calls = fill_colors.len()
            + self.rounded_rects.len()
            + path_fill_colors.len()
            + stroke_colors.len()
            + self.strokes.len()
            + self.gradients.len()
            + self.shadows.len()
            + self.images.len()
            + glyph_keys.len()
            + self.clips.len().min(1); // clips 合并为一个

        RenderStats {
            fill_count: self.fills.len(),
            rounded_rect_count: self.rounded_rects.len(),
            path_fill_count: self.path_fills.len(),
            path_stroke_count: self.path_strokes.len(),
            stroke_count: self.strokes.len(),
            gradient_count: self.gradients.len(),
            shadow_count: self.shadows.len(),
            image_count: self.images.len(),
            glyph_count: self.glyphs.len(),
            clip_count: self.clips.len(),
            estimated_draw_calls,
            culled_count: 0,
        }
    }

    /// 对填充图元进行批处理 — 合并相同颜色的相邻矩形。
    ///
    /// 优化策略：
    /// - 相同颜色的填充按 y 坐标排序
    /// - 如果两个同色矩形在 y 方向相邻（一个的 bottom == 另一个的 top，且 x 范围重叠），
    ///   合并为一个大矩形
    ///
    /// 返回优化后的新 `RenderPrimitives`，原始数据不变。
    pub fn batch_fills(&self) -> RenderPrimitives {
        if self.fills.len() <= 1 {
            return self.clone();
        }

        // 按颜色分组
        let mut color_groups: std::collections::HashMap<[u8; 4], Vec<&FillPrimitive>> =
            std::collections::HashMap::new();
        for fill in &self.fills {
            let key = [fill.color.r, fill.color.g, fill.color.b, fill.color.a];
            color_groups.entry(key).or_default().push(fill);
        }

        let mut batched_fills = Vec::new();

        for (_color_key, fills) in color_groups {
            if fills.is_empty() {
                continue;
            }

            let color = fills[0].color;

            // 尝试在垂直方向合并同色矩形
            // 简单策略：合并完全同列（x 和 width 相同）且垂直相邻的矩形
            let merged: Vec<Rect> = fills.iter().map(|f| f.rect).collect();

            // 按列（x, width）分组，在每列内按 y 排序
            let mut columns: std::collections::HashMap<(u32, u32), Vec<Rect>> = std::collections::HashMap::new();
            for rect in &merged {
                // 使用固定精度来分组（避免浮点误差）
                let x_key = (rect.origin.x.to_bits(), rect.size.width.to_bits());
                columns.entry(x_key).or_default().push(*rect);
            }

            for (_, mut rects) in columns {
                rects.sort_by(|a, b| a.origin.y.partial_cmp(&b.origin.y).unwrap_or(std::cmp::Ordering::Equal));

                let mut result = Vec::new();
                let mut current = rects[0];

                for rect in rects.iter().skip(1) {
                    let current_bottom = current.origin.y + current.size.height;
                    // 如果垂直相邻（间距 < 1px），合并
                    if (rect.origin.y - current_bottom).abs() < 1.0
                        && (rect.origin.x - current.origin.x).abs() < 1.0
                        && (rect.size.width - current.size.width).abs() < 1.0
                    {
                        // 合并：扩展当前矩形的高度
                        let new_bottom = rect.origin.y + rect.size.height;
                        current.size.height = new_bottom - current.origin.y;
                    } else {
                        result.push(current);
                        current = *rect;
                    }
                }
                result.push(current);

                for rect in result {
                    batched_fills.push(FillPrimitive { rect, color });
                }
            }
        }

        let mut result = self.clone();
        result.fills = batched_fills;
        result
    }

    /// 视口剔除 — 移除完全在视口外的图元。
    ///
    /// 只剔除 fills、rounded_rects、strokes、shadows、images。
    /// clips 和 glyphs 保留（clips 是全局状态，glyphs 可能被后续使用）。
    ///
    /// 返回剔除后的新 `RenderPrimitives` 和统计信息。
    pub fn cull_invisible(&self, viewport: Rect) -> (RenderPrimitives, RenderStats) {
        let original_len = self.len();

        let fills: Vec<FillPrimitive> = self
            .fills
            .iter()
            .filter(|f| viewport.intersects(&f.rect))
            .cloned()
            .collect();

        let rounded_rects: Vec<RoundedRectPrimitive> = self
            .rounded_rects
            .iter()
            .filter(|rr| viewport.intersects(&rr.rect))
            .cloned()
            .collect();

        let strokes: Vec<StrokePrimitive> = self
            .strokes
            .iter()
            .filter(|s| {
                let half_w = s.width / 2.0;
                let stroke_rect = Rect::new(
                    s.x1.min(s.x2) - half_w,
                    s.y1.min(s.y2) - half_w,
                    (s.x1.max(s.x2) - s.x1.min(s.x2)) + s.width,
                    (s.y1.max(s.y2) - s.y1.min(s.y2)) + s.width,
                );
                viewport.intersects(&stroke_rect)
            })
            .cloned()
            .collect();

        let shadows: Vec<ShadowPrimitive> = self
            .shadows
            .iter()
            .filter(|s| {
                let shadow_rect = Rect::new(
                    s.rect.origin.x + s.offset_x - s.spread_radius - s.blur_radius,
                    s.rect.origin.y + s.offset_y - s.spread_radius - s.blur_radius,
                    s.rect.size.width + 2.0 * (s.spread_radius + s.blur_radius),
                    s.rect.size.height + 2.0 * (s.spread_radius + s.blur_radius),
                );
                viewport.intersects(&shadow_rect)
            })
            .cloned()
            .collect();

        let images: Vec<ImagePrimitive> = self
            .images
            .iter()
            .filter(|img| viewport.intersects(&img.rect))
            .cloned()
            .collect();

        let gradients: Vec<GradientPrimitive> = self
            .gradients
            .iter()
            .filter(|g| viewport.intersects(&g.rect))
            .cloned()
            .collect();

        let path_fills: Vec<PathFillPrimitive> = self
            .path_fills
            .iter()
            .filter(|pf| {
                // 使用路径顶点计算包围盒
                if pf.vertices.is_empty() {
                    return true; // 空路径保留
                }
                let mut min_x = f32::MAX;
                let mut min_y = f32::MAX;
                let mut max_x = f32::MIN;
                let mut max_y = f32::MIN;
                for chunk in pf.vertices.chunks_exact(2) {
                    min_x = min_x.min(chunk[0]);
                    min_y = min_y.min(chunk[1]);
                    max_x = max_x.max(chunk[0]);
                    max_y = max_y.max(chunk[1]);
                }
                let bbox = Rect::new(min_x, min_y, max_x - min_x, max_y - min_y);
                viewport.intersects(&bbox)
            })
            .cloned()
            .collect();

        let path_strokes: Vec<PathStrokePrimitive> = self
            .path_strokes
            .iter()
            .filter(|ps| {
                if ps.vertices.is_empty() {
                    return true;
                }
                let mut min_x = f32::MAX;
                let mut min_y = f32::MAX;
                let mut max_x = f32::MIN;
                let mut max_y = f32::MIN;
                for chunk in ps.vertices.chunks_exact(2) {
                    min_x = min_x.min(chunk[0]);
                    min_y = min_y.min(chunk[1]);
                    max_x = max_x.max(chunk[0]);
                    max_y = max_y.max(chunk[1]);
                }
                let bbox = Rect::new(min_x, min_y, max_x - min_x, max_y - min_y);
                viewport.intersects(&bbox)
            })
            .cloned()
            .collect();

        let result = RenderPrimitives {
            clips: self.clips.clone(), // clips 保留
            fills,
            rounded_rects,
            path_fills,
            path_strokes,
            strokes,
            gradients,
            shadows,
            images,
            glyphs: self.glyphs.clone(), // glyphs 保留
        };

        let culled_count = original_len - result.len();
        let mut stats = result.stats();
        stats.culled_count = culled_count;

        (result, stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn test_primitives_empty() {
        let p = RenderPrimitives::new();
        assert!(p.is_empty());
        assert_eq!(p.len(), 0);
        assert!(p.bounding_box().is_none());
    }

    #[test]
    fn test_primitives_add_fill() {
        let mut p = RenderPrimitives::new();
        p.add_fill(Rect::new(0.0, 0.0, 100.0, 100.0), Color::RED);
        assert!(!p.is_empty());
        assert_eq!(p.fills.len(), 1);
        assert_eq!(p.glyphs.len(), 0);
    }

    #[test]
    fn test_primitives_bounding_box() {
        let mut p = RenderPrimitives::new();
        p.add_fill(Rect::new(10.0, 20.0, 100.0, 50.0), Color::BLACK);
        p.add_fill(Rect::new(200.0, 100.0, 50.0, 50.0), Color::BLACK);

        let bb = p.bounding_box().unwrap();
        assert_eq!(bb.origin, Point::new(10.0, 20.0));
        // 右边界 250, 下边界 150
        assert_eq!(bb.right(), 250.0);
        assert_eq!(bb.bottom(), 150.0);
    }

    #[test]
    fn test_fill_primitive_fields() {
        let fill = FillPrimitive {
            rect: Rect::new(1.0, 2.0, 3.0, 4.0),
            color: Color::BLUE,
        };
        assert_eq!(fill.rect.origin.x, 1.0);
        assert_eq!(fill.color, Color::BLUE);
    }

    #[test]
    fn test_rounded_rect_uniform() {
        let rr = RoundedRectPrimitive::uniform(Rect::new(0.0, 0.0, 100.0, 50.0), Color::RED, 10.0);
        assert_eq!(rr.top_left_radius, 10.0);
        assert_eq!(rr.top_right_radius, 10.0);
        assert_eq!(rr.bottom_right_radius, 10.0);
        assert_eq!(rr.bottom_left_radius, 10.0);
    }

    #[test]
    fn test_rounded_rect_in_primitives() {
        let mut p = RenderPrimitives::new();
        p.add_rounded_rect(RoundedRectPrimitive::uniform(
            Rect::new(10.0, 10.0, 80.0, 80.0),
            Color::GREEN,
            15.0,
        ));
        assert_eq!(p.rounded_rects.len(), 1);
        assert!(!p.is_empty());
    }

    #[test]
    fn test_stroke_primitive() {
        let mut p = RenderPrimitives::new();
        p.add_stroke(StrokePrimitive {
            x1: 0.0,
            y1: 0.0,
            x2: 100.0,
            y2: 100.0,
            width: 2.0,
            color: Color::BLACK,
            style: LineStyle::Dashed,
            cap: LineCap::Butt,
        });
        assert_eq!(p.strokes.len(), 1);
        assert!(!p.is_empty());
    }

    #[test]
    fn test_clip_primitive() {
        let mut p = RenderPrimitives::new();
        p.add_clip(Rect::new(0.0, 0.0, 200.0, 200.0));
        assert_eq!(p.clips.len(), 1);
        assert!(!p.is_empty());
    }

    #[test]
    fn test_gradient_primitive() {
        let mut p = RenderPrimitives::new();
        p.add_gradient(GradientPrimitive {
            rect: Rect::new(0.0, 0.0, 100.0, 100.0),
            kind: GradientKind::Linear {
                x0: 0.0,
                y0: 0.0,
                x1: 100.0,
                y1: 0.0,
            },
            stops: vec![
                GradientStop {
                    offset: 0.0,
                    color: Color::RED,
                },
                GradientStop {
                    offset: 1.0,
                    color: Color::BLUE,
                },
            ],
        });
        assert_eq!(p.gradients.len(), 1);
    }

    #[test]
    fn test_shadow_primitive() {
        let mut p = RenderPrimitives::new();
        p.add_shadow(ShadowPrimitive {
            rect: Rect::new(10.0, 10.0, 80.0, 80.0),
            color: Color::rgba(0, 0, 0, 128),
            offset_x: 4.0,
            offset_y: 4.0,
            blur_radius: 8.0,
            spread_radius: 0.0,
        });
        assert_eq!(p.shadows.len(), 1);
    }

    #[test]
    fn test_image_primitive() {
        let mut p = RenderPrimitives::new();
        p.add_image(ImagePrimitive {
            rect: Rect::new(0.0, 0.0, 50.0, 50.0),
            image_key: ImageKey::new(42),
        });
        assert_eq!(p.images.len(), 1);
    }

    #[test]
    fn test_path_fill_primitive() {
        let mut p = RenderPrimitives::new();
        p.add_path_fill(vec![0.0, 0.0, 50.0, 0.0, 50.0, 50.0, 0.0, 50.0], Color::RED);
        assert_eq!(p.path_fills.len(), 1);
        assert!(!p.is_empty());
    }

    #[test]
    fn test_path_stroke_primitive() {
        let mut p = RenderPrimitives::new();
        p.add_path_stroke(vec![0.0, 0.0, 100.0, 100.0], Color::BLACK, 2.0, false);
        assert_eq!(p.path_strokes.len(), 1);
    }

    #[test]
    fn test_bounding_box_with_rounded_rect() {
        let mut p = RenderPrimitives::new();
        p.add_rounded_rect(RoundedRectPrimitive::uniform(
            Rect::new(5.0, 5.0, 50.0, 50.0),
            Color::BLACK,
            10.0,
        ));
        let bb = p.bounding_box().unwrap();
        assert_eq!(bb.left(), 5.0);
        assert_eq!(bb.top(), 5.0);
        assert_eq!(bb.right(), 55.0);
        assert_eq!(bb.bottom(), 55.0);
    }

    #[test]
    fn test_bounding_box_with_stroke() {
        let mut p = RenderPrimitives::new();
        p.add_stroke(StrokePrimitive {
            x1: 10.0,
            y1: 20.0,
            x2: 50.0,
            y2: 60.0,
            width: 4.0,
            color: Color::BLACK,
            style: LineStyle::Solid,
            cap: LineCap::Butt,
        });
        let bb = p.bounding_box().unwrap();
        assert_eq!(bb.left(), 8.0); // 10 - 2
        assert_eq!(bb.top(), 18.0); // 20 - 2
        assert_eq!(bb.right(), 52.0); // 50 + 2
        assert_eq!(bb.bottom(), 62.0); // 60 + 2
    }

    #[test]
    fn test_bounding_box_with_shadow() {
        let mut p = RenderPrimitives::new();
        p.add_shadow(ShadowPrimitive {
            rect: Rect::new(10.0, 10.0, 50.0, 50.0),
            color: Color::BLACK,
            offset_x: 5.0,
            offset_y: 5.0,
            blur_radius: 3.0,
            spread_radius: 2.0,
        });
        let bb = p.bounding_box().unwrap();
        assert_eq!(bb.left(), 10.0);
        assert_eq!(bb.top(), 10.0);
        assert_eq!(bb.right(), 70.0);
        assert_eq!(bb.bottom(), 70.0);
    }

    #[test]
    fn test_len_counts_all_types() {
        let mut p = RenderPrimitives::new();
        p.add_clip(Rect::new(0.0, 0.0, 100.0, 100.0));
        p.add_fill(Rect::new(0.0, 0.0, 10.0, 10.0), Color::RED);
        p.add_stroke(StrokePrimitive {
            x1: 0.0,
            y1: 0.0,
            x2: 10.0,
            y2: 10.0,
            width: 1.0,
            color: Color::BLACK,
            style: LineStyle::Solid,
            cap: LineCap::Butt,
        });
        assert!(p.len() >= 3);
    }

    #[test]
    fn test_line_style_equality() {
        assert_eq!(LineStyle::Solid, LineStyle::Solid);
        assert_ne!(LineStyle::Dashed, LineStyle::Dotted);
    }

    #[test]
    fn test_line_cap_equality() {
        assert_eq!(LineCap::Round, LineCap::Round);
        assert_ne!(LineCap::Butt, LineCap::Square);
    }

    #[test]
    fn test_gradient_kind_radial() {
        let kind = GradientKind::Radial {
            cx: 50.0,
            cy: 50.0,
            inner_radius: 0.0,
            outer_radius: 50.0,
        };
        if let GradientKind::Radial { outer_radius, .. } = kind {
            assert_eq!(outer_radius, 50.0);
        } else {
            panic!("Expected Radial");
        }
    }

    #[test]
    fn test_glyph_primitive_creation() {
        let g = GlyphPrimitive {
            x: 10.0,
            y: 20.0,
            font_size: 16.0,
            color: Color::BLACK,
            glyph_id: 42,
            font_id: FontId(1),
            bitmap_width: Some(12),
            bitmap_height: Some(16),
        };
        assert_eq!(g.x, 10.0);
        assert_eq!(g.font_id, FontId(1));
        assert_eq!(g.bitmap_width, Some(12));
    }

    #[test]
    fn test_glyph_in_render_primitives() {
        let mut p = RenderPrimitives::new();
        p.add_glyph(GlyphPrimitive {
            x: 0.0,
            y: 0.0,
            font_size: 12.0,
            color: Color::BLACK,
            glyph_id: 65,
            font_id: FontId(0),
            bitmap_width: None,
            bitmap_height: None,
        });
        assert_eq!(p.glyphs.len(), 1);
        assert!(!p.is_empty());
    }

    #[test]
    fn test_font_id_equality() {
        assert_eq!(FontId(1), FontId(1));
        assert_ne!(FontId(1), FontId(2));
    }

    #[test]
    fn test_bounding_box_with_glyphs() {
        let mut p = RenderPrimitives::new();
        p.add_glyph(GlyphPrimitive {
            x: 5.0,
            y: 10.0,
            font_size: 16.0,
            color: Color::BLACK,
            glyph_id: 0,
            font_id: FontId(0),
            bitmap_width: None,
            bitmap_height: None,
        });
        let bb = p.bounding_box().unwrap();
        assert_eq!(bb.left(), 5.0);
        assert_eq!(bb.top(), 10.0);
        assert_eq!(bb.right(), 21.0); // x + font_size
        assert_eq!(bb.bottom(), 26.0); // y + font_size
    }

    #[test]
    fn test_bounding_box_with_images() {
        let mut p = RenderPrimitives::new();
        p.add_image(ImagePrimitive {
            rect: Rect::new(50.0, 60.0, 100.0, 80.0),
            image_key: ImageKey::new(1),
        });
        let bb = p.bounding_box().unwrap();
        assert_eq!(bb.left(), 50.0);
        assert_eq!(bb.top(), 60.0);
        assert_eq!(bb.right(), 150.0);
        assert_eq!(bb.bottom(), 140.0);
    }

    #[test]
    fn test_bounding_box_with_gradient() {
        let mut p = RenderPrimitives::new();
        p.add_gradient(GradientPrimitive {
            rect: Rect::new(0.0, 0.0, 200.0, 100.0),
            kind: GradientKind::Linear {
                x0: 0.0,
                y0: 0.0,
                x1: 200.0,
                y1: 0.0,
            },
            stops: vec![],
        });
        let bb = p.bounding_box().unwrap();
        assert_eq!(bb.right(), 200.0);
        assert_eq!(bb.bottom(), 100.0);
    }

    #[test]
    fn test_bounding_box_with_path_fill() {
        let mut p = RenderPrimitives::new();
        p.add_path_fill(vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0], Color::RED);
        let bb = p.bounding_box().unwrap();
        // Points: (10,20), (30,40), (50,60)
        assert_eq!(bb.left(), 10.0);
        assert_eq!(bb.top(), 20.0);
        assert_eq!(bb.right(), 50.0);
        assert_eq!(bb.bottom(), 60.0);
    }

    #[test]
    fn test_render_primitives_mixed_types_count() {
        let mut p = RenderPrimitives::new();
        p.add_clip(Rect::new(0.0, 0.0, 100.0, 100.0));
        p.add_fill(Rect::new(0.0, 0.0, 50.0, 50.0), Color::RED);
        p.add_fill(Rect::new(0.0, 0.0, 50.0, 50.0), Color::BLUE);
        p.add_stroke(StrokePrimitive {
            x1: 0.0,
            y1: 0.0,
            x2: 10.0,
            y2: 10.0,
            width: 1.0,
            color: Color::BLACK,
            style: LineStyle::Solid,
            cap: LineCap::Round,
        });
        p.add_glyph(GlyphPrimitive {
            x: 0.0,
            y: 0.0,
            font_size: 12.0,
            color: Color::BLACK,
            glyph_id: 0,
            font_id: FontId(0),
            bitmap_width: None,
            bitmap_height: None,
        });
        assert_eq!(p.len(), 5);
        assert!(!p.is_empty());
    }

    #[test]
    fn test_rounded_rect_individual_radii() {
        let rr = RoundedRectPrimitive {
            rect: Rect::new(0.0, 0.0, 100.0, 100.0),
            color: Color::GREEN,
            top_left_radius: 5.0,
            top_right_radius: 10.0,
            bottom_right_radius: 15.0,
            bottom_left_radius: 20.0,
        };
        assert_eq!(rr.top_left_radius, 5.0);
        assert_eq!(rr.top_right_radius, 10.0);
        assert_eq!(rr.bottom_right_radius, 15.0);
        assert_eq!(rr.bottom_left_radius, 20.0);
    }

    // -- 边界条件测试 --
    /// 测试 bounding_box 只包含 clips 时返回 None
    #[test]
    fn test_bounding_box_clips_only_returns_none() {
        let mut p = RenderPrimitives::new();
        p.add_clip(Rect::new(0.0, 0.0, 100.0, 100.0));
        p.add_clip(Rect::new(50.0, 50.0, 100.0, 100.0));
        // clips 不参与 bounding_box 计算
        assert!(p.bounding_box().is_none());
    }

    /// 测试 RenderPrimitives::len 包含所有类型
    #[test]
    fn test_len_all_primitive_types() {
        let mut p = RenderPrimitives::new();
        p.add_clip(Rect::new(0.0, 0.0, 10.0, 10.0));
        p.add_fill(Rect::new(0.0, 0.0, 10.0, 10.0), Color::BLACK);
        p.add_rounded_rect(RoundedRectPrimitive::uniform(
            Rect::new(0.0, 0.0, 10.0, 10.0),
            Color::BLACK,
            5.0,
        ));
        p.add_path_fill(vec![0.0, 0.0, 10.0, 10.0], Color::BLACK);
        p.add_path_stroke(vec![0.0, 0.0, 10.0, 10.0], Color::BLACK, 1.0, false);
        p.add_stroke(StrokePrimitive {
            x1: 0.0,
            y1: 0.0,
            x2: 10.0,
            y2: 10.0,
            width: 1.0,
            color: Color::BLACK,
            style: LineStyle::Solid,
            cap: LineCap::Butt,
        });
        p.add_gradient(GradientPrimitive {
            rect: Rect::new(0.0, 0.0, 10.0, 10.0),
            kind: GradientKind::Linear {
                x0: 0.0,
                y0: 0.0,
                x1: 10.0,
                y1: 0.0,
            },
            stops: vec![],
        });
        p.add_shadow(ShadowPrimitive {
            rect: Rect::new(0.0, 0.0, 10.0, 10.0),
            color: Color::BLACK,
            offset_x: 0.0,
            offset_y: 0.0,
            blur_radius: 0.0,
            spread_radius: 0.0,
        });
        p.add_image(ImagePrimitive {
            rect: Rect::new(0.0, 0.0, 10.0, 10.0),
            image_key: ImageKey::new(0),
        });
        p.add_glyph(GlyphPrimitive {
            x: 0.0,
            y: 0.0,
            font_size: 12.0,
            color: Color::BLACK,
            glyph_id: 0,
            font_id: FontId(0),
            bitmap_width: None,
            bitmap_height: None,
        });
        assert_eq!(p.len(), 10);
    }

    /// 测试 bounding_box 包含负坐标
    #[test]
    fn test_bounding_box_negative_coordinates() {
        let mut p = RenderPrimitives::new();
        p.add_fill(Rect::new(-50.0, -30.0, 100.0, 60.0), Color::BLACK);
        let bb = p.bounding_box().unwrap();
        assert_eq!(bb.left(), -50.0);
        assert_eq!(bb.top(), -30.0);
        assert_eq!(bb.right(), 50.0);
        assert_eq!(bb.bottom(), 30.0);
    }

    /// 透明度 alpha=0.0 的图元应不可见（预乘 alpha 后所有通道为零）。
    #[test]
    fn test_composite_primitive_opacity_zero() {
        // alpha=0 的颜色（完全透明），预乘后 RGB 通道全部归零
        let invisible_color = Color::rgba(255, 0, 0, 0);
        let premultiplied = invisible_color.premultiplied();
        assert!(premultiplied[0].abs() < f32::EPSILON, "R 通道预乘后应为 0");
        assert!(premultiplied[1].abs() < f32::EPSILON, "G 通道预乘后应为 0");
        assert!(premultiplied[2].abs() < f32::EPSILON, "B 通道预乘后应为 0");
        assert!(premultiplied[3].abs() < f32::EPSILON, "A 通道预乘后应为 0");

        // 添加一个完全透明的 fill 图元
        let mut p = RenderPrimitives::new();
        p.add_fill(Rect::new(0.0, 0.0, 100.0, 100.0), invisible_color);
        // 图元本身存在（len=1），但颜色完全透明
        assert_eq!(p.fills.len(), 1);
        assert_eq!(p.fills[0].color.a, 0, "alpha 应为 0");

        // 同理：添加一个完全透明的阴影图元
        p.add_shadow(ShadowPrimitive {
            rect: Rect::new(10.0, 10.0, 50.0, 50.0),
            color: Color::TRANSPARENT,
            offset_x: 5.0,
            offset_y: 5.0,
            blur_radius: 3.0,
            spread_radius: 0.0,
        });
        let shadow = &p.shadows[0];
        assert_eq!(shadow.color.a, 0);
        let shadow_premul = shadow.color.premultiplied();
        assert!(shadow_premul.iter().all(|&c| c.abs() < f32::EPSILON));
    }

    /// 测试 path_fill 空 vertices 的 bounding_box
    #[test]
    fn test_bounding_box_empty_path_fill_vertices() {
        let mut p = RenderPrimitives::new();
        p.add_path_fill(vec![], Color::BLACK);
        // Empty vertices means nothing contributes to bounding box
        assert!(p.bounding_box().is_none());
    }

    /// 测试 StrokePrimitive width=0.0
    #[test]
    fn test_stroke_primitive_zero_width() {
        let s = StrokePrimitive {
            x1: 0.0,
            y1: 0.0,
            x2: 10.0,
            y2: 10.0,
            width: 0.0,
            color: Color::BLACK,
            style: LineStyle::Solid,
            cap: LineCap::Butt,
        };
        assert_eq!(s.width, 0.0);

        let mut p = RenderPrimitives::new();
        p.add_stroke(s);
        let bb = p.bounding_box().unwrap();
        assert_eq!(bb.left(), 0.0);
        assert_eq!(bb.top(), 0.0);
        assert_eq!(bb.right(), 10.0);
        assert_eq!(bb.bottom(), 10.0);
    }

    /// 测试 bounding_box 在 GlyphPrimitive 含 bitmap_width/bitmap_height 时
    /// 仍基于 font_size 计算包围盒（不使用 bitmap 尺寸）。
    ///
    /// 当 glyph 有预缓存位图时，bounding_box 应忽略 bitmap_width/bitmap_height，
    /// 而使用 (x, y) 到 (x + font_size, y + font_size) 的矩形。
    #[test]
    fn test_bounding_box_glyph_with_bitmap_dims() {
        let mut p = RenderPrimitives::new();
        p.add_glyph(GlyphPrimitive {
            x: 100.0,
            y: 200.0,
            font_size: 24.0,
            color: Color::BLACK,
            glyph_id: 65,
            font_id: FontId(0),
            bitmap_width: Some(12),
            bitmap_height: Some(16),
        });

        let bb = p.bounding_box().expect("glyph 应产生包围盒");
        // bounding_box 使用 font_size，不使用 bitmap 尺寸
        assert_eq!(bb.left(), 100.0, "left 应为 glyph.x");
        assert_eq!(bb.top(), 200.0, "top 应为 glyph.y");
        assert_eq!(bb.right(), 124.0, "right 应为 x + font_size = 124");
        assert_eq!(bb.bottom(), 224.0, "bottom 应为 y + font_size = 224");
    }

    /// 测试 ShadowPrimitive 大模糊半径 bounding_box 计算。
    #[test]
    fn test_edge_shadow_large_blur_radius_bounding_box() {
        let mut p = RenderPrimitives::new();
        // rect at (100,100) size (50,50), offset (0,0), spread=0, blur=200
        p.add_shadow(ShadowPrimitive {
            rect: Rect::new(100.0, 100.0, 50.0, 50.0),
            color: Color::BLACK,
            offset_x: 0.0,
            offset_y: 0.0,
            blur_radius: 200.0,
            spread_radius: 0.0,
        });
        let bb = p.bounding_box().unwrap();
        // left  = 100 + 0 - 0 - 200 = -100
        // top   = 100 + 0 - 0 - 200 = -100
        // right = 150 + 0 + 0 + 200 = 350
        // bottom= 150 + 0 + 0 + 200 = 350
        assert_eq!(bb.left(), -100.0);
        assert_eq!(bb.top(), -100.0);
        assert_eq!(bb.right(), 350.0);
        assert_eq!(bb.bottom(), 350.0);
    }

    /// 测试 ShadowPrimitive 负偏移 bounding_box 计算。
    #[test]
    fn test_edge_shadow_negative_offset_bounding_box() {
        let mut p = RenderPrimitives::new();
        // rect at (50,50) size (40,40), offset (-10,-20), blur=0, spread=0
        p.add_shadow(ShadowPrimitive {
            rect: Rect::new(50.0, 50.0, 40.0, 40.0),
            color: Color::BLACK,
            offset_x: -10.0,
            offset_y: -20.0,
            blur_radius: 0.0,
            spread_radius: 0.0,
        });
        let bb = p.bounding_box().unwrap();
        // left  = 50 + (-10) - 0 - 0 = 40
        // top   = 50 + (-20) - 0 - 0 = 30
        // right = 90 + (-10) + 0 + 0 = 80
        // bottom= 90 + (-20) + 0 + 0 = 70
        assert_eq!(bb.left(), 40.0);
        assert_eq!(bb.top(), 30.0);
        assert_eq!(bb.right(), 80.0);
        assert_eq!(bb.bottom(), 70.0);
    }

    /// 测试 ShadowPrimitive 大扩展半径 bounding_box 计算。
    #[test]
    fn test_edge_shadow_large_spread_radius_bounding_box() {
        let mut p = RenderPrimitives::new();
        // rect at (20,20) size (30,30), offset (0,0), blur=0, spread=50
        p.add_shadow(ShadowPrimitive {
            rect: Rect::new(20.0, 20.0, 30.0, 30.0),
            color: Color::BLACK,
            offset_x: 0.0,
            offset_y: 0.0,
            blur_radius: 0.0,
            spread_radius: 50.0,
        });
        let bb = p.bounding_box().unwrap();
        // left  = 20 + 0 - 50 - 0 = -30
        // top   = 20 + 0 - 50 - 0 = -30
        // right = 50 + 0 + 50 + 0 = 100
        // bottom= 50 + 0 + 50 + 0 = 100
        assert_eq!(bb.left(), -30.0);
        assert_eq!(bb.top(), -30.0);
        assert_eq!(bb.right(), 100.0);
        assert_eq!(bb.bottom(), 100.0);
    }

    /// 测试多个 ShadowPrimitive bounding_box 合并计算。
    #[test]
    fn test_edge_multiple_shadows_bounding_box_merge() {
        let mut p = RenderPrimitives::new();
        // Shadow 1: rect(0,0,50,50) offset(5,5) blur=2 spread=1
        // left=0+5-1-2=2, top=0+5-1-2=2, right=50+5+1+2=58, bottom=50+5+1+2=58
        p.add_shadow(ShadowPrimitive {
            rect: Rect::new(0.0, 0.0, 50.0, 50.0),
            color: Color::BLACK,
            offset_x: 5.0,
            offset_y: 5.0,
            blur_radius: 2.0,
            spread_radius: 1.0,
        });
        // Shadow 2: rect(200,200,50,50) offset(-5,-5) blur=10 spread=0
        // left=200+(-5)-0-10=185, top=200+(-5)-0-10=185, right=250+(-5)+0+10=255, bottom=250+(-5)+0+10=255
        p.add_shadow(ShadowPrimitive {
            rect: Rect::new(200.0, 200.0, 50.0, 50.0),
            color: Color::BLACK,
            offset_x: -5.0,
            offset_y: -5.0,
            blur_radius: 10.0,
            spread_radius: 0.0,
        });
        let bb = p.bounding_box().unwrap();
        // Merged: min of lefts, min of tops, max of rights, max of bottoms
        assert_eq!(bb.left(), 2.0);
        assert_eq!(bb.top(), 2.0);
        assert_eq!(bb.right(), 255.0);
        assert_eq!(bb.bottom(), 255.0);
    }

    /// 测试 ImagePrimitive 不同 ImageKey 区分。
    #[test]
    fn test_edge_image_primitive_different_keys() {
        let mut p = RenderPrimitives::new();
        let key_a = ImageKey::new(100);
        let key_b = ImageKey::new(200);
        p.add_image(ImagePrimitive {
            rect: Rect::new(0.0, 0.0, 50.0, 50.0),
            image_key: key_a,
        });
        p.add_image(ImagePrimitive {
            rect: Rect::new(10.0, 10.0, 50.0, 50.0),
            image_key: key_b,
        });
        assert_eq!(p.images.len(), 2);
        // Verify keys are distinct
        assert_ne!(p.images[0].image_key, p.images[1].image_key);
        assert_eq!(p.images[0].image_key, ImageKey::new(100));
        assert_eq!(p.images[1].image_key, ImageKey::new(200));
        // Verify rects are preserved independently
        assert_eq!(p.images[0].rect.origin.x, 0.0);
        assert_eq!(p.images[1].rect.origin.x, 10.0);
    }

    /// 测试 RenderPrimitives 包含阴影和图片时的 len 计数。
    #[test]
    fn test_edge_len_with_shadows_and_images() {
        let mut p = RenderPrimitives::new();
        p.add_shadow(ShadowPrimitive {
            rect: Rect::new(0.0, 0.0, 100.0, 100.0),
            color: Color::BLACK,
            offset_x: 3.0,
            offset_y: 3.0,
            blur_radius: 5.0,
            spread_radius: 0.0,
        });
        p.add_shadow(ShadowPrimitive {
            rect: Rect::new(50.0, 50.0, 100.0, 100.0),
            color: Color::rgba(0, 0, 0, 80),
            offset_x: 0.0,
            offset_y: 0.0,
            blur_radius: 10.0,
            spread_radius: 2.0,
        });
        p.add_image(ImagePrimitive {
            rect: Rect::new(0.0, 0.0, 200.0, 200.0),
            image_key: ImageKey::new(1),
        });
        p.add_image(ImagePrimitive {
            rect: Rect::new(10.0, 10.0, 150.0, 150.0),
            image_key: ImageKey::new(2),
        });
        p.add_image(ImagePrimitive {
            rect: Rect::new(20.0, 20.0, 100.0, 100.0),
            image_key: ImageKey::new(3),
        });
        // 2 shadows + 3 images = 5 total
        assert_eq!(p.shadows.len(), 2);
        assert_eq!(p.images.len(), 3);
        assert_eq!(p.len(), 5);
        assert!(!p.is_empty());
    }

    /// 测试 ShadowPrimitive 零尺寸矩形。
    #[test]
    fn test_edge_shadow_zero_size_rect() {
        let mut p = RenderPrimitives::new();
        // rect at (50,50) size (0,0) — left=right=50, top=bottom=50
        // With offset=0, blur=5, spread=3:
        // left  = 50 + 0 - 3 - 5 = 42
        // top   = 50 + 0 - 3 - 5 = 42
        // right = 50 + 0 + 3 + 5 = 58
        // bottom= 50 + 0 + 3 + 5 = 58
        p.add_shadow(ShadowPrimitive {
            rect: Rect::new(50.0, 50.0, 0.0, 0.0),
            color: Color::BLACK,
            offset_x: 0.0,
            offset_y: 0.0,
            blur_radius: 5.0,
            spread_radius: 3.0,
        });
        let bb = p.bounding_box().unwrap();
        assert_eq!(bb.left(), 42.0);
        assert_eq!(bb.top(), 42.0);
        assert_eq!(bb.right(), 58.0);
        assert_eq!(bb.bottom(), 58.0);
    }

    /// 测试 GradientPrimitive::Linear 加入 RenderPrimitives。
    #[test]
    fn test_gradient_primitive_linear_in_primitives() {
        let mut p = RenderPrimitives::new();
        p.add_gradient(GradientPrimitive {
            rect: Rect::new(0.0, 0.0, 100.0, 100.0),
            kind: GradientKind::Linear {
                x0: 0.0,
                y0: 0.0,
                x1: 100.0,
                y1: 0.0,
            },
            stops: vec![
                GradientStop {
                    offset: 0.0,
                    color: Color::RED,
                },
                GradientStop {
                    offset: 1.0,
                    color: Color::BLUE,
                },
            ],
        });
        assert_eq!(p.gradients.len(), 1);
    }

    /// 测试 GradientPrimitive::Radial 加入 RenderPrimitives。
    #[test]
    fn test_gradient_primitive_radial_in_primitives() {
        let mut p = RenderPrimitives::new();
        p.add_gradient(GradientPrimitive {
            rect: Rect::new(0.0, 0.0, 100.0, 100.0),
            kind: GradientKind::Radial {
                cx: 50.0,
                cy: 50.0,
                inner_radius: 10.0,
                outer_radius: 50.0,
            },
            stops: vec![
                GradientStop {
                    offset: 0.0,
                    color: Color::WHITE,
                },
                GradientStop {
                    offset: 1.0,
                    color: Color::BLACK,
                },
            ],
        });
        assert_eq!(p.gradients.len(), 1);
        if let GradientKind::Radial {
            cx,
            cy,
            inner_radius,
            outer_radius,
        } = &p.gradients[0].kind
        {
            assert_eq!(*cx, 50.0);
            assert_eq!(*cy, 50.0);
            assert_eq!(*inner_radius, 10.0);
            assert_eq!(*outer_radius, 50.0);
        } else {
            panic!("Expected Radial gradient kind");
        }
    }

    /// 测试渐变 bounding_box 计算。
    #[test]
    fn test_gradient_bounding_box() {
        let mut p = RenderPrimitives::new();
        p.add_gradient(GradientPrimitive {
            rect: Rect::new(10.0, 20.0, 100.0, 80.0),
            kind: GradientKind::Linear {
                x0: 10.0,
                y0: 20.0,
                x1: 110.0,
                y1: 20.0,
            },
            stops: vec![
                GradientStop {
                    offset: 0.0,
                    color: Color::RED,
                },
                GradientStop {
                    offset: 1.0,
                    color: Color::BLUE,
                },
            ],
        });
        let bb = p.bounding_box().unwrap();
        assert_eq!(bb.left(), 10.0);
        assert_eq!(bb.top(), 20.0);
        assert_eq!(bb.right(), 110.0);
        assert_eq!(bb.bottom(), 100.0);
    }

    /// 测试多个渐变图元 bounding_box 合并。
    #[test]
    fn test_multiple_gradients_bounding_box() {
        let mut p = RenderPrimitives::new();
        p.add_gradient(GradientPrimitive {
            rect: Rect::new(10.0, 20.0, 100.0, 80.0),
            kind: GradientKind::Linear {
                x0: 10.0,
                y0: 20.0,
                x1: 110.0,
                y1: 20.0,
            },
            stops: vec![],
        });
        p.add_gradient(GradientPrimitive {
            rect: Rect::new(200.0, 150.0, 50.0, 50.0),
            kind: GradientKind::Radial {
                cx: 225.0,
                cy: 175.0,
                inner_radius: 0.0,
                outer_radius: 25.0,
            },
            stops: vec![],
        });
        let bb = p.bounding_box().unwrap();
        // First: left=10, top=20, right=110, bottom=100
        // Second: left=200, top=150, right=250, bottom=200
        // Merged: left=10, top=20, right=250, bottom=200
        assert_eq!(bb.left(), 10.0);
        assert_eq!(bb.top(), 20.0);
        assert_eq!(bb.right(), 250.0);
        assert_eq!(bb.bottom(), 200.0);
    }

    /// 测试 GradientStop 顺序。
    #[test]
    fn test_gradient_stops_order() {
        let mut p = RenderPrimitives::new();
        p.add_gradient(GradientPrimitive {
            rect: Rect::new(0.0, 0.0, 100.0, 100.0),
            kind: GradientKind::Linear {
                x0: 0.0,
                y0: 0.0,
                x1: 100.0,
                y1: 0.0,
            },
            stops: vec![
                GradientStop {
                    offset: 0.0,
                    color: Color::RED,
                },
                GradientStop {
                    offset: 0.5,
                    color: Color::GREEN,
                },
                GradientStop {
                    offset: 1.0,
                    color: Color::BLUE,
                },
            ],
        });
        let stops = &p.gradients[0].stops;
        assert_eq!(stops.len(), 3);
        assert_eq!(stops[0].offset, 0.0);
        assert_eq!(stops[1].offset, 0.5);
        assert_eq!(stops[2].offset, 1.0);
        // Verify order preserved: offsets must be monotonically increasing
        for i in 1..stops.len() {
            assert!(
                stops[i].offset > stops[i - 1].offset,
                "stops should be in ascending order"
            );
        }
    }

    /// 测试 RenderPrimitives::default 等价于 new
    ///
    /// 验证 default() 和 new() 都产生空的图元列表。
    #[test]
    fn test_render_primitives_default_equals_new() {
        let p1 = RenderPrimitives::new();
        let p2 = RenderPrimitives::default();
        assert!(p1.is_empty());
        assert!(p2.is_empty());
        assert_eq!(p1.len(), 0);
        assert_eq!(p2.len(), 0);
    }

    /// 测试 bounding_box 包含重合点时返回 None
    ///
    /// 当所有图元只有一个点（面积为 0）时，
    /// min_x == max_x 或 min_y == max_y，bounding_box 应返回 None。
    #[test]
    fn test_bounding_box_coincident_points() {
        let mut p = RenderPrimitives::new();
        // 零面积矩形
        p.add_fill(Rect::new(10.0, 10.0, 0.0, 0.0), Color::BLACK);
        // left == right (10.0 == 10.0) 和 top == bottom (10.0 == 10.0)
        assert!(p.bounding_box().is_none(), "零面积矩形不应产生包围盒");
    }

    /// 测试 path_stroke 空 vertices 不影响 bounding_box
    #[test]
    fn test_bounding_box_empty_path_stroke_vertices() {
        let mut p = RenderPrimitives::new();
        p.add_path_stroke(vec![], Color::BLACK, 1.0, false);
        assert!(p.bounding_box().is_none(), "空 path_stroke 不应产生包围盒");
    }

    /// 测试 add_glyph 多次添加后 len 正确
    #[test]
    fn test_add_glyph_multiple() {
        let mut p = RenderPrimitives::new();
        for i in 0..10 {
            p.add_glyph(GlyphPrimitive {
                x: i as f32,
                y: 0.0,
                font_size: 12.0,
                color: Color::BLACK,
                glyph_id: i,
                font_id: FontId(0),
                bitmap_width: None,
                bitmap_height: None,
            });
        }
        assert_eq!(p.glyphs.len(), 10);
        assert_eq!(p.len(), 10);
        assert!(!p.is_empty());
    }

    // ── RenderStats + batch_fills + cull_invisible 测试 ──

    #[test]
    fn test_stats_empty_primitives() {
        let p = RenderPrimitives::new();
        let stats = p.stats();
        assert_eq!(stats.total_primitives(), 0);
        assert_eq!(stats.estimated_draw_calls, 0);
    }

    #[test]
    fn test_stats_single_fill() {
        let mut p = RenderPrimitives::new();
        p.add_fill(Rect::new(0.0, 0.0, 100.0, 100.0), Color::RED);
        let stats = p.stats();
        assert_eq!(stats.fill_count, 1);
        assert_eq!(stats.estimated_draw_calls, 1);
    }

    #[test]
    fn test_stats_same_color_fills_batched_draw_calls() {
        let mut p = RenderPrimitives::new();
        // 5 个相同颜色的 fill → 只需 1 个 draw call
        for i in 0..5 {
            p.add_fill(Rect::new(i as f32 * 100.0, 0.0, 50.0, 50.0), Color::RED);
        }
        let stats = p.stats();
        assert_eq!(stats.fill_count, 5);
        assert_eq!(stats.estimated_draw_calls, 1); // 同色合并
    }

    #[test]
    fn test_stats_different_color_fills_separate_draw_calls() {
        let mut p = RenderPrimitives::new();
        p.add_fill(Rect::new(0.0, 0.0, 50.0, 50.0), Color::RED);
        p.add_fill(Rect::new(100.0, 0.0, 50.0, 50.0), Color::BLUE);
        p.add_fill(Rect::new(200.0, 0.0, 50.0, 50.0), Color::GREEN);
        let stats = p.stats();
        assert_eq!(stats.fill_count, 3);
        assert_eq!(stats.estimated_draw_calls, 3); // 不同颜色各一次
    }

    #[test]
    fn test_stats_mixed_primitives() {
        let mut p = RenderPrimitives::new();
        p.add_fill(Rect::new(0.0, 0.0, 100.0, 100.0), Color::RED);
        p.add_fill(Rect::new(0.0, 0.0, 50.0, 50.0), Color::RED); // 同色
        p.add_glyph(GlyphPrimitive {
            x: 0.0,
            y: 0.0,
            font_size: 12.0,
            color: Color::BLACK,
            glyph_id: 65,
            font_id: FontId(0),
            bitmap_width: None,
            bitmap_height: None,
        });
        p.add_glyph(GlyphPrimitive {
            x: 10.0,
            y: 0.0,
            font_size: 12.0,
            color: Color::BLACK,
            glyph_id: 66,
            font_id: FontId(0),
            bitmap_width: None,
            bitmap_height: None,
        });
        // 同色 fills = 1 draw call, 同 font+color glyphs = 1 draw call → total 2
        let stats = p.stats();
        assert_eq!(stats.total_primitives(), 4);
        assert_eq!(stats.estimated_draw_calls, 2);
    }

    #[test]
    fn test_batch_fills_no_merge_different_colors() {
        let mut p = RenderPrimitives::new();
        p.add_fill(Rect::new(0.0, 0.0, 100.0, 50.0), Color::RED);
        p.add_fill(Rect::new(0.0, 0.0, 100.0, 50.0), Color::BLUE);
        let batched = p.batch_fills();
        assert_eq!(batched.fills.len(), 2);
    }

    #[test]
    fn test_batch_fills_merge_adjacent_same_color() {
        let mut p = RenderPrimitives::new();
        // 两个同色、同宽、垂直相邻的矩形 → 应合并
        p.add_fill(Rect::new(0.0, 0.0, 100.0, 50.0), Color::RED);
        p.add_fill(Rect::new(0.0, 50.0, 100.0, 50.0), Color::RED);
        let batched = p.batch_fills();
        // 合并后应该只有 1 个 fill（覆盖 0,0 到 100,100）
        assert_eq!(batched.fills.len(), 1);
        let merged = &batched.fills[0];
        assert_eq!(merged.rect.origin.y, 0.0);
        assert_eq!(merged.rect.size.height, 100.0);
    }

    #[test]
    fn test_batch_fills_no_merge_non_adjacent() {
        let mut p = RenderPrimitives::new();
        p.add_fill(Rect::new(0.0, 0.0, 100.0, 50.0), Color::RED);
        p.add_fill(Rect::new(0.0, 200.0, 100.0, 50.0), Color::RED); // 不相邻
        let batched = p.batch_fills();
        assert_eq!(batched.fills.len(), 2);
    }

    #[test]
    fn test_batch_fills_preserves_other_primitives() {
        let mut p = RenderPrimitives::new();
        p.add_fill(Rect::new(0.0, 0.0, 100.0, 50.0), Color::RED);
        p.add_fill(Rect::new(0.0, 50.0, 100.0, 50.0), Color::RED);
        p.add_glyph(GlyphPrimitive {
            x: 0.0,
            y: 0.0,
            font_size: 12.0,
            color: Color::BLACK,
            glyph_id: 65,
            font_id: FontId(0),
            bitmap_width: None,
            bitmap_height: None,
        });
        let batched = p.batch_fills();
        assert_eq!(batched.fills.len(), 1); // 合并后 1 个
        assert_eq!(batched.glyphs.len(), 1); // glyphs 不变
    }

    #[test]
    fn test_cull_invisible_removes_offscreen_fills() {
        let mut p = RenderPrimitives::new();
        let viewport = Rect::new(0.0, 0.0, 800.0, 600.0);
        p.add_fill(Rect::new(10.0, 10.0, 50.0, 50.0), Color::RED); // 在视口内
        p.add_fill(Rect::new(900.0, 10.0, 50.0, 50.0), Color::RED); // 在视口外
        let (culled, stats) = p.cull_invisible(viewport);
        assert_eq!(culled.fills.len(), 1);
        assert_eq!(stats.culled_count, 1);
    }

    #[test]
    fn test_cull_invisible_keeps_clips_and_glyphs() {
        let mut p = RenderPrimitives::new();
        let viewport = Rect::new(0.0, 0.0, 800.0, 600.0);
        p.add_clip(Rect::new(0.0, 0.0, 1000.0, 1000.0)); // 超出视口但保留
        p.add_glyph(GlyphPrimitive {
            x: 900.0,
            y: 10.0,
            font_size: 12.0,
            color: Color::BLACK,
            glyph_id: 65,
            font_id: FontId(0),
            bitmap_width: None,
            bitmap_height: None,
        }); // 超出视口但保留
        let (culled, _) = p.cull_invisible(viewport);
        assert_eq!(culled.clips.len(), 1);
        assert_eq!(culled.glyphs.len(), 1);
    }

    #[test]
    fn test_cull_invisible_nothing_removed_when_all_visible() {
        let mut p = RenderPrimitives::new();
        let viewport = Rect::new(0.0, 0.0, 800.0, 600.0);
        p.add_fill(Rect::new(10.0, 10.0, 50.0, 50.0), Color::RED);
        p.add_fill(Rect::new(100.0, 100.0, 50.0, 50.0), Color::BLUE);
        let (culled, stats) = p.cull_invisible(viewport);
        assert_eq!(culled.fills.len(), 2);
        assert_eq!(stats.culled_count, 0);
    }

    #[test]
    fn test_cull_invisible_partial_overlap_kept() {
        let mut p = RenderPrimitives::new();
        let viewport = Rect::new(0.0, 0.0, 800.0, 600.0);
        // 矩形部分在视口内
        p.add_fill(Rect::new(750.0, 10.0, 100.0, 50.0), Color::RED);
        let (culled, stats) = p.cull_invisible(viewport);
        assert_eq!(culled.fills.len(), 1); // 部分可见保留
        assert_eq!(stats.culled_count, 0);
    }
}
