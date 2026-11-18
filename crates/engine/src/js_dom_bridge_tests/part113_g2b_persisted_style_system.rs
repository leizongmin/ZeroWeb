// part113 — t-g2b impl-005：跨代持久 StyleSystem 实例（暖 CascadeCache）oracle 钉面。
// 契约：实例持久化只改变缓存温度、不改变任何输出——同代复跑（暖缓存命中）与每代
// 冷启动（fresh 实例全量）、结构 mutation 代的暖缓存增量与 fresh 全量，ComputedStyle
// 全文档逐位一致（#[derive(PartialEq)] 语义：结构相等即双路径可观察样式一致）。
// `:has()` 规则页是歧视形状：任何非空 changed 代走全文档 compute（style-system
// 增量通道兜底），冷/暖缓存若在任何声明组合上产出分歧即翻红。
// 复位语义钉：clear_generation_cache（页面导航/沙盒重注册调用）清持久实例后同
// html 冷启动结果不变——复位只降缓存温度，不引入输出差。

use crate::js_dom_bridge::computed_style::compute_document_styles;
use zero_style_system::StyleSystem;

/// :has() 歧视形状 fixture（105 处 :has() 的 github.com 瘦身版：后代/子代/结构性
/// 伪类 + nth-child 组合器面）。
const P113_HTML: &str = "<html><head><style>\
   body { margin: 0; color: #111 }\
   .card { display: block; color: #036; padding: 4px }\
   div:has(> p) { border: 1px solid red }\
   ul:has(li) { outline: 1px solid blue }\
   li:nth-child(2n+1) { color: green }\
   .deep span { visibility: visible }\
   </style></head><body>\
   <div class=\"card\"><p>t1</p></div>\
   <ul><li>a</li><li>b</li><li>c</li></ul>\
   <div class=\"deep\"><span>s</span></div>\
   </body></html>";

/// 钉 1（同代暖 vs 冷逐位一致）：同 html 连续两次 `compute_document_styles`——
/// 第二次命中暖 CascadeCache（同文档同声明组合键）；对照 fresh 实例全冷 compute。
/// 三值全等：暖缓存复用不改变任何输出（含伪元素/继承面）。
#[test]
fn p113_persisted_sys_same_generation_bitident() {
    crate::js_dom_bridge::clear_generation_cache();
    let (_, styles_first) = compute_document_styles(P113_HTML);
    // 持久实例已装填（首次 compute stash）——第二次同 html 走暖 CascadeCache。
    let (_, styles_warm) = compute_document_styles(P113_HTML);
    assert_eq!(styles_first, styles_warm, "同 html 复跑（暖缓存）== 首跑逐位一致");
    // fresh 冷实例对照（绕开持久槽：直接构造实例全量 compute）。
    let doc = zero_dom::parse_html(P113_HTML);
    let sheets = crate::pipeline::collect_stylesheets_with_parsed_external(&doc, "", None, (1280.0, 800.0));
    let mut fresh = StyleSystem::new();
    fresh.set_viewport(1280.0, 800.0);
    let styles_fresh = fresh.compute_styles(&doc, &sheets);
    assert_eq!(styles_warm, styles_fresh, "暖缓存复跑 == 冷启动全量逐位一致（:has() 页歧视形状）");
}

/// 钉 2（跨代暖增量 vs 冷全量逐位一致）：代 1 全量 → 结构 mutation（body 追加
/// `.card > p` 子树，命中 `div:has(> p)` 与 `.card` 声明组合）→ 暖实例
/// `compute_styles_incremental`；对照 fresh 实例对同一终态 doc 全量 compute。
/// 暖缓存的增量输出与冷缓存全量输出全文档逐位一致（继承面含根推入 changed）。
#[test]
fn p113_persisted_sys_incremental_generation_bitident() {
    let mut doc = zero_dom::parse_html(P113_HTML);
    let sheets = crate::pipeline::collect_stylesheets_with_parsed_external(&doc, "", None, (1280.0, 800.0));
    // 代 1：暖实例全量（装填 CascadeCache）。
    let mut warm = StyleSystem::new();
    warm.set_viewport(1280.0, 800.0);
    let mut styles = warm.compute_styles(&doc, &sheets);
    // 代 2 结构 mutation：body 追加 <div class="card"><p>new</p></div>。
    let body = crate::find_by_selector(&doc, "body").expect("body");
    let nu = doc.create_element("div");
    doc.set_attribute(nu, "class", "card");
    let np = doc.create_element("p");
    let nt = doc.create_text_node("new");
    doc.append_child(np, nt).expect("text");
    doc.append_child(nu, np).expect("nest");
    doc.append_child(body, nu).expect("append");
    let root = crate::find_by_selector(&doc, "html").expect("html");
    let changed = vec![nu, np, root];
    warm.compute_styles_incremental(&doc, &sheets, &changed, &mut styles);
    // 对照：fresh 冷实例对同一终态 doc 全量 compute。
    let mut fresh = StyleSystem::new();
    fresh.set_viewport(1280.0, 800.0);
    let styles_fresh = fresh.compute_styles(&doc, &sheets);
    assert_eq!(styles, styles_fresh, "暖缓存代际增量 == 冷启动全量逐位一致（结构 mutation + :has() 命中面）");
}

/// 钉 3（clear_generation_cache 复位语义）：页面导航/沙盒重注册清持久实例——
/// 复位后同 html 冷启动结果与复位前（暖）逐位一致，复位只降缓存温度。
#[test]
fn p113_persisted_sys_clear_resets_coldstart() {
    crate::js_dom_bridge::clear_generation_cache();
    let (_, styles_warm) = compute_document_styles(P113_HTML);
    let (_, styles_warm2) = compute_document_styles(P113_HTML);
    crate::js_dom_bridge::clear_generation_cache();
    let (_, styles_cold) = compute_document_styles(P113_HTML);
    assert_eq!(styles_warm, styles_warm2, "复位前暖复跑一致（前置健康）");
    assert_eq!(styles_warm, styles_cold, "clear_generation_cache 复位后冷启动 == 暖实例逐位一致");
}
