//! Reftest 字体加载 —— 构造加载了系统字体、CJK 回退与 `@font-face` 自定义字体的 FontLoader。
//!
//! reftest 截图的可信度依赖字体栈与真实浏览器一致：系统 sans（DejaVu/Liberation）+
//! CJK 回退（Noto Sans CJK）+ WPT 标准 Ahem 字体 + 文档声明的 `@font-face`。

use std::path::Path;

use zero_css_parser::ast::Rule as CssRule;
use zero_css_parser::parser::Parser as CssParser;
use zero_render_foundation::font::loader::FontLoader;

/// 创建加载了系统字体和 Ahem 测试字体的 FontLoader。
///
/// 加载顺序：
/// 1. 系统字体（DejaVu/Liberation 系列）
/// 2. Ahem 测试字体（WPT 标准测试字体，每个字符渲染为实心方块）
///
/// base 字体集解析昂贵（19MB CJK ttc `from_bytes` ~0.5s/次）；reftest 单测
/// 每测试调用一次，测试进程内用 OnceLock 只解析一次，之后 `duplicate()` 复用
/// （字体数据 Arc 共享，见 FontLoader::duplicate 注释）。
pub(super) fn create_font_loader() -> FontLoader {
    static CACHED: std::sync::OnceLock<FontLoader> = std::sync::OnceLock::new();
    CACHED.get_or_init(build_base_font_loader).duplicate()
}

/// 构建 base 字体集（系统 + CJK 回退 + Ahem）。
fn build_base_font_loader() -> FontLoader {
    let mut loader = FontLoader::new();
    let mut fallback_ids: Vec<u32> = Vec::new();

    // R1259：先加载 Liberation Serif 作为 FontId(0)（initial font-family 的解析目标）。
    // 原因：chromium 的 initial font-family 为 "Times New Roman"，经 fontconfig 在本环境
    // 解析为 Liberation Serif（fc-match "Times New Roman" → LiberationSerif-Regular.ttf，
    // Times-metric-compatible serif，细字干）。ZeroWeb 旧版 FontId(0)=DejaVuSans（sans 宽字干），
    // 致无显式 font-family 的描述性文本（WPT reftest 的 <p> 指示文本）用 DejaVuSans 渲染，
    // 而 chromium 用 Liberation Serif，产生纯字体匹配差异（R1257 证 float-width cluster 的
    // 4.85% diff 全在此 <p> 文本，非布局 bug）。R1257 试 NotoSansCJK（sans，错方向）net -1pp；
    // Liberation Serif（serif，CHR 真实默认）是正确匹配。Ahem 测试元素显式 font-family:Ahem
    // 不受影响；sans-serif/serif 显式声明经 build_font_resolver 仍各自映射，不受影响。
    let primary_serif_paths = [
        "/usr/share/fonts/truetype/liberation/LiberationSerif-Regular.ttf",
        "/System/Library/Fonts/Supplemental/Times New Roman.ttf",
    ];
    for path in &primary_serif_paths {
        if let Ok(data) = std::fs::read(path) {
            let _ = loader.load_font(&data);
            break;
        }
    }

    // 系统字体路径（Linux / macOS）作回退
    let system_font_paths = [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans.ttf",
        "/System/Library/Fonts/Supplemental/Arial.ttf",
        "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
    ];

    for path in &system_font_paths {
        if let Ok(data) = std::fs::read(path)
            && let Ok(id) = loader.load_font(&data)
            && fallback_ids.is_empty()
        {
            // 第一个成功加载的系统字体（DejaVuSans）加入回退链，供主字体缺字形时回退
            fallback_ids.push(id);
        }
    }

    // 加载 CJK 字体（Noto Sans CJK）并加入回退链——主字体缺 CJK 字形时回退到此，
    // 使中文/日文/韩文字符可渲染（DC-13 welcome.html 等含 CJK 文本的真实页面）。
    let cjk_font_paths = [
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
    ];
    for path in &cjk_font_paths {
        if let Ok(data) = std::fs::read(path) {
            if let Ok(id) = loader.load_font(&data) {
                fallback_ids.push(id);
            }
            break;
        }
    }

    // 加载 Ahem 测试字体（WPT reftest 标准字体）
    let ahem_path = "tests/wpt-runner/fonts/Ahem.ttf";
    if let Ok(data) = std::fs::read(ahem_path) {
        let _ = loader.load_font(&data);
    }

    if !fallback_ids.is_empty() {
        loader.set_fallback_chain(fallback_ids);
    }

    loader
}

