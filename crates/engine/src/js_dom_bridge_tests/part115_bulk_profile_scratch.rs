// part115 — t-g2b impl-007 画像 scratch（非钉面）：bulk __zw_innertext_styles 冷/热
// 构成分解——github 形状合成语料（20 sheet × N 规则 + Grid/nth-child/:has() 结构）
// 上计时 parse / 冷 bulk（parse+collect+cascade+DFS）/ 热 bulk（DFS only）/
// tree-mutation 后重算 / 单元素 gCS 参照。默认 1/8 形状（400/20/100，全测约 10s）；
// 全形状画像按需 env：P115_RULES=800 P115_SECTIONS=40 P115_SIBS=200（基线
// t-g2b-p115-postchange.json，约 6 分钟）。运行：cargo test -p zero-engine --lib
// p115 -- --nocapture（单测独占，避免 GENERATION_CACHE 进程槽与他人交错）。

/// 合成 github 形状文档：20 个 <style> sheet × `rules_per_sheet` 条规则，
/// body 由 `grid_sections` 个 Grid 段组成（每段 wide_siblings 个 span + 页脚 nav 树）。
///
/// `arm` 差分归因（P115_ARM，默认 base）：r%5==2 的 nth-child 规则按臂替换——
/// base 原样；dropnth 整条省略（候选装配+nth 尝试全消失）；cheapnth 换
/// `section.Grid-{s} > span`（span tag 桶容量不变、尝试次数不变，去 nth 逐位扫描
/// 与父链走查形状改为 O(1) 精判）；ghostnth 换 `section.ZwGhost-{s} > span`
///（桶容量不变、永不命中——去掉声明收集/级联 finalize，只留尝试+装配成本）。
/// cheapnth−ghostnth ≈ 命中规则的声明收集+finalize；base−cheapnth ≈ nth 逐位扫描；
/// base−dropnth ≈ nth 规则总贡献（含其装配份额）。
fn p115_github_shape_doc(arm: &str, rules_per_sheet: usize, grid_sections: usize, wide_siblings: usize) -> String {
    let mut head = String::from("<head>");
    for s in 0..20 {
        head.push_str(&format!("<style id=\"sheet{s}\">"));
        // 每段规则形状：工具类 / 后代链 / nth-child / 属性选择器 / :has()。
        let mut css = String::new();
        for r in 0..rules_per_sheet {
            match r % 5 {
                0 => css.push_str(&format!(".u-{s}-{r}{{display:block;padding:1px;margin:0;}}\n")),
                1 => css.push_str(&format!(
                    ".footer-{s} nav ul li a:hover{{color:red;text-decoration:underline;}}\n"
                )),
                2 => match arm {
                    "dropnth" => {}
                    "cheapnth" => css.push_str(&format!("section.Grid-{s} > span{{flex:1 1 auto;min-width:0;}}\n")),
                    "ghostnth" => {
                        css.push_str(&format!("section.ZwGhost-{s} > span{{flex:1 1 auto;min-width:0;}}\n"))
                    }
                    _ => css.push_str(&format!(
                        ".Grid-{s} > span:nth-child({}n+3){{flex:1 1 auto;min-width:0;}}\n",
                        r % 7 + 2
                    )),
                },
                3 => css.push_str(&format!(
                    "div[data-marker=\"m{s}-{r}\"] > .cell{{visibility:visible;}}\n"
                )),
                _ => css.push_str(&format!(
                    ".card-{s}:has(.badge-{r}){{outline:1px solid blue;}}\n"
                )),
            }
        }
        head.push_str(&css);
        head.push_str("</style>");
    }
    head.push_str("</head><body>");
    let mut body = String::new();
    for g in 0..grid_sections {
        body.push_str(&format!("<section class=\"Grid Grid-{g}\">"));
        for i in 0..wide_siblings {
            body.push_str(&format!(
                "<span data-marker=\"m{}-{i}\" class=\"cell u-{}-{}\"><em>t{i}</em></span>",
                g % 20,
                g % 20,
                i % 800,
            ));
        }
        body.push_str("</section>");
    }
    // 页脚 nav 树（后代链规则的宿主）。
    body.push_str("<div class=\"footer footer-0\"><nav><ul>");
    for i in 0..40 {
        body.push_str(&format!("<li><a href=\"/f{i}\">f{i}</a></li>"));
    }
    body.push_str("</ul></nav></div>");
    body.push_str("</body>");
    format!("<html>{}{}</html>", head, body)
}

