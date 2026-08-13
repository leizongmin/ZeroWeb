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
}