#[cfg(all(test, target_os = "macos"))]
mod macos_tests {
    use super::*;

    #[test]
    fn default_loader_includes_macos_system_fonts() {
        let loader = create_font_loader();
        let resolver = loader.build_font_resolver();

        assert!(loader.len() >= 4);
        assert!(resolver.contains_key("Arial"));
        assert!(resolver.contains_key("sans-serif"));
    }
}

type FontFaceSpec = (
    String,
    Vec<String>,
    Option<u16>,
    bool,
    zero_css_parser::values::FontFeatureSettingsValue,
);

/// 从 CSS 文本中提取所有 `@font-face` 规则的 face 列表。
///
/// 用 `zero_css_parser` 解析样式表，收集 `Rule::FontFace`（含 R2417 `weight` + R2493 `is_italic`）。
/// 解析失败或无规则时返回空。
pub(super) fn extract_font_faces(css: &str) -> Vec<FontFaceSpec> {
    use zero_css_parser::values::types::FontStyleValue;
    let stylesheet = CssParser::parse_stylesheet(css);
    stylesheet
        .rules
        .iter()
        .filter_map(|rule| match rule {
            CssRule::FontFace(ff) => {
                let is_italic = matches!(
                    ff.style,
                    Some(FontStyleValue::Italic) | Some(FontStyleValue::Oblique(_))
                );
                Some((
                    ff.family.clone(),
                    ff.sources.clone(),
                    ff.weight,
                    is_italic,
                    ff.feature_settings.clone(),
                ))
            }
            _ => None,
        })
        .collect()
}

/// 提取 HTML 中所有 `<style>` 元素的文本内容（与 engine `collect_stylesheets` 同源）。
///
/// `@font-face` 常声明在文档内联 `<style>`（非外链 CSS），须一并扫描才能加载。
pub(super) fn extract_inline_style_css(html: &str) -> String {
    let doc = zero_dom::parse_html(html);
    let mut css = String::new();
    for style_id in doc.get_elements_by_tag_name("style") {
        if let Some(text) = doc.text_content(style_id) {
            let text = text.trim();
            // 去 CDATA 包裹（XHTML 惯例 `<![CDATA[ ... ]]>`）
            let text = text
                .strip_prefix("<![CDATA[")
                .and_then(|t| t.strip_suffix("]]>"))
                .map(|t| t.trim())
                .unwrap_or(text);
            if !text.is_empty() {
                if !css.is_empty() {
                    css.push('\n');
                }
                css.push_str(text);
            }
        }
    }
    css
}

/// 解析 `@font-face` 的 src URL 到本地文件路径（与 `load_linked_stylesheets` 同约定）。
///
/// - `/abs` → `tests/wpt-runner/wpt-data/<abs>`
/// - 相对路径 → `base_dir.join(rel)`
/// - `data:`/`http(s):` → None（本地不可读）
pub(super) fn resolve_font_src(href: &str, base_dir: Option<&Path>) -> Option<std::path::PathBuf> {
    if href.starts_with("data:") || href.starts_with("http://") || href.starts_with("https://") {
        return None;
    }
    let path = if href.starts_with('/') {
        Path::new("tests/wpt-runner/wpt-data").join(href.trim_start_matches('/'))
    } else {
        base_dir?.join(href)
    };
    Some(path)
}

