//! 图片对象缓存与 GC — 管理已解码图片的缓存和生命周期
//!
//! 提供：
//! - 基于引用计数的图片缓存
//! - LRU 风格的垃圾回收
//! - 图片数据存储（RGBA 像素数据）

use crate::geometry::Size;
use hashbrown::HashMap;
use std::sync::Arc;

/// 图片缓存键 — 唯一标识一张图片
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ImageKey(pub u64);

impl ImageKey {
    /// 创建新的图片键
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// 已解码的图片数据
#[derive(Debug, Clone)]
pub struct ImageData {
    /// RGBA 像素数据（行优先）
    pub pixels: Vec<u8>,
    /// 宽度（像素）
    pub width: u32,
    /// 高度（像素）
    pub height: u32,
    /// 像素内容摘要（R3254-M2：插入时计算一次，GPU 纹理缓存键复用——像素在插入
    /// ImageCache 后不可变，避免每帧对每张图全量哈希）。
    pub content_hash: u64,
    /// 纯色检测结果 — 当所有像素相同时缓存该颜色，用于优化渲染
    solid_color: Option<[u8; 4]>,
    /// 仅含宽高比、无确定固有尺寸的信号（CSS §10.3.2）。
    ///
    /// 仅 SVG 会出现：当 `<svg>` 的 width/height 为百分比或缺失（仅有 viewBox）时，
    /// 替换元素**无确定固有尺寸**，仅有 viewBox 给出的宽高比。此时栅格化仍按 usvg
    /// 解析尺寸产出像素（供绘制），但布局须把此 `ratio` 当作唯一信号——不设确定 size，
    /// 让 taffy/flex 按上下文 ratio-derive（如 flex column width 拉伸 → height=width/ratio）。
    /// PNG/JPEG/绝对尺寸 SVG 为 `None`（走 image_sizes 正常固有尺寸路径）。
    intrinsic_ratio: Option<f32>,
    /// no-ratio 信号（CSS §10.3.2）：SVG 的 width/height 非双绝对且**无**可用 viewBox
    /// 宽高比时，替换元素既无确定固有尺寸、也无固有宽高比。usvg 对缺失维给出默认值
    ///（如缺 height 时 h=100），该默认值不是真实固有尺寸——故 pixmap 的该维不可用于
    /// 比例推导。`Some((w, h))` 仅 no-ratio SVG 出现，`w`/`h` 为真实固有宽高（仅 abs
    /// 属性存在的维，缺失维为 `None`）；布局须**不设 aspect_ratio**，按 CSS §10.3.2
    /// default object size（宽 300 / 高 150）回退。PNG/JPEG/both-abs/ratio-only SVG 为 `None`。
    no_ratio: Option<(Option<f32>, Option<f32>)>,
    /// 计算的真实固有尺寸（CSS §10.3.2）：一维 abs + 另一维缺失 + viewBox 宽高比时，
    /// usvg 对缺失维用原始 viewBox 值（pixmap bogus），须按 abs 维 × ratio 计算。`Some((w,h))`
    /// 覆盖 pixmap 用于 image_sizes（aspect_ratio 由 w/h 推导）。仅该类 SVG 出现；其余为 `None`。
    computed_intrinsic: Option<(f32, f32)>,
    /// R3761：SVG 源字节（仅 SVG 置 Some）。放大绘制时渲染层按目标尺寸矢量重栅格化
    ///（`ImageCache::get_rasterized`），替代位图插值的宽渐变带。
    pub svg_source: Option<Arc<[u8]>>,
}

/// RGBA 像素内容摘要（R3254-M2 共享实现——ImageData 插入时预存一次，GPU 纹理缓存
/// 键复用同一算法，保证「同像素同 hash」）。
pub fn hash_pixels(pixels: &[u8]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    pixels.hash(&mut hasher);
    hasher.finish()
}

impl ImageData {
    /// 从 RGBA 字节数据创建图片
    ///
    /// # Errors
    /// 如果数据长度不等于 `width * height * 4` 则返回错误
    pub fn from_rgba(pixels: Vec<u8>, width: u32, height: u32) -> Result<Self, String> {
        let expected = (width as usize) * (height as usize) * 4;
        if pixels.len() != expected {
            return Err(format!(
                "pixel data size mismatch: expected {expected}, got {}",
                pixels.len()
            ));
        }
        // 检测纯色图片：所有像素相同时缓存颜色值
        let solid_color = if pixels.len() >= 4 {
            let first = [pixels[0], pixels[1], pixels[2], pixels[3]];
            let all_same = pixels.as_chunks::<4>().0.iter().all(|chunk| {
                chunk[0] == first[0] && chunk[1] == first[1] && chunk[2] == first[2] && chunk[3] == first[3]
            });
            if all_same { Some(first) } else { None }
        } else {
            None
        };
        let content_hash = hash_pixels(&pixels);
        Ok(Self {
            pixels,
            width,
            height,
            content_hash,
            solid_color,
            intrinsic_ratio: None,
            no_ratio: None,
            computed_intrinsic: None,
            svg_source: None,
        })
    }

    /// 创建指定尺寸的空（全透明）图片
    pub fn new_empty(width: u32, height: u32) -> Self {
        let pixels = vec![0u8; (width as usize) * (height as usize) * 4];
        let content_hash = hash_pixels(&pixels);
        Self {
            pixels,
            width,
            height,
            content_hash,
            solid_color: Some([0, 0, 0, 0]),
            intrinsic_ratio: None,
            no_ratio: None,
            computed_intrinsic: None,
            svg_source: None,
        }
    }

    /// 获取指定位置的像素 (R, G, B, A)。
    ///
    /// 越界坐标钳制到最近有效像素；像素缓冲与宽高不一致时返回透明（避免渲染崩溃）。
    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let expected = (self.width as usize) * (self.height as usize) * 4;
        if self.width == 0 || self.height == 0 || self.pixels.len() != expected {
            return [0, 0, 0, 0];
        }
        let x = x.min(self.width - 1);
        let y = y.min(self.height - 1);
        let idx = ((y * self.width + x) * 4) as usize;
        [
            self.pixels[idx],
            self.pixels[idx + 1],
            self.pixels[idx + 2],
            self.pixels[idx + 3],
        ]
    }

    /// 获取图片尺寸
    pub fn size(&self) -> Size {
        Size::new(self.width as f32, self.height as f32)
    }

    /// 如果图片是纯色（所有像素相同），返回该颜色
    ///
    /// 用于渲染优化：纯色图片缩放时不需要双线性插值，直接填充目标矩形即可。
    /// 这消除了 WPT reftest 中 swatch 图片（如 1x1-green.png、15x15-green.png）
    /// 缩放到大尺寸时的边缘抗锯齿伪影。
    pub fn solid_color(&self) -> Option<[u8; 4]> {
        self.solid_color
    }

    /// 仅含宽高比、无确定固有尺寸的信号（CSS §10.3.2，仅 SVG 出现）。
    ///
    /// 返回 `Some(ratio)` 表示此图无确定固有尺寸、仅有 viewBox 宽高比——布局须以
    /// ratio-only 信号处理（不设确定 size，仅设 aspect_ratio）。`None` 表示有确定
    /// 固有尺寸（width/height 字段有效，走 image_sizes 路径）。
    pub fn intrinsic_ratio(&self) -> Option<f32> {
        self.intrinsic_ratio
    }

    /// no-ratio 信号（CSS §10.3.2）：返回 `Some((w, h))` 表示此 SVG 既无确定固有尺寸
    /// 也无固有宽高比（width/height 非双绝对且无 viewBox），`w`/`h` 为真实固有宽高
    ///（仅 abs 属性存在的维，缺失维 `None`）。布局须不设 aspect_ratio，缺失维按
    /// default object size（宽 300 / 高 150）回退。`None` 表示非 no-ratio（走 sizes/ratios 路径）。
    pub fn no_ratio_intrinsic(&self) -> Option<(Option<f32>, Option<f32>)> {
        self.no_ratio
    }

    /// 计算的真实固有尺寸（CSS §10.3.2）：`Some((w, h))` 表示此 SVG 为「一维 abs + 另一维
    /// 缺失 + viewBox」类，usvg pixmap 对缺失维用原始 viewBox 值（bogus），须用此计算值
    ///（abs × ratio）覆盖 pixmap 用于 image_sizes。`None` 表示 pixmap 尺寸有效。
    pub fn computed_intrinsic(&self) -> Option<(f32, f32)> {
        self.computed_intrinsic
    }

    /// 字节大小估算
    pub fn byte_size(&self) -> usize {
        self.pixels.len()
    }
}

/// 图片缓存条目
#[derive(Debug)]
struct CacheEntry {
    /// 图片数据
    data: ImageData,
    /// 引用计数
    ref_count: u32,
    /// 最近访问的代数（用于 GC）
    last_access_gen: u64,
}

/// 图片对象缓存 — 管理已解码图片的生命周期
///
/// 使用引用计数和代际标记实现 GC：
/// - 插入图片时 ref_count = 1
/// - 每次 `get` 时递增 ref_count 并更新 last_access_gen
/// - `gc()` 移除 ref_count == 0 或长时间未访问的条目
pub struct ImageCache {
    /// 缓存条目
    entries: HashMap<ImageKey, CacheEntry>,
    /// 下一个键 ID
    next_key: u64,
    /// 当前世代（每次 GC 递增）
    current_gen: u64,
    /// 最大缓存条目数
    max_entries: usize,
    /// 最大字节数
    max_bytes: usize,
    /// R3761：SVG 按目标尺寸重栅格化缓存（(ImageKey, w, h) → 高分 ImageData）。
    svg_rasterized: HashMap<(ImageKey, u32, u32), ImageData>,
}

/// R3761：SVG 重栅格化缓存条目上限（每条最大 4M px ≈ 16MB，上限 8 条防内存膨胀）。
const SVG_RASTERIZED_MAX: usize = 8;

/// R3762：无 abs width/height 属性的 SVG，把目标尺寸注入根标签（width/height 属性
/// 替换或插入）——SVG 作为图像使用且无固有尺寸时 viewport = 使用处尺寸（css-images-4
/// default sizing + SVG2 viewport 建立语义），百分比内容（stroke-width、rect 几何）按
/// 真实 viewport 解析而非 usvg 默认 100×100（driving: WPT diagonal-percentage-vector-
/// background，10% stroke-width 应按定位区对角线 d=√(vw²+vh²)/√2 解析）。
/// 双 abs SVG 不注入（真固有 viewport，from_scale 矢量放大语义与 chromium 一致）。
fn inject_svg_target_dims(source: &[u8], target_w: u32, target_h: u32) -> Vec<u8> {
    let Ok(text) = std::str::from_utf8(source) else {
        return source.to_vec();
    };
    // 仅非双 abs 时注入（与本仓 svg_intrinsic_kind 分类一致：abs 维 = 正数非 % 值）。
    let Some(svg_start) = text.find("<svg") else {
        return source.to_vec();
    };
    let after = &text[svg_start..];
    let Some(tag_end_rel) = after.find('>') else {
        return source.to_vec();
    };
    let tag = &after[..tag_end_rel]; // 含 "<svg"
    let width_attr = extract_svg_attr(tag, "width");
    let height_attr = extract_svg_attr(tag, "height");
    let w_abs = width_attr.as_deref().and_then(parse_abs_length_value);
    let h_abs = height_attr.as_deref().and_then(parse_abs_length_value);
    if w_abs.is_some() && h_abs.is_some() {
        return source.to_vec();
    }
    // 替换或插入属性（改写根标签片段；值按目标 px 取整）。
    let mut new_tag = tag.to_string();
    for (name, value) in [("width", target_w), ("height", target_h)] {
        let v = format!("{value}");
        if extract_svg_attr(&new_tag, name).is_some() {
            // 替换现有值（引号样式保持简单：统一双引号重写该属性）。
            if let Some(idx) = new_tag.find(&format!("{name}=")) {
                let rest = &new_tag[idx + name.len() + 1..];
                let quote = rest.chars().next().unwrap_or('"');
                if (quote == '"' || quote == '\'')
                    && let Some(end_rel) = rest[1..].find(quote)
                {
                    let attr_end = idx + name.len() + 1 + 1 + end_rel + 1;
                    new_tag.replace_range(idx..attr_end, &format!("{name}=\"{v}\""));
                }
            }
        } else {
            // 插入到 "<svg" 之后（紧跟标签名，前置空格）。
            let insert_at = 4; // "<svg".len()
            new_tag.insert_str(insert_at, &format!(" {name}=\"{v}\""));
        }
    }
    let mut out = String::with_capacity(text.len() + 32);
    out.push_str(&text[..svg_start]);
    out.push_str(&new_tag);
    out.push_str(&after[tag_end_rel..]);
    out.into_bytes()
}

/// R3761：把 SVG 源栅格化到指定目标尺寸（矢量重渲染）。
///
/// 与 [`decode_svg_bytes`] 的固有尺寸栅格化相对：按 `target_w × target_h` 直接渲染。
/// R3762：无 abs 固有尺寸的 SVG 先注入目标 viewport（[`inject_svg_target_dims`]）再
/// 解析——百分比内容按真实 viewport 解析（usvg 对缺失 width/height 用默认 100×100，
/// 位图缩放会把 10% stroke 之类的 viewport 相对值错位）。双 abs SVG 保持固有 viewport
/// 等比 `Transform::from_scale` 放大。目标含 0 维返回 Err。
/// R3935 隔离验证结论：usvg 0.47 对 transform-origin 的 **px 值**形式处理正确
///（attr 与手写等价链渲染逐字节一致），**关键字**形式（center/left/right/top/bottom——
/// WPT svg-origin 簇全用）解析有缺陷（渲染错位）。本预处理把非 px 值翻译成 px 值，
/// 交 usvg 已验证正确的路径。
///
/// R3936（CSS Transforms 1 §transform-origin + SVG2 + css-transforms-1 §svg-transform）：
/// 把 SVG 文本中 `transform-origin="<value>"` 的**关键字/百分比**值翻译成 px（按 origin
/// 所在 `<rect>` 元素 bbox，即 fill-box 语义——WPT svg-origin-relative-length 簇形态；
/// 与 usvg 0.47 px origin 参照 = 元素 bbox 一致，r3936c 可区分对照实证）。非法组合
/// 按声明忽略回落 bbox 中心。**纯 px/无单位数值**透传：usvg 按用户空间绝对坐标解释，
/// 与 view-box 参照原点重合，本就正确（svg-origin-length 簇全绿实证）。非 rect 元素/
/// 几何缺失 → 原样保留（fail-closed，语义保守勿误翻）。
///
/// 注：WPT 案的 `transform-box: fill-box` 声明位于 HTML `<head>` 的 `<style>` 内，
/// 序列化 SVG 子树时不可见——参照系只能以值形态判别（关键字/百分比 → bbox 翻译）。
pub(crate) fn preprocess_svg_transform_origin(source: &[u8]) -> Vec<u8> {
    let Ok(text) = std::str::from_utf8(source) else {
        return source.to_vec();
    };
    if !text.contains("transform-origin") {
        return source.to_vec();
    }
    let token = "transform-origin=\"";
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut changed = false;
    while let Some(pos) = rest.find(token) {
        let value_start = pos + token.len();
        let Some(value_len) = rest[value_start..].find('"') else {
            // 未闭合（畸形）——原样拷贝余下文本退出，fail-closed。
            out.push_str(rest);
            return out.into_bytes();
        };
        let raw_value = &rest[value_start..value_start + value_len];
        out.push_str(&rest[..value_start]);
        // 纯数值双值（px/无单位，可负）→ 透传（usvg 用户空间绝对坐标 = view-box 语义，
        // svg-origin-length 簇全绿实证）；单值数字/关键字/百分比 → rect bbox 翻译；
        // 非法组合 → 删除 attr（CSS 声明忽略——usvg 缺省 pivot = viewport 中心，与
        // chromium 对无效 SVG attr origin 的行为一致——PROBE23 实证）。
        // rect 几何从 **token 前缀**解析（origin 所在元素的开标签必在 token 之前；
        // 传全文会 rfind 到其后标签）。
        let token_count = raw_value.split_whitespace().count();
        // 未知单位（cm/in/pt/mm/q/em 等）：view-box 语义下 usvg 自行解析物理单位
        //（svg-origin-length-{cm,in,pt} 簇 run1 透传全绿实证）——Keep 勿动。
        // 识别集：无单位数字 / px / 百分比 / 关键字。
        let known = raw_value.split_whitespace().all(|t| {
            origin_value_is_two_numbers(t)
                || t.ends_with('%')
                || matches!(t, "left" | "center" | "right" | "top" | "bottom")
        });
        let action = if !known {
            OriginAction::Keep
        } else if token_count == 1 && origin_value_is_two_numbers(raw_value) {
            // 单值数字：CSS 第二轴缺省 center，usvg 单值 Y 缺省 = 0（缺陷）——
            // 翻成「值 + bbox 中心 Y」双值（PROBE22：012/013 案实证）。
            match resolve_transform_origin_rect_bbox_single(&rest[..pos], raw_value) {
                Some(px) => OriginAction::Rewrite(px),
                None => OriginAction::Keep,
            }
        } else if origin_value_is_two_numbers(raw_value) {
            // 双值纯数值 → 透传（usvg 用户空间绝对坐标 = view-box 语义）。
            OriginAction::Keep
        } else {
            match resolve_transform_origin_rect_bbox(&rest[..pos], raw_value) {
                Some(px) => OriginAction::Rewrite(px),
                None => OriginAction::Drop,
            }
        };
        match action {
            OriginAction::Rewrite(px) => {
                out.push_str(&px);
                changed = true;
            }
            OriginAction::Keep => out.push_str(raw_value),
            OriginAction::Drop => {
                // 删除整个 attr：回退到 token 前面已写入的部分去掉尾部 `transform-origin="`。
                // out 当前以 `...transform-origin="` 结尾——截掉 token 本身。
                let truncate = out.len() - token.len();
                out.truncate(truncate);
                // 若截断后尾部残留多余空白，保留原样（无害）。
                changed = true;
                rest = &rest[value_start + value_len..]; // 跳过值，闭引号由下一轮外的 push 处理
                // 注意：value 后的闭引号 `"` 属于 rest 的第一个字符——也须删除。
                rest = rest.strip_prefix('"').unwrap_or(rest);
                // 此时 rest 以空格或 `/>` 开头，正常继续。
                continue;
            }
        }
        rest = &rest[value_start + value_len..];
    }
    out.push_str(rest);
    if changed { out.into_bytes() } else { source.to_vec() }
}

/// 预处理对单个 transform-origin attr 的动作。
#[derive(Clone, PartialEq, Debug)]
enum OriginAction {
    /// 改写为给定 px 值。
    Rewrite(String),
    /// 原样保留。
    Keep,
    /// 删除 attr（CSS 声明无效 → 忽略整条声明）。
    Drop,
}

/// origin 值是否全部为纯数值 token（px 或无单位数字，可负；1-2 个空白分隔）。
fn origin_value_is_two_numbers(value: &str) -> bool {
    let parts: Vec<&str> = value.split_whitespace().collect();
    !parts.is_empty()
        && parts.len() <= 2
        && parts.iter().all(|t| {
            let n = t.strip_suffix("px").unwrap_or(t).trim();
            n.parse::<f32>().is_ok()
        })
}

