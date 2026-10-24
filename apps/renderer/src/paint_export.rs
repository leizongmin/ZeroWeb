//! 渲染进程：RenderPrimitives → IPC 绘制快照。

use std::collections::HashMap;
use std::sync::Arc;
use zero_css_parser::values::{
    AnimationDirectionValue, AnimationFillModeValue, TimingFunctionValue, TransformFunction,
};
use zero_engine::animation::compositor_export::{ExportedAnimationDeclaration, ExportedAnimationValue};
use zero_engine::{HitTestCache, HitTestLayoutSnapshot, node_id_to_u64};
use zero_engine::{extract_img_srcs, image_resource_key, resolve_document_url};
use zero_protocol::{
    AnimationDirection, AnimationEasing, AnimationFillMode, AnimationKeyframe, AnimationPropertyKind,
    AnimationPropertyValue, AnimationTiming, CompositorAnimationDeclaration, IpcAnimationScope, IpcBlendMode,
    IpcBlendModePrimitive, IpcClip, IpcColor, IpcDrawOp, IpcFill, IpcFilter, IpcFilterKind, IpcFontVariation, IpcGlyph,
    IpcGlyphSource, IpcGlyphTextRun, IpcGradient, IpcGradientColorSpace, IpcGradientInterpolation, IpcGradientKind,
    IpcGradientStop, IpcHitTestCache, IpcHitTestLayoutNode, IpcHitTestNodeMeta, IpcHueMethod, IpcImage,
    IpcImagePayload, IpcLineCap, IpcLineStyle, IpcPathFill, IpcPathStroke, IpcRect, IpcRoundedRect, IpcShadow,
    IpcStroke, IpcTextControlBoundary, IpcTransform, PaintSnapshotParams, TransformFunctionValue,
};
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::image_cache::{decode_data_uri, decode_image_bytes};
use zero_render_foundation::primitive::{
    BlendMode, BlendModePrimitive, ClipPrimitive, DrawOp, FilterKind, FilterPrimitive, GlyphSource, GradientColorSpace,
    GradientKind, GradientPrimitive, HueMethod, LineCap, LineStyle, PaintAnimationProperty, PaintAnimationScope,
    PathFillPrimitive, PathStrokePrimitive, RenderPrimitives, ShadowPrimitive, StrokePrimitive, TransformPrimitive,
};

fn rect_to_ipc(r: &Rect) -> IpcRect {
    IpcRect {
        x: r.origin.x,
        y: r.origin.y,
        width: r.size.width,
        height: r.size.height,
    }
}

fn color_to_ipc(c: Color) -> IpcColor {
    IpcColor {
        r: c.r,
        g: c.g,
        b: c.b,
        a: c.a,
    }
}

fn glyph_source_run_id(
    source: &GlyphSource,
    source_ids: &mut HashMap<usize, u64>,
    text_runs: &mut Vec<IpcGlyphTextRun>,
) -> u64 {
    let key = Arc::as_ptr(&source.text) as *const () as usize;
    if let Some(run_id) = source_ids.get(&key) {
        return *run_id;
    }
    let run_id = source_ids.len() as u64 + 1;
    source_ids.insert(key, run_id);
    text_runs.push(IpcGlyphTextRun {
        run_id,
        text: source.text.to_string(),
    });
    run_id
}

fn gradient_kind_to_ipc(k: &GradientKind) -> IpcGradientKind {
    match k {
        GradientKind::Linear { x0, y0, x1, y1 } => IpcGradientKind::Linear {
            x0: *x0,
            y0: *y0,
            x1: *x1,
            y1: *y1,
        },
        GradientKind::Radial {
            cx,
            cy,
            inner_radius,
            outer_radius,
        } => IpcGradientKind::Radial {
            cx: *cx,
            cy: *cy,
            inner_radius: *inner_radius,
            outer_radius: *outer_radius,
        },
        GradientKind::Conic { cx, cy, start_angle } => IpcGradientKind::Conic {
            cx: *cx,
            cy: *cy,
            start_angle: *start_angle,
        },
    }
}

fn line_cap_to_ipc(c: LineCap) -> IpcLineCap {
    match c {
        LineCap::Butt => IpcLineCap::Butt,
        LineCap::Round => IpcLineCap::Round,
        LineCap::Square => IpcLineCap::Square,
    }
}

fn line_style_to_ipc(s: LineStyle) -> IpcLineStyle {
    match s {
        LineStyle::Solid => IpcLineStyle::Solid,
        LineStyle::Dashed => IpcLineStyle::Dashed,
        LineStyle::Dotted => IpcLineStyle::Dotted,
    }
}

fn filter_kind_to_ipc(k: &FilterKind) -> IpcFilterKind {
    match k {
        FilterKind::Blur(v) => IpcFilterKind::Blur(*v),
        FilterKind::Brightness(v) => IpcFilterKind::Brightness(*v),
        FilterKind::Contrast(v) => IpcFilterKind::Contrast(*v),
        FilterKind::Grayscale(v) => IpcFilterKind::Grayscale(*v),
        FilterKind::HueRotate(v) => IpcFilterKind::HueRotate(*v),
        FilterKind::Invert(v) => IpcFilterKind::Invert(*v),
        FilterKind::Opacity(v) => IpcFilterKind::Opacity(*v),
        FilterKind::Saturate(v) => IpcFilterKind::Saturate(*v),
        FilterKind::Sepia(v) => IpcFilterKind::Sepia(*v),
        FilterKind::DropShadow(x, y, blur, color) => IpcFilterKind::DropShadow {
            offset_x: *x,
            offset_y: *y,
            blur: *blur,
            color: color_to_ipc(*color),
        },
    }
}