/// 把 CSS 中 `@font-face` 声明的自定义字体加载进 FontLoader。
///
/// 对每个 face，按 src 顺序尝试解析到本地文件并 `load_font`；首个成功加载的源即注册
/// （fontdue 解码 .ttf/.otf；.woff 需解压，当前 fontdue 不支持 woff 容器，会静默失败并
/// 跳到下一个 src）。加载后 `build_font_resolver` 即可按 family 匹配到该字体。
pub(super) fn load_font_faces_into(loader: &mut FontLoader, base_dir: Option<&Path>, css: &str) {
    for (family, sources, weight, is_italic, feature_settings) in extract_font_faces(css) {
        // Ahem 由 FontLoader 特殊处理（按 family 名合成方块），无需加载文件
        if family.eq_ignore_ascii_case("Ahem") {
            continue;
        }
        for src in &sources {
            let Some(path) = resolve_font_src(src, base_dir) else {
                continue;
            };
            if let Ok(data) = std::fs::read(&path)
                && let Ok(id) = loader.load_font(&data)
            {
                loader.register_font_features(id, zero_engine::font_feature_settings_to_opentype(&feature_settings));
                // R2417/R2493（镜像生产 drain）：按 (weight, style) 构注册键——bold+italic →
                // `{family}:700:italic`、bold → `{family}:700`、italic → `{family}:italic`、
                // regular → plain。bold/italic face 不注册 plain family（避 build_font_resolver
                // 「second face=bold」启发式顺序错配，R2417）。使 harness 多 weight/style 同族
                // @font-face 匹配与生产一致、顺序无关。
                let want_bold = weight.is_some_and(|w| w >= 600);
                let key = match (want_bold, is_italic) {
                    (true, true) => format!("{family}:700:italic"),
                    (true, false) => format!("{family}:700"),
                    (false, true) => format!("{family}:italic"),
                    (false, false) => family.clone(),
                };
                loader.register_family_alias(&key, id);
                break;
            }
        }
    }
}

#[cfg(test)]
mod feature_tests {
    use super::*;
    use zero_render_foundation::font::TextDirection;

    #[test]
    fn font_face_feature_settings_register_on_loaded_face() {
        let fonts_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("wpt-data/fonts");
        let css = r#"@font-face {
            font-family: FeatureLato;
            src: url(Lato-Medium-Liga.ttf);
            font-feature-settings: "liga" off;
        }"#;
        let faces = extract_font_faces(css);
        assert_eq!(faces.len(), 1, "descriptor must remain in parsed font face");
        assert!(
            resolve_font_src(&faces[0].1[0], Some(&fonts_dir))
                .as_deref()
                .is_some_and(Path::exists),
            "font source must resolve"
        );
        let mut loader = FontLoader::new();
        load_font_faces_into(&mut loader, Some(&fonts_dir), css);
        let resolver = loader.build_font_resolver();
        let font_id = *resolver.get("FeatureLato").expect("feature face alias");

        let glyphs = loader
            .shape_text_cached_with_features(font_id, "fi", 16.0, TextDirection::LeftToRight, &[])
            .expect("shape with face defaults");
        assert_eq!(glyphs.len(), 2, "face-level liga=off must suppress fi ligature");
    }

    #[test]
    fn ordered_font_ids_shape_missing_glyph_with_secondary_face() {
        let fonts_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("wpt-data/css/css-fonts/resources");
        let css = r#"
            @font-face { font-family: PrimaryAhem; src: url(ahem-ex-500.otf); }
            @font-face { font-family: SecondaryAhem; src: url(ahem-ex-250.otf); }
        "#;
        let mut loader = FontLoader::new();
        load_font_faces_into(&mut loader, Some(&fonts_dir), css);
        let resolver = loader.build_font_resolver();
        let primary = *resolver.get("PrimaryAhem").expect("primary alias");
        let secondary = *resolver.get("SecondaryAhem").expect("secondary alias");

        let glyphs = loader
            .shape_text_cached_with_font_ids(&[primary, secondary], "xA", 100.0, TextDirection::LeftToRight, &[])
            .expect("shape ordered fallback");

        assert_eq!(glyphs.len(), 2);
        assert_eq!(glyphs[0].font_id.0, primary);
        assert_eq!(glyphs[1].font_id.0, secondary);
        assert_eq!(glyphs.iter().map(|glyph| glyph.cluster).collect::<Vec<_>>(), vec![0, 1]);

        let reversed = loader
            .shape_text_cached_with_font_ids(&[secondary, primary], "xA", 100.0, TextDirection::LeftToRight, &[])
            .expect("shape reversed fallback");
        assert_eq!(reversed[0].font_id.0, secondary, "ordered cache keys must not collide");
    }
}
