//! RenderPrimitives 批量操作 — bounding_box、snapshot、stats、batch_fills、cull_invisible
//!
//! 将较大的方法实现从 mod.rs 类型定义中分离出来。

use super::*;

impl RenderPrimitives {
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
            + self.filters.len()
            + self.blend_modes.len()
            + self.transforms.len()
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
            filter_count: self.filters.len(),
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
    /// 保持颜色首次出现的顺序以确保确定性输出。
    /// 返回优化后的新 `RenderPrimitives`，原始数据不变。
    pub fn batch_fills(&self) -> RenderPrimitives {
        if self.fills.len() <= 1 {
            return self.clone();
        }

        // 按颜色分组，保持首次出现的顺序
        let mut color_order: Vec<[u8; 4]> = Vec::new();
        let mut color_seen: std::collections::HashSet<[u8; 4]> = std::collections::HashSet::new();
        let mut color_groups: std::collections::HashMap<[u8; 4], Vec<&FillPrimitive>> =
            std::collections::HashMap::new();
        for fill in &self.fills {
            let key = [fill.color.r, fill.color.g, fill.color.b, fill.color.a];
            if !color_seen.contains(&key) {
                color_seen.insert(key);
                color_order.push(key);
            }
            color_groups.entry(key).or_default().push(fill);
        }

        let mut batched_fills = Vec::new();

        for color_key in color_order {
            let fills = color_groups.get(&color_key).unwrap();
            if fills.is_empty() {
                continue;
            }

            let color = fills[0].color;

            // 尝试在垂直方向合并同色矩形
            // 简单策略：合并完全同列（x 和 width 相同）且垂直相邻的矩形
            let merged: Vec<Rect> = fills.iter().map(|f| f.rect).collect();

            // 按列（x, width）分组，在每列内按 y 排序
            let mut column_order: Vec<(u32, u32)> = Vec::new();
            let mut column_seen: std::collections::HashSet<(u32, u32)> = std::collections::HashSet::new();
            let mut columns: std::collections::HashMap<(u32, u32), Vec<Rect>> = std::collections::HashMap::new();
            for rect in &merged {
                // 使用固定精度来分组（避免浮点误差）
                let x_key = (rect.origin.x.to_bits(), rect.size.width.to_bits());
                if !column_seen.contains(&x_key) {
                    column_seen.insert(x_key);
                    column_order.push(x_key);
                }
                columns.entry(x_key).or_default().push(*rect);
            }

            for x_key in column_order {
                let mut rects = columns.get(&x_key).unwrap().clone();
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

        let filters: Vec<super::FilterPrimitive> = self
            .filters
            .iter()
            .filter(|f| viewport.intersects(&f.rect))
            .cloned()
            .collect();

        let transforms: Vec<super::TransformPrimitive> = self
            .transforms
            .iter()
            .filter(|t| viewport.intersects(&t.rect))
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
            filters,
            blend_modes: self.blend_modes.clone(), // blend_modes 保留
            transforms,
            // cull 重建后 typed Vec 索引已变（剔除丢弃元素），draw_order 失效故清空。
            // draw_order 仅在原始 paint 路径有意义；cull 是优化旁路。
            draw_order: Vec::new(),
        };

        let culled_count = original_len - result.len();
        let mut stats = result.stats();
        stats.culled_count = culled_count;

        (result, stats)
    }
}