#[test]
fn p115_bulk_profile_cold_warm_recompute() {
    use std::sync::{Arc, Mutex};
    use std::time::Instant;
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let rules_per_sheet = std::env::var("P115_RULES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(400);
    let grid_sections = std::env::var("P115_SECTIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20);
    let wide_siblings = std::env::var("P115_SIBS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);
    let arm = std::env::var("P115_ARM").unwrap_or_else(|_| "base".to_string());
    let html = p115_github_shape_doc(&arm, rules_per_sheet, grid_sections, wide_siblings);
    println!(
        "p115 arm={arm} doc: {} bytes, rules={}x{}, sections={}x{}",
        html.len(),
        20,
        rules_per_sheet,
        grid_sections,
        wide_siblings
    );

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(html.clone()));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/g2b/profile.html".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(
        &mut sandbox,
        &mutations,
        &dom_html,
        &page_url,
        &canvas_registry,
        None,
    );

    let t_parse = Instant::now();
    let _ = zero_dom::parse_html(&html);
    println!("p115 parse_html: {} ms", t_parse.elapsed().as_millis());

    let t_cold = Instant::now();
    let cold = sandbox
        .execute("__zw_innertext_styles(document.body.__zwSelector || 'body').length")
        .unwrap()
        .value;
    println!(
        "p115 COLD bulk (parse+collect+cascade+DFS): {} ms, wire_len={}",
        t_cold.elapsed().as_millis(),
        cold
    );

    let t_warm = Instant::now();
    let warm = sandbox
        .execute("__zw_innertext_styles(document.body.__zwSelector || 'body').length")
        .unwrap()
        .value;
    println!(
        "p115 WARM bulk (DFS+record stream): {} ms, wire_len={}",
        t_warm.elapsed().as_millis(),
        warm
    );
    assert_eq!(cold, warm, "同代两次 bulk 记录流逐位一致");

    // tree mutation → tree-dirty 全量臂（s168 全量兜底）。
    sandbox
        .execute(
            "var d = document.createElement('div'); d.setAttribute('class','cell'); d.textContent='x'; document.body.appendChild(d); 'ok'",
        )
        .unwrap();
    let t_tree = Instant::now();
    let after = sandbox
        .execute("__zw_innertext_styles(document.body.__zwSelector || 'body').length")
        .unwrap()
        .value;
    println!(
        "p115 TREE-DIRTY recompute bulk: {} ms, wire_len={}",
        t_tree.elapsed().as_millis(),
        after
    );

    // 单元素 gCS 参照（每元素查询成本基线）。
    let t_gcs = Instant::now();
    let gcs = sandbox
        .execute(
            "var n = 0; var el = document.querySelector('section'); var cs = getComputedStyle(el); n = (cs.display + '|' + cs.visibility).length; String(n)",
        )
        .unwrap()
        .value;
    println!(
        "p115 single gCS (1 element): {} ms, ret={}",
        t_gcs.elapsed().as_millis(),
        gcs
    );

    let t_gcs100 = Instant::now();
    sandbox
        .execute(
            "var els = document.querySelectorAll('span'); var acc = ''; var cap = Math.min(els.length, 100); for (var i = 0; i < cap; i++) { var c = getComputedStyle(els[i]); acc += c.display; } String(acc.length)",
        )
        .unwrap()
        .value;
    println!(
        "p115 100x gCS: {} ms",
        t_gcs100.elapsed().as_millis()
    );
}
