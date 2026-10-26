// part89 — html-semantics M2 切片 71（s71，2026-10-25）：F1b SVG 固有尺寸/密度
// 校正尾片（naturalWidth-naturalHeight-width-height SVG 面）。
//
// 根因：runner 位图解码面（createImageBitmap）不解 SVG，宿主 `__zw_get_image_size`
// 快照的 SVG 语义与本核不同（viewBox 原始宽高/缺省 100×100/部分属性+AR 补全），
// 且无逐轴密度元数据——SVG img 的 naturalWidth/Height 与 .width/.height 全族
// 183F。s71 三刀：① data:image/svg+xml payload 抽取 + CSS-images natural
// dimensions 组合核（`__zwSvgPayload5277`/`__zwSvgNaturalDims5277`——length 解析
// 非负数|px、0 有效、负值/百分比/不可解析缺席；viewBox 四数 w>0 且 h>0 → AR）；
// ② settle 集成——sweep/运行 fetch 臂 SVG 核前置（axes 元数据入 resourceState +
// available-images 缓存并随 avail 命中传播）；③ IDL 面——naturalWidth/Height
// 逐轴密度折算（axes：'both'/'width'/'height'/'none'，默认 300/150 具体对象轴
// 不校正）+ .width/.height 未渲染臂（display:none → 原始 intrinsic 不折密度，
// whatwg#11287）。
// https://html.spec.whatwg.org/multipage/images.html#density-corrected-intrinsic-width-and-height
// https://drafts.csswg.org/css-images/#natural-dimensions
// https://github.com/whatwg/html/issues/11287

