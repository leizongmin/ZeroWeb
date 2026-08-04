use super::*;

#[test]
fn test_apply_set_attr_src() {
    let html = "<html><body><img id=\"i\" src=\"old.png\"></body></html>";
    let mutations = vec![DomMutation::SetAttr {
        selector: "#i".into(),
        name: "src".into(),
        value: "new.png".into(),
    }];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    assert!(out.contains("src=\"new.png\""));
}

#[test]
fn test_apply_set_style() {
    let html = "<html><body><div id=\"d\"></div></body></html>";
    let mutations = vec![DomMutation::SetStyle {
        selector: "#d".into(),
        property: "color".into(),
        value: "red".into(),
    }];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    assert!(out.contains("color: red") || out.contains("color:red"));
}

#[test]
fn test_apply_class_name_via_set_attr() {
    let html = "<html><body><div id=\"d\"></div></body></html>";
    let mutations = vec![DomMutation::SetAttr {
        selector: "#d".into(),
        name: "class".into(),
        value: "active".into(),
    }];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    assert!(out.contains("class=\"active\""));
}

#[test]
fn test_apply_create_and_append() {
    let html = "<html><body id=\"b\"></body></html>";
    let mutations = vec![
        DomMutation::CreateElement {
            handle: "__n1".into(),
            tag: "p".into(),
        },
        DomMutation::SetAttrOnHandle {
            handle: "__n1".into(),
            name: "id".into(),
            value: "p1".into(),
        },
        DomMutation::CreateTextNode {
            handle: "__n2".into(),
            text: "hello".into(),
        },
        DomMutation::AppendChild {
            parent_selector: "#b".into(),
            child_handle: "__n1".into(),
        },
        DomMutation::AppendChild {
            parent_selector: "#p1".into(),
            child_handle: "__n2".into(),
        },
    ];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    assert!(out.contains("<p id=\"p1\">hello</p>"));
}

#[test]
fn test_apply_with_handles_maps_unique_selectors() {
    // 创建带 id 的元素（唯一）→ 入 map；创建无 id 的同 tag 元素 + 文档已有同 tag → 歧义 → 不入 map。
    let html = "<html><body id=\"b\"><div></div></body></html>";
    let mutations = vec![
        // __n1: <p id="unique"> —— id 唯一 → "#unique"
        DomMutation::CreateElement {
            handle: "__n1".into(),
            tag: "p".into(),
        },
        DomMutation::SetAttrOnHandle {
            handle: "__n1".into(),
            name: "id".into(),
            value: "unique".into(),
        },
        DomMutation::AppendChild {
            parent_selector: "#b".into(),
            child_handle: "__n1".into(),
        },
        // __n2: <div>（无 id）—— 文档已有 1 个 div，加这个共 2 个 → "div" 歧义 → 不入 map
        DomMutation::CreateElement {
            handle: "__n2".into(),
            tag: "div".into(),
        },
        DomMutation::AppendChild {
            parent_selector: "#b".into(),
            child_handle: "__n2".into(),
        },
        // __n3: <span>（无 id）—— 文档无其他 span → "span" 唯一 → 入 map
        DomMutation::CreateElement {
            handle: "__n3".into(),
            tag: "span".into(),
        },
        DomMutation::AppendChild {
            parent_selector: "#b".into(),
            child_handle: "__n3".into(),
        },
    ];
    let (out, handles) = apply_mutations_to_html_with_handles(html, &mutations).unwrap();
    // 序列化正确
    assert!(out.contains("<p id=\"unique\">"));
    assert!(out.contains("<span>"));
    // handle→selector 映射：唯一者用短选择器，歧义者回落 nth-child 结构路径。
    assert_eq!(handles.get("__n1"), Some(&"#unique".to_string()), "id 唯一 → #unique");
    assert_eq!(handles.get("__n3"), Some(&"span".to_string()), "唯一 span → tag");
    let n2_sel = handles.get("__n2").expect("歧义 div 仍入 map（结构路径）");
    assert!(
        n2_sel.contains("nth-child"),
        "无 id 的歧义 div → nth-child 结构路径，got: {n2_sel}"
    );
    // 不变量：每个 selector 在 fresh-parse(序列化 html) 上都能解析（path A 依赖此）。
    let fresh = parse_html(&out);
    for sel in handles.values() {
        assert!(
            find_by_selector(&fresh, sel).is_some(),
            "selector {sel} 须在 fresh-parse 后可解析"
        );
    }
}

