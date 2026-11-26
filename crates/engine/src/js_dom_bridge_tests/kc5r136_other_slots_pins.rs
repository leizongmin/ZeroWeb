// wptfyi-css k136 刀 常驻钉测（2026-11-26）：异种伪元素槽表 gCS 读面＋
// first-line/first-letter part 通道读形＋highlight 实参归一。
//   H1 handle 通道异种裸键投影：doc 表 `::backdrop` 规则 →
//      gCS(el,'::backdrop').backgroundColor 槽值；跨键不串（::details-content
//      读回落元素值——exact-key 检索面）；
//   H2 highlight 五别名同键读绿：`::highlight(name)` 全形（大小写/空白 trim/
//      escape 解码/无闭合括号容忍——CSSOM API 语法，getComputedStyle-pseudo-
//      with-argument parsable 名单）归一 host 单点（shim 名单门透传）；
//   H3 highlight nonParsable 形回落：`::highlight(1)`/`::highlight(name)a`
//      非 ident 实参/尾垃圾 → None → 元素自身值（页断言 length==0 经 shim
//      `.length` 硬编码空洞等价，本钉核 color 读不投影）；
//   H4 first-line/first-letter part 通道读形：`::part(p)::first-line`/
//      `::part(p)::first-letter`（含 legacy 单冒号读形 `:first-letter`）→
//      first_line_pseudo/first_letter_pseudo 槽投影（pseudo-elements-after-part
//      判据页读形）；
//   H5 零规则零漂移：无异种规则页 `::backdrop` 读＝元素值（槽 None 回落）。
// https://drafts.csswg.org/cssom-1/#dom-window-getcomputedstyle

/// 沙箱装具（k134/k135 同款）：base html 携静态规则，JS 动态建元素（handle
/// 通道——k134 K1 已证该通道伪元素槽投影可达）。
fn k136_sandbox_with_html(html: &str) -> zero_script_sandbox::V8Sandbox {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::V8Sandbox;
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(html.to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    sandbox
}

const K136_BACKDROP_HTML: &str = "<html><head><style>::backdrop { background: rgb(1, 2, 3); }</style>\
</head><body></body></html>";

const K136_HIGHLIGHT_HTML: &str = "<html><head><style>::highlight(name) { color: rgb(0, 128, 0); }</style>\
</head><body></body></html>";

const K136_FIRSTL_HTML: &str = "<html><head><style>::part(p)::first-line { color: rgb(1, 2, 3); } \
::part(p)::first-letter { background: rgb(4, 5, 6); }</style></head><body></body></html>";

/// H1：handle 通道异种裸键投影＋跨键不串。
#[test]
fn test_kc5r136_backdrop_read_exact_key() {
    use zero_script_sandbox::Sandbox;
    let mut sb = k136_sandbox_with_html(K136_BACKDROP_HTML);
    let r = sb
        .execute(
            r#"
        var d = document.createElement('div');
        document.body.appendChild(d);
        [getComputedStyle(d, '::backdrop').backgroundColor,
         getComputedStyle(d, '::details-content').backgroundColor].join('|')
        "#,
        )
        .unwrap()
        .value;
    assert_eq!(
        r, "rgb(1, 2, 3)|rgba(0, 0, 0, 0)",
        "H1 ::backdrop reads slot, ::details-content exact-key miss falls back, got {r:?}"
    );
}

/// H2：highlight 五别名同键读绿——全形/大小写/空白 trim/escape 解码/无闭合括号。
#[test]
fn test_kc5r136_highlight_aliases_read_green() {
    use zero_script_sandbox::Sandbox;
    let mut sb = k136_sandbox_with_html(K136_HIGHLIGHT_HTML);
    let r = sb
        .execute(
            r#"
        var d = document.createElement('div');
        document.body.appendChild(d);
        ['::highlight(name)', '::highlight(NAME)', '::highlight( name )',
         '::highlight( n\\61me )', '::highlight(name\t']
            .map(function (f) { return getComputedStyle(d, f).color; }).join('|')
        "#,
        )
        .unwrap()
        .value;
    assert_eq!(
        r,
        "rgb(0, 128, 0)|rgb(0, 128, 0)|rgb(0, 128, 0)|rgb(0, 128, 0)|rgb(0, 128, 0)",
        "H2 five highlight aliases read same slot green, got {r:?}"
    );
}

/// H3：highlight nonParsable 形不投影——`::highlight(1)`/`::highlight(name)a`
/// 回落元素自身值（黑），无槽误读。
#[test]
fn test_kc5r136_highlight_nonparsable_fallback() {
    use zero_script_sandbox::Sandbox;
    let mut sb = k136_sandbox_with_html(K136_HIGHLIGHT_HTML);
    let r = sb
        .execute(
            r#"
        var d = document.createElement('div');
        document.body.appendChild(d);
        [getComputedStyle(d, '::highlight(1)').color,
         getComputedStyle(d, '::highlight(name)a').color,
         getComputedStyle(d, '::highlight( )').color].join('|')
        "#,
        )
        .unwrap()
        .value;
    assert_eq!(
        r, "rgb(0, 0, 0)|rgb(0, 0, 0)|rgb(0, 0, 0)",
        "H3 non-ident/尾垃圾/空参 highlight forms fall back to element, got {r:?}"
    );
}

/// H4：first-line/first-letter part 通道读形（含 legacy 单冒号 `:first-letter`）
/// 槽投影——part 元素经 shadow innerHTML 物化（k135 读路径形）。
#[test]
fn test_kc5r136_first_line_letter_part_read() {
    use zero_script_sandbox::Sandbox;
    let mut sb = k136_sandbox_with_html(K136_FIRSTL_HTML);
    let r = sb
        .execute(
            r#"
        var c = document.createElement('div');
        var sr = c.attachShadow({mode:'open'});
        sr.innerHTML = "<span part='p'></span>";
        document.body.appendChild(c);
        var el = sr.firstElementChild;
        [getComputedStyle(el, '::first-line').color,
         getComputedStyle(el, '::first-letter').backgroundColor,
         getComputedStyle(el, ':first-letter').backgroundColor].join('|')
        "#,
        )
        .unwrap()
        .value;
    assert_eq!(
        r, "rgb(1, 2, 3)|rgb(4, 5, 6)|rgb(4, 5, 6)",
        "H4 ::part(p)::first-line/::first-letter (+legacy single colon) read slots, got {r:?}"
    );
}

/// H5：无规则页零漂移——`::backdrop` 读＝元素值（槽 None 回落链）。
#[test]
fn test_kc5r136_no_rule_zero_drift() {
    use zero_script_sandbox::Sandbox;
    let mut sb = k136_sandbox_with_html("<html><head></head><body></body></html>");
    let r = sb
        .execute(
            r#"
        var d = document.createElement('div');
        document.body.appendChild(d);
        [getComputedStyle(d, '::backdrop').backgroundColor,
         getComputedStyle(d, '::highlight(name)').color].join('|')
        "#,
        )
        .unwrap()
        .value;
    assert_eq!(
        r, "rgba(0, 0, 0, 0)|rgb(0, 0, 0)",
        "H5 no-rule page pseudo reads fall back to element values, got {r:?}"
    );
}
