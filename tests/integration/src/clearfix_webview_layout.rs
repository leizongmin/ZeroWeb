//! wrapper::after clearfix + 仅含浮动 UL 的 webview 级布局回归。
//!
//! baidu 热榜叠字根因形状（R1323 float 子 clear 误置 clearance_active /
//! R1392 嵌套浮动底边帧错位）端到端：`WebView::load_html`（浏览器同款入口）
//! → 图元 rect 断言 row2 clear 换行 + wrapper 撑起 + 后续兄弟不叠压。
//!
//! 该测试钉住**单进程 webview 层**的几何基线。多进程浏览器（zero-renderer
//! 子进程）曾观测到同 HTML 布局分叉（row2 不换行、clearfix 不撑起；CSS
//! 规则完整到达 js-dom），见 run.md 问题池「多进程渲染分叉簇」——本测试是
//! 那一簇的对照锚：若此处红，先修 webview 层；若绿而浏览器红，分叉在
//! renderer 宿主层。

use zero_webview::{WebView, WebViewConfig};

const HTML: &str = r#"<html><head><style>body{margin:0}.w{position:relative;margin:93px auto 0;padding-top:21px;width:760px}.w::after{content:"";display:block;clear:both}.t{height:24px;margin-bottom:18px;background:yellow}.ul{list-style:none;padding:0;margin:0}.li{float:left;width:369px;height:36px;background:silver}.li.odd{clear:both;margin-right:20px}.after{height:40px;background:lime}</style></head><body><div class="w"><div class="t"></div><ul class="ul"><li class="li odd"></li><li class="li"></li><li class="li odd"></li><li class="li"></li></ul></div><div class="after"></div></body></html>"#;

fn silver_rows(fills: &[zero_render_foundation::primitive::FillPrimitive]) -> Vec<(f32, f32)> {
    let mut rows = vec![];
    for f in fills.iter() {
        let (x, y) = (f.rect.origin.x, f.rect.origin.y);
        let (w, h) = (f.rect.size.width, f.rect.size.height);
        if (f.color.r, f.color.g, f.color.b) == (192, 192, 192) && (w - 369.0).abs() < 1.0 && (h - 36.0).abs() < 1.0 {
            rows.push((x, y));
        }
    }
    rows.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.total_cmp(&b.0)));
    rows
}

#[test]
fn webview_clearfix_row2_clearance_and_wrapper_height() {
    let mut wv = WebView::new(WebViewConfig {
        width: 800,
        height: 600,
        ..Default::default()
    });
    // 浏览器导航形态 = complete_fetched_page → load_html(html, Some(external_css))，
    // 无外链样式页 external_css 为空串；前置一次空白加载覆盖 pipeline 复用路径。
    let _ = wv.load_html("<html><body></body></html>", None);
    let result = wv.load_html(HTML, Some(""));

    // 几何（viewport 800）：wrapper margin-top 93 + padding-top 21；
    // title 0..24 + mb 18 → float 行 42 起；row1 y=156、row2（clear:both 换行）y=192、
    // wrapper::after clearance 底 114 → wrapper 高 21+114=135 → lime y=228。
    let silvers = silver_rows(&result.primitives().fills);
    assert_eq!(silvers.len(), 4, "应有 4 块 369×36 银色行块");
    assert!(
        silvers.iter().any(|s| (s.1 - 192.0).abs() < 2.0),
        "row2 应落 y≈192（clear:both 换行），实际 {silvers:?}"
    );
    assert!(
        result.primitives().fills.iter().any(|f| {
            (f.color.r, f.color.g, f.color.b) == (0, 255, 0)
                && f.rect.size.height > 30.0
                && (f.rect.origin.y - 228.0).abs() < 2.0
        }),
        "后续兄弟 lime 块应落 y≈228（wrapper 高 135 撑起）"
    );
}