#[test]
fn test_structural_path_resolves_to_correct_node() {
    // 多个无 id/class 同 tag 元素：nth-child 结构路径唯一锁定每一个，且 round-trip 正确
    // （在 mutated doc 与 fresh-parse 序列化 html 上都解析到同一节点）。
    let html = "<html><body><ul><li>1</li><li>2</li><li>3</li></ul></body></html>";
    let mutations = vec![
        DomMutation::CreateElement {
            handle: "__n9".into(),
            tag: "li".into(),
        },
        DomMutation::AppendChild {
            parent_selector: "ul".into(),
            child_handle: "__n9".into(),
        },
    ];
    let (out, handles) = apply_mutations_to_html_with_handles(html, &mutations).unwrap();
    let sel = handles.get("__n9").expect("__n9 入 map");
    assert!(sel.contains("nth-child"), "应回落结构路径: {sel}");
    // 在 fresh-parse(out) 上解析 → 应是第 4 个 li（"4" 无文本，但节点存在且唯一）。
    let fresh = parse_html(&out);
    let resolved = find_by_selector(&fresh, sel).expect("结构路径须可解析");
    // resolved 应是 ul 的最后一个 li（第 4 个）。
    let all_li = fresh.query_selector_all(fresh.root(), "li");
    assert_eq!(all_li.last().copied(), Some(resolved), "结构路径解析到 append 的末 li");
}

#[test]
fn test_apply_with_handles_uses_post_mutation_state() {
    // id 在 CreateElement 之后、AppendChild 之前设置（同 batch）→ 末尾算选择器时 id 已生效。
    let html = "<html><body id=\"b\"></body></html>";
    let mutations = vec![
        DomMutation::CreateElement {
            handle: "__n1".into(),
            tag: "div".into(),
        },
        DomMutation::SetAttrOnHandle {
            handle: "__n1".into(),
            name: "id".into(),
            value: "late".into(),
        },
        DomMutation::AppendChild {
            parent_selector: "#b".into(),
            child_handle: "__n1".into(),
        },
    ];
    let (_out, handles) = apply_mutations_to_html_with_handles(html, &mutations).unwrap();
    assert_eq!(handles.get("__n1"), Some(&"#late".to_string()));
}

#[test]
fn test_find_all_selectors() {
    let html = "<html><body><p class=\"x\"></p><p class=\"x\"></p></body></html>";
    let doc = parse_html(html);
    let sels = find_all_selectors(&doc, "p.x");
    assert_eq!(sels.len(), 2);
}

#[test]
fn test_query_all_selector_list_unique() {
    // querySelectorAll 对歧义集合（无 id/class 的 option）每元素返**唯一**选择器（nth-child
    // 结构路径），而非 stable_selector（"option" 重复→全指向首个）。各 selector 互异且可解析。
    let html = "<html><body><select id='s'>\
                    <option value='a'>A</option>\
                    <option value='b' selected>B</option>\
                    <option value='c'>C</option>\
                    </select></body></html>";
    let list = query_all_selector_list(html, "#s option");
    let sels: Vec<&str> = list.split('|').collect();
    assert_eq!(sels.len(), 3, "应返回 3 个 option 的选择器");
    // 互异（唯一化后无重复）。
    let uniq: std::collections::HashSet<&str> = sels.iter().copied().collect();
    assert_eq!(uniq.len(), 3, "3 个 selector 应互异（非全 'option'）");
    // 每个 selector 在 fresh-parse 上可解析且指向不同 option。
    let doc = parse_html(html);
    let mut resolved = Vec::new();
    for s in &sels {
        let id = find_by_selector(&doc, s).expect("每个 selector 须可解析");
        resolved.push(option_value(&doc, id));
    }
    resolved.sort();
    assert_eq!(resolved, vec!["a".to_string(), "b".to_string(), "c".to_string()]);
}