/// R3937（CSS Transforms 1 §transform-attribute-specificity + SVG2 presentation
/// attributes）：`style="...transform:..."` **覆盖**同元素 `transform` presentation
/// attr（CSS 级联序：inline style > presentation attribute）。usvg 0.47 忽略
/// style attr 的 transform（a-vs-attr=0 探针实证）——本预处理把 style attr 中的
/// CSS transform 函数串翻译为 SVG transform 语法并改写 transform attr。
/// CSS transform **非法**（如 `scale(invalid)`）→ 声明忽略，presentation attr 生效
///（保留原 attr，WPT inline-styles-005/006/010/013 形态）。
///
/// stylesheet（`<style>`/外部）的 transform 须 paint_svg_element 级联合成（序列化
/// 文本不含 stylesheet 规则）——独立切片。
pub(crate) fn preprocess_svg_style_transform(source: &[u8]) -> Vec<u8> {
    let Ok(text) = std::str::from_utf8(source) else {
        return source.to_vec();
    };
    if !text.contains("style=") || !text.contains("transform") {
        return source.to_vec();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut changed = false;
    // 逐个开标签处理：找到 `<tag ...>`（含 >），在开标签内部做 style→transform 合成。
    while let Some(lt) = rest.find('<') {
        // 拷贝 '<' 之前的文本。
        out.push_str(&rest[..lt]);
        let after_lt = &rest[lt..];
        let Some(gt_rel) = after_lt.find('>') else {
            out.push_str(after_lt);
            rest = "";
            break;
        };
        let tag = &after_lt[..=gt_rel]; // 含 '<' 与 '>'
        rest = &rest[lt + gt_rel + 1..];
        // 仅处理带 style=" 的开标签。
        if let Some(style_pos) = tag.find("style=\"") {
            let style_val_start = style_pos + "style=\"".len();
            let style_val = &tag[style_val_start
                ..tag[style_val_start..]
                    .find('"')
                    .map(|i| style_val_start + i)
                    .unwrap_or(style_val_start)];
            let css_transform = style_value_decls(style_val)
                .into_iter()
                .find(|(prop, _)| prop.eq_ignore_ascii_case("transform"))
                .map(|(_, v)| v);
            if let Some(css_value) = css_transform
                && let Some(svg_value) = css_transform_to_svg(&css_value)
            {
                // CSS transform 合法 → 改写/插入 transform attr（覆盖语义）+
                // **剥掉 style 中的 transform 声明**（usvg 对纯 style transform 的
                // pivot = CSS 缺省 origin，与 attr 覆盖值语义不同——残留会干扰
                // style-residue diff=10000 探针实证）。
                let stripped_style = strip_transform_decl(style_val);
                let tag_no_style = replace_style_value(tag, &stripped_style);
                let new_tag = set_transform_attr(&tag_no_style, &svg_value);
                out.push_str(&new_tag);
                changed = true;
                continue;
            }
        }
        out.push_str(tag);
    }
    out.push_str(rest);
    if changed { out.into_bytes() } else { source.to_vec() }
}

/// 解析 style attr 值为 (属性, 值) 声明列表。
fn style_value_decls(value: &str) -> Vec<(String, String)> {
    value
        .split(';')
        .filter_map(|decl| decl.split_once(':'))
        .map(|(p, v)| (p.trim().to_string(), v.trim().to_string()))
        .collect()
}

/// 在开标签文本（含 `<` `>`）中改写既有 `transform="..."` 值；无该 attr 则在
/// `>` 前插入 ` transform="..."`。
fn set_transform_attr(tag: &str, svg_value: &str) -> String {
    debug_assert!(tag.starts_with('<') && tag.ends_with('>'));
    let inner = &tag[..tag.len() - 1]; // 去掉 '>'
    if let Some(tpos) = inner.find("transform=\"") {
        let tval_start = tpos + "transform=\"".len();
        let Some(tval_rel) = inner[tval_start..].find('"') else {
            return tag.to_string();
        };
        let tval_end = tval_start + tval_rel;
        format!("{}{}{}>", &inner[..tval_start], svg_value, &inner[tval_end..])
    } else if let Some(stripped) = inner.strip_suffix('/') {
        // 自闭合标签：插在 `/` 之前。
        format!("{stripped} transform=\"{svg_value}\"/>")
    } else {
        format!("{inner} transform=\"{svg_value}\">")
    }
}

/// 把 style attr 值中的 transform 声明剥除（其余声明保留）。
fn strip_transform_decl(style_val: &str) -> String {
    style_val
        .split(';')
        .filter(|decl| {
            decl.split_once(':')
                .map(|(p, _)| !p.trim().eq_ignore_ascii_case("transform"))
                .unwrap_or(!decl.trim().is_empty())
        })
        .collect::<Vec<_>>()
        .join(";")
}

/// 把开标签文本中的 style="..." 值替换为给定新值。
fn replace_style_value(tag: &str, new_style: &str) -> String {
    debug_assert!(tag.starts_with('<') && tag.ends_with('>'));
    let inner = &tag[..tag.len() - 1];
    let Some(spos) = inner.find("style=\"") else {
        return tag.to_string();
    };
    let sval_start = spos + "style=\"".len();
    let Some(sval_rel) = inner[sval_start..].find('"') else {
        return tag.to_string();
    };
    let sval_end = sval_start + sval_rel;
    format!("{}{}{}>", &inner[..sval_start], new_style, &inner[sval_end..])
}

/// CSS transform 函数串 → SVG transform 语法（deg/px 后缀剥除、数值归一）。
/// 非法（函数名不识别 / 参数非数 / 个数不符）→ None。
fn css_transform_to_svg(css: &str) -> Option<String> {
    let mut parts_out: Vec<String> = Vec::new();
    let bytes = css.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b' ' || bytes[i] == b'\t' {
            i += 1;
            continue;
        }
        let open_rel = css[i..].find('(')?;
        let name = css[i..i + open_rel].trim().to_ascii_lowercase();
        let after_open = i + open_rel + 1;
        let close = css[after_open..].find(')')? + after_open;
        let args_raw = &css[after_open..close];
        let arg_tokens: Vec<&str> = if args_raw.trim().is_empty() {
            vec![]
        } else {
            args_raw.split(',').collect()
        };
        let args: Option<Vec<f64>> = arg_tokens
            .iter()
            .map(|a| {
                let v = a.trim().trim_end_matches("deg").trim_end_matches("px").trim();
                v.parse::<f64>().ok().filter(|n| n.is_finite())
            })
            .collect();
        let args = args?;
        let n = args.len();
        let part = match name.as_str() {
            "rotate" if n == 1 => format!("rotate({})", args[0]),
            "scale" if n == 1 => format!("scale({})", args[0]),
            "scale" if n == 2 => format!("scale({} {})", args[0], args[1]),
            "scalex" if n == 1 => format!("scale({} 1)", args[0]),
            "scaley" if n == 1 => format!("scale(1 {})", args[0]),
            "translate" if n == 1 => format!("translate({} {})", args[0], args[0]),
            "translate" if n == 2 => format!("translate({} {})", args[0], args[1]),
            "translatex" if n == 1 => format!("translate({} 0)", args[0]),
            "translatey" if n == 1 => format!("translate(0 {})", args[0]),
            "skewx" if n == 1 => format!("skewX({})", args[0]),
            "skewy" if n == 1 => format!("skewY({})", args[0]),
            "matrix" if n == 6 => format!(
                "matrix({} {} {} {} {} {})",
                args[0], args[1], args[2], args[3], args[4], args[5]
            ),
            _ => return None,
        };
        parts_out.push(part);
        i = close + 1;
    }
    if parts_out.is_empty() {
        None
    } else {
        Some(parts_out.join(" "))
    }
}

/// R3988（CSS Transforms 1 §svg-transform-functions + SVG2 transform attribute
/// 语法）：SVG transform attr 的函数参数列表**不允许空参数/尾随逗号**
///（`rotate(90,)` 整条无效 → 无 transform）。usvg 0.47 宽容解析尾随逗号
///（rotate 生效，探针 diff=6400 实证）——本预处理把含畸形参数列表的 transform
/// attr **删除**（chromium 语义 = 声明忽略）。
///
/// 判定（保守）：仅当参数列表存在「逗号后紧跟 `)` / 连续逗号 / 开括号后直接
/// 逗号」形态才删；其余一律不动。已知 usvg 与 chromium 一致的宽容域（如无单位
/// 数字）不在此列（R3936 已实证透传正确）。
pub(crate) fn preprocess_svg_transform_attr_syntax(source: &[u8]) -> Vec<u8> {
    let Ok(text) = std::str::from_utf8(source) else {
        return source.to_vec();
    };
    if !text.contains("transform=\"") {
        return source.to_vec();
    }
    let token = "transform=\"";
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut changed = false;
    while let Some(pos) = rest.find(token) {
        // 先拷 token 前文本（Drop 分支也须保留开标签前缀）。
        out.push_str(&rest[..pos]);
        let value_start = pos + token.len();
        let Some(value_len) = rest[value_start..].find('"') else {
            out.push_str(&rest[pos..]);
            return out.into_bytes();
        };
        let raw_value = &rest[value_start..value_start + value_len];
        if transform_attr_has_malformed_args(raw_value) {
            // 删除整个 attr：token+值+闭引号全不拷，直接跳过。
            changed = true;
            rest = &rest[value_start + value_len..];
            rest = rest.strip_prefix('"').unwrap_or(rest);
            continue;
        }
        out.push_str(token);
        out.push_str(raw_value);
        out.push('"');
        rest = &rest[value_start + value_len..];
    }
    out.push_str(rest);
    if changed { out.into_bytes() } else { source.to_vec() }
}

/// transform attr 值是否含畸形参数列表（空参数/尾随逗号/连续逗号）。
fn transform_attr_has_malformed_args(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let open = match value[i..].find('(') {
            Some(o) => i + o,
            None => return false,
        };
        let after_open = open + 1;
        let Some(close_rel) = value[after_open..].find(')') else {
            return false;
        };
        let args = &value[after_open..after_open + close_rel];
        let t = args.trim();
        // 尾随逗号 / 前导逗号 / 连续逗号 / 纯逗号 = 畸形。
        if t.ends_with(',')
            || t.starts_with(',')
            || t.contains(", ,")
            || t.contains(",,")
            || t.is_empty() && !args.is_empty() && args.contains(',')
        {
            return true;
        }
        i = after_open + close_rel + 1;
    }
    false
}

/// rect bbox 参照：从 origin attr 所在元素的开标签前缀解析 rect 几何（x/y/width/height
/// attr；缺 x/y 默认 0），按 fill-box 语义翻译 origin 分量为绝对 px。
/// 非 rect 元素或几何缺失 → None（调用方落 Drop/Keep，fail-closed）。
fn resolve_transform_origin_rect_bbox(prefix: &str, value: &str) -> Option<String> {
    let tag_open = prefix.rfind('<')?;
    let tag = &prefix[tag_open..];
    let after_tag_name = tag
        .strip_prefix("<rect")
        .filter(|t| t.starts_with(char::is_whitespace))?;
    let attr = |name: &str| -> Option<f64> {
        // 左边界要求空白前缀，防 `x="` 误匹配 `rx="` / `width="` 内子串。
        let pat = format!(r#" {name}=""#);
        let idx = after_tag_name.find(&pat)?;
        let after = &after_tag_name[idx + pat.len()..];
        let end = after.find('"')?;
        after[..end].trim().parse::<f64>().ok()
    };
    let x = attr("x").unwrap_or(0.0) as f32;
    let y = attr("y").unwrap_or(0.0) as f32;
    let w = attr("width")? as f32;
    let h = attr("height")? as f32;
    resolve_origin_components(value, (x, y), (x + w, y + h))
}

/// 单值数字的 bbox 参照翻译：x = 值（相对 bbox 左缘），y = bbox 垂直中心（CSS 单值
/// 第二轴缺省 center——usvg 单值时 Y 缺省 0 是缺陷，须补齐）。
/// 非 rect 元素或几何缺失 → None（保留原样）。
fn resolve_transform_origin_rect_bbox_single(prefix: &str, value: &str) -> Option<String> {
    let tag_open = prefix.rfind('<')?;
    let tag = &prefix[tag_open..];
    let after_tag_name = tag
        .strip_prefix("<rect")
        .filter(|t| t.starts_with(char::is_whitespace))?;
    let attr = |name: &str| -> Option<f64> {
        let pat = format!(r#" {name}=""#);
        let idx = after_tag_name.find(&pat)?;
        let after = &after_tag_name[idx + pat.len()..];
        let end = after.find('"')?;
        after[..end].trim().parse::<f64>().ok()
    };
    let x = attr("x").unwrap_or(0.0) as f32;
    let y = attr("y").unwrap_or(0.0) as f32;
    let h = attr("height")? as f32;
    let v = value.strip_suffix("px").unwrap_or(value).trim().parse::<f32>().ok()?;
    Some(format!("{}px {}px", x + v, y + h / 2.0))
}

/// origin 分量 → px：`origin` = 参照盒左上，`extent` = 右下。关键字 = 对应缘/中点；
/// 百分比 = origin + p%×轴跨；px 长度 = origin + 值。返回 `Some` = 绝对 px 改写；
/// `None` = 非法组合 → 调用方 Drop（删除 attr；CSS 声明忽略语义——usvg 缺省 pivot
/// = viewport 中心，与 chromium 忽略无效 SVG presentation attr 一致——PROBE23 实证）。
///
/// 词法（CSS Position §4 `<position>`）：单值 = 水平词/长度（垂直单关键字 top/bottom
/// 翻为水平 center + 该词）；双词歧义交换（垂直关键字首词或 center+水平词）；同轴对
/// / 垂直首词+偏移（`top 100%`）等非法组合 → None（Drop）。
fn resolve_origin_components(value: &str, origin: (f32, f32), extent: (f32, f32)) -> Option<String> {
    let (ox, oy) = origin;
    let (ex, ey) = extent;
    let parts: Vec<&str> = value.split_whitespace().collect();
    if parts.is_empty() || parts.len() > 2 {
        return None;
    }
    let is_h = |t: &str| matches!(t, "left" | "center" | "right");
    let is_v = |t: &str| matches!(t, "top" | "center" | "bottom");
    let is_len = |t: &str| {
        t.strip_suffix('%').map_or_else(
            || {
                let v = t.strip_suffix("px").unwrap_or(t).trim();
                v.parse::<f32>().is_ok() && (t.ends_with("px") || !v.is_empty() && !t.contains(['e', 'm']))
            },
            |p| p.trim().parse::<f32>().is_ok(),
        )
    };
    let (h_word, v_word) = match parts.len() {
        2 => {
            let (a, b) = (parts[0], parts[1]);
            // 词法按 WPT svg-origin-relative-length 全簇（含 invalid 12 案）校准：
            // 合法 = 直排（水平词/长度 + 垂直词/长度）、交换（垂直关键字首词 +
            // 水平关键字：`top left`/`center right`——CSS Position §4 关键字无歧义
            // 可换序）；非法（`top 100%` 垂直首词+偏移、`left left` 同轴）→ 声明
            // 忽略（None = Drop，删 attr 走 usvg 缺省 pivot——PROBE23 实证）。
            let pair = if matches!(a, "top" | "bottom") {
                if is_h(b) { Some((b, a)) } else { None }
            } else if a == "center" && matches!(b, "left" | "right") {
                Some((b, a))
            } else if (is_h(a) || is_len(a)) && (is_v(b) || is_len(b)) {
                Some((a, b))
            } else {
                None
            };
            match pair {
                Some((h, v)) => (h, v),
                // 非法组合 → None = Drop（删除 attr；usvg 缺省 pivot = viewport 中心，
                // 与 chromium 对无效 SVG presentation attr 的忽略行为一致——PROBE23）。
                None => return None,
            }
        }
        _ => {
            // 单值：水平词/长度（垂直缺省 center）或垂直单关键字（水平缺省 center）。
            if is_v(parts[0]) && !is_h(parts[0]) {
                ("center", parts[0])
            } else {
                (parts[0], "center")
            }
        }
    };
    let axis = |component: &str, o: f32, lo: f32, mid: f32, hi: f32, span: f32| -> Option<f32> {
        match component {
            "left" | "top" => Some(lo),
            "center" => Some(mid),
            "right" | "bottom" => Some(hi),
            other => {
                if let Some(pct) = other.strip_suffix('%') {
                    let v = pct.trim().parse::<f32>().ok()?;
                    return Some(o + v / 100.0 * span);
                }
                let v = other.strip_suffix("px").unwrap_or(other).trim().parse::<f32>().ok()?;
                if other.ends_with("px") || !other.contains(['e', 'm']) {
                    Some(o + v)
                } else {
                    None
                }
            }
        }
    };
    let x = axis(h_word, ox, ox, ox + (ex - ox) / 2.0, ex, ex - ox)?;
    let y = axis(v_word, oy, oy, oy + (ey - oy) / 2.0, ey, ey - oy)?;
    Some(format!("{x}px {y}px"))
}

/// R3933（inline `<svg>` paint）：按目标尺寸矢量栅格化 SVG 源字节——
/// inline `<svg>` 元素无外部 URL，painter 序列化其 DOM 子树后直接调用本函数产像素
/// （canvas/video 同款两段式：painter 产 rgba + ImagePrimitive，调用方注入 ImageCache）。
pub fn rasterize_svg_at(source: &[u8], target_w: u32, target_h: u32) -> Result<ImageData, String> {
    if target_w == 0 || target_h == 0 {
        return Err("SVG 目标尺寸为 0".to_string());
    }
    // R3996：根 `background-color` 提升为 viewport 级填充（chromium 语义）。usvg 把根
    // 背景画成 viewBox 区域的 path（letterbox 居中时不覆盖 viewport 全区），而 chromium
    // 对 replaced/document SVG 的根背景铺满整个 viewport——剥离该声明改用 `fill()`。
    let (bytes, root_background) = promote_svg_root_background(source);
    // R3936：transform-origin 预处理（fill-box/单 rect 参照，px 输出与源 viewport
    // 无关）；须在 inject 之前保持源结构可解析。
    // R3937：style attr transform 覆盖 presentation attr（CSS 级联序 > attr；
    // usvg 忽略 style transform——预处理把合法 CSS transform 翻译改写 attr）。
    let bytes = preprocess_svg_style_transform(&bytes);
    // R3988：transform attr 畸形参数列表（尾随逗号等）→ attr 删除（chromium 声明忽略）。
    let bytes = preprocess_svg_transform_attr_syntax(&bytes);
    let bytes = preprocess_svg_transform_origin(&bytes);
    let bytes = inject_svg_target_dims(&bytes, target_w, target_h);
    let tree = resvg::usvg::Tree::from_data(&bytes, &resvg::usvg::Options::default())
        .map_err(|e| format!("SVG 解析失败: {e}"))?;
    let size = tree.size();
    let (iw, ih) = (size.width(), size.height());
    if iw <= 0.0 || ih <= 0.0 {
        return Err("SVG 固有尺寸为 0".to_string());
    }
    let mut pixmap = resvg::tiny_skia::Pixmap::new(target_w, target_h)
        .ok_or_else(|| format!("SVG pixmap 分配失败 {target_w}x{target_h}"))?;
    if let Some(color) = root_background {
        pixmap.fill(resvg::tiny_skia::Color::from_rgba8(
            color[0], color[1], color[2], color[3],
        ));
    }
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(target_w as f32 / iw, target_h as f32 / ih),
        &mut pixmap.as_mut(),
    );
    ImageData::from_rgba(pixmap.take(), target_w, target_h)
}

/// R3996：把 `<svg>` 根元素的 `background-color`（presentation attr 或 style 声明）从源中
/// 剥离并返回（RGBA，未知/非法值返回 None 不动源）。
///
/// 根因（css-sizing replaced-element-004/006/010/020 簇实证）：usvg 0.47 把根背景生成为
/// **viewBox rect** 的 path（`converter.rs` `background_path(background_color, view_box.rect)`），
/// 位于 viewBox→viewport 的 letterbox transform 之内——`viewBox='0 0 5 1'` 的 SVG 栅格化到
/// 100×100 时绿带只有中央 20px。chromium 语义（HTML replaced element + CSS backgrounds）：
/// 根背景铺满整个 viewport（背景在 viewBox transform 之外）。修复 = 剥离声明 + 渲染前
/// `Pixmap::fill`（viewport 级）。根元素之外的 background 声明不受影响（仅扫根标签）。
/// https://drafts.csswg.org/css-backgrounds-3/#background-color
/// https://html.spec.whatwg.org/multipage/rendering.html#replaced-elements
fn promote_svg_root_background(source: &[u8]) -> (Vec<u8>, Option<[u8; 4]>) {
    let Ok(text) = std::str::from_utf8(source) else {
        return (source.to_vec(), None);
    };
    let Some(svg_start) = text.find("<svg") else {
        return (source.to_vec(), None);
    };
    let after = &text[svg_start..];
    let Some(tag_end_rel) = after.find('>') else {
        return (source.to_vec(), None);
    };
    let tag = &after[..tag_end_rel]; // 含 "<svg"
    // 值来源优先级（与 usvg parse 一致）：style 声明 > presentation attr。两者皆无 → 不动。
    let style_val = extract_svg_attr(tag, "style");
    let style_color = style_val.as_deref().and_then(svg_style_background_color);
    let attr_color = extract_svg_attr(tag, "background-color").and_then(|v| parse_svg_color(&v));
    let color = style_color.or(attr_color);
    let Some(color) = color else {
        return (source.to_vec(), None);
    };
    // 改写根标签：删 style 中的 background-color 声明（若来自 style），否则删 attr。
    let new_tag = if style_color.is_some() {
        let stripped_style = strip_background_decl(style_val.as_deref().unwrap_or_default());
        if stripped_style.is_empty() {
            // style 仅含 background-color → 整个 style attr 删除。
            remove_svg_attr(tag, "style")
        } else {
            replace_svg_attr_value(tag, "style", &stripped_style)
        }
    } else {
        remove_svg_attr(tag, "background-color")
    };
    let mut out = String::with_capacity(text.len());
    out.push_str(&text[..svg_start]);
    out.push_str(&new_tag);
    out.push_str(&after[tag_end_rel..]);
    (out.into_bytes(), Some(color))
}

/// 从 style 声明值中提取 `background-color`（合法 CSS 颜色 → RGBA）。
fn svg_style_background_color(style_val: &str) -> Option<[u8; 4]> {
    style_val
        .split(';')
        .filter_map(|decl| decl.split_once(':'))
        .find_map(|(prop, value)| {
            prop.trim()
                .eq_ignore_ascii_case("background-color")
                .then_some(value.trim())
                .and_then(parse_svg_color)
        })
}

/// 删除 style 声明值中的 background-color 项（返回剩余声明，分号规范化）。
fn strip_background_decl(style_val: &str) -> String {
    style_val
        .split(';')
        .filter(|decl| {
            decl.split_once(':')
                .is_none_or(|(prop, _)| !prop.trim().eq_ignore_ascii_case("background-color"))
        })
        .collect::<Vec<_>>()
        .join(";")
}

/// 解析 SVG/CSS 颜色字面量为 RGBA（命名色子集 + #rgb/#rrggbb + rgb()/rgba()）。
fn parse_svg_color(value: &str) -> Option<[u8; 4]> {
    let v = value.trim();
    if let Some(hex) = v.strip_prefix('#') {
        return match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                Some([r, g, b, 255])
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some([r, g, b, 255])
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some([r, g, b, a])
            }
            _ => None,
        };
    }
    // rgb()/rgba() 数值形式（逗号或空格分隔）。
    let lower = v.to_ascii_lowercase();
    for (prefix, default_a) in [("rgb(", 255u8), ("rgba(", 255)] {
        if let Some(inner) = lower.strip_prefix(prefix)
            && let Some(close) = inner.find(')')
        {
            let nums: Vec<f32> = inner[..close]
                .split([',', ' ', '/'])
                .filter(|t| !t.is_empty())
                .filter_map(|t| t.trim().parse::<f32>().ok())
                .collect();
            if nums.len() >= 3 {
                let clamp = |x: f32| x.clamp(0.0, 255.0) as u8;
                let a = nums
                    .get(3)
                    .map(|a| (a.clamp(0.0, 1.0) * 255.0) as u8)
                    .unwrap_or(default_a);
                return Some([clamp(nums[0]), clamp(nums[1]), clamp(nums[2]), a]);
            }
        }
    }
    // 命名色（WPT 常用子集；完整表非本切片所需）。
    let named: &[(&str, [u8; 3])] = &[
        ("green", [0, 128, 0]),
        ("lime", [0, 255, 0]),
        ("red", [255, 0, 0]),
        ("blue", [0, 0, 255]),
        ("white", [255, 255, 255]),
        ("black", [0, 0, 0]),
        ("yellow", [255, 255, 0]),
        ("transparent", [0, 0, 0]),
    ];
    for (name, rgb) in named {
        if lower.eq_ignore_ascii_case(name) {
            if *name == "transparent" {
                return Some([0, 0, 0, 0]);
            }
            return Some([rgb[0], rgb[1], rgb[2], 255]);
        }
    }
    None
}

