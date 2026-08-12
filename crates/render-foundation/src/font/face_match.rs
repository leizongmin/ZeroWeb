//! CSS font face alias registration and width matching.

use std::collections::HashMap;

/// CSS `font-stretch: normal` percentage.
pub const NORMAL_FONT_STRETCH: f32 = 100.0;

fn stretch_basis(stretch: f32) -> u16 {
    (stretch.clamp(0.1, 6553.5) * 10.0).round() as u16
}

fn legacy_suffix(want_bold: bool, italic: bool) -> &'static str {
    match (want_bold, italic) {
        (true, true) => ":700:italic",
        (true, false) => ":700",
        (false, true) => ":italic",
        (false, false) => "",
    }
}

fn width_key(family: &str, basis: u16, suffix: &str) -> String {
    format!("{family}:stretch={basis}{suffix}")
}

/// Returns aliases used to register one `@font-face`.
///
/// Every face receives a width-specific alias. Normal-width faces also retain
/// the legacy weight/style alias so existing system-font and host integrations
/// remain compatible.
pub fn font_face_aliases(family: &str, weight: Option<u16>, italic: bool, stretch: Option<f32>) -> Vec<String> {
    let want_bold = weight.is_some_and(|weight| weight >= 600);
    let suffix = legacy_suffix(want_bold, italic);
    let basis = stretch_basis(stretch.unwrap_or(NORMAL_FONT_STRETCH));
    let mut aliases = vec![width_key(family, basis, suffix)];
    if basis == stretch_basis(NORMAL_FONT_STRETCH) {
        aliases.push(format!("{family}{suffix}"));
    }
    aliases
}

fn lookup_face(resolver: &HashMap<String, u32>, key: &str) -> Option<u32> {
    resolver.get(key).copied().or_else(|| {
        resolver
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(key))
            .map(|(_, id)| *id)
    })
}

fn available_widths(resolver: &HashMap<String, u32>, family: &str) -> Vec<u16> {
    let width_prefix = format!("{family}:stretch=").to_ascii_lowercase();
    let family_lower = family.to_ascii_lowercase();
    let mut widths = Vec::new();
    for key in resolver.keys() {
        let key_lower = key.to_ascii_lowercase();
        if let Some(rest) = key_lower.strip_prefix(&width_prefix) {
            let basis = rest.split(':').next().and_then(|value| value.parse::<u16>().ok());
            if let Some(basis) = basis {
                widths.push(basis);
            }
        } else if key_lower == family_lower
            || key_lower == format!("{family_lower}:700")
            || key_lower == format!("{family_lower}:italic")
            || key_lower == format!("{family_lower}:700:italic")
        {
            widths.push(stretch_basis(NORMAL_FONT_STRETCH));
        }
    }
    widths.sort_unstable();
    widths.dedup();
    widths
}

fn width_preference(mut widths: Vec<u16>, desired: u16) -> Vec<u16> {
    if desired <= stretch_basis(NORMAL_FONT_STRETCH) {
        widths.sort_unstable_by_key(|width| {
            if *width <= desired {
                (0, u16::MAX - *width)
            } else {
                (1, *width)
            }
        });
    } else {
        widths.sort_unstable_by_key(|width| {
            if *width >= desired {
                (0, *width)
            } else {
                (1, u16::MAX - *width)
            }
        });
    }
    widths
}

/// Resolves one face using CSS Fonts width-first matching, then the existing
/// weight/style fallback order.
///
/// Returns `(font_id, resolved_italic)`.
pub fn resolve_font_face(
    resolver: &HashMap<String, u32>,
    family: &str,
    want_bold: bool,
    want_italic: bool,
    desired_stretch: f32,
) -> Option<(u32, bool)> {
    let style_suffixes: &[&str] = match (want_bold, want_italic) {
        (true, true) => &[":700:italic", ":700", ":italic", ""],
        (true, false) => &[":700", ""],
        (false, true) => &[":italic", ""],
        (false, false) => &[""],
    };
    let normal_basis = stretch_basis(NORMAL_FONT_STRETCH);
    for basis in width_preference(available_widths(resolver, family), stretch_basis(desired_stretch)) {
        for suffix in style_suffixes {
            if let Some(id) = lookup_face(resolver, &width_key(family, basis, suffix)) {
                return Some((id, suffix.contains("italic")));
            }
            if basis == normal_basis
                && let Some(id) = lookup_face(resolver, &format!("{family}{suffix}"))
            {
                return Some((id, suffix.contains("italic")));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_face_keeps_legacy_alias() {
        assert_eq!(
            font_face_aliases("Demo", Some(700), true, Some(100.0)),
            vec!["Demo:stretch=1000:700:italic", "Demo:700:italic"]
        );
    }

    #[test]
    fn width_matching_prefers_narrower_below_normal() {
        let resolver = HashMap::from([
            ("Demo:stretch=500".to_string(), 1),
            ("Demo:stretch=750".to_string(), 2),
            ("Demo:stretch=1000".to_string(), 3),
            ("Demo:stretch=1250".to_string(), 4),
        ]);
        assert_eq!(
            resolve_font_face(&resolver, "Demo", false, false, 87.5),
            Some((2, false))
        );
        assert_eq!(
            resolve_font_face(&resolver, "Demo", false, false, 60.0),
            Some((1, false))
        );
    }

    #[test]
    fn width_matching_prefers_wider_above_normal() {
        let resolver = HashMap::from([
            ("Demo:stretch=750".to_string(), 1),
            ("Demo:stretch=1000".to_string(), 2),
            ("Demo:stretch=1250".to_string(), 3),
            ("Demo:stretch=1500".to_string(), 4),
        ]);
        assert_eq!(
            resolve_font_face(&resolver, "Demo", false, false, 112.5),
            Some((3, false))
        );
        assert_eq!(
            resolve_font_face(&resolver, "Demo", false, false, 175.0),
            Some((4, false))
        );
    }
}
