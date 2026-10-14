// part51：P9-9/R5107——getComputedStyle 合成源纳入外部样式表（webview cached_css 经
// SetDomSnapshot 钉版装线程局部，collect_stylesheets css 参数与布局管线同源）。

/// 沙箱级通路测试公共装具：注册回调 + 装 shim，返回 sandbox（与 part36 同款）。
fn p51_sandbox_with_html(html: &str) -> zero_script_sandbox::V8Sandbox {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
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

/// gCS 查询助手（真实 shim 通路：getComputedStyle(...).getPropertyValue）。
fn p51_gcs(sandbox: &mut zero_script_sandbox::V8Sandbox, sel: &str, prop: &str) -> String {
    use zero_script_sandbox::Sandbox;
    let script = format!("getComputedStyle(document.querySelector('{sel}')).getPropertyValue('{prop}')");
    sandbox.execute(&script).unwrap().value
}

#[test]
fn test_p99_gcs_reflects_external_css() {
    // 无外部 CSS：font-family 返空（合成源只有内联 <style>，h1 无 font 规则）。
    let html = "<html><head><style>body { color: black }</style></head><body><h1>t</h1></body></html>";
    crate::js_dom_bridge::clear_generation_cache();
    crate::js_dom_bridge::set_active_external_css("");
    let mut sb = p51_sandbox_with_html(html);
    assert_eq!(p51_gcs(&mut sb, "h1", "font-family"), "");

    // 装外部 CSS（模拟 SetDomSnapshot 钉版装填）：gCS 立即反映（代际键含 css，换代替换）。
    crate::js_dom_bridge::set_active_external_css("h1 { font-family: GithubMono, monospace; font-weight: 560; }");
    let mut sb = p51_sandbox_with_html(html);
    assert_eq!(p51_gcs(&mut sb, "h1", "font-family"), "GithubMono, monospace");
    assert_eq!(p51_gcs(&mut sb, "h1", "font-weight"), "560");
}

#[test]
fn test_p99_external_css_var_resolution() {
    // :root 自定义属性经外部 CSS 声明、声明值 var() 消费——布局管线既有语义，gCS 同源。
    let html = "<html><head><style></style></head><body><h1>t</h1><p>p</p></body></html>";
    crate::js_dom_bridge::clear_generation_cache();
    crate::js_dom_bridge::set_active_external_css(
        ":root { --color-accent: rgb(9, 105, 218); } h1 { color: var(--color-accent); } p { color: var(--color-accent); }",
    );
    let mut sb = p51_sandbox_with_html(html);
    assert_eq!(p51_gcs(&mut sb, "h1", "color"), "rgb(9, 105, 218)");
    assert_eq!(p51_gcs(&mut sb, "p", "color"), "rgb(9, 105, 218)");
}

#[test]
fn test_p99_inline_style_overrides_external() {
    // 级联序：外部表（collect_stylesheets css 参数）先于内联 <style> 元素——同特异性
    // 同属性后者胜，与布局管线同函数同序（一致性由构造保证）。
    let html = "<html><head><style>h1 { color: rgb(0, 128, 0); }</style></head><body><h1>t</h1></body></html>";
    crate::js_dom_bridge::clear_generation_cache();
    crate::js_dom_bridge::set_active_external_css("h1 { color: rgb(255, 0, 0); }");
    let mut sb = p51_sandbox_with_html(html);
    assert_eq!(p51_gcs(&mut sb, "h1", "color"), "rgb(0, 128, 0)", "内联 <style> 必须压过外部表（同序于布局管线）");
}

#[test]
fn test_p99_css_change_invalidates_generation() {
    // 同 html、css 换代（晚到/更新样式表）：代际键含 css → 旧代表不得 stale 存活。
    let html = "<html><head><style></style></head><body><h1>t</h1></body></html>";
    crate::js_dom_bridge::clear_generation_cache();
    crate::js_dom_bridge::set_active_external_css("h1 { color: rgb(255, 0, 0); }");
    let mut sb = p51_sandbox_with_html(html);
    assert_eq!(p51_gcs(&mut sb, "h1", "color"), "rgb(255, 0, 0)");
    // 快照换代装新 css（worker 臂语义：每次快照无条件覆盖）。
    crate::js_dom_bridge::set_active_external_css("h1 { color: rgb(0, 0, 255); }");
    assert_eq!(p51_gcs(&mut sb, "h1", "color"), "rgb(0, 0, 255)", "css 变化必须换代（重收集 + 全文档重算）");
    // 清空 css（跨文档）：回落纯内联语义——color 无规则 → 初始值 black。
    crate::js_dom_bridge::set_active_external_css("");
    assert_eq!(p51_gcs(&mut sb, "h1", "color"), "rgb(0, 0, 0)", "css 清空后外部规则不得残留（回落初始值）");
}

#[test]
fn test_p99_clear_generation_cache_keeps_css_semantics() {
    // register_dom_callbacks → clear_generation_cache 清的是代际槽；外部 CSS 线程局部
    // 独立存活（快照臂装填先于注册重入），清槽后同 css 查询语义不变。
    let html = "<html><head><style></style></head><body><h1>t</h1></body></html>";
    crate::js_dom_bridge::clear_generation_cache();
    crate::js_dom_bridge::set_active_external_css("h1 { font-weight: 460; }");
    let mut sb = p51_sandbox_with_html(html);
    assert_eq!(p51_gcs(&mut sb, "h1", "font-weight"), "460");
    crate::js_dom_bridge::clear_generation_cache();
    assert_eq!(p51_gcs(&mut sb, "h1", "font-weight"), "460", "清代际槽不影响 css 合成源");
}

#[test]
fn test_p99_pure_path_unaffected_by_ambient_css() {
    // 纯快照路径（compute_document_styles / computed_style_property）不读线程局部：
    // 同线程 ambient css 已装，纯函数仍按「仅内联」语义（WPT/测试逐位不变）。用
    // font-family（h1 无默认值）钉「外部规则未进纯路径级联」。
    let html = "<html><head><style></style></head><body><h1>t</h1></body></html>";
    crate::js_dom_bridge::clear_generation_cache();
    crate::js_dom_bridge::set_active_external_css("h1 { font-family: GithubMono, monospace; }");
    assert_eq!(
        crate::js_dom_bridge::computed_style_property(html, "h1", "font-family"),
        "",
        "纯路径不吃 ambient css"
    );
}

// ── P9-9 切片 2（R5114）：外部样式表进 document.styleSheets + gCS 自定义属性暴露 ──

/// gCS camelCase 属性直读助手（`--` 名免 kebab 归一分支的哨兵面，TV-1）。
fn p51_gcs_camel(sandbox: &mut zero_script_sandbox::V8Sandbox, sel: &str, prop: &str) -> String {
    use zero_script_sandbox::Sandbox;
    let script = format!("getComputedStyle(document.querySelector('{sel}'))['{prop}']");
    sandbox.execute(&script).unwrap().value
}

#[test]
fn test_p99s_gcs_custom_property_from_external_sheet() {
    // :root 自定义属性经外部 CSS 声明 → gCS `--` 查询暴露；继承属性（html/body 同值）。
    let html = "<html><head><style></style></head><body><p>p</p></body></html>";
    crate::js_dom_bridge::clear_generation_cache();
    crate::js_dom_bridge::set_active_external_css(":root { --zw-accent: rgb(9, 105, 218); }");
    let mut sb = p51_sandbox_with_html(html);
    assert_eq!(p51_gcs(&mut sb, "html", "--zw-accent"), "rgb(9, 105, 218)");
    assert_eq!(p51_gcs(&mut sb, "body", "--zw-accent"), "rgb(9, 105, 218)", "自定义属性继承到后代");
    assert_eq!(p51_gcs(&mut sb, "p", "--zw-accent"), "rgb(9, 105, 218)");
    // 未声明属性返 ''（同标准属性 host miss 语义）。
    assert_eq!(p51_gcs(&mut sb, "html", "--zw-missing"), "");
    // css-variables-1：名称 case-sensitive（serializer 不做 lowercase 归一）。
    crate::js_dom_bridge::set_active_external_css(":root { --Zw-X: 1px; --zw-x: 2px; }");
    crate::js_dom_bridge::clear_generation_cache();
    let mut sb = p51_sandbox_with_html(html);
    assert_eq!(p51_gcs(&mut sb, "html", "--Zw-X"), "1px");
    assert_eq!(p51_gcs(&mut sb, "html", "--zw-x"), "2px");
    // TV-1（返修补哨兵）：camelCase 直读与 getPropertyValue 分属两个归一分支，
    // 各自需独立断言（属性访问分支 part01.js 免 kebab 归一）。
    assert_eq!(p51_gcs_camel(&mut sb, "html", "--Zw-X"), "1px");
    assert_eq!(p51_gcs_camel(&mut sb, "html", "--zw-x"), "2px");
}

#[test]
fn test_p99s_gcs_custom_property_invalid_is_empty() {
    // DF-2（R5114 返修）：环引用 / 未定义且无 fallback → css-variables-1 §3.1
    // guaranteed-invalid，gCS 序列化为空串（Chrome 同）。合法链式替换不受影响。
    let html = "<html><head><style></style></head><body></body></html>";
    crate::js_dom_bridge::clear_generation_cache();
    crate::js_dom_bridge::set_active_external_css(
        ":root { --cyc-a: var(--cyc-b); --cyc-b: var(--cyc-a); --undef: var(--zw-missing-var); }",
    );
    let mut sb = p51_sandbox_with_html(html);
    assert_eq!(p51_gcs(&mut sb, "html", "--cyc-a"), "", "环引用 → guaranteed-invalid 空串");
    assert_eq!(p51_gcs(&mut sb, "html", "--undef"), "", "未定义无 fallback → 空串");
    crate::js_dom_bridge::set_active_external_css(":root { --a: var(--b); --b: green; }");
    crate::js_dom_bridge::clear_generation_cache();
    let mut sb = p51_sandbox_with_html(html);
    assert_eq!(p51_gcs(&mut sb, "html", "--a"), "green", "合法 var() 链不受 invalid 语义影响");
}

#[test]
fn test_p99s_external_sheet_entry_is_read_only() {
    // DF-3（R5114 返修）：外部合成条目 insertRule/deleteRule 显式抛
    // NoModificationAllowedError（CSSOM 不可修改样式表语义）——旧实现静默写本次访问的
    // 临时 cache，跨访问即失且从不影响渲染；合成条目置表头后索引位移使页面脚本按旧
    // 习惯的 index 0 写入正好落进只读条目。内联表（ownerNode 非空）写路径不受影响。
    let html = "<html><head><style>body { color: black }</style></head><body></body></html>";
    crate::js_dom_bridge::clear_generation_cache();
    crate::js_dom_bridge::set_active_external_css("h1 { color: red; }");
    let mut sb = p51_sandbox_with_html(html);
    let eval = |sb: &mut zero_script_sandbox::V8Sandbox, js: &str| {
        use zero_script_sandbox::Sandbox;
        sb.execute(js).unwrap().value
    };
    assert_eq!(eval(&mut sb, "document.styleSheets[0].ownerNode === null"), "true");
    assert_eq!(
        eval(&mut sb,
            r#"(()=>{try{document.styleSheets[0].insertRule('p { color: blue }', 0);return 'no-throw';}catch(e){return e.name||String(e);}})()"#),
        "NoModificationAllowedError",
        "外部条目 insertRule 必须显式拒绝"
    );
    assert_eq!(
        eval(&mut sb,
            r#"(()=>{try{document.styleSheets[0].deleteRule(0);return 'no-throw';}catch(e){return e.name||String(e);}})()"#),
        "NoModificationAllowedError",
        "外部条目 deleteRule 必须显式拒绝"
    );
    assert_eq!(eval(&mut sb, "document.styleSheets[1].ownerNode !== null"), "true");
    // 内联表写回（shim client cache 语义）：insertRule 返插入位、cache 同步读回真值。
    // 注意 shim 已知限制③——每次 styleSheets 访问重新查询（live DOM），故写读断言
    // 收敛在同一 sheet 对象引用上（flush 写回 `<style>` 文本的视觉生效路径由 R2809 既有覆盖）。
    assert_eq!(
        eval(&mut sb,
            r#"(()=>{
              const inline = document.styleSheets[1];
              const ret = inline.insertRule('h2 { color: green }', 0);
              return JSON.stringify({ ret, len: inline.cssRules.length,
                first: inline.cssRules[0].selectorText });
            })()"#),
        r#"{"ret":0,"len":2,"first":"h2"}"#,
        "内联表 insertRule 返 index + cache 同步读回"
    );
}

#[test]
fn test_p99s_rule_style_preserves_custom_property_case() {
    // DF-5（R5114 返修）：CSSRule.style 写 `--` 前缀名 case-sensitive 保留原样落 cssText
    //（旧写路径 lowercase 归一使 --Zw-R 落盘 --zw-r，与 gCS 大小写敏感读分裂）；
    // 非 `--` 名维持既有 `_stylePropName` camelCase→kebab + lowercase（R2810 语义，
    // camelCase 直读写由 test_stylesheets_rule_style_r2810 覆盖）。host 侧 inline
    // style 去重键同款语义（style_prop_key）。
    let html = "<html><head><style>h1 { color: black; }</style></head><body><h1>t</h1></body></html>";
    crate::js_dom_bridge::clear_generation_cache();
    crate::js_dom_bridge::set_active_external_css("");
    let mut sb = p51_sandbox_with_html(html);
    let script = r#"
      (() => {
        const rule = document.styleSheets[document.styleSheets.length - 1].cssRules[0];
        rule.style.setProperty('--Zw-R', 'blue');
        rule.style.setProperty('color', 'red');
        rule.style.fontSize = '14px';
        return JSON.stringify({ cssText: rule.cssText, readBack: rule.style.getPropertyValue('--Zw-R'),
          camel: rule.style.fontSize });
      })()
    "#;
    use zero_script_sandbox::Sandbox;
    let v = Sandbox::execute(&mut sb, script).unwrap().value;
    assert_eq!(
        v,
        r#"{"cssText":"h1 { color: red; --Zw-R: blue; font-size: 14px }","readBack":"blue","camel":"14px"}"#,
        "`--` 名 case 保留 + 非 `--` 名 kebab 归一与 camelCase IDL 读写不变"
    );
    // host 侧 style 去重键（style_prop_key）：--Foo/--foo 为独立声明互不覆盖；
    // 常规属性名仍按既有语义去重替换。
    let merged = super::merge_style_property("--Foo: red", "--foo", "blue");
    assert_eq!(merged, "--Foo: red; --foo: blue");
    let merged = super::merge_style_property("color: red", "color", "blue");
    assert_eq!(merged, "color: blue", "常规名维持既有去重语义");
}

#[test]
fn test_p99s_gcs_custom_property_var_chain_resolved() {
    // css-variables-1 §2：计算值 = 指定值经 var() 完整替换（链式 --a: var(--b)）。
    let html = "<html><head><style></style></head><body></body></html>";
    crate::js_dom_bridge::clear_generation_cache();
    crate::js_dom_bridge::set_active_external_css(":root { --a: var(--b); --b: green; }");
    let mut sb = p51_sandbox_with_html(html);
    assert_eq!(p51_gcs(&mut sb, "html", "--a"), "green");
    assert_eq!(p51_gcs(&mut sb, "html", "--b"), "green");
}

#[test]
fn test_p99s_style_sheets_includes_external_entry() {
    // 空外部 CSS：styleSheets 仅内联 <style>（既有语义，逐位不变）。
    let html = "<html><head><style>body { color: black }</style></head><body></body></html>";
    crate::js_dom_bridge::clear_generation_cache();
    crate::js_dom_bridge::set_active_external_css("");
    let mut sb = p51_sandbox_with_html(html);
    let summary = |sb: &mut zero_script_sandbox::V8Sandbox| -> String {
        use zero_script_sandbox::Sandbox;
        sb.execute(
            "(function(){ var ss = document.styleSheets; var out = { len: ss.length }; \
             if (ss.length) { out.first = { sel: ss[0].cssRules.length ? ss[0].cssRules[0].selectorText : null, \
             owner: ss[0].ownerNode === null ? 'null' : 'node' }; } return JSON.stringify(out); })()",
        )
        .unwrap()
        .value
    };
    assert_eq!(summary(&mut sb), r#"{"len":1,"first":{"sel":"body","owner":"node"}}"#);

    // 装外部 CSS：合成条目在表头（级联源同序），规则可读，ownerNode null。
    crate::js_dom_bridge::set_active_external_css("h1 { font-family: GithubMono, monospace; }");
    crate::js_dom_bridge::clear_generation_cache();
    let mut sb = p51_sandbox_with_html(html);
    assert_eq!(
        summary(&mut sb),
        r#"{"len":2,"first":{"sel":"h1","owner":"null"}}"#
    );
}

#[test]
fn test_p99s_pure_path_inline_custom_property() {
    // 纯快照路径（无 ambient css）：内联 <style> 的自定义属性经字段下发后 gCS 可读
    //（字段装配语义与外部 css 无关）；ambient 外部 css 仍不进纯路径（切片 1 语义不变）。
    let html = "<html><head><style>:root { --inline-x: red; }</style></head><body></body></html>";
    crate::js_dom_bridge::clear_generation_cache();
    crate::js_dom_bridge::set_active_external_css(":root { --ext-only: blue; }");
    assert_eq!(
        crate::js_dom_bridge::computed_style_property(html, "html", "--inline-x"),
        "red",
        "内联自定义属性经 ComputedStyle 字段暴露"
    );
    assert_eq!(
        crate::js_dom_bridge::computed_style_property(html, "html", "--ext-only"),
        "",
        "纯路径不吃 ambient css（既有语义不变）"
    );
}

#[test]
fn test_p99s_style_rules_external_memo_recomputes_on_css_change() {
    // TV-2（R5114 返修二轮补测）：__zw_style_rules_external 单槽记忆化的「css 换代键失配
    // 即重算」路径——同一沙箱先装非空 css A 读 cssRules，再换非空 css B 重读：wire 必须按
    // B 重算（非 stale 命中）；内容路径（同 css 命中）由既有 test_p99s_style_sheets_* 覆盖。
    let html = "<html><head><style></style></head><body></body></html>";
    crate::js_dom_bridge::clear_generation_cache();
    crate::js_dom_bridge::set_active_external_css("h1 { color: red; }");
    let mut sb = p51_sandbox_with_html(html);
    let eval = |sb: &mut zero_script_sandbox::V8Sandbox, js: &str| {
        use zero_script_sandbox::Sandbox;
        sb.execute(js).unwrap().value
    };
    assert_eq!(
        eval(&mut sb, "document.styleSheets[0].cssRules[0].selectorText"),
        "h1",
        "css A 的 wire 命中"
    );
    crate::js_dom_bridge::set_active_external_css("h2 { color: blue; }");
    assert_eq!(
        eval(&mut sb, "document.styleSheets[0].cssRules[0].selectorText"),
        "h2",
        "css 换代（非空 A → 非空 B）键失配必须重算 wire，不得 stale 命中"
    );
    assert_eq!(eval(&mut sb, "document.styleSheets[0].cssRules.length"), "1", "重算后规则数按 B");
}
