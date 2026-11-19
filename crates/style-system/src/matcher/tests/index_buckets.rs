//! P11 索引差分 + 规模统计（2026-10-11）。
//!
//! 两件事：
//! 1. 差分正确性：索引路径（collect_matching_declarations_with_media）与无索引
//!    全量参考（collect_from_rules 传 None）在对抗性样式表 + 多元素 × 伪元素 ×
//!    媒体/容器上下文组合下逐声明相等（桶是必要条件过滤，不改变匹配与级联序）。
//! 2. 规模统计（仅 stderr 输出，供 P11 诊断读取；无断言）：
//!    ZW_P11_FIXTURE_CSS 指向真实 CSS 时打印桶大小分布与候选数样本。

use crate::matcher::{
    ContainerContext, MatchingDecl, StylesheetIndex, build_stylesheet_index, collect_from_rules,
    collect_matching_declarations_with_media, collect_pseudo_declarations_with_media,
};
use zero_css_parser::media_query::{
    MediaContext, MediaType, PointerValue, PrefersColorSchemeValue, ReducedMotionValue,
};
use zero_css_parser::{Parser, Stylesheet};
use zero_dom::{Document, NodeId, parse_html};

// ── 差分 ──

fn media_ctx() -> MediaContext {
    MediaContext {
        viewport_width: 800.0,
        viewport_height: 600.0,
        media_type: MediaType::Screen,
        prefers_color_scheme: PrefersColorSchemeValue::Light,
        prefers_reduced_motion: ReducedMotionValue::NoPreference,
        pointer_type: PointerValue::Fine,
        resolution_dpi: 96.0,
    }
}

/// 无索引全量参考：直接走 collect_from_rules(None)（self_at_rule_dispatch 全量路径）。
fn full_scan(
    doc: &Document,
    element: NodeId,
    sheets: &[Stylesheet],
    media_ctx: Option<&MediaContext>,
    container_ctx: Option<&ContainerContext>,
    pseudo: Option<&str>,
) -> Vec<MatchingDecl> {
    let mut results = Vec::new();
    let mut layer_counter: usize = 0;
    let mut layer_map = std::collections::HashMap::new();
    for sheet in sheets {
        collect_from_rules(
            doc,
            element,
            &sheet.rules,
            None,
            &mut results,
            media_ctx,
            container_ctx,
            None,
            &mut layer_counter,
            &mut layer_map,
            pseudo,
        );
    }
    results
}

