//! 渲染管线高级合规性测试。
//!
//! 测试完整的 CSS → Style → Layout → Paint → Composite 管线，
//! 验证多属性组合、复杂布局、层叠上下文等高级场景。

use super::TestCase;

/// 返回渲染管线高级合规性测试用例。
pub fn render_rendance_tests() -> Vec<TestCase> {
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

        // ═══════════════════════════════════════════════════════════════
        //  background-position / size / clip / origin 渲染
        // ═══════════════════════════════════════════════════════════════

        // ── background-position: center ──
        TestCase {
            id: "render/bg-position-center".to_string(),
            description: "background-position:center renders correctly".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:300px;height:150px;background-color:#e0e0e0;background-image:url('photo.jpg');background-position:center">Centered</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_image_primitives".to_string(),
            ],
        },

        // ── background-position: right bottom ──
        TestCase {
            id: "render/bg-position-right-bottom".to_string(),
            description: "background-position:right bottom offset".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:300px;height:150px;background-color:#eee;background-image:url('icon.png');background-position:right bottom;background-repeat:no-repeat">RB</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_image_primitives".to_string(),
            ],
        },

        // ── background-position 百分比 + 长度组合 ──
        TestCase {
            id: "render/bg-position-two-value".to_string(),
            description: "background-position with two values (percent and length)".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:400px;height:200px;background-color:#f0f0f0;background-image:url('bg.jpg');background-position:50% 20px">Two Values</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_image_primitives".to_string(),
            ],
        },

        // ── background-size: cover ──
        TestCase {
            id: "render/bg-size-cover".to_string(),
            description: "background-size:cover scales to cover container".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:300px;height:200px;background-color:#ccc;background-image:url('hero.jpg');background-size:cover">Cover</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_image_primitives".to_string(),
            ],
        },

        // ── background-size: contain ──
        TestCase {
            id: "render/bg-size-contain".to_string(),
            description: "background-size:contain fits within container".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:400px;height:200px;background-color:#ddd;background-image:url('logo.png');background-size:contain;background-repeat:no-repeat">Contain</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_image_primitives".to_string(),
            ],
        },

        // ── background-size: 50% 百分比 ──
        TestCase {
            id: "render/bg-size-percent".to_string(),
            description: "background-size:50% scales to half container width".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:400px;height:200px;background-color:#f5f5f5;background-image:url('bg.jpg');background-size:50%">Half Width</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_image_primitives".to_string(),
            ],
        },

        // ── background-clip: content-box ──
        TestCase {
            id: "render/bg-clip-content-box".to_string(),
            description: "background-clip:content-box clips to content area".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:200px;height:100px;padding:20px;border:5px solid #333;background-color:#ff6b6b;background-clip:content-box">Clipped</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── background-clip: padding-box ──
        TestCase {
            id: "render/bg-clip-padding-box".to_string(),
            description: "background-clip:padding-box clips to padding area".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:200px;height:100px;padding:15px;border:8px solid #555;background-color:#4ecdc4;background-clip:padding-box">Padded</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── background-origin: content-box + position ──
        TestCase {
            id: "render/bg-origin-content-box".to_string(),
            description: "background-origin:content-box positions from content area".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:300px;height:150px;padding:20px;border:10px solid #999;background-color:#f0f0f0;background-image:url('bg.jpg');background-origin:content-box;background-position:center">Origin</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_image_primitives".to_string(),
            ],
        },

        // ── 渐变 + background-size + position 组合 ──
        TestCase {
            id: "render/gradient-with-size-position".to_string(),
            description: "Gradient with background-size and position".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:400px;height:200px;background-color:#f8f8f8;background:linear-gradient(135deg,#667eea,#764ba2);background-size:50% 50%;background-position:center">Gradient Positioned</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "gradient_count_ge:1".to_string(),
            ],
        },

        // ── background 完整简写测试 ──
        TestCase {
            id: "render/bg-shorthand-comprehensive".to_string(),
            description: "Background shorthand with color image position/size".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:300px;height:180px;background:#e8f4f8 url('bg.png') no-repeat center/contain">Shorthand</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Column-rule 渲染
        // ═══════════════════════════════════════════════════════════════

        // ── column-rule: solid ──
        TestCase {
            id: "render/column-rule-solid".to_string(),
            description: "column-rule: solid 多列分隔线渲染".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<div style="column-count:3;column-gap:20px;column-rule:2px solid gray;width:600px">
<p>Column one content with some text.</p>
<p>Column two content with some text.</p>
<p>Column three content with some text.</p>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ── column-rule: dashed ──
        TestCase {
            id: "render/column-rule-dashed".to_string(),
            description: "column-rule: dashed 多列虚线分隔线渲染".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<div style="column-count:2;column-gap:30px;column-rule:3px dashed blue;width:400px">
<p>Left column text content.</p>
<p>Right column text content.</p>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  List-style-image 渲染
        // ═══════════════════════════════════════════════════════════════

        // ── list-style-image: url() ──
        TestCase {
            id: "render/list-style-image-url".to_string(),
            description: "list-style-image: url() 图片列表标记渲染".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<ul style="list-style-image:url('bullet.png')">
<li>First item</li>
<li>Second item</li>
<li>Third item</li>
</ul>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Empty-cells 渲染
        // ═══════════════════════════════════════════════════════════════

        // ── empty-cells: hide ──
        TestCase {
            id: "render/empty-cells-hide".to_string(),
            description: "empty-cells:hide 空单元格不显示边框和背景".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<table style="border-collapse:separate;empty-cells:hide">
<tr><td style="background:#ccc;border:1px solid black">Content</td><td style="background:#ccc;border:1px solid black"></td></tr>
<tr><td style="background:#ccc;border:1px solid black">Data</td><td style="background:#ccc;border:1px solid black">More</td></tr>
</table>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── empty-cells: show (default) ──
        TestCase {
            id: "render/empty-cells-show".to_string(),
            description: "empty-cells:show 空单元格显示边框和背景".to_string(),
            category: "css-layout".to_string(),
            html: r#"<html><body>
<table style="border-collapse:separate;empty-cells:show">
<tr><td style="background:#ccc;border:1px solid black">Content</td><td style="background:#ccc;border:1px solid black"></td></tr>
</table>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "fill_count_ge:2".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  渲染管线扩展（+5 测试）
        // ═══════════════════════════════════════════════════════════════

        // ── 多层 box-shadow 组合渲染 ──
        TestCase {
            id: "render/box-shadow-multi-layer".to_string(),
            description: "多层 box-shadow 与 background-color 组合渲染".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width:200px;height:100px;background:white;margin:40px;box-shadow:0 2px 4px rgba(0,0,0,0.1),0 8px 16px rgba(0,0,0,0.1),0 16px 32px rgba(0,0,0,0.05)">Multi shadow</div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string(), "has_fill_primitives".to_string()],
        },

        // ── border-image 简写渲染 ──
        TestCase {
            id: "render/border-image-shorthand".to_string(),
            description: "border-image 简写渲染验证".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width:200px;height:100px;border:20px solid;border-image:url('border.png') 30 round">Border image</div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },

        // ── text-overflow: ellipsis 溢出截断 ──
        TestCase {
            id: "render/text-overflow-ellipsis".to_string(),
            description: "text-overflow:ellipsis 溢出文本截断渲染".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width:150px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;border:1px solid #ccc;padding:4px">This text is too long and should be truncated with ellipsis</div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string(), "glyph_count_ge:1".to_string()],
        },

        // ── CSS filter blur 组合渲染 ──
        TestCase {
            id: "render/filter-blur-composite".to_string(),
            description: "CSS filter:blur 与 opacity 组合渲染".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width:200px;height:80px;background:coral;filter:blur(2px);opacity:0.8">Blurred content</div>
            <div style="width:200px;height:80px;background:steelblue;filter:grayscale(50%) brightness(1.2)">Filtered</div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string(), "has_fill_primitives".to_string()],
        },

        // ── 复杂渐变组合渲染 ──
        TestCase {
            id: "render/gradient-layered".to_string(),
            description: "多层渐变叠加渲染".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width:300px;height:200px;background:linear-gradient(135deg,rgba(255,0,0,0.3),rgba(0,0,255,0.3)),linear-gradient(to right,#e0e0e0,#f0f0f0)">Layered gradients</div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string(), "gradient_count_ge:1".to_string()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS 动画/过渡渲染
        // ═══════════════════════════════════════════════════════════════

        // ── @keyframes 动画定义 + 渲染 ──
        TestCase {
            id: "render/animation-keyframes".to_string(),
            description: "@keyframes 动画定义渲染不崩溃".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="anim">Animated</div></body></html>"#.to_string(),
            css: r#"
                @keyframes fadeIn {
                    from { opacity: 0.0; }
                    to { opacity: 1.0; }
                }
                .anim { animation: fadeIn 1s linear; background-color: blue; width: 100px; height: 80px; }
            "#.to_string(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string(), "has_fill_primitives".to_string()],
        },

        // ── 动画 timing function: ease ──
        TestCase {
            id: "render/animation-timing-ease".to_string(),
            description: "animation timing ease 渲染".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="ease-box">Ease</div></body></html>"#.to_string(),
            css: r#"
                @keyframes slide { from { opacity: 0.2; } to { opacity: 1.0; } }
                .ease-box { animation: slide 2s ease; background-color: green; width: 150px; height: 100px; }
            "#.to_string(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string(), "has_fill_primitives".to_string()],
        },

        // ── 动画 timing function: steps ──
        TestCase {
            id: "render/animation-timing-steps".to_string(),
            description: "animation timing steps 渲染".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="steps-box">Steps</div></body></html>"#.to_string(),
            css: r#"
                @keyframes fade { 0% { opacity: 1.0; } 100% { opacity: 0.0; } }
                .steps-box { animation: fade 1s steps(4); background-color: red; width: 100px; height: 100px; }
            "#.to_string(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string(), "has_fill_primitives".to_string()],
        },

        // ── 动画 fill-mode: forwards ──
        TestCase {
            id: "render/animation-fill-forwards".to_string(),
            description: "animation fill-mode forwards 渲染".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="fill-box">Fill</div></body></html>"#.to_string(),
            css: r#"
                @keyframes grow { from { opacity: 0.0; } to { opacity: 1.0; } }
                .fill-box { animation: grow 0.5s linear forwards; background-color: orange; width: 200px; height: 120px; }
            "#.to_string(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string(), "has_fill_primitives".to_string()],
        },

        // ── 动画 direction: alternate ──
        TestCase {
            id: "render/animation-direction-alternate".to_string(),
            description: "animation direction alternate 渲染".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="alt-box">Alt</div></body></html>"#.to_string(),
            css: r#"
                @keyframes pulse { 0% { opacity: 0.3; } 100% { opacity: 1.0; } }
                .alt-box { animation: pulse 1s linear infinite alternate; background-color: purple; width: 100px; height: 100px; }
            "#.to_string(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string(), "has_fill_primitives".to_string()],
        },

        // ── 多元素同时动画 ──
        TestCase {
            id: "render/animation-multiple-elements".to_string(),
            description: "多元素同时动画渲染".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="a1">One</div>
                <div class="a2">Two</div>
                <div class="a3">Three</div>
            </body></html>"#.to_string(),
            css: r#"
                @keyframes fade { from { opacity: 0.0; } to { opacity: 1.0; } }
                .a1 { animation: fade 1s linear; background-color: red; width: 80px; height: 60px; }
                .a2 { animation: fade 1.5s ease; background-color: blue; width: 80px; height: 60px; }
                .a3 { animation: fade 2s ease-in-out; background-color: green; width: 80px; height: 60px; }
            "#.to_string(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string(), "fill_count_ge:3".to_string()],
        },

        // ── CSS transition 属性定义渲染 ──
        TestCase {
            id: "render/transition-property".to_string(),
            description: "CSS transition 属性定义渲染不崩溃".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="trans">Transition</div></body></html>"#.to_string(),
            css: r#"
                .trans {
                    transition: opacity 0.5s ease, background-color 0.3s linear;
                    opacity: 1.0; background-color: steelblue;
                    width: 200px; height: 100px; color: white;
                }
            "#.to_string(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string(), "has_fill_primitives".to_string()],
        },

        // ── transition with delay ──
        TestCase {
            id: "render/transition-delay".to_string(),
            description: "CSS transition delay 渲染".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="delayed">Delayed</div></body></html>"#.to_string(),
            css: r#"
                .delayed {
                    transition: opacity 1s 0.5s ease-in-out;
                    opacity: 0.8; background-color: coral;
                    width: 150px; height: 80px;
                }
            "#.to_string(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string(), "has_fill_primitives".to_string()],
        },

        // ── transition 多属性 ──
        TestCase {
            id: "render/transition-multi-property".to_string(),
            description: "CSS transition 多属性过渡渲染".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="multi">Multi</div></body></html>"#.to_string(),
            css: r#"
                .multi {
                    transition-property: opacity, width, background-color;
                    transition-duration: 0.3s, 0.5s, 0.4s;
                    transition-timing-function: ease, linear, ease-in;
                    opacity: 0.7; width: 180px; background-color: teal; height: 100px;
                }
            "#.to_string(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string(), "has_fill_primitives".to_string()],
        },

        // ── 动画 + transition 组合 ──
        TestCase {
            id: "render/animation-transition-combo".to_string(),
            description: "动画与过渡组合渲染".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="combo">Combo</div></body></html>"#.to_string(),
            css: r#"
                @keyframes colorShift { 0% { opacity: 0.5; } 100% { opacity: 1.0; } }
                .combo {
                    animation: colorShift 1s linear;
                    transition: background-color 0.3s ease;
                    background-color: navy; width: 200px; height: 120px; color: white;
                }
            "#.to_string(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string(), "has_fill_primitives".to_string()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Transform-origin + 非 translate 变换渲染
        // ═══════════════════════════════════════════════════════════════

        // ── rotate + transform-origin 渲染 ──
        TestCase {
            id: "render/transform-origin-rotate".to_string(),
            description: "CSS rotate with transform-origin".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:100px;height:100px;background:#e74c3c;transform:rotate(45deg);transform-origin:0 0">Rotated</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ── scale 变换渲染 ──
        TestCase {
            id: "render/transform-scale".to_string(),
            description: "CSS scale transform rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:100px;height:50px;background:#3498db;transform:scale(2,0.5)">Scaled</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ── skew 变换渲染 ──
        TestCase {
            id: "render/transform-skew".to_string(),
            description: "CSS skew transform rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:100px;height:80px;background:#9b59b6;transform:skew(20deg,10deg)">Skewed</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ── matrix() 变换渲染 ──
        TestCase {
            id: "render/transform-matrix".to_string(),
            description: "CSS matrix() transform rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:100px;height:60px;background:#1abc9c;transform:matrix(0.866,0.5,-0.5,0.866,10,20)">Matrix</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ── 变换组合 (translate + rotate + scale) ──
        TestCase {
            id: "render/transform-combined".to_string(),
            description: "Combined transform functions".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:80px;height:80px;background:#e67e22;transform:translate(50px,20px) rotate(30deg) scale(1.5)">Combined</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Conic-gradient 渲染
        // ═══════════════════════════════════════════════════════════════

        // ── conic-gradient 基础渲染 ──
        TestCase {
            id: "render/conic-gradient-basic".to_string(),
            description: "Basic conic-gradient rendering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:200px;height:200px;background:conic-gradient(red,yellow,green,blue,red)">Color Wheel</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "gradient_count_ge:1".to_string(),
            ],
        },

        // ── conic-gradient with from angle ──
        TestCase {
            id: "render/conic-gradient-from-angle".to_string(),
            description: "Conic-gradient with from angle".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:150px;height:150px;background:conic-gradient(from 90deg,#ff0,#0ff,#f0f,#ff0)">Angle</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "gradient_count_ge:1".to_string(),
            ],
        },

        // ── conic-gradient with position ──
        TestCase {
            id: "render/conic-gradient-position".to_string(),
            description: "Conic-gradient with center position".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:200px;height:200px;background:conic-gradient(from 45deg at 25% 75%,#2ecc71,#e74c3c,#2ecc71)">Position</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "gradient_count_ge:1".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS Counters
        // ═══════════════════════════════════════════════════════════════

        // ── 有序列表 + counter-increment ──
        TestCase {
            id: "render/counter-ordered-list".to_string(),
            description: "Ordered list with counter-increment".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<ol style="counter-reset:item">
  <li style="counter-increment:item">First</li>
  <li style="counter-increment:item">Second</li>
  <li style="counter-increment:item">Third</li>
</ol>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ── 嵌套计数器 ──
        TestCase {
            id: "render/counter-nested".to_string(),
            description: "Nested counters with reset/increment".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="counter-reset:section 0">
  <h2 style="counter-increment:section">Section 1</h2>
  <div style="counter-reset:subsection 0">
    <p style="counter-increment:subsection">Sub 1.1</p>
    <p style="counter-increment:subsection">Sub 1.2</p>
  </div>
  <h2 style="counter-increment:section">Section 2</h2>
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
        //  background-repeat 渲染
        // ═══════════════════════════════════════════════════════════════

        // ── background-repeat: repeat 默认平铺 ──
        TestCase {
            id: "render/bg-repeat-default".to_string(),
            description: "background-repeat default tiling".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:200px;height:100px;background-image:url('tile.png');background-size:50px 50px;">Tiled</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "image_count_ge:4".to_string(),
            ],
        },

        // ── background-repeat: repeat-x 仅水平平铺 ──
        TestCase {
            id: "render/bg-repeat-x".to_string(),
            description: "background-repeat-x horizontal only".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:200px;height:100px;background-image:url('stripe.png');background-size:40px 100px;background-repeat:repeat-x;">H-Stripe</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "image_count_ge:3".to_string(),
            ],
        },

        // ── background-repeat: repeat-y 仅垂直平铺 ──
        TestCase {
            id: "render/bg-repeat-y".to_string(),
            description: "background-repeat-y vertical only".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:100px;height:200px;background-image:url('stripe.png');background-size:100px 40px;background-repeat:repeat-y;">V-Stripe</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "image_count_ge:3".to_string(),
            ],
        },

        // ── background-repeat: no-repeat 不平铺 ──
        TestCase {
            id: "render/bg-no-repeat".to_string(),
            description: "background-repeat no-repeat single tile".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:200px;height:100px;background-image:url('photo.png');background-size:50px 50px;background-repeat:no-repeat;">Single</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "image_count_ge:1".to_string(),
            ],
        },

        // ── background-repeat: round 缩放平铺 ──
        TestCase {
            id: "render/bg-repeat-round".to_string(),
            description: "background-repeat round scaled tiles".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:200px;height:100px;background-image:url('tile.png');background-size:60px 60px;background-repeat:round;">Rounded</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "image_count_ge:3".to_string(),
            ],
        },

        // ── background-repeat: space 均匀分布 ──
        TestCase {
            id: "render/bg-repeat-space".to_string(),
            description: "background-repeat space evenly distributed".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:200px;height:100px;background-image:url('dot.png');background-size:30px 30px;background-repeat:space;">Spaced</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "image_count_ge:2".to_string(),
            ],
        },

        // ── background-repeat + position + size 组合 ──
        TestCase {
            id: "render/bg-repeat-position-size".to_string(),
            description: "background-repeat with position and size".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="width:300px;height:200px;background-image:url('icon.png');background-size:40px 40px;background-position:10px 10px;background-repeat:repeat;">Pattern</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "image_count_ge:10".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS 表格渲染
        // ═══════════════════════════════════════════════════════════════

        // ── 基础 HTML 表格渲染 ──
        TestCase {
            id: "render/html-table-basic".to_string(),
            description: "Basic HTML table rendering".to_string(),
            category: "html-layout".to_string(),
            html: r#"<html><body>
<table style="border-collapse:collapse;width:100%;">
  <tr><th style="border:1px solid #333;background:#eee;padding:4px;">Name</th><th style="border:1px solid #333;background:#eee;padding:4px;">Value</th></tr>
  <tr><td style="border:1px solid #333;padding:4px;">Alpha</td><td style="border:1px solid #333;padding:4px;">100</td></tr>
  <tr><td style="border:1px solid #333;padding:4px;">Beta</td><td style="border:1px solid #333;padding:4px;">200</td></tr>
</table>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── 带标题的表格 ──
        TestCase {
            id: "render/html-table-caption".to_string(),
            description: "HTML table with caption".to_string(),
            category: "html-layout".to_string(),
            html: r#"<html><body>
<table style="border-collapse:collapse;">
  <caption style="text-align:center;font-weight:bold;padding:4px;">Data Table</caption>
  <tr><td style="border:1px solid;padding:4px;">A</td><td style="border:1px solid;padding:4px;">B</td></tr>
</table>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ── 嵌套表格 ──
        TestCase {
            id: "render/html-table-nested".to_string(),
            description: "Nested HTML tables".to_string(),
            category: "html-layout".to_string(),
            html: r#"<html><body>
<table style="border:1px solid #000;"><tr><td style="padding:8px;">
  <table style="border:1px solid #999;"><tr><td style="padding:4px;">Inner</td></tr></table>
</td></tr></table>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS 多列布局渲染
        // ═══════════════════════════════════════════════════════════════

        // ── column-count 多列文本 ──
        TestCase {
            id: "render/multi-column-text".to_string(),
            description: "Multi-column text layout with column-count".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="column-count:3;column-gap:20px;column-rule:1px solid #ccc;">
  <p>Column one text content for testing multi-column layout rendering.</p>
  <p>Column two continues with more content to fill the space.</p>
  <p>Column three wraps up the text across three columns.</p>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ── column-width 固定列宽 ──
        TestCase {
            id: "render/multi-column-width".to_string(),
            description: "Multi-column layout with column-width".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
<div style="column-width:150px;column-gap:16px;column-rule:2px dashed #999;width:500px;">
  <p>Fixed width columns with dashed rules between them for visual separation.</p>
  <p>More content to demonstrate the column-width property rendering.</p>
</div>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },
    ]
}
