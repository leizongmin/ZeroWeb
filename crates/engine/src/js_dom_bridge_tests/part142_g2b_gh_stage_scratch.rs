// t-g2b impl-010 分段画像 scratch（定去留，非钉面）：真页快照分段计时——parse /
// compute_document_styles（parse+collect+cascade 共汇点）/ innertext bulk（impl-009
// cached 变体）/ 换代臂模拟（with_cached_document_styles 键前进 → 全量重算 + f）。
// 背景：g29 定谳 heavy 臂 6.7-7.6s = 换代分支全量重算（非 bulk 遍历），impl-007
// verdict 预告的「级联内非匹配成分」待画像定占比。learning #35 纪律：归因只认分段
// 实测。运行：P142_SNAPSHOT=<html 路径> cargo test -p zero-engine --lib p142 --
// --nocapture --test-threads=1；env 未设时静默跳过。

#[test]
fn p142_gh_snapshot_stage_profile() {
    let Ok(path) = std::env::var("P142_SNAPSHOT") else {
        eprintln!("[p142] P142_SNAPSHOT 未设——跳过（scratch 按需运行）");
        return;
    };
    let html = std::fs::read_to_string(&path).expect("快照可读");
    eprintln!("[p142] snapshot_bytes={}", html.len());

    // ① parse 单段。
    let t = std::time::Instant::now();
    let doc = parse_html(&html);
    eprintln!("[p142] stage_parse_ms={} nodes={}", t.elapsed().as_millis(), doc.node_count());

    // ② compute_document_styles 共汇点（parse+collect+cascade 全链）。
    let t = std::time::Instant::now();
    let (doc2, styles) = crate::js_dom_bridge::computed_style::compute_document_styles(&html);
    let compute_ms = t.elapsed().as_millis();
    eprintln!("[p142] stage_compute_ms={}", compute_ms);

    // ③ innertext bulk（cached 变体——与生产回调同实现路径）。
    let ctx = crate::js_dom_bridge::UniqSelCtx::new(0, 0, 0, 0, doc2.node_count());
    let t = std::time::Instant::now();
    let wire = crate::js_dom_bridge::computed_style::innertext_bulk_styles_doc(&doc2, &styles, "body", ctx);
    eprintln!("[p142] stage_bulk_ms={} wire_len={}", t.elapsed().as_millis(), wire.len());

    // ④ 换代臂模拟：style_version 前进（0→1）+ 一条 pending attr 写——复现生产
    // heavy 路径（lazy else 分支 clone + with_cached_document_styles 换代全量重算 + f）。
    let mutations = vec![crate::js_dom_bridge::DomMutation::SetAttr {
        selector: "body".to_string(),
        name: "data-p142".to_string(),
        value: "1".to_string(),
    }];
    let t = std::time::Instant::now();
    let wire2 = crate::js_dom_bridge::computed_style_cache::with_cached_document_styles(
        &html,
        0,
        1,
        &mutations,
        |doc, styles| {
            let c = crate::js_dom_bridge::UniqSelCtx::new(0, 0, 1, 0, doc.node_count());
            crate::js_dom_bridge::computed_style::innertext_bulk_styles_doc(doc, styles, "body", c)
        },
    );
    eprintln!(
        "[p142] stage_heavy_arm_ms={} wire_len={}（键前进全量重算 + bulk，生产 heavy 臂同链）",
        t.elapsed().as_millis(),
        wire2.len()
    );

    // ⑤ 同代二次调用（轻臂对照——lazy 预检命中 + 空队列）。
    let t = std::time::Instant::now();
    let wire3 = crate::js_dom_bridge::computed_style_cache::with_cached_document_styles(
        &html,
        0,
        1,
        &[],
        |doc, styles| {
            let c = crate::js_dom_bridge::UniqSelCtx::new(0, 0, 1, 0, doc.node_count());
            crate::js_dom_bridge::computed_style::innertext_bulk_styles_doc(doc, styles, "body", c)
        },
    );
    eprintln!("[p142] stage_light_arm_ms={} wire_len={}", t.elapsed().as_millis(), wire3.len());

    // ⑥ 外部 CSS 注入全链（live 形状补差——本地纯快照臂 css=""，live 走
    // active_external_css 注入 github 数千条外部规则；compute_styles_for_doc 与
    // 生产 cascade 共汇点）。P142_CSS=<css 路径> 门控。
    if let Ok(css_path) = std::env::var("P142_CSS") {
        let css = std::fs::read_to_string(&css_path).expect("css 可读");
        eprintln!("[p142] css_bytes={}", css.len());
        let t = std::time::Instant::now();
        let (styles_css, sheets) =
            crate::js_dom_bridge::computed_style::compute_styles_for_doc(&doc, &css);
        let ms = t.elapsed().as_millis();
        eprintln!(
            "[p142] stage_compute_extcss_ms={} sheets={}（parse+collect(含外部)+cascade 全链，live 重臂主体候选）",
            ms,
            sheets.len()
        );
        let ctx = crate::js_dom_bridge::UniqSelCtx::new(1, 0, 0, 0, doc.node_count());
        let t = std::time::Instant::now();
        let wire4 = crate::js_dom_bridge::computed_style::innertext_bulk_styles_doc(&doc, &styles_css, "body", ctx);
        eprintln!("[p142] stage_bulk_extcss_ms={} wire_len={}", t.elapsed().as_millis(), wire4.len());
    }
}