/// 钉①：SVG 固有尺寸核真值点。(a) payload：百分号编码/base64 解码、非 SVG
/// data URL 与普通 URL → null；(b) 组合表：w+h → both、w+viewBox → both、
/// 仅 w 无 AR → width（h 回落 150）、h+viewBox → both、仅 h 无 AR → height
///（w 回落 300）、仅 viewBox → none（300,300/ar）、全缺 → none（300,150）；
/// (c) length 解析：负值/百分比/不可解析缺席、px 后缀合法、0 有效；
/// (d) viewBox 退化：含 0 宽或不足四数 → 无 AR；(e) 引号形态：单引号/无引号。
/// 断言翻转：SVG 核撤（全红）。
#[test]
fn svg_intrinsic_dims_r5277() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> =
        Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s71p".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var D = function (u) { return JSON.stringify(globalThis.__zwSvgNaturalDims5277(globalThis.__zwSvgPayload5277(u))); };\
             var S = function (t) { return JSON.stringify(globalThis.__zwSvgNaturalDims5277(t)); };\
             globalThis.__r = [\
               // (a) payload
               D(\"data:image/svg+xml,%3Csvg width='100' height='50'%3E%3C/svg%3E\"),\
               D('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTAwIiBoZWlnaHQ9IjUwIj48L3N2Zz4='),\
               String(globalThis.__zwSvgPayload5277('data:image/png;base64,AAAA')),\
               String(globalThis.__zwSvgPayload5277('https://zero.test/a.svg')),\
               // (b) 组合表
               S(\"<svg xmlns='x' width='100' height='50'/>\"),\
               S(\"<svg width='100' viewBox='0 0 200 100'/>\"),\
               S(\"<svg width='100'/>\"),\
               S(\"<svg height='100' viewBox='0 0 200 100'/>\"),\
               S(\"<svg height='100'/>\"),\
               S(\"<svg viewBox='0 0 200 100'/>\"),\
               S('<svg/>'),\
               // (c) length 解析
               S(\"<svg width='-5' height='50'/>\"),\
               S(\"<svg width='50%' height='50'/>\"),\
               S(\"<svg width='abc' height='50'/>\"),\
               S(\"<svg width='100px' height='50'/>\"),\
               S(\"<svg width='0' height='50'/>\"),\
               // (d) viewBox 退化
               S(\"<svg viewBox='0 0 0 100'/>\"),\
               S(\"<svg viewBox='0 0 200'/>\"),\
               // (e) 引号形态（无引号值；HTML tokenizer 不在 / 处截断——height=40/> 的值
               // 含斜杠非法缺席，故钉无斜杠形态）
               S('<svg width=80 height=40>')\
             ];",
        )
        .unwrap();
    let get = |sandbox: &mut V8Sandbox, i: usize| -> String {
        sandbox
            .execute(&format!("globalThis.__r[{}]", i))
            .unwrap()
            .value
    };
    // (a)
    assert_eq!(
        get(&mut sandbox, 0),
        r#"{"width":100,"height":50,"axes":"both"}"#,
        "(a1) 百分号编码 payload 解码"
    );
    assert_eq!(
        get(&mut sandbox, 1),
        r#"{"width":100,"height":50,"axes":"both"}"#,
        "(a2) base64 payload 解码"
    );
    assert_eq!(get(&mut sandbox, 2), "null", "(a3) 非 SVG data URL → null");
    assert_eq!(get(&mut sandbox, 3), "null", "(a4) 普通 URL → null");
    // (b)
    assert_eq!(
        get(&mut sandbox, 4),
        r#"{"width":100,"height":50,"axes":"both"}"#,
        "(b1) w+h → both"
    );
    assert_eq!(
        get(&mut sandbox, 5),
        r#"{"width":100,"height":50,"axes":"both"}"#,
        "(b2) w+viewBox → w,w/ar both"
    );
    assert_eq!(
        get(&mut sandbox, 6),
        r#"{"width":100,"height":150,"axes":"width"}"#,
        "(b3) 仅 w 无 AR → (w,150,width)"
    );
    assert_eq!(
        get(&mut sandbox, 7),
        r#"{"width":200,"height":100,"axes":"both"}"#,
        "(b4) h+viewBox → h*ar,h both"
    );
    assert_eq!(
        get(&mut sandbox, 8),
        r#"{"width":300,"height":100,"axes":"height"}"#,
        "(b5) 仅 h 无 AR → (300,h,height)"
    );
    assert_eq!(
        get(&mut sandbox, 9),
        r#"{"width":300,"height":150,"axes":"none"}"#,
        "(b6) 仅 viewBox → (300,300/ar,none)"
    );
    assert_eq!(
        get(&mut sandbox, 10),
        r#"{"width":300,"height":150,"axes":"none"}"#,
        "(b7) 全缺 → 默认对象尺寸 (300,150,none)"
    );
    // (c)
    assert_eq!(
        get(&mut sandbox, 11),
        r#"{"width":300,"height":50,"axes":"height"}"#,
        "(c1) 负 width 缺席 → 仅 h 臂"
    );
    assert_eq!(
        get(&mut sandbox, 12),
        r#"{"width":300,"height":50,"axes":"height"}"#,
        "(c2) 百分比 width 缺席"
    );
    assert_eq!(
        get(&mut sandbox, 13),
        r#"{"width":300,"height":50,"axes":"height"}"#,
        "(c3) 不可解析 width 缺席"
    );
    assert_eq!(
        get(&mut sandbox, 14),
        r#"{"width":100,"height":50,"axes":"both"}"#,
        "(c4) px 后缀合法"
    );
    assert_eq!(
        get(&mut sandbox, 15),
        r#"{"width":0,"height":50,"axes":"both"}"#,
        "(c5) 0 有效固有（零尺寸 SVG = loaded naturalWidth 0 面）"
    );
    // (d)
    assert_eq!(
        get(&mut sandbox, 16),
        r#"{"width":300,"height":150,"axes":"none"}"#,
        "(d1) viewBox 零宽退化 → 无 AR"
    );
    assert_eq!(
        get(&mut sandbox, 17),
        r#"{"width":300,"height":150,"axes":"none"}"#,
        "(d2) viewBox 不足四数 → 无 AR"
    );
    // (e)
    assert_eq!(
        get(&mut sandbox, 18),
        r#"{"width":80,"height":40,"axes":"both"}"#,
        "(e) 无引号属性值"
    );
}

/// 钉②：settle 集成 + IDL 面。(a) sweep SVG 核前置——静态 data:SVG img settle
/// 尺寸来自本核而非宿主快照（宿主桩返回 999,999 证伪位），axes 入 avail 缓存；
/// (b) srcset 2x 单候选（DPR 1 取最小密度）→ 密度校正 naturalWidth/Height +
/// 渲染 .width/.height 折算；(c) 逐轴面——仅 w 无 AR（axes width）：naturalWidth
/// 折算、naturalHeight 150 不折；仅 viewBox（axes none）：双轴均不折；仅 h 无 AR
///（axes height）：naturalWidth 300 不折；(d) 未渲染臂——display:none 后
/// .width/.height 回原始 intrinsic（不折密度），naturalWidth 不变（whatwg#11287）；
/// (e) raster 同语义钉——avail 2x 命中 .width 折算、display:none 回原始；
/// (f) 运行期 avail 命中传播（sweep 写入的 axes 项）→ naturalWidth 按 axes 折算；
/// (g) avail axes 传播——预置带 axes 的 avail 项命中后密度逐轴折算；
/// (h) sweep 覆写面——无 axes 的 host 形态（999 证伪位）被 SVG 核原位覆写
/// （viewBox AR 组合表 + axes），load 不重派。
/// 断言翻转：SVG 前置撤（(a) 翻 999）；axes 折算撤（(b)(c) 翻原始）；未渲染
/// 臂撤（(d)(e) 翻折算值）；avail 传播撤（(f)(g) 翻默认轴折算/miss）；覆写撤
///（(h1) 翻回 999 并双派 load）。
#[test]
fn svg_img_settle_density_r5277() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // 静态面：SVG src/img、单候选 2x srcset（w+h）、仅 w、仅 viewBox、仅 h、
    // raster 2x。data URL 百分号编码且 srcset 面空格全编码（srcset 解析按空白
    // 切分——语料 cloneTemplateWithDensity 的 replaceAll(" ", "%20") 同款）。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <img id='a' src=\"data:image/svg+xml,%3Csvg width='100' height='50'%3E%3C/svg%3E\">\
         <img id='b' srcset=\"data:image/svg+xml,%3Csvg%20width='200'%20height='100'%3E%3C/svg%3E 2x\">\
         <img id='w' srcset=\"data:image/svg+xml,%3Csvg%20width='100'%3E%3C/svg%3E 2x\">\
         <img id='v' srcset=\"data:image/svg+xml,%3Csvg%20viewBox='0%200%20200%20100'%3E%3C/svg%3E 2x\">\
         <img id='h' srcset=\"data:image/svg+xml,%3Csvg%20height='100'%3E%3C/svg%3E 2x\">\
         <img id='r' srcset='r.png 2x'>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s71s".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 宿主快照证伪位：任何查询都返回 999,999——SVG settle 尺寸若来自宿主即翻红。
    // raster r.png 除外（(e) 面）。
    sandbox
        .execute(
            "globalThis.devicePixelRatio = 1;\
             globalThis.__zw_get_image_size = function (src) {\
               if (src === 'https://zero.test/r.png') return '600,240';\
               return '999,999';\
             };\
             globalThis._zwAvailableImages = { 'https://zero.test/r.png': { width: 600, height: 240 } };\
             globalThis.dispatchEvent(new Event('load'));\
             var _a = document.getElementById('a'), _b = document.getElementById('b'),\
                 _w = document.getElementById('w'), _v = document.getElementById('v'),\
                 _h = document.getElementById('h'), _r = document.getElementById('r');\
             globalThis.__r1 = String(_a.naturalWidth) + '/' + String(_a.naturalHeight) + '/' + String(_a.complete) + '/' + String(_a.width) + '/' + String(_a.height);\
             globalThis.__r2 = String(_b.naturalWidth) + '/' + String(_b.naturalHeight) + '/' + String(_b.width) + '/' + String(_b.height);\
             globalThis.__r3 = String(_w.naturalWidth) + '/' + String(_w.naturalHeight);\
             globalThis.__r4 = String(_v.naturalWidth) + '/' + String(_v.naturalHeight);\
             globalThis.__r5 = String(_h.naturalWidth) + '/' + String(_h.naturalHeight);\
             globalThis.__r6 = String(_r.naturalWidth) + '/' + String(_r.naturalHeight) + '/' + String(_r.width) + '/' + String(_r.height);\
             globalThis.__r7 = JSON.stringify(globalThis._zwAvailableImages[\"data:image/svg+xml,%3Csvg width='100' height='50'%3E%3C/svg%3E\"]);",
        )
        .unwrap();
    let v = |sandbox: &mut V8Sandbox, name: &str| -> String {
        sandbox
            .execute(&format!("globalThis.{}", name))
            .unwrap()
            .value
    };
    assert_eq!(
        v(&mut sandbox, "__r1"),
        "100/50/true/100/50",
        "(a) sweep SVG 核前置（宿主证伪位 999 未生效）+ 密度 1 渲染 .width = intrinsic"
    );
    assert_eq!(
        v(&mut sandbox, "__r2"),
        "100/50/100/50",
        "(b) 2x 单候选 → naturalWidth/Height 与渲染 .width/.height 均密度校正 100/50"
    );
    assert_eq!(
        v(&mut sandbox, "__r3"),
        "50/150",
        "(c1) 仅 w 无 AR：naturalWidth 折算 50、naturalHeight 默认 150 不折"
    );
    assert_eq!(
        v(&mut sandbox, "__r4"),
        "300/150",
        "(c2) 仅 viewBox：默认轴不折（300/150）"
    );
    assert_eq!(
        v(&mut sandbox, "__r5"),
        "300/50",
        "(c3) 仅 h 无 AR：naturalWidth 300 不折、naturalHeight 折算 50"
    );
    assert_eq!(
        v(&mut sandbox, "__r6"),
        "300/120/300/120",
        "(e) raster 2x：naturalWidth/Height 与渲染 .width/.height 折算 300/120"
    );
    assert_eq!(
        v(&mut sandbox, "__r7"),
        r#"{"width":100,"height":50,"axes":"both"}"#,
        "(a2) avail 缓存携 axes"
    );
    // (d) 未渲染臂：display:none 后 .width/.height 回原始 intrinsic（SVG 与
    // raster 同语义），naturalWidth/Height 不变。
    sandbox
        .execute(
            "var _a = document.getElementById('a'), _b = document.getElementById('b'),\
                 _w = document.getElementById('w'), _v = document.getElementById('v'),\
                 _h = document.getElementById('h'), _r = document.getElementById('r');\
             _a.style.display = 'none'; _b.style.display = 'none'; _w.style.display = 'none';\
             _v.style.display = 'none'; _h.style.display = 'none'; _r.style.display = 'none';\
             globalThis.__d1 = String(_a.width) + '/' + String(_a.height) + '/' + String(_a.naturalWidth);\
             globalThis.__d2 = String(_b.width) + '/' + String(_b.height) + '/' + String(_b.naturalWidth) + '/' + String(_b.naturalHeight);\
             globalThis.__d3 = String(_w.width) + '/' + String(_w.height);\
             globalThis.__d4 = String(_v.width) + '/' + String(_v.height);\
             globalThis.__d5 = String(_h.width) + '/' + String(_h.height);\
             globalThis.__d6 = String(_r.width) + '/' + String(_r.height) + '/' + String(_r.naturalWidth);",
        )
        .unwrap();
    assert_eq!(
        v(&mut sandbox, "__d1"),
        "100/50/100",
        "(d1) SVG display:none → .width/.height 原始、naturalWidth 不变"
    );
    assert_eq!(
        v(&mut sandbox, "__d2"),
        "200/100/100/50",
        "(d2) SVG 2x display:none → 原始 200/100（未折密度）、naturalWidth/Height 仍校正"
    );
    assert_eq!(
        v(&mut sandbox, "__d3"),
        "100/150",
        "(d3) 仅 w：width 原始 100"
    );
    assert_eq!(v(&mut sandbox, "__d4"), "300/150", "(d4) 仅 viewBox：无折算面（不变）");
    assert_eq!(v(&mut sandbox, "__d5"), "300/100", "(d5) 仅 h：height 原始 100");
    assert_eq!(
        v(&mut sandbox, "__d6"),
        "600/240/300",
        "(e2) raster 2x display:none → 原始 600/240、naturalWidth 仍校正"
    );
    // (f) 运行期 avail 命中传播（真实 sweep 写入源）：sweep settle 已把 a 图的
    // data:SVG 连 axes 写入 avail（(a2)）；运行期新建 img 引同 URL → avail 命中
    // → resourceState 携 axes → naturalWidth 按 axes 折算。注：运行期 fetch 臂
    // 无 SVG 前置（runner 位图解码面 SVG 语义收口属 host 解码面切片）——运行期
    // SVG settle 面即 avail 传播面。
    sandbox
        .execute(
            "var img = document.createElement('img');\
             document.body.appendChild(img);\
             img.src = \"data:image/svg+xml,%3Csvg width='100' height='50'%3E%3C/svg%3E\";\
             globalThis.__f1 = String(img.complete) + '/' + String(img.naturalWidth) + '/' + String(img.naturalHeight);",
        )
        .unwrap();
    assert_eq!(
        v(&mut sandbox, "__f1"),
        "true/100/50",
        "(f) 运行期 avail 命中（sweep 写入 axes）→ naturalWidth/Height 密度 1 原始"
    );
    sandbox.execute("0;").unwrap();
    // (g) avail axes 传播：预置带 axes 的 avail 项（w+h 200/100 both），img 2x
    // 命中 → naturalWidth 100/50 密度折算。
    sandbox
        .execute(
            "globalThis._zwAvailableImages[\"data:image/svg+xml,%3Csvg%20width='200'%20height='100'%3E%3C/svg%3E\"] = { width: 200, height: 100, axes: 'both' };\
             var img2 = document.createElement('img');\
             img2.srcset = \"data:image/svg+xml,%3Csvg%20width='200'%20height='100'%3E%3C/svg%3E 2x\";\
             globalThis.__g1 = String(img2.naturalWidth) + '/' + String(img2.naturalHeight);",
        )
        .unwrap();
    assert_eq!(
        v(&mut sandbox, "__g1"),
        "100/50",
        "(g) avail 命中 axes 传播 → 密度逐轴折算"
    );
    // (h) sweep 覆写面：预置无 axes 的 host 形 avail 项（999×999 证伪位，模拟
    // 既有 host 快照语义）→ 运行期 avail 命中写 host 态（无 axes）→ sweep SVG
    // 核原位覆写为组合表值并补 axes；load 不重派（总计仅 1 次）。
    sandbox
        .execute(
            "globalThis._zwAvailableImages[\"data:image/svg+xml,%3Csvg%20viewBox='0%200%20600%20200'%3E%3C/svg%3E\"] = { width: 999, height: 999 };\
             var ovi = document.createElement('img');\
             document.body.appendChild(ovi);\
             globalThis.__ovi = ovi;\
             globalThis.__ovLoads = 0;\
             ovi.addEventListener('load', function () { globalThis.__ovLoads++; });\
             ovi.src = \"data:image/svg+xml,%3Csvg%20viewBox='0%200%20600%20200'%3E%3C/svg%3E\";\
             globalThis.__h0 = String(ovi.complete) + '/' + String(ovi.naturalWidth) + '/' + String(globalThis.__ovLoads);",
        )
        .unwrap();
    assert_eq!(
        v(&mut sandbox, "__h0"),
        "true/999/0",
        "(h0) avail 命中 host 形态（无 axes）→ host 快照值 999 原始生效（load 延迟派发）"
    );
    sandbox
        .execute(
            "globalThis.dispatchEvent(new Event('load'));\
             globalThis.__h1 = String(globalThis.__ovi.naturalWidth) + '/' + String(globalThis.__ovi.naturalHeight) + '/' + String(globalThis.__ovLoads) + '/' + JSON.stringify(globalThis._zwAvailableImages[\"data:image/svg+xml,%3Csvg%20viewBox='0%200%20600%20200'%3E%3C/svg%3E\"]);",
        )
        .unwrap();
    assert_eq!(
        v(&mut sandbox, "__h1"),
        "300/100/1/{\"width\":300,\"height\":100,\"axes\":\"none\"}",
        "(h1) sweep 覆写 host 态 → viewBox AR 组合表 300/100 + axes 入账，load 不重派"
    );
}