#[test]
fn test_element_matches_test_selector() {
    let html = "<html><body>\
                    <div id='outer' class='row'><div id='inner' class='cell active'>x</div></div>\
                    <span id='sib'>y</span>\
                    </body></html>";
    // 自身选择器 → find_by_selector 解析元素；test_sel 含组合器/伪类/属性运算符均支持。
    assert!(element_matches_test_selector(html, "#inner", ".cell.active"));
    assert!(element_matches_test_selector(html, "#inner", "div"));
    // 组合器：#inner 是 #outer > .cell。
    assert!(element_matches_test_selector(html, "#inner", "#outer > .cell"));
    // :has（R2669）+ 属性运算符（R2671）经 query_selector_all 全匹配集正确判定。
    assert!(element_matches_test_selector(html, "#outer", "div:has(.active)"));
    assert!(element_matches_test_selector(html, "#inner", "[class^=cell]"));
    // 不匹配。
    assert!(!element_matches_test_selector(html, "#inner", "span"));
    assert!(!element_matches_test_selector(html, "#sib", ".cell"));
    assert!(!element_matches_test_selector(html, "#inner", "#outer > span"));
    // elem_sel 不存在 → false。
    assert!(!element_matches_test_selector(html, "#nope", "div"));
}

#[test]
fn test_closest_matching_selector() {
    let html = "<html><body>\
                    <div id='outer' class='row'><section id='mid'><div id='inner' class='cell'>x</div></section></div>\
                    </body></html>";
    // 自身匹配 → 返自身唯一选择器。
    assert_eq!(closest_matching_selector(html, "#inner", ".cell"), "#inner");
    // 向上找最近祖先：#inner 的最近 .row 祖先 = #outer（跨 section）。
    assert_eq!(closest_matching_selector(html, "#inner", ".row"), "#outer");
    // 含组合器：找匹配 `section .cell` 的最近祖先（#inner 自身匹配）。
    assert_eq!(closest_matching_selector(html, "#inner", "section .cell"), "#inner");
    // 标签祖先：#inner 最近 section 祖先 = #mid。
    assert_eq!(closest_matching_selector(html, "#inner", "section"), "#mid");
    // body 也可作为 closest 目标。
    assert_eq!(closest_matching_selector(html, "#inner", "body"), "body");
    // 无匹配 → 空串。
    assert_eq!(closest_matching_selector(html, "#inner", ".nonexistent"), "");
    assert_eq!(closest_matching_selector(html, "#inner", "span"), "");
    // elem_sel 不存在 → 空串。
    assert_eq!(closest_matching_selector(html, "#nope", "div"), "");
}