/// 把 `name="old"` / `name='old'` 属性值替换为新值（保持引号样式；无该 attr 时原样返回）。
fn replace_svg_attr_value(tag: &str, name: &str, new_value: &str) -> String {
    let Some(idx) = tag.find(&format!("{name}=")) else {
        return tag.to_string();
    };
    let after = &tag[idx + name.len() + 1..];
    let Some(quote) = after.chars().next() else {
        return tag.to_string();
    };
    if quote != '"' && quote != '\'' {
        return tag.to_string();
    }
    let Some(end_rel) = after[1..].find(quote) else {
        return tag.to_string();
    };
    let attr_end = idx + name.len() + 1 + 1 + end_rel + 1;
    let mut out = String::with_capacity(tag.len());
    out.push_str(&tag[..idx]);
    out.push_str(&format!("{name}={quote}{new_value}{quote}"));
    out.push_str(&tag[attr_end..]);
    out
}

/// 从标签片段中删除整个 `name="..."` 属性（含前后一个空格；无该 attr 时原样返回）。
fn remove_svg_attr(tag: &str, name: &str) -> String {
    let pat = format!(" {name}=");
    let Some(idx) = tag.find(&pat) else {
        return tag.to_string();
    };
    let after = &tag[idx + pat.len()..];
    let Some(quote) = after.chars().next() else {
        return tag.to_string();
    };
    if quote != '"' && quote != '\'' {
        return tag.to_string();
    }
    let Some(end_rel) = after[1..].find(quote) else {
        return tag.to_string();
    };
    let attr_end = idx + pat.len() + 1 + end_rel + 1;
    let mut out = String::with_capacity(tag.len());
    out.push_str(&tag[..idx]);
    out.push_str(&tag[attr_end..]);
    out
}

impl ImageCache {
    /// 创建新的图片缓存
    ///
    /// - `max_entries`: 最大缓存条目数
    /// - `max_bytes`: 最大缓存字节数
    pub fn new(max_entries: usize, max_bytes: usize) -> Self {
        Self {
            entries: HashMap::new(),
            next_key: 0,
            current_gen: 0,
            max_entries,
            max_bytes,
            svg_rasterized: HashMap::new(),
        }
    }

    /// R34xx：快照全部缓存条目（key hash → ImageData clone），供 host 回调查询
    ///（webview 的 __zw_get_image_wire——headless canvas drawImage img 源 G5）。
    pub fn snapshot_entries(&self) -> std::collections::HashMap<u64, ImageData> {
        self.entries.iter().map(|(k, e)| (k.0, e.data.clone())).collect()
    }

    /// 插入图片数据，返回缓存键
    pub fn insert(&mut self, data: ImageData) -> ImageKey {
        let key = ImageKey::new(self.next_key);
        self.next_key += 1;

        let entry = CacheEntry {
            data,
            ref_count: 1,
            last_access_gen: self.current_gen,
        };
        self.entries.insert(key.clone(), entry);
        key
    }

    /// 使用指定的缓存键插入图片数据。
    ///
    /// 用于 reftest 场景：paint 系统通过 `simple_hash(src)` 生成 ImageKey，
    /// 外部加载器需要用相同的 key 将解码后的图片数据注入缓存。
    pub fn insert_with_key(&mut self, key: ImageKey, data: ImageData) {
        let entry = CacheEntry {
            data,
            ref_count: 1,
            last_access_gen: self.current_gen,
        };
        self.entries.insert(key, entry);
    }

    /// 获取图片数据的引用，并递增引用计数
    pub fn get(&mut self, key: &ImageKey) -> Option<&ImageData> {
        let entry = self.entries.get_mut(key)?;
        entry.ref_count = entry.ref_count.saturating_add(1);
        entry.last_access_gen = self.current_gen;
        Some(&entry.data)
    }

    /// R3761：按目标尺寸取 SVG 栅格化结果（矢量重栅格化，替代位图插值）。
    ///
    /// 放大绘制（background-size cover/contain、大尺寸替换元素）时，固有尺寸位图
    /// 的双线性插值会在色块边界产生数倍于源像素的渐变带（chromium 矢量栅格化边界
    /// 锐利）。此 API 按目标矩形尺寸对 SVG 源**重栅格化**（usvg 矢量重渲染，边界
    /// 精确），结果缓存在 `svg_rasterized`（FIFO，上限 [`SVG_RASTERIZED_MAX`]）。
    /// 非 SVG 条目或目标尺寸不大于固有尺寸（无放大收益）返回 `None`，调用方回退
    /// 原位图路径。
    pub fn get_rasterized(&mut self, key: &ImageKey, target_w: u32, target_h: u32) -> Option<&ImageData> {
        let source = {
            let entry = self.entries.get(key)?;
            let src = entry.data.svg_source.clone()?;
            // 固有尺寸（栅格图尺寸即上报尺寸）——仅放大时重栅格化有收益。
            let (iw, ih) = (entry.data.width, entry.data.height);
            if target_w <= iw || target_h <= ih || target_w == 0 || target_h == 0 {
                return None;
            }
            // 像素预算：超大目标回退位图路径（重栅格化 + 缓存不划算）。
            if u64::from(target_w) * u64::from(target_h) > 4_000_000 {
                return None;
            }
            src
        };
        let cache_key = (key.clone(), target_w, target_h);
        if !self.svg_rasterized.contains_key(&cache_key) {
            if self.svg_rasterized.len() >= SVG_RASTERIZED_MAX {
                // FIFO 驱逐最旧条目（HashMap 迭代序不定，驱逐任一即可——缓存语义）。
                let victim = self.svg_rasterized.keys().next().cloned().expect("len >= 1");
                self.svg_rasterized.remove(&victim);
            }
            let data = rasterize_svg_at(&source, target_w, target_h).ok()?;
            self.svg_rasterized.insert(cache_key, data);
        }
        self.svg_rasterized.get(&(key.clone(), target_w, target_h))
    }

    /// 释放一次引用（递减引用计数）
    pub fn release(&mut self, key: &ImageKey) {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.ref_count = entry.ref_count.saturating_sub(1);
        }
    }

    /// 立即删除指定图片，返回该键是否存在。
    pub fn remove(&mut self, key: &ImageKey) -> bool {
        self.entries.remove(key).is_some()
    }

    /// 执行垃圾回收
    ///
    /// 移除以下条目：
    /// - ref_count == 0 的条目
    /// - 如果总条目数或总字节数超过限制，按 LRU 淘汰最旧条目
    pub fn gc(&mut self) {
        self.current_gen += 1;

        // 先移除 ref_count == 0 的条目
        self.entries.retain(|_, entry| entry.ref_count > 0);

        // 如果仍然超限，按 last_access_gen 排序淘汰最旧条目
        while self.entries.len() > self.max_entries || self.total_bytes() > self.max_bytes {
            if self.entries.is_empty() {
                break;
            }
            // 找到最旧的条目并移除
            let oldest_key = self
                .entries
                .iter()
                .min_by_key(|(_, e)| e.last_access_gen)
                .map(|(k, _)| k.clone());

            if let Some(key) = oldest_key {
                self.entries.remove(&key);
            } else {
                break;
            }
        }
    }

    /// 当前缓存条目数
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 当前缓存字节总数
    pub fn total_bytes(&self) -> usize {
        self.entries.values().map(|e| e.data.byte_size()).sum()
    }

    /// 当前世代
    pub fn generation(&self) -> u64 {
        self.current_gen
    }

    /// 获取指定键的引用计数（用于测试）
    pub fn ref_count(&self, key: &ImageKey) -> Option<u32> {
        self.entries.get(key).map(|e| e.ref_count)
    }

    /// 清空所有缓存
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_gen += 1;
    }

    /// 复制当前缓存内容供 UI 线程快照使用（不共享可变状态）。
    pub fn duplicate_for_snapshot(&self) -> Self {
        let mut out = Self::new(self.max_entries, self.max_bytes);
        out.next_key = self.next_key;
        for (key, entry) in &self.entries {
            out.entries.insert(
                key.clone(),
                CacheEntry {
                    data: entry.data.clone(),
                    ref_count: 1,
                    last_access_gen: 0,
                },
            );
        }
        out
    }
}

impl Default for ImageCache {
    fn default() -> Self {
        Self::new(256, 64 * 1024 * 1024) // 256 entries, 64 MB
    }
}

// ─── 图片解码（PNG / JPEG / SVG）──────────────────────────────────────
//
// 供 URL 导航路径（webview fetch_url）抓取 `<img src>` 子资源后解码为 ImageData。
// render-foundation 拥有 ImageData 结构，故解码逻辑置此，供 webview / reftest 等复用。
// 支持格式：PNG（最常见）/ JPEG（goal doc DC-13「PNG/JPEG 基础解码」）/ WebP
// （R1793「PNG/JPEG/WebP 基础解码」，image-webp 纯 Rust）/ SVG 栅格化
// （resvg + tiny-skia，goal doc DC-13「SVG 栅格化」）。`decode_image_bytes` 按
// 魔数字节（PNG/JPEG/WebP）或文本内容嗅探（SVG）分发格式。