const ADVERSARIAL_CSS: &str = r#"
/* tag 桶 */
div { color: black; }
li, p { margin: 1px; }
/* TV-2：tag 主键小写归一——大写 tag 选择器须落 div 桶，否则索引路径漏配 */
DIV { letter-spacing: 2px; }
/* class 桶（含链、同规则多选择器跨界桶） */
.card { color: red; }
.card.featured { color: blue; }
#hero.card { color: green; }
#hero { border: 1px solid; }
.special, .card { padding: 2px; }
/* :not/:is 参数不挖（元素可能匹配也可能不匹配） */
.card:not(.y) { color: purple; }
:is(.list, .menu) .item { color: teal; }
div:not(#hero) { outline: 1px; }
/* attr-only / 裸伪类 → universal */
[data-x] { color: olive; }
[data-x="1"] { color: navy; }
*:hover { color: pink; }
/* universal */
* { line-height: normal; }
/* 伪元素（tag / class 键） */
.card::before { content: "B"; }
em::after { content: "A"; }
#hero::after { content: "H"; }
/* @media 嵌套（匹配 / 不匹配 / 深嵌套） */
@media screen {
  .card { color: maroon; }
  div span { font-weight: bold; }
  @media (min-width: 500px) {
    .featured { outline: 2px; }
  }
  @supports (display: grid) {
    .card { display: grid; }
    @container (min-width: 200px) {
      .item { color: khaki; }
    }
  }
  /* P124/DEF-1：层名首现于条件块内、块外复现且与他层交错——pseudo 门跳过
     本块时不得丢 R4246 首现注册（含层子树禁用门禁）。 */
  @layer condreg { .card { color: sienna; } }
  @layer condstmt;
}
@media print and (min-width: 2000px) {
  .card { color: silver; }
}
@media (min-width: 99999px) {
  .card { color: coral; }
}
/* @layer：语句 + 命名块 + 匿名块，与顶层规则交错 */
@layer base, top;
@layer base {
  .card { color: gray; }
}
@layer top {
  .card { color: lightgreen; }
}
@layer {
  .item { color: gold; }
}
/* P124/DEF-1：条件块内首现层的块外复现（伪元素规则与他层交错） */
@layer base { .card::before { content: "base-before"; } }
@layer condreg { .card::before { content: "condreg-before"; } }
@layer condstmt { .card::after { content: "condstmt-after"; } }
/* @supports 真/假 */
@supports (display: block) {
  .text { letter-spacing: 1px; }
}
@supports (foo-bar-xyz: nope) {
  .text { letter-spacing: 99px; }
}
/* @container 匹配/不匹配 */
@container (min-width: 200px) {
  .card { color: crimson; }
}
@container (min-width: 5000px) {
  .card { color: whitesmoke; }
}
/* 长后代链（评估成本形态） */
ul li p span em { vertical-align: top; }
/* 靠后的顶层规则（源码序 vs @media 块） */
.card { color: final; }
"#;

const ADVERSARIAL_HTML: &str = r##"<html><body>
<div id="hero" class="card featured">h</div>
<span class="card">s</span>
<p class="text muted" data-x="1">p</p>
<ul class="list"><li class="item">one</li><li>two</li></ul>
<section id="main">m</section>
<a class="link external" href="#">a</a>
<div class="y">y</div>
<em class="card">e</em>
<div>plain</div>
<div class="menu"><li class="item">mi</li></div>
<!-- TV-1：同规则多选择器跨桶双匹配——.special 与 .card 同规则分落两 class 桶，
     元素两类命中产生同 (ri, 异 si) 双候选，last_matched_rule 去重须保持单份声明 -->
<div class="special card">sc</div>
</body></html>"##;

fn elements_by_tags(doc: &Document, tags: &[&str]) -> Vec<NodeId> {
    let mut out = Vec::new();
    for t in tags {
        out.extend(doc.get_elements_by_tag_name(t));
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// 差分：索引路径 == 无索引全量，覆盖 常规/伪元素 × 媒体上下文 × 容器上下文。
#[test]
fn p11_indexed_equals_full_scan() {
    let sheet = Parser::parse_stylesheet(ADVERSARIAL_CSS);
    let sheets = vec![sheet];
    let index = build_stylesheet_index(&sheets);
    let doc = parse_html(ADVERSARIAL_HTML);
    let els = elements_by_tags(&doc, &["div", "span", "p", "li", "section", "a", "em", "ul"]);
    assert!(els.len() >= 10, "样本元素过少: {}", els.len());

    let mctx = media_ctx();
    let cctx_match = ContainerContext::with_size(300.0, 600.0);
    let cctx_miss = ContainerContext::with_size(100.0, 600.0);
    let ctx_variants: [(&str, Option<&MediaContext>, Option<&ContainerContext>); 5] = [
        ("media+container", Some(&mctx), Some(&cctx_match)),
        ("media only", Some(&mctx), None),
        ("container only", None, Some(&cctx_match)),
        ("no ctx", None, None),
        ("container miss", Some(&mctx), Some(&cctx_miss)),
    ];

    for el in els {
        let desc = format!("{:?}", el);
        for (cname, mc, cc) in &ctx_variants {
            let indexed = collect_matching_declarations_with_media(&doc, el, &sheets, &index, mc.as_deref(), *cc);
            let full = full_scan(&doc, el, &sheets, mc.as_deref(), *cc, None);
            assert_eq!(
                indexed, full,
                "常规路径差分失败: el={desc} ctx={cname}\nindexed={:?}\nfull={:?}",
                indexed, full
            );
            for pseudo in ["before", "after"] {
                let ip = collect_pseudo_declarations_with_media(&doc, el, &sheets, &index, mc.as_deref(), *cc, pseudo);
                let fp = full_scan(&doc, el, &sheets, mc.as_deref(), *cc, Some(pseudo));
                assert_eq!(
                    ip, fp,
                    "伪元素 ::{pseudo} 差分失败: el={desc} ctx={cname}\nindexed={:?}\nfull={:?}",
                    ip, fp
                );
            }
        }
    }
}

/// 多样式表（引擎把每个 <link> CSS 作为独立样式表传入）下的同性质差分。
#[test]
fn p11_indexed_equals_full_scan_multi_sheet() {
    let sheet_a = Parser::parse_stylesheet("@media all { .card { background: a; } } .card { background: b; }");
    let sheet_b = Parser::parse_stylesheet(
        "@layer top { .card { color: l1; } } #hero { color: h1; } @media (min-width: 500px) { .featured { color: f1; } }",
    );
    let sheets = vec![sheet_a, sheet_b];
    let index = build_stylesheet_index(&sheets);
    let doc = parse_html(r#"<div id="hero" class="card featured">x</div><span class="card">y</span>"#);
    let els = elements_by_tags(&doc, &["div", "span"]);
    let mctx = media_ctx();
    for el in els {
        let indexed = collect_matching_declarations_with_media(&doc, el, &sheets, &index, Some(&mctx), None);
        let full = full_scan(&doc, el, &sheets, Some(&mctx), None, None);
        assert_eq!(indexed, full, "多样式表差分失败 el={el:?}");
    }
}

// ── 规模统计（P11 诊断输出，无断言；ZW_P11_FIXTURE_CSS 未设时跳过） ──

#[test]
fn p11_index_stats() {
    let Ok(path) = std::env::var("ZW_P11_FIXTURE_CSS") else {
        eprintln!("ZW_P11_FIXTURE_CSS not set; skip");
        return;
    };
    let Ok(css) = std::fs::read_to_string(&path) else {
        eprintln!("read failed: {path}");
        return;
    };
    let sheet = Parser::parse_stylesheet(&css);
    let index = build_stylesheet_index(std::slice::from_ref(&sheet));

    // 桶大小：借用内部字段的统计（同 crate 内可访问私有字段——经由 pub(crate) 访问器）。
    let (n_selector_total, n_universal, n_buckets_class, n_buckets_id, n_buckets_tag, n_nested) = index.stats();
    eprintln!(
        "selectors={n_selector_total} universal={n_universal} \
         buckets: class={n_buckets_class} id={n_buckets_id} tag={n_buckets_tag} nested={n_nested}"
    );

    // 候选数样本：flat CSS 全顶层，元素样本直接从 HTML 取。
    let html_path = std::env::var("ZW_P11_FIXTURE_HTML").unwrap_or_default();
    let Ok(html) = std::fs::read_to_string(&html_path) else {
        eprintln!("ZW_P11_FIXTURE_HTML not set; skip candidate sampling");
        return;
    };
    let doc = parse_html(&html);
    // 抽样 200 个元素：每 50 个取 1 个
    let mut sample = Vec::new();
    for tag in ["div", "span", "a", "p", "li", "button"] {
        for (i, el) in doc.get_elements_by_tag_name(tag).into_iter().enumerate() {
            if i % 50 == 0 {
                sample.push(el);
            }
        }
    }
    for el in sample.iter().take(20) {
        let n = index.candidate_count(&doc, *el);
        eprintln!("el={el:?} candidates={n}");
    }
}

// ── P-g2b 第二级 hinted 桶差分（2026-11-19，t-g2b impl-007 leg①）──
//
// 右端键 × 祖先提示复合桶的专用差分：hinted 只改候选来源、不改变匹配与级联序
// （必要条件论证见 RuleListBuckets::hinted 桶注）。除逐声明差分外，显式断言
// hinted 桶实际吃到形状（差分可能空转绿——选择器从未入 hinted 桶则本改动未被
// 测试覆盖），并钉互斥边界：sibling 组合器（`+`/`~`）与 `:not(.x)` 前置复合
// 留平桶，裸 id 前置复合（无 class/tag 可挖）留平桶。
#[test]
fn p_g2b_hinted_bucket_differential() {
    let css = r#"
.wrap .item { color: red; }
.wrap > .item { color: green; }
div > span { color: blue; }
.hov:hover .item { color: olive; }
.w2 #it { color: purple; }
.a + .item { color: cyan; }
.a ~ span { color: magenta; }
:not(.x) .item { color: black; }
#only .item { color: brown; }
@media all { .wrap .m2 { color: navy; } }
.wrap .item::before { content: "b"; }
"#;
    let html = r#"<html><body>
<div class="wrap"><div class="mid"><span class="item">in</span><b class="m2">n</b></div></div>
<span class="item orphan">out</span>
<div id="only"><p class="item">po</p></div>
<div class="a"><span class="item">first</span><em class="item">next</em></div>
<div class="hov"><span class="item">hov</span></div>
<div class="w2"><b id="it">x</b></div>
</body></html>"#;
    let sheet = Parser::parse_stylesheet(css);
    let sheets = vec![sheet];
    let index = build_stylesheet_index(&sheets);

    // 桶形状：顶层 hinted 恰四键（descendant class-hint 双选择器共桶 / tag-hint /
    // :hover 提示 / id 右端 + class 提示）；sibling、:not、裸 id 前置不入 hinted。
    let top = &index.sheets[0].buckets;
    let k_item_wrap = ((b'c', "item".to_string()), (b'c', "wrap".to_string()));
    let k_span_div = ((b't', "span".to_string()), (b't', "div".to_string()));
    let k_item_hov = ((b'c', "item".to_string()), (b'c', "hov".to_string()));
    let k_it_w2 = ((b'i', "it".to_string()), (b'c', "w2".to_string()));
    assert_eq!(
        {
            let mut ks: Vec<_> = top.hinted.keys().cloned().collect();
            ks.sort();
            ks
        },
        {
            let mut ks = vec![
                k_item_wrap.clone(),
                k_span_div.clone(),
                k_item_hov.clone(),
                k_it_w2.clone(),
            ];
            ks.sort();
            ks
        },
        "顶层 hinted 桶键集漂移: {:?}",
        top.hinted.keys().collect::<Vec<_>>()
    );
    // 同键三候选：`.wrap .item` / `.wrap > .item`，外加 `.wrap .item::before`
    // 的主体（伪元素尾选择器按既有行为同入主桶，仅桶域随提示平移——常规 pass
    // 由精确匹配拒绝，语义不变）。
    assert_eq!(top.hinted[&k_item_wrap].len(), 3, "同键候选数漂移");
    // 伪元素尾镜像桶同吃 hinted 形状。
    let before_mirror = index.sheets[0]
        .pseudo_by_tail
        .get("before")
        .expect("::before 镜像桶缺失");
    assert!(
        before_mirror.hinted.contains_key(&k_item_wrap),
        "伪元素镜像 hinted 未入桶: {:?}",
        before_mirror.hinted.keys().collect::<Vec<_>>()
    );
    // @media 嵌套块内 hinted 规则随递归建索引入桶。
    let media_nested = index.sheets[0].nested.values().next().expect("@media 嵌套缺失");
    assert!(
        media_nested
            .index
            .buckets
            .hinted
            .contains_key(&((b'c', "m2".to_string()), (b'c', "wrap".to_string()))),
        "@media 内 hinted 未入桶"
    );

    // 逐声明差分（常规 + ::before，无上下文变体——桶路由与媒体/容器正交）。
    let doc = parse_html(html);
    let els = elements_by_tags(&doc, &["div", "span", "p", "b", "em"]);
    assert!(els.len() >= 10, "样本元素过少: {}", els.len());
    for el in els {
        let desc = format!("{el:?}");
        let indexed = collect_matching_declarations_with_media(&doc, el, &sheets, &index, None, None);
        let full = full_scan(&doc, el, &sheets, None, None, None);
        assert_eq!(
            indexed, full,
            "hinted 差分失败: el={desc}\nindexed={:?}\nfull={:?}",
            indexed, full
        );
        let ip = collect_pseudo_declarations_with_media(&doc, el, &sheets, &index, None, None, "before");
        let fp = full_scan(&doc, el, &sheets, None, None, Some("before"));
        assert_eq!(
            ip, fp,
            "hinted ::before 差分失败: el={desc}\nindexed={:?}\nfull={:?}",
            ip, fp
        );
    }
}