#[test]
fn test_query_in_subtree_scoping() {
    // 两个容器各含 .item；container 之外也有 .item。元素子树作用域须仅返该容器后代。
    let html = "<html><body>\
                <div id='a'><span class='item'>a1</span><span class='item'>a2</span></div>\
                <div id='b'><span class='item'>b1</span></div>\
                <span class='item'>outside</span>\
                </body></html>";
    // query_match_in_subtree：#a 子树首个 .item = a1（不返 outside 或 b1）。
    let a_first = query_match_in_subtree(html, "#a", ".item");
    let doc = parse_html(html);
    let n = find_by_selector(&doc, &a_first).expect("须可解析");
    assert_eq!(doc.text_content(n), Some("a1".to_string()));
    // #b 子树首个 .item = b1。
    let b_first = query_match_in_subtree(html, "#b", ".item");
    let nb = find_by_selector(&doc, &b_first).expect("须可解析");
    assert_eq!(doc.text_content(nb), Some("b1".to_string()));
    // query_all_in_subtree：#a 子树全部 .item = a1,a2（2 个，不含 outside/b1）。
    let a_all = query_all_in_subtree(html, "#a", ".item");
    let sels: Vec<&str> = a_all.split('|').collect();
    assert_eq!(sels.len(), 2, "#a 子树应有 2 个 .item（不含外部）");
    let texts: Vec<String> = sels
        .iter()
        .map(|s| doc.text_content(find_by_selector(&doc, s).unwrap()).unwrap())
        .collect();
    assert_eq!(texts, vec!["a1".to_string(), "a2".to_string()]);
    // 子树无匹配 → 空串（#a 内无 .nonexistent）。
    assert_eq!(query_match_in_subtree(html, "#a", ".nonexistent"), "");
    assert_eq!(query_all_in_subtree(html, "#a", ".nonexistent"), "");
    // 含组合器：#a 子树内 `span.item` 仍命中（后代组合器在子树内求值）。
    assert!(!query_all_in_subtree(html, "#a", "span.item").is_empty());
    // elem_sel 不存在 → 空串。
    assert_eq!(query_match_in_subtree(html, "#nope", ".item"), "");
    assert_eq!(query_all_in_subtree(html, "#nope", ".item"), "");
}

#[test]
fn test_collect_element_ids_dedup_preserve_order() {
    let html = "<html><body>\
                    <div id=\"container\"></div>\
                    <span id=\"target\"></span>\
                    <p id=\"container\"></p>\
                    <b></div>\
                    </body></html>";
    let ids = collect_element_ids(html);
    // 去重（首个 container 保留），保序，跳过无 id 元素。
    assert_eq!(ids, "container|target");
}

#[test]
fn test_collect_element_ids_empty() {
    let html = "<html><body><div></div><p class=\"x\"></p></body></html>";
    assert_eq!(collect_element_ids(html), "");
}

#[test]
fn test_apply_inner_html() {
    let html = "<html><body><div id=\"d\">old</div></body></html>";
    let mutations = vec![DomMutation::SetInnerHtml {
        selector: "#d".into(),
        html: "<b>new</b>".into(),
    }];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    assert!(out.contains("<b>new</b>"));
}

#[test]
fn test_shim_not_empty() {
    assert!(generate_js_dom_shim().contains("__zw_set_attr"));
    assert!(generate_js_dom_shim().contains("addEventListener"));
}

#[test]
fn test_shim_async_resolve_callback_e2e() {
    // P1b S1（方案 A）端到端：注入**生产** DOM shim（含 __zwResolveCallback + pending 表），
    // 验证 V8Sandbox::resolve_async_callback 经 shim 的 JS 契约真实 resolve Promise。
    // 宿主回调同步返「回调 ID」，JS 建 pending Promise；Rust resolve 触发 .then。
    use zero_script_sandbox::{Sandbox, SandboxConfig, V8Sandbox};
    let config = SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    // 注入生产 shim（tab_js_worker.rs / js_worker.rs 同款）。
    sandbox.execute(generate_js_dom_shim()).unwrap();
    sandbox.register_callback("__zw_start_async", Box::new(|args| format!("aid:{}", args[0])));
    sandbox
        .execute(
            "var id = __zw_start_async('99');
                 new Promise(function(resolve){ globalThis.__zw_pending[id] = resolve; })
                     .then(function(v){ globalThis.__result = v; });",
        )
        .unwrap();
    // resolve 前：Promise pending。
    let before = sandbox.execute("typeof globalThis.__result").unwrap();
    assert_eq!(before.value, "undefined");
    // Rust 异步完成 → resolve（shim 的 __zwResolveCallback 触发 + microtask drain）。
    sandbox.resolve_async_callback("aid:99", "resolved!");
    let after = sandbox.execute("globalThis.__result").unwrap();
    assert_eq!(after.value, "resolved!");
}