/// 将 PNG 字节流解码为 RGBA `ImageData`。
///
/// 正确处理任意 PNG color type（palette / grayscale / RGB / RGBA）与位深：
/// `EXPAND | STRIP_16` 变换把 palette→RGB(A)、grayscale→RGB(A)、16bit→8bit，
/// 再按 `OutputInfo.color_type` 转换为 RGBA。直接按 4 字节直读会导致非 RGBA 输入
/// 错位（alpha=0 退化透明）。
///
/// # Errors
/// 解码失败时返回描述性错误字符串（调用方决定降级策略）。
pub fn decode_png_bytes(bytes: &[u8]) -> Result<ImageData, String> {
    use png::Transformations;
    let mut decoder = png::Decoder::new(bytes);
    decoder.set_transformations(Transformations::EXPAND | Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(|e| format!("PNG 解码失败: {e}"))?;

    let info = reader.info().clone();
    let width = info.width;
    let height = info.height;

    let mut raw = vec![0u8; reader.output_buffer_size()];
    let output_info = reader.next_frame(&mut raw).map_err(|e| format!("PNG 读取失败: {e}"))?;
    let rgba = convert_png_buffer_to_rgba(&raw, output_info.color_type, output_info.bit_depth);
    ImageData::from_rgba(rgba, width, height)
}

/// 解码 JPEG 字节为 `ImageData`（RGBA）。
///
/// goal doc DC-13「图片子资源 / ImageCache」要求 PNG/JPEG 基础解码。
/// 使用 `jpeg-decoder`（纯 Rust，MIT/Apache-2.0），输出统一转 RGBA。
pub fn decode_jpeg_bytes(bytes: &[u8]) -> Result<ImageData, String> {
    use jpeg_decoder::Decoder;
    let mut decoder = Decoder::new(bytes);
    let pixels = decoder.decode().map_err(|e| format!("JPEG 解码失败: {e}"))?;
    let info = decoder.info().ok_or_else(|| "JPEG 无图像元数据".to_string())?;
    let width = info.width as u32;
    let height = info.height as u32;
    let rgba = convert_jpeg_pixels_to_rgba(&pixels, info.pixel_format);
    ImageData::from_rgba(rgba, width, height)
}

/// 解码 WebP 字节为 `ImageData`（RGBA）。
///
/// goal doc Support Envelope「图片子资源 / ImageCache」要求 PNG/JPEG/WebP 基础解码
///（rendering-compat.md line 76）。使用 `image-webp`（纯 Rust，MIT/Apache-2.0）；
/// 该 crate 已作为 `image` 的传递依赖存在于 Cargo.lock（0.2.4），此处提升为直接依赖。
/// `read_image` 在 `has_alpha` 时输出 RGBA，否则输出 RGB（补 alpha=255），统一转 RGBA。
pub fn decode_webp_bytes(bytes: &[u8]) -> Result<ImageData, String> {
    use std::io::Cursor;
    let mut decoder = image_webp::WebPDecoder::new(Cursor::new(bytes)).map_err(|e| format!("WebP 解码失败: {e}"))?;
    let (width, height) = decoder.dimensions();
    let has_alpha = decoder.has_alpha();
    let buf_size = decoder
        .output_buffer_size()
        .ok_or_else(|| "WebP 无输出缓冲区大小（可能为动画，暂不支持）".to_string())?;
    let mut raw = vec![0u8; buf_size];
    decoder
        .read_image(&mut raw)
        .map_err(|e| format!("WebP 读取失败: {e}"))?;
    let rgba = if has_alpha {
        // 已是 RGBA，直接使用。
        raw
    } else {
        // RGB → RGBA（补 alpha=255）。
        let mut out = Vec::with_capacity(raw.len() / 3 * 4);
        for px in raw.as_chunks::<3>().0 {
            out.extend_from_slice(&[px[0], px[1], px[2], 255]);
        }
        out
    };
    ImageData::from_rgba(rgba, width, height)
}

/// 把 `jpeg-decoder` 输出的像素（RGB24/L8/L16/CMYK32）转换为 RGBA。
///
/// JPEG 不含 alpha 通道，故统一补 alpha=255。CMYK 按 JPEG 惯例（Adobe 倒置 K）
/// 用「255 - value」近似转 RGB。
fn convert_jpeg_pixels_to_rgba(raw: &[u8], pixel_format: jpeg_decoder::PixelFormat) -> Vec<u8> {
    use jpeg_decoder::PixelFormat;
    match pixel_format {
        PixelFormat::RGB24 => {
            let mut out = Vec::with_capacity(raw.len() / 3 * 4);
            for px in raw.as_chunks::<3>().0 {
                out.extend_from_slice(&[px[0], px[1], px[2], 255]);
            }
            out
        }
        PixelFormat::L8 => {
            let mut out = Vec::with_capacity(raw.len() * 4);
            for &g in raw {
                out.extend_from_slice(&[g, g, g, 255]);
            }
            out
        }
        PixelFormat::L16 => {
            // 16-bit grayscale（big-endian u16），降级为 8-bit RGBA（取高字节）。
            let mut out = Vec::with_capacity(raw.len() / 2 * 4);
            for px in raw.as_chunks::<2>().0 {
                let hi = px[0];
                out.extend_from_slice(&[hi, hi, hi, 255]);
            }
            out
        }
        PixelFormat::CMYK32 => {
            // CMYK → RGB（Adobe JPEG 惯例：K 倒置，C/M/Y 取 255-value）
            let mut out = Vec::with_capacity(raw.len() / 4 * 4);
            for px in raw.as_chunks::<4>().0 {
                let c = 255 - px[0];
                let m = 255 - px[1];
                let y = 255 - px[2];
                let k = px[3] as u32;
                let r = (c as u32 * k / 255).min(255) as u8;
                let g = (m as u32 * k / 255).min(255) as u8;
                let b = (y as u32 * k / 255).min(255) as u8;
                out.extend_from_slice(&[r, g, b, 255]);
            }
            out
        }
    }
}

/// 按魔数字节嗅探图片格式并解码（PNG / JPEG / SVG）。
///
/// 比 URL 扩展名更可靠（URL 可能无扩展名或扩展名错误）。PNG 文件以 `\x89PNG` 开头；
/// JPEG 文件以 `\xFF\xD8\xFF` 开头；SVG 为文本，嗅探 UTF-8 内容起始 `<svg` / `<?xml`。
/// 未知格式返回错误，调用方可记录日志并降级（不阻断页面加载）。
pub fn decode_image_bytes(bytes: &[u8]) -> Result<ImageData, String> {
    if bytes.starts_with(b"\x89PNG") {
        decode_png_bytes(bytes)
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        decode_jpeg_bytes(bytes)
    } else if is_webp_magic(bytes) {
        decode_webp_bytes(bytes)
    } else if looks_like_svg(bytes) {
        decode_svg_bytes(bytes)
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        decode_gif_bytes(bytes)
    } else {
        Err(format!(
            "unsupported image format (magic bytes: {:?}); only PNG/JPEG/WebP/SVG/GIF supported",
            bytes.get(..4).unwrap_or(&[])
        ))
    }
}

/// R34xx（drawing-images 目录）：GIF 首帧解码（2d.drawImage.animated.gif 的
/// drawImage 画首帧——anim-gr.gif 首帧绿）。GIF89a：逻辑屏幕 + 全局色表 +
/// 首个图像描述符 + LZW 压缩数据（多帧忽略——静态首帧）。
/// https://www.w3.org/Graphics/GIF/spec-gif89a.txt
fn decode_gif_bytes(bytes: &[u8]) -> Result<ImageData, String> {
    if bytes.len() < 13 {
        return Err("GIF too short".into());
    }
    let w = u16::from_le_bytes([bytes[6], bytes[7]]) as u32;
    let h = u16::from_le_bytes([bytes[8], bytes[9]]) as u32;
    if w == 0 || h == 0 || w > 16384 || h > 16384 {
        return Err("GIF invalid dimensions".into());
    }
    let flags = bytes[10];
    let gct_flag = flags & 0x80 != 0;
    let gct_size = if gct_flag {
        2usize.pow((flags & 0x07) as u32 + 1)
    } else {
        0
    };
    // 全局色表（RGB 每项 3 字节）
    let mut pos = 13usize;
    let mut gct: Vec<[u8; 3]> = Vec::new();
    if gct_flag {
        if bytes.len() < pos + gct_size * 3 {
            return Err("GIF truncated GCT".into());
        }
        for i in 0..gct_size {
            gct.push([bytes[pos + i * 3], bytes[pos + i * 3 + 1], bytes[pos + i * 3 + 2]]);
        }
        pos += gct_size * 3;
    }
    // 扫描块到首个图像描述符（0x2C）
    loop {
        if pos >= bytes.len() {
            return Err("GIF no image descriptor".into());
        }
        match bytes[pos] {
            0x2C => break,
            0x21 => {
                // 扩展块：跳过标签 + 子块
                pos += 2;
                while pos < bytes.len() && bytes[pos] != 0 {
                    pos += 1 + bytes[pos] as usize;
                }
                pos += 1; // 0 终止
            }
            0x3B => return Err("GIF no image".into()),
            _ => return Err("GIF unexpected block".into()),
        }
    }
    // 图像描述符
    if bytes.len() < pos + 10 {
        return Err("GIF truncated image descriptor".into());
    }
    let left = u16::from_le_bytes([bytes[pos + 1], bytes[pos + 2]]) as u32;
    let top = u16::from_le_bytes([bytes[pos + 3], bytes[pos + 4]]) as u32;
    let iw = u16::from_le_bytes([bytes[pos + 5], bytes[pos + 6]]) as u32;
    let ih = u16::from_le_bytes([bytes[pos + 7], bytes[pos + 8]]) as u32;
    let iflags = bytes[pos + 9];
    pos += 10;
    // 局部色表
    let mut lct: Vec<[u8; 3]> = Vec::new();
    if iflags & 0x80 != 0 {
        let lct_size = 2usize.pow((iflags & 0x07) as u32 + 1);
        if bytes.len() < pos + lct_size * 3 {
            return Err("GIF truncated LCT".into());
        }
        for i in 0..lct_size {
            lct.push([bytes[pos + i * 3], bytes[pos + i * 3 + 1], bytes[pos + i * 3 + 2]]);
        }
        pos += lct_size * 3;
    }
    let use_lct = iflags & 0x80 != 0;
    if pos >= bytes.len() {
        return Err("GIF truncated LZW min code".into());
    }
    let min_code = bytes[pos] as usize;
    pos += 1;
    if min_code > 8 {
        return Err("GIF invalid LZW min code".into());
    }
    // LZW 数据子块
    let mut lzw: Vec<u8> = Vec::new();
    while pos < bytes.len() && bytes[pos] != 0 {
        let len = bytes[pos] as usize;
        if pos + 1 + len > bytes.len() {
            return Err("GIF truncated LZW data".into());
        }
        lzw.extend_from_slice(&bytes[pos + 1..pos + 1 + len]);
        pos += 1 + len;
    }
    let indices = gif_lzw_decode(&lzw, min_code)?;
    if indices.len() < (iw * ih) as usize {
        return Err("GIF LZW output too short".into());
    }
    // 上采样到画布尺寸（左/上偏移 + 透明色索引）
    let transparent = if iflags & 0x01 != 0 {
        bytes.get(pos + 1).copied()
    } else {
        None
    };
    let mut data = vec![0u8; (w * h * 4) as usize];
    let palette = if use_lct { &lct } else { &gct };
    for y in 0..ih {
        for x in 0..iw {
            let si = (y * iw + x) as usize;
            let idx = indices[si] as usize;
            let (dx, dy) = (left + x, top + y);
            if dx >= w || dy >= h {
                continue;
            }
            let di = ((dy * w + dx) * 4) as usize;
            if Some(idx as u8) == transparent {
                data[di + 3] = 0;
            } else if let Some(c) = palette.get(idx) {
                data[di] = c[0];
                data[di + 1] = c[1];
                data[di + 2] = c[2];
                data[di + 3] = 255;
            }
        }
    }
    Ok(ImageData {
        width: w,
        height: h,
        pixels: data,
        content_hash: 0,
        solid_color: None,
        intrinsic_ratio: None,
        no_ratio: None,
        computed_intrinsic: None,
        svg_source: None,
    })
}

/// GIF LZW 解码（spec gif89a LZW——clear/EOI 代码 + 变长码宽 + 字典增长）。
fn gif_lzw_decode(data: &[u8], min_code: usize) -> Result<Vec<u8>, String> {
    let clear_code = 1usize << min_code;
    let eoi_code = clear_code + 1;
    let mut code_width = min_code + 1;
    let mut dict: Vec<Vec<u8>> = (0..clear_code).map(|i| vec![i as u8]).collect();
    dict.push(Vec::new()); // clear
    dict.push(Vec::new()); // eoi
    let mut out: Vec<u8> = Vec::new();
    let mut bit_pos = 0usize;
    let mut prev: Option<Vec<u8>> = None;
    loop {
        // 读 code_width 位
        if bit_pos + code_width > data.len() * 8 {
            break;
        }
        let mut code = 0usize;
        for i in 0..code_width {
            let byte = data[(bit_pos + i) / 8] as usize;
            let bit = (byte >> ((bit_pos + i) % 8)) & 1;
            code |= bit << i;
        }
        bit_pos += code_width;
        if code == eoi_code {
            break;
        }
        if code == clear_code {
            dict = (0..clear_code).map(|i| vec![i as u8]).collect();
            dict.push(Vec::new());
            dict.push(Vec::new());
            code_width = min_code + 1;
            prev = None;
            continue;
        }
        let entry = if code < dict.len() {
            dict[code].clone()
        } else if let Some(p) = &prev {
            // code == dict.len()：KwKwK 情形（前串 + 首字节）
            let mut e = p.clone();
            e.push(p[0]);
            e
        } else {
            return Err("GIF LZW invalid code".into());
        };
        out.extend_from_slice(&entry);
        if let Some(p) = &prev {
            let mut np = p.clone();
            np.push(entry[0]);
            dict.push(np);
            if dict.len() == (1usize << code_width) && code_width < 12 {
                code_width += 1;
            }
        }
        prev = Some(entry);
    }
    Ok(out)
}

/// WebP magic：`RIFF` (offset 0..4) + 文件大小 4 字节 + `WEBP` (offset 8..12)。
/// 见 https://developers.google.com/speed/webp/docs/riff_container。
fn is_webp_magic(bytes: &[u8]) -> bool {
    bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP"
}

/// 是否为可独立进程解码的栅格图像字节（PNG/JPEG/WebP magic 检测）。
///
/// D1：image-decoder 独立进程只处理栅格格式；SVG（依赖资源加载）与
/// data URI 保持在调用进程内解码。供 webview 侧解码分发使用。
pub fn is_raster_image_bytes(bytes: &[u8]) -> bool {
    bytes.starts_with(b"\x89PNG") || bytes.starts_with(&[0xFF, 0xD8, 0xFF]) || is_webp_magic(bytes)
}

/// R34xx：从 SVG 源提取根 `<svg>` 元素的 width/height 属性（0 尺寸 SVG 的 usvg 拒绝兜底——
/// [`decode_svg_bytes`]）。仅接受无单位或 px 的数字；任一缺失/非有限/负 → None。
fn extract_svg_attr_dims(bytes: &[u8]) -> Option<(u32, u32)> {
    let text = std::str::from_utf8(bytes).ok()?;
    let svg_start = text.find("<svg")?;
    let rest = &text[svg_start..];
    let tag_end = rest.find('>')?;
    let tag = &rest[..tag_end];
    let attr = |name: &str| -> Option<f64> {
        let pat = format!("{name}=");
        let idx = tag.find(&pat)?;
        let after = &tag[idx + pat.len()..];
        let v = after.trim_start();
        let quote = v.chars().next()?;
        if quote != '"' && quote != '\'' {
            return None;
        }
        let inner = v.get(1..)?.split(quote).next()?;
        let num = inner.strip_suffix("px").unwrap_or(inner).trim();
        let val: f64 = num.parse().ok()?;
        if !val.is_finite() || val < 0.0 {
            return None;
        }
        Some(val)
    };
    Some((attr("width")?.ceil() as u32, attr("height")?.ceil() as u32))
}

/// R3760：提取 SVG 根元素 `viewBox` 的 `(width, height)`（第 3/4 个分量）。
///
/// 仅用于退化检测（0 维 → 空图像，css-images-4 §5.4.1）；无 viewBox 或分量缺失/非数
/// 返回 None（不做退化处理，走正常栅格化）。
fn extract_svg_viewbox_dims(bytes: &[u8]) -> Option<(f32, f32)> {
    let text = std::str::from_utf8(bytes).ok()?;
    let svg_start = text.find("<svg")?;
    let rest = &text[svg_start..];
    let tag_end = rest.find('>')?;
    let viewbox =
        extract_svg_attr(&rest[..tag_end], "viewBox").or_else(|| extract_svg_attr(&rest[..tag_end], "viewbox"))?;
    let nums: Vec<&str> = viewbox.split([' ', ',']).filter(|t| !t.is_empty()).collect();
    if nums.len() != 4 {
        return None;
    }
    let vw: f32 = nums[2].parse().ok()?;
    let vh: f32 = nums[3].parse().ok()?;
    Some((vw, vh))
}

/// 解析 `data:` URI 为原始字节。
///
/// `data:[<mediatype>][;base64],<payload>` —— header 含 `base64` 则 base64 解码 payload，
/// 否则按字节 percent-decode（%XX）。无逗号或解码失败返回 Err。
///
/// https://url.spec.whatwg.org/#data-urls
pub fn decode_data_uri_bytes(src: &str) -> Result<Vec<u8>, String> {
    if !src.get(..5).is_some_and(|prefix| prefix.eq_ignore_ascii_case("data:")) {
        return Err("不是 data URI".to_string());
    }
    let Some(comma) = src.find(',') else {
        return Err("data URI 缺逗号分隔符".to_string());
    };
    let header = &src[..comma];
    let payload = &src[comma + 1..];
    let bytes = if header.split(';').any(|part| part.eq_ignore_ascii_case("base64")) {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(payload)
            .map_err(|e| format!("data URI base64 解码失败: {e}"))?
    } else {
        percent_decode_bytes(payload)
    };
    Ok(bytes)
}

/// R1705：解析 `data:` URI 并解码为 `ImageData`（renderer 多进程路径 + wpt-runner 共用）。
///
/// 所得字节交 `decode_image_bytes` 按 magic 分派（PNG/JPEG/WebP/SVG）。
/// 解码失败返回 Err（调用方降级，不阻断页面）。
pub fn decode_data_uri(src: &str) -> Result<ImageData, String> {
    decode_image_bytes(&decode_data_uri_bytes(src)?)
}

/// 字节级 percent-decode（%XX → byte）；非 % 字节原样保留（data URI 非 base64 payload）。
fn percent_decode_bytes(input: &str) -> Vec<u8> {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Ok(b) = u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""), 16)
        {
            out.push(b);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    out
}

/// 嗅探字节是否为 SVG（文本，跳过 UTF-8 BOM 与前导空白后以 `<svg` 或 `<?xml` 开头）。
///
/// `<?xml` 声明不一定是 SVG，但在图片加载上下文中非 PNG/JPEG 的 XML 应尝试 SVG
/// 解码（resvg 对非 SVG XML 会解析失败并返回错误，降级安全）。
fn looks_like_svg(bytes: &[u8]) -> bool {
    let Ok(s) = std::str::from_utf8(bytes) else {
        return false;
    };
    let trimmed = s.trim_start_matches('\u{feff}').trim_start();
    trimmed.starts_with("<svg") || trimmed.starts_with("<?xml")
}

/// 把 SVG 字节栅格化为 RGBA `ImageData`（goal doc DC-13「SVG 栅格化」）。
///
/// 用 resvg + tiny-skia 按 SVG 内在尺寸（width/height 属性或 viewBox）栅格化。
/// 字体走默认空 fontdb（logo 一般无文本）；过大尺寸由 pixmap 分配失败自然兜底。
///
/// CSS §10.3.2：当 `<svg>` 的 width/height 为百分比或缺失（仅有 viewBox）时，替换元素
/// **无确定固有尺寸**，仅有 viewBox 宽高比。此时栅格化仍按 usvg 解析尺寸产出像素，
/// 但会设置 `intrinsic_ratio` 信号，让布局以 ratio-only 处理（不设确定 size）。
/// 当 width/height 非双绝对且**无** viewBox 时，替换元素既无确定固有尺寸也无固有宽高比
///（no-ratio）——usvg 对缺失维给出默认值（如缺 height 时 h=100），该默认值非真实固有
/// 尺寸，布局须以 `no_ratio` 信号处理（不设 aspect_ratio，缺失维按 default object size 回退）。
pub fn decode_svg_bytes(bytes: &[u8]) -> Result<ImageData, String> {
    // R3760：viewBox 含 0 维（`viewBox="0 0 8 0"` 等）的退化 SVG——css-images-4 §5.4.1
    //（default sizing algorithm）：固有宽度/高度为 0 的图像视为空。usvg 对缺失 width/
    // height 属性的此类 SVG 仍按默认 100×100 栅格化（内容 100%×100% 会被画出 lime 实心，
    // WPT background-size/vector zero-*-ratio 簇期望**空**）。绕过栅格化直接返回 1×1
    // 全透明；固有分类仍走 svg_intrinsic_kind（0 维 viewBox 无有效 ratio → NoRatio），
    // width-only 等属性维照常保留（布局/painter 的 no_ratio 语义不变）。
    if let Some((vw, vh)) = extract_svg_viewbox_dims(bytes)
        && (vw <= 0.0 || vh <= 0.0)
    {
        let mut data = ImageData::from_rgba(vec![0u8; 4], 1, 1)?;
        if let SvgIntrinsicKind::NoRatio { width, height } = svg_intrinsic_kind(bytes) {
            data.no_ratio = Some((width, height));
        }
        return Ok(data);
    }
    let tree = match resvg::usvg::Tree::from_data(bytes, &resvg::usvg::Options::default()) {
        Ok(tree) => tree,
        Err(e) => {
            // R34xx：usvg 拒绝 0 尺寸 SVG（width="0"/height="0"）——手工提取根元素
            // width/height 属性，双绝对且含 0 维 → 返回 0 维 ImageData（2d.pattern.
            // image.zerowidth/zeroheight——createPattern 期望 null 而非解码错误）。
            if let Some((w, h)) = extract_svg_attr_dims(bytes)
                && (w == 0 || h == 0)
            {
                // 空像素缓冲须与 (max(w,1) × max(h,1)) 匹配（from_rgba 校验长度）。
                return ImageData::from_rgba(
                    vec![0u8; (w.max(1) as usize).saturating_mul(h.max(1) as usize).saturating_mul(4)],
                    w.max(1),
                    h.max(1),
                )
                .map(|mut d| {
                    d.width = w;
                    d.height = h;
                    d
                });
            }
            return Err(format!("SVG 解析失败: {e}"));
        }
    };
    let size = tree.size();
    // usvg Size 的 width()/height() 返回 f32（SVG 内在尺寸）
    let w = size.width().ceil() as u32;
    let h = size.height().ceil() as u32;
    if w == 0 || h == 0 {
        // R34xx：0 尺寸 SVG 合法（2d.pattern.image.zerowidth/zeroheight——createPattern
        // 期望 null 而非 InvalidStateError；尺寸记录供 naturalWidth 查询）。usvg 对
        // width="0"/height="0" 的 SVG **拒绝解析**（不达此处）——零维解析兜底在下方
        // from_data 失败分支。
        return ImageData::from_rgba(
            vec![0u8; (w.max(1) as usize).saturating_mul(h.max(1) as usize).saturating_mul(4)],
            w.max(1),
            h.max(1),
        )
        .map(|mut d| {
            d.width = w;
            d.height = h;
            d
        });
    }
    let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h).ok_or_else(|| format!("SVG pixmap 分配失败 {w}x{h}"))?;
    resvg::render(&tree, resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());
    let rgba = pixmap.take();
    let mut data = ImageData::from_rgba(rgba, w, h)?;
    // R3761：携带 SVG 源字节——放大绘制（背景 cover/contain、大尺寸替换元素）时
    // 渲染层可按目标尺寸矢量重栅格化（`ImageCache::get_rasterized`），替代位图插值
    // 的宽渐变带。仅 SVG 置 Some（栅格图无矢量语义）。
    data.svg_source = Some(Arc::from(bytes.to_vec()));
    // ★ chromium 实测（visudet replaced-elements 簇 4 变体 × 7 SVG + css-flexbox
    // aspect-ratio-intrinsic-size-007，2026-07-15）：
    // - **INLINE `<img>`**（CSS2 §10.3.2）：非 BothAbs SVG 一律按 **default object size 300×150**
    //   sizing，**不**对显式 CSS 维应用 viewBox 宽高比（height-25-ratio-2.svg 配 width:40 → 40×150
    //   非 40×20）。仅 width+height 双绝对属性提供真固有尺寸 + ratio。
    // - **FLEX item**（flexbox §9.2.4 transferred-size）：ratio-only / 单 abs 维+viewBox SVG **保留**
    //   viewBox 宽高比（aspect-ratio-intrinsic-007：large-green-rectangle.svg 配 flex column →
    //   784×392，width stretch + height=width/ratio）。
    // 故 viewBox 比（RatioOnly / ComputedIntrinsic）仍走 intrinsic_ratio（flex transferred-size 用），
    // 但布局层对 INLINE 不应用该比（tree.rs branch 3 inline 分支用 default object size）。
    // NoRatio（无 viewBox）→ no_ratio(None,None)（inline default object size，无 ratio）。
    // （纠正 R1438「单 abs 维+viewBox → computed intrinsic 进 image_sizes」——inline 方向远离 chromium。）
    match svg_intrinsic_kind(bytes) {
        // 双绝对 → 真固有尺寸（pixmap w/h 有效，走 image_sizes；两信号均 None）
        SvgIntrinsicKind::BothAbs => {}
        // viewBox 比（无 abs 维 / 单 abs 维）：保留 ratio 供 flex transferred-size；inline 由
        // 布局层 default object size 处理（不应用此 ratio）。
        SvgIntrinsicKind::RatioOnly(ratio) => data.intrinsic_ratio = Some(ratio),
        // R2054：一维 abs + 另一维缺失 + viewBox 比 → **计算的真实固有尺寸**（abs × ratio）
        // 走 computed_intrinsic（→ image_sizes），使 INLINE auto+auto 用该固有尺寸（chromium
        // visudet replaced-elements-all-auto：height-25-ratio-2 → 50×25，非 300×150）。
        // 旧实现设 intrinsic_ratio 致 layout ratio-only 分支 INLINE 走 default 300×150（错）。
        // flex transferred-size 仍经 image_sizes 的 aspect_ratio 推导（tree.rs:439-441）。
        SvgIntrinsicKind::ComputedIntrinsic(cw, ch) if ch > 0.0 && cw > 0.0 => {
            // R3998：usvg 对「一维 abs + 另一维缺失 + viewBox」的 SVG 栅格化 viewport 不施
            // viewBox 比（`width="200"` + viewBox 1:1 → pixmap 200×100，内容按
            // preserveAspectRatio letterbox 居中——两侧透明）。该 bogus 位图若直接进
            // ImageCache，绘制放大/缩小采样时把透明 letterbox 一并拉进元素盒（010 红边
            // 根因）。按计算出的真实固有尺寸重栅格化位图（viewport = computed，viewBox
            // 不 letterbox），像素与固有尺寸一致。
            if ((cw - data.width as f32).abs() > 0.5 || (ch - data.height as f32).abs() > 0.5)
                && let Ok(mut true_data) =
                    rasterize_svg_at(bytes, cw.round().max(1.0) as u32, ch.round().max(1.0) as u32)
            {
                true_data.computed_intrinsic = Some((cw, ch));
                true_data.svg_source = data.svg_source.take();
                data = true_data;
            }
            if data.computed_intrinsic.is_none() {
                data.computed_intrinsic = Some((cw, ch));
            }
        }
        // 无 viewBox 比：default object size，无 ratio。
        SvgIntrinsicKind::ComputedIntrinsic(_, _) => {}
        // R2054：NoRatio 保留真实 abs 固有维（仅 abs 属性存在的维，缺失维 None）——旧实现
        // 丢弃为 (None,None) 致 height-25-no-ratio / width-50-no-ratio 失去固有维走全 default。
        // 布局 no_ratio 分支（tree.rs:543）用 w_opt/h_opt + unwrap_or(300/150) 正确分派。
        SvgIntrinsicKind::NoRatio { width, height } => data.no_ratio = Some((width, height)),
    }
    Ok(data)
}

