//! 多进程渲染边界一致性：单进程布局 → renderer 编码 → IPC 图元快照 → 共享层解码。
//!
//! 背景（run.md 问题池「多进程渲染分叉簇」）：多进程 zero-renderer 子进程与单
//! 进程 webview 曾观测到同 HTML 不同布局产出（float clear 摆位 + clearfix 撑
//! 起失效）。多进程相对单进程的特异面 = renderer 侧编码（`paint_snapshot_from_
//! primitives`）→ IPC wire → browser 侧解码（`apply_paint_snapshot`，其图元映
//! 射即公共层 `zero_paint_convert::to_render_primitives`，见 apps/browser/
//! src/paint_ipc.rs 模块文档）。本测试用 clearfix_webview_layout 同款复现形状
//! 钉住该边界的几何/顺序保真：布局产出 P1 编码再解码为 P2，断言 P2 == P1——
//! 若 IPC 边界失真（rect/颜色/draw_order 漂移）此处即红，把分叉钉在边界而非
//! 布局核心。布局核心本身由 clearfix_webview_layout 与 reftest
//! `css-float/clearfix-after-nested-float-clear` 锚定。

use zero_paint_convert::to_render_primitives;
use zero_render_foundation::primitive::RenderPrimitives;
use zero_webview::{WebView, WebViewConfig};

const HTML: &str = r#"<html><head><style>body{margin:0}.w{position:relative;margin:93px auto 0;padding-top:21px;width:760px}.w::after{content:"";display:block;clear:both}.t{height:24px;margin-bottom:18px;background:yellow}.ul{list-style:none;padding:0;margin:0}.li{float:left;width:369px;height:36px;background:silver}.li.odd{clear:both;margin-right:20px}.after{height:40px;background:lime}</style></head><body><div class="w"><div class="t"></div><ul class="ul"><li class="li odd"></li><li class="li"></li><li class="li odd"></li><li class="li"></li></ul></div><div class="after"></div></body></html>"#;

/// 提取银色 369×36 行块 (x, y)，按 y 再 x 排序（与锚测试同口径）。
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

/// fills 的逐图元几何+颜色指纹（FillPrimitive 无 PartialEq，用指纹结构体对比）。
#[derive(Debug, PartialEq)]
struct FillFingerprint {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    rgba: [u8; 4],
}

fn fill_fingerprints(p: &RenderPrimitives) -> Vec<FillFingerprint> {
    p.fills
        .iter()
        .map(|f| FillFingerprint {
            x: f.rect.origin.x,
            y: f.rect.origin.y,
            w: f.rect.size.width,
            h: f.rect.size.height,
            rgba: [f.color.r, f.color.g, f.color.b, f.color.a],
        })
        .collect()
}

#[test]
fn ipc_paint_snapshot_roundtrip_preserves_clearfix_layout() {
    let mut wv = WebView::new(WebViewConfig {
        width: 800,
        height: 600,
        ..Default::default()
    });
    // 浏览器导航形态 = complete_fetched_page → load_html(html, Some(external_css))；
    // 前置一次空白加载覆盖 pipeline 复用路径（与锚测试同款）。
    let _ = wv.load_html("<html><body></body></html>", None);
    let result = wv.load_html(HTML, Some(""));
    let p1: RenderPrimitives = result.primitives().clone();

    // renderer 生产编码入口（apps/renderer paint_export）→ 共享层解码
    //（zero_paint_convert，即 browser 侧 apply_paint_snapshot 的图元映射）。
    let snapshot = zero_renderer::paint_snapshot_from_primitives(
        800,
        600,
        1.0,
        268.0, // 文档高：lime 底 228 + 40
        &p1,
        &[],
        Vec::new(),
        None,
        1, // navigation_epoch
        1, // document_generation
    );
    let p2 = to_render_primitives(snapshot);

    // 1) 几何锚：4 银块 369×36，row1 y=156、row2（clear:both 换行）y=192。
    let rows = silver_rows(&p2.fills);
    assert_eq!(rows.len(), 4, "解码后应有 4 块 369×36 银色行块，实际 {rows:?}");
    assert!(
        rows.iter().any(|r| (r.1 - 156.0).abs() < 2.0),
        "row1 应落 y≈156，实际 {rows:?}"
    );
    assert!(
        rows.iter().any(|r| (r.1 - 192.0).abs() < 2.0),
        "row2 应落 y≈192（clear:both 换行），实际 {rows:?}"
    );

    // 2) clearfix 撑起：lime 兄弟块 y≈228、h=40（依赖 wrapper 高 135）。
    assert!(
        p2.fills.iter().any(|f| {
            (f.color.r, f.color.g, f.color.b) == (0, 255, 0)
                && (f.rect.size.height - 40.0).abs() < 1.0
                && (f.rect.origin.y - 228.0).abs() < 2.0
        }),
        "lime 兄弟块应落 y≈228（wrapper::after clearance 撑起），实际 {:?}",
        p2.fills
    );

    // 3) 逐图元保真：fills 指纹与 draw_order 完全一致（顺序漂移即叠画错序）。
    assert_eq!(
        fill_fingerprints(&p1),
        fill_fingerprints(&p2),
        "IPC 快照往返后 fills 几何/颜色应逐图元一致"
    );
    assert_eq!(p1.draw_order, p2.draw_order, "IPC 快照往返后 draw_order 应完全一致");
}
