//! R5424（html-semantics M2 s293）：运行核图片布局表同步——load 期
//! `extract_img_srcs` 清单（静态 HTML 源解析 + R5275 模板 walk）之外的运行时
//! img（script 注入 + srcset 换源）在位图 settle 臂经 `__zwImgSyncRuntimeTable`
//! 入待补队列，异步回调泵轮钩子逐 resolve 后排空补 `cached_*` 三表 + 信号门
//! `runtime_img_sync_applied` 容器并重绘。
//!
//! 修复前：运行时 srcset img 无布局表键——布局零盒 + s292 IDL 几何臂信号门
//! 关，IDL width 回落 naturalDim（300），corpus
//! naturalWidth-naturalHeight-width-height srcset 克隆 12F（勘根 2026-11-05）。
//! 修复后：同泵轮内落表 + `repaint_cached_viewport` + gBCR 快照刷新，几何臂
//! 读到真实布局（viewBox 600×200 → ratio 3 → 740 CB content 720×240）。
//!
//! 驱动形 = corpus 12F 面（cloneTemplateWithDensity 1x 变体）：`SVG + " 1x"`
//! srcset、740px containing block、img border 5px/margin 5px/display:block。
//! 静态哨兵 = corpus 静态 viewBox 面（data-width=720/height=240 口径）：load
//! 期清单路径不受补表影响。
//!
//! 已知边界（先在引擎缺口，非本片范围，挂账 s293 账本）：运行时 **plain-src**
//! img（createElement + setAttribute('.src')/IDL setter）不 settle——set 期 run
//! detached 延步（s291 ⓷ isConnected 检查）、插入钩子 s290 ⓵ spec 正确排除
//! plain-src（env 门只管 srcset/picture）、sweep 逐元素臂对运行时创建元素的
//! wrapper 身份链断裂（s285 物化缺口族）。corpus 无此形态（security 语料
//! new Image() 页由 runner 生命周期 sweep/timer 驱动 settle）。

use zero_webview::{WebView, WebViewConfig};

/// viewBox-only SVG（RatioOnly：R717 分类进 ratios 表，decode_data_uri 同构
/// r5421 RATIO_SRC；`%20` 编码 = corpus cloneTemplateWithDensity 的 srcset 写法）。
const RATIO_SRCSET_VAL: &str =
    "data:image/svg+xml,<svg%20xmlns='http://www.w3.org/2000/svg'%20viewBox='0%200%20600%20200'></svg> 1x";

/// 静态哨兵臂用同一 SVG（字面空格 data: URL，load 期清单可解析形态）。
const RATIO_SRC: &str = "data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 600 200'></svg>";

/// 页面：script 注入 srcset img（运行核 settle 驱动形）+ load 期 src img（静态哨兵），
/// 同居 740px 显式 CB（约束方程要求 CB 可解析——body auto CB 落 default object size
/// 300×100 fallback，s292 缺口1 臂设计内；corpus 静态面同形）。JS 侧 SVG 串用双引号
/// 包裹（data: URL 内含单引号属性——zz-s293-probe 同形）。
fn page_html() -> String {
    let svg_js = RATIO_SRCSET_VAL.strip_suffix(" 1x").unwrap_or(RATIO_SRCSET_VAL);
    "<!doctype html><html><head><script>globalThis.__errs=[];globalThis.onerror=function(){globalThis.__errs.push(String(arguments[0])+' @'+arguments[2])};</script><style>\
#cb { width: 740px; }\
img { border: 5px solid teal; margin: 5px; display: block; }\
</style></head><body><div id=\"cb\"><img id=\"loadtime\" src=\"SRC\" alt=\"\"></div>\
<script>\
globalThis.__mark='ran';\
var SVG = \"SVGVAL\";\
var img = document.createElement('img');\
img.id = 'rt';\
img.setAttribute('srcset', SVG + \" 1x\");\
document.getElementById('cb').appendChild(img);\
</script></body></html>"
        .replace("SVGVAL", svg_js)
        .replace("SRC", RATIO_SRC)
}

/// 最小 fetch handler：只服务测试自身的 data: SVG（payload 百分号解码），其余 404。
/// 形态对齐 runner `wpt_data_fetch_handler`（FetchResponse 直构）。
fn svg_fetch_handler() -> zero_engine::fetch_bridge::FetchHandler {
    std::sync::Arc::new(move |req: &zero_engine::fetch_bridge::FetchRequest| {
        if let Some(rest) = req.url.strip_prefix("data:image/svg+xml,") {
            let mut bytes = String::new();
            let mut chars = rest.chars();
            while let Some(c) = chars.next() {
                if c != '%' {
                    bytes.push(c);
                    continue;
                }
                let hex: String = chars.by_ref().take(2).collect();
                if let Ok(v) = u8::from_str_radix(&hex, 16) {
                    bytes.push(v as char);
                }
            }
            let headers: Vec<(String, String)> = vec![("content-type".into(), "image/svg+xml".into())];
            return Ok(zero_engine::fetch_bridge::FetchResponse {
                status: 200,
                status_text: "OK".to_string(),
                headers,
                body: bytes.clone(),
                body_bytes: Some(bytes.into_bytes()),
            });
        }
        Err("r5424: unexpected fetch".to_string())
    })
}