/// SVG `<svg>` 根元素的固有尺寸分类（CSS §10.3.2）。
#[derive(Debug, Clone, Copy, PartialEq)]
enum SvgIntrinsicKind {
    /// width/height 双绝对 → 真固有尺寸（pixmap w/h 有效，走 image_sizes）。
    BothAbs,
    /// 非双绝对 + 有 viewBox → 仅宽高比（走 image_ratios，不设确定 size）。
    RatioOnly(f32),
    /// 非双绝对 + 无 viewBox → 无固有宽高比；`width`/`height` 为真实固有维（仅 abs 属性
    /// 存在的维，缺失维 `None`）。usvg 对缺失维的默认值非真实固有尺寸，不可用于比例推导。
    NoRatio { width: Option<f32>, height: Option<f32> },
    /// 一维 abs + 另一维**缺失或百分比** + viewBox 宽高比 → 有可计算的真实固有尺寸（abs 维
    /// × ratio）。usvg 对缺失维用原始 viewBox 值、对百分比维按默认 viewport 100 解析（pixmap
    /// bogus，如 `height="25" viewBox 1000×500` → pixmap (1000,25) 应 (50,25)），故携带计算值
    /// `(w, h)` 覆盖 pixmap（走 image_sizes）。R3764：百分比维不贡献固有尺寸（css-images-4
    /// default sizing），同缺失维处理。
    ComputedIntrinsic(f32, f32),
}

/// 解析 SVG `<svg>` 根元素属性，分类其固有尺寸类型（CSS §10.3.2）。
///
/// - `BothAbs`：width/height 双绝对（真固有尺寸，走 image_sizes）。
/// - `RatioOnly(ratio)`：非双绝对且 viewBox 提供有效宽高比（ratio = viewBox_w / viewBox_h）。
/// - `NoRatio { width, height }`：非双绝对且无可用 viewBox 比；`width`/`height` 为 abs 属性
///   存在维的真实值（缺失维 `None`）。
/// - `ComputedIntrinsic(w, h)`：一维 abs + 另一维缺失或百分比 + viewBox 比 → 计算的真实
///   固有尺寸（abs × ratio），覆盖 usvg bogus pixmap。
fn svg_intrinsic_kind(bytes: &[u8]) -> SvgIntrinsicKind {
    let Ok(s) = std::str::from_utf8(bytes) else {
        return SvgIntrinsicKind::NoRatio {
            width: None,
            height: None,
        };
    };
    let trimmed = s.trim_start_matches('\u{feff}').trim_start();
    let Some(svg_start) = trimmed.find("<svg") else {
        return SvgIntrinsicKind::NoRatio {
            width: None,
            height: None,
        };
    };
    let after = &trimmed[svg_start..];
    let Some(tag_end) = after.find('>') else {
        return SvgIntrinsicKind::NoRatio {
            width: None,
            height: None,
        };
    };
    let tag = &after[4..tag_end]; // 去掉 "<svg"

    let width = extract_svg_attr(tag, "width");
    let height = extract_svg_attr(tag, "height");
    let viewbox = extract_svg_attr(tag, "viewBox").or_else(|| extract_svg_attr(tag, "viewbox"));

    let w_val = width.as_deref().and_then(parse_abs_length_value);
    let h_val = height.as_deref().and_then(parse_abs_length_value);
    // 两维都绝对 → 真固有尺寸（走 image_sizes 正常路径）
    if w_val.is_some() && h_val.is_some() {
        return SvgIntrinsicKind::BothAbs;
    }
    // 否则优先 ratio-only：从 viewBox 取比（min-x min-y width height）
    if let Some(vb) = viewbox {
        let nums: Vec<&str> = vb.split([' ', ',']).filter(|t| !t.is_empty()).collect();
        if nums.len() == 4
            && let (Ok(vw), Ok(vh)) = (nums[2].parse::<f32>(), nums[3].parse::<f32>())
            && vh > 0.0
        {
            let ratio = vw / vh;
            // 一维 abs + 另一维**缺失或百分比** + viewBox → 计算真实固有尺寸（css-images-4
            // default sizing：百分比属性维不贡献固有尺寸，仅 abs 维 + viewBox 比推导另一维）。
            // 如 `height="25" viewBox 1000×500`（ratio 2）→ (50,25)；R3764：`width="50%"
            // height="32px" viewBox="0 0 4 64"` → (2,32)。旧实现仅「另一维属性缺失」触发，
            // 百分比维误走 RatioOnly → 背景 auto 用 contain-fit 伪尺寸（48×768 应 2×32）。
            // computed_intrinsic 只进 image_sizes 不设 intrinsic_ratio——flex transferred-size
            // 经 image_sizes 的 aspect_ratio 推导（R1438 ratio-derivation 回归路径不复现）。
            if let Some(h) = h_val
                && w_val.is_none()
            {
                return SvgIntrinsicKind::ComputedIntrinsic(h * ratio, h);
            }
            if let Some(w) = w_val
                && h_val.is_none()
            {
                return SvgIntrinsicKind::ComputedIntrinsic(w, w / ratio);
            }
            return SvgIntrinsicKind::RatioOnly(ratio);
        }
    }
    // 非双绝对 + 无可用 viewBox → no-ratio（真实固有维 = abs 属性存在维）
    SvgIntrinsicKind::NoRatio {
        width: w_val,
        height: h_val,
    }
}

/// 从 SVG 起始标签内容中提取 `name="value"` / `name='value'` 属性值。
fn extract_svg_attr(tag: &str, name: &str) -> Option<String> {
    let pat = format!("{name}=");
    let idx = tag.find(&pat)?;
    let after = &tag[idx + pat.len()..];
    let quote = after.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let inner = &after[1..];
    let end = inner.find(quote)?;
    Some(inner[..end].to_string())
}

/// SVG width/height 属性值解析为绝对长度值（正数 + 可选单位，非百分比/auto/none）。
///
/// 返回 `Some(value)` 当且仅当该值是有效正数绝对长度（缺失/百分比/auto/none/非数 → `None`）。
fn parse_abs_length_value(v: &str) -> Option<f32> {
    let v = v.trim();
    if v.is_empty() || v.ends_with('%') {
        return None;
    }
    let lower = v.to_ascii_lowercase();
    if lower == "auto" || lower == "none" {
        return None;
    }
    let num_end = v
        .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-' && c != '+')
        .unwrap_or(v.len());
    let n: f32 = v[..num_end].parse().ok()?;
    (n > 0.0).then_some(n)
}

/// 把 PNG 解码后的原始缓冲（经 EXPAND 后的 RGB/RGBA/Grayscale/GrayscaleAlpha 8-bit）
/// 转换为 RGBA。
fn convert_png_buffer_to_rgba(raw: &[u8], color_type: png::ColorType, bit_depth: png::BitDepth) -> Vec<u8> {
    use png::ColorType::*;
    if bit_depth != png::BitDepth::Eight {
        return raw.to_vec();
    }
    match color_type {
        Rgba => raw.to_vec(),
        Rgb => {
            let n = raw.len() / 3;
            let mut out = Vec::with_capacity(n * 4);
            for px in raw.as_chunks::<3>().0 {
                out.extend_from_slice(&[px[0], px[1], px[2], 255]);
            }
            out
        }
        Grayscale => {
            let mut out = Vec::with_capacity(raw.len() * 4);
            for &g in raw {
                out.extend_from_slice(&[g, g, g, 255]);
            }
            out
        }
        GrayscaleAlpha => {
            let n = raw.len() / 2;
            let mut out = Vec::with_capacity(n * 4);
            for px in raw.as_chunks::<2>().0 {
                out.extend_from_slice(&[px[0], px[0], px[0], px[1]]);
            }
            out
        }
        _ => raw.to_vec(),
    }
}

#[cfg(test)]
mod decode_tests {
    use super::*;

    /// 2×2 纯红 RGBA PNG（手工构造的合法 PNG 字节）。
    fn red_2x2_png() -> Vec<u8> {
        use png::{BitDepth, ColorType, Encoder};
        let mut buf = Vec::new();
        {
            let mut encoder = Encoder::new(&mut buf, 2, 2);
            encoder.set_color(ColorType::Rgba);
            encoder.set_depth(BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();
            // write_image_data 接收原始像素（无行过滤字节，编码器自加）：
            // 2×2×4 = 16 字节，4 个纯红像素。
            let data: Vec<u8> = [255, 0, 0, 255].repeat(4);
            writer.write_image_data(&data).unwrap();
        }
        buf
    }

    #[test]
    fn decode_png_bytes_rgba_2x2() {
        let png = red_2x2_png();
        let img = decode_png_bytes(&png).expect("decode should succeed");
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 2);
        // 左上像素应为纯红
        assert_eq!(img.get_pixel(0, 0), [255, 0, 0, 255]);
    }

    #[test]
    fn decode_png_bytes_invalid_returns_err() {
        let result = decode_png_bytes(b"not a png");
        assert!(result.is_err());
    }

    #[test]
    fn convert_jpeg_pixels_to_rgba_rgb() {
        use jpeg_decoder::PixelFormat;
        // 2 个 RGB 像素：红、绿
        let raw = [255, 0, 0, 0, 255, 0];
        let rgba = convert_jpeg_pixels_to_rgba(&raw, PixelFormat::RGB24);
        assert_eq!(rgba, [255, 0, 0, 255, 0, 255, 0, 255]);
    }

    #[test]
    fn convert_jpeg_pixels_to_rgba_grayscale() {
        use jpeg_decoder::PixelFormat;
        // 2 个灰度像素：128、64
        let raw = [128, 64];
        let rgba = convert_jpeg_pixels_to_rgba(&raw, PixelFormat::L8);
        assert_eq!(rgba, [128, 128, 128, 255, 64, 64, 64, 255]);
    }

    /// 解码真实的 4×3 纯绿 JPEG fixture（quality 95，DCT 在纯色块上近无损）。
    #[test]
    fn decode_jpeg_bytes_green_4x3() {
        let bytes = include_bytes!("testdata/green_4x3.jpg");
        let img = decode_jpeg_bytes(bytes).expect("JPEG decode should succeed");
        assert_eq!(img.width, 4);
        assert_eq!(img.height, 3);
        // JPEG 有损：断言绿色主导（G 高、R/B 低）而非精确 (0,255,0)。
        let px = img.get_pixel(1, 1);
        assert!(px[1] > 200, "green channel should be high, got {}", px[1]);
        assert!(px[0] < 50, "red channel should be low, got {}", px[0]);
        assert!(px[2] < 50, "blue channel should be low, got {}", px[2]);
        assert_eq!(px[3], 255, "alpha should be fully opaque");
    }

    #[test]
    fn decode_jpeg_bytes_invalid_returns_err() {
        // JPEG 魔数 + 无效正文 → 库解码失败
        let result = decode_jpeg_bytes(&[0xFF, 0xD8, 0xFF, 0x00, 0x00]);
        assert!(result.is_err());
    }

    /// R1793：解码真实 4×3 纯绿 WebP fixture（lossless，RGB → 补 alpha=255）。
    #[test]
    fn decode_webp_bytes_green_4x3() {
        let bytes = include_bytes!("testdata/green_4x3.webp");
        let img = decode_webp_bytes(bytes).expect("WebP decode should succeed");
        assert_eq!(img.width, 4);
        assert_eq!(img.height, 3);
        // 纯绿，补 alpha=255
        assert_eq!(&img.pixels[..4], &[0, 255, 0, 255]);
    }

    /// R1793：WebP 魔数（RIFF....WEBP）+ 无效正文 → 库解码失败。
    #[test]
    fn decode_webp_bytes_invalid_returns_err() {
        // RIFF + WEBP 魔数但无有效 VP8/VP8L chunk
        let bad = b"RIFF\x00\x00\x00\x00WEBPrest is garbage";
        let result = decode_webp_bytes(bad);
        assert!(result.is_err());
    }

    /// R1793：`is_webp_magic` 边界（≥12 字节、RIFF+WEBP 双魔数）。
    #[test]
    fn is_webp_magic_detects_riff_webp() {
        assert!(is_webp_magic(b"RIFF\x00\x00\x00\x00WEBPVP8 "));
        assert!(!is_webp_magic(b"RIFF\x00\x00\x00\x00")); // < 12 字节，无 WEBP
        assert!(!is_webp_magic(b"RIFF\x00\x00\x00\x00WAVI")); // RIFF 但非 WEBP
        assert!(!is_webp_magic(b"\x89PNG\r\n\x1a\n")); // PNG 不匹配
    }

    /// 分发器：按魔数字节/内容正确路由 PNG / JPEG / WebP / SVG / 未知格式。
    #[test]
    fn decode_image_bytes_dispatches_by_magic() {
        // PNG → 成功
        let png = red_2x2_png();
        let img = decode_image_bytes(&png).expect("PNG should dispatch and decode");
        assert_eq!(img.width, 2);

        // JPEG → 成功
        let jpeg = include_bytes!("testdata/green_4x3.jpg");
        let img = decode_image_bytes(jpeg).expect("JPEG should dispatch and decode");
        assert_eq!(img.width, 4);

        // WebP → 成功（RIFF....WEBP 魔数路由）
        let webp = include_bytes!("testdata/green_4x3.webp");
        let img = decode_image_bytes(webp).expect("WebP should dispatch and decode");
        assert_eq!(img.width, 4);
        assert_eq!(img.height, 3);

        // SVG → 成功（文本内容嗅探路由）
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"4\" height=\"3\">\
                   <rect width=\"4\" height=\"3\" fill=\"rgb(0,255,0)\"/></svg>";
        let img = decode_image_bytes(svg).expect("SVG should dispatch and rasterize");
        assert_eq!(img.width, 4);
        assert_eq!(img.height, 3);

        // GIF 现受支持（首帧解码——2d.drawImage.animated.gif）；残缺 GIF → 错误。
        let result = decode_image_bytes(b"GIF89a rest of gif");
        assert!(result.is_err(), "残缺 GIF 应报错");

        // 未知魔数 → 错误（unsupported）
        let result = decode_image_bytes(b"UNKNOWNMAGIC123");
        assert!(result.is_err());
        assert!(
            result.unwrap_err().contains("unsupported"),
            "should report unsupported format"
        );
    }

