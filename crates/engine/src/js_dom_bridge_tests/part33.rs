// slice28（site-compat Window named access name 属性面，RP-1）：
// `collect_element_ids_doc` 的 name 面钉。spec「Named access on the Window object」
// 的 supported property names = ①文档树内所有带 id 元素的 id 值 + ②embed/form/
// img/object 四元素的非空 name 内容属性值——单遍 tree order 合并、忽略后现重复。
// iframe 属「child navigable target name」源，委托 R139 `__zwRegisterNamedIframes`
// （contentWindow 值更贴 spec；本面收 iframe 会以元素先占名压制 R139）——不收。
// 修前 `collect_element_ids_doc` 仅收 `[id]`（name 面整体缺失——缺陷轮 S1）。
// slice33 缺陷轮 B-1：I-1 曾误收 iframe 入面（「现行 spec named objects 含
// HTMLIFrameElement」为错误规范断言——spec name 面供名元素逐字仅 embed/form/
// img/object），本文件回退恢复 slice32 口径。
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
    // spec 负面：仅四元素参与 name 面——input/a 等带 name 不收集（input name 是
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

// slice30（RP-1 同名多命中 HTMLCollection 面，2026-10-04）：`collect_element_ids_multi`
// 钉——「named object 命中 ≥2 的名字」清单（树序首现、去重）。spec WindowProperties
// 命名属性取值：唯一 named object 返元素，多命中返以文档为根、树序全集的 HTMLCollection；
// shim `_installNamedAccess` 据本清单安装集合（单命中名走元素路径）。named object 以
// 元素计——同元素 id/name 双臂同值只算一个。
// https://html.spec.whatwg.org/multipage/window-object.html#named-access-on-the-window-object
// https://webidl.spec.whatwg.org/#WindowProperties

#[test]
fn test_collect_multi_match_id_face_tree_order_s30() {
    // id 面多命中：双 div 同 id + 混合面（img name=dup ×2 + div id=dup）——清单按
    // 树序首现、去重；单命中名不入选。
    let html = "<html><body>\
                <div id=\"dupe\"></div>\
                <img name=\"mm\">\
                <div id=\"dupe\"></div>\
                <div id=\"dupe2\"></div>\
                <img name=\"mm\">\
                <div id=\"dupe2\"></div>\
                <div id=\"dupe2\"></div>\
                <div id=\"solo\"></div>\
                </body></html>";
    assert_eq!(
        collect_element_ids_multi(html),
        "dupe|mm|dupe2",
        "id 面多命中 + name 面多命中按树序首现；单命中名（solo）不入选"
    );
}

#[test]
fn test_collect_multi_match_same_element_id_name_once_s30() {
    // 同元素 id=name 同值 + 另一同名元素：该元素对同名只算一个 named object——
    // 两者合计命中 2 → 入选；单元素自身同值不重复计。
    let html = "<html><body>\
                <img id=\"both\" name=\"both\">\
                <div id=\"both\"></div>\
                <form name=\"solo\"></form>\
                </body></html>";
    assert_eq!(
        collect_element_ids_multi(html),
        "both",
        "同元素 id/name 同值计一个 named object；两元素同名入选；单命中名不入选"
    );
}

#[test]
fn test_collect_multi_match_negative_non_nameable_and_single_s30() {
    // 负面：input/a 等非 name-able 元素同名多命中不入清单（name 面仅四元素参与，
    // 同 collect 口径）；全部单命中名不入选；iframe name= 不入（R139 委托）。
    let html = "<html><body>\
                <input name=\"q\"><input name=\"q\">\
                <a name=\"anc\"></a><a name=\"anc\"></a>\
                <iframe name=\"fr\"></iframe><iframe name=\"fr\"></iframe>\
                <img name=\"one\">\
                </body></html>";
    assert_eq!(
        collect_element_ids_multi(html),
        "",
        "非 name-able 多命中（input/a/iframe）不入选；全单命中不入选"
    );
}
