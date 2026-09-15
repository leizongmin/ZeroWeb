//! 跨 browser / renderer / compositor 进程一致的系统字体加载。

use std::path::PathBuf;

use super::loader::FontLoader;

/// 已加载的平台字体集合。
pub struct PlatformFonts {
    /// 字体加载器。
    pub loader: FontLoader,
    /// 主 UI / sans 字体 ID。
    pub primary_id: Option<u32>,
}

/// 按跨进程稳定顺序加载 primary、bold 与 fallback 字体。
pub fn load_platform_fonts() -> PlatformFonts {
    let mut loader = FontLoader::new();
    let primary = load_first(&mut loader, primary_font_paths(), 0, "primary");
    let Some(primary_id) = primary else {
        tracing::warn!("No platform primary font found; text rendering will be limited");
        return PlatformFonts {
            loader,
            primary_id: None,
        };
    };

    let _ = load_first(&mut loader, bold_font_paths(), 0, "bold");

    // R4377：generic 族 face 与 reftest harness 对齐（R1259 serif / R1263 sans / R4373 mono
    // 谱系——chromium fontconfig 默认：serif/initial = Times New Roman → Liberation Serif、
    // sans-serif = Arial → Liberation Sans、monospace = Noto Sans Mono CJK SC）。生产侧此前
    // 不载这些 face：`font-family: serif/monospace` 经 build_font_resolver 的通用族名匹配
    // 全部落 default（NotoSans）——与 chromium 渲染分歧（monospace 文本宽 ~20% 等）。
    // 仅加载为 family 成员（family_map 供名匹配），**不进 fallback chain**（缺字回退行为
    // 零变化）；加载顺序在 primary/bold 之后、CJK chain 之前（各进程同序 → font_id 一致）。
    for path in generic_family_font_paths() {
        match std::fs::read(path) {
            Ok(data) => {
                if let Err(error) = loader.load_font(&data) {
                    tracing::debug!(path, %error, "Failed to load generic family font");
                }
            }
            Err(error) => tracing::debug!(path, %error, "Generic family font not found"),
        }
    }

    let mut fallback_ids = Vec::new();
    for (path, face_index) in fallback_font_candidates() {
        let Ok(data) = std::fs::read(&path) else {
            continue;
        };
        match loader.load_font_at_index(&data, face_index) {
            Ok(id) if id != primary_id => {
                tracing::info!(
                    path = %path.display(),
                    face_index,
                    font_id = id,
                    "Loaded platform fallback font"
                );
                fallback_ids.push(id);
            }
            Ok(_) => {}
            Err(error) => tracing::debug!(path = %path.display(), %error, "Failed to load fallback font"),
        }
        // R4373 谱系：monospace 通用族 face——NotoSansCJK ttc face 7 = "Noto Sans Mono CJK SC"
        // （fc-match monospace 别名真身，ch = 0.5em）。face 7 存在性随字体配置而异：失败
        // 静默跳过，mono_names 兜底照旧（与 reftest harness 同款）。
        if path.extension().and_then(|e| e.to_str()) == Some("ttc")
            && let Err(error) = loader.load_font_at_index(&data, 7)
        {
            tracing::debug!(path = %path.display(), %error, "No monospace face in ttc");
        }
    }
    loader.set_fallback_chain(fallback_ids);
    if loader.fallback_chain().is_empty() {
        tracing::warn!("No CJK fallback font found; set ZW_CJK_FONT_PATH or ZW_CJK_FONT_DIR before starting ZeroWeb");
    }

    PlatformFonts {
        loader,
        primary_id: Some(primary_id),
    }
}

fn load_first(loader: &mut FontLoader, paths: &[&str], face_index: u32, role: &str) -> Option<u32> {
    paths.iter().find_map(|path| {
        let data = std::fs::read(path).ok()?;
        let id = loader.load_font_at_index(&data, face_index).ok()?;
        tracing::info!(path, font_id = id, "Loaded platform {role} font");
        Some(id)
    })
}

fn fallback_font_candidates() -> Vec<(PathBuf, u32)> {
    let configured_index = std::env::var("ZW_CJK_FACE_INDEX")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(2);
    let mut candidates = Vec::new();
    if let Some(path) = std::env::var_os("ZW_CJK_FONT_PATH") {
        candidates.push((PathBuf::from(path), configured_index));
    }
    if let Some(directory) = std::env::var_os("ZW_CJK_FONT_DIR").map(PathBuf::from) {
        candidates.push((directory.join("NotoSansCJK-Regular.ttc"), configured_index));
    }
    candidates.extend(platform_fallback_font_paths().iter().map(|path| {
        let face_index = if path.ends_with("NotoSansCJK-Regular.ttc") {
            2
        } else {
            0
        };
        (PathBuf::from(path), face_index)
    }));
    deduplicate_paths(candidates)
}

fn deduplicate_paths(candidates: Vec<(PathBuf, u32)>) -> Vec<(PathBuf, u32)> {
    let mut result = Vec::new();
    for candidate in candidates {
        if !result.iter().any(|(path, _)| path == &candidate.0) {
            result.push(candidate);
        }
    }
    result
}

