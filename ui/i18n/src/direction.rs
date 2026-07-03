//! 文本方向（spec IF-007 `TextDirection`）与 locale → 方向推断（RTL 检测）。

use crate::locale::LocaleId;
use serde::{Deserialize, Serialize};

/// 文本方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TextDirection {
    #[default]
    Ltr,
    Rtl,
    /// 由内容首段强方向字符决定（M1 = Ltr 兜底）。
    Auto,
}

/// 默认 RTL 书写的语言前缀（CLDR script metadata）。
///
/// ISO 639-1：`ar`/`he`/`iw`/`fa`/`ur`/`yi`/`ps`/`sd`；
/// ISO 639-3：`ckb`（中库尔德 Sorani，阿拉伯字母）/ `dv`（迪维希 Thaana）/ `nqo`（N'Ko）。
/// 注意 `ku`（库尔德 Kurmanji，拉丁字母）默认 LTR，**不**在此列。
/// 按语言首段（script 子标签未细化）匹配；带显式 `-Latn`/`-Arab` 等脚本的细分由宿主按需扩展。
const RTL_LANG_PREFIXES: &[&str] = &["ar", "he", "iw", "fa", "ur", "yi", "ps", "sd", "ckb", "dv", "nqo"];

/// 推断 locale 的默认文本方向。
pub fn direction_for(locale: &LocaleId) -> TextDirection {
    let tag = locale.0.as_str();
    let lang = tag.split('-').next().unwrap_or("").to_ascii_lowercase();
    if RTL_LANG_PREFIXES.contains(&lang.as_str()) {
        TextDirection::Rtl
    } else {
        TextDirection::Ltr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rtl_detection() {
        assert_eq!(direction_for(&LocaleId::new("ar")), TextDirection::Rtl);
        assert_eq!(direction_for(&LocaleId::new("ar-SA")), TextDirection::Rtl);
        assert_eq!(direction_for(&LocaleId::new("he-IL")), TextDirection::Rtl);
        assert_eq!(direction_for(&LocaleId::new("en-US")), TextDirection::Ltr);
        assert_eq!(direction_for(&LocaleId::new("zh-Hans-CN")), TextDirection::Ltr);
    }

    // ── 深度审查（lei-deep-review）：CLDR RTL 集合完整性 ──────────────────
    #[test]
    fn ckb_dv_nqo_detected_as_rtl() {
        // 补齐 docstring 标注的「完整集合留 M2」：CLDR 现代 RTL 语言还包括
        // ckb（中库尔德 Sorani，阿拉伯字母）/ dv（迪维希）/ nqo（N'Ko）。
        // ku（库尔德 Kurmanji，拉丁字母）仍为 Ltr——只 ckb 是 Rtl。
        assert_eq!(direction_for(&LocaleId::new("ckb")), TextDirection::Rtl);
        assert_eq!(direction_for(&LocaleId::new("ckb-IQ")), TextDirection::Rtl);
        assert_eq!(direction_for(&LocaleId::new("dv")), TextDirection::Rtl);
        assert_eq!(direction_for(&LocaleId::new("nqo")), TextDirection::Rtl);
        // ku（Kurmanji 拉丁字母）非 RTL。
        assert_eq!(direction_for(&LocaleId::new("ku")), TextDirection::Ltr);
        // 未知 locale 回落 Ltr。
        assert_eq!(direction_for(&LocaleId::new("xx-YY")), TextDirection::Ltr);
    }
}