fn blend_mode_to_ipc(mode: BlendMode) -> IpcBlendMode {
    match mode {
        BlendMode::Normal => IpcBlendMode::Normal,
        BlendMode::Multiply => IpcBlendMode::Multiply,
        BlendMode::Screen => IpcBlendMode::Screen,
        BlendMode::Overlay => IpcBlendMode::Overlay,
        BlendMode::Darken => IpcBlendMode::Darken,
        BlendMode::Lighten => IpcBlendMode::Lighten,
        BlendMode::ColorDodge => IpcBlendMode::ColorDodge,
        BlendMode::ColorBurn => IpcBlendMode::ColorBurn,
        BlendMode::HardLight => IpcBlendMode::HardLight,
        BlendMode::SoftLight => IpcBlendMode::SoftLight,
        BlendMode::Difference => IpcBlendMode::Difference,
        BlendMode::Exclusion => IpcBlendMode::Exclusion,
        BlendMode::Hue => IpcBlendMode::Hue,
        BlendMode::Saturation => IpcBlendMode::Saturation,
        BlendMode::Color => IpcBlendMode::Color,
        BlendMode::Luminosity => IpcBlendMode::Luminosity,
    }
}

fn draw_op_to_ipc(op: DrawOp) -> IpcDrawOp {
    match op {
        DrawOp::Fill(i) => IpcDrawOp::Fill(i),
        DrawOp::RoundedRect(i) => IpcDrawOp::RoundedRect(i),
        DrawOp::Gradient(i) => IpcDrawOp::Gradient(i),
        DrawOp::Shadow(i) => IpcDrawOp::Shadow(i),
        DrawOp::Image(i) => IpcDrawOp::Image(i),
        DrawOp::Stroke(i) => IpcDrawOp::Stroke(i),
        DrawOp::PathFill(i) => IpcDrawOp::PathFill(i),
        DrawOp::PathStroke(i) => IpcDrawOp::PathStroke(i),
        DrawOp::Clip(i) => IpcDrawOp::Clip(i),
        DrawOp::Transform(i) => IpcDrawOp::Transform(i),
        DrawOp::Filter(i) => IpcDrawOp::Filter(i),
        DrawOp::BlendMode(i) => IpcDrawOp::BlendMode(i),
        DrawOp::Glyph(i) => IpcDrawOp::Glyph(i),
    }
}

/// 发布路径图片子资源两阶段取回回调（P14，2026-10-25）。
///
/// 旧契约 `FnMut(&str) -> Option<Vec<u8>>` 令调用方逐图阻塞等待网络往返——DOM 序
/// 每张图完整串行，同 origin 图片并发恒为 1（cnblogs 107 请求 106 串行的第一根因）。
/// 两阶段契约：调用方先把全部缺图请求**非阻塞**发起（`issue`），再按 DOM 序逐图
/// `resolve` 收取；同 origin 并发由 zero-net 调度器按每来源上限自然展开。
pub trait ImageBatchFetcher {
    /// 非阻塞发起一张图的取回；发起失败静默（对应 `resolve` 返回 `None`，图片按既有宽松降级跳过）。
    fn issue(&mut self, url: &str);
    /// 收取已发起 URL 的响应体；网络失败/非 2xx/未发起 → `None`。
    ///
    /// 调用方保证全部 `issue` 先于首个 `resolve`；实现方可在首个 `resolve` 统一收齐在途响应。
    fn resolve(&mut self, url: &str) -> Option<Vec<u8>>;
}

/// 无网络取回器：不发起任何请求，`resolve` 恒 `None`（无网络发布/stub 网络路径，行为同旧 no_fetch 闭包）。
pub struct NoImageFetch;

impl ImageBatchFetcher for NoImageFetch {
    fn issue(&mut self, _url: &str) {}
    fn resolve(&mut self, _url: &str) -> Option<Vec<u8>> {
        None
    }
}

/// 优先从已解码 `ImageCache` 取图，缺失时经 `fetch` 两阶段批量抓取。
///
/// `sent_keys`（性能门禁优化 S8，2026-08-08）：已发送过且 browser 端 ImageCache
/// 已存的 key 不再重传像素——DOM 变更后每次 publish 全量 clone 图片像素是
/// ViewPainted IPC 体积的大头。仅「成功取到数据」的 key 标记 sent（fetch 失败
/// 的不标，下次仍重试——负缓存由 fetch 层处理）。
///
/// R3254-M1：本函数**只读** `sent_keys`（判断是否重传），不再在构造时插入——
/// sent 标记移到**实际写出之后**：compositor 异步路径由发布线程写出成功回传
/// （`CompositorPublishThread::drain_sent_image_keys`），legacy 同步路径由
/// `publish_render_with_layout` 同步发送成功后返回。否则 latest-wins 邮箱丢弃
/// 的帧（携带某图片首个 payload 且从未发出）会留下「已发送」假象，图片永久缺失。
pub fn fetch_image_payloads_with_cache<F: ImageBatchFetcher>(
    html: &str,
    page_url: &str,
    cache: &mut zero_render_foundation::image_cache::ImageCache,
    fetch: &mut F,
    sent_keys: &std::collections::HashSet<u64>,
) -> Vec<IpcImagePayload> {
    use zero_render_foundation::image_cache::ImageKey;

    let sources = extract_img_srcs(html);

    // P14：PASS 1 先把全部缺图请求非阻塞发起——本循环内不得发生任何阻塞等待，
    // 同 origin 并发由 zero-net 调度器展开（旧逐图闭包契约在此退化为
    // max-in-flight=1，cnblogs 107 请求 106 串行的第一根因）。
    let mut issued = std::collections::HashSet::new();
    for src in &sources {
        let resolved = resolve_document_url(page_url, src);
        let key = image_resource_key(&resolved, None);
        if !issued.insert(key) {
            continue;
        }
        if cache.get(&ImageKey::new(key)).is_some() || src.starts_with("data:") {
            continue;
        }
        fetch.issue(&resolved);
    }

    // PASS 2：DOM 序逐图收取/解码/组 payload（语义与旧逐图路径一致）。
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for src in &sources {
        let resolved = resolve_document_url(page_url, src);
        let key = image_resource_key(&resolved, None);
        if !seen.insert(key) {
            continue;
        }
        let data = if let Some(img) = cache.get(&ImageKey::new(key)) {
            img.clone()
        } else if src.starts_with("data:") {
            // R1705：data URI 自包含（无文件系统/网络），inline 解码（不经 fetch 回调）。
            match decode_data_uri(&resolved) {
                Ok(data) => data,
                Err(_) => continue,
            }
        } else if let Some(body) = fetch.resolve(&resolved) {
            match decode_image_bytes(&body) {
                Ok(data) => data,
                Err(_) => continue,
            }
        } else {
            continue;
        };
        if sent_keys.contains(&key) {
            // S8：browser 端 ImageCache 已有该 key 像素，不重传
            continue;
        }
        out.push(IpcImagePayload {
            image_key: key,
            width: data.width,
            height: data.height,
            rgba: data.pixels,
        });
    }
    out
}

