//! P11 索引差分 + 规模统计（2026-10-11）。
//!
//! 两件事：
//! 1. 差分正确性：索引路径（collect_matching_declarations_with_media）与无索引
//!    全量参考（collect_from_rules 传 None）在对抗性样式表 + 多元素 × 伪元素 ×
//!    媒体/容器上下文组合下逐声明相等（桶是必要条件过滤，不改变匹配与级联序）。
//! 2. 规模统计（仅 stderr 输出，供 P11 诊断读取；无断言）：
//!    ZW_P11_FIXTURE_CSS 指向真实 CSS 时打印桶大小分布与候选数样本。

use crate::matcher::{
    ContainerContext, HintKey, MatchingDecl, StylesheetIndex, build_stylesheet_index, collect_from_rules,
    collect_matching_declarations_with_media, collect_pseudo_declarations_with_media, fnv1a64,
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

    // 桶形状：顶层 hinted 外层（提示键）恰四键（wrap/hov/w2 class 提示、div tag
    // 提示——id 右端同挂 w2 提示下）；sibling、:not、裸 id 前置不入 hinted。
    // 键名称域为 FNV-1a 压缩（t-g2b impl-010 刀 1），构造与实现同函数。
    let top = &index.sheets[0].buckets;
    let hk = |kind: u8, name: &str| -> HintKey { (kind, fnv1a64(name.as_bytes())) };
    assert_eq!(
        {
            let mut ks: Vec<HintKey> = top.hinted.keys().copied().collect();
            ks.sort();
            ks
        },
        {
            let mut ks = vec![hk(b'c', "wrap"), hk(b'c', "hov"), hk(b'c', "w2"), hk(b't', "div")];
            ks.sort();
            ks
        },
        "顶层 hinted 提示键集漂移: {:?}",
        top.hinted.keys().collect::<Vec<_>>()
    );
    // 同键三候选：`.wrap .item` / `.wrap > .item`，外加 `.wrap .item::before`
    // 的主体（伪元素尾选择器按既有行为同入主桶，仅桶域随提示平移——常规 pass
    // 由精确匹配拒绝，语义不变）。两级键：外层提示 → 内层右端。
    assert_eq!(
        top.hinted[&hk(b'c', "wrap")][&hk(b'c', "item")].len(),
        3,
        "同键候选数漂移"
    );
    // 伪元素尾镜像桶同吃 hinted 形状。
    let before_mirror = index.sheets[0]
        .pseudo_by_tail
        .get("before")
        .expect("::before 镜像桶缺失");
    assert!(
        before_mirror
            .hinted
            .get(&hk(b'c', "wrap"))
            .is_some_and(|m| m.contains_key(&hk(b'c', "item"))),
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
            .get(&hk(b'c', "wrap"))
            .is_some_and(|m| m.contains_key(&hk(b'c', "m2"))),
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

// ── t-g2b impl-010 刀 5 归因普查（scratch，无断言；ZW_P11_UNIV_CENSUS_CSS 未设跳过）──
//
// drain_ms = 每候选 matches_selector 全匹配；live 普查 src_universal≈3.16M/3.2M
//（98.6%）、命中率 1.4%——universal 平桶 ~1500 条/元素 × ~2105 元素为 walk 主项。
// 本普查分类真实 CSS 的平桶选择器形状：右端无主键时前置复合能否提取 (class|tag)
// 祖先必要条件提示（刀 5 univ_hinted 桶的可行域），并在真页 DOM 上投影候选缩减。
#[test]
fn p_g2b_univ_hint_census() {
    use zero_css_parser::ast::{Combinator, SubclassSelector, TypeSelector};

    let Ok(css_path) = std::env::var("ZW_P11_UNIV_CENSUS_CSS") else {
        eprintln!("ZW_P11_UNIV_CENSUS_CSS not set; skip");
        return;
    };
    let Ok(css) = std::fs::read_to_string(&css_path) else {
        eprintln!("read failed: {css_path}");
        return;
    };
    let sheet = Parser::parse_stylesheet(&css);
    let sheets = vec![sheet];
    let index = build_stylesheet_index(&sheets);

    // 前置复合直接提示可提取性（刀 5 univ_hinted 判据草样——与 selector_hint_keys
    // 同提取面，仅免除右端主键要求）。
    fn prefix_hint(sel: &zero_css_parser::ast::Selector) -> Option<(u8, String)> {
        let parts = &sel.complex.parts;
        if parts.len() < 2 {
            return None;
        }
        match parts[parts.len() - 2].1 {
            Some(Combinator::Descendant) | Some(Combinator::Child) => {}
            _ => return None,
        }
        let prev = &parts[parts.len() - 2].0;
        let cls = prev.subclass_selectors.iter().find_map(|s| match s {
            SubclassSelector::Class(c) => Some((b'c', c.clone())),
            _ => None,
        });
        if let Some(c) = cls {
            return Some(c);
        }
        match &prev.type_selector {
            Some(TypeSelector::Tag(tag)) => Some((b't', tag.to_ascii_lowercase())),
            Some(TypeSelector::Namespaced { name, .. }) if !name.is_empty() => Some((b't', name.to_ascii_lowercase())),
            _ => None,
        }
    }

    // 递归走 (rules, index) 分类 universal 平桶。
    // 刀 6 归因采样（learning #35）：kind = 桶位可提示性；sub_kind = 右端 compound
    // 的 subclass 构成（伪元素尾/属性/伪类/裸星）——普通元素 drain 对 pe_tail 候选
    // 必然匹配失败，attr/pc 决定下一步快速过滤桶的方向。
    fn right_sub_kind(sel: &zero_css_parser::ast::Selector) -> &'static str {
        let right = &sel.complex.parts.last().unwrap().0;
        if right
            .subclass_selectors
            .iter()
            .any(|s| matches!(s, SubclassSelector::PseudoElement(_)))
        {
            return "pe_tail";
        }
        let has_attr = right
            .subclass_selectors
            .iter()
            .any(|s| matches!(s, SubclassSelector::Attribute(_)));
        let has_pc = right
            .subclass_selectors
            .iter()
            .any(|s| matches!(s, SubclassSelector::PseudoClass(_)));
        match (has_attr, has_pc) {
            (true, _) => "attr_sub",
            (false, true) => "pc_sub",
            (false, false) => {
                if right.type_selector.is_none() {
                    "bare_universal"
                } else {
                    "other"
                }
            }
        }
    }
    fn walk_census(
        rules: &[zero_css_parser::ast::Rule],
        idx: &crate::matcher::RuleListIndex,
        acc: &mut std::collections::HashMap<&'static str, usize>,
        hint_names: &mut std::collections::HashMap<(u8, String), usize>,
        sub_acc: &mut std::collections::HashMap<&'static str, usize>,
    ) {
        let b = &idx.buckets;
        for &(ri, si) in &b.universal {
            let zero_css_parser::ast::Rule::Style(style_rule) = &rules[ri] else {
                continue;
            };
            let sel = &style_rule.selectors[si];
            let parts = &sel.complex.parts;
            let kind = if parts.len() < 2 {
                "single_compound"
            } else {
                match parts[parts.len() - 2].1 {
                    Some(Combinator::Descendant) | Some(Combinator::Child) => match prefix_hint(sel) {
                        Some(k) => {
                            *hint_names.entry(k).or_default() += 1;
                            "hintable"
                        }
                        None => "no_key_prefix",
                    },
                    _ => "sibling_prefix",
                }
            };
            *acc.entry(kind).or_default() += 1;
            *sub_acc.entry(right_sub_kind(sel)).or_default() += 1;
        }
        for (ri, nb) in &idx.nested {
            if let Some(inner) = super::super::nested_block_inner_rules(&rules[*ri]) {
                walk_census(inner, &nb.index, acc, hint_names, sub_acc);
            }
        }
    }

    let mut acc = std::collections::HashMap::new();
    let mut hint_names = std::collections::HashMap::new();
    let mut sub_acc = std::collections::HashMap::new();
    let mut universal_total = 0usize;
    for (sheet_idx, sheet) in index.sheets.iter().enumerate() {
        universal_total += sheet.buckets.universal.len();
        walk_census(&sheets[sheet_idx].rules, sheet, &mut acc, &mut hint_names, &mut sub_acc);
    }
    eprintln!(
        "[p143] css_bytes={} universal_flat_total={} kinds={:?} sub_kinds={:?}",
        css.len(),
        universal_total,
        {
            let mut ks: Vec<_> = acc.iter().collect();
            ks.sort();
            ks
        },
        {
            let mut ks: Vec<_> = sub_acc.iter().collect();
            ks.sort();
            ks
        }
    );
    let mut tops: Vec<_> = hint_names.iter().collect();
    tops.sort_by(|a, b| b.1.cmp(a.1));
    let top_line: Vec<String> = tops
        .iter()
        .take(12)
        .map(|((k, n), c)| format!("{}{}={}", if *k == b'c' { "." } else { "" }, n, c))
        .collect();
    eprintln!("[p143] top_hint_keys: {}", top_line.join(" "));

    // 候选投影：真页 DOM 采样元素——当前候选数 vs 模拟 univ_hinted 后候选数
    //（after = before − universal.len + 祖先链命中的 hintable 条数）。
    let Ok(html_path) = std::env::var("ZW_P11_UNIV_CENSUS_HTML") else {
        eprintln!("ZW_P11_UNIV_CENSUS_HTML not set; skip projection");
        return;
    };
    let Ok(html) = std::fs::read_to_string(&html_path) else {
        eprintln!("read failed: {html_path}");
        return;
    };
    let doc = parse_html(&html);
    // 模拟 univ_hinted 桶：hint 键 → 条数（真实实现为 hint → 候选 (ri,si) 列表，
    // 探测语义同——祖先链逐 (tag, class) 探键命中累加）。
    let mut sim_map: std::collections::HashMap<(u8, u64), usize> = std::collections::HashMap::new();
    for ((k, n), c) in &hint_names {
        let key = if *k == b'c' {
            (b'c', super::super::fnv1a64(n.as_bytes()))
        } else {
            (b't', super::super::fnv1a64_ascii_lower(n.as_bytes()))
        };
        *sim_map.entry(key).or_default() += c;
    }
    let mut elems_all: Vec<NodeId> = Vec::new();
    for tag in ["div", "span", "a", "p", "li", "button", "input"] {
        elems_all.extend(doc.get_elements_by_tag_name(tag));
    }
    let mut before_sum = 0usize;
    let mut flat_sum = 0usize;
    let mut after_sum = 0usize;
    let mut hits_sum = 0usize;
    let n = elems_all.len();
    for el in &elems_all {
        let before = index.candidate_count(&doc, *el);
        let mut flat = 0usize;
        for s in &index.sheets {
            flat += s.buckets.universal.len();
        }
        // 祖先链键集（tag 借用小写哈希 + class 全量）。
        let mut seen: std::collections::HashSet<(u8, u64)> = std::collections::HashSet::new();
        let mut cur = doc.parent_node(*el);
        while let Some(a) = cur {
            if let Some(zero_dom::NodeKind::Element(anc)) = doc.get(a).map(|nd| &nd.kind) {
                seen.insert((b't', super::super::fnv1a64_ascii_lower(anc.local_name().as_bytes())));
                for c in &anc.class_list {
                    seen.insert((b'c', super::super::fnv1a64(c.as_bytes())));
                }
            }
            cur = doc.parent_node(a);
        }
        let hits: usize = sim_map.keys().filter(|k| seen.contains(k)).map(|k| sim_map[k]).sum();
        before_sum += before;
        flat_sum += flat;
        hits_sum += hits;
        after_sum += before - flat + hits;
    }
    if n > 0 {
        let avg = |sum: usize| sum.checked_div(n).unwrap_or_default();
        eprintln!(
            "[p143] projection elems={n} cand_before_avg={} univ_flat_avg={} cand_after_avg={} hint_hits_avg={} drain_cand_reduction={:.1}%",
            avg(before_sum),
            avg(flat_sum),
            avg(after_sum),
            avg(hits_sum),
            if before_sum > 0 {
                (flat_sum - hits_sum) as f64 * 100.0 / before_sum as f64
            } else {
                0.0
            }
        );
    }
}

// ── P-g2b 刀 5 univ_hinted 桶差分（2026-11-20，t-g2b impl-010）──
//
// 右端无主键选择器的祖先提示桶：`.w *`/`body *`/`form > *` 形状右端 universal
// 时前置复合 class/tag 是祖先链必要条件（判据见 selector_univ_hint）。桶是
// 必要条件过滤，不改变匹配与级联序——除逐声明差分外钉桶形状（互斥边界）与
// 提示真命中（差分空转绿防护）：sibling 前置、attr-only 前置、`:is()` 前置、
// 前置 universal（`* *`）留平桶。
#[test]
fn p_g2b_univ_hinted_bucket_differential() {
    let css = r#"
.w * { color: red; }
body * { color: blue; }
form > * { color: green; }
.a.b * { color: cyan; }
.x .y * { color: purple; }
div *::before { content: "v"; }
* * { color: maroon; }
[hidden] * { color: olive; }
:is(.p,.q) * { color: navy; }
s + * { color: gray; }
"#;
    let html = r#"<html><body>
<div class="w"><span class="s1">a</span><b>b-in</b>
  <div class="x"><p class="y"><u>yy</u></p></div>
</div>
<span class="s2 orphan">out</span>
<form><input type="text"><label>lbl</label><section><i>deep</i></section></form>
</body></html>"#;
    let sheet = Parser::parse_stylesheet(css);
    let sheets = vec![sheet];
    let index = build_stylesheet_index(&sheets);

    // 桶形状：univ_hinted 外层恰六键——w/a/y class 提示 + body/form/div tag
    // 提示；sibling（`s + *`）、attr 前置（`[hidden] *`）、:is 前置、前置
    // universal（`* *`）留平桶（恰四条）。
    let top = &index.sheets[0].buckets;
    let hk = |kind: u8, name: &str| -> HintKey { (kind, fnv1a64(name.as_bytes())) };
    assert_eq!(
        {
            let mut ks: Vec<HintKey> = top.univ_hinted.keys().copied().collect();
            ks.sort();
            ks
        },
        {
            let mut ks = vec![
                hk(b'c', "w"),
                hk(b'c', "a"),
                hk(b'c', "y"),
                hk(b't', "body"),
                hk(b't', "form"),
                hk(b't', "div"),
            ];
            ks.sort();
            ks
        },
        "univ_hinted 键域不符"
    );
    assert_eq!(top.universal.len(), 4, "平桶应恰留四条无提示选择器");
    assert!(
        top.univ_hinted.get(&hk(b'c', "w")).is_some_and(|v| v.len() == 1),
        ".w * 应单独入 w 提示桶"
    );

    // 逐声明差分（常规 + ::before，无上下文变体——桶路由与媒体/容器正交）。
    let doc = parse_html(html);
    let els = elements_by_tags(&doc, &["div", "span", "p", "b", "em", "u", "label", "section", "i"]);
    assert!(els.len() >= 8, "样本元素过少: {}", els.len());
    for el in els {
        let desc = format!("{el:?}");
        let indexed = collect_matching_declarations_with_media(&doc, el, &sheets, &index, None, None);
        let full = full_scan(&doc, el, &sheets, None, None, None);
        assert_eq!(
            indexed, full,
            "univ_hinted 差分失败: el={desc}\nindexed={:?}\nfull={:?}",
            indexed, full
        );
        let ip = collect_pseudo_declarations_with_media(&doc, el, &sheets, &index, None, None, "before");
        let fp = full_scan(&doc, el, &sheets, None, None, Some("before"));
        assert_eq!(
            ip, fp,
            "univ_hinted ::before 差分失败: el={desc}\nindexed={:?}\nfull={:?}",
            ip, fp
        );
    }

    // 提示真命中（候选流经 univ_hinted 且匹配语义正确，声明存在性断言——
    // 结果为源码序全量命中集，不重实现级联胜者）：s1 祖先 .w → `.w *` red 命中；
    // u 祖先链 .y → `.x .y *` purple 命中；s2 无 .w/.y 祖先 → blue 命中而
    // red/purple 排除（无提示祖先即无候选）。其余排除侧由上方差分覆盖。
    let has_color = |el: NodeId, v: &str| {
        collect_matching_declarations_with_media(&doc, el, &sheets, &index, None, None)
            .iter()
            .any(|(p, val, _, _, _)| p == "color" && val == v)
    };
    let spans = elements_by_tags(&doc, &["span"]);
    assert_eq!(spans.len(), 2, "两 span（s1/s2）");
    let (s1, s2) = (spans[0], spans[1]);
    assert!(
        doc.get_attribute_str(s1, "class").is_some_and(|c| c.contains("s1")),
        "s1 应为首个 span"
    );
    assert!(has_color(s1, "red"), ".w * 应在 s1 命中");
    assert!(has_color(s2, "blue"), "body * 应在 s2 命中");
    assert!(!has_color(s2, "red"), "s2 无 .w 祖先应排除 .w *");
    assert!(!has_color(s2, "purple"), "s2 无 .y 祖先应排除 .x .y *");
    let u_el = elements_by_tags(&doc, &["u"]);
    assert_eq!(u_el.len(), 1, "u 唯一");
    assert!(has_color(u_el[0], "purple"), ".x .y * 应在 u 命中");
}

// ── 刀 5 真页差分（scratch，无独立断言面——差分即断言；env 未设跳过）──
//
// 真实快照（github.com 3.4MB CSS + 537KB html）上抽样元素 index 路径 vs 全量
// 路径逐声明相等——univ_hinted 在真页桶形状（嵌套 @layer/@media 桶、镜像桶）
// 下的必要条件过滤不变式。ZW_P11_UNIV_DIFF_CSS/HTML 未设时静默跳过。
#[test]
fn p_g2b_univ_hint_real_diff() {
    let Ok(css_path) = std::env::var("ZW_P11_UNIV_DIFF_CSS") else {
        eprintln!("ZW_P11_UNIV_DIFF_CSS not set; skip");
        return;
    };
    let Ok(css) = std::fs::read_to_string(&css_path) else {
        eprintln!("read failed: {css_path}");
        return;
    };
    let Ok(html_path) = std::env::var("ZW_P11_UNIV_DIFF_HTML") else {
        eprintln!("ZW_P11_UNIV_DIFF_HTML not set; skip");
        return;
    };
    let Ok(html) = std::fs::read_to_string(&html_path) else {
        eprintln!("read failed: {html_path}");
        return;
    };
    let sheet = Parser::parse_stylesheet(&css);
    let sheets = vec![sheet];
    let index = build_stylesheet_index(&sheets);
    let doc = parse_html(&html);

    // 抽样：混合标签 + 首尾元素（浅/深树位）。
    let mut els: Vec<NodeId> = Vec::new();
    for tag in ["div", "span", "a", "p", "button", "input", "svg"] {
        let found = doc.get_elements_by_tag_name(tag);
        if let Some(first) = found.first() {
            els.push(*first);
        }
        if let Some(last) = found.last() {
            els.push(*last);
        }
    }
    let (total, univ, ..) = index.stats();
    eprintln!("[p143d] selectors={total} universal_flat={univ} sampled={}", els.len());
    for el in &els {
        let desc = format!("{el:?}");
        let indexed = collect_matching_declarations_with_media(&doc, *el, &sheets, &index, None, None);
        let full = full_scan(&doc, *el, &sheets, None, None, None);
        assert_eq!(
            indexed, full,
            "真页差分失败: el={desc}\nindexed={:?}\nfull={:?}",
            indexed, full
        );
    }
}

// ── 刀 6a：univ_attr 属性需求桶差分 ──
//
// 桶形状断言 + 逐元素 indexed vs full_scan 差分 + 提示真命中存在性：
// - 直属 Attribute 取首个属性名作键（[hidden][disabled] → hidden 单键）；
// - `:is`/`:where` 参数内属性**不挖**（非直属，参数语义是子匹配非存在性要求）
//   → 留平桶，候选全量 + 精判兜底；
// - `*::before` 右端无直属 attr → 平桶；`p[lang]` 右端有 tag 主键 → tagged 桶。
#[test]
fn p_g2b_univ_attr_bucket_differential() {
    let css = r#"
[hidden] { display: none; }
[type="text"] { color: red; }
[data-x="1"] { color: blue; }
[hidden][disabled] { color: green; }
[aria-expanded] { color: cyan; }
:is(.p,.q) { color: purple; }
:where([href]) { color: olive; }
*::before { content: "b"; }
p[lang] { color: pink; }
* { outline: none; }
"#;
    let html = r##"<html><body>
<div hidden>h</div>
<input type="text" data-x="1">
<a href="#a">link</a>
<button disabled>btn</button>
<div class="p">pc</div>
<span>plain</span>
<p lang="zh">l</p>
</body></html>"##;
    let sheet = Parser::parse_stylesheet(css);
    let sheets = vec![sheet];
    let index = build_stylesheet_index(&sheets);

    // 桶形状：univ_attr 键域恰四——hidden/type/data-x/aria-expanded（小写化）；
    // 平桶恰四——:is/:where 参数形态、前置 universal 伪元素、裸星。
    let top = &index.sheets[0].buckets;
    let ak = |name: &str| super::super::fnv1a64_ascii_lower(name.as_bytes());
    assert_eq!(
        {
            let mut ks: Vec<u64> = top.univ_attr.keys().copied().collect();
            ks.sort();
            ks
        },
        {
            let mut ks = vec![ak("hidden"), ak("type"), ak("data-x"), ak("aria-expanded")];
            ks.sort();
            ks
        },
        "univ_attr 键域不符"
    );
    assert_eq!(top.universal.len(), 4, "平桶应恰留四条无属性需求选择器");
    assert!(
        top.univ_attr.get(&ak("hidden")).is_some_and(|v| v.len() == 2),
        "[hidden] 与 [hidden][disabled]（首个属性键）应同入 hidden 键"
    );

    let doc = parse_html(html);
    let els = elements_by_tags(&doc, &["div", "span", "a", "input", "button", "p"]);
    assert!(els.len() >= 6, "样本元素过少: {}", els.len());
    for el in els {
        let desc = format!("{el:?}");
        let indexed = collect_matching_declarations_with_media(&doc, el, &sheets, &index, None, None);
        let full = full_scan(&doc, el, &sheets, None, None, None);
        assert_eq!(
            indexed, full,
            "univ_attr 差分失败: el={desc}\nindexed={:?}\nfull={:?}",
            indexed, full
        );
    }

    // 存在性断言（结果为源码序全量命中集，断言存在性不重实现级联胜者）。
    let has = |el: NodeId, prop: &str, v: &str| {
        collect_matching_declarations_with_media(&doc, el, &sheets, &index, None, None)
            .iter()
            .any(|(p, val, _, _, _)| p == prop && val == v)
    };
    let by_tag = |t: &str, i: usize| elements_by_tags(&doc, &[t])[i];
    let div_h = by_tag("div", 0);
    let input = by_tag("input", 0);
    let a_el = by_tag("a", 0);
    let btn = by_tag("button", 0);
    let div_p = by_tag("div", 1);
    let span = by_tag("span", 0);
    let p_el = by_tag("p", 0);
    assert!(has(div_h, "display", "none"), "[hidden] 应在 div[hidden] 命中");
    assert!(!has(div_h, "color", "green"), "[hidden][disabled] 缺 disabled 应失败");
    assert!(has(input, "color", "red"), "[type=text] 应在 input 命中");
    assert!(has(input, "color", "blue"), "[data-x=1] 应在 input 命中");
    assert!(
        has(a_el, "color", "olive"),
        ":where([href]) 应在 a[href] 精判命中（平桶候选）"
    );
    assert!(!has(span, "color", "olive"), "span 无 href 应排除 :where([href])");
    assert!(
        !has(btn, "color", "green"),
        "button 无 hidden 键应不取 [hidden][disabled] 候选"
    );
    assert!(has(div_p, "color", "purple"), ":is(.p,.q) 应在 div.p 命中");
    assert!(!has(span, "color", "purple"), "span 无 .p/.q 应排除 :is");
    assert!(has(p_el, "color", "pink"), "p[lang] 应在 p[lang] 命中（tagged 桶路径）");
    assert!(has(div_h, "outline", "none"), "裸星平桶候选应对全部元素命中");
}
