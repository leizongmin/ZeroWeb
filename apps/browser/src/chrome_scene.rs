// 浏览器 UI 场景图元包（DC-14 替换式迁移：chrome / 页面内容 / chrome 浮层 分离）。
//
// 经 `include!` 并入 `app.rs` 同一模块，故下方类型（`FillPrimitive` 等）直接沿用 `app.rs`
// 的 `use` 作用域，无需重复 import。本文件非独立模块起点，故用 `//` 注释（非 `//!`）。
//
// 历史上是单 `fills`/`glyphs`（chrome + 页面内容 + 自动补全/链接浮层混在一起）。DC-14 替换式
// 迁移需要把**页面内容**与 chrome 分离，使 feature-on 时可用 SDK chrome（`render_chrome_via_sdk`）
// 替换 chrome 主层、保留页面内容。故拆为三层：
// - `chrome_fills`/`chrome_glyphs`：chrome 主层（工具栏/标签/书签/页面框架等，绘制于页面之前）。
// - `page_fills`/`page_glyphs`：**页面内容**（`render_page_content`）。
// - `chrome_overlay_fills`/`chrome_overlay_glyphs`：chrome 浮层（自动补全/链接状态栏，覆盖于页面之上）。
// - `overlay_*`：顶层 overlay（滚动条/装饰/查找栏/下载/上下文菜单/缩放），最后绘制。
// - `chrome_shadows`：壳层阴影；`overlay_rounded_rects`：overlay 圆角。
//
// feature-off 按主层→页面→chrome 浮层顺序拼接（`combined_fills` / `combined_glyphs`），
// 与历史单 Vec 逐位等价（bit-identical）。

/// 浏览器 UI 场景图元包（三层分离，见文件头注释）。
pub(crate) struct ChromeScene {
    pub chrome_fills: Vec<FillPrimitive>,
    pub chrome_glyphs: Vec<GlyphDraw>,
    pub page_fills: Vec<FillPrimitive>,
    pub page_glyphs: Vec<GlyphDraw>,
    pub chrome_overlay_fills: Vec<FillPrimitive>,
    pub chrome_overlay_glyphs: Vec<GlyphDraw>,
    pub overlay_fills: Vec<FillPrimitive>,
    pub overlay_glyphs: Vec<GlyphDraw>,
    pub chrome_shadows: Vec<ShadowPrimitive>,
    pub overlay_rounded_rects: Vec<RoundedRectPrimitive>,
}

impl ChromeScene {
    /// feature-off：按 chrome 主层 → 页面内容 → chrome 浮层顺序拼接 fills（与历史单 Vec 逐位等价）。
    ///
    /// feature-on（替换式迁移，后续 GUI 环境）改为 `[SDK_chrome_fills, page_fills, chrome_overlay_fills]`。
    pub fn combined_fills(&self) -> Vec<FillPrimitive> {
        let mut v = self.chrome_fills.clone();
        v.extend_from_slice(&self.page_fills);
        v.extend_from_slice(&self.chrome_overlay_fills);
        v
    }

    /// 见 [`ChromeScene::combined_fills`]，glyphs 版本。
    pub fn combined_glyphs(&self) -> Vec<GlyphDraw> {
        let mut v = self.chrome_glyphs.clone();
        v.extend_from_slice(&self.page_glyphs);
        v.extend_from_slice(&self.chrome_overlay_glyphs);
        v
    }
}

/// 把 fills 整体沿 Y 轴平移 `dy`（DC-14 替换式迁移：页面内容/chrome 浮层从手绘 chrome 视口
/// 位置迁到 SDK chrome 视口位置）。颜色与 X 不变。
pub fn translate_fills(fills: &[FillPrimitive], dy: f32) -> Vec<FillPrimitive> {
    fills
        .iter()
        .map(|f| FillPrimitive {
            rect: Rect::new(f.rect.origin.x, f.rect.origin.y + dy, f.rect.size.width, f.rect.size.height),
            color: f.color,
        })
        .collect()
}

/// 把 glyphs 整体沿 Y 轴平移 `dy`（与 [`translate_fills`] 配套，保持页面文本与 fill 对齐）。
pub fn translate_glyphs(glyphs: &[GlyphDraw], dy: f32) -> Vec<GlyphDraw> {
    glyphs.iter().map(|g| GlyphDraw { baseline_y: g.baseline_y + dy, ..*g }).collect()
}

#[cfg(test)]
mod chrome_scene_tests {
    use super::*;

    fn fill(x: f32, y: f32) -> FillPrimitive {
        FillPrimitive {
            rect: Rect::new(x, y, 1.0, 1.0),
            color: Color::rgb(25, 51, 76),
        }
    }

    fn empty_scene() -> ChromeScene {
        ChromeScene {
            chrome_fills: vec![],
            chrome_glyphs: vec![],
            page_fills: vec![],
            page_glyphs: vec![],
            chrome_overlay_fills: vec![],
            chrome_overlay_glyphs: vec![],
            overlay_fills: vec![],
            overlay_glyphs: vec![],
            chrome_shadows: vec![],
            overlay_rounded_rects: vec![],
        }
    }

    #[test]
    fn combined_fills_preserves_layer_order() {
        // feature-off：combined = [chrome, page, chrome_overlay] 顺序（bit-identical 契约）。
        let mut scene = empty_scene();
        scene.chrome_fills = vec![fill(0.0, 0.0)];
        scene.page_fills = vec![fill(0.0, 1.0)];
        scene.chrome_overlay_fills = vec![fill(0.0, 2.0)];
        let combined = scene.combined_fills();
        assert_eq!(combined.len(), 3);
        assert_eq!(combined[0].rect.origin.y, 0.0, "chrome 主层先");
        assert_eq!(combined[1].rect.origin.y, 1.0, "page 居中");
        assert_eq!(combined[2].rect.origin.y, 2.0, "chrome 浮层最后");
    }

    #[test]
    fn translate_fills_shifts_y_preserves_x_and_color() {
        let fills = vec![fill(10.0, 100.0)];
        let moved = translate_fills(&fills, -24.0);
        assert_eq!(moved[0].rect.origin.x, 10.0, "x 不变");
        assert_eq!(moved[0].rect.origin.y, 76.0, "y 平移 dy");
        assert_eq!(moved[0].color, fills[0].color, "颜色不变");
    }

    #[test]
    fn translate_glyphs_shifts_baseline_preserves_x() {
        let g = GlyphDraw {
            ch: 'A',
            x: 5.0,
            baseline_y: 130.0,
            color: Color::rgb(0, 0, 0),
            font_id: 0,
            font_size: 16.0,
        };
        let moved = translate_glyphs(&[g], -30.0);
        assert_eq!(moved[0].x, 5.0, "x 不变");
        assert_eq!(moved[0].baseline_y, 100.0, "baseline_y 平移 dy");
        assert_eq!(moved[0].ch, 'A');
    }
}