/// 按像素字节上限把图片 payload 贪心切批，用于超出单帧上限的全页图片传输。
///
/// 返回 `(batches, skipped_keys)`：`batches` 内每批像素字节总量 ≤
/// `max_batch_bytes`（顺序保持）；单个 payload 超过上限时整图跳过并记入
/// `skipped_keys`——key 不进任何 sent 集合，下帧 publish 重试仍跳过。与字体
/// payload 超限行为一致（`MAX_PAINT_FONT_BYTES` 超限即丢）。
pub fn split_image_payload_batches(
    payloads: Vec<IpcImagePayload>,
    max_batch_bytes: usize,
) -> (Vec<Vec<IpcImagePayload>>, Vec<u64>) {
    let mut batches: Vec<Vec<IpcImagePayload>> = Vec::new();
    let mut skipped_keys = Vec::new();
    let mut current: Vec<IpcImagePayload> = Vec::new();
    let mut current_bytes = 0usize;
    for payload in payloads {
        let bytes = payload.rgba.len();
        if bytes > max_batch_bytes {
            tracing::warn!(
                "跳过超大图片 payload key={} bytes={bytes}（单批上限 {max_batch_bytes}）",
                payload.image_key
            );
            // FIXME(p100-image-batch): 单 payload 超批上限整图丢弃（该图空白）；
            // 后续可引入单图分块传输或下采样。
            skipped_keys.push(payload.image_key);
            continue;
        }
        if !current.is_empty() && current_bytes + bytes > max_batch_bytes {
            batches.push(std::mem::take(&mut current));
            current_bytes = 0;
        }
        current_bytes += bytes;
        current.push(payload);
    }
    if !current.is_empty() {
        batches.push(current);
    }
    (batches, skipped_keys)
}

// ── M2-S1b：合成动画导出换算（engine 域 → protocol 域） ─────────────────

/// engine timing → protocol easing。engine `Steps(n, pos)` 的 pos 在应用侧本就不
/// 参与进度映射（`apply_timing_function` 只取 n），协议侧同语义折叠为 `Steps(n)`。
fn animation_easing_to_ipc(value: &TimingFunctionValue) -> AnimationEasing {
    match value {
        TimingFunctionValue::Ease => AnimationEasing::Ease,
        TimingFunctionValue::Linear => AnimationEasing::Linear,
        TimingFunctionValue::EaseIn => AnimationEasing::EaseIn,
        TimingFunctionValue::EaseOut => AnimationEasing::EaseOut,
        TimingFunctionValue::EaseInOut => AnimationEasing::EaseInOut,
        TimingFunctionValue::CubicBezier(x1, y1, x2, y2) => AnimationEasing::CubicBezier(*x1, *y1, *x2, *y2),
        TimingFunctionValue::StepStart => AnimationEasing::StepStart,
        TimingFunctionValue::StepEnd => AnimationEasing::StepEnd,
        TimingFunctionValue::Steps(n, _) => AnimationEasing::Steps((*n).max(1) as u32),
    }
}

fn animation_direction_to_ipc(value: &AnimationDirectionValue) -> AnimationDirection {
    match value {
        AnimationDirectionValue::Normal => AnimationDirection::Normal,
        AnimationDirectionValue::Reverse => AnimationDirection::Reverse,
        AnimationDirectionValue::Alternate => AnimationDirection::Alternate,
        AnimationDirectionValue::AlternateReverse => AnimationDirection::AlternateReverse,
    }
}

fn animation_fill_to_ipc(value: &AnimationFillModeValue) -> AnimationFillMode {
    match value {
        AnimationFillModeValue::None => AnimationFillMode::None,
        AnimationFillModeValue::Forwards => AnimationFillMode::Forwards,
        AnimationFillModeValue::Backwards => AnimationFillMode::Backwards,
        AnimationFillModeValue::Both => AnimationFillMode::Both,
    }
}

/// engine transform 函数 → protocol 采样域值。导出分类（规则 2）已限定 2D px/deg
/// 子集，此处遇子集外变体返回 `None`（调用方整条丢弃——防御性，正常不可达）。
fn transform_function_to_ipc(value: &TransformFunction) -> Option<TransformFunctionValue> {
    Some(match value {
        TransformFunction::Translate(tx, ty) => TransformFunctionValue::Translate(*tx, *ty),
        TransformFunction::TranslateX(x) => TransformFunctionValue::TranslateX(*x),
        TransformFunction::TranslateY(y) => TransformFunctionValue::TranslateY(*y),
        TransformFunction::Rotate(a) | TransformFunction::RotateZ(a) => TransformFunctionValue::Rotate(*a),
        TransformFunction::Scale(sx, Some(sy)) => TransformFunctionValue::Scale(*sx, *sy),
        TransformFunction::Scale(sx, None) => TransformFunctionValue::Scale(*sx, *sx),
        TransformFunction::ScaleX(x) => TransformFunctionValue::ScaleX(*x),
        TransformFunction::ScaleY(y) => TransformFunctionValue::ScaleY(*y),
        TransformFunction::Skew(ax, Some(ay)) => TransformFunctionValue::Skew(*ax, *ay),
        TransformFunction::Skew(ax, None) => TransformFunctionValue::Skew(*ax, 0.0),
        _ => return None,
    })
}

