//! 可访问性和国际化测试。
//!
//! 覆盖 ARIA 属性、语义化 HTML、键盘导航、CJK 文本、
//! RTL 布局、Unicode、emoji 等可访问性和国际化场景。

use super::TestCase;

/// 返回可访问性和国际化测试用例。
pub fn a11y_i18n_tests() -> Vec<TestCase> {
    vec![
        // ═══════════════════════════════════════════════════════════════
        // ARIA 角色
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "a11y-i18n/aria/roles".into(),
            description: "ARIA 角色属性不崩溃".into(),
            category: "a11y-i18n".into(),
            html: r#"<html><body>
            <nav role="navigation" aria-label="Main">Nav</nav>
            <main role="main">
                <div role="alert">Alert message</div>
                <div role="status">Status update</div>
                <button role="button" aria-pressed="false">Toggle</button>
            </main>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_nav".into(), "no_panic".into()],
        },
        TestCase {
            id: "a11y-i18n/aria/live-region".into(),
            description: "aria-live 区域不崩溃".into(),
            category: "a11y-i18n".into(),
            html: r#"<html><body>
            <div aria-live="polite">Updates appear here</div>
            <div aria-live="assertive">Urgent update</div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        TestCase {
            id: "a11y-i18n/aria/expanded".into(),
            description: "aria-expanded 菜单不崩溃".into(),
            category: "a11y-i18n".into(),
            html: r#"<html><body>
            <button aria-expanded="false" aria-controls="menu">Menu</button>
            <ul id="menu" role="menu" hidden>
                <li role="menuitem">Item 1</li>
                <li role="menuitem">Item 2</li>
            </ul>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_button".into(), "dom_has_list".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 语义化 HTML
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "a11y-i18n/semantic/landmarks".into(),
            description: "语义化 landmark 元素不崩溃".into(),
            category: "a11y-i18n".into(),
            html: r#"<html><body>
            <header>Header</header>
            <nav>Navigation</nav>
            <main>
                <article>
                    <h1>Title</h1>
                    <section>Content</section>
                </article>
                <aside>Sidebar</aside>
            </main>
            <footer>Footer</footer>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_nav".into(),
                "dom_has_article".into(),
                "dom_has_header".into(),
                "dom_has_footer".into(),
                "no_panic".into(),
            ],
        },
        TestCase {
            id: "a11y-i18n/semantic/details-summary".into(),
            description: "details/summary 可折叠内容".into(),
            category: "a11y-i18n".into(),
            html: r#"<html><body>
            <details>
                <summary>Click to expand</summary>
                <p>Hidden content revealed on click</p>
            </details>
            <details open>
                <summary>Already open</summary>
                <p>Visible content</p>
            </details>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        TestCase {
            id: "a11y-i18n/semantic/dialog".into(),
            description: "dialog 元素不崩溃".into(),
            category: "a11y-i18n".into(),
            html: r#"<html><body>
            <dialog id="dlg">
                <p>Dialog content</p>
                <button>Close</button>
            </dialog>
            <button onclick="document.getElementById('dlg').showModal()">Open</button>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_button".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 表单可访问性
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "a11y-i18n/form/labels".into(),
            description: "label + input 关联不崩溃".into(),
            category: "a11y-i18n".into(),
            html: r#"<html><body>
            <form>
                <label for="name">Name:</label>
                <input id="name" type="text" aria-required="true">
                <label for="email">Email:</label>
                <input id="email" type="email" aria-describedby="email-hint">
                <span id="email-hint">Enter a valid email</span>
                <fieldset>
                    <legend>Preferences</legend>
                    <label><input type="checkbox"> Option A</label>
                    <label><input type="checkbox"> Option B</label>
                </fieldset>
            </form>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_form".into(),
                "dom_has_input".into(),
                "no_panic".into(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // CJK 文本渲染
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "a11y-i18n/cjk/chinese-text".into(),
            description: "中文文本渲染不崩溃".into(),
            category: "a11y-i18n".into(),
            html: r#"<html><body>
            <p>这是一段中文文本，用于测试中文渲染。</p>
            <p>繁體中文測試文本。</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_paragraph".into(), "no_panic".into()],
        },
        TestCase {
            id: "a11y-i18n/cjk/japanese-text".into(),
            description: "日文文本渲染不崩溃".into(),
            category: "a11y-i18n".into(),
            html: r#"<html><body>
            <p>これは日本語のテストテキストです。</p>
            <p>漢字、ひらがな、カタカナの混在テスト。</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_paragraph".into(), "no_panic".into()],
        },
        TestCase {
            id: "a11y-i18n/cjk/korean-text".into(),
            description: "韩文文本渲染不崩溃".into(),
            category: "a11y-i18n".into(),
            html: r#"<html><body>
            <p>한국어 테스트 텍스트입니다.</p>
            <p>대한민국의 언어 렌더링 테스트.</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_paragraph".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // RTL 布局
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "a11y-i18n/rtl/arabic-text".into(),
            description: "阿拉伯文 RTL 布局不崩溃".into(),
            category: "a11y-i18n".into(),
            html: r#"<html><body>
            <div dir="rtl">
                <p>هذا نص عربي للتجربة.</p>
                <p>اختبار التخطيط من اليمين إلى اليسار.</p>
            </div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        TestCase {
            id: "a11y-i18n/rtl/hebrew-text".into(),
            description: "希伯来文 RTL 布局不崩溃".into(),
            category: "a11y-i18n".into(),
            html: r#"<html><body>
            <div dir="rtl">
                <p>זהו טקסט בעברית לבדיקה.</p>
            </div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // Unicode / Emoji
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "a11y-i18n/unicode/emoji".into(),
            description: "Emoji 渲染不崩溃".into(),
            category: "a11y-i18n".into(),
            html: r#"<html><body>
            <p>Emoji: 😀 🎉 🚀 ❤️ 👍 🌍 🎨 ⭐ 🔥 💯</p>
            <p>Flags: 🇺🇸 🇬🇧 🇯🇵 🇰🇷 🇨🇳 🇫🇷 🇩🇪</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        TestCase {
            id: "a11y-i18n/unicode/special-chars".into(),
            description: "特殊 Unicode 字符不崩溃".into(),
            category: "a11y-i18n".into(),
            html: r#"<html><body>
            <p>Symbols: © ® ™ § ¶ † ‡ € £ ¥ ¢ ₽</p>
            <p>Math: ∑ ∏ √ ∞ ≈ ≠ ≤ ≥ ± × ÷</p>
            <p>Arrows: ← → ↑ ↓ ↔ ↕ ⇒ ⇐ ⇑ ⇓</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 混合语言
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "a11y-i18n/mixed/multilingual".into(),
            description: "多语言混合文本不崩溃".into(),
            category: "a11y-i18n".into(),
            html: r#"<html><body>
            <p>Mixed: Hello 你好 こんにちは 안녕하세요 Hola مرحبا</p>
            <p>The 快速 brown fox 狐狸 jumps over ひと 13.</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        TestCase {
            id: "a11y-i18n/mixed/bidi-mixed".into(),
            description: "双向文本混合不崩溃".into(),
            category: "a11y-i18n".into(),
            html: r#"<html><body>
            <p>English and العربية mixed in one paragraph.</p>
            <p>LTR + RTL: Hello مرحبا World</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
    ]
}
