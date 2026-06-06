//! 平台和输入测试。
//!
//! 覆盖输入事件、视口调整、HiDPI 缩放、滚动容器、
//! IME/CJK 输入场景、触摸友好布局和焦点管理。

use super::TestCase;

/// 返回平台和输入测试用例。
pub fn platform_input_tests() -> Vec<TestCase> {
    vec![
        // ═══════════════════════════════════════════════════════════════
        // 键盘事件处理页面
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "platform-input/keyboard/event-handlers".into(),
            description: "键盘事件处理器不崩溃".into(),
            category: "platform-input".into(),
            html: r#"<html><body>
            <div id="output">Press any key</div>
            <input type="text" id="input" placeholder="Type here">
            <textarea placeholder="Multiline input"></textarea>
            <div tabindex="0" id="focusable">Focusable with keyboard</div>
            <script>
                document.addEventListener('keydown', function(e) {
                    document.getElementById('output').textContent = 'Key: ' + e.key;
                });
                document.addEventListener('keyup', function(e) {
                    document.getElementById('output').textContent = 'Released: ' + e.key;
                });
            </script>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_input".into(),
                "nonzero_primitives".into(),
            ],
        },
        TestCase {
            id: "platform-input/keyboard/shortcut-page".into(),
            description: "快捷键页面不崩溃".into(),
            category: "platform-input".into(),
            html: r#"<html><body>
            <div id="editor" contenteditable="true">Editable text here</div>
            <div id="status">Ready</div>
            <script>
                document.addEventListener('keydown', function(e) {
                    if (e.ctrlKey && e.key === 's') { e.preventDefault(); }
                    if (e.ctrlKey && e.key === 'c') { /* copy */ }
                    if (e.ctrlKey && e.key === 'v') { /* paste */ }
                    if (e.ctrlKey && e.key === 'z') { /* undo */ }
                    if (e.ctrlKey && e.shiftKey && e.key === 'Z') { /* redo */ }
                });
            </script>
            </body></html>"#
                .into(),
            css: r#"#editor { border: 1px solid #ccc; padding: 8px; min-height: 100px; }"#.into(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 鼠标事件处理页面
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "platform-input/mouse/click-targets".into(),
            description: "鼠标点击目标区域不崩溃".into(),
            category: "platform-input".into(),
            html: r##"<html><body>
            <div id="click-area" style="width:200px; height:100px; background:#e0e0e0;">
                Click me
            </div>
            <div id="result">No click yet</div>
            <a href="#section1">Link 1</a>
            <a href="#section2">Link 2</a>
            <button id="btn1">Button 1</button>
            <button id="btn2">Button 2</button>
            <div id="section1">Section 1</div>
            <div id="section2">Section 2</div>
            </body></html>"##
                .into(),
            css: "a { display: block; margin: 4px; } button { margin: 4px; }".into(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_button".into(),
                "dom_has_link".into(),
                "nonzero_primitives".into(),
            ],
        },
        TestCase {
            id: "platform-input/mouse/hover-states".into(),
            description: "悬停状态 CSS 不崩溃".into(),
            category: "platform-input".into(),
            html: r##"<html><body>
            <nav>
                <a href="#" class="nav-link">Home</a>
                <a href="#" class="nav-link">About</a>
                <a href="#" class="nav-link">Contact</a>
            </nav>
            <div class="card">Hover over me</div>
            <button class="hover-btn">Hover Button</button>
            </body></html>"##
                .into(),
            css: r##"
                nav { display: flex; gap: 8px; }
                .nav-link:hover { color: blue; text-decoration: underline; }
                .card:hover { background: #f0f0f0; box-shadow: 0 2px 8px rgba(0,0,0,0.1); }
                .hover-btn:hover { background: #0056b3; color: white; }
            "##
            .into(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_link".into(),
                "dom_has_button".into(),
                "render_completes".into(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // 触摸友好布局
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "platform-input/touch/touch-targets".into(),
            description: "触摸友好目标尺寸不崩溃".into(),
            category: "platform-input".into(),
            html: r#"<html><body>
            <div class="touch-container">
                <button class="touch-btn">Large Button 1</button>
                <button class="touch-btn">Large Button 2</button>
                <button class="touch-btn">Large Button 3</button>
            </div>
            <div class="touch-grid">
                <div class="touch-cell">1</div>
                <div class="touch-cell">2</div>
                <div class="touch-cell">3</div>
                <div class="touch-cell">4</div>
            </div>
            </body></html>"#
                .into(),
            css: r#"
                .touch-btn {
                    min-height: 48px; min-width: 48px;
                    padding: 12px 24px; margin: 8px;
                    font-size: 16px;
                }
                .touch-grid {
                    display: grid; grid-template-columns: 1fr 1fr;
                    gap: 12px; padding: 12px;
                }
                .touch-cell {
                    min-height: 56px; background: #e0e0e0;
                    display: flex; align-items: center; justify-content: center;
                    border-radius: 8px;
                }
            "#
            .into(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_button".into(),
                "nonzero_primitives".into(),
            ],
        },
        TestCase {
            id: "platform-input/touch/touch-action".into(),
            description: "touch-action CSS 属性不崩溃".into(),
            category: "platform-input".into(),
            html: r#"<html><body>
            <div style="touch-action: none;">No touch action</div>
            <div style="touch-action: pan-x;">Horizontal pan only</div>
            <div style="touch-action: pan-y;">Vertical pan only</div>
            <div style="touch-action: manipulation;">Manipulation only</div>
            <div style="touch-action: pinch-zoom;">Pinch zoom only</div>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "render_completes".into(),
                "nonzero_primitives".into(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // 滚动容器
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "platform-input/scroll/overflow-scroll".into(),
            description: "overflow scroll 容器不崩溃".into(),
            category: "platform-input".into(),
            html: r#"<html><body>
            <div class="scroll-container">
                <p>Line 1 of scrollable content</p>
                <p>Line 2 of scrollable content</p>
                <p>Line 3 of scrollable content</p>
                <p>Line 4 of scrollable content</p>
                <p>Line 5 of scrollable content</p>
                <p>Line 6 of scrollable content</p>
                <p>Line 7 of scrollable content</p>
                <p>Line 8 of scrollable content</p>
                <p>Line 9 of scrollable content</p>
                <p>Line 10 of scrollable content</p>
            </div>
            </body></html>"#
                .into(),
            css: r#"
                .scroll-container {
                    height: 150px; overflow-y: auto;
                    border: 1px solid #ccc; padding: 8px;
                }
                .scroll-container p { margin: 4px 0; }
            "#
            .into(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_paragraph".into(),
                "nonzero_primitives".into(),
            ],
        },
        TestCase {
            id: "platform-input/scroll/scroll-snap".into(),
            description: "scroll-snap 容器不崩溃".into(),
            category: "platform-input".into(),
            html: r#"<html><body>
            <div class="snap-container">
                <div class="snap-item" style="background:#e0e0e0;">Item 1</div>
                <div class="snap-item" style="background:#d0d0d0;">Item 2</div>
                <div class="snap-item" style="background:#c0c0c0;">Item 3</div>
                <div class="snap-item" style="background:#b0b0b0;">Item 4</div>
            </div>
            </body></html>"#
                .into(),
            css: r#"
                .snap-container {
                    height: 200px; overflow-y: auto;
                    scroll-snap-type: y mandatory;
                }
                .snap-item {
                    height: 200px; scroll-snap-align: start;
                    display: flex; align-items: center; justify-content: center;
                    font-size: 24px;
                }
            "#
            .into(),
            assertions: vec![
                "dom_has_body".into(),
                "render_completes".into(),
                "nonzero_primitives".into(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // 视口/响应式布局
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "platform-input/viewport/media-query-responsive".into(),
            description: "媒体查询响应式布局不崩溃".into(),
            category: "platform-input".into(),
            html: r#"<html><body>
            <div class="container">
                <div class="sidebar">Sidebar</div>
                <div class="main">Main content area with responsive layout</div>
            </div>
            </body></html>"#
                .into(),
            css: r#"
                .container { display: flex; gap: 16px; }
                .sidebar { width: 200px; background: #f0f0f0; padding: 8px; }
                .main { flex: 1; background: #e8e8e8; padding: 8px; }
                @media (max-width: 600px) {
                    .container { flex-direction: column; }
                    .sidebar { width: 100%; }
                }
            "#
            .into(),
            assertions: vec!["dom_has_body".into(), "nonzero_primitives".into()],
        },
        TestCase {
            id: "platform-input/viewport/flexible-grid".into(),
            description: "弹性网格布局不崩溃".into(),
            category: "platform-input".into(),
            html: r#"<html><body>
            <div class="grid">
                <div class="item">1</div>
                <div class="item">2</div>
                <div class="item">3</div>
                <div class="item">4</div>
                <div class="item">5</div>
                <div class="item">6</div>
            </div>
            </body></html>"#
                .into(),
            css: r#"
                .grid {
                    display: grid;
                    grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
                    gap: 8px;
                }
                .item {
                    background: #e0e0e0; padding: 16px;
                    text-align: center; border-radius: 4px;
                }
            "#
            .into(),
            assertions: vec!["dom_has_body".into(), "nonzero_primitives".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // HiDPI / 缩放
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "platform-input/hidpi/rem-viewport-units".into(),
            description: "rem 和 viewport 单位不崩溃".into(),
            category: "platform-input".into(),
            html: r#"<html><body>
            <div class="hero">Hero Section with viewport units</div>
            <div class="content">
                <p>Content with rem sizing</p>
                <p>Another paragraph</p>
            </div>
            </body></html>"#
                .into(),
            css: r#"
                html { font-size: 16px; }
                .hero {
                    font-size: 2rem;
                    padding: 2vh 5vw;
                    background: #e0e0e0;
                }
                .content {
                    font-size: 1rem;
                    padding: 1rem;
                    max-width: 80vw;
                }
            "#
            .into(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_paragraph".into(),
                "nonzero_primitives".into(),
            ],
        },
        TestCase {
            id: "platform-input/hidpi/css-zoom".into(),
            description: "CSS zoom 属性不崩溃".into(),
            category: "platform-input".into(),
            html: r#"<html><body>
            <div style="zoom: 1.5;">Zoomed 150%</div>
            <div style="zoom: 0.75;">Zoomed 75%</div>
            <div style="zoom: 2;">Zoomed 200%</div>
            <div>No zoom</div>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "render_completes".into(),
                "nonzero_primitives".into(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // IME/CJK 输入场景
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "platform-input/ime/cjk-input-form".into(),
            description: "CJK 输入法表单不崩溃".into(),
            category: "platform-input".into(),
            html: r#"<html><body>
            <form>
                <label>姓名（中文）</label>
                <input type="text" lang="zh-CN" placeholder="请输入中文姓名">
                <label>名前（日本語）</label>
                <input type="text" lang="ja" placeholder="名前を入力してください">
                <label>이름（한국어）</label>
                <input type="text" lang="ko" placeholder="이름을 입력하세요">
                <textarea lang="zh-CN" placeholder="中文备注信息"></textarea>
            </form>
            </body></html>"#
                .into(),
            css: "input, textarea { width: 100%; padding: 8px; margin: 4px 0; }".into(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_form".into(),
                "dom_has_input".into(),
                "nonzero_primitives".into(),
            ],
        },
        TestCase {
            id: "platform-input/ime/composition-events".into(),
            description: "composition 事件页面不崩溃".into(),
            category: "platform-input".into(),
            html: r#"<html><body>
            <input type="text" id="ime-input" placeholder="IME input here">
            <div id="composition-status">No composition</div>
            <script>
                var input = document.getElementById('ime-input');
                input.addEventListener('compositionstart', function() {
                    document.getElementById('composition-status').textContent = 'Composing...';
                });
                input.addEventListener('compositionupdate', function(e) {
                    document.getElementById('composition-status').textContent = 'Update: ' + e.data;
                });
                input.addEventListener('compositionend', function(e) {
                    document.getElementById('composition-status').textContent = 'Done: ' + e.data;
                });
            </script>
            </body></html>"#
                .into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_input".into(), "render_completes".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 焦点管理
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "platform-input/focus/tab-navigation".into(),
            description: "Tab 导航焦点管理不崩溃".into(),
            category: "platform-input".into(),
            html: r##"<html><body>
            <div class="toolbar">
                <button>File</button>
                <button>Edit</button>
                <button>View</button>
            </div>
            <form>
                <input type="text" placeholder="First field">
                <input type="text" placeholder="Second field">
                <select><option>Option A</option><option>Option B</option></select>
                <textarea placeholder="Text area"></textarea>
                <button type="submit">Submit</button>
                <button type="button">Cancel</button>
            </form>
            <a href="#link1">Link 1</a>
            <a href="#link2">Link 2</a>
            <div tabindex="0">Custom focusable</div>
            </body></html>"##
                .into(),
            css: r#"
                .toolbar { display: flex; gap: 4px; margin-bottom: 8px; }
                :focus { outline: 2px solid blue; outline-offset: 2px; }
            "#
            .into(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_button".into(),
                "dom_has_input".into(),
                "dom_has_link".into(),
                "nonzero_primitives".into(),
            ],
        },
        TestCase {
            id: "platform-input/focus/focus-visible".into(),
            description: "focus-visible 样式不崩溃".into(),
            category: "platform-input".into(),
            html: r##"<html><body>
            <nav>
                <a href="#" class="nav-item">Home</a>
                <a href="#" class="nav-item">About</a>
                <a href="#" class="nav-item">Services</a>
                <a href="#" class="nav-item">Contact</a>
            </nav>
            <main>
                <button class="action-btn">Primary Action</button>
                <button class="action-btn secondary">Secondary</button>
            </main>
            </body></html>"##
                .into(),
            css: r##"
                nav { display: flex; gap: 8px; }
                .nav-item { padding: 8px 16px; }
                .nav-item:focus-visible { outline: 2px solid blue; background: #f0f0ff; }
                .action-btn { padding: 8px 16px; margin: 4px; }
                .action-btn:focus-visible { box-shadow: 0 0 0 3px rgba(0,0,255,0.3); }
                .secondary { background: #e0e0e0; }
            "##
            .into(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_link".into(),
                "dom_has_button".into(),
                "render_completes".into(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // 滚轮/手势
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "platform-input/wheel/scrollable-list".into(),
            description: "可滚动列表不崩溃".into(),
            category: "platform-input".into(),
            html: r#"<html><body>
            <div class="list-container">
                <div class="list-item">Item 1</div>
                <div class="list-item">Item 2</div>
                <div class="list-item">Item 3</div>
                <div class="list-item">Item 4</div>
                <div class="list-item">Item 5</div>
                <div class="list-item">Item 6</div>
                <div class="list-item">Item 7</div>
                <div class="list-item">Item 8</div>
                <div class="list-item">Item 9</div>
                <div class="list-item">Item 10</div>
            </div>
            </body></html>"#
                .into(),
            css: r#"
                .list-container {
                    height: 200px; overflow-y: scroll;
                    border: 1px solid #ccc;
                }
                .list-item {
                    padding: 12px 16px; border-bottom: 1px solid #eee;
                }
            "#
            .into(),
            assertions: vec!["dom_has_body".into(), "nonzero_primitives".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 综合场景
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "platform-input/composite/input-dashboard".into(),
            description: "输入仪表盘综合页面不崩溃".into(),
            category: "platform-input".into(),
            html: r##"<html><body>
            <header>
                <nav>
                    <a href="#" class="logo">Logo</a>
                    <input type="search" placeholder="Search..." class="search-box">
                    <button class="menu-btn">Menu</button>
                </nav>
            </header>
            <main>
                <section class="form-section">
                    <h2>Data Entry Form</h2>
                    <form>
                        <div class="field">
                            <label>名称</label>
                            <input type="text" placeholder="输入名称">
                        </div>
                        <div class="field">
                            <label>Email</label>
                            <input type="email" placeholder="user@example.com">
                        </div>
                        <div class="field">
                            <label>备注</label>
                            <textarea placeholder="中文备注"></textarea>
                        </div>
                        <button type="submit">提交</button>
                    </form>
                </section>
                <section class="data-section">
                    <h2>Scrollable Data</h2>
                    <div class="data-scroll">
                        <table>
                            <tr><td>Row 1</td></tr>
                            <tr><td>Row 2</td></tr>
                            <tr><td>Row 3</td></tr>
                            <tr><td>Row 4</td></tr>
                            <tr><td>Row 5</td></tr>
                            <tr><td>Row 6</td></tr>
                            <tr><td>Row 7</td></tr>
                            <tr><td>Row 8</td></tr>
                        </table>
                    </div>
                </section>
            </main>
            </body></html>"##
                .into(),
            css: r##"
                header { background: #f0f0f0; padding: 8px; }
                nav { display: flex; align-items: center; gap: 8px; }
                .search-box { flex: 1; padding: 8px; }
                main { display: flex; gap: 16px; padding: 16px; }
                .form-section { flex: 1; }
                .field { margin-bottom: 8px; }
                .field label { display: block; margin-bottom: 4px; }
                .field input, .field textarea { width: 100%; padding: 8px; }
                .data-section { flex: 1; }
                .data-scroll { height: 200px; overflow-y: auto; border: 1px solid #ccc; }
                table { width: 100%; }
                td { padding: 8px; border-bottom: 1px solid #eee; }
                button { padding: 8px 16px; }
                :focus { outline: 2px solid blue; }
                @media (max-width: 600px) {
                    main { flex-direction: column; }
                }
            "##
            .into(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_form".into(),
                "dom_has_input".into(),
                "dom_has_button".into(),
                "dom_has_heading".into(),
                "nonzero_primitives".into(),
            ],
        },
    ]
}