/// 泵至 img settle 完成（`img.complete` 翻真），再补数轮让同泵轮钩子的补表+重绘
/// 与后续 timer 载荷收敛；返回末轮泵次数。
fn pump_until_settled(wv: &mut WebView, max_rounds: usize) -> usize {
    let mut rounds = 0;
    for _ in 0..max_rounds {
        rounds += 1;
        let _ = wv.execute_script("1");
        let done = wv
            .execute_script_with_dom("var i=document.getElementById('rt'); (i && i.complete) ? '1' : '0'")
            .unwrap_or_default();
        if done.contains('1') {
            break;
        }
    }
    // settle resolve 的同泵轮钩子已补表+重绘；再泵一轮确认稳态（timer 载荷面）。
    let _ = wv.execute_script("1");
    // runner 生命周期 task2 同款：静态 settle 扫描（part04 s271 SVG 核臂）——
    // resourceState 覆写为 SVG default sizing 维（300×100 + axes），corpus
    // naturalWidth 断言面在该核值上；补表/布局 ratio 与此独立（native 分类）。
    let _ = wv.execute_script(
        "if (typeof globalThis.__zwFireStaticImgSettle === 'function') globalThis.__zwFireStaticImgSettle()",
    );
    rounds
}

/// ★load-bearing（corpus naturalWidth 12F 驱动形）：运行核 settle 补同步链全路径
/// ——script 注入 srcset img settle → 队列 → 泵轮钩子补 ratios 表（viewBox-only
/// SVG ratio 3）+ 信号门开 + 重绘 → IDL 几何臂读到真实渲染维 720×240（修复前
/// 零盒关门回落 naturalDim 300/100）。naturalWidth/Height 走 resourceState 密度
/// 臂不受补表影响（300/100 = SVG default sizing）。
#[test]
#[serial_test::serial]
fn r5424_runtime_img_sync_flips_geometry_arm() {
    let mut wv = WebView::new(WebViewConfig {
        width: 800,
        height: 600,
        fetch_handler: Some(svg_fetch_handler()),
        ..Default::default()
    });
    // runner 装载序同款：fetch_page_images（load 期清单抓取）→ load_html(external_css)。
    let page = page_html();
    let external_css = wv.fetch_page_images(&page, "https://zero.test/r5424.html");
    wv.load_html(&page, Some(&external_css));
    assert!(wv.run_page_scripts().is_ok(), "run_page_scripts 须成功");
    let rounds = pump_until_settled(&mut wv, 200);
    assert!(rounds < 200, "img settle 须在泵预算内完成");

    let width = wv
        .execute_script_with_dom("document.getElementById('rt').width")
        .unwrap_or_default();
    assert!(
        width.contains("720"),
        "运行核补表后 IDL width = 740 CB − margin 10 − border 10 = 720（修复前 naturalDim 回落 300），实测 {width:?}"
    );
    let height = wv
        .execute_script_with_dom("document.getElementById('rt').height")
        .unwrap_or_default();
    assert!(
        height.contains("240"),
        "IDL height = 720 / ratio 3 = 240（修复前 100），实测 {height:?}"
    );
    let natural = wv
        .execute_script_with_dom(
            "document.getElementById('rt').naturalWidth + ',' + document.getElementById('rt').naturalHeight",
        )
        .unwrap_or_default();
    assert!(
        natural.contains("300,100"),
        "naturalWidth/Height 走 resourceState 臂（SVG default sizing）不受补表影响，实测 {natural:?}"
    );
}

/// 静态哨兵（corpus 静态 viewBox 面 data-width=720/height=240 口径）：load 期
/// 清单 img（fetch_image_subresources 原路径）s293 零变更——快照三表已含键，
/// s292 几何臂照常读 gBCR 渲染维（740 CB 约束方程 content 720×240），
/// naturalWidth/Height 走 SVG default sizing 核（300×100）。
#[test]
#[serial_test::serial]
fn r5424_static_loadtime_sentinel() {
    let mut wv = WebView::new(WebViewConfig {
        width: 800,
        height: 600,
        fetch_handler: Some(svg_fetch_handler()),
        ..Default::default()
    });
    let page = page_html();
    let external_css = wv.fetch_page_images(&page, "https://zero.test/r5424.html");
    wv.load_html(&page, Some(&external_css));
    assert!(wv.run_page_scripts().is_ok(), "run_page_scripts 须成功");
    let _ = pump_until_settled(&mut wv, 200);
    let width = wv
        .execute_script_with_dom("document.getElementById('loadtime').width")
        .unwrap_or_default();
    assert!(
        width.contains("720"),
        "静态 img IDL width = 720（s292 约束方程既有行为，s293 零变更哨兵；corpus data-width=720 口径），实测 {width:?}"
    );
    let height = wv
        .execute_script_with_dom("document.getElementById('loadtime').height")
        .unwrap_or_default();
    assert!(
        height.contains("240"),
        "静态 img IDL height = 240（s292 既有行为哨兵），实测 {height:?}"
    );
    let natural = wv
        .execute_script_with_dom(
            "document.getElementById('loadtime').naturalWidth + ',' + document.getElementById('loadtime').naturalHeight",
        )
        .unwrap_or_default();
    assert!(
        natural.contains("300,100"),
        "静态 img naturalWidth/Height = SVG default sizing 核（300×100），实测 {natural:?}"
    );
}