#[test]
fn test_shim_includes_runtime_stubs() {
    let shim = generate_js_dom_shim();
    assert!(shim.contains("globalThis.setTimeout"));
    assert!(shim.contains("globalThis.navigator"));
    assert!(shim.contains("attachEvent"));
    assert!(shim.contains("__zw_get_page_url"));
    assert!(shim.contains("globalThis.screen"));
    assert!(shim.contains("parentNode"));
    // P1b S1（方案 A）异步回调 resolve 通道 JS 侧契约。
    assert!(shim.contains("globalThis.__zwResolveCallback"));
    assert!(shim.contains("globalThis.__zw_pending"));
    // P1a select：<select>.value/selectedIndex getter + setter 经 host 回调。
    assert!(shim.contains("__zw_select_value"));
    assert!(shim.contains("__zw_select_index"));
    assert!(shim.contains("__zw_select_option"));
}

#[test]
fn test_shim_includes_modern_reftest_stubs() {
    // 现代动态 reftest 的 `requestAnimationFrame(() => …; takeScreenshot())` 模式
    // 要求这两个全局存在，否则 setup mutation 永不执行（R917 未捕获的 yield gap）。
    let shim = generate_js_dom_shim();
    assert!(shim.contains("globalThis.requestAnimationFrame"));
    assert!(shim.contains("globalThis.cancelAnimationFrame"));
    assert!(shim.contains("globalThis.takeScreenshot"));
    // `Element.append(...nodesOrStrings)` 现代 API（区别于 appendChild）。
    assert!(shim.contains("if (prop === 'append')"));
    // `getBoundingClientRect()` 方法必须返回零 DOMRect，否则调用抛 TypeError
    // 中断脚本，使其后的 mutation 丢失（120 reftest 文件用作 reflow 触发器）。
    assert!(shim.contains("if (prop === 'getBoundingClientRect')"));
    // HTML 规范 named access on window（`id="x"` → 全局 `x`，257 reftest 文件）。
    assert!(shim.contains("_installNamedAccess"));
    assert!(shim.contains("__zw_collect_ids"));
    // `createElementNS`（XHTML 命名空间 alias createElement；SVG OOS 不渲染但不中断）。
    assert!(shim.contains("createElementNS:"));
    // `getComputedStyle`：动态 reftest 常作「强制 reflow」触发器调用，缺失则抛
    // ReferenceError 中断脚本丢失后续 mutation。返空 CSSStyleDeclaration 桩不抛。
    assert!(shim.contains("globalThis.getComputedStyle"));
    assert!(shim.contains("getPropertyValue"));
}

#[test]
fn test_merge_style_property() {
    let merged = merge_style_property("color: blue", "width", "10px");
    assert!(merged.contains("color: blue"));
    assert!(merged.contains("width: 10px"));
    let replaced = merge_style_property(&merged, "color", "red");
    assert!(!replaced.contains("blue"));
    assert!(replaced.contains("color: red"));
}

#[test]
fn test_enclosing_form_selector() {
    // P1a form submit：input 在 form 内 → 返 form 的 stable selector。
    let html = "<html><body><form id='f'><input id='i'></form></body></html>";
    assert_eq!(enclosing_form_selector(html, "#i").as_deref(), Some("#f"));
    // input 无 enclosing form → None。
    let no_form = "<html><body><div><input id='i'></div></body></html>";
    assert_eq!(enclosing_form_selector(no_form, "#i"), None);
    // 嵌套：input 在 form 内的 div 内 → 仍解析到 form。
    let nested = "<html><body><form id='outer'><div><input id='deep'></div></form></body></html>";
    assert_eq!(enclosing_form_selector(nested, "#deep").as_deref(), Some("#outer"));
    // 未命中 selector → None。
    assert_eq!(enclosing_form_selector(html, "#missing"), None);
}