fn animation_property_to_ipc(value: PaintAnimationProperty) -> AnimationPropertyKind {
    match value {
        PaintAnimationProperty::Opacity => AnimationPropertyKind::Opacity,
        PaintAnimationProperty::Transform => AnimationPropertyKind::Transform,
    }
}

/// engine paint 作用域 → protocol 作用域（字段一一对应；rect 展开 `[x,y,w,h]`）。
fn animation_scope_to_ipc(scope: &PaintAnimationScope) -> IpcAnimationScope {
    IpcAnimationScope {
        animation_id: scope.animation_id,
        property: animation_property_to_ipc(scope.property),
        draw_begin: scope.draw_begin,
        draw_end: scope.draw_end,
        rect: [
            scope.rect.origin.x,
            scope.rect.origin.y,
            scope.rect.size.width,
            scope.rect.size.height,
        ],
        transform_index: scope.transform_index,
    }
}

/// DC-2 offscreen 声明级门控（后台资源生命周期 M3）：按本帧存活作用域过滤声明——
/// 视口剔除后无采样面（`cull_invisible_with_scopes` 丢弃区间端点/transform 槽被剔的
/// 作用域）的动画不进 compositor 状态，即「声明活跃 = 存在可见采样面」。全部被剔
/// （视口外动画页）即空集 → compositor `animations_active` 不置位/回落，采样重光栅
/// 与 keep-alive 呈现链不对不可见动画空转（M3 证据 offscreen 腿 0.5 cores）；元素
/// 回视口后下一帧作用域重现，声明经集合差分恢复（advance 锚点「消失剪除/重现重新
/// 开始」语义）。filter 在 `advance` 输入侧——差分/锚点/发送闸门都按可见集运转。
///
/// M4 GUI 腿补 `page_hidden` 维度（同语义延伸）：tab 隐藏页无采样面——声明清空
/// 使 `animations_active` 回落，browser keep-alive 停对不可见 surface 的常态 Get
/// 重光栅（GUI 判别腿实测 hidden tab compositor 0.59 cores 空转）；恢复可见后
/// 首帧集合差分重现。gCS 覆盖缺失/终结事件丢失/时间轴重启边界与 offscreen 维度
/// 一致。
pub(crate) fn visible_declarations(
    declarations: &[ExportedAnimationDeclaration],
    scopes: &[PaintAnimationScope],
    page_hidden: bool,
) -> Vec<ExportedAnimationDeclaration> {
    if page_hidden {
        return Vec::new();
    }
    let visible_ids: std::collections::HashSet<u64> = scopes.iter().map(|scope| scope.animation_id).collect();
    declarations
        .iter()
        .filter(|declaration| visible_ids.contains(&declaration.animation_id))
        .cloned()
        .collect()
}

/// engine 声明 → protocol 声明（时间单位秒 → 毫秒；`start_monotonic_ns` = 该动画
/// 首次导出的共享单调钟锚点 + engine 时钟域 start——S1b 导出域 `start_time_s` 恒
/// 0.0，公式保留通用形供后续切片复用）。键帧端点含子集外 transform 变体或锚点
/// 缺失时返回 `None`，调用方整条丢弃（不发送部分声明）。
pub fn animation_declaration_to_ipc(
    declaration: &ExportedAnimationDeclaration,
    anchors: &HashMap<u64, u64>,
) -> Option<CompositorAnimationDeclaration> {
    // 属性通道取首帧值变体（导出规则 1 保证端点均质，不会出现混合通道声明）。
    let property = match declaration.keyframes.first().map(|kf| &kf.value) {
        Some(ExportedAnimationValue::Opacity(_)) => AnimationPropertyKind::Opacity,
        Some(ExportedAnimationValue::Transform(_)) => AnimationPropertyKind::Transform,
        None => return None,
    };
    let keyframes = declaration
        .keyframes
        .iter()
        .map(|kf| -> Option<AnimationKeyframe> {
            let value = match &kf.value {
                ExportedAnimationValue::Opacity(o) => AnimationPropertyValue::Opacity(*o),
                ExportedAnimationValue::Transform(fns) => {
                    let mut converted = Vec::with_capacity(fns.len());
                    for f in fns {
                        converted.push(transform_function_to_ipc(f)?);
                    }
                    AnimationPropertyValue::Transform(converted)
                }
            };
            Some(AnimationKeyframe {
                offset: kf.offset,
                value,
                easing: kf.easing.as_ref().map(animation_easing_to_ipc),
            })
        })
        .collect::<Option<Vec<_>>>()?;
    let anchor = anchors.get(&declaration.animation_id).copied()?;
    Some(CompositorAnimationDeclaration {
        animation_id: declaration.animation_id,
        // 导出分类已剔除不可采样成分，到达此处的声明全可合成（协议字段保留给
        // 未来放宽导出面时的降级标记）。
        composable: true,
        start_monotonic_ns: anchor.saturating_add((declaration.start_time_s * 1e9) as u64),
        timing: AnimationTiming {
            duration_ms: declaration.duration_s * 1000.0,
            delay_ms: declaration.delay_s * 1000.0,
            iterations: declaration.iterations,
            direction: animation_direction_to_ipc(&declaration.direction),
            fill: animation_fill_to_ipc(&declaration.fill),
            easing: animation_easing_to_ipc(&declaration.easing),
        },
        keyframes,
        property,
        // M2-S3b：冻结相位由 `advance` 暂停态机在换算后附着（runtime
        // `AnimationExportState`，paused 翻转驱动），纯换算恒 None。
        frozen_phase_s: None,
    })
}

