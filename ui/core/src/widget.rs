//! Widget 基础接口与三棵树之「声明树」（spec IF-001 / FR-004）。
//!
//! `WidgetSpec` 是用户/Rust API/YAML DSL 产出的**声明**结构（可频繁重建）；
//! 实例状态落在 Element tree（`element.rs`），渲染输出落在 Render·Scene tree（`ui/render`）。
//!
//! M1 只定义 trait 与上下文边界；具体控件实现位于 `ui/widgets`/`ui/patterns`/`browser-ui/chrome`。

use crate::action::{ActionBinding, EventResult};
use crate::binding::{Binding, PropsMap};
use crate::event::UiEvent;
use crate::geometry::{Constraints, Point, Rect, Size, Vec2};
use crate::invalidation::InvalidationFlags;
use crate::semantics::SemanticsNode;
use crate::theme::Color;
use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// 稳定组件标识。WidgetSpec 重建时同 `WidgetId` 的组件在 Element tree 中保留状态（光标/选区/焦点）。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct WidgetId(pub CompactString);

impl WidgetId {
    pub fn new(name: &str) -> WidgetId {
        WidgetId(CompactString::new(name))
    }
}

/// 组件类型名（如 `Button`、`TextInput`、`browser.AddressBar`）。
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ComponentType(pub CompactString);

impl ComponentType {
    pub fn new(name: &str) -> ComponentType {
        ComponentType(CompactString::new(name))
    }
}

/// `Props` 类型别名（IF-001 `update(props: &Props)`）。
pub type Props = PropsMap;

/// 控制指令（spec IF-005 `ControlDirectives`）。
///
/// M1 以表达式**原文**承载（字符串）；M3 由 `ui/dsl` parser 解析为强类型 `Expression`。
/// 故 `ui/core` 不依赖 `ui/dsl`，依赖方向保持 dsl → core。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ControlDirectives {
    pub visible_when: Option<CompactString>,
    pub enabled_when: Option<CompactString>,
    pub for_each: Option<ForEachSpec>,
}

/// `for_each` 迭代声明。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForEachSpec {
    /// 数据源（状态路径/表达式原文）。
    pub source: CompactString,
    /// 迭代变量别名（默认 `item`）。
    pub item_alias: CompactString,
}

/// 声明树节点（spec §8.4.2 / IF-005 `WidgetSpec`）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WidgetSpec {
    pub component: ComponentType,
    pub id: Option<WidgetId>,
    pub props: PropsMap,
    pub bindings: Vec<Binding>,
    pub actions: Vec<ActionBinding>,
    pub control: ControlDirectives,
    pub children: Vec<WidgetSpec>,
}

impl WidgetSpec {
    pub fn new(component: &str) -> WidgetSpec {
        WidgetSpec {
            component: ComponentType::new(component),
            id: None,
            props: PropsMap::new(),
            bindings: Vec::new(),
            actions: Vec::new(),
            control: ControlDirectives::default(),
            children: Vec::new(),
        }
    }
}

/// 绘制后端抽象（M1 最小契约）。`ui/render` 提供具体实现，把记录转成 RenderPrimitives。
pub trait PaintRecorder {
    fn fill_rect(&mut self, rect: Rect, color: Color);
    /// 填充圆角矩形（`corner_radius` 同时应用到四角，逻辑像素）。
    ///
    /// 用于 chrome 控件的圆角背景：按钮 hover/pressed 圆盘（半径 = 半边长）、地址栏圆角 pill、
    /// 圆角标签等。默认实现忽略圆角委托 [`fill_rect`](Self::fill_rect)（测试 mock 沿用）；
    /// 真实场景由 `SceneRecorder` 覆写产出带 `Rounding` 的 `FillRect` 图元，经 `paint_scene`
    /// → 后端 `fill_rect(rect, color, rounding)` → `RoundedRectPrimitive`。
    fn fill_rounded_rect(&mut self, rect: Rect, corner_radius: f32, color: Color) {
        let _ = corner_radius;
        self.fill_rect(rect, color);
    }
    fn stroke_rect(&mut self, rect: Rect, color: Color, stroke_width: f32);
    /// 绘制原始字符串文本（后端负责 shape/measure/raster；简单场景与测试用）。
    ///
    /// 生产文本路径应优先用预 shape 的 `TextBlob`（见 `ui/render::SceneRecorder::draw_text_blob`），
    /// 避免每帧重复 shaping；本方法供 label/调试文本等不需精确度量的场景。
    fn draw_text(&mut self, text: &str, position: Point, size_px: f32, color: Color);

