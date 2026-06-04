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
            </body></html>"#
                .into(),
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
            </body></html>"#
                .into(),
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
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_button".into(),
                "dom_has_list".into(),
                "no_panic".into(),
            ],
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
            </body></html>"#
                .into(),
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
            </body></html>"#
                .into(),
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
            </body></html>"#
                .into(),
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
            </body></html>"#
                .into(),
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
            </body></html>"#
                .into(),
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
            </body></html>"#
                .into(),
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
            </body></html>"#
                .into(),
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
            </body></html>"#
                .into(),
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
            </body></html>"#
                .into(),
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
            </body></html>"#
                .into(),
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
            </body></html>"#
                .into(),
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
            </body></html>"#
                .into(),
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
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // ARIA 扩展
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "a11y-i18n/aria/expanded-controls".into(),
            description: "ARIA expanded 控件状态".into(),
            category: "a11y-i18n".into(),
            html: r#"<html><body>
            <button aria-expanded="false" aria-controls="panel1">Toggle Panel</button>
            <div id="panel1" aria-hidden="true">
                <p>Hidden panel content</p>
            </div>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_button".into(), "no_panic".into()],
        },
        TestCase {
            id: "a11y-i18n/aria/tab-interface".into(),
            description: "ARIA tab 界面角色".into(),
            category: "a11y-i18n".into(),
            html: r#"<html><body>
            <div role="tablist">
                <button role="tab" aria-selected="true" aria-controls="panel-a">Tab A</button>
                <button role="tab" aria-selected="false" aria-controls="panel-b">Tab B</button>
            </div>
            <div role="tabpanel" id="panel-a"><p>Panel A content</p></div>
            <div role="tabpanel" id="panel-b" hidden><p>Panel B content</p></div>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_button".into(), "no_panic".into()],
        },
        TestCase {
            id: "a11y-i18n/aria/checkbox-radio".into(),
            description: "ARIA checkbox 和 radio 角色".into(),
            category: "a11y-i18n".into(),
            html: r#"<html><body>
            <div role="group" aria-label="Options">
                <div role="checkbox" aria-checked="true">Option A</div>
                <div role="checkbox" aria-checked="false">Option B</div>
            </div>
            <div role="radiogroup" aria-label="Size">
                <div role="radio" aria-checked="true">Small</div>
                <div role="radio" aria-checked="false">Large</div>
            </div>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 键盘导航
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "a11y-i18n/keyboard/focus-management".into(),
            description: "tabindex 焦点管理".into(),
            category: "a11y-i18n".into(),
            html: r##"<html><body>
            <div tabindex="0">Focusable div</div>
            <span tabindex="-1">Programmatically focusable</span>
            <button>Native focusable</button>
            <a href="#section">Focusable link</a>
            </body></html>"##
                .into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_button".into(), "dom_has_link".into(), "no_panic".into()],
        },
        TestCase {
            id: "a11y-i18n/keyboard/accesskey".into(),
            description: "accesskey 属性".into(),
            category: "a11y-i18n".into(),
            html: r#"<html><body>
            <button accesskey="s">Save (Alt+S)</button>
            <a href="/help" accesskey="h">Help (Alt+H)</a>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_button".into(), "dom_has_link".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 高对比度模式相关
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "a11y-i18n/contrast/high-contrast-page".into(),
            description: "高对比度页面结构".into(),
            category: "a11y-i18n".into(),
            html: r##"<html><body style="background:#000; color:#fff;">
            <h1 style="color:#ff0;">High Contrast</h1>
            <p style="color:#fff;">White text on black background</p>
            <a href="#" style="color:#0ff; text-decoration:underline;">Cyan link</a>
            <button style="background:#333; color:#fff; border:2px solid #fff;">Action</button>
            </body></html>"##
                .into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_heading".into(),
                "dom_has_button".into(),
                "dom_has_link".into(),
                "no_panic".into(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // CJK 扩展
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "a11y-i18n/cjk/multilingual-form".into(),
            description: "多语言表单".into(),
            category: "a11y-i18n".into(),
            html: r#"<html><body>
            <form>
                <label for="name-cn">姓名</label>
                <input type="text" id="name-cn" lang="zh-CN">
                <label for="name-jp">名前</label>
                <input type="text" id="name-jp" lang="ja">
                <label for="name-kr">이름</label>
                <input type="text" id="name-kr" lang="ko">
                <button type="submit">提交 / 送信 / 제출</button>
            </form>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_form".into(),
                "dom_has_input".into(),
                "dom_has_button".into(),
                "no_panic".into(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // RTL 布局扩展
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "a11y-i18n/rtl/full-page-rtl".into(),
            description: "完整 RTL 页面布局".into(),
            category: "a11y-i18n".into(),
            html: r##"<html dir="rtl" lang="ar"><body>
            <header>عنوان الموقع</header>
            <nav><a href="#">الرئيسية</a> | <a href="#">عن الموقع</a></nav>
            <main>
                <article>
                    <h1>عنوان المقال</h1>
                    <p>محتوى المقال باللغة العربية.</p>
                </article>
                <aside>محتوى جانبي</aside>
            </main>
            <footer>حقوق النشر</footer>
            </body></html>"##
                .into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_nav".into(),
                "dom_has_link".into(),
                "dom_has_heading".into(),
                "no_panic".into(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // 屏幕阅读器相关
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "a11y-i18n/screenreader/sr-only".into(),
            description: "sr-only 视觉隐藏但屏幕阅读器可读".into(),
            category: "a11y-i18n".into(),
            html: r#"<html><body>
            <button>
                <span aria-hidden="true">✕</span>
                <span class="sr-only">Close dialog</span>
            </button>
            <style>
                .sr-only { position:absolute; width:1px; height:1px; overflow:hidden; clip:rect(0,0,0,0); }
            </style>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_button".into(), "no_panic".into()],
        },
    ]
}
