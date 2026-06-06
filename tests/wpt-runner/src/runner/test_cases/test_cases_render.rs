//! 渲染管线高级合规性测试。
//!
//! 测试完整的 CSS → Style → Layout → Paint → Composite 管线，
//! 验证多属性组合、复杂布局、层叠上下文等高级场景。

use super::TestCase;

/// 返回渲染管线高级合规性测试用例。
fn render_layout_tests() -> Vec<TestCase> {
    vec![
        // ═══════════════════════════════════════════════════════════════
        //  多属性组合渲染
        // ═══════════════════════════════════════════════════════════════

        // ── border-radius + box-shadow 组合 ──
        TestCase {
            id: "render/border-radius-shadow".to_string(),
            description: "border-radius with box-shadow combination".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div id="card" style="width:200px;height:100px;background:#fff;border-radius:12px;box-shadow:0 4px 12px rgba(0,0,0,0.15)">Card</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── 多层渐变背景 ──
        TestCase {
            id: "render/multi-gradient-bg".to_string(),
            description: "Multiple gradient backgrounds layered".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div id="hero" style="width:400px;height:200px;background:linear-gradient(135deg,#667eea 0%,#764ba2 100%)">Hero Section</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "gradient_count_ge:1".to_string(),
            ],
        },

        // ── CSS Grid 嵌套布局 ──
        TestCase {
            id: "render/nested-grid".to_string(),
            description: "Nested CSS Grid layout".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div id="outer" style="display:grid;grid-template-columns:1fr 2fr;gap:16px;width:600px">
  <div id="sidebar" style="background:#f0f0f0;padding:16px">
    <div style="display:grid;grid-template-columns:1fr 1fr;gap:8px">
      <div style="background:#ddd;height:40px">S1</div>
      <div style="background:#ddd;height:40px">S2</div>
    </div>
  </div>
  <div id="main" style="background:#fafafa;padding:16px">
    <div style="display:grid;grid-template-columns:repeat(3,1fr);gap:8px">
      <div style="background:#e0e0e0;height:60px">M1</div>
      <div style="background:#e0e0e0;height:60px">M2</div>
      <div style="background:#e0e0e0;height:60px">M3</div>
    </div>
  </div>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "layout_has_children".to_string(),
            ],
        },

        // ── Flexbox 嵌套：水平导航 + 垂直内容 ──
        TestCase {
            id: "render/flex-nav-content".to_string(),
            description: "Flexbox horizontal nav with vertical content".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<nav id="navbar" style="display:flex;justify-content:space-between;align-items:center;background:#333;color:#fff;padding:8px 16px">
  <div class="logo">Brand</div>
  <div style="display:flex;gap:16px">
    <a style="color:#fff">Home</a>
    <a style="color:#fff">About</a>
    <a style="color:#fff">Contact</a>
  </div>
</nav>
<main style="display:flex;flex-direction:column;gap:16px;padding:16px">
  <div style="background:#f5f5f5;padding:16px;border-radius:8px">Section 1</div>
  <div style="background:#f5f5f5;padding:16px;border-radius:8px">Section 2</div>
</main>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "layout_has_children".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  z-index 层叠上下文
        // ═══════════════════════════════════════════════════════════════

        // ── z-index 叠放顺序 ──
        TestCase {
            id: "render/z-index-stacking".to_string(),
            description: "z-index stacking order verification".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="position:relative;width:300px;height:200px">
  <div style="position:absolute;top:0;left:0;width:100px;height:100px;background:red;z-index:3">Top</div>
  <div style="position:absolute;top:20px;left:20px;width:100px;height:100px;background:green;z-index:2">Middle</div>
  <div style="position:absolute;top:40px;left:40px;width:100px;height:100px;background:blue;z-index:1">Bottom</div>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "fill_count_ge:3".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  响应式布局模式
        // ═══════════════════════════════════════════════════════════════

        // ── 圣杯布局 ──
        TestCase {
            id: "render/holy-grail".to_string(),
            description: "Holy Grail layout with flexbox".to_string(),
            category: "css".to_string(),
            html: r#"<html><body style="margin:0">
<div id="container" style="display:flex;flex-direction:column;min-height:100vh">
  <header style="background:#2c3e50;color:#fff;padding:16px">Header</header>
  <div style="display:flex;flex:1">
    <nav style="width:200px;background:#ecf0f1;padding:16px">Nav</nav>
    <main style="flex:1;padding:16px">
      <article style="background:#fff;padding:16px;border:1px solid #ddd">Content</article>
    </main>
    <aside style="width:200px;background:#ecf0f1;padding:16px">Sidebar</aside>
  </div>
  <footer style="background:#2c3e50;color:#fff;padding:8px;text-align:center">Footer</footer>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "layout_has_children".to_string(),
            ],
        },

        // ── 卡片网格 ──
        TestCase {
            id: "render/card-grid".to_string(),
            description: "Responsive card grid with auto-fill".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div id="grid" style="display:grid;grid-template-columns:repeat(auto-fill,minmax(250px,1fr));gap:16px;padding:16px">
  <div class="card" style="background:#fff;border:1px solid #e0e0e0;border-radius:8px;padding:16px">
    <h3>Card 1</h3><p>Description text</p>
  </div>
  <div class="card" style="background:#fff;border:1px solid #e0e0e0;border-radius:8px;padding:16px">
    <h3>Card 2</h3><p>Description text</p>
  </div>
  <div class="card" style="background:#fff;border:1px solid #e0e0e0;border-radius:8px;padding:16px">
    <h3>Card 3</h3><p>Description text</p>
  </div>
  <div class="card" style="background:#fff;border:1px solid #e0e0e0;border-radius:8px;padding:16px">
    <h3>Card 4</h3><p>Description text</p>
  </div>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "layout_has_children".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  文本渲染
        // ═══════════════════════════════════════════════════════════════

        // ── 多行文本截断 ──
        TestCase {
            id: "render/text-multiline".to_string(),
            description: "Multi-line text rendering with overflow".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:200px;overflow:hidden;padding:8px;background:#f9f9f9">
  <p style="margin:0;font-size:14px;line-height:1.5">This is a longer paragraph of text that should wrap across multiple lines within the constrained width container.</p>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "glyph_count_ge:1".to_string(),
            ],
        },

        // ── 混合字号文本 ──
        TestCase {
            id: "render/mixed-font-sizes".to_string(),
            description: "Mixed font sizes in same container".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="padding:16px">
  <h1 style="font-size:32px;margin:0 0 8px">Heading</h1>
  <h2 style="font-size:24px;margin:0 0 8px">Subheading</h2>
  <p style="font-size:16px;margin:0 0 8px">Body text at normal size.</p>
  <small style="font-size:12px">Small print text</small>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "glyph_count_ge:1".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS 变量 + 自定义属性
        // ═══════════════════════════════════════════════════════════════

        // ── CSS 变量主题 ──
        TestCase {
            id: "render/css-vars-theme".to_string(),
            description: "CSS custom properties for theming".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div class="theme-light" style="padding:16px">
  <div class="card">Themed Card</div>
</div>
</body></html>"#.to_string(),
            css: r#"
.theme-light {
    --bg: #ffffff;
    --fg: #333333;
    --accent: #3b82f6;
    --border: #e5e7eb;
    --radius: 8px;
}
.card {
    background: var(--bg);
    color: var(--fg);
    border: 2px solid var(--accent);
    border-radius: var(--radius);
    padding: 16px;
}"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  综合页面测试
        // ═══════════════════════════════════════════════════════════════

        // ── 登录页面 ──
        TestCase {
            id: "render/login-page".to_string(),
            description: "Login page with centered form".to_string(),
            category: "css".to_string(),
            html: r#"<html><body style="margin:0;min-height:100vh;display:flex;align-items:center;justify-content:center;background:linear-gradient(135deg,#667eea 0%,#764ba2 100%)">
<div id="login-card" style="background:#fff;border-radius:12px;padding:32px;width:360px;box-shadow:0 20px 60px rgba(0,0,0,0.3)">
  <h2 style="margin:0 0 24px;text-align:center;color:#333">Sign In</h2>
  <form style="display:flex;flex-direction:column;gap:16px">
    <input type="email" placeholder="Email" style="padding:12px;border:1px solid #ddd;border-radius:6px;font-size:16px">
    <input type="password" placeholder="Password" style="padding:12px;border:1px solid #ddd;border-radius:6px;font-size:16px">
    <button type="submit" style="padding:12px;background:#667eea;color:#fff;border:none;border-radius:6px;font-size:16px;cursor:pointer">Sign In</button>
  </form>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "gradient_count_ge:1".to_string(),
                "layout_has_children".to_string(),
            ],
        },

        // ── 仪表盘布局 ──
        TestCase {
            id: "render/dashboard".to_string(),
            description: "Dashboard layout with grid + flexbox".to_string(),
            category: "css".to_string(),
            html: r#"<html><body style="margin:0;font-family:system-ui">
<div style="display:flex;min-height:100vh">
  <aside style="width:240px;background:#1e293b;color:#fff;padding:16px">
    <div style="font-size:20px;font-weight:bold;margin-bottom:24px">Dashboard</div>
    <nav style="display:flex;flex-direction:column;gap:8px">
      <a style="color:#94a3b8;padding:8px;border-radius:4px">Overview</a>
      <a style="color:#94a3b8;padding:8px;border-radius:4px">Analytics</a>
      <a style="color:#94a3b8;padding:8px;border-radius:4px">Settings</a>
    </nav>
  </aside>
  <main style="flex:1;padding:24px;background:#f1f5f9">
    <h1 style="margin:0 0 24px;font-size:24px">Overview</h1>
    <div style="display:grid;grid-template-columns:repeat(3,1fr);gap:16px;margin-bottom:24px">
      <div style="background:#fff;padding:20px;border-radius:8px;box-shadow:0 1px 3px rgba(0,0,0,0.1)">
        <div style="font-size:14px;color:#64748b">Revenue</div>
        <div style="font-size:28px;font-weight:bold">$45,231</div>
      </div>
      <div style="background:#fff;padding:20px;border-radius:8px;box-shadow:0 1px 3px rgba(0,0,0,0.1)">
        <div style="font-size:14px;color:#64748b">Users</div>
        <div style="font-size:28px;font-weight:bold">2,350</div>
      </div>
      <div style="background:#fff;padding:20px;border-radius:8px;box-shadow:0 1px 3px rgba(0,0,0,0.1)">
        <div style="font-size:14px;color:#64748b">Orders</div>
        <div style="font-size:28px;font-weight:bold">1,247</div>
      </div>
    </div>
  </main>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "layout_has_children".to_string(),
                "fill_count_ge:5".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Transform + Opacity
        // ═══════════════════════════════════════════════════════════════

        // ── 2D Transform 组合 ──
        TestCase {
            id: "render/transform-combo".to_string(),
            description: "Combined 2D transforms".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:100px;height:100px;background:red;transform:translate(50px,20px) rotate(45deg) scale(1.5)">Transformed</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ── Opacity 半透明叠加 ──
        TestCase {
            id: "render/opacity-overlay".to_string(),
            description: "Opacity overlay elements".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="position:relative;width:200px;height:200px">
  <div style="position:absolute;width:200px;height:200px;background:red"></div>
  <div style="position:absolute;width:150px;height:150px;top:25px;left:25px;background:blue;opacity:0.5">Overlay</div>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "fill_count_ge:2".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Overflow + Clipping
        // ═══════════════════════════════════════════════════════════════

        // ── overflow:hidden 裁剪 ──
        TestCase {
            id: "render/overflow-hidden-clip".to_string(),
            description: "overflow:hidden clipping content".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:150px;height:100px;overflow:hidden;border:1px solid #ccc;padding:8px">
  <div style="width:300px;height:200px;background:linear-gradient(45deg,#ff6b6b,#feca57)">This content overflows the container</div>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Table 布局
        // ═══════════════════════════════════════════════════════════════

        // ── 表格布局 ──
        TestCase {
            id: "render/table-layout".to_string(),
            description: "HTML table rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<table style="width:100%;border-collapse:collapse">
  <thead>
    <tr style="background:#f1f5f9">
      <th style="padding:12px;text-align:left;border-bottom:2px solid #e2e8f0">Name</th>
      <th style="padding:12px;text-align:left;border-bottom:2px solid #e2e8f0">Role</th>
      <th style="padding:12px;text-align:left;border-bottom:2px solid #e2e8f0">Status</th>
    </tr>
  </thead>
  <tbody>
    <tr><td style="padding:12px;border-bottom:1px solid #e2e8f0">Alice</td><td style="padding:12px;border-bottom:1px solid #e2e8f0">Admin</td><td style="padding:12px;border-bottom:1px solid #e2e8f0">Active</td></tr>
    <tr><td style="padding:12px;border-bottom:1px solid #e2e8f0">Bob</td><td style="padding:12px;border-bottom:1px solid #e2e8f0">User</td><td style="padding:12px;border-bottom:1px solid #e2e8f0">Inactive</td></tr>
    <tr><td style="padding:12px">Carol</td><td style="padding:12px">Editor</td><td style="padding:12px">Active</td></tr>
  </tbody>
</table>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "glyph_count_ge:1".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  媒体查询
        // ═══════════════════════════════════════════════════════════════

        // ── @media 宽度条件 ──
        TestCase {
            id: "render/media-query-width".to_string(),
            description: "@media query with width condition".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div id="container" class="responsive">
  <div class="sidebar">Sidebar</div>
  <div class="content">Content</div>
</div>
</body></html>"#.to_string(),
            css: r#"
.responsive { display: flex; }
.sidebar { width: 200px; background: #e0e0e0; padding: 16px; }
.content { flex: 1; padding: 16px; }
@media (max-width: 768px) {
    .responsive { flex-direction: column; }
    .sidebar { width: 100%; }
}"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "layout_has_children".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS Grid 高级
        // ═══════════════════════════════════════════════════════════════

        // ── Grid 模板区域 ──
        TestCase {
            id: "render/grid-template-areas".to_string(),
            description: "CSS Grid with template areas".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div id="grid" style="display:grid;grid-template-areas:'header header header' 'sidebar main aside' 'footer footer footer';grid-template-columns:200px 1fr 200px;grid-template-rows:auto 1fr auto;gap:8px;min-height:400px">
  <div style="grid-area:header;background:#2c3e50;color:#fff;padding:12px">Header</div>
  <div style="grid-area:sidebar;background:#ecf0f1;padding:12px">Sidebar</div>
  <div style="grid-area:main;background:#fff;padding:12px;border:1px solid #ddd">Main Content</div>
  <div style="grid-area:aside;background:#ecf0f1;padding:12px">Aside</div>
  <div style="grid-area:footer;background:#2c3e50;color:#fff;padding:8px;text-align:center">Footer</div>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "layout_has_children".to_string(),
                "fill_count_ge:5".to_string(),
            ],
        },

        // ── Grid auto-fill minmax ──
        TestCase {
            id: "render/grid-auto-fill-minmax".to_string(),
            description: "Grid auto-fill with minmax responsive".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(180px,1fr));gap:12px;padding:16px">
  <div style="background:#e3f2fd;padding:16px;border-radius:8px">Item 1</div>
  <div style="background:#e8f5e9;padding:16px;border-radius:8px">Item 2</div>
  <div style="background:#fff3e0;padding:16px;border-radius:8px">Item 3</div>
  <div style="background:#fce4ec;padding:16px;border-radius:8px">Item 4</div>
  <div style="background:#f3e5f5;padding:16px;border-radius:8px">Item 5</div>
  <div style="background:#e0f7fa;padding:16px;border-radius:8px">Item 6</div>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "layout_has_children".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Box Model 精确
        // ═══════════════════════════════════════════════════════════════

        // ── box-sizing:border-box ──
        TestCase {
            id: "render/box-sizing-border-box".to_string(),
            description: "box-sizing:border-box with padding and border".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:200px;padding:20px;border:5px solid #333;background:#f0f0f0;box-sizing:border-box">
  Box sizing test
</div>
<div style="width:200px;padding:20px;border:5px solid #666;background:#e0e0e0;box-sizing:content-box;margin-top:8px">
  Content box comparison
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "fill_count_ge:2".to_string(),
            ],
        },

        // ── margin 折叠 ──
        TestCase {
            id: "render/margin-collapse".to_string(),
            description: "Vertical margin collapsing between siblings".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="background:#ff6b6b;margin-bottom:30px;padding:8px">Block 1 (margin-bottom:30px)</div>
<div style="background:#4ecdc4;margin-top:20px;padding:8px">Block 2 (margin-top:20px)</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "fill_count_ge:2".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Position 模式
        // ═══════════════════════════════════════════════════════════════

        // ── Sticky 定位 ──
        TestCase {
            id: "render/sticky-header".to_string(),
            description: "Sticky positioned header".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="height:50px;background:#eee">Spacer</div>
<div style="position:sticky;top:0;background:#fff;border-bottom:2px solid #333;padding:12px;z-index:10">Sticky Header</div>
<div style="height:500px;padding:16px">
  <p>Content that scrolls under the sticky header</p>
  <p>More content lines</p>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ── Fixed 定位导航栏 ──
        TestCase {
            id: "render/fixed-navbar".to_string(),
            description: "Fixed position navigation bar".to_string(),
            category: "css".to_string(),
            html: r#"<html><body style="margin:0">
<nav style="position:fixed;top:0;left:0;right:0;height:48px;background:#1a1a2e;color:#fff;display:flex;align-items:center;padding:0 16px;z-index:100">Fixed Nav</nav>
<main style="margin-top:64px;padding:16px">
  <div style="height:1000px;background:#f5f5f5;padding:16px">Scrollable content area</div>
</main>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "layout_has_children".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  视觉效果组合
        // ═══════════════════════════════════════════════════════════════

        // ── Filter + Transform ──
        TestCase {
            id: "render/filter-transform".to_string(),
            description: "CSS filter combined with transform".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="display:flex;gap:16px;padding:16px">
  <div style="width:100px;height:100px;background:#e74c3c;filter:blur(2px)">Blurred</div>
  <div style="width:100px;height:100px;background:#3498db;filter:grayscale(100%)">Grayscale</div>
  <div style="width:100px;height:100px;background:#2ecc71;transform:scale(0.8)">Scaled</div>
  <div style="width:100px;height:100px;background:#f39c12;transform:rotate(15deg)">Rotated</div>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "fill_count_ge:4".to_string(),
            ],
        },

        // ── Radial gradient 按钮 ──
        TestCase {
            id: "render/radial-gradient-button".to_string(),
            description: "Button with radial gradient".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="display:flex;gap:12px;padding:24px">
  <button style="padding:12px 24px;border:none;border-radius:6px;background:radial-gradient(circle,#ff6b6b,#ee5a24);color:#fff;font-size:16px;cursor:pointer">Button A</button>
  <button style="padding:12px 24px;border:none;border-radius:6px;background:radial-gradient(circle,#4ecdc4,#2d98da);color:#fff;font-size:16px;cursor:pointer">Button B</button>
  <button style="padding:12px 24px;border:none;border-radius:20px;background:radial-gradient(circle,#a29bfe,#6c5ce7);color:#fff;font-size:16px;cursor:pointer">Pill Button</button>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "gradient_count_ge:1".to_string(),
            ],
        },

        // ── 产品定价页面 ──
        TestCase {
            id: "render/pricing-page".to_string(),
            description: "Pricing page with Grid cards".to_string(),
            category: "css".to_string(),
            html: r#"<html><body style="margin:0;font-family:system-ui;background:#f8fafc">
<div style="max-width:960px;margin:0 auto;padding:48px 16px">
  <h1 style="text-align:center;margin:0 0 32px">Pricing Plans</h1>
  <div style="display:grid;grid-template-columns:repeat(3,1fr);gap:24px">
    <div style="background:#fff;border:1px solid #e2e8f0;border-radius:12px;padding:32px;text-align:center">
      <h3 style="margin:0 0 8px;color:#64748b">Basic</h3>
      <div style="font-size:36px;font-weight:bold;margin:0 0 16px">$9</div>
      <ul style="list-style:none;padding:0;margin:0 0 24px;text-align:left">
        <li style="padding:4px 0">1 User</li>
        <li style="padding:4px 0">5GB Storage</li>
      </ul>
      <button style="width:100%;padding:12px;background:#e2e8f0;border:none;border-radius:6px;font-size:16px">Choose</button>
    </div>
    <div style="background:#fff;border:2px solid #3b82f6;border-radius:12px;padding:32px;text-align:center;box-shadow:0 4px 12px rgba(59,130,246,0.15)">
      <h3 style="margin:0 0 8px;color:#3b82f6">Pro</h3>
      <div style="font-size:36px;font-weight:bold;margin:0 0 16px">$29</div>
      <ul style="list-style:none;padding:0;margin:0 0 24px;text-align:left">
        <li style="padding:4px 0">10 Users</li>
        <li style="padding:4px 0">50GB Storage</li>
      </ul>
      <button style="width:100%;padding:12px;background:#3b82f6;color:#fff;border:none;border-radius:6px;font-size:16px">Choose</button>
    </div>
    <div style="background:#fff;border:1px solid #e2e8f0;border-radius:12px;padding:32px;text-align:center">
      <h3 style="margin:0 0 8px;color:#64748b">Enterprise</h3>
      <div style="font-size:36px;font-weight:bold;margin:0 0 16px">$99</div>
      <ul style="list-style:none;padding:0;margin:0 0 24px;text-align:left">
        <li style="padding:4px 0">Unlimited</li>
        <li style="padding:4px 0">500GB Storage</li>
      </ul>
      <button style="width:100%;padding:12px;background:#e2e8f0;border:none;border-radius:6px;font-size:16px">Choose</button>
    </div>
  </div>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "layout_has_children".to_string(),
                "fill_count_ge:3".to_string(),
                "glyph_count_ge:1".to_string(),
            ],
        },

        // ── 博客文章布局 ──
        TestCase {
            id: "render/blog-article".to_string(),
            description: "Blog article with typography".to_string(),
            category: "css".to_string(),
            html: r#"<html><body style="margin:0;font-family:Georgia,serif">
<article style="max-width:680px;margin:0 auto;padding:32px 16px">
  <h1 style="font-size:36px;line-height:1.2;margin:0 0 16px;color:#1a1a1a">Understanding CSS Grid Layout</h1>
  <div style="color:#666;margin:0 0 32px;font-size:14px">Published on June 5, 2026 by Author</div>
  <p style="font-size:18px;line-height:1.7;color:#333;margin:0 0 20px">CSS Grid Layout is a two-dimensional layout system that revolutionizes how we design web pages. It provides a powerful way to create complex layouts with simple, intuitive CSS.</p>
  <h2 style="font-size:24px;margin:0 0 12px">Key Concepts</h2>
  <p style="font-size:18px;line-height:1.7;color:#333;margin:0 0 20px">Grid containers define the overall structure, while grid items are positioned within that structure using various placement properties.</p>
  <blockquote style="border-left:4px solid #3b82f6;padding:12px 20px;margin:0 0 20px;background:#f8fafc;color:#555;font-style:italic">
    Grid makes it possible to align elements in rows and columns simultaneously.
  </blockquote>
  <p style="font-size:18px;line-height:1.7;color:#333;margin:0">The combination of grid-template-columns, grid-template-rows, and gap properties provides extensive control over layout.</p>
</article>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "glyph_count_ge:1".to_string(),
            ],
        },
    ]
}

/// 返回渲染管线高级合规性测试用例（合并布局 + 效果测试）。
pub fn render_rendance_tests() -> Vec<TestCase> {
    let mut tests = render_layout_tests();
    tests.extend(super::test_cases_render_detail::render_detail_tests());
    tests
}