/// 从渲染图元构建 IPC 绘制快照。
#[allow(clippy::too_many_arguments)] // 构造器：图元快照字段多，逐参数透传 PaintSnapshotParams
pub fn paint_snapshot_from_primitives(
    viewport_width: u32,
    viewport_height: u32,
    device_scale_factor: f32,
    document_height: f32,
    primitives: &RenderPrimitives,
    dirty_rects: &[(f32, f32, f32, f32)],
    image_payloads: Vec<IpcImagePayload>,
    hit_test: Option<HitTestCache>,
    navigation_epoch: u64,
    document_generation: u64,
    animation_scopes: &[PaintAnimationScope],
) -> PaintSnapshotParams {
    let mut glyph_source_ids = HashMap::new();
    let mut glyph_text_runs = Vec::new();
    let glyphs = primitives
        .glyphs
        .iter()
        .map(|g| IpcGlyph {
            x: g.x,
            y: g.y,
            font_size: g.font_size,
            glyph_id: g.glyph_id,
            font_glyph_index: g.font_glyph_index,
            source: g.source.as_ref().map(|source| IpcGlyphSource {
                run_id: glyph_source_run_id(source, &mut glyph_source_ids, &mut glyph_text_runs),
                start: source.start,
                end: source.end,
            }),
            font_id: g.font_id.0,
            font_variation_id: g.font_variation_id.map(|id| id.0),
            color: color_to_ipc(g.color),
            rotation: g.rotation,
            synthetic_italic: g.synthetic_italic,
        })
        .collect();
    PaintSnapshotParams {
        viewport_width,
        viewport_height,
        device_scale_factor,
        document_height,
        navigation_epoch,
        document_generation,
        fills: primitives
            .fills
            .iter()
            .map(|f| IpcFill {
                rect: rect_to_ipc(&f.rect),
                color: color_to_ipc(f.color),
            })
            .collect(),
        rounded_rects: primitives
            .rounded_rects
            .iter()
            .map(|rr| IpcRoundedRect {
                rect: rect_to_ipc(&rr.rect),
                color: color_to_ipc(rr.color),
                top_left_radius: rr.top_left_radius,
                top_right_radius: rr.top_right_radius,
                bottom_right_radius: rr.bottom_right_radius,
                bottom_left_radius: rr.bottom_left_radius,
            })
            .collect(),
        gradients: primitives
            .gradients
            .iter()
            .map(|g: &GradientPrimitive| IpcGradient {
                rect: rect_to_ipc(&g.rect),
                kind: gradient_kind_to_ipc(&g.kind),
                stops: g
                    .stops
                    .iter()
                    .map(|s| IpcGradientStop {
                        offset: s.offset,
                        color: color_to_ipc(s.color),
                    })
                    .collect(),
                repeating: g.repeating,
                interpolation: IpcGradientInterpolation {
                    space: match g.interpolation.space {
                        GradientColorSpace::Srgb => IpcGradientColorSpace::Srgb,
                        GradientColorSpace::SrgbLinear => IpcGradientColorSpace::SrgbLinear,
                        GradientColorSpace::Lab => IpcGradientColorSpace::Lab,
                        GradientColorSpace::Oklab => IpcGradientColorSpace::Oklab,
                        GradientColorSpace::Lch => IpcGradientColorSpace::Lch,
                        GradientColorSpace::Oklch => IpcGradientColorSpace::Oklch,
                        GradientColorSpace::Hsl => IpcGradientColorSpace::Hsl,
                        GradientColorSpace::Hwb => IpcGradientColorSpace::Hwb,
                        GradientColorSpace::Xyz => IpcGradientColorSpace::Xyz,
                        GradientColorSpace::XyzD50 => IpcGradientColorSpace::XyzD50,
                        GradientColorSpace::ProphotoRgb => IpcGradientColorSpace::ProphotoRgb,
                        GradientColorSpace::DisplayP3 => IpcGradientColorSpace::DisplayP3,
                        GradientColorSpace::DisplayP3Linear => IpcGradientColorSpace::DisplayP3Linear,
                        GradientColorSpace::A98Rgb => IpcGradientColorSpace::A98Rgb,
                        GradientColorSpace::Rec2020 => IpcGradientColorSpace::Rec2020,
                    },
                    hue: match g.interpolation.hue {
                        HueMethod::Shorter => IpcHueMethod::Shorter,
                        HueMethod::Longer => IpcHueMethod::Longer,
                        HueMethod::Increasing => IpcHueMethod::Increasing,
                        HueMethod::Decreasing => IpcHueMethod::Decreasing,
                    },
                },
                clip: g.clip.as_ref().map(rect_to_ipc),
            })
            .collect(),
        shadows: primitives
            .shadows
            .iter()
            .map(|s: &ShadowPrimitive| IpcShadow {
                rect: rect_to_ipc(&s.rect),
                color: color_to_ipc(s.color),
                offset_x: s.offset_x,
                offset_y: s.offset_y,
                blur_radius: s.blur_radius,
                spread_radius: s.spread_radius,
            })
            .collect(),
        images: primitives
            .images
            .iter()
            .map(|img| IpcImage {
                rect: rect_to_ipc(&img.rect),
                image_key: img.image_key.0,
                clip: img.clip.as_ref().map(rect_to_ipc),
            })
            .collect(),
        image_payloads,
        font_payloads: Vec::new(),
        strokes: primitives
            .strokes
            .iter()
            .map(|s: &StrokePrimitive| IpcStroke {
                x1: s.x1,
                y1: s.y1,
                x2: s.x2,
                y2: s.y2,
                width: s.width,
                color: color_to_ipc(s.color),
                style: line_style_to_ipc(s.style),
                cap: line_cap_to_ipc(s.cap),
            })
            .collect(),
        path_fills: primitives
            .path_fills
            .iter()
            .map(|pf: &PathFillPrimitive| IpcPathFill {
                vertices: pf.vertices.clone(),
                color: color_to_ipc(pf.color),
            })
            .collect(),
        path_strokes: primitives
            .path_strokes
            .iter()
            .map(|ps: &PathStrokePrimitive| IpcPathStroke {
                vertices: ps.vertices.clone(),
                color: color_to_ipc(ps.color),
                line_width: ps.line_width,
                closed: ps.closed,
            })
            .collect(),
        clips: primitives
            .clips
            .iter()
            .map(|c: &ClipPrimitive| IpcClip {
                rect: rect_to_ipc(&c.rect),
            })
            .collect(),
        transforms: primitives
            .transforms
            .iter()
            .map(|t: &TransformPrimitive| IpcTransform {
                rect: rect_to_ipc(&t.rect),
                origin_x: t.origin_x,
                origin_y: t.origin_y,
                a: t.a,
                b: t.b,
                c: t.c,
                d: t.d,
                tx: t.tx,
                ty: t.ty,
            })
            .collect(),
        filters: primitives
            .filters
            .iter()
            .map(|f: &FilterPrimitive| IpcFilter {
                rect: rect_to_ipc(&f.rect),
                filters: f.filters.iter().map(filter_kind_to_ipc).collect(),
            })
            .collect(),
        blend_modes: primitives
            .blend_modes
            .iter()
            .map(|b: &BlendModePrimitive| IpcBlendModePrimitive {
                rect: rect_to_ipc(&b.rect),
                mode: blend_mode_to_ipc(b.mode),
            })
            .collect(),
        glyph_text_runs,
        font_variations: primitives
            .font_variations
            .iter()
            .map(|variations| {
                variations
                    .iter()
                    .map(|variation| IpcFontVariation {
                        tag: variation.tag,
                        value: variation.value,
                    })
                    .collect()
            })
            .collect(),
        glyphs,
        draw_order: primitives.draw_order.iter().copied().map(draw_op_to_ipc).collect(),
        dirty_rects: dirty_rects
            .iter()
            .map(|(x, y, w, h)| IpcRect {
                x: *x,
                y: *y,
                width: *w,
                height: *h,
            })
            .collect(),
        hit_test: hit_test.map(hit_test_cache_to_ipc),
        text_control_boundaries: primitives
            .text_control_boundaries
            .iter()
            .map(|boundary| IpcTextControlBoundary {
                node_handle: boundary.node_handle,
                utf16_offset: boundary.utf16_offset,
                x: boundary.x,
                y: boundary.y,
                height: boundary.height,
            })
            .collect(),
        // M2-S1b：paint 动画作用域随帧发布（engine 域换算 protocol；flag 关恒空）。
        animation_scopes: animation_scopes.iter().map(animation_scope_to_ipc).collect(),
    }
}

