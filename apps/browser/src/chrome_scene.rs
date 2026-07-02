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
