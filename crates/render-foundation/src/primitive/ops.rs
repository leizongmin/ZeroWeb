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
            dirty_rects: Vec::new(), // S3：由渲染管线（RenderPipeline）填充
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

        // draw_order 是默认渲染路径（R155，CPU render_draw_order）：按 paint_node
        // 的真实 DFS 顺序逐个渲染图元。本函数的颜色分组会把 fills 重排为
        // [所有同色 A, 所有同色 B, ...]，但 draw_order 仍指向**旧**索引 → 渲染时
        // Fill(i) 取到重排后的错误 fill，破坏 CSS painting order（例：position:relative
        // 的 #cover 本应绘于前置 in-flow flex items 之上，同色分组把它提前到其下
        // → flex-grow-003 red 可见）。draw_order 路径下颜色批无收益（render_draw_order
        // 逐个 fill_rect 无颜色状态批），故直接跳过；合并对不透明 fill 视觉中性，
        // 仅 render_typed_buckets 回退路径（draw_order 为空）保留 batching 优化。
        if !self.draw_order.is_empty() {
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
    pub fn cull_invisible(&mut self, viewport: Rect) -> RenderStats {
        let original_len = self.len();

        // 性能门禁优化 S7b（2026-08-08）：原位剔除——旧实现每类型 clone 全部存活
        // 图元（4400 元素页每帧 ~11k fills + 22k glyphs 拷贝）并新建 13 个 remap vec。
        // retain 在 Vec 内 memmove（无 clone）；glyphs/clips/blend_modes 本就不剔除，
        // 原位保留（旧实现也 clone 一份纯浪费）。

        // 对每个 typed Vec：保留满足视口相交条件的元素，同时记录「旧索引→新索引」
        // 重映射，供 draw_order 重建使用（draw_order 索引指向旧 typed Vec）。
        fn cull_vec<T, F: FnMut(&T) -> bool>(vec: &mut Vec<T>, keep: &mut F, remap: &mut [Option<usize>]) {
            let mut new_idx = 0;
            let mut i = 0;
            vec.retain(|item| {
                let k = keep(item);
                if k {
                    remap[i] = Some(new_idx);
                    new_idx += 1;
                }
                i += 1;
                k
            });
        }

        let mut fills_remap = vec![None; self.fills.len()];
        cull_vec(&mut self.fills, &mut |f| viewport.intersects(&f.rect), &mut fills_remap);

        let mut rounded_remap = vec![None; self.rounded_rects.len()];
        cull_vec(
            &mut self.rounded_rects,
            &mut |rr| viewport.intersects(&rr.rect),
            &mut rounded_remap,
        );

        let mut strokes_remap = vec![None; self.strokes.len()];
        cull_vec(
            &mut self.strokes,
            &mut |s| {
                let half_w = s.width / 2.0;
                let stroke_rect = Rect::new(
                    s.x1.min(s.x2) - half_w,
                    s.y1.min(s.y2) - half_w,
                    (s.x1.max(s.x2) - s.x1.min(s.x2)) + s.width,
                    (s.y1.max(s.y2) - s.y1.min(s.y2)) + s.width,
                );
                viewport.intersects(&stroke_rect)
            },
            &mut strokes_remap,
        );

        let mut shadows_remap = vec![None; self.shadows.len()];
        cull_vec(
            &mut self.shadows,
            &mut |s| {
                let shadow_rect = Rect::new(
                    s.rect.origin.x + s.offset_x - s.spread_radius - s.blur_radius,
                    s.rect.origin.y + s.offset_y - s.spread_radius - s.blur_radius,
                    s.rect.size.width + 2.0 * (s.spread_radius + s.blur_radius),
                    s.rect.size.height + 2.0 * (s.spread_radius + s.blur_radius),
                );
                viewport.intersects(&shadow_rect)
            },
            &mut shadows_remap,
        );

        let mut images_remap = vec![None; self.images.len()];
        cull_vec(
            &mut self.images,
            &mut |img| viewport.intersects(&img.rect),
            &mut images_remap,
        );

        let mut gradients_remap = vec![None; self.gradients.len()];
        cull_vec(
            &mut self.gradients,
            &mut |g| viewport.intersects(&g.rect),
            &mut gradients_remap,
        );

        let mut path_fills_remap = vec![None; self.path_fills.len()];
        cull_vec(
            &mut self.path_fills,
            &mut |pf| {
                if pf.vertices.is_empty() {
                    true
                } else {
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
                }
            },
            &mut path_fills_remap,
        );

        let mut path_strokes_remap = vec![None; self.path_strokes.len()];
        cull_vec(
            &mut self.path_strokes,
            &mut |ps| {
                if ps.vertices.is_empty() {
                    true
                } else {
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
                }
            },
            &mut path_strokes_remap,
        );

        let mut filters_remap = vec![None; self.filters.len()];
        cull_vec(
            &mut self.filters,
            &mut |f| viewport.intersects(&f.rect),
            &mut filters_remap,
        );

        let mut transforms_remap = vec![None; self.transforms.len()];
        cull_vec(
            &mut self.transforms,
            &mut |t| viewport.intersects(&t.rect),
            &mut transforms_remap,
        );

        // 重建 draw_order：把每个旧 DrawOp 索引按重映射更新到新 typed Vec 索引；
        // 被剔除的图元（remap=None）从 draw_order 中移除。clips/glyphs/blend_modes
        // 全保留（索引不变），其 DrawOp 直接保留。
        self.draw_order = self
            .draw_order
            .iter()
            .filter_map(|op| match op {
                DrawOp::Fill(i) => fills_remap.get(*i).copied().flatten().map(DrawOp::Fill),
                DrawOp::RoundedRect(i) => rounded_remap.get(*i).copied().flatten().map(DrawOp::RoundedRect),
                DrawOp::PathFill(i) => path_fills_remap.get(*i).copied().flatten().map(DrawOp::PathFill),
                DrawOp::PathStroke(i) => path_strokes_remap.get(*i).copied().flatten().map(DrawOp::PathStroke),
                DrawOp::Stroke(i) => strokes_remap.get(*i).copied().flatten().map(DrawOp::Stroke),
                DrawOp::Gradient(i) => gradients_remap.get(*i).copied().flatten().map(DrawOp::Gradient),
                DrawOp::Shadow(i) => shadows_remap.get(*i).copied().flatten().map(DrawOp::Shadow),
                DrawOp::Image(i) => images_remap.get(*i).copied().flatten().map(DrawOp::Image),
                DrawOp::Filter(i) => filters_remap.get(*i).copied().flatten().map(DrawOp::Filter),
                DrawOp::Transform(i) => transforms_remap.get(*i).copied().flatten().map(DrawOp::Transform),
                // clips/glyphs/blend_modes 全保留，索引不变
                DrawOp::Glyph(i) => Some(DrawOp::Glyph(*i)),
                DrawOp::BlendMode(i) => Some(DrawOp::BlendMode(*i)),
                DrawOp::Clip(i) => Some(DrawOp::Clip(*i)),
            })
            .collect();

        let culled_count = original_len - self.len();
        let mut stats = self.stats();
        stats.culled_count = culled_count;
        stats
    }
}
