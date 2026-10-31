//! p125r1(TV-2) 常驻回归：每查询位置索引的广谱语义——多父键控、nth-of-type 分桶、
//! 负系数 an+b、nth-last 族、matches() 单调用与查询路径两路同源、`:has` 内嵌位置
//! 伪类复用。防回归点：索引实现若漂移（键控/分桶/回填语义），以下断言先红。

use zero_dom::parse_html;

fn wide_menus(n: usize) -> String {
    let mut h = String::from("<html><body>");
    for m in 0..2 {
        h.push_str(&format!("<div id=\"m{}\">", m));
        for i in 0..n {
            h.push_str(&format!("<a href=\"x{}\" data-i=\"{}\">a{}</a>", i, i, i));
        }
        h.push_str("</div>");
    }
    // 混 tag 宽父：a/b 交替 1000。
    h.push_str("<div id=\"mix\">");
    for i in 0..1000 {
        if i % 2 == 0 {
            h.push_str(&format!("<a>ma{}</a>", i));
        } else {
            h.push_str(&format!("<b>mb{}</b>", i));
        }
    }
    h.push_str("</div></body></html>");
    h
}

#[test]
fn p125_position_index_broad_spectrum_shapes() {
    let doc = parse_html(&wide_menus(2000));
    let root = doc.root();
    let m0 = doc.query_selector(root, "#m0").expect("m0");
    let m1 = doc.query_selector(root, "#m1").expect("m1");
    let all0 = doc.query_selector_all(root, "#m0 > a");
    let all1 = doc.query_selector_all(root, "#m1 > a");
    assert_eq!(all0.len(), 2000, "m0 锚点数");
    assert_eq!(all1.len(), 2000, "m1 锚点数");

    // 多父键控：同一次查询会话内两个父各自命中精确 k。
    let h1 = doc.query_selector(root, "#m0 > a:nth-child(1999)").expect("m0 1999");
    let h2 = doc.query_selector(root, "#m1 > a:nth-child(7)").expect("m1 7");
    assert_eq!(h1, all0[1998], "多父键控 m0:nth-child(1999)");
    assert_eq!(h2, all1[6], "多父键控 m1:nth-child(7)");

    // 3n 文档序 oracle。
    let hits = doc.query_selector_all(root, "#m0 > a:nth-child(3n)");
    assert_eq!(hits.len(), 666, "3n 命中数");
    for (k, a) in all0.iter().enumerate() {
        assert_eq!(hits.contains(a), (k + 1) % 3 == 0, "3n 文档序 @{}", k + 1);
    }

    // 负系数 an+b。
    let neg = doc.query_selector_all(root, "#m1 > a:nth-child(-n+3)");
    assert_eq!(neg, all1[..3], "-n+3 → 前 3");
    let neg2 = doc.query_selector_all(root, "#m1 > a:nth-child(-2n+5)");
    let want5: Vec<_> = [1usize, 3, 5].iter().map(|&k| all1[k - 1]).collect();
    assert_eq!(neg2, want5, "-2n+5 → {{5,3,1}} 倒序");

    // odd/even/2n+1。
    assert_eq!(doc.query_selector_all(root, "#m0 > a:nth-child(odd)").len(), 1000);
    assert_eq!(doc.query_selector_all(root, "#m0 > a:nth-child(2n+1)").len(), 1000);
    assert_eq!(doc.query_selector_all(root, "#m0 > a:nth-child(even)").len(), 1000);
    assert!(
        doc.query_selector(root, "#m0 > a:nth-child(2001)").is_none(),
        "越界 miss"
    );

    // nth-of-type 分桶（a/b 交替）。
    let mix_a = doc.query_selector_all(root, "#mix > a");
    let mix_b = doc.query_selector_all(root, "#mix > b");
    assert_eq!(mix_a.len(), 500, "mix a 数");
    assert_eq!(mix_b.len(), 500, "mix b 数");
    let a500 = doc.query_selector(root, "#mix > a:nth-of-type(500)").expect("a500");
    assert_eq!(a500, mix_a[499], "a:nth-of-type(500) 与文档序一致");
    let b_first = doc.query_selector(root, "#mix > b:first-of-type").expect("b first");
    assert_eq!(b_first, mix_b[0], "b:first-of-type 与文档序一致");
    assert_eq!(doc.query_selector_all(root, "#mix > a:nth-of-type(2n)").len(), 250);

    // nth-last 族。
    let last1 = doc.query_selector(root, "#m0 > a:nth-last-child(1)").expect("last");
    assert_eq!(last1, all0[1999], "nth-last-child(1) = 末位锚点");
    let last3 = doc.query_selector_all(root, "#m0 > a:nth-last-child(3)");
    assert_eq!(last3.len(), 1);
    assert_eq!(last3[0], all0[1997], "nth-last-child(3) = 倒数第三");

    // matches() 单调用（索引未装载）与查询路径同值（抽样）。
    for i in (0..2000).step_by(137) {
        assert!(
            doc.matches(all0[i], &format!("a:nth-child({})", i + 1)),
            "matches 两路同源 @{}",
            i + 1
        );
        if i + 1 < 2000 {
            assert!(
                !doc.matches(all0[i], &format!("a:nth-child({})", i + 2)),
                "matches 负向 @{}",
                i + 1
            );
        }
    }

    // `:has` 内嵌位置伪类（参数取首个 `)` 为既有解析边界，此处用无括号位置伪类）。
    let has_d = doc.query_selector_all(root, "div:has(a:first-child)");
    assert_eq!(has_d.len(), 3, ":has(a:first-child) 命中全部 3 容器");
    assert!(doc.query_selector_all(root, "div:has(> b:first-child)").is_empty());

    // 链式：廉价成分 + 位置伪类组合。
    assert_eq!(
        doc.query_selector_all(root, "#m0 > a[href^=x]:nth-child(2n)").len(),
        1000
    );
    assert!(
        doc.query_selector_all(root, "#m0 > a[href=zzz]:nth-child(2n)")
            .is_empty()
    );

    let _ = (m0, m1);
}