#[test]
fn test_is_submit_button() {
    // P1a form submit：submit-button 判定。
    assert!(is_submit_button(
        "<html><body><form><input id='b' type='submit'></form></body></html>",
        "#b",
    ));
    assert!(is_submit_button(
        "<html><body><form><input id='i' type='image'></form></body></html>",
        "#i",
    ));
    // button 默认 type=submit → 提交。
    assert!(is_submit_button(
        "<html><body><form><button id='btn'>Go</button></form></body></html>",
        "#btn",
    ));
    assert!(is_submit_button(
        "<html><body><form><button id='s' type='submit'>Go</button></form></body></html>",
        "#s",
    ));
    // 非提交：
    assert!(!is_submit_button(
        "<html><body><form><input id='t' type='text'></form></body></html>",
        "#t",
    ));
    assert!(!is_submit_button(
        "<html><body><form><button id='nb' type='button'>Go</button></form></body></html>",
        "#nb",
    ));
    assert!(!is_submit_button(
        "<html><body><form><div id='d'>x</div></form></body></html>",
        "#d",
    ));
}

#[test]
fn test_remove_attr_and_has_attribute() {
    // P1a checkbox：RemoveAttr 真正移除属性；has_attribute 判存在性。
    let html = "<html><body><input id='c' type='checkbox' checked></body></html>";
    assert!(has_attribute(html, "#c", "checked"));
    let out = apply_mutations_to_html(
        html,
        &[DomMutation::RemoveAttr {
            selector: "#c".into(),
            name: "checked".into(),
        }],
    )
    .unwrap();
    assert!(!out.contains("checked"));
    assert!(!has_attribute(&out, "#c", "checked"));
    // 无该属性 → has_attribute false。
    assert!(!has_attribute(
        "<html><body><input id='n' type='checkbox'></body></html>",
        "#n",
        "checked",
    ));
}

#[test]
fn test_is_checkbox() {
    assert!(is_checkbox(
        "<html><body><input id='c' type='checkbox'></body></html>",
        "#c",
    ));
    assert!(!is_checkbox(
        "<html><body><input id='t' type='text'></body></html>",
        "#t",
    ));
    assert!(!is_checkbox("<html><body><div id='d'></div></body></html>", "#d",));
}

#[test]
fn test_toggle_radio_html() {
    // P1a radio：toggle target → set checked + 同 name 组兄弟 unset（直接 doc 操作）。
    let html = "<html><body><form>\
            <input id='a' type='radio' name='g' checked>\
            <input id='b' type='radio' name='g'>\
            <input id='c' type='checkbox' checked>\
            </form></body></html>";
    // toggle #b → #b checked、#a unchecked（同 name 组）；#c checkbox 不受影响。
    let out = toggle_radio_html(html, "#b").unwrap();
    assert!(has_attribute(&out, "#b", "checked"));
    assert!(!has_attribute(&out, "#a", "checked"));
    assert!(has_attribute(&out, "#c", "checked"));
    // 非 radio → None。
    assert_eq!(toggle_radio_html(html, "#c"), None);
}

#[test]
fn test_select_value_read() {
    let html = "<html><body><select id='s'>\
            <option value='a'>A</option>\
            <option value='b' selected>B</option>\
            <option value='c'>C</option>\
            </select></body></html>";
    assert!(is_select(html, "#s"));
    // selected option b → "b"。
    assert_eq!(select_value_from_html(html, "#s"), "b");
    assert_eq!(select_index_from_html(html, "#s"), 1);
    // 无 selected 属性 → 默认首个 option。
    let html2 =
        "<html><body><select id='s'><option value='x'>X</option><option value='y'>Y</option></select></body></html>";
    assert_eq!(select_value_from_html(html2, "#s"), "x");
    assert_eq!(select_index_from_html(html2, "#s"), 0);
    // option 无 value 属性 → text content。
    let html3 = "<html><body><select id='s'><option>Plain</option></select></body></html>";
    assert_eq!(select_value_from_html(html3, "#s"), "Plain");
    // 无 option → 空串 / -1。
    let html4 = "<html><body><select id='s'></select></body></html>";
    assert_eq!(select_value_from_html(html4, "#s"), "");
    assert_eq!(select_index_from_html(html4, "#s"), -1);
}