    /// R1705：`decode_data_uri` base64 PNG → ImageData（renderer + wpt-runner 共用入口）。
    #[test]
    fn decode_data_uri_base64_png() {
        use base64::Engine;
        let png = red_2x2_png();
        let b64 = base64::engine::general_purpose::STANDARD.encode(&png);
        let src = format!("data:image/png;base64,{b64}");
        let img = decode_data_uri(&src).expect("base64 PNG data URI should decode");
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 2);
        assert_eq!(&img.pixels[..4], &[255, 0, 0, 255]); // 红
    }

    #[test]
    fn decode_data_uri_bytes_supports_font_payloads() {
        assert_eq!(
            decode_data_uri_bytes("data:font/woff2;BASE64,d09GMg==").unwrap(),
            b"wOF2"
        );
        assert_eq!(
            decode_data_uri_bytes("data:application/font-sfnt,%00%01%00%00").unwrap(),
            [0, 1, 0, 0]
        );
        assert!(decode_data_uri_bytes("https://example.com/font.woff2").is_err());
    }

    /// R1705：无逗号的非法 data URI → Err（调用方降级，不阻断）。
    #[test]
    fn decode_data_uri_missing_comma_is_err() {
        assert!(decode_data_uri("data:image/png;base64").is_err());
    }

    /// 4×3 纯绿 SVG（含 `<?xml` 声明）栅格化往返：断言绿色主导 + alpha=255。
    /// R3996：根 `background-color` 提升为 viewport 级填充——usvg 把根背景画成 viewBox
    /// 区域 path（letterbox 内），chromium 铺满整个 viewport（css-sizing replaced-element-004
    /// 簇根因）。style 声明与 presentation attr 两来源均须剥离；style 仅含背景时整个
    /// style attr 删除。
    #[test]
    fn promote_svg_root_background_fills_viewport() {
        // style 声明来源（replaced-element-004 形态）
        let src = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 5 1" style="background-color: green"></svg>"#;
        let (bytes, bg) = promote_svg_root_background(src.as_bytes());
        assert_eq!(bg, Some([0, 128, 0, 255]));
        let out = String::from_utf8(bytes).unwrap();
        assert!(!out.contains("background-color"), "declaration must be stripped: {out}");
        // 提升后栅格化：letterbox 区（viewBox 5:1 → 100×100 视口的上下 40px）也被填充
        let img = rasterize_svg_at(src.as_bytes(), 100, 100).expect("rasterize");
        assert_eq!(
            img.get_pixel(50, 5)[1],
            128,
            "top letterbox filled by promoted background"
        );
        assert_eq!(
            img.get_pixel(50, 95)[1],
            128,
            "bottom letterbox filled by promoted background"
        );
        assert_eq!(img.get_pixel(50, 50)[1], 128, "viewBox content area still green");

        // presentation attr 来源
        let src2 =
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 5 1" background-color="rgb(0, 128, 0)"></svg>"#;
        let (bytes2, bg2) = promote_svg_root_background(src2.as_bytes());
        assert_eq!(bg2, Some([0, 128, 0, 255]));
        let out2 = String::from_utf8(bytes2).unwrap();
        assert!(!out2.contains("background-color"), "attr must be stripped: {out2}");

        // style 与其他声明混合：只删 background-color 项
        let src3 = r#"<svg viewBox="0 0 5 1" style="background-color: green; opacity: 0.5"></svg>"#;
        let (bytes3, bg3) = promote_svg_root_background(src3.as_bytes());
        assert_eq!(bg3, Some([0, 128, 0, 255]));
        let out3 = String::from_utf8(bytes3).unwrap();
        assert!(out3.contains("opacity: 0.5"), "other decls kept: {out3}");
        assert!(!out3.contains("background-color"));

        // 无背景声明：源不动、返回 None
        let src4 = r#"<svg viewBox="0 0 5 1"><rect width="5" height="1" fill="green"/></svg>"#;
        let (bytes4, bg4) = promote_svg_root_background(src4.as_bytes());
        assert_eq!(bg4, None);
        assert_eq!(bytes4, src4.as_bytes());
    }

    /// R3998：一维 abs attr + viewBox 的 SVG，usvg 栅格化 viewport 不施 viewBox 比
    ///（`width="200"` + viewBox 1:1 → pixmap 200×100 letterbox，两侧透明）。该 bogus
    /// 位图直接进 ImageCache 会把透明 letterbox 拉进元素盒采样（flex-aspect-ratio-
    /// img-row-010/011、img-column-014/015、replaced-element-039/040 红边根因）。
    /// 修复 = 按 ComputedIntrinsic 真实固有尺寸重栅格化，位图与固有尺寸一致。
    #[test]
    fn decode_svg_one_abs_attr_viewbox_rasterizes_at_computed_intrinsic() {
        let src = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" width="200"><rect width="100%" height="100%" fill="green"/></svg>"#;
        let img = decode_svg_bytes(src.as_bytes()).expect("decode");
        // 位图尺寸 = 计算出的真实固有尺寸（200×200），不再是 usvg bogus 200×100
        assert_eq!(img.width, 200, "pixmap must use computed intrinsic width");
        assert_eq!(
            img.height, 200,
            "pixmap must use computed intrinsic height (viewBox ratio)"
        );
        assert_eq!(img.computed_intrinsic(), Some((200.0, 200.0)));
        // 内容无 letterbox：左缘与中央都是绿（旧 200×100 位图左缘透明）
        let left = img.get_pixel(5, img.height / 2);
        let center = img.get_pixel(img.width / 2, img.height / 2);
        assert_eq!(left[1], 128, "left edge must be green (no letterbox): {left:?}");
        assert_eq!(center[1], 128, "center must be green: {center:?}");
        // SVG 源字节保留（放大重栅格化路径仍可用）
        assert!(
            img.svg_source.is_some(),
            "svg_source must be preserved through re-raster"
        );
    }

    #[test]
    fn decode_svg_bytes_green_4x3() {
        let svg = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
                   <svg xmlns=\"http://www.w3.org/2000/svg\" width=\"4\" height=\"3\">\
                   <rect width=\"4\" height=\"3\" fill=\"rgb(0,255,0)\"/></svg>";
        let img = decode_svg_bytes(svg.as_bytes()).expect("SVG rasterize should succeed");
        assert_eq!(img.width, 4);
        assert_eq!(img.height, 3);
        let px = img.get_pixel(1, 1);
        assert!(px[1] > 200, "green channel should be high, got {}", px[1]);
        assert!(px[3] == 255, "alpha should be fully opaque, got {}", px[3]);
    }

    /// R34xx：0 尺寸 SVG（width="0" height="100"——usvg 拒绝解析）→ 兜底提取属性返回
    /// 0×100 空像素 ImageData（2d.pattern.image.zerowidth 期望 createPattern null）。
    #[test]
    fn decode_svg_bytes_zero_width_fallback() {
        let svg = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"0\" height=\"100\">\
                   <rect fill=\"red\" width=\"100\" height=\"100\"/></svg>";
        let img = decode_svg_bytes(svg.as_bytes()).expect("zero-width SVG should decode");
        assert_eq!(img.width, 0);
        assert_eq!(img.height, 100);
        // 缓冲按存储维 (max(w,1)×max(h,1)) 分配（from_rgba 长度校验）。
        assert_eq!(img.pixels.len(), 1 * 100 * 4);
    }

    /// R34xx：extract_svg_attr_dims 属性提取（px 后缀 / 单引号 / 缺失任一 → None）。
    #[test]
    fn extract_svg_attr_dims_parses_root_attrs() {
        assert_eq!(
            extract_svg_attr_dims(b"<svg xmlns=\"x\" width=\"0\" height=\"100px\"></svg>"),
            Some((0, 100))
        );
        assert_eq!(
            extract_svg_attr_dims(b"<svg width='10' height='20'></svg>"),
            Some((10, 20))
        );
        assert_eq!(extract_svg_attr_dims(b"<svg width=\"10\"></svg>"), None);
        assert_eq!(extract_svg_attr_dims(b"<div></div>"), None);
        assert_eq!(extract_svg_attr_dims(b"not xml"), None);
    }

    #[test]
    fn decode_svg_bytes_invalid_returns_err() {
        // 非 SVG XML → resvg 解析失败
        let result = decode_svg_bytes(b"<not-a-svg></not-a-svg>");
        assert!(result.is_err());
    }

    /// 绝对 width/height 的 SVG → 真固有尺寸 → BothAbs（走 image_sizes 路径）。
    #[test]
    fn svg_kind_absolute_dims_is_both_abs() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"50\" viewBox=\"0 0 200 100\">\
                   <rect width=\"100\" height=\"50\" fill=\"green\"/></svg>";
        assert_eq!(svg_intrinsic_kind(svg), SvgIntrinsicKind::BothAbs);
    }

    /// 百分比 width/height + viewBox → ratio-only，ratio = viewBox w/h。
    #[test]
    fn svg_kind_percent_dims_uses_viewbox() {
        // aspect-ratio-intrinsic-size-007 驱动案：100%×100% + viewBox 7500×3750 → ratio 2.0
        let svg =
            b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100%\" height=\"100%\" viewBox=\"0 0 7500 3750\">\
                   <rect width=\"7500\" height=\"3750\" fill=\"green\"/></svg>";
        assert_eq!(svg_intrinsic_kind(svg), SvgIntrinsicKind::RatioOnly(2.0));
    }

    /// 仅 viewBox 无 width/height → ratio-only。
    #[test]
    fn svg_kind_viewbox_only() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1000 500\">\
                   <rect width=\"1000\" height=\"500\" fill=\"green\"/></svg>";
        assert_eq!(svg_intrinsic_kind(svg), SvgIntrinsicKind::RatioOnly(2.0));
    }

    /// 一维百分比、一维绝对 → ComputedIntrinsic（R3764：百分比维不贡献固有尺寸，
    /// abs 维 + viewBox 比 → 计算真实固有尺寸 (100, 50)）。
    #[test]
    fn svg_kind_mixed_percent_and_absolute() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100%\" height=\"50\" viewBox=\"0 0 200 100\">\
                   <rect width=\"200\" height=\"100\" fill=\"green\"/></svg>";
        assert_eq!(
            svg_intrinsic_kind(svg),
            SvgIntrinsicKind::ComputedIntrinsic(100.0, 50.0)
        );
    }

    /// 无 width/height 也无 viewBox → no-ratio，无任何固有维。
    #[test]
    fn svg_kind_no_dims_no_viewbox_is_no_ratio() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\"><rect fill=\"green\"/></svg>";
        assert_eq!(
            svg_intrinsic_kind(svg),
            SvgIntrinsicKind::NoRatio {
                width: None,
                height: None
            }
        );
    }

    /// 仅 width 绝对、无 height、无 viewBox → no-ratio，真实固有宽 = width 值，高 None。
    /// 驱动案：visudet width-50-no-ratio.svg（CSS §10.3.2 no-ratio replaced sizing）。
    #[test]
    fn svg_kind_width_only_no_ratio() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"50\" preserveAspectRatio=\"none\">\
                   <rect fill=\"orange\" width=\"100%\" height=\"100%\"/></svg>";
        assert_eq!(
            svg_intrinsic_kind(svg),
            SvgIntrinsicKind::NoRatio {
                width: Some(50.0),
                height: None
            }
        );
    }

    /// 仅 height 绝对、无 width、无 viewBox → no-ratio，真实固有高 = height 值，宽 None。
    /// 驱动案：visudet height-25-no-ratio.svg。
    #[test]
    fn svg_kind_height_only_no_ratio() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" height=\"25\" preserveAspectRatio=\"none\">\
                   <rect fill=\"blue\" width=\"100%\" height=\"100%\"/></svg>";
        assert_eq!(
            svg_intrinsic_kind(svg),
            SvgIntrinsicKind::NoRatio {
                width: None,
                height: Some(25.0)
            }
        );
    }

    /// R1438：height abs + width 缺失 + viewBox → ComputedIntrinsic（abs×ratio）。
    /// 驱动案：visudet height-25-ratio-2.svg（height="25" viewBox 1000×500，ratio 2 → (50,25)）。
    #[test]
    fn svg_kind_height_abs_viewbox_computed_intrinsic() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1000 500\" height=\"25\" preserveAspectRatio=\"none\">\
                   <rect fill=\"fuchsia\" width=\"1000\" height=\"500\"/></svg>";
        assert_eq!(svg_intrinsic_kind(svg), SvgIntrinsicKind::ComputedIntrinsic(50.0, 25.0));
    }

    /// R1438：width abs + height 缺失 + viewBox → ComputedIntrinsic（abs, abs/ratio）。
    /// 驱动案：visudet width-50-ratio-2.svg（width="50" viewBox 1000×500，ratio 2 → (50,25)）。
    #[test]
    fn svg_kind_width_abs_viewbox_computed_intrinsic() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1000 500\" width=\"50\" preserveAspectRatio=\"none\">\
                   <rect fill=\"silver\" width=\"1000\" height=\"500\"/></svg>";
        assert_eq!(svg_intrinsic_kind(svg), SvgIntrinsicKind::ComputedIntrinsic(50.0, 25.0));
    }

    /// R3764：一维 abs + 另一维**百分比**（属性存在）+ viewBox → ComputedIntrinsic
    ///（css-images-4 default sizing：百分比属性维不贡献固有尺寸，abs 维 + viewBox 比
    /// 推导另一维；`width="100%" height="50" viewBox="0 0 200 100"` → (100, 50)）。
    /// 旧 R1438 gate 判 RatioOnly（避 flex ratio-derivation 回归）——computed_intrinsic
    /// 只进 image_sizes 不设 intrinsic_ratio，flex 经 image_sizes aspect_ratio 推导，回归
    /// 路径不复现；driving：WPT vector-023 / tall-viewbox 混合型 auto sizing。
    #[test]
    fn svg_kind_mixed_percent_abs_viewbox_computed_intrinsic() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100%\" height=\"50\" viewBox=\"0 0 200 100\">\
                   <rect width=\"200\" height=\"100\" fill=\"green\"/></svg>";
        assert_eq!(
            svg_intrinsic_kind(svg),
            SvgIntrinsicKind::ComputedIntrinsic(100.0, 50.0)
        );
    }

    /// R3764：镜像方向（width abs + height 百分比）+ viewBox → ComputedIntrinsic
    ///（`width="8px" height="50%" viewBox="0 0 4 64"`（ratio 1/16）→ (8, 128)）。
    #[test]
    fn svg_kind_width_abs_percent_height_viewbox_computed_intrinsic() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"8px\" height=\"50%\" viewBox=\"0 0 4 64\" preserveAspectRatio=\"none\">\
                   <rect width=\"4\" height=\"64\" fill=\"lime\"/></svg>";
        assert_eq!(svg_intrinsic_kind(svg), SvgIntrinsicKind::ComputedIntrinsic(8.0, 128.0));
    }

    /// R3764：双百分比维 + viewBox 仍 RatioOnly（两维都不贡献固有尺寸，仅 viewBox 比）。
    #[test]
    fn svg_kind_both_percent_dims_viewbox_still_ratio_only() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100%\" height=\"50%\" viewBox=\"0 0 200 100\">\
                   <rect width=\"200\" height=\"100\" fill=\"green\"/></svg>";
        assert_eq!(svg_intrinsic_kind(svg), SvgIntrinsicKind::RatioOnly(2.0));
    }

    /// 端到端：百分比维 + viewBox SVG（RatioOnly）经 decode 保留 intrinsic_ratio（供 flex
    /// transferred-size；INLINE 由布局层 default object size 处理，不应用此 ratio）。
    #[test]
    fn decode_svg_percent_dims_keeps_intrinsic_ratio() {
        let svg =
            b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100%\" height=\"100%\" viewBox=\"0 0 7500 3750\">\
                   <rect width=\"7500\" height=\"3750\" fill=\"green\"/></svg>";
        let img = decode_svg_bytes(svg).expect("SVG rasterize should succeed");
        assert_eq!(img.intrinsic_ratio(), Some(2.0));
        assert_eq!(img.no_ratio_intrinsic(), None);
    }

    /// R2054：width-only no-ratio SVG 经 decode 后 no_ratio **保留真实 abs 固有维**
    ///（Some(50), None）——旧实现丢弃为 (None,None) 致 visudet width-50-no-ratio.svg
    /// 失去固有宽走全 default（应 50×150）。布局 no_ratio 分支用 w_opt/h_opt + unwrap_or
    /// 正确分派（显式 width 时 width=50，auto height 时 default 150）。
    #[test]
    fn decode_svg_width_only_no_ratio_sets_field() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"50\" preserveAspectRatio=\"none\">\
                   <rect fill=\"orange\" width=\"100%\" height=\"100%\"/></svg>";
        let img = decode_svg_bytes(svg).expect("SVG rasterize should succeed");
        assert_eq!(img.intrinsic_ratio(), None);
        assert_eq!(img.no_ratio_intrinsic(), Some((Some(50.0), None)));
    }

    /// R2054：height abs + viewBox ComputedIntrinsic SVG 经 decode → **computed_intrinsic**
    ///（→ image_sizes），使 INLINE auto+auto 用 (50,25)（chromium visudet all-auto：
    /// height-25-ratio-2 → 50×25）。旧实现设 intrinsic_ratio 致 layout ratio-only 分支
    /// INLINE 走 default 300×150（错）。computed_intrinsic 不再设 intrinsic_ratio（flex
    /// transferred-size 经 image_sizes aspect_ratio 推导）。
    #[test]
    fn decode_svg_height_abs_viewbox_sets_computed_intrinsic() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1000 500\" height=\"25\" preserveAspectRatio=\"none\">\
                   <rect fill=\"fuchsia\" width=\"1000\" height=\"500\"/></svg>";
        let img = decode_svg_bytes(svg).expect("SVG rasterize should succeed");
        assert_eq!(img.computed_intrinsic(), Some((50.0, 25.0)));
        assert_eq!(img.intrinsic_ratio(), None);
        assert_eq!(img.no_ratio_intrinsic(), None);
    }

    /// 端到端：双绝对 SVG（width+height 均 abs）经 decode 后**不**进 no_ratio（走 image_sizes
    /// 固有尺寸路径），intrinsic_ratio/no_ratio 均 None。
    #[test]
    fn decode_svg_both_abs_not_no_ratio() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"50\" height=\"25\" preserveAspectRatio=\"none\">\
                   <rect fill=\"black\" width=\"100%\" height=\"100%\"/></svg>";
        let img = decode_svg_bytes(svg).expect("SVG rasterize should succeed");
        assert_eq!(img.intrinsic_ratio(), None);
        assert_eq!(img.no_ratio_intrinsic(), None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_image(w: u32, h: u32, fill: u8) -> ImageData {
        let pixels = vec![fill; (w as usize) * (h as usize) * 4];
        ImageData::from_rgba(pixels, w, h).unwrap()
    }

    /// R3762：无 width/height 属性的 SVG 注入目标 viewport——属性插入且原字节保留。
    #[test]
    fn r3762_inject_target_dims_into_attrless_svg() {
        let src = b"<svg xmlns=\"http://www.w3.org/2000/svg\"><rect width=\"50%\" fill=\"lime\"/></svg>";
        let out = inject_svg_target_dims(src, 420, 344);
        let text = std::str::from_utf8(&out).unwrap();
        assert!(text.contains("<svg height=\"344\" width=\"420\" xmlns="), "got: {text}");
        assert!(text.contains("<rect width=\"50%\""), "内容保留");
    }

    /// R3762：% 值 width 属性被替换为目标的 abs px。
    #[test]
    fn r3762_inject_replaces_percent_attr() {
        let src = b"<svg width=\"50%\" height='32px' xmlns=\"x\"><rect/></svg>";
        let out = inject_svg_target_dims(src, 256, 768);
        let text = std::str::from_utf8(&out).unwrap();
        assert!(text.contains("width=\"256\""), "got: {text}");
        assert!(text.contains("height=\"768\""), "got: {text}");
    }

    /// R3762：双 abs SVG 不注入（真固有 viewport 语义）。
    #[test]
    fn r3762_inject_skips_both_abs_svg() {
        let src = b"<svg width=\"8px\" height=\"32px\" xmlns=\"x\"><rect/></svg>";
        let out = inject_svg_target_dims(src, 420, 344);
        assert_eq!(out.as_slice(), &src[..]);
    }

    /// R3762：端到端——无尺寸 SVG 栅格化到目标尺寸后 viewport = 目标（50% 宽 rect
    /// 占据目标宽一半），而非 usvg 默认 100×100 的 50%。
    #[test]
    fn r3762_rasterize_at_uses_target_viewport() {
        let src =
            b"<svg xmlns=\"http://www.w3.org/2000/svg\"><rect width=\"50%\" height=\"100%\" fill=\"#ff0000\"/></svg>";
        let img = rasterize_svg_at(src, 400, 200).expect("rasterize");
        assert_eq!((img.width, img.height), (400, 200));
        // x=300（>50% 宽）应透明，x=100（<50%）应为红。
        assert_eq!(img.get_pixel(100, 100), [255, 0, 0, 255]);
        assert_eq!(img.get_pixel(300, 100), [0, 0, 0, 0]);
    }

    /// R3760：WPT vector 簇 `nonpercent-width-omitted-height.svg`（width="8px"，
    /// height 缺失，无 viewBox）应分类 NoRatio 且保留真实固有宽 8——painter
    /// §3.9 逐维解析依赖该维值（auto = 8×定位区高，非全定位区）。
    #[test]
    fn r3760_width_only_svg_no_ratio_keeps_dim() {
        let bytes = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"8px\"><rect width=\"100%\" height=\"50%\" fill=\"lime\"/></svg>";
        let img = decode_image_bytes(bytes).expect("decode");
        assert_eq!(img.no_ratio_intrinsic(), Some((Some(8.0), None)));
        assert_eq!(img.intrinsic_ratio(), None);
    }

    /// R3760：零维 viewBox（`viewBox="0 0 8 0"`）退化 SVG → 1×1 全透明（css-images-4
    /// §5.4.1：固有宽/高为 0 的图像视为空），且无 ratio 信号——usvg 对缺 width/height
    /// 属性的此类 SVG 会按默认 100×100 栅格化出实心内容，WPT 期望空。
    #[test]
    fn r3760_zero_viewbox_svg_renders_transparent() {
        let bytes = b"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 8 0\"><rect width=\"100%\" height=\"100%\" fill=\"lime\"/></svg>";
        let img = decode_image_bytes(bytes).expect("decode");
        assert_eq!((img.width, img.height), (1, 1));
        assert!(img.pixels.iter().all(|&b| b == 0), "1x1 应全透明");
        assert_eq!(img.no_ratio_intrinsic(), Some((None, None)));
        assert_eq!(img.intrinsic_ratio(), None);
    }

    #[test]
    fn test_image_data_from_rgba() {
        let pixels = vec![255u8; 2 * 2 * 4];
        let img = ImageData::from_rgba(pixels, 2, 2).unwrap();
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 2);
        assert_eq!(img.get_pixel(0, 0), [255, 255, 255, 255]);
    }

    #[test]
    fn test_image_data_from_rgba_wrong_size() {
        let pixels = vec![255u8; 10];
        let result = ImageData::from_rgba(pixels, 2, 2);
        assert!(result.is_err());
    }

    #[test]
    fn test_image_data_new_empty() {
        let img = ImageData::new_empty(4, 4);
        assert_eq!(img.pixels.len(), 4 * 4 * 4);
        assert_eq!(img.get_pixel(0, 0), [0, 0, 0, 0]);
    }

    #[test]
    fn test_image_data_size() {
        let img = ImageData::new_empty(10, 20);
        let size = img.size();
        assert_eq!(size.width, 10.0);
        assert_eq!(size.height, 20.0);
    }

    #[test]
    fn test_image_data_byte_size() {
        let img = ImageData::new_empty(3, 4);
        assert_eq!(img.byte_size(), 3 * 4 * 4);
    }

    #[test]
    fn test_cache_insert_and_get() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key = cache.insert(make_image(2, 2, 255));
        assert_eq!(cache.len(), 1);

        let img = cache.get(&key).unwrap();
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 2);
        assert_eq!(cache.ref_count(&key), Some(2)); // insert gives 1, get adds 1
    }

    #[test]
    fn test_cache_release() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key = cache.insert(make_image(2, 2, 128));
        assert_eq!(cache.ref_count(&key), Some(1));

        cache.release(&key);
        assert_eq!(cache.ref_count(&key), Some(0));
    }

    #[test]
    fn test_cache_gc_removes_zero_ref() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key = cache.insert(make_image(2, 2, 100));
        cache.release(&key);
        assert_eq!(cache.ref_count(&key), Some(0));

        cache.gc();
        assert!(cache.ref_count(&key).is_none());
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_remove_deletes_only_requested_image() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let removed = cache.insert(make_image(2, 2, 100));
        let retained = cache.insert(make_image(1, 1, 200));

        assert!(cache.remove(&removed));
        assert!(!cache.remove(&removed));
        assert!(cache.ref_count(&removed).is_none());
        assert!(cache.ref_count(&retained).is_some());
        assert_eq!(cache.total_bytes(), 4);
    }

    #[test]
    fn test_cache_gc_keeps_referenced() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key = cache.insert(make_image(2, 2, 200));
        // ref_count is 1, should be kept
        cache.gc();
        assert!(cache.ref_count(&key).is_some());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_cache_gc_evicts_by_lru_when_over_max_entries() {
        let mut cache = ImageCache::new(2, 1024 * 1024);
        let _key1 = cache.insert(make_image(1, 1, 10)); // gen 0
        let _key2 = cache.insert(make_image(1, 1, 20));
        let _key3 = cache.insert(make_image(1, 1, 30)); // triggers over max_entries

        cache.gc();
        assert!(cache.len() <= 2);
    }

    #[test]
    fn test_cache_gc_evicts_by_lru_when_over_max_bytes() {
        let mut cache = ImageCache::new(100, 32); // 32 bytes max
        let key1 = cache.insert(make_image(2, 2, 10)); // 16 bytes
        let _key2 = cache.insert(make_image(2, 2, 20)); // 16 bytes = total 32
        assert_eq!(cache.total_bytes(), 32);

        // Access key1 so it's newer
        let _ = cache.get(&key1);

        // Insert another, total > 32
        let _key3 = cache.insert(make_image(2, 2, 30)); // total = 48
        cache.gc();
        assert!(cache.total_bytes() <= 32);
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        cache.insert(make_image(1, 1, 0));
        cache.insert(make_image(1, 1, 1));
        assert_eq!(cache.len(), 2);

        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_default() {
        let cache = ImageCache::default();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_get_nonexistent() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key = ImageKey::new(999);
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_cache_release_nonexistent_is_noop() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key = ImageKey::new(999);
        // Should not panic
        cache.release(&key);
    }

    #[test]
    fn test_cache_generation_increases_on_gc() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        assert_eq!(cache.generation(), 0);
        cache.gc();
        assert_eq!(cache.generation(), 1);
        cache.gc();
        assert_eq!(cache.generation(), 2);
    }

    #[test]
    fn test_cache_total_bytes() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        cache.insert(make_image(2, 2, 0)); // 2*2*4 = 16 bytes
        cache.insert(make_image(3, 3, 0)); // 3*3*4 = 36 bytes
        assert_eq!(cache.total_bytes(), 52);
    }

    #[test]
    fn test_image_key_new() {
        let key = ImageKey::new(42);
        assert_eq!(key.0, 42);
    }

    #[test]
    fn test_image_data_get_pixel_clamps_out_of_bounds() {
        let pixels = vec![255u8, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255];
        let img = ImageData::from_rgba(pixels, 2, 2).unwrap();
        // 双线性采样在右/下边界可能传入 x=width 或 y=height，应钳制而非 panic。
        assert_eq!(img.get_pixel(2, 0), [0, 255, 0, 255]);
        assert_eq!(img.get_pixel(0, 2), [0, 0, 255, 255]);
        assert_eq!(img.get_pixel(99, 99), [255, 255, 0, 255]);
    }

    #[test]
    fn test_image_data_get_pixel_mismatched_buffer_returns_transparent() {
        let img = ImageData {
            pixels: vec![255; 8],
            width: 2,
            height: 2,
            content_hash: 0,
            solid_color: None,
            intrinsic_ratio: None,
            no_ratio: None,
            computed_intrinsic: None,
            svg_source: None,
        };
        assert_eq!(img.get_pixel(0, 0), [0, 0, 0, 0]);
    }

    #[test]
    fn test_image_data_get_pixel_various_positions() {
        // 2x2 image with distinct pixel values
        let pixels: Vec<u8> = vec![
            255, 0, 0, 255, // (0,0) red
            0, 255, 0, 255, // (1,0) green
            0, 0, 255, 255, // (0,1) blue
            255, 255, 0, 255, // (1,1) yellow
        ];
        let img = ImageData::from_rgba(pixels, 2, 2).unwrap();
        assert_eq!(img.get_pixel(0, 0), [255, 0, 0, 255]);
        assert_eq!(img.get_pixel(1, 0), [0, 255, 0, 255]);
        assert_eq!(img.get_pixel(0, 1), [0, 0, 255, 255]);
        assert_eq!(img.get_pixel(1, 1), [255, 255, 0, 255]);
    }

    #[test]
    fn test_image_key_equality_and_hash() {
        let k1 = ImageKey::new(10);
        let k2 = ImageKey::new(10);
        let k3 = ImageKey::new(20);
        assert_eq!(k1, k2);
        assert_ne!(k1, k3);
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(k1.clone());
        set.insert(k2.clone());
        assert_eq!(set.len(), 1);
        set.insert(k3.clone());
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_cache_multiple_get_increments_ref_count() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key = cache.insert(make_image(2, 2, 100));
        assert_eq!(cache.ref_count(&key), Some(1));

        let _ = cache.get(&key);
        assert_eq!(cache.ref_count(&key), Some(2));

        let _ = cache.get(&key);
        assert_eq!(cache.ref_count(&key), Some(3));
    }

    #[test]
    fn test_cache_release_saturating_sub() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key = cache.insert(make_image(1, 1, 0));
        // ref_count starts at 1
        cache.release(&key);
        assert_eq!(cache.ref_count(&key), Some(0));
        // Releasing below 0 should saturate at 0
        cache.release(&key);
        assert_eq!(cache.ref_count(&key), Some(0));
    }

    #[test]
    fn test_cache_clear_increases_generation() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        assert_eq!(cache.generation(), 0);
        cache.insert(make_image(1, 1, 0));
        cache.clear();
        assert_eq!(cache.generation(), 1);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_sequential_insert_unique_keys() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let k1 = cache.insert(make_image(1, 1, 10));
        let k2 = cache.insert(make_image(1, 1, 20));
        let k3 = cache.insert(make_image(1, 1, 30));
        assert_ne!(k1, k2);
        assert_ne!(k2, k3);
        assert_eq!(cache.len(), 3);
        // Each has ref_count 1
        assert_eq!(cache.ref_count(&k1), Some(1));
        assert_eq!(cache.ref_count(&k2), Some(1));
        assert_eq!(cache.ref_count(&k3), Some(1));
    }

    #[test]
    fn test_image_data_from_rgba_large() {
        let pixels = vec![128u8; 100 * 100 * 4];
        let img = ImageData::from_rgba(pixels, 100, 100).unwrap();
        assert_eq!(img.byte_size(), 100 * 100 * 4);
        assert_eq!(img.get_pixel(50, 50), [128, 128, 128, 128]);
    }

    /// 测试 LRU 淘汰顺序：先插入（最旧世代）的条目应被优先淘汰
    #[test]
    fn test_cache_gc_lru_eviction_order() {
        let mut cache = ImageCache::new(2, 1024 * 1024);
        let key1 = cache.insert(make_image(1, 1, 10)); // gen 0
        let key2 = cache.insert(make_image(1, 1, 20)); // gen 0
        // 推进世代后插入 key3，使 key1/key2 有更旧的 last_access_gen
        cache.gc(); // gen -> 1，key1 和 key2 的 last_access_gen 仍为 0
        let key3 = cache.insert(make_image(1, 1, 30)); // last_access_gen = 1

        cache.gc(); // gen -> 2，应淘汰 gen=0 的条目（key1 和 key2 之一）
        assert_eq!(cache.len(), 2);
        // key3 最新（gen=1），一定保留
        assert!(cache.ref_count(&key3).is_some(), "最新的 key3 应保留");
        // key1 和 key2 同为 gen=0，淘汰其中一个即可
        let remaining = cache.ref_count(&key1).is_some() as usize + cache.ref_count(&key2).is_some() as usize;
        assert_eq!(remaining, 1, "key1 和 key2 中应恰好保留一个");
    }

    /// 测试多次 GC 后访问时间更新使条目免于被淘汰
    #[test]
    fn test_cache_gc_get_updates_lru_and_protects_entry() {
        let mut cache = ImageCache::new(2, 1024 * 1024);
        let key1 = cache.insert(make_image(1, 1, 10));
        let key2 = cache.insert(make_image(1, 1, 20));

        // 先执行一次 GC 使世代推进到 1
        cache.gc();
        // 此时 key1 和 key2 的 last_access_gen 仍为 0

        // 访问 key1 使其 last_access_gen 更新为 1
        let _ = cache.get(&key1);
        // key2 的 last_access_gen 仍为 0（更旧）

        // 插入第三个，触发超限
        let key3 = cache.insert(make_image(1, 1, 30)); // gen=1

        cache.gc(); // 世代推进到 2
        assert_eq!(cache.len(), 2);
        // key1 被访问过（gen=1），应保留；key2 最旧（gen=0），应被淘汰
        assert!(cache.ref_count(&key1).is_some(), "被访问过的 key1 应保留");
        assert!(cache.ref_count(&key2).is_none(), "未被访问的 key2 应被淘汰");
        assert!(cache.ref_count(&key3).is_some(), "key3 应保留");
    }

    /// 测试混合 release + get 模式下 GC 的正确性
    #[test]
    fn test_cache_gc_mixed_release_and_get_pattern() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key1 = cache.insert(make_image(1, 1, 10));
        let key2 = cache.insert(make_image(1, 1, 20));
        let key3 = cache.insert(make_image(1, 1, 30));

        // key1: release → ref_count=0（应被 GC 移除）
        cache.release(&key1);
        // key2: get → ref_count=2（应保留）
        let _ = cache.get(&key2);
        // key3: 保持 ref_count=1（应保留）

        cache.gc();
        assert!(cache.ref_count(&key1).is_none(), "ref_count=0 的 key1 应被移除");
        assert_eq!(cache.ref_count(&key2), Some(2), "key2 ref_count 应为 2");
        assert_eq!(cache.ref_count(&key3), Some(1), "key3 ref_count 应为 1");
        assert_eq!(cache.len(), 2);
    }

    /// 测试超大图片超过 max_bytes 时被淘汰
    #[test]
    fn test_cache_gc_single_image_exceeds_max_bytes() {
        let mut cache = ImageCache::new(10, 16); // 仅允许 16 字节
        let _key1 = cache.insert(make_image(1, 1, 10)); // 4 字节
        let _key2 = cache.insert(make_image(1, 1, 20)); // 4 字节
        assert_eq!(cache.total_bytes(), 8);

        // 插入一个 4x4（64 字节）的图片，远超 max_bytes
        let _key_big = cache.insert(make_image(4, 4, 255)); // 64 字节
        assert!(cache.total_bytes() > 16);

        cache.gc();
        // GC 应不断淘汰最旧条目直到总字节数 ≤ max_bytes
        // 即使淘汰所有旧条目后只剩 key_big (64 > 16)，也会继续淘汰
        // 最终 key_big 也会被淘汰（因为 64 > 16）
        assert!(cache.total_bytes() <= 16, "GC 后总字节数应不超过 max_bytes");
    }

    // -- 边界条件测试 --
    /// 测试 ImageKey 边界值
    #[test]
    fn test_image_key_boundary_values() {
        let k_min = ImageKey::new(0);
        assert_eq!(k_min.0, 0);
        let k_max = ImageKey::new(u64::MAX);
        assert_eq!(k_max.0, u64::MAX);
        assert_ne!(k_min, k_max);
    }

    /// 测试 ImageData::new_empty 零尺寸
    #[test]
    fn test_image_data_new_empty_zero_dims() {
        let img = ImageData::new_empty(0, 0);
        assert_eq!(img.width, 0);
        assert_eq!(img.height, 0);
        assert!(img.pixels.is_empty());
    }

    /// 测试 ImageData::from_rgba 零宽度
    #[test]
    fn test_image_data_from_rgba_zero_width() {
        let result = ImageData::from_rgba(vec![], 0, 5);
        assert!(result.is_ok());
        let img = result.unwrap();
        assert_eq!(img.width, 0);
        assert_eq!(img.height, 5);
        assert!(img.pixels.is_empty());
    }

    /// 测试 ImageData get_pixel 1x1 图片
    #[test]
    fn test_image_data_get_pixel_1x1() {
        let pixels = vec![42, 84, 126, 255];
        let img = ImageData::from_rgba(pixels, 1, 1).unwrap();
        assert_eq!(img.get_pixel(0, 0), [42, 84, 126, 255]);
    }

    /// 测试 ImageCache gc 后再 insert
    #[test]
    fn test_cache_insert_after_gc() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let k1 = cache.insert(make_image(1, 1, 100));
        cache.release(&k1);
        cache.gc();
        assert!(cache.is_empty());

        let k2 = cache.insert(make_image(1, 1, 200));
        assert_eq!(cache.ref_count(&k2), Some(1));
        assert_eq!(cache.len(), 1);
    }

    /// 测试 ImageCache release 然后 get（ref_count 从 0 回到 1）
    #[test]
    fn test_cache_release_then_get() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key = cache.insert(make_image(2, 2, 128));
        cache.release(&key);
        assert_eq!(cache.ref_count(&key), Some(0));

        // get 仍然能取回数据（条目还在，只是 ref_count=0）
        let img = cache.get(&key);
        assert!(img.is_some());
        assert_eq!(cache.ref_count(&key), Some(1));
    }

    /// 测试 ImageCache gc 在空缓存上
    #[test]
    fn test_cache_gc_empty() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        assert!(cache.is_empty());
        assert_eq!(cache.generation(), 0);
        cache.gc();
        assert!(cache.is_empty());
        assert_eq!(cache.generation(), 1);
    }

    /// 测试 ImageData::new_empty 所有像素透明
    #[test]
    fn test_image_data_new_empty_all_transparent() {
        let img = ImageData::new_empty(2, 2);
        assert_eq!(img.get_pixel(0, 0), [0, 0, 0, 0]);
        assert_eq!(img.get_pixel(1, 0), [0, 0, 0, 0]);
        assert_eq!(img.get_pixel(0, 1), [0, 0, 0, 0]);
        assert_eq!(img.get_pixel(1, 1), [0, 0, 0, 0]);
    }

    /// 测试 ImageCache max_entries=1
    #[test]
    fn test_cache_max_entries_one() {
        let mut cache = ImageCache::new(1, 1024 * 1024);
        let k1 = cache.insert(make_image(1, 1, 10));
        // gc to advance generation so k1 has older gen
        cache.gc();
        let k2 = cache.insert(make_image(1, 1, 20));
        // Now len = 2 > max_entries = 1
        cache.gc();
        // Should evict oldest entry, keeping only 1
        assert!(cache.len() <= 1);
        assert!(cache.ref_count(&k2).is_some(), "较新的 k2 应保留");
        assert!(cache.ref_count(&k1).is_none(), "较旧的 k1 应被淘汰");
    }

    /// 测试连续插入相同内容的数据产生不同的缓存键，缓存中两个条目并存。
    ///
    /// 对 ImageCache 调用两次 insert 传入相同像素数据，应返回不同的 key，
    /// 缓存中两个条目独立存在，可通过各自的 key 分别访问。
    #[test]
    fn test_image_cache_double_insert_same_key() {
        let mut cache = ImageCache::new(10, 1024 * 1024);

        let data1 = make_image(2, 2, 128);
        let data2 = make_image(2, 2, 128); // 相同尺寸和填充值

        let key1 = cache.insert(data1);
        let key2 = cache.insert(data2);

        // 两次插入应返回不同的 key
        assert_ne!(key1, key2, "两次插入应返回不同的 key");
        assert_eq!(cache.len(), 2, "缓存中应有 2 个条目");

        // 两个 key 都能独立获取
        let img1 = cache.get(&key1);
        assert!(img1.is_some(), "key1 应能获取到图片");
        assert_eq!(img1.unwrap().width, 2);

        let img2 = cache.get(&key2);
        assert!(img2.is_some(), "key2 应能获取到图片");
        assert_eq!(img2.unwrap().width, 2);

        // 引用计数各被增加（insert=1 + get=1 = 2）
        assert_eq!(cache.ref_count(&key1), Some(2));
        assert_eq!(cache.ref_count(&key2), Some(2));

        // 释放 key1 后 GC，key1 被移除，key2 保留
        cache.release(&key1);
        cache.release(&key1); // release the extra ref from get
        cache.gc();
        assert!(cache.ref_count(&key1).is_none(), "key1 应被 GC 移除");
        assert!(cache.ref_count(&key2).is_some(), "key2 应保留");
    }

    /// 测试 max_entries=0 时 GC 会清除所有条目
    ///
    /// 当 max_entries 设置为 0 时，每次 GC 都会淘汰所有条目，
    /// 因为任何条目数（>0）都超过限制。insert 本身不会拒绝插入，
    /// 但 GC 后缓存一定为空。
    #[test]
    fn test_image_cache_zero_max_entries() {
        let mut cache = ImageCache::new(0, 1024 * 1024);

        // 插入多个条目
        let k1 = cache.insert(make_image(1, 1, 10));
        let k2 = cache.insert(make_image(2, 2, 20));
        let k3 = cache.insert(make_image(3, 3, 30));
        assert_eq!(cache.len(), 3, "插入后应有 3 个条目");

        // GC 后所有条目因 max_entries=0 被淘汰
        cache.gc();
        assert!(cache.is_empty(), "GC 后缓存应为空");
        assert_eq!(cache.len(), 0);
        assert!(cache.ref_count(&k1).is_none(), "k1 应被淘汰");
        assert!(cache.ref_count(&k2).is_none(), "k2 应被淘汰");
        assert!(cache.ref_count(&k3).is_none(), "k3 应被淘汰");

        // 再次插入后立即 GC，同样被淘汰
        let k4 = cache.insert(make_image(1, 1, 40));
        cache.gc();
        assert!(cache.is_empty(), "再次 GC 后缓存仍应为空");
        assert!(cache.ref_count(&k4).is_none());
    }

    /// 测试 ImageData::from_rgba 使用 0x0 尺寸的空数据创建成功
    ///
    /// 当 width=0、height=0 时，expected 字节数为 0，
    /// 传入空 Vec 应成功创建一个零尺寸图片。
    /// 验证零尺寸图片的 size() 返回 (0,0)，byte_size() 返回 0。
    #[test]
    fn test_image_data_from_rgba_zero_dimensions() {
        let img = ImageData::from_rgba(vec![], 0, 0).expect("0x0 应创建成功");
        assert_eq!(img.width, 0);
        assert_eq!(img.height, 0);
        assert!(img.pixels.is_empty());
        assert_eq!(img.byte_size(), 0);

        let size = img.size();
        assert_eq!(size.width, 0.0);
        assert_eq!(size.height, 0.0);
    }

    /// 测试 GC 在 max_bytes=0 时清除所有条目
    ///
    /// 当最大字节限制为 0 时，任何非零大小的图片都会被 GC 淘汰。
    #[test]
    fn test_cache_gc_zero_max_bytes() {
        let mut cache = ImageCache::new(100, 0);
        let k1 = cache.insert(make_image(1, 1, 10));
        assert_eq!(cache.len(), 1);

        cache.gc();
        assert!(cache.is_empty(), "max_bytes=0 时 GC 应淘汰所有条目");
        assert!(cache.ref_count(&k1).is_none());
    }

    /// 测试释放所有条目后 GC 清空缓存
    ///
    /// 所有条目 ref_count 降为 0 后，GC 应移除全部。
    #[test]
    fn test_cache_gc_after_release_all() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let k1 = cache.insert(make_image(1, 1, 10));
        let k2 = cache.insert(make_image(2, 2, 20));
        let k3 = cache.insert(make_image(3, 3, 30));

        cache.release(&k1);
        cache.release(&k2);
        cache.release(&k3);

        cache.gc();
        assert!(cache.is_empty(), "释放全部引用后 GC 应清空缓存");
    }

    /// 测试插入 1x1 图片后 total_bytes 正确
    ///
    /// 1x1 RGBA 图片占用 4 字节。
    #[test]
    fn test_cache_total_bytes_single_pixel() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        cache.insert(make_image(1, 1, 255));
        assert_eq!(cache.total_bytes(), 4, "1x1 RGBA 图片应为 4 字节");
    }

    /// 测试多次 clear 后 generation 持续递增
    #[test]
    fn test_cache_clear_generation_multiple() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        assert_eq!(cache.generation(), 0);
        cache.clear();
        assert_eq!(cache.generation(), 1);
        cache.clear();
        assert_eq!(cache.generation(), 2);
        cache.clear();
        assert_eq!(cache.generation(), 3);
    }
}