fn hit_test_cache_to_ipc(cache: HitTestCache) -> IpcHitTestCache {
    let snap = cache.snapshot();
    IpcHitTestCache {
        doc_root: node_id_to_u64(snap.doc_root),
        layout_root: hit_test_layout_to_ipc(&snap.layout_root),
        nodes: snap
            .nodes
            .into_iter()
            .map(|(id, meta)| {
                (
                    node_id_to_u64(id),
                    IpcHitTestNodeMeta {
                        tag_name: meta.tag_name,
                        id: meta.id,
                        class_name: meta.class_name,
                        selector: meta.selector,
                        href: meta.href,
                        src: meta.src,
                        link_text: meta.link_text,
                        draggable: meta.draggable,
                        drag_barrier: meta.drag_barrier,
                    },
                )
            })
            .collect(),
        parents: snap
            .parents
            .into_iter()
            .map(|(child, parent)| (node_id_to_u64(child), node_id_to_u64(parent)))
            .collect(),
        hidden_nodes: snap.hidden_nodes,
        pe_none_nodes: snap.pe_none_nodes,
    }
}

fn hit_test_layout_to_ipc(node: &HitTestLayoutSnapshot) -> IpcHitTestLayoutNode {
    IpcHitTestLayoutNode {
        node_id: node.node_id.map(node_id_to_u64),
        x: node.x,
        y: node.y,
        width: node.width,
        height: node.height,
        reported: node.reported,
        children: node.children.iter().map(hit_test_layout_to_ipc).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use zero_render_foundation::primitive::{FontId, GlyphPrimitive};

    #[test]
    fn glyph_source_run_id_tracks_shared_text_allocation() {
        let text: Arc<str> = Arc::from("A\u{301}");
        let first = GlyphSource::new(text.clone(), 0, 3).expect("first cluster");
        let second = GlyphSource::new(text, 0, 3).expect("second cluster");
        let independent = GlyphSource::new(Arc::from("A\u{301}"), 0, 3).expect("independent cluster");
        let mut source_ids = HashMap::new();
        let mut text_runs = Vec::new();

        assert_eq!(
            glyph_source_run_id(&first, &mut source_ids, &mut text_runs),
            glyph_source_run_id(&second, &mut source_ids, &mut text_runs)
        );
        assert_ne!(
            glyph_source_run_id(&first, &mut source_ids, &mut text_runs),
            glyph_source_run_id(&independent, &mut source_ids, &mut text_runs)
        );
        assert_eq!(text_runs.len(), 2);
        assert_eq!(text_runs[0].text, "A\u{301}");
    }

    #[test]
    fn paint_snapshot_preserves_glyph_raster_metadata() {
        let mut primitives = RenderPrimitives::new();
        let font_variation_id =
            primitives.intern_font_variations(&[zero_render_foundation::font::OpenTypeVariation::new(*b"wdth", 125.0)]);
        primitives.add_glyph(GlyphPrimitive {
            x: 1.0,
            y: 2.0,
            font_size: 16.0,
            color: Color::BLACK,
            glyph_id: 'A' as u32,
            font_glyph_index: None,
            source: None,
            font_id: FontId(0),
            font_variation_id,
            bitmap_width: None,
            bitmap_height: None,
            rotation: 0.0,
            synthetic_italic: true,
        });

        let snapshot = paint_snapshot_from_primitives(10, 10, 1.0, 10.0, &primitives, &[], vec![], None, 1, 2, &[]);
        assert!(snapshot.glyphs[0].synthetic_italic);
        assert_eq!(snapshot.glyphs[0].font_variation_id, Some(0));
        assert_eq!(snapshot.font_variations[0][0].tag, *b"wdth");
        assert_eq!(snapshot.font_variations[0][0].value, 125.0);
        assert_eq!(snapshot.document_generation, 2);
    }

    fn p100_payload(key: u64, rgba_len: usize) -> IpcImagePayload {
        IpcImagePayload {
            image_key: key,
            width: 1,
            height: 1,
            rgba: vec![0u8; rgba_len],
        }
    }

    #[test]
    fn split_image_batches_empty_input_yields_nothing() {
        let (batches, skipped) = split_image_payload_batches(Vec::new(), 100);
        assert!(batches.is_empty());
        assert!(skipped.is_empty());
    }

    #[test]
    fn split_image_batches_greedy_fill_respects_cap_and_order() {
        // 40+40+40，上限 80：批 1 = [1,2]（80），批 2 = [3]（40）；顺序保持。
        let payloads = vec![p100_payload(1, 40), p100_payload(2, 40), p100_payload(3, 40)];
        let (batches, skipped) = split_image_payload_batches(payloads, 80);
        assert_eq!(skipped, Vec::<u64>::new());
        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].iter().map(|p| p.image_key).collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!(batches[1].iter().map(|p| p.image_key).collect::<Vec<_>>(), vec![3]);
        for batch in &batches {
            assert!(batch.iter().map(|p| p.rgba.len()).sum::<usize>() <= 80);
        }
    }

    #[test]
    fn split_image_batches_exact_cap_payload_gets_own_batch() {
        let payloads = vec![p100_payload(7, 80), p100_payload(8, 10)];
        let (batches, skipped) = split_image_payload_batches(payloads, 80);
        assert!(skipped.is_empty());
        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].len(), 1);
        assert_eq!(batches[0][0].image_key, 7);
        assert_eq!(batches[1][0].image_key, 8);
    }

    #[test]
    fn split_image_batches_oversize_payload_skipped_with_key() {
        let payloads = vec![p100_payload(1, 10), p100_payload(2, 200), p100_payload(3, 10)];
        let (batches, skipped) = split_image_payload_batches(payloads, 80);
        assert_eq!(skipped, vec![2]);
        // 跳过不中断切批：其余两个 payload 仍合为一批。
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), 2);
    }

    /// P100：生产者→消费者契约——切批输出的每个 payload 必须能经消费端解码路径
    /// （`ImageData::from_rgba` 要求 len == w*h*4）无损还原尺寸与像素。两侧字段
    /// 或尺寸语义漂移时，浏览器端只会看到 from_rgba Err（图片静默缺失），故在
    /// 生产者一侧钉住往返一致。
    #[test]
    fn split_output_payloads_roundtrip_through_consumer_decode() {
        let payloads = vec![
            IpcImagePayload {
                image_key: 1,
                width: 2,
                height: 3,
                rgba: (0..24).map(|i| i as u8).collect(),
            },
            IpcImagePayload {
                image_key: 2,
                width: 4,
                height: 1,
                rgba: vec![9; 16],
            },
            IpcImagePayload {
                image_key: 3,
                width: 1,
                height: 1,
                rgba: vec![0; 4],
            },
        ];
        let original: Vec<(u64, u32, u32, Vec<u8>)> = payloads
            .iter()
            .map(|p| (p.image_key, p.width, p.height, p.rgba.clone()))
            .collect();
        let (batches, skipped) = split_image_payload_batches(payloads, 24);
        assert!(skipped.is_empty());
        let round: Vec<&IpcImagePayload> = batches.iter().flatten().collect();
        assert_eq!(round.len(), original.len());
        for (payload, (key, width, height, rgba)) in round.iter().zip(&original) {
            assert_eq!(payload.image_key, *key);
            assert_eq!(payload.width, *width);
            assert_eq!(payload.height, *height);
            let data = zero_render_foundation::image_cache::ImageData::from_rgba(
                payload.rgba.clone(),
                payload.width,
                payload.height,
            )
            .expect("消费端解码必须接受生产者切批输出");
            assert_eq!(data.width, *width);
            assert_eq!(data.height, *height);
            assert_eq!(data.pixels, *rgba);
        }
    }

    // ── P14：两阶段批量取回契约 ─────────────────────────────────────────

    const SVG_BODY: &[u8] =
        b"<svg xmlns='http://www.w3.org/2000/svg' width='2' height='3'><rect width='2' height='3' fill='red'/></svg>";
    const DATA_URI: &str = "data:image/svg+xml;base64,PHN2ZyB4bWxucz0naHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmcnIHdpZHRoPScyJyBoZWlnaHQ9JzMnPjxyZWN0IHdpZHRoPScyJyBoZWlnaHQ9JzMnIGZpbGw9J3JlZCcvPjwvc3ZnPg==";
    const PAGE_URL: &str = "https://page.test/index.html";

    /// 测试桩：记录 issue/resolve 次序；resolve 返回预置表（None = 网络失败）。
    struct RecordingBatchFetcher {
        bodies: HashMap<String, Option<Vec<u8>>>,
        issued: Vec<String>,
        resolved: Vec<String>,
    }

    impl RecordingBatchFetcher {
        fn new(bodies: HashMap<String, Option<Vec<u8>>>) -> Self {
            Self {
                bodies,
                issued: Vec::new(),
                resolved: Vec::new(),
            }
        }
    }

    impl ImageBatchFetcher for RecordingBatchFetcher {
        fn issue(&mut self, url: &str) {
            self.issued.push(url.to_string());
        }
        fn resolve(&mut self, url: &str) -> Option<Vec<u8>> {
            self.resolved.push(url.to_string());
            self.bodies.get(url).cloned().flatten()
        }
    }

    fn key_of(src: &str) -> u64 {
        image_resource_key(&resolve_document_url(PAGE_URL, src), None)
    }

    fn fixture_html() -> String {
        format!(
            "<html><body>\
             <img src='https://pic.test/a.svg'>\
             <img src='https://pic.test/b.svg'>\
             <img src='https://pic.test/a.svg'>\
             <img src='https://cache.test/hit.svg'>\
             <img src='{DATA_URI}'>\
             <img src='https://pic.test/fail.svg'>\
             <img src='https://pic.test/missing.svg'>\
             </body></html>"
        )
    }

    /// P14 契约钉：全部缺图（cache miss 且非 data:，去重）先于任何 resolve 被
    /// 非阻塞 issue，且按 DOM 序；resolve 同样按 DOM 序且只问 issued 的 URL。
    #[test]
    fn batch_fetch_issues_all_missing_before_any_resolve() {
        let html = fixture_html();
        let a = "https://pic.test/a.svg".to_string();
        let b = "https://pic.test/b.svg".to_string();
        let fail = "https://pic.test/fail.svg".to_string();
        let missing = "https://pic.test/missing.svg".to_string();
        let mut bodies: HashMap<String, Option<Vec<u8>>> = HashMap::new();
        bodies.insert(a.clone(), Some(SVG_BODY.to_vec()));
        bodies.insert(b.clone(), Some(SVG_BODY.to_vec()));
        bodies.insert(fail.clone(), Some(b"not an image".to_vec()));
        bodies.insert(missing.clone(), None);
        let mut fetcher = RecordingBatchFetcher::new(bodies);

        let mut cache = zero_render_foundation::image_cache::ImageCache::new(64, 1 << 20);
        cache.insert_with_key(
            zero_render_foundation::image_cache::ImageKey::new(key_of("https://cache.test/hit.svg")),
            zero_render_foundation::image_cache::ImageData::from_rgba(vec![0, 0, 0, 255], 1, 1).expect("1x1 ImageData"),
        );
        let sent_keys = std::collections::HashSet::new();

        let payloads = fetch_image_payloads_with_cache(&html, PAGE_URL, &mut cache, &mut fetcher, &sent_keys);

        // issue：全部缺图、DOM 序、去重；cache hit 与 data: 不发起。
        assert_eq!(
            fetcher.issued,
            vec![a.clone(), b.clone(), fail.clone(), missing.clone()]
        );
        // resolve：DOM 序，且只问 issued 的 URL（cache hit/data: 不经 fetch）。
        assert_eq!(fetcher.resolved, vec![a, b, fail, missing]);
        // payload：成功图按 DOM 序（含 cache hit 与 data: 内联）；decode 失败/网络失败跳过。
        assert_eq!(
            payloads.iter().map(|p| p.image_key).collect::<Vec<_>>(),
            vec![
                key_of("https://pic.test/a.svg"),
                key_of("https://pic.test/b.svg"),
                key_of("https://cache.test/hit.svg"),
                key_of(DATA_URI),
            ]
        );
        // 解码成功图携带真实尺寸（2x3），证明确实走了 SVG 解码而非占位。
        assert_eq!((payloads[0].width, payloads[0].height), (2, 3));
    }

    /// S8 语义保持：sent_keys 命中的成功图不进 payload（含 cache hit 路径）。
    #[test]
    fn batch_fetch_sent_keys_filter_unchanged() {
        let html = "<html><body><img src='https://pic.test/a.svg'></body></html>";
        let a = "https://pic.test/a.svg".to_string();
        let mut bodies: HashMap<String, Option<Vec<u8>>> = HashMap::new();
        bodies.insert(a, Some(SVG_BODY.to_vec()));
        let mut fetcher = RecordingBatchFetcher::new(bodies);
        let mut cache = zero_render_foundation::image_cache::ImageCache::new(64, 1 << 20);
        let mut sent_keys = std::collections::HashSet::new();
        sent_keys.insert(key_of("https://pic.test/a.svg"));

        let payloads = fetch_image_payloads_with_cache(html, PAGE_URL, &mut cache, &mut fetcher, &sent_keys);
        assert!(payloads.is_empty());
        // sent 命中的图仍然发起了网络请求（取回后丢弃像素，语义同旧路径）。
        assert_eq!(fetcher.issued, vec!["https://pic.test/a.svg".to_string()]);
    }

    /// 无网络取回器：不发起、恒 None（stub/无网络发布路径行为同旧 no_fetch 闭包）。
    #[test]
    fn no_image_fetch_never_issues_and_resolves_none() {
        let html = "<html><body><img src='https://pic.test/a.svg'></body></html>";
        let mut fetcher = NoImageFetch;
        let mut cache = zero_render_foundation::image_cache::ImageCache::new(64, 1 << 20);
        let sent_keys = std::collections::HashSet::new();

        let payloads = fetch_image_payloads_with_cache(html, PAGE_URL, &mut cache, &mut fetcher, &sent_keys);
        assert!(payloads.is_empty());
    }
}
