//! 字体回退和国际化渲染管线集成测试（质量测试矩阵 Layer 8）
//!
//! 验证不同文字系统的文本通过完整渲染管线（WebView → Engine → Layout → Paint）
//! 正确生成渲染图元，覆盖 CJK、emoji、RTL、混合文本和字体回退场景。

use zero_webview::{WebView, WebViewConfig};

// ── 辅助函数 ──

/// 创建标准测试 WebView（800×600）
fn create_webview() -> WebView {
    WebView::new(WebViewConfig {
        width: 800,
        height: 600,
        ..Default::default()
    })
}

// ── CJK 文本渲染管线测试 ──

/// 中文文本通过完整渲染管线生成 glyph
#[test]
fn test_font_fallback_chinese_glyphs() {
    let mut wv = create_webview();
    let result = wv.load_html(r#"<html><body><p>这是中文文本测试</p></body></html>"#, None);

    // 即使字体不支持 CJK，布局引擎也应生成 glyph 图元
    assert!(!result.primitives().glyphs.is_empty(), "中文文本应产生 glyph 图元");
}

/// 日文文本（平假名+片假名+汉字）渲染管线
#[test]
fn test_font_fallback_japanese_glyphs() {
    let mut wv = create_webview();
    let result = wv.load_html(
        r#"<html><body><p>ひらがなとカタカナと漢字の混在テスト</p></body></html>"#,
        None,
    );

    assert!(!result.primitives().glyphs.is_empty(), "日文文本应产生 glyph 图元");
}

/// 韩文文本渲染管线
#[test]
fn test_font_fallback_korean_glyphs() {
    let mut wv = create_webview();
    let result = wv.load_html(r#"<html><body><p>한국어 텍스트 렌더링 테스트</p></body></html>"#, None);

    assert!(!result.primitives().glyphs.is_empty(), "韩文文本应产生 glyph 图元");
}

/// CJK 大段落文本渲染管线不 panic
#[test]
fn test_font_fallback_cjk_large_paragraph() {
    let mut wv = create_webview();
    let long_text = "这是一段较长的中文文本。".repeat(20);
    let html = format!(r#"<html><body><p>{}</p></body></html>"#, long_text);
    let result = wv.load_html(&html, None);

    assert!(!result.primitives().glyphs.is_empty(), "大段落 CJK 文本应产生 glyph");
    assert!(result.timings.total_ms >= 0.0);
}

// ── Emoji 渲染管线测试 ──

/// 基础 emoji 渲染不 panic
#[test]
fn test_font_fallback_emoji_basic() {
    let mut wv = create_webview();
    let result = wv.load_html(r#"<html><body><p>Hello 😀 🎉 🚀 World</p></body></html>"#, None);

    // emoji 可能无法渲染为可视 glyph，但管线不应崩溃
    assert!(result.timings.total_ms >= 0.0, "emoji 渲染不应崩溃");
    assert!(!result.primitives().glyphs.is_empty(), "混合文本应至少产生部分 glyph");
}

/// 复杂 emoji 序列渲染不 panic
#[test]
fn test_font_fallback_emoji_zwj_sequences() {
    let mut wv = create_webview();
    let result = wv.load_html(
        r#"<html><body>
        <p>Family: 👨‍👩‍👧‍👦</p>
        <p>Worker: 👩‍💻</p>
        <p>Astronaut: 👨‍🚀</p>
        </body></html>"#,
        None,
    );

    assert!(result.timings.total_ms >= 0.0, "ZWJ emoji 不应崩溃");
}

/// 国旗 emoji 渲染不 panic
#[test]
fn test_font_fallback_emoji_flags() {
    let mut wv = create_webview();
    let result = wv.load_html(r#"<html><body><p>Flags: 🇺🇸 🇬🇧 🇯🇵 🇰🇷 🇨🇳 🇫🇷 🇩🇪</p></body></html>"#, None);

    assert!(result.timings.total_ms >= 0.0, "国旗 emoji 渲染不应崩溃");
}

// ── RTL 文本渲染管线测试 ──

/// 阿拉伯文 RTL 渲染管线
#[test]
fn test_font_fallback_arabic_rtl() {
    let mut wv = create_webview();
    let result = wv.load_html(
        r#"<html><body><div dir="rtl"><p>مرحبا بالعالم</p></div></body></html>"#,
        None,
    );

    assert!(!result.primitives().glyphs.is_empty(), "阿拉伯文应产生 glyph");
    assert!(result.timings.total_ms >= 0.0);
}

/// 希伯来文 RTL 渲染管线
#[test]
fn test_font_fallback_hebrew_rtl() {
    let mut wv = create_webview();
    let result = wv.load_html(
        r#"<html><body><div dir="rtl"><p>שלום עולם</p></div></body></html>"#,
        None,
    );

    assert!(!result.primitives().glyphs.is_empty(), "希伯来文应产生 glyph");
}

/// 双向混合文本渲染管线
#[test]
fn test_font_fallback_bidi_mixed() {
    let mut wv = create_webview();
    let result = wv.load_html(
        r#"<html><body>
        <p>English and العربية mixed text</p>
        <p>LTR + RTL: Hello مرحبا World</p>
        </body></html>"#,
        None,
    );

    assert!(!result.primitives().glyphs.is_empty(), "双向混合文本应产生 glyph");
    assert!(result.timings.total_ms >= 0.0);
}

// ── 多语言混合渲染管线测试 ──

/// 多语言混合文本渲染管线
#[test]
fn test_font_fallback_multilingual_mixed() {
    let mut wv = create_webview();
    let result = wv.load_html(
        r#"<html><body>
        <p>English 你好 こんにちは 안녕하세요 Hola مرحبا</p>
        <p>The 快速 brown fox 狐狸 jumps over ひと 13.</p>
        </body></html>"#,
        None,
    );

    assert!(!result.primitives().glyphs.is_empty(), "多语言混合文本应产生 glyph");
}

/// 泰文/天城文渲染管线不 panic
#[test]
fn test_font_fallback_thai_devanagari() {
    let mut wv = create_webview();
    let result = wv.load_html(
        r#"<html><body>
        <p>Thai: สวัสดีครับ ภาษาไทย</p>
        <p>Hindi: नमस्ते दुनिया</p>
        <p>Bengali: হ্যালো বিশ্ব</p>
        </body></html>"#,
        None,
    );

    assert!(result.timings.total_ms >= 0.0, "泰文/天城文渲染不应崩溃");
    assert!(!result.primitives().glyphs.is_empty(), "泰文/天城文应产生 glyph");
}

// ── 字体样式回退管线测试 ──

/// 指定不存在字体时的回退渲染
#[test]
fn test_font_fallback_nonexistent_font_family() {
    let mut wv = create_webview();
    let result = wv.load_html(
        r#"<html><body>
        <p style="font-family: 'NonExistentFont123', sans-serif;">Fallback text</p>
        <p style="font-family: 'AnotherMissingFont', serif;">Serif fallback</p>
        </body></html>"#,
        None,
    );

    assert!(
        !result.primitives().glyphs.is_empty(),
        "不存在的字体应回退并仍产生 glyph"
    );
}

/// 不同字号的 CJK 文本渲染
#[test]
fn test_font_fallback_cjk_various_sizes() {
    let mut wv = create_webview();
    let result = wv.load_html(
        r#"<html><body>
        <p style="font-size: 12px;">小号中文</p>
        <p style="font-size: 16px;">中号中文</p>
        <p style="font-size: 24px;">大号中文</p>
        <p style="font-size: 48px;">超大号中文</p>
        </body></html>"#,
        None,
    );

    assert!(!result.primitives().glyphs.is_empty(), "不同字号 CJK 文本应产生 glyph");
    assert!(result.timings.total_ms >= 0.0);
}

/// 粗体/斜体 CJK 文本渲染不 panic
#[test]
fn test_font_fallback_cjk_bold_italic() {
    let mut wv = create_webview();
    let result = wv.load_html(
        r#"<html><body>
        <p><b>粗体中文文本</b></p>
        <p><i>斜体中文文本</i></p>
        <p><b><i>粗斜体中文</i></b></p>
        </body></html>"#,
        None,
    );

    assert!(!result.primitives().glyphs.is_empty(), "粗斜体 CJK 应产生 glyph");
}

// ── Unicode 特殊字符渲染管线测试 ──

/// 数学符号和特殊 Unicode 字符渲染
#[test]
fn test_font_fallback_math_symbols() {
    let mut wv = create_webview();
    let result = wv.load_html(
        r#"<html><body>
        <p>∀x∈ℝ: x² ≥ 0 ≈ ∑ ∫ ∂ √ ∞</p>
        <p>α β γ δ ε ζ η θ λ μ π σ φ ψ ω</p>
        <p>← → ↑ ↓ ⇐ ⇒ ⇑ ⇓ ↔ ↕</p>
        </body></html>"#,
        None,
    );

    assert!(!result.primitives().glyphs.is_empty(), "数学符号应产生 glyph");
}

/// 货币和特殊符号渲染
#[test]
fn test_font_fallback_currency_symbols() {
    let mut wv = create_webview();
    let result = wv.load_html(
        r#"<html><body>
        <p>€ £ ¥ $ ₽ ₹ ₩ ₺ ¢ ₴</p>
        <p>© ® ™ § ¶ † ‡ • … – —</p>
        </body></html>"#,
        None,
    );

    assert!(!result.primitives().glyphs.is_empty(), "货币符号应产生 glyph");
}

// ── 竖排文本渲染管线测试 ──

/// writing-mode: vertical-rl 渲染不 panic
#[test]
fn test_font_fallback_vertical_text() {
    let mut wv = create_webview();
    let result = wv.load_html(
        r#"<html><body>
        <div style="writing-mode: vertical-rl; height: 300px;">
            <p>日本語の縦書き</p>
            <p>中文竖排文字</p>
        </div>
        </body></html>"#,
        None,
    );

    assert!(!result.primitives().glyphs.is_empty(), "竖排文本应产生 glyph");
    assert!(result.timings.total_ms >= 0.0);
}

// ── 综合多语言页面渲染测试 ──

/// 多语言仪表盘页面完整渲染管线
#[test]
fn test_font_fallback_multilingual_dashboard() {
    let mut wv = create_webview();
    let result = wv.load_html(
        r#"<html><head><style>
            .grid { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
            .card { background: #f0f0f0; padding: 12px; border-radius: 4px; }
        </style></head><body>
        <h1>多语言仪表盘</h1>
        <div class="grid">
            <div class="card">
                <h3>中文</h3>
                <p>欢迎使用多语言仪表盘</p>
            </div>
            <div class="card">
                <h3>日本語</h3>
                <p>多言語ダッシュボードへようこそ</p>
            </div>
            <div class="card">
                <h3>한국어</h3>
                <p>다국어 대시보드에 오신 것을 환영합니다</p>
            </div>
            <div class="card" dir="rtl">
                <h3>العربية</h3>
                <p>مرحبا بك في لوحة المعلومات متعددة اللغات</p>
            </div>
        </div>
        <p>Mixed: Hello 世界 🌍 2024</p>
        </body></html>"#,
        None,
    );

    assert!(!result.primitives().glyphs.is_empty(), "多语言仪表盘应产生 glyph");
    assert!(
        !result.primitives().fills.is_empty() || !result.primitives().rounded_rects.is_empty(),
        "多语言仪表盘应产生 fill"
    );
    assert!(result.timings.total_ms >= 0.0);
}