// R3935 隔离验证：usvg 0.47 对 transform-origin presentation attribute 的支持语义。
#[test]
fn r3935_usvg_transform_origin_attr_semantics() {
    // rect 150x150 位于 (75,75)，viewBox/viewport 200x200。
    // A 版：SVG2 attr 形式。
    let a = rasterize_svg_at(
        br##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200" viewBox="0 0 200 200"><rect x="75" y="75" width="150" height="150" fill="#00ff00" transform="rotate(90)" transform-origin="150px 75px"/></svg>"##,
        100, 100,
    ).expect("rasterize A");
    // B 版：等价手写 transform（绕 origin (150,75) 旋转 90°）。
    let b = rasterize_svg_at(
        br##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200" viewBox="0 0 200 200"><rect x="75" y="75" width="150" height="150" fill="#00ff00" transform="translate(150 75) rotate(90) translate(-150 -75)"/></svg>"##,
        100, 100,
    ).expect("rasterize B");
    let diff = a.pixels.iter().zip(b.pixels.iter()).filter(|(x, y)| x != y).count();
    println!("R3935: pixel-byte diffs between attr-form and manual-form = {diff}");
    assert_eq!(diff, 0, "usvg 应把 transform-origin attr 等价合成为手写 transform");
}

#[test]
fn r3935b_usvg_transform_origin_keyword_semantics() {
    // 关键字 "center right"（= origin x=150,y=75 对 200x200 viewport 中的 150 rect at 75,75
    // ——center of x（75+150/2=150），right? "center right" = y center? CSS 语法：第一词水平
    // 或垂直。SVG2 transform-origin 关键字相对 reference box。此处比较 attr 关键字版与
    // 手写等价版；不等价即 usvg 关键字解析有缺陷。
    let a = rasterize_svg_at(
        br##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200"><rect x="75" y="75" width="150" height="150" fill="#00ff00" transform="rotate(90)" transform-origin="center right"/></svg>"##,
        100, 100,
    ).expect("A");
    let b = rasterize_svg_at(
        br##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200"><rect x="75" y="75" width="150" height="150" fill="#00ff00" transform="translate(150 150) rotate(90) translate(-150 -150)"/></svg>"##,
        100, 100,
    ).expect("B");
    let diff = a.pixels.iter().zip(b.pixels.iter()).filter(|(x, y)| x != y).count();
    println!("R3935b: keyword-form vs manual(center,center) diffs = {diff}");
    // center right 对 viewport 200x200：x=right(200)? y=center(100)？或相对 bbox？
    // 不做硬断言，先观测——打印两版中心像素颜色供归因。
    let pa = |x: usize, y: usize| {
        let i = (y * 100 + x) * 4;
        (a.pixels[i], a.pixels[i + 1], a.pixels[i + 2])
    };
    let pb = |x: usize, y: usize| {
        let i = (y * 100 + x) * 4;
        (b.pixels[i], b.pixels[i + 1], b.pixels[i + 2])
    };
    println!("R3935b: A center={:?} B center={:?}", pa(50, 50), pb(50, 50));
}