    /// 量取文本以 `size_px` 渲染时的视觉宽度（逻辑像素，DC-17 SourceCode 真实 metrics）。
    ///
    /// 默认实现按字符 Unicode 属性估算（向后兼容没接入字体后端的实现）：
    /// - ASCII 字母/数字（12px Noto Sans）：~6.6px
    /// - 空格：~3.3px
    /// - ASCII 标点：~3.5–7.0px（按字符查表）
    /// - CJK / 全角符号：~12.0px
    ///
    /// 生产实现（如 RenderFoundationBackend）应覆写为 fontdue 的真实 advance 累加。
    fn measure_text(&mut self, text: &str, size_px: f32) -> f32 {
        let scale = size_px / 12.0;
        let mut w = 0.0_f32;
        for c in text.chars() {
            w += char_width_estimate(c) * scale;
        }
        w
    }

    /// 记录外部合成表面（DC-3：WebView/平台视图/视频纹理）。
    ///
    /// UI SDK 只算外部矩形；真实纹理/primitives 由后端按 `surface_id` 取回合成。
    /// 本方法不引入浏览器类型 → `PaintRecorder` 保持浏览器无关（DC-1）。
    fn draw_external_surface(&mut self, rect: Rect, surface_id: u64);

    /// 绘制预注册图像（如 SVG 图标）到 `rect`，按 `tint` 着色。
    ///
    /// `image_ref` 引用一张由宿主/桥接预注册的图像（通常是单通道 alpha 掩码，如浏览器把
    /// SVG 图标经 resvg 光栅后注册到桥接 `ImageCache`）；`tint` 为着色（典型 = 主题前景
    /// semantic token）。后端负责按 `image_ref` 取回位图、按 `tint` 着色、缩放到 `rect` 光栅。
    /// 未注册的 ref 安静跳过（不 panic）。
    ///
    /// 与 glyph 文本路径对称：glyph = 字体内 alpha 掩码 + 文本色；本方法把「任意宿主提供的
    /// alpha 掩码」（图标 / 自定义符号）以同样 tint 模型暴露给控件。`ui/render` 不依赖
    /// render-foundation（DC-1）——本方法只携带 SDK 层 [`ImageRef`](crate::image::ImageRef)。
    fn draw_image(&mut self, rect: Rect, image_ref: crate::image::ImageRef, tint: Color);
}

/// paint 上下文。
pub struct PaintCtx<'a> {
    pub recorder: &'a mut dyn PaintRecorder,
    pub clip: Option<Rect>,
    pub offset: Vec2,
    /// 当前主题的 semantic token（DC-5：组件消费 token，不硬编码色值）。
    ///
    /// 由 `WidgetHost` 持有当前主题 token，paint 时注入；控件据此派生交互态色
    /// （如按钮 default=`primary`、hover=`primary.lighten(..)`），无需硬编码浏览器色值。
    pub tokens: &'a crate::theme::SemanticTokens,
    /// 可供 widgets 查询的实时字体度量 `(ascent, descent)`（DC-11 text path 统一）。
    ///
    /// 由 `WidgetHost` 从关联的 `FontdueBackend` 查询后注入。`None` 时
    /// [`line_metrics`](Self::line_metrics) 回落回 heuristic 默认值。
    pub font_metrics: Option<(f32, f32)>,
}

impl<'a> PaintCtx<'a> {
    /// 返回默认字体度量 `(ascent, descent)`（DC-11 text path 统一）。
    ///
    /// 若 [`font_metrics`](Self::font_metrics) 域被 host 注入真实字体后端度量
    /// （`FontdueBackend::line_metrics`），则优先使用；否则回落 heuristic 近似值。
    ///
    /// `ascent` 为正值（基线上方高度），`descent` 为负值（基线下方深度）。
    /// 控件应据此计算文本基线，与手绘 chrome `ui_text_centered_in_height` 一致：
    /// ```ignore
    /// let line_h = ascent - descent;
    /// let text_top = (box_h - line_h) / 2.0;
    /// let baseline = text_top + ascent;
    /// ```
    /// `font_metrics` 存储为归一化比率（per-px），需乘以 `font_size` 得到实际大小。
    /// 回落 heuristic：`(font_size * 0.92, -(font_size * 0.23))` ≈ 典型 UI 字体。
    pub fn line_metrics(&self, font_size: f32) -> (f32, f32) {
        match self.font_metrics {
            Some((ascent_ratio, descent_ratio)) => (ascent_ratio * font_size, descent_ratio * font_size),
            None => (font_size * 0.92, -(font_size * 0.23)),
        }
    }
}

/// mount 上下文（首次实例化）。
pub struct MountCtx<'a> {
    pub id: &'a WidgetId,
    pub invalidation: &'a mut InvalidationFlags,
}

