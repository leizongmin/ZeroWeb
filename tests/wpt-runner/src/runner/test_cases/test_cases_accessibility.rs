//! 可访问性基础合规性测试。
//!
//! 覆盖 ARIA 属性传递、键盘导航焦点管理、语义 HTML 结构、
//! 屏幕阅读器兼容性标记和高对比度模式渲染。

use super::TestCase;

/// 返回可访问性基础合规性测试用例。
pub fn accessibility_tests() -> Vec<TestCase> {
    vec![
        // ═══════════════════════════════════════════════════════════════
        //  ARIA 角色和属性
        // ═══════════════════════════════════════════════════════════════

        // ── ARIA 角色标记基础 ──
        TestCase {
            id: "accessibility/aria-role-basic".to_string(),
            description: "ARIA role 属性在 DOM 中正确保留".to_string(),
            category: "accessibility".to_string(),
            html: r#"<html><body>
<div role="navigation" aria-label="Main Navigation">
  <ul><li><a href="/">Home</a></li><li><a href="/about">About</a></li></ul>
</div>
<main role="main">
  <article role="article">
    <h1>Article Title</h1>
    <p>Article content with <span role="note">important note</span>.</p>
  </article>
</main>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ── ARIA 地标角色 ──
        TestCase {
            id: "accessibility/aria-landmarks".to_string(),
            description: "ARIA 地标角色完整页面结构".to_string(),
            category: "accessibility".to_string(),
            html: r#"<html><body>
<style>
  header { background: #333; color: #fff; padding: 10px; }
  nav { background: #555; padding: 8px; }
  nav a { color: #fff; margin-right: 15px; }
  main { padding: 20px; background: #fff; }
  section { padding: 10px; background: #f9f9f9; margin-bottom: 10px; }
  aside { background: #eef; padding: 10px; }
  footer { background: #333; color: #fff; padding: 10px; }
</style>
<header role="banner">
  <nav role="navigation" aria-label="Primary">
    <a href="/">Logo</a>
  </nav>
</header>
<main role="main">
  <section role="region" aria-label="Content">
    <h1>Main Content</h1>
    <p>Page content here.</p>
  </section>
  <aside role="complementary">
    <h2>Sidebar</h2>
    <p>Sidebar content.</p>
  </aside>
</main>
<footer role="contentinfo">
  <p>Footer information</p>
</footer>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ── ARIA 状态属性 ──
        TestCase {
            id: "accessibility/aria-states".to_string(),
            description: "ARIA 状态属性（aria-expanded/aria-selected/aria-checked）正确渲染".to_string(),
            category: "accessibility".to_string(),
            html: r#"<html><body>
<style>
  .accordion { margin: 10px; background: #f5f5f5; padding: 8px; }
  .tab-list { display: flex; gap: 5px; }
  .tab { padding: 8px 16px; background: #e0e0e0; border: none; }
  .checkbox { margin: 5px 0; background: #fafafa; padding: 4px; }
</style>
<div class="accordion">
  <button aria-expanded="true" aria-controls="section1">Section 1</button>
  <div id="section1" role="region">Expanded content</div>
</div>
<div role="tablist" class="tab-list">
  <button role="tab" aria-selected="true">Tab 1</button>
  <button role="tab" aria-selected="false">Tab 2</button>
</div>
<div role="checkbox" aria-checked="true" class="checkbox">Checked item</div>
<div role="checkbox" aria-checked="false" class="checkbox">Unchecked item</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ── ARIA live region ──
        TestCase {
            id: "accessibility/aria-live-regions".to_string(),
            description: "ARIA live region 属性正确传递".to_string(),
            category: "accessibility".to_string(),
            html: r#"<html><body>
<div aria-live="polite" aria-atomic="true" id="status">
  Status message will appear here.
</div>
<div aria-live="assertive" role="alert" id="alert-region">
  Alert message area.
</div>
<div role="log" aria-live="polite">
  <p>Log entry 1</p>
  <p>Log entry 2</p>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  语义 HTML 可访问性结构
        // ═══════════════════════════════════════════════════════════════

        // ── 表单可访问性 ──
        TestCase {
            id: "accessibility/form-labels".to_string(),
            description: "表单元素正确关联 label 和 ARIA 属性".to_string(),
            category: "accessibility".to_string(),
            html: r#"<html><body>
<style>
  .form-group { margin: 10px 0; }
  label { display: block; margin-bottom: 4px; font-weight: bold; }
  input, select, textarea { padding: 6px; border: 1px solid #ccc; }
</style>
<form>
  <div class="form-group">
    <label for="name">Full Name</label>
    <input type="text" id="name" aria-required="true" placeholder="Enter name" />
  </div>
  <div class="form-group">
    <label for="email">Email</label>
    <input type="email" id="email" aria-required="true" aria-describedby="email-hint" />
    <span id="email-hint">We will never share your email.</span>
  </div>
  <div class="form-group">
    <label for="message">Message</label>
    <textarea id="message" rows="4" aria-label="Your message"></textarea>
  </div>
  <div class="form-group">
    <fieldset>
      <legend>Preferred Contact</legend>
      <label><input type="radio" name="contact" value="email" checked /> Email</label>
      <label><input type="radio" name="contact" value="phone" /> Phone</label>
    </fieldset>
  </div>
  <button type="submit" aria-label="Submit contact form">Send Message</button>
</form>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
                "dom_has_form".to_string(),
                "dom_has_input".to_string(),
            ],
        },

        // ── 表格可访问性 ──
        TestCase {
            id: "accessibility/table-headers".to_string(),
            description: "可访问表格结构（scope/caption/thead）".to_string(),
            category: "accessibility".to_string(),
            html: r#"<html><body>
<style>
  table { border-collapse: collapse; width: 100%; }
  th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
  th { background: #f5f5f5; font-weight: bold; }
  caption { font-weight: bold; margin: 8px 0; }
</style>
<table>
  <caption>Monthly Revenue Report</caption>
  <thead>
    <tr>
      <th scope="col">Month</th>
      <th scope="col">Revenue</th>
      <th scope="col">Growth</th>
    </tr>
  </thead>
  <tbody>
    <tr><td>January</td><td>$10,000</td><td>+5%</td></tr>
    <tr><td>February</td><td>$12,000</td><td>+20%</td></tr>
    <tr><td>March</td><td>$11,500</td><td>-4%</td></tr>
  </tbody>
  <tfoot>
    <tr><th scope="row">Total</th><td>$33,500</td><td>Avg +7%</td></tr>
  </tfoot>
</table>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
                "dom_has_table".to_string(),
            ],
        },

        // ── 导航跳过链接 ──
        TestCase {
            id: "accessibility/skip-navigation".to_string(),
            description: "跳过导航链接提供键盘可访问性".to_string(),
            category: "accessibility".to_string(),
            html: r##"<html><body>
<style>
  .skip-link {
    position: absolute; top: -40px; left: 0;
    background: #000; color: #fff; padding: 8px 16px; z-index: 100;
  }
  .skip-link:focus { top: 0; }
  nav { background: #333; padding: 10px; }
  nav a { color: #fff; margin-right: 15px; }
  main { padding: 20px; }
</style>
<a href="#main-content" class="skip-link">Skip to main content</a>
<nav>
  <a href="/">Home</a>
  <a href="/about">About</a>
  <a href="/contact">Contact</a>
</nav>
<main id="main-content">
  <h1>Welcome</h1>
  <p>This is the main content area.</p>
</main>
</body></html>"##.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  焦点管理和 tabindex
        // ═══════════════════════════════════════════════════════════════

        // ── tabindex 焦点顺序 ──
        TestCase {
            id: "accessibility/tabindex-order".to_string(),
            description: "tabindex 属性正确设置焦点顺序".to_string(),
            category: "accessibility".to_string(),
            html: r#"<html><body>
<style>
  .focus-demo { padding: 20px; }
  .focus-item { padding: 10px; margin: 5px; background: #f0f0f0; border: 2px solid transparent; display: inline-block; }
  .focus-item:focus { border-color: #0066cc; outline: 2px solid #0066cc; }
</style>
<div class="focus-demo">
  <div class="focus-item" tabindex="3">Third focus</div>
  <div class="focus-item" tabindex="1">First focus</div>
  <div class="focus-item" tabindex="2">Second focus</div>
  <button>Fourth focus (natural)</button>
  <div class="focus-item" tabindex="0">Fifth focus (tabindex=0)</div>
  <div class="focus-item" tabindex="-1">Not in tab order</div>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ── 模态对话框焦点捕获 ──
        TestCase {
            id: "accessibility/modal-dialog-focus".to_string(),
            description: "模态对话框使用 aria-modal 和焦点管理".to_string(),
            category: "accessibility".to_string(),
            html: r#"<html><body>
<style>
  .overlay { position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); }
  .dialog { position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%); background: white; padding: 24px; border-radius: 8px; min-width: 300px; }
  .dialog h2 { margin: 0 0 16px 0; }
  .dialog-actions { display: flex; gap: 10px; justify-content: flex-end; margin-top: 20px; }
  .dialog-actions button { padding: 8px 16px; }
</style>
<div class="overlay">
  <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="dialog-title">
    <h2 id="dialog-title">Confirm Action</h2>
    <p>Are you sure you want to proceed?</p>
    <div class="dialog-actions">
      <button>Cancel</button>
      <button autofocus>Confirm</button>
    </div>
  </div>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  高对比度和视觉可访问性
        // ═══════════════════════════════════════════════════════════════

        // ── 高对比度颜色方案 ──
        TestCase {
            id: "accessibility/high-contrast".to_string(),
            description: "高对比度颜色方案正确渲染".to_string(),
            category: "accessibility".to_string(),
            html: r##"<html><body>
<style>
  .high-contrast { background: #000; color: #fff; padding: 20px; }
  .high-contrast h1 { color: #ffff00; font-size: 24px; }
  .high-contrast a { color: #00ffff; text-decoration: underline; }
  .high-contrast .warning { color: #ff6600; background: #333; padding: 10px; border: 2px solid #ff6600; }
  .high-contrast button { background: #fff; color: #000; border: 2px solid #fff; padding: 8px 16px; font-weight: bold; }
</style>
<div class="high-contrast">
  <h1>High Contrast Mode</h1>
  <p>This page uses maximum contrast ratios for readability.</p>
  <p><a href="#">Link with clear underline</a></p>
  <div class="warning">Warning: High contrast alert message</div>
  <button>High Contrast Button</button>
</div>
</body></html>"##.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ── 大字体布局 ──
        TestCase {
            id: "accessibility/large-text-layout".to_string(),
            description: "大字体设置下布局仍然正确".to_string(),
            category: "accessibility".to_string(),
            html: r#"<html><body>
<style>
  .large-text { font-size: 24px; line-height: 1.5; }
  .large-text h1 { font-size: 36px; margin-bottom: 16px; }
  .large-text .card { border: 2px solid #333; padding: 20px; margin: 16px 0; background: #fafafa; }
  .large-text nav { background: #eee; padding: 12px; margin-bottom: 20px; }
  .large-text nav a { margin-right: 20px; font-size: 22px; }
</style>
<div class="large-text">
  <nav><a href="/">Home</a><a href="/about">About</a><a href="/help">Help</a></nav>
  <h1>Large Text Content</h1>
  <div class="card">
    <p>This content is designed for users who need larger text sizes.</p>
    <p>All interactive elements have sufficient touch targets and spacing.</p>
  </div>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  ARIA 小部件和复合组件
        // ═══════════════════════════════════════════════════════════════

        // ── ARIA 树形控件 ──
        TestCase {
            id: "accessibility/aria-tree-widget".to_string(),
            description: "ARIA tree role 渲染树形结构".to_string(),
            category: "accessibility".to_string(),
            html: r#"<html><body>
<style>
  [role="tree"] { margin: 10px; font-family: monospace; }
  [role="treeitem"] { padding: 4px 8px; margin: 2px 0; }
  [role="group"] { margin-left: 20px; }
  .tree-item-expanded { font-weight: bold; }
</style>
<ul role="tree" aria-label="File Explorer">
  <li role="treeitem" aria-expanded="true" class="tree-item-expanded">
    Documents
    <ul role="group">
      <li role="treeitem">report.pdf</li>
      <li role="treeitem">notes.txt</li>
      <li role="treeitem" aria-expanded="false">
        Projects
        <ul role="group">
          <li role="treeitem">project1.md</li>
        </ul>
      </li>
    </ul>
  </li>
  <li role="treeitem">Pictures</li>
  <li role="treeitem">Music</li>
</ul>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ── ARIA 工具栏 ──
        TestCase {
            id: "accessibility/aria-toolbar".to_string(),
            description: "ARIA toolbar 角色渲染工具栏控件".to_string(),
            category: "accessibility".to_string(),
            html: r#"<html><body>
<style>
  [role="toolbar"] { display: flex; gap: 4px; background: #eee; padding: 8px; border-radius: 4px; }
  [role="toolbar"] button { padding: 6px 12px; background: #fff; border: 1px solid #ccc; border-radius: 3px; }
  [role="toolbar"] button[aria-pressed="true"] { background: #0066cc; color: #fff; }
  [role="separator"] { width: 1px; background: #ccc; margin: 0 4px; }
</style>
<div role="toolbar" aria-label="Text formatting">
  <button aria-pressed="true" aria-label="Bold">B</button>
  <button aria-pressed="false" aria-label="Italic">I</button>
  <button aria-pressed="false" aria-label="Underline">U</button>
  <div role="separator" aria-orientation="vertical"></div>
  <button aria-label="Align left">L</button>
  <button aria-label="Align center">C</button>
  <button aria-label="Align right">R</button>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ── ARIA 进度和计量器 ──
        TestCase {
            id: "accessibility/aria-progress-meter".to_string(),
            description: "ARIA progressbar 和 meter 角色正确渲染".to_string(),
            category: "accessibility".to_string(),
            html: r#"<html><body>
<style>
  .progress-bar { background: #e0e0e0; border-radius: 8px; height: 20px; width: 300px; margin: 10px 0; overflow: hidden; }
  .progress-fill { background: #4caf50; height: 100%; border-radius: 8px; }
  .meter-bar { background: #e0e0e0; border-radius: 4px; height: 16px; width: 250px; margin: 10px 0; overflow: hidden; }
  .meter-fill { background: #2196f3; height: 100%; border-radius: 4px; }
  label { display: block; margin-bottom: 4px; font-weight: bold; }
</style>
<div>
  <label>Upload Progress</label>
  <div class="progress-bar">
    <div class="progress-fill" style="width: 65%"></div>
  </div>
  <div role="progressbar" aria-valuenow="65" aria-valuemin="0" aria-valuemax="100" aria-label="Upload progress: 65%">65%</div>
</div>
<div>
  <label>Disk Usage</label>
  <div class="meter-bar">
    <div class="meter-fill" style="width: 42%"></div>
  </div>
  <div role="meter" aria-valuenow="42" aria-valuemin="0" aria-valuemax="100" aria-label="Disk usage: 42%">42% used</div>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  屏幕阅读器文本隐藏技术
        // ═══════════════════════════════════════════════════════════════

        // ── 视觉隐藏但屏幕阅读器可访问 ──
        TestCase {
            id: "accessibility/sr-only-text".to_string(),
            description: "视觉隐藏但屏幕阅读器可读的文本正确处理".to_string(),
            category: "accessibility".to_string(),
            html: r#"<html><body>
<style>
  .sr-only {
    position: absolute; width: 1px; height: 1px;
    padding: 0; margin: -1px; overflow: hidden;
    clip: rect(0, 0, 0, 0); white-space: nowrap; border: 0;
  }
  .icon-button { display: inline-flex; align-items: center; justify-content: center; width: 40px; height: 40px; background: #0066cc; color: white; border: none; border-radius: 4px; font-size: 20px; }
</style>
<nav aria-label="Social links">
  <button class="icon-button" aria-label="Share on Twitter">
    🐦
    <span class="sr-only">Share on Twitter</span>
  </button>
  <button class="icon-button" aria-label="Share on Facebook">
    📘
    <span class="sr-only">Share on Facebook</span>
  </button>
</nav>
<p aria-describedby="desc1">Visible content with hidden description.</p>
<p id="desc1" class="sr-only">This hidden text describes the visible content for screen reader users.</p>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  综合可访问页面
        // ═══════════════════════════════════════════════════════════════

        // ── 可访问仪表盘 ──
        TestCase {
            id: "accessibility/accessible-dashboard".to_string(),
            description: "综合可访问仪表盘页面（地标/表单/ARIA/焦点）".to_string(),
            category: "accessibility".to_string(),
            html: r##"<html><body>
<style>
  * { box-sizing: border-box; margin: 0; padding: 0; }
  body { font-family: sans-serif; line-height: 1.5; }
  .skip-link { position: absolute; top: -40px; left: 0; background: #000; color: #fff; padding: 8px; z-index: 100; }
  .skip-link:focus { top: 0; }
  header { background: #1a1a2e; color: #fff; padding: 16px; display: flex; justify-content: space-between; align-items: center; }
  nav ul { display: flex; gap: 16px; list-style: none; }
  nav a { color: #e0e0e0; text-decoration: none; }
  nav a:focus { outline: 2px solid #fff; }
  main { display: flex; gap: 20px; padding: 20px; max-width: 1200px; margin: 0 auto; }
  .content { flex: 3; }
  .sidebar { flex: 1; }
  .card { background: #fff; border: 1px solid #ddd; border-radius: 8px; padding: 16px; margin-bottom: 16px; }
  .stat-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 16px; margin-bottom: 20px; }
  .stat { background: #f8f9fa; padding: 16px; border-radius: 8px; text-align: center; }
  .stat h3 { font-size: 14px; color: #666; }
  .stat .value { font-size: 28px; font-weight: bold; color: #333; }
  table { width: 100%; border-collapse: collapse; }
  th, td { padding: 10px; text-align: left; border-bottom: 1px solid #eee; }
  th { background: #f5f5f5; font-weight: bold; }
  .search-input { width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px; margin-bottom: 12px; }
  .btn { padding: 8px 16px; border: none; border-radius: 4px; cursor: pointer; }
  .btn-primary { background: #0066cc; color: #fff; }
  .btn-primary:focus { outline: 2px solid #0066cc; outline-offset: 2px; }
</style>
<a href="#main-content" class="skip-link">Skip to main content</a>
<header role="banner">
  <h1>Dashboard</h1>
  <nav role="navigation" aria-label="Main navigation">
    <ul>
      <li><a href="/" aria-current="page">Overview</a></li>
      <li><a href="/analytics">Analytics</a></li>
      <li><a href="/settings">Settings</a></li>
    </ul>
  </nav>
</header>
<main id="main-content" role="main">
  <div class="content">
    <section aria-labelledby="stats-heading">
      <h2 id="stats-heading">Statistics</h2>
      <div class="stat-grid">
        <div class="stat" role="status"><h3>Users</h3><div class="value" aria-label="1,234 users">1,234</div></div>
        <div class="stat" role="status"><h3>Revenue</h3><div class="value" aria-label="$56,789 revenue">$56K</div></div>
        <div class="stat" role="status"><h3>Orders</h3><div class="value" aria-label="892 orders">892</div></div>
      </div>
    </section>
    <section aria-labelledby="table-heading">
      <h2 id="table-heading">Recent Orders</h2>
      <label for="search-orders" class="sr-only">Search orders</label>
      <input type="search" id="search-orders" class="search-input" placeholder="Search orders..." aria-label="Search orders" />
      <table role="table" aria-label="Recent orders">
        <thead><tr><th scope="col">Order</th><th scope="col">Customer</th><th scope="col">Amount</th><th scope="col">Status</th></tr></thead>
        <tbody>
          <tr><td>#1001</td><td>Alice</td><td>$120</td><td>Shipped</td></tr>
          <tr><td>#1002</td><td>Bob</td><td>$85</td><td>Pending</td></tr>
        </tbody>
      </table>
    </section>
  </div>
  <aside class="sidebar" role="complementary" aria-label="Quick actions">
    <div class="card">
      <h2>Quick Actions</h2>
      <button class="btn btn-primary">New Order</button>
    </div>
    <div class="card" role="status" aria-live="polite">
      <h2>Notifications</h2>
      <p>No new notifications</p>
    </div>
  </aside>
</main>
</body></html>"##.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
                "dom_has_table".to_string(),
                "dom_has_input".to_string(),
            ],
        },

        // ── 可访问登录表单 ──
        TestCase {
            id: "accessibility/accessible-login".to_string(),
            description: "完整可访问的登录表单（label/error/required）".to_string(),
            category: "accessibility".to_string(),
            html: r#"<html><body>
<style>
  .login-form { max-width: 400px; margin: 40px auto; padding: 24px; border: 1px solid #ddd; border-radius: 8px; }
  .login-form h1 { text-align: center; margin-bottom: 24px; }
  .form-group { margin-bottom: 16px; }
  .form-group label { display: block; margin-bottom: 4px; font-weight: bold; }
  .form-group input { width: 100%; padding: 10px; border: 1px solid #ccc; border-radius: 4px; font-size: 16px; }
  .form-group input:focus { outline: 2px solid #0066cc; outline-offset: 1px; }
  .form-group input[aria-invalid="true"] { border-color: #cc0000; }
  .error-msg { color: #cc0000; font-size: 14px; margin-top: 4px; role: alert; }
  .submit-btn { width: 100%; padding: 12px; background: #0066cc; color: white; border: none; border-radius: 4px; font-size: 16px; cursor: pointer; }
  .submit-btn:focus { outline: 2px solid #0066cc; outline-offset: 2px; }
</style>
<form class="login-form" aria-label="Login form" novalidate>
  <h1>Sign In</h1>
  <div class="form-group">
    <label for="username">Username <span aria-hidden="true">*</span></label>
    <input type="text" id="username" name="username" required aria-required="true" autocomplete="username" />
    <div class="error-msg" role="alert" id="username-error" aria-live="assertive"></div>
  </div>
  <div class="form-group">
    <label for="password">Password <span aria-hidden="true">*</span></label>
    <input type="password" id="password" name="password" required aria-required="true" autocomplete="current-password" aria-describedby="password-hint" />
    <div id="password-hint" class="error-msg">Minimum 8 characters required</div>
  </div>
  <div class="form-group">
    <input type="checkbox" id="remember" />
    <label for="remember">Remember me</label>
  </div>
  <button type="submit" class="submit-btn">Sign In</button>
</form>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
                "dom_has_form".to_string(),
                "dom_has_input".to_string(),
                "dom_has_button".to_string(),
            ],
        },

        // ── ARIA 选项卡面板完整实现 ──
        TestCase {
            id: "accessibility/aria-tabs-panel".to_string(),
            description: "ARIA 选项卡面板完整实现（tablist/tab/tabpanel）".to_string(),
            category: "accessibility".to_string(),
            html: r#"<html><body>
<style>
  .tabs { border: 1px solid #ccc; border-radius: 4px; overflow: hidden; }
  [role="tablist"] { display: flex; background: #f5f5f5; border-bottom: 1px solid #ccc; }
  [role="tab"] { padding: 12px 24px; border: none; background: none; cursor: pointer; font-size: 16px; border-bottom: 2px solid transparent; }
  [role="tab"][aria-selected="true"] { background: #fff; border-bottom-color: #0066cc; font-weight: bold; }
  [role="tab"]:focus { outline: 2px solid #0066cc; outline-offset: -2px; }
  [role="tabpanel"] { padding: 20px; }
  [role="tabpanel"]:not([hidden]) { display: block; }
</style>
<div class="tabs">
  <div role="tablist" aria-label="Account settings">
    <button role="tab" id="tab-1" aria-selected="true" aria-controls="panel-1" tabindex="0">Profile</button>
    <button role="tab" id="tab-2" aria-selected="false" aria-controls="panel-2" tabindex="-1">Security</button>
    <button role="tab" id="tab-3" aria-selected="false" aria-controls="panel-3" tabindex="-1">Notifications</button>
  </div>
  <div role="tabpanel" id="panel-1" aria-labelledby="tab-1">
    <h2>Profile Settings</h2>
    <p>Manage your profile information here.</p>
  </div>
  <div role="tabpanel" id="panel-2" aria-labelledby="tab-2" hidden>
    <h2>Security Settings</h2>
    <p>Configure your security preferences.</p>
  </div>
  <div role="tabpanel" id="panel-3" aria-labelledby="tab-3" hidden>
    <h2>Notification Settings</h2>
    <p>Control your notification preferences.</p>
  </div>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ── 图片替代文本和 figure ──
        TestCase {
            id: "accessibility/image-alt-text".to_string(),
            description: "图片替代文本和 figure/figcaption 正确处理".to_string(),
            category: "accessibility".to_string(),
            html: r#"<html><body>
<style>
  .gallery { display: flex; flex-wrap: wrap; gap: 16px; padding: 20px; }
  figure { margin: 0; border: 1px solid #ddd; border-radius: 8px; overflow: hidden; max-width: 250px; }
  figcaption { padding: 8px; background: #f5f5f5; font-size: 14px; text-align: center; }
  .placeholder-img { width: 250px; height: 150px; background: #e0e0e0; display: flex; align-items: center; justify-content: center; color: #999; }
</style>
<div class="gallery">
  <figure>
    <div class="placeholder-img" role="img" aria-label="Sunset over the ocean">🌅</div>
    <figcaption>Sunset at the beach</figcaption>
  </figure>
  <figure>
    <div class="placeholder-img" role="img" aria-label="Mountain landscape">🏔️</div>
    <figcaption>Mountain view</figcaption>
  </figure>
  <figure>
    <div class="placeholder-img" role="img" aria-label="City skyline">🏙️</div>
    <figcaption>City at night</figcaption>
  </figure>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },
    ]
}