fn primary_font_paths() -> &'static [&'static str] {
    #[cfg(target_os = "macos")]
    {
        &[
            "/System/Library/Fonts/SFNS.ttf",
            "/System/Library/Fonts/SFCompact.ttf",
            "/System/Library/Fonts/HelveticaNeue.ttc",
            "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
            "/System/Library/Fonts/Helvetica.ttc",
        ]
    }
    #[cfg(target_os = "windows")]
    {
        &["C:\\Windows\\Fonts\\segoeui.ttf", "C:\\Windows\\Fonts\\arial.ttf"]
    }
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        &[
            "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
            "/usr/share/fonts/opentype/noto/NotoSans-Regular.ttf",
            "/usr/share/fonts/opentype/cantarell/Cantarell-VF.otf",
            "/usr/share/fonts/truetype/cantarell/Cantarell-Regular.otf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
        ]
    }
}

fn bold_font_paths() -> &'static [&'static str] {
    #[cfg(target_os = "macos")]
    {
        &[
            "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
            "/System/Library/Fonts/Helvetica.ttc",
        ]
    }
    #[cfg(target_os = "windows")]
    {
        &["C:\\Windows\\Fonts\\arialbd.ttf", "C:\\Windows\\Fonts\\segoeuib.ttf"]
    }
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        &[
            "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
            "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
        ]
    }
}

fn platform_fallback_font_paths() -> &'static [&'static str] {
    #[cfg(target_os = "macos")]
    {
        &[
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/STHeiti Light.ttc",
            "/System/Library/Fonts/Apple Symbols.ttf",
        ]
    }
    #[cfg(target_os = "windows")]
    {
        &["C:\\Windows\\Fonts\\msyh.ttc", "C:\\Windows\\Fonts\\seguiemj.ttf"]
    }
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        &[
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansSC-Regular.otf",
            "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
            "/usr/share/fonts/truetype/noto/NotoEmoji-Regular.ttf",
        ]
    }
}

/// R4377：generic 族 face（serif / sans-serif 及各自 bold）——`build_font_resolver`
/// 按 nameID1 族名匹配（`serif_names`/`sans_names` 优先序），缺 face 则通用族落
/// default。清单与 reftest harness（reftest_fonts.rs）chromium 对齐口径一致。
/// macOS/Windows 待各自字体清单核对后补（Linux 先行，与 harness Bold 加载同范围）。
fn generic_family_font_paths() -> &'static [&'static str] {
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        &[
            "/usr/share/fonts/truetype/liberation/LiberationSerif-Regular.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSerif-Bold.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
        ]
    }
    #[cfg(target_os = "macos")]
    {
        &[
            "/System/Library/Fonts/Supplemental/Times New Roman.ttf",
            "/System/Library/Fonts/Supplemental/Arial.ttf",
        ]
    }
    #[cfg(target_os = "windows")]
    {
        &["C:\\Windows\\Fonts\\times.ttf", "C:\\Windows\\Fonts\\arial.ttf"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_paths_are_unique() {
        let candidates = deduplicate_paths(vec![
            (PathBuf::from("a.ttc"), 2),
            (PathBuf::from("a.ttc"), 0),
            (PathBuf::from("b.ttf"), 0),
        ]);
        assert_eq!(candidates, [(PathBuf::from("a.ttc"), 2), (PathBuf::from("b.ttf"), 0)]);
    }

    #[test]
    fn platform_has_primary_candidates() {
        assert!(!primary_font_paths().is_empty());
        assert!(
            primary_font_paths()
                .iter()
                .all(|path| std::path::Path::new(path).is_absolute())
        );
    }

    #[test]
    fn platform_font_registration_does_not_expand_fontdue_outlines() {
        let platform = load_platform_fonts();
        assert_eq!(platform.loader.parsed_fontdue_count(), 0);

        let _ = platform.loader.build_font_resolver();
        let _ = platform.loader.build_line_metric_map();
        assert_eq!(
            platform.loader.parsed_fontdue_count(),
            0,
            "startup metadata must not eagerly parse CJK or Emoji glyph geometry"
        );
    }

    /// R4377：generic 族解析与 chromium 对齐（R1259 serif / R1263 sans / R4373 mono 谱系）
    /// ——生产 `load_platform_fonts` 载入 Liberation Serif/Sans 后，`build_font_resolver`
    /// 的 serif/sans-serif/monospace 键应命中对应 face（与 reftest harness 同口径）。
    /// 字体缺失环境（CI 最小容器）静默跳过对应断言。
    #[test]
    fn r4377_platform_generic_families_resolve_like_harness() {
        let platform = load_platform_fonts();
        let resolver = platform.loader.build_font_resolver();
        let family_of = |key: &str| -> Option<String> {
            let id = resolver.get(key).copied()?;
            let data = platform.loader.get_font_data(id)?;
            use crate::font::loader::parse_font_family_name_at;
            parse_font_family_name_at(data, platform.loader.face_index(id))
        };
        if std::path::Path::new("/usr/share/fonts/truetype/liberation/LiberationSerif-Regular.ttf").exists() {
            assert_eq!(
                family_of("serif").as_deref(),
                Some("Liberation Serif"),
                "serif generic must resolve to Liberation Serif (chromium Times New Roman alias), got {:?}",
                family_of("serif")
            );
        }
        if std::path::Path::new("/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf").exists() {
            assert_eq!(
                family_of("sans-serif").as_deref(),
                Some("Liberation Sans"),
                "sans-serif generic must resolve to Liberation Sans (chromium Arial alias), got {:?}",
                family_of("sans-serif")
            );
        }
        let cjk_ttc = std::path::Path::new("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc");
        if cjk_ttc.exists() {
            assert_eq!(
                family_of("monospace").as_deref(),
                Some("Noto Sans Mono CJK SC"),
                "monospace generic must resolve to ttc face 7 (fc-match monospace true face), got {:?}",
                family_of("monospace")
            );
        }
    }
}