/// update 上下文（props 变化）。
pub struct UpdateCtx<'a> {
    pub invalidation: &'a mut InvalidationFlags,
}

/// event 上下文。
pub struct EventCtx<'a> {
    pub invalidation: &'a mut InvalidationFlags,
}

/// layout 上下文。
pub struct LayoutCtx {
    pub scale_factor: f32,
}

/// semantics 上下文（a11y 树构建器）。
pub struct SemanticsCtx<'a> {
    pub nodes: &'a mut Vec<SemanticsNode>,
}

/// Widget 基础 trait（spec IF-001）。
///
/// 控件不应 panic；无效 props 在 update 阶段转为诊断；未处理事件返回 `EventResult::Ignored`。
pub trait Widget {
    fn mount(&mut self, ctx: &mut MountCtx);
    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props);
    fn event(&mut self, ctx: &mut EventCtx, event: &UiEvent) -> EventResult;
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: Constraints) -> Size;
    fn paint(&mut self, ctx: &mut PaintCtx);
    fn semantics(&self, ctx: &mut SemanticsCtx);
    /// 是否参与焦点遍历（Tab）与接收键盘事件（spec FR-011 / DC-8）。默认 `false`。
    fn focusable(&self) -> bool {
        false
    }

    /// 焦点时的 IME 输入矩形（局部坐标，原点 = 节点左上角；spec FR-011 / DC-8）。
    ///
    /// 文本控件返回 caret 所在行高矩形，供宿主换算为绝对坐标交给平台 IME / 软键盘定位。
    /// 非文本控件返回 `None`（默认）。
    fn ime_rect(&self) -> Option<Rect> {
        None
    }
}

/// 单字符在 12px Noto Sans + CJK fallback 下的近似视觉宽度（DC-17 SourceCode 真实 metrics）。
///
/// 估算规则：
/// - 空格：3.3
/// - ASCII 字母/数字：6.6
/// - 常见窄标点（. , ; : ! '）：3.0–4.5
/// - 常见宽标点（{ } [ ] ( ) < > = + - * / 等）：6.0–7.5
/// - CJK / 全角符号（U+3000–U+9FFF、U+FF00–U+FFEF）：12.0
/// - 其它（控制字符等）：0
///
/// 误差通常 < 1.5px/字符，对代码块场景肉眼可接受。
fn char_width_estimate(c: char) -> f32 {
    if c.is_control() {
        return 0.0;
    }
    match c {
        ' ' => 3.3,
        '.' | ',' => 3.0,
        ';' | ':' | '\'' | '"' => 4.0,
        '!' | '?' => 4.5,
        '/' | '\\' => 5.0,
        '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>' => 5.5,
        '=' | '+' | '-' | '*' | '|' | '&' | '^' | '%' | '~' => 6.5,
        '_' | '@' | '#' | '$' => 7.0,
        _ if c.is_ascii() => 6.6,
        _ => {
            let cp = c as u32;
            // CJK 统一表意文字 + 全角符号 + 韩文 + 日文假名
            if (0x3000..=0x9FFF).contains(&cp)
                || (0xA000..=0xA4CF).contains(&cp)
                || (0xAC00..=0xD7AF).contains(&cp)
                || (0xFF00..=0xFFEF).contains(&cp)
            {
                12.0
            } else {
                7.0 // 其它非 ASCII 字符（如带音调拉丁字母、阿拉伯文）取保守估值
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::ActionId;
    use crate::binding::Value;

    #[test]
    fn widget_spec_roundtrip_and_children() {
        let mut root = WidgetSpec::new("Column");
        root.id = Some(WidgetId::new("root"));
        let mut btn = WidgetSpec::new("Button");
        btn.props.insert("label", Value::Text("OK".into()));
        btn.actions.push(ActionBinding {
            trigger: CompactString::new("click"),
            action: ActionId::new("app.confirm"),
            payload: None,
        });
        root.children.push(btn);

        assert_eq!(root.component, ComponentType::new("Column"));
        assert_eq!(root.children.len(), 1);
        assert_eq!(root.children[0].actions.len(), 1);
    }

    #[test]
    fn stable_widget_id_equality() {
        // 同名 WidgetId 视为同一组件实例（Element tree 状态保持的依据）。
        let a = WidgetId::new("address_bar");
        let b = WidgetId::new("address_bar");
        assert_eq!(a, b);
    }

    #[test]
    fn control_directives_default_none() {
        let spec = WidgetSpec::new("Row");
        assert!(spec.control.visible_when.is_none());
        assert!(spec.control.enabled_when.is_none());
        assert!(spec.control.for_each.is_none());
    }
}