#[cfg(test)]
mod r3936_tests {
    use super::*;

    /// R3936：关键字 origin 按 rect bbox 改写（"center right" 对 rect(75,75,150,150)
    /// → (225,150)）；纯 px 数值透传（usvg 用户空间绝对坐标 = view-box 语义）。
    #[test]
    fn r3936_preprocess_rewrites_keyword_origin() {
        let kw = br##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200"><rect x="75" y="75" width="150" height="150" transform="rotate(90)" transform-origin="center right"/></svg>"##;
        let out = preprocess_svg_transform_origin(kw);
        let text = std::str::from_utf8(&out).unwrap();
        assert!(
            text.contains(r#"transform-origin="225px 150px""#),
            "关键字 origin 应按 rect bbox 改写为 px 值: {text}"
        );
        // 纯数值：透传不翻。
        let num = br##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200"><rect x="100" width="100" height="100" transform="rotate(90)" transform-origin="100px 0"/></svg>"##;
        assert_eq!(
            preprocess_svg_transform_origin(num),
            num.to_vec(),
            "纯数值 origin 应透传（usvg 用户空间绝对坐标）"
        );
    }

    /// R3936：百分比/词序变体/单垂直关键字/非法回落/无 attr 不变（bbox 参照）。
    #[test]
    fn r3936_preprocess_variants() {
        let rect = br##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200"><rect x="0" y="0" width="200" height="200" transform-origin="%ORIGIN%"/></svg>"##;
        let with_origin = |o: &str| -> String {
            let src = String::from_utf8_lossy(rect).replacen("%ORIGIN%", o, 1);
            String::from_utf8_lossy(&preprocess_svg_transform_origin(src.as_bytes())).into_owned()
        };
        // 百分比（bbox=viewport 200×200）：50% → 100px。
        assert!(with_origin("50%").contains(r#"transform-origin="100px 100px""#));
        // 词序交换：top right → h=right(200), v=top(0)。
        assert!(with_origin("top right").contains(r#"transform-origin="200px 0px""#));
        // 单水平关键字 left → x=0, y=center=100。
        assert!(with_origin("left").contains(r#"transform-origin="0px 100px""#));
        // 单垂直关键字 top → x=center=100, y=0（018/019 案形态）。
        assert!(with_origin("top").contains(r#"transform-origin="100px 0px""#));
        assert!(with_origin("bottom").contains(r#"transform-origin="100px 200px""#));
        // 非法组合（top 100%/left left，invalid 簇形态）→ attr 删除（声明忽略，
        // usvg 缺省 pivot = viewport 中心，与 chromium 忽略无效 attr 一致）。
        assert!(!with_origin("top 100%").contains("transform-origin"));
        assert!(!with_origin("left left").contains("transform-origin"));
        // 无该 attr：原字节不变。
        let e = preprocess_svg_transform_origin(br##"<rect fill="red"/>"##);
        assert_eq!(e, br##"<rect fill="red"/>"##);
    }

    /// R3936：单值数字 origin（"75"，svg-origin-relative-length-001/012/013 案形态）
    /// → 「值 + bbox 垂直中心」双值 px（usvg 单值 Y 缺省 0 是缺陷，CSS 语义第二轴
    /// = center）；未知单位（"2cm"，svg-origin-length-{cm,in,pt} 案形态）→ 原样透传。
    #[test]
    fn r3936_preprocess_single_value_and_unknown_units() {
        let src = br##"<svg xmlns="http://www.w3.org/2000/svg"><rect width="150" height="150" transform="rotate(90)" transform-origin="75"/></svg>"##;
        let out = preprocess_svg_transform_origin(src);
        assert!(
            String::from_utf8_lossy(&out).contains(r#"transform-origin="75px 75px""#),
            "单值数字应翻成「值+bbox中心」双值: {}",
            String::from_utf8_lossy(&out)
        );
        // 单值数字 + 非 0 bbox 原点（x=20,y=10）：x = 20+75, y = 10+75/2。
        let src2 = br##"<svg xmlns="http://www.w3.org/2000/svg"><rect x="20" y="10" width="75" height="75" transform-origin="75"/></svg>"##;
        let out2 = preprocess_svg_transform_origin(src2);
        assert!(String::from_utf8_lossy(&out2).contains(r#"transform-origin="95px 47.5px""#));
        // 未知物理单位：透传（usvg 自行解析，view-box 簇实证正确）。
        let cm = br##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200"><rect width="150" height="150" transform="rotate(90)" transform-origin="2cm 0"/></svg>"##;
        assert_eq!(preprocess_svg_transform_origin(cm), cm.to_vec());
    }

    /// R3936：单值数字翻成双值后端到端渲染与 ref 期望一致（012 案精确形态——
    /// transform="rotate(90) translate(-75,-75)" origin="0" 期望 (0,75)）。
    #[test]
    fn r3936_single_value_render_matches_ref_expectation() {
        let wrap = |inner: &str| {
            format!(
                r##"<svg xmlns="http://www.w3.org/2000/svg"><defs><linearGradient id="grad" x2="0%" y2="100%"><stop offset="50%" stop-color="orange"/><stop offset="50%" stop-color="fuchsia"/></linearGradient></defs><rect x="1" y="1" width="148" height="148" fill="red"/>{inner}</svg>"##
            )
        };
        let ref_svg = r##"<svg xmlns="http://www.w3.org/2000/svg"><defs><linearGradient id="grad"><stop offset="50%" stop-color="fuchsia"/><stop offset="50%" stop-color="orange"/></linearGradient></defs><rect width="150" height="150" fill="url(#grad)"/></svg>"##;
        let r = rasterize_svg_at(ref_svg.as_bytes(), 200, 200).expect("ref");
        let t = rasterize_svg_at(
            wrap(r##"<rect width="150" height="150" fill="url(#grad)" transform="rotate(90) translate(-75,-75)" transform-origin="0"/>"##).as_bytes(),
            200, 200,
        )
        .expect("test");
        let diff = t
            .pixels
            .chunks(4)
            .zip(r.pixels.chunks(4))
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(diff, 0, "预处理后单值 origin 渲染应与 ref 期望一致（diff={diff}）");
    }

    /// R3936 隔离：usvg 0.47 px origin 参照系（可区分对照——origin (100,100) 对
    /// rect bbox 中点 (150,150) 与 viewport 中心 (100,100)）。
    #[test]
    fn r3936c_usvg_px_origin_reference_frame() {
        let svg = |extra: &str| -> ImageData {
            rasterize_svg_at(
                format!(r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200"><rect x="75" y="75" width="150" height="150" fill="#00ff00" {extra}/></svg>"##).as_bytes(),
                100,
                100,
            )
            .expect("rasterize")
        };
        let attr = svg(r#"transform="rotate(90)" transform-origin="100px 100px""#);
        let vp_manual = svg(r#"transform="translate(100 100) rotate(90) translate(-100 -100)""#);
        let bb_manual = svg(r#"transform="translate(150 150) rotate(90) translate(-150 -150)""#);
        let d = |x: &ImageData, y: &ImageData| x.pixels.iter().zip(y.pixels.iter()).filter(|(a, b)| a != b).count();
        println!(
            "R3936c: attr-vs-viewport(100,100)={} attr-vs-bbox-center(150,150)={}",
            d(&attr, &vp_manual),
            d(&attr, &bb_manual)
        );
    }

    /// R3936：端到端——预处理后关键字 origin（042 案形态）渲染与手写
    /// 等价链一致（origin 绝对位 = bbox 右缘中点 (225,150)）。
    #[test]
    fn r3936_keyword_origin_render_matches_manual_after_preprocess() {
        let attr_form = rasterize_svg_at(
            br##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200"><rect x="75" y="75" width="150" height="150" fill="#00ff00" transform="rotate(90)" transform-origin="center right"/></svg>"##,
            100, 100,
        ).expect("attr form");
        let manual = rasterize_svg_at(
            br##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200"><rect x="75" y="75" width="150" height="150" fill="#00ff00" transform="translate(225 150) rotate(90) translate(-225 -150)"/></svg>"##,
            100, 100,
        ).expect("manual form");
        let diff = attr_form
            .pixels
            .iter()
            .zip(manual.pixels.iter())
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(diff, 0, "预处理后关键字 origin 渲染应与手写等价链一致（diff={diff}）");
    }
}

#[cfg(test)]
mod r3937_tests {
    use super::*;

    /// R3937：style attr 的合法 CSS transform 覆盖 presentation attr
    ///（inline-styles-001 形态：rotate(90deg) 胜 scale(0.5)）。
    #[test]
    fn r3937_style_attr_overrides_transform_attr() {
        let src = br##"<svg xmlns="http://www.w3.org/2000/svg"><rect width="100" height="100" transform="scale(0.5)" style="transform: rotate(90deg)"/></svg>"##;
        let out = preprocess_svg_style_transform(src);
        let text = std::str::from_utf8(&out).unwrap();
        assert!(
            text.contains(r#"transform="rotate(90)""#),
            "style transform 应覆盖 attr: {text}"
        );
    }

    /// R3937：非法 CSS transform → 声明忽略，attr 生效（005/013 形态）。
    #[test]
    fn r3937_invalid_css_keeps_attr() {
        let src = br##"<svg xmlns="http://www.w3.org/2000/svg"><rect width="100" height="100" transform="rotate(90)" style="transform: scale(invalid)"/></svg>"##;
        assert_eq!(preprocess_svg_style_transform(src), src.to_vec());
    }

    /// R3937：translate/skew/多函数串 + 无 style 透传。
    #[test]
    fn r3937_function_forms() {
        let multi = br##"<svg xmlns="http://www.w3.org/2000/svg"><rect width="100" height="100" transform="scale(0.5)" style="transform: translate(20px, 20px) rotate(90deg) translate(-20px, -20px)"/></svg>"##;
        let out_bytes = preprocess_svg_style_transform(multi);
        let text = std::str::from_utf8(&out_bytes).unwrap();
        assert!(
            text.contains(r#"transform="translate(20 20) rotate(90) translate(-20 -20)""#),
            "多函数串应翻译: {text}"
        );
        // translateY 单函数。
        let ty = br##"<svg xmlns="http://www.w3.org/2000/svg"><rect transform="rotate(90)" style="transform: translateY(-100px)"/></svg>"##;
        let out_bytes2 = preprocess_svg_style_transform(ty);
        let text2 = std::str::from_utf8(&out_bytes2).unwrap();
        assert!(text2.contains(r#"transform="translate(0 -100)""#));
        // 无 style attr：原样。
        let none = br##"<svg xmlns="http://www.w3.org/2000/svg"><rect transform="scale(0.5)"/></svg>"##;
        assert_eq!(preprocess_svg_style_transform(none), none.to_vec());
    }

    /// R3988：transform attr 畸形参数列表（尾随逗号）→ attr 删除（rotate-3args-
    /// invalid-002 案语义：`rotate(90,)` 整条无效）；合法值不动。
    #[test]
    fn r3988_malformed_transform_attr_dropped() {
        let src = br##"<svg xmlns="http://www.w3.org/2000/svg"><rect width="80" height="80" transform="rotate(90,)"/></svg>"##;
        let out = preprocess_svg_transform_attr_syntax(src);
        let text = std::str::from_utf8(&out).unwrap();
        assert!(!text.contains("transform="), "尾随逗号 transform attr 应删除: {text}");
        // 合法值不动。
        let ok = br##"<svg xmlns="http://www.w3.org/2000/svg"><rect width="80" height="80" transform="rotate(90) translate(0 -100)"/></svg>"##;
        assert_eq!(preprocess_svg_transform_attr_syntax(ok), ok.to_vec());
        // 端到端：畸形 attr 删除后渲染 = 无 transform 形态。
        let with = rasterize_svg_at(
            br##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200"><rect width="80" height="80" fill="#00ff00" transform="rotate(90,)"/></svg>"##,
            200, 200,
        ).expect("w");
        let none = rasterize_svg_at(
            br##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200"><rect width="80" height="80" fill="#00ff00"/></svg>"##,
            200, 200,
        ).expect("n");
        let diff = with
            .pixels
            .chunks(4)
            .zip(none.pixels.chunks(4))
            .filter(|(p, q)| p != q)
            .count();
        assert_eq!(diff, 0, "畸形 attr 删除后应与无 transform 一致（diff={diff}）");
    }

    /// R3937：端到端——覆盖后渲染与「attr=rotate(90)」形态一致（001 案语义）。
    #[test]
    fn r3937_style_override_render_matches() {
        let a = rasterize_svg_at(
            br##"<svg xmlns="http://www.w3.org/2000/svg" width="300" height="300"><rect y="-100" width="100" height="100" fill="#00ff00" transform="scale(0.5)" style="transform: rotate(90deg)"/></svg>"##,
            300, 300,
        ).expect("a");
        let expect = rasterize_svg_at(
            br##"<svg xmlns="http://www.w3.org/2000/svg" width="300" height="300"><rect y="-100" width="100" height="100" fill="#00ff00" transform="rotate(90)"/></svg>"##,
            300, 300,
        ).expect("expect");
        let diff = a
            .pixels
            .chunks(4)
            .zip(expect.pixels.chunks(4))
            .filter(|(p, q)| p != q)
            .count();
        assert_eq!(diff, 0, "预处理后 style 覆盖应与纯 attr rotate 一致（diff={diff}）");
    }
}
