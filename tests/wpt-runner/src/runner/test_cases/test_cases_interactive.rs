//! HTML 交互元素和表单合规性测试。
//!
//! 覆盖表单控件、对话框、details/summary、
//! 进度条、meter、datalist、fieldset、output 等交互元素。

use super::TestCase;

/// 返回交互元素合规性测试用例。
pub fn interactive_tests() -> Vec<TestCase> {
    vec![
        // ═══════════════════════════════════════════════════════════════
        // 表单基础结构
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "interactive/form/basic-text-input".into(),
            description: "基础文本输入框不崩溃".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <form>
                <label for="name">Name:</label>
                <input type="text" id="name" name="name" value="Hello">
            </form>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_form".into(), "dom_has_input".into(), "dom_has_element:label".into(), "no_panic".into()],
        },
        TestCase {
            id: "interactive/form/multiple-input-types".into(),
            description: "多种 input type 共存不崩溃".into(),
            category: "interactive".into(),
            html: r##"<html><body>
            <form>
                <input type="text" placeholder="text">
                <input type="password" placeholder="pass">
                <input type="email" placeholder="email">
                <input type="number" value="42">
                <input type="tel" placeholder="tel">
                <input type="url" placeholder="url">
                <input type="search" placeholder="search">
                <input type="date">
                <input type="time">
                <input type="datetime-local">
                <input type="month">
                <input type="week">
                <input type="color" value="#ff0000">
                <input type="range" min="0" max="100" value="50">
                <input type="file">
                <input type="hidden" value="secret">
                <input type="checkbox">
                <input type="radio" name="choice" value="a">
                <input type="radio" name="choice" value="b">
            </form>
            </body></html>"##.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "layout_has_children".into(), "no_panic".into()],
        },
        TestCase {
            id: "interactive/form/textarea".into(),
            description: "textarea 元素渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <form>
                <textarea rows="5" cols="40" placeholder="Enter text here...">Hello World</textarea>
            </form>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:textarea".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        TestCase {
            id: "interactive/form/select-option".into(),
            description: "select 和 option 渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <form>
                <select name="color">
                    <option value="red">Red</option>
                    <option value="green" selected>Green</option>
                    <option value="blue">Blue</option>
                    <optgroup label="Primary">
                        <option value="p-red">Primary Red</option>
                        <option value="p-blue">Primary Blue</option>
                    </optgroup>
                </select>
            </form>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_select".into(), "dom_has_element:option".into(), "dom_has_element:optgroup".into(), "no_panic".into()],
        },
        TestCase {
            id: "interactive/form/button-types".into(),
            description: "各种 button 类型渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <form>
                <button type="submit">Submit</button>
                <button type="reset">Reset</button>
                <button type="button">Click Me</button>
                <input type="submit" value="Go">
                <input type="reset" value="Clear">
            </form>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_button".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 表单验证
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "interactive/validation/required".into(),
            description: "required 属性不崩溃".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <form>
                <input type="text" required>
                <input type="email" required placeholder="email required">
                <input type="text" name="optional">
                <button type="submit">Submit</button>
            </form>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_form".into(), "no_panic".into()],
        },
        TestCase {
            id: "interactive/validation/pattern".into(),
            description: "pattern 属性不崩溃".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <form>
                <input type="text" pattern="[A-Za-z]{3}" title="Three letters">
                <input type="text" pattern="\d{3}-\d{4}" placeholder="123-4567">
            </form>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        TestCase {
            id: "interactive/validation/min-max".into(),
            description: "min/max/step 属性不崩溃".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <form>
                <input type="number" min="0" max="100" step="5" value="50">
                <input type="date" min="2024-01-01" max="2025-12-31">
                <input type="range" min="0" max="10" step="0.5">
            </form>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "no_panic".into()],
        },
        TestCase {
            id: "interactive/validation/maxlength".into(),
            description: "maxlength/minlength 属性不崩溃".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <form>
                <input type="text" maxlength="10" minlength="3">
                <textarea maxlength="500" minlength="10"></textarea>
            </form>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:textarea".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // fieldset / legend / datalist / output
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "interactive/fieldset/basic".into(),
            description: "fieldset + legend 渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <form>
                <fieldset>
                    <legend>Personal Info</legend>
                    <label>Name: <input type="text"></label>
                    <label>Email: <input type="email"></label>
                </fieldset>
                <fieldset disabled>
                    <legend>Disabled Section</legend>
                    <input type="text" value="cannot edit">
                </fieldset>
            </form>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:fieldset".into(), "dom_has_element:legend".into(), "no_panic".into()],
        },
        TestCase {
            id: "interactive/fieldset/nested".into(),
            description: "嵌套 fieldset 渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <form>
                <fieldset>
                    <legend>Outer</legend>
                    <input type="text" placeholder="outer field">
                    <fieldset>
                        <legend>Inner</legend>
                        <input type="text" placeholder="inner field">
                    </fieldset>
                </fieldset>
            </form>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "layout_has_deep_children".into(), "no_panic".into()],
        },
        TestCase {
            id: "interactive/datalist".into(),
            description: "datalist 元素渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <form>
                <input list="colors" placeholder="Pick a color">
                <datalist id="colors">
                    <option value="Red">
                    <option value="Green">
                    <option value="Blue">
                </datalist>
            </form>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:datalist".into(), "no_panic".into()],
        },
        TestCase {
            id: "interactive/output".into(),
            description: "output 元素渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <form oninput="result.value=parseInt(a.value)+parseInt(b.value)">
                <input type="number" id="a" value="10"> +
                <input type="number" id="b" value="20"> =
                <output name="result" for="a b">30</output>
            </form>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:output".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // progress / meter
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "interactive/progress".into(),
            description: "progress 元素渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <label>Downloading: <progress value="70" max="100">70%</progress></label>
            <label>Processing: <progress>Indeterminate</progress></label>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:progress".into(), "no_panic".into()],
        },
        TestCase {
            id: "interactive/meter".into(),
            description: "meter 元素渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <label>Disk usage: <meter min="0" max="100" low="30" high="80" optimum="50" value="65">65%</meter></label>
            <label>Battery: <meter value="0.3">30%</meter></label>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:meter".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // details / summary
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "interactive/details/basic".into(),
            description: "details/summary 基础渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <details>
                <summary>Click to expand</summary>
                <p>Hidden content that appears on click.</p>
            </details>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:details".into(), "dom_has_element:summary".into(), "no_panic".into()],
        },
        TestCase {
            id: "interactive/details/open".into(),
            description: "details open 属性渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <details open>
                <summary>Already expanded</summary>
                <p>This content is visible by default.</p>
                <ul>
                    <li>Item 1</li>
                    <li>Item 2</li>
                </ul>
            </details>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:details".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        TestCase {
            id: "interactive/details/nested".into(),
            description: "嵌套 details 渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <details open>
                <summary>Level 1</summary>
                <details>
                    <summary>Level 2</summary>
                    <p>Deep content</p>
                </details>
            </details>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "layout_has_deep_children".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // dialog
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "interactive/dialog/basic".into(),
            description: "dialog 元素渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <dialog id="dlg">
                <p>This is a dialog.</p>
                <button>Close</button>
            </dialog>
            <button onclick="document.getElementById('dlg').showModal()">Open</button>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:dialog".into(), "dom_has_button".into(), "no_panic".into()],
        },
        TestCase {
            id: "interactive/dialog/open".into(),
            description: "dialog open 属性渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <dialog open>
                <form method="dialog">
                    <p>Modal dialog content</p>
                    <button value="ok">OK</button>
                    <button value="cancel">Cancel</button>
                </form>
            </dialog>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:dialog".into(), "dom_has_form".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // table 交互
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "interactive/table/complete".into(),
            description: "完整表格结构渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <table>
                <caption>Monthly Sales</caption>
                <colgroup>
                    <col>
                    <col span="2" class="data">
                </colgroup>
                <thead>
                    <tr><th>Month</th><th>Revenue</th><th>Profit</th></tr>
                </thead>
                <tbody>
                    <tr><td>Jan</td><td>$1000</td><td>$200</td></tr>
                    <tr><td>Feb</td><td>$1200</td><td>$300</td></tr>
                    <tr><td>Mar</td><td>$900</td><td>$150</td></tr>
                </tbody>
                <tfoot>
                    <tr><th>Total</th><td>$3100</td><td>$650</td></tr>
                </tfoot>
            </table>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_table".into(), "dom_has_element:thead".into(), "dom_has_element:tbody".into(), "dom_has_element:tfoot".into(), "dom_has_element:caption".into(), "dom_has_element:colgroup".into(), "no_panic".into()],
        },
        TestCase {
            id: "interactive/table/nested".into(),
            description: "嵌套表格渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <table>
                <tr><td>Outer 1</td><td>
                    <table><tr><td>Inner</td></tr></table>
                </td></tr>
            </table>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_table".into(), "no_panic".into()],
        },
        TestCase {
            id: "interactive/table/colspan-rowspan".into(),
            description: "colspan/rowspan 渲染不崩溃".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <table>
                <tr><td colspan="2">Wide cell</td><td>Normal</td></tr>
                <tr><td rowspan="2">Tall cell</td><td>A</td><td>B</td></tr>
                <tr><td>C</td><td>D</td></tr>
            </table>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_table".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // template / slot / picture
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "interactive/template".into(),
            description: "template 元素渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <template id="my-template">
                <div class="card">
                    <h2>Title</h2>
                    <p>Content</p>
                </div>
            </template>
            <p>Template above is not rendered.</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:template".into(), "no_panic".into()],
        },
        TestCase {
            id: "interactive/picture".into(),
            description: "picture 元素渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <picture>
                <source media="(min-width: 800px)" srcset="large.jpg">
                <source media="(min-width: 400px)" srcset="medium.jpg">
                <img src="small.jpg" alt="Responsive image">
            </picture>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:picture".into(), "dom_has_element:source".into(), "dom_has_img".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // iframe / embed / object
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "interactive/iframe/basic".into(),
            description: "iframe 元素渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <iframe src="about:blank" width="300" height="200" title="Embedded page"></iframe>
            <iframe srcdoc="<p>Hello iframe</p>" width="200" height="100"></iframe>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:iframe".into(), "no_panic".into()],
        },
        TestCase {
            id: "interactive/iframe/sandbox".into(),
            description: "iframe sandbox 属性不崩溃".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <iframe sandbox="allow-scripts allow-same-origin" src="about:blank"></iframe>
            <iframe sandbox="" src="about:blank"></iframe>
            <iframe sandbox="allow-forms" src="about:blank"></iframe>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:iframe".into(), "no_panic".into()],
        },
        TestCase {
            id: "interactive/embed-object".into(),
            description: "embed/object 元素渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <embed type="text/html" src="about:blank" width="200" height="100">
            <object data="about:blank" type="text/html" width="200" height="100">
                <p>Fallback content</p>
            </object>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:embed".into(), "dom_has_element:object".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // media 占位
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "interactive/video-placeholder".into(),
            description: "video 元素占位渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <video width="320" height="240" controls>
                <source src="movie.mp4" type="video/mp4">
                <source src="movie.ogg" type="video/ogg">
                Your browser does not support video.
            </video>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:video".into(), "dom_has_element:source".into(), "no_panic".into()],
        },
        TestCase {
            id: "interactive/audio-placeholder".into(),
            description: "audio 元素占位渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <audio controls>
                <source src="audio.mp3" type="audio/mpeg">
                <source src="audio.ogg" type="audio/ogg">
                Your browser does not support audio.
            </audio>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:audio".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 导航和链接
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "interactive/links/navigation".into(),
            description: "各种链接类型渲染".into(),
            category: "interactive".into(),
            html: r##"<html><body>
            <nav>
                <a href="https://example.com">External</a>
                <a href="/about">Internal</a>
                <a href="#section">Anchor</a>
                <a href="mailto:test@example.com">Email</a>
                <a href="tel:+1234567890">Phone</a>
                <a href="javascript:void(0)">JS Link</a>
                <a href="data:text/plain,Hello">Data URI</a>
            </nav>
            <h2 id="section">Section Target</h2>
            </body></html>"##.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_link".into(), "dom_has_nav".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        TestCase {
            id: "interactive/links/download".into(),
            description: "download 链接渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <a href="file.pdf" download>Download PDF</a>
            <a href="image.png" download="my-image.png">Download Image</a>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_link".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 综合页面：登录表单
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "interactive/composite/login-form".into(),
            description: "完整登录表单渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <main>
                <h1>Sign In</h1>
                <form action="/login" method="post">
                    <fieldset>
                        <legend>Account</legend>
                        <label for="email">Email:</label>
                        <input type="email" id="email" name="email" required placeholder="you@example.com">
                        <label for="password">Password:</label>
                        <input type="password" id="password" name="password" required minlength="8">
                    </fieldset>
                    <fieldset>
                        <legend>Options</legend>
                        <label><input type="checkbox" name="remember"> Remember me</label>
                    </fieldset>
                    <button type="submit">Sign In</button>
                    <a href="/forgot">Forgot password?</a>
                </form>
            </main>
            </body></html>"#.into(),
            css: r#"
                body { font-family: sans-serif; margin: 20px; }
                h1 { color: #333; }
                form { max-width: 400px; margin: 0 auto; }
                fieldset { border: 1px solid #ccc; padding: 10px; margin: 10px 0; }
                legend { font-weight: bold; }
                label { display: block; margin: 5px 0; }
                input { width: 100%; padding: 8px; margin: 4px 0; box-sizing: border-box; }
                button { background: #0066cc; color: white; padding: 10px 20px; border: none; }
            "#.into(),
            assertions: vec!["dom_has_body".into(), "dom_has_form".into(), "dom_has_element:fieldset".into(), "has_fill_primitives".into(), "has_glyph_primitives".into(), "layout_has_children".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 综合页面：设置页面
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "interactive/composite/settings".into(),
            description: "设置页面渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <main>
                <h1>Settings</h1>
                <section>
                    <h2>Appearance</h2>
                    <label>Theme:
                        <select name="theme">
                            <option value="light">Light</option>
                            <option value="dark" selected>Dark</option>
                            <option value="auto">Auto</option>
                        </select>
                    </label>
                    <label>Font size:
                        <input type="range" min="12" max="24" value="16">
                    </label>
                </section>
                <section>
                    <h2>Privacy</h2>
                    <label><input type="checkbox" checked> Block third-party cookies</label>
                    <label><input type="checkbox"> Send Do Not Track</label>
                    <button type="button">Clear browsing data</button>
                </section>
                <section>
                    <h2>Notifications</h2>
                    <label><input type="radio" name="notif" value="all" checked> All notifications</label>
                    <label><input type="radio" name="notif" value="important"> Important only</label>
                    <label><input type="radio" name="notif" value="none"> None</label>
                </section>
                <progress value="0.5" max="1">Syncing...</progress>
            </main>
            </body></html>"#.into(),
            css: r#"
                body { font-family: sans-serif; max-width: 600px; margin: 20px; }
                section { border-bottom: 1px solid #eee; padding: 15px 0; }
                label { display: block; margin: 5px 0; }
            "#.into(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:section".into(), "dom_has_element:select".into(), "dom_has_element:progress".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 综合页面：产品卡片网格
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "interactive/composite/product-grid".into(),
            description: "产品卡片网格布局".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <main>
                <h1>Products</h1>
                <div class="grid">
                    <div class="card">
                        <img src="https://via.placeholder.com/200" alt="Product 1">
                        <h3>Widget A</h3>
                        <p class="price">$29.99</p>
                        <button>Add to Cart</button>
                    </div>
                    <div class="card">
                        <img src="https://via.placeholder.com/200" alt="Product 2">
                        <h3>Gadget B</h3>
                        <p class="price">$49.99</p>
                        <button>Add to Cart</button>
                    </div>
                    <div class="card">
                        <img src="https://via.placeholder.com/200" alt="Product 3">
                        <h3>Tool C</h3>
                        <p class="price">$19.99</p>
                        <button>Add to Cart</button>
                    </div>
                    <div class="card">
                        <img src="https://via.placeholder.com/200" alt="Product 4">
                        <h3>Device D</h3>
                        <p class="price">$99.99</p>
                        <meter min="0" max="100" value="15">15 left</meter>
                        <button>Add to Cart</button>
                    </div>
                </div>
            </main>
            </body></html>"#.into(),
            css: r#"
                .grid { display: flex; flex-wrap: wrap; gap: 16px; }
                .card { border: 1px solid #ddd; padding: 16px; width: 220px; }
                .price { font-weight: bold; color: #006600; }
                button { background: #ff6600; color: white; border: none; padding: 8px 16px; }
            "#.into(),
            assertions: vec!["dom_has_body".into(), "dom_has_img".into(), "dom_has_button".into(), "dom_has_element:meter".into(), "has_fill_primitives".into(), "layout_has_children".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 综合页面：FAQ 折叠
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "interactive/composite/faq-accordion".into(),
            description: "FAQ 折叠列表渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <main>
                <h1>FAQ</h1>
                <details open>
                    <summary>What is ZeroWeb?</summary>
                    <p>ZeroWeb is a browser built with Rust.</p>
                </details>
                <details>
                    <summary>Is it open source?</summary>
                    <p>Yes, ZeroWeb is MIT licensed.</p>
                </details>
                <details>
                    <summary>Which platforms are supported?</summary>
                    <p>macOS, Linux, and Windows.</p>
                </details>
                <details>
                    <summary>Does it support extensions?</summary>
                    <p>Not yet, but it's planned for a future release.</p>
                </details>
            </main>
            </body></html>"#.into(),
            css: r#"
                details { border: 1px solid #ddd; margin: 4px 0; padding: 8px; }
                summary { font-weight: bold; cursor: pointer; }
            "#.into(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:details".into(), "dom_has_element:summary".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 综合页面：仪表板
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "interactive/composite/dashboard".into(),
            description: "仪表板综合渲染".into(),
            category: "interactive".into(),
            html: r##"<html><body>
            <header>
                <h1>Dashboard</h1>
                <nav>
                    <a href="#overview">Overview</a>
                    <a href="#stats">Stats</a>
                    <a href="#settings">Settings</a>
                </nav>
            </header>
            <main>
                <section id="overview">
                    <h2>Overview</h2>
                    <div class="stats">
                        <div class="stat-card">
                            <h3>Users</h3>
                            <p class="number">1,234</p>
                            <progress value="75" max="100">75%</progress>
                        </div>
                        <div class="stat-card">
                            <h3>Revenue</h3>
                            <p class="number">$56,789</p>
                            <meter min="0" max="100000" value="56789">$56,789</meter>
                        </div>
                    </div>
                </section>
                <section id="stats">
                    <h2>Statistics</h2>
                    <table>
                        <thead><tr><th>Month</th><th>Visits</th><th>Conv.</th></tr></thead>
                        <tbody>
                            <tr><td>Jan</td><td>10,000</td><td>3.2%</td></tr>
                            <tr><td>Feb</td><td>12,000</td><td>3.5%</td></tr>
                            <tr><td>Mar</td><td>15,000</td><td>4.1%</td></tr>
                        </tbody>
                    </table>
                </section>
            </main>
            <footer>
                <p>&copy; 2026 ZeroWeb</p>
            </footer>
            </body></html>"##.into(),
            css: r##"
                body { font-family: sans-serif; margin: 0; }
                header { background: #333; color: white; padding: 10px 20px; }
                header nav a { color: #ccc; margin-left: 15px; }
                .stats { display: flex; gap: 20px; margin: 20px 0; }
                .stat-card { border: 1px solid #ddd; padding: 15px; flex: 1; }
                table { width: 100%; border-collapse: collapse; }
                th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
                footer { background: #f5f5f5; padding: 10px 20px; margin-top: 20px; }
            "##.into(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_header".into(),
                "dom_has_nav".into(),
                "dom_has_element:section".into(),
                "dom_has_element:footer".into(),
                "dom_has_table".into(),
                "dom_has_element:progress".into(),
                "dom_has_element:meter".into(),
                "has_fill_primitives".into(),
                "has_glyph_primitives".into(),
                "layout_has_children".into(),
                "no_panic".into(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // 综合页面：文章页面
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "interactive/composite/article".into(),
            description: "文章页面渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <article>
                <header>
                    <h1>Building a Browser in Rust</h1>
                    <time datetime="2026-06-01">June 1, 2026</time>
                    <address>By <a href="/author">Jane Doe</a></address>
                </header>
                <p>Rust provides memory safety without garbage collection...</p>
                <h2>Architecture</h2>
                <p>The browser is organized into multiple crates...</p>
                <figure>
                    <img src="architecture.png" alt="Architecture diagram">
                    <figcaption>Figure 1: Browser architecture overview</figcaption>
                </figure>
                <h2>Performance</h2>
                <p>Benchmarks show competitive results...</p>
                <pre><code>cargo bench --workspace</code></pre>
                <aside>
                    <h3>Related Articles</h3>
                    <ul>
                        <li><a href="/rust-wasm">Rust and WebAssembly</a></li>
                        <li><a href="/css-parsing">CSS Parsing in Rust</a></li>
                    </ul>
                </aside>
                <footer>
                    <p>Tags: <a href="/tag/rust">rust</a>, <a href="/tag/browser">browser</a></p>
                </footer>
            </article>
            </body></html>"#.into(),
            css: r#"
                article { max-width: 800px; margin: 0 auto; padding: 20px; }
                h1 { font-size: 2em; margin-bottom: 0.5em; }
                h2 { margin-top: 1.5em; }
                figure { margin: 1em 0; padding: 1em; background: #f9f9f9; }
                aside { float: right; width: 250px; border-left: 2px solid #ccc; padding-left: 15px; margin-left: 15px; }
                pre { background: #2d2d2d; color: #ccc; padding: 15px; overflow: auto; }
                code { font-family: monospace; }
            "#.into(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_article".into(),
                "dom_has_header".into(),
                "dom_has_element:figure".into(),
                "dom_has_element:figcaption".into(),
                "dom_has_element:aside".into(),
                "dom_has_element:time".into(),
                "dom_has_element:address".into(),
                "has_fill_primitives".into(),
                "has_glyph_primitives".into(),
                "layout_has_children".into(),
                "no_panic".into(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // 标记和 ruby
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "interactive/text-mark-ruby".into(),
            description: "mark/ruby/abbr 等行内语义元素".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <p>This is <mark>highlighted</mark> text.</p>
            <p>The <abbr title="World Wide Web">WWW</abbr> was invented in 1989.</p>
            <ruby>
                漢<rp>(</rp><rt>kan</rt><rp>)</rp>
                字<rp>(</rp><rt>ji</rt><rp>)</rp>
            </ruby>
            <p><bdi>إيان</bdi>: 3 posts</p>
            <p><bdo dir="rtl">This text goes right-to-left</bdo></p>
            <p>Price: <del>old</del> <ins>new</ins></p>
            <p>H<sub>2</sub>O and E=mc<sup>2</sup></p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_element:mark".into(),
                "dom_has_element:ruby".into(),
                "dom_has_element:rt".into(),
                "dom_has_element:abbr".into(),
                "has_glyph_primitives".into(),
                "no_panic".into(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // 列表嵌套
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "interactive/lists/nested".into(),
            description: "深度嵌套列表渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <ul>
                <li>Item 1
                    <ol>
                        <li>Sub 1.1
                            <ul>
                                <li>Deep 1.1.1</li>
                                <li>Deep 1.1.2</li>
                            </ul>
                        </li>
                        <li>Sub 1.2</li>
                    </ol>
                </li>
                <li>Item 2</li>
                <li>Item 3
                    <dl>
                        <dt>Term 1</dt>
                        <dd>Definition 1</dd>
                        <dt>Term 2</dt>
                        <dd>Definition 2</dd>
                    </dl>
                </li>
            </ul>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_list".into(), "dom_has_element:dl".into(), "dom_has_element:dt".into(), "dom_has_element:dd".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // map / area
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "interactive/map-area".into(),
            description: "map/area 图像映射渲染".into(),
            category: "interactive".into(),
            html: r##"<html><body>
            <map name="imagemap">
                <area shape="rect" coords="0,0,100,100" href="/top-left" alt="Top Left">
                <area shape="circle" coords="150,150,50" href="/center" alt="Center">
                <area shape="poly" coords="200,0,250,100,150,100" href="/triangle" alt="Triangle">
            </map>
            <img usemap="#imagemap" src="image.png" alt="Image map example">
            </body></html>"##.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_img".into(), "dom_has_element:map".into(), "dom_has_element:area".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // script 标签变体
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "interactive/script-variants".into(),
            description: "各种 script 标签属性渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <script>console.log("inline");</script>
            <script src="app.js"></script>
            <script defer src="deferred.js"></script>
            <script async src="async.js"></script>
            <script type="module">import './module.js';</script>
            <script type="importmap">{"imports": {"lodash": "/lib/lodash.js"}}</script>
            <noscript><p>JavaScript is disabled.</p></noscript>
            <p>Page content after scripts.</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:script".into(), "dom_has_element:noscript".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // head 元素完整性
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "interactive/head-elements".into(),
            description: "完整 head 元素渲染".into(),
            category: "interactive".into(),
            html: r#"<html>
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <meta name="description" content="Test page">
                <meta name="author" content="ZeroWeb">
                <meta http-equiv="X-UA-Compatible" content="IE=edge">
                <title>Head Elements Test</title>
                <base href="https://example.com/">
                <link rel="stylesheet" href="style.css">
                <link rel="icon" href="favicon.ico">
                <link rel="canonical" href="https://example.com/page">
                <style>body { background: white; }</style>
            </head>
            <body>
                <p>Page with full head elements.</p>
            </body>
            </html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_head".into(), "dom_has_title".into(), "dom_has_meta".into(), "dom_has_element:base".into(), "dom_has_element:link".into(), "dom_has_element:style".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // CSS 样式化表单
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "interactive/styled-form".into(),
            description: "CSS 样式化表单渲染".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <form class="styled-form">
                <div class="form-group">
                    <label>Username</label>
                    <input type="text" class="form-input" placeholder="Enter username">
                </div>
                <div class="form-group">
                    <label>Password</label>
                    <input type="password" class="form-input" placeholder="Enter password">
                </div>
                <div class="form-group">
                    <label>Role</label>
                    <select class="form-select">
                        <option>Admin</option>
                        <option>User</option>
                    </select>
                </div>
                <div class="form-group">
                    <label>Bio</label>
                    <textarea class="form-textarea" rows="3" placeholder="Tell us about yourself"></textarea>
                </div>
                <button type="submit" class="btn-primary">Submit</button>
            </form>
            </body></html>"#.into(),
            css: r#"
                .styled-form { max-width: 400px; margin: 20px; }
                .form-group { margin-bottom: 15px; }
                .form-group label { display: block; margin-bottom: 5px; font-weight: bold; color: #333; }
                .form-input, .form-select, .form-textarea {
                    width: 100%; padding: 10px; border: 1px solid #ccc;
                    border-radius: 4px; font-size: 14px; box-sizing: border-box;
                }
                .form-input:focus { border-color: #0066cc; outline: none; }
                .btn-primary {
                    background: #0066cc; color: white; padding: 12px 24px;
                    border: none; border-radius: 4px; cursor: pointer;
                }
                .btn-primary:hover { background: #0052a3; }
            "#.into(),
            assertions: vec!["dom_has_body".into(), "dom_has_form".into(), "dom_has_input".into(), "dom_has_element:textarea".into(), "dom_has_element:select".into(), "dom_has_button".into(), "has_fill_primitives".into(), "has_glyph_primitives".into(), "layout_has_children".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // CSS Grid 表单布局
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "interactive/grid-form".into(),
            description: "CSS Grid 表单布局".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <form class="grid-form">
                <label class="label">First Name</label>
                <input type="text" class="field" value="John">
                <label class="label">Last Name</label>
                <input type="text" class="field" value="Doe">
                <label class="label">Email</label>
                <input type="email" class="field span-2" value="john@example.com">
                <label class="label">Phone</label>
                <input type="tel" class="field" value="+1234567890">
                <label class="label">City</label>
                <input type="text" class="field" value="Anytown">
                <div class="span-2">
                    <button type="submit">Save</button>
                    <button type="reset">Cancel</button>
                </div>
            </form>
            </body></html>"#.into(),
            css: r#"
                .grid-form {
                    display: grid;
                    grid-template-columns: 120px 1fr 120px 1fr;
                    gap: 10px;
                    max-width: 600px;
                    margin: 20px;
                }
                .label { font-weight: bold; align-self: center; }
                .field { padding: 8px; border: 1px solid #ccc; }
                .span-2 { grid-column: span 2; }
                button { padding: 8px 16px; margin-right: 10px; }
            "#.into(),
            assertions: vec!["dom_has_body".into(), "dom_has_form".into(), "has_fill_primitives".into(), "has_glyph_primitives".into(), "layout_has_children".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // CSS Flexbox 表单布局
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "interactive/flex-form".into(),
            description: "CSS Flexbox 表单布局".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <form class="flex-form">
                <div class="form-row">
                    <input type="text" placeholder="Search..." class="search-input">
                    <button type="submit" class="search-btn">Search</button>
                </div>
                <div class="form-row">
                    <select><option>All</option><option>Images</option><option>News</option></select>
                    <input type="text" placeholder="Filter" class="filter-input">
                    <input type="date" class="date-input">
                </div>
                <div class="form-row">
                    <label><input type="checkbox"> Option A</label>
                    <label><input type="checkbox"> Option B</label>
                    <label><input type="checkbox"> Option C</label>
                </div>
            </form>
            </body></html>"#.into(),
            css: r#"
                .flex-form { max-width: 600px; margin: 20px; }
                .form-row {
                    display: flex;
                    gap: 8px;
                    align-items: center;
                    margin-bottom: 8px;
                }
                .search-input { flex: 1; padding: 10px; border: 2px solid #ddd; border-radius: 20px; }
                .search-btn { padding: 10px 20px; background: #4285f4; color: white; border: none; border-radius: 20px; }
                .filter-input { flex: 1; padding: 8px; }
                .date-input { padding: 8px; }
                label { white-space: nowrap; }
            "#.into(),
            assertions: vec!["dom_has_body".into(), "dom_has_form".into(), "dom_has_input".into(), "dom_has_button".into(), "has_fill_primitives".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // HTML 错误恢复
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "interactive/error-recovery/unclosed-tags".into(),
            description: "未闭合标签错误恢复".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <p>Paragraph 1
            <p>Paragraph 2
            <ul>
                <li>Item 1
                <li>Item 2
                <li>Item 3
            </ul>
            <table>
                <tr><td>Cell
            </table>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_paragraph".into(), "dom_has_list".into(), "dom_has_table".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        TestCase {
            id: "interactive/error-recovery/invalid-nesting".into(),
            description: "无效嵌套错误恢复".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <div><p><div>Nested div inside p</div></p></div>
            <a><a>Nested links</a></a>
            <select><select></select></select>
            <form><form></form></form>
            <p>Recovery test continues here.</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        TestCase {
            id: "interactive/error-recovery/mixed-content".into(),
            description: "混合内容错误恢复".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <div>Before <script>var x = 1;</script> After</div>
            <p>Text with <!-- comment --> hidden comment</p>
            <div><span>Unclosed span<div>Block inside inline</div></span></div>
            <![CDATA[This is CDATA in HTML]]>
            <?xml version="1.0"?>
            <p>Recovery complete.</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // HTML 注释和空白处理
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "interactive/whitespace-handling".into(),
            description: "HTML 空白和注释处理".into(),
            category: "interactive".into(),
            html: r#"<html><body>
            <!-- This is a comment -->
            <p>Text with   multiple    spaces</p>
            <p>
                Text with
                newlines
                and tabs
            </p>
            <div>
                <div>
                    <div>
                        Deeply nested content
                    </div>
                </div>
            </div>
            <hr>
            <br>
            <wbr>
            <p>After void elements</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_glyph_primitives".into(), "dom_has_paragraph".into(), "dom_has_element:hr".into(), "no_panic".into()],
        },
    ]
}