#[test]
fn test_set_selected_option_html() {
    let html = "<html><body><select id='s'>\
            <option value='a' selected>A</option>\
            <option value='b'>B</option>\
            <option value='c'>C</option>\
            </select></body></html>";
    // 设 value='c' → c selected、a/b deselect。
    let out = set_selected_option_html(html, "#s", "c").unwrap();
    assert!(has_attribute(&out, "#s > option:nth-of-type(3)", "selected") || out.contains("value=\"c\" selected"));
    assert!(!has_attribute(&out, "#s > option:nth-of-type(1)", "selected"));
    // 经 value 读回 = "c"。
    assert_eq!(select_value_from_html(&out, "#s"), "c");
    // 匹配 option 无 value 属性（按 text content）。
    let html2 = "<html><body><select id='s'><option>One</option><option>Two</option></select></body></html>";
    let out2 = set_selected_option_html(html2, "#s", "Two").unwrap();
    assert_eq!(select_value_from_html(&out2, "#s"), "Two");
    // 无匹配 value → None（不改）。
    assert_eq!(set_selected_option_html(html, "#s", "zzz"), None);
    // 非 select → None。
    assert_eq!(set_selected_option_html(html, "body", "a"), None);
}

#[test]
fn test_apply_select_option_mutation() {
    let html = "<html><body><select id='s'>\
            <option value='a' selected>A</option>\
            <option value='b'>B</option>\
            </select></body></html>";
    let out = apply_mutations_to_html(
        html,
        &[DomMutation::SelectOption {
            selector: "#s".into(),
            value: "b".into(),
        }],
    )
    .unwrap();
    assert_eq!(select_value_from_html(&out, "#s"), "b");
    // SelectOption 也参与 handle→selector map（apply_dom_mutations 末尾无新 handle，map 空）。
    let (_, handles) = apply_mutations_to_html_with_handles(
        html,
        &[DomMutation::SelectOption {
            selector: "#s".into(),
            value: "b".into(),
        }],
    )
    .unwrap();
    assert!(handles.is_empty(), "SelectOption 不创建 handle");
}

#[test]
fn test_is_text_input() {
    // P1a change-on-blur：文本输入判定（textarea + input 文本类；排除 action 类型）。
    assert!(is_text_input(
        "<html><body><input id='t' type='text'></body></html>",
        "#t",
    ));
    assert!(is_text_input(
        "<html><body><input id='e' type='email'></body></html>",
        "#e",
    ));
    assert!(is_text_input(
        "<html><body><textarea id='ta'></textarea></body></html>",
        "#ta",
    ));
    // input 无 type → 默认 text。
    assert!(is_text_input("<html><body><input id='n'></body></html>", "#n"));
    // action 类型排除（change 在 click 派发）。
    assert!(!is_text_input(
        "<html><body><input id='cb' type='checkbox'></body></html>",
        "#cb",
    ));
    assert!(!is_text_input(
        "<html><body><input id='s' type='submit'></body></html>",
        "#s",
    ));
    assert!(!is_text_input("<html><body><div id='d'></div></body></html>", "#d",));
}

#[test]
fn test_next_focus_selector() {
    // P1a Tab 焦点导航：tabindex>0 升序在前（d=1, c=2），0/默认文档序在后（a, b）→ [d,c,a,b]。
    let html = "<html><body>\
            <input id='a'>\
            <button id='b'>x</button>\
            <input id='c' tabindex='2'>\
            <input id='d' tabindex='1'>\
            </body></html>";
    // 无 current → first = d（tabindex=1）。
    assert_eq!(next_focus_selector(html, None, true).as_deref(), Some("#d"));
    // current=d → c（tabindex=2）。
    assert_eq!(next_focus_selector(html, Some("#d"), true).as_deref(), Some("#c"));
    // current=c → a（文档序 tabindex=0/default）。
    assert_eq!(next_focus_selector(html, Some("#c"), true).as_deref(), Some("#a"));
    // backward：current=a → prev=c。
    assert_eq!(next_focus_selector(html, Some("#a"), false).as_deref(), Some("#c"));
    // 无 focusable → None。
    assert_eq!(
        next_focus_selector("<html><body><div>no focusable</div></body></html>", None, true),
        None
    );
}
