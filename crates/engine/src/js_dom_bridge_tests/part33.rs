// slice28（site-compat Window named access name 属性面，RP-1）：
// `collect_element_ids_doc` 的 name 面钉。spec「Named access on the Window object」
// 的 supported property names = ①文档树内所有带 id 元素的 id 值 + ②embed/form/
// img/object 四元素的非空 name 内容属性值——单遍 tree order 合并、忽略后现重复。
// iframe 属「child navigable target name」源，委托 R139 `__zwRegisterNamedIframes`
// （contentWindow 值更贴 spec；本面收 iframe 会以元素先占名压制 R139）——不收。
// 修前 `collect_element_ids_doc` 仅收 `[id]`（name 面整体缺失——缺陷轮 S1）。
// https://html.spec.whatwg.org/multipage/window-object.html#named-access-on-the-window-object

#[test]
fn test_collect_name_face_elements_tree_order_s28() {
    // 四 name-able 元素 + id 面同页：单遍树序合并（form 无 id → 仅 name 面；
    // img id+name 并存 → 两个名字都贡献；embed/object 仅 name 面）。iframe name=
    // 入 fixture 作负面：不收（R139 委托面，收了会压制 contentWindow 注册）。
    let html = "<html><body>\
                <form name=\"f1\"></form>\
                <img name=\"i1\" id=\"ii\">\
                <iframe name=\"fr\"></iframe>\
                <embed name=\"e1\"></embed>\
                <object name=\"o1\"></object>\
                </body></html>";
    assert_eq!(
        collect_element_ids(html),
        "f1|ii|i1|e1|o1",
        "name 面四元素与 id 面单遍树序合并收集；iframe 不入本面（R139 委托）"
    );
}

#[test]
fn test_collect_name_face_id_name_same_value_dedup_s28() {
    // 同元素 id=name 同值：spec「ignoring later duplicates」→ 只出现一次。
    let html = "<html><body><img name=\"dupe\" id=\"dupe\"></body></html>";
    assert_eq!(collect_element_ids(html), "dupe", "id=name 同值去重单现");
}

#[test]
fn test_collect_name_face_non_nameable_tags_s28() {
    // spec 负面：仅五元素参与 name 面——input/a 等带 name 不收集（input name 是
    // 表单控件名，a name 是锚点名，均非 named access 面）。
    let html = "<html><body>\
                <input name=\"q\">\
                <a name=\"anc\"></a>\
                <textarea name=\"ta\"></textarea>\
                <select name=\"sel\"></select>\
                </body></html>";
    assert_eq!(
        collect_element_ids(html),
        "",
        "非 name-able 元素（input/a/textarea/select）不参与 name 面"
    );
}

#[test]
fn test_collect_name_face_empty_name_skipped_s28() {
    // spec「non-empty name content attribute」：空 name 不收集；无 id 元素不进 id 面。
    let html = "<html><body><img name=\"\"><form name=\"ok\"></form></body></html>";
    assert_eq!(collect_element_ids(html), "ok", "空 name 跳过、非空照收");
}

#[test]
fn test_collect_name_face_order_id_only_regression_s28() {
    // id 面回归守卫：仅 id 元素的收集结果与修前逐字节一致（RP-1 不改 id 面语义）。
    let html = "<html><body>\
                <div id=\"container\"></div>\
                <span id=\"target\"></span>\
                <img name=\"nm\" id=\"mix\">\
                </body></html>";
    assert_eq!(
        collect_element_ids(html),
        "container|target|mix|nm",
        "id 面保序不回归 + name 面追加树序位"
    );
}
