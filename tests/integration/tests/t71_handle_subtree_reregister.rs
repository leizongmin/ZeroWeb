//! T71（G1 Gap A 根修，t69 诊断 §4）：handle 族子树跨批换代（slotmap 全量 doc
//! 重建）后子树整体再注册语义钉测。旧实现 `end_script_batch` 的换代重锚只覆盖
//! 本批 apply 产出的 handle→selector 累积——flush 路径（`apply_pending_shared_
//! mutations`）有 sel_map 反演转发兜底，renderer 异步 drain 路径（`begin_script_
//! batch` → `apply_dom_mutations_and_render` × N → `end_script_batch`）没有：
//! 上一批注册的 handle 族子树在新 doc 上失锚，跨批 handle 写（SetAttrOnHandle
//! 等）在管线表 miss 被 lenient skip、跨批 InsertBefore ref 失锚降级（T69 掘金
//! 40s 窗 509 stale 事件根因）。规范语义：DOM §4.2.2 concept-node-pre-insert
//! adopted-children——节点插入宿主子树后身份随子树存活，换代（引擎内部全量
//! 重解析）不得丢失其锚（https://dom.spec.whatwg.org/#concept-node-pre-insert、
//! §4.4 ParentNode）。
//!
//! 驱动形 = 生产 drain 路径的手工等价：`begin_script_batch` + wire mutations
//! `apply_dom_mutations_and_render` + `end_script_batch`（同 webview s34/s35
//! 测试形制；绕过 shim，直接钉管线注册表语义）。
//!
//! 红先证（0965ead60 基线树实跑 nail-red.log）：钉②③红（Gap 形——摘除子树
//! 同 sel 重挂后旧 handle 失锚），钉①绿（存活子树自愈不变式，防回退钉）。

use zero_engine::DomMutation;
use zero_webview::{WebView, WebViewConfig};

/// 钉测页：单容器 `#wrap`（sel-only，初始一个子 `p.old`），无外链资源。
const PAGE: &str = "<!doctype html><html><body><div id=\"wrap\"><p class=\"old\">o</p></div></body></html>";

fn wv() -> WebView {
    WebView::new(WebViewConfig {
        width: 800,
        height: 600,
        ..Default::default()
    })
}

/// 一个 drain 批：begin → apply（defer 渲染）→ end（批量边界统一渲染）——
/// 生产 renderer 异步 drain 路径（drain_pending_script_mutations_inner）的手工等价。
fn apply_batch(wv: &mut WebView, mutations: &[DomMutation]) {
    wv.begin_script_batch();
    wv.apply_dom_mutations_and_render(mutations).expect("批内 apply 成功");
    wv.end_script_batch().expect("批量边界渲染成功");
}

/// 钉①子树换代写路径不变式（存活子树——基线已绿，钉防回退）：批1 建 handle 族
/// 子树（div#a 嵌 a[data-t71]）挂进 sel-only `#wrap`；批2 无关变更换代重建；批3
/// 经批1 旧 handle 写属性。存活子树的自愈机制：apply 的 ephemeral 表预植持久表
/// 存活条目（js_dom_bridge apply 尾段），每批重发 attached+unique 的 handle→selector，
/// 批量边界重锚随之覆盖——修复不得破坏该路径。修复后 Gap 补注册（钉②③的失锚形）
/// 与本路径共存，写同样落新 doc 节点，qsa/读回同步。
#[test]
#[serial_test::serial]
fn t71_subtree_reregister_write_and_qsa_resolvable_after_generation() {
    let mut wv = wv();
    wv.load_html(PAGE, None);

    // 批1：handle 族子树挂进 sel-only 容器（注册 + 本批重锚成立）。
    apply_batch(
        &mut wv,
        &[
            DomMutation::CreateElement {
                handle: "__t71p".into(),
                tag: "div".into(),
            },
            DomMutation::SetAttrOnHandle {
                handle: "__t71p".into(),
                name: "id".into(),
                value: "t71-list".into(),
            },
            DomMutation::CreateElement {
                handle: "__t71c".into(),
                tag: "a".into(),
            },
            DomMutation::SetAttrOnHandle {
                handle: "__t71c".into(),
                name: "id".into(),
                value: "t71-a".into(),
            },
            DomMutation::SetAttrOnHandle {
                handle: "__t71c".into(),
                name: "data-t71".into(),
                value: "v1".into(),
            },
            DomMutation::AppendChildByHandle {
                parent_handle: "__t71p".into(),
                child_handle: "__t71c".into(),
            },
            DomMutation::AppendChild {
                parent_selector: "#wrap".into(),
                child_handle: "__t71p".into(),
            },
        ],
    );

    // 批2：无关变更触发换代重建（批1 族子树 NodeId 全灭，靠 ephemeral 预植
    // 重发自愈重锚——本钉锁定的正是该路径不被修复破坏）。
    apply_batch(
        &mut wv,
        &[
            DomMutation::CreateElement {
                handle: "__t71o".into(),
                tag: "div".into(),
            },
            DomMutation::SetAttrOnHandle {
                handle: "__t71o".into(),
                name: "id".into(),
                value: "t71-other".into(),
            },
            DomMutation::AppendChild {
                parent_selector: "#wrap".into(),
                child_handle: "__t71o".into(),
            },
        ],
    );

    // 批3：经批1 旧 handle 跨批写（spec set an attribute value——节点对象身份
    // 跨换代存活，写必须落当前 doc 的同一元素）。
    apply_batch(
        &mut wv,
        &[DomMutation::SetAttrOnHandle {
            handle: "__t71c".into(),
            name: "data-t71".into(),
            value: "v2".into(),
        }],
    );

    // 落地断言：innerHTML 宿主真相 + qsa 可解析 + 读回同步，三者全绿。
    let landed = wv
        .execute_script(
            r#"
            var host = String(document.getElementById('wrap').innerHTML);
            var qsaOk = document.querySelectorAll('[data-t71="v2"]').length === 1;
            var read = document.querySelector('#t71-a').getAttribute('data-t71');
            String(host.indexOf('data-t71="v2"') >= 0) + ',' + String(qsaOk) + ',' + String(read)
            "#,
        )
        .expect("钉①读回成功");
    assert!(
        landed.contains("true,true,v2"),
        "换代后旧 handle 写须落地宿主（innerHTML 含 data-t71=\"v2\"）且 qsa 唯一命中、\
         getAttribute 读回 v2（修复前管线表 miss 静默 skip 属性停在 v1），实测 {landed:?}"
    );
}

/// 钉②evict 根同 sel 重挂·drain 路径写落地（t70 钉④的 drain 变体，caseG 根因
/// 形）：批1 建柄挂载 → 批2 RemoveHandle（evict 双向清表 + 换代）→ 批3 同 sel
/// 宿主重挂（insertAdjacentHTML，仅 sel 无 handle 注册）→ 批4 经旧 handle 写。
/// 修复前：批4 管线表 miss skip，重挂节点无痕（`set_attr_on_handle unresolvable
/// handle skipped`）；修复后：换代重锚按 evict 前身份（应用史 ledger）回锚到同
/// sel 重挂节点（t70 回收语义/R5010 锚回同 sel 占位者已接受的等价语义），写落地。
#[test]
#[serial_test::serial]
fn t71_evicted_root_remount_write_lands_drain_path() {
    let mut wv = wv();
    wv.load_html(PAGE, None);

    // 批1：建柄挂载。
    apply_batch(
        &mut wv,
        &[
            DomMutation::CreateElement {
                handle: "__t71r".into(),
                tag: "div".into(),
            },
            DomMutation::SetAttrOnHandle {
                handle: "__t71r".into(),
                name: "id".into(),
                value: "t71-r".into(),
            },
            DomMutation::AppendChild {
                parent_selector: "#wrap".into(),
                child_handle: "__t71r".into(),
            },
        ],
    );

    // 批2：handle 移除（evict 清表 + 换代——selector 从 doc 消失）。
    apply_batch(
        &mut wv,
        &[DomMutation::RemoveHandle {
            handle: "__t71r".into(),
        }],
    );

    // 批3：同 sel 宿主重挂（无 handle 注册——caseG 批3 serialized-landing 等价）。
    apply_batch(
        &mut wv,
        &[DomMutation::InsertAdjacentHtml {
            selector: "#wrap".into(),
            position: "beforeend".into(),
            html: "<div id=\"t71-r\">remounted</div>".into(),
        }],
    );

    // 批4：经旧 handle 写重挂节点。
    apply_batch(
        &mut wv,
        &[DomMutation::SetAttrOnHandle {
            handle: "__t71r".into(),
            name: "data-t71-r".into(),
            value: "w1".into(),
        }],
    );

    let landed = wv
        .execute_script(
            r#"
            var host = String(document.getElementById('wrap').innerHTML);
            var qsaOk = document.querySelectorAll('#t71-r').length === 1;
            String(host.indexOf('data-t71-r="w1"') >= 0) + ',' + String(qsaOk)
            "#,
        )
        .expect("钉②读回成功");
    assert!(
        landed.contains("true,true"),
        "evict→同 sel 重挂后旧 handle 写须落地重挂节点（innerHTML 含 data-t71-r=\"w1\" 且 \
         #t71-r 唯一；修复前管线表 miss skip 静默丢写），实测 {landed:?}"
    );
}

/// 钉③跨批 InsertBefore ref 保位（Gap 形——ref 随摘除子树失锚后同 sel 重挂）：
/// 批1 建父柄子树（div#t71-list > b#t71-c）挂 `#wrap`；批2 RemoveHandle 摘除整族
///（R346 游离不产出 selector——父/子柄退出换代重锚清单）；批3 同 sel 宿主重挂
///（insertAdjacentHTML，仅 sel 无 handle 注册）；批4 以批1 父柄为父、批1 子柄为
/// ref 带位插入新节点。修复前：父/ref 双失锚——父失锚整条 mutation lenient skip，
/// 新节点永不落地；修复后：换代补注册回锚到重挂子树，insertBefore 按 ref 原位
/// 落地（https://dom.spec.whatwg.org/#dom-node-insertbefore）。
#[test]
#[serial_test::serial]
fn t71_insert_before_ref_position_preserved_across_generation() {
    let mut wv = wv();
    wv.load_html(PAGE, None);

    // 批1：父柄子树（div#t71-list > b#t71-c）挂进 #wrap。
    apply_batch(
        &mut wv,
        &[
            DomMutation::CreateElement {
                handle: "__t71p".into(),
                tag: "div".into(),
            },
            DomMutation::SetAttrOnHandle {
                handle: "__t71p".into(),
                name: "id".into(),
                value: "t71-list".into(),
            },
            DomMutation::CreateElement {
                handle: "__t71c".into(),
                tag: "b".into(),
            },
            DomMutation::SetAttrOnHandle {
                handle: "__t71c".into(),
                name: "id".into(),
                value: "t71-c".into(),
            },
            DomMutation::AppendChildByHandle {
                parent_handle: "__t71p".into(),
                child_handle: "__t71c".into(),
            },
            DomMutation::AppendChild {
                parent_selector: "#wrap".into(),
                child_handle: "__t71p".into(),
            },
        ],
    );

    // 批2：摘除整族（父柄指名移除——后代随子树游离，双双退出注册）。
    apply_batch(
        &mut wv,
        &[DomMutation::RemoveHandle {
            handle: "__t71p".into(),
        }],
    );

    // 批3：同 sel 宿主重挂（无 handle 注册——caseG 批3 serialized-landing 等价）。
    apply_batch(
        &mut wv,
        &[DomMutation::InsertAdjacentHtml {
            selector: "#wrap".into(),
            position: "beforeend".into(),
            html: "<div id=\"t71-list\"><b id=\"t71-c\"></b></div>".into(),
        }],
    );

    // 批4：以批1 父柄为父、批1 子柄为 ref 的带位插入。
    apply_batch(
        &mut wv,
        &[
            DomMutation::CreateElement {
                handle: "__t71z".into(),
                tag: "u".into(),
            },
            DomMutation::SetAttrOnHandle {
                handle: "__t71z".into(),
                name: "id".into(),
                value: "t71-z".into(),
            },
            DomMutation::InsertBeforeByHandleHandle {
                parent_handle: "__t71p".into(),
                child_handle: "__t71z".into(),
                ref_handle: "__t71c".into(),
            },
        ],
    );

    // 位置断言：z 已落地且在 c 之前（ref 保位；修复前父失锚整条 skip，z 不存在）。
    let ordered = wv
        .execute_script(
            r#"
            var list = document.getElementById('t71-list');
            var html = String(list.innerHTML);
            var zi = html.indexOf('<u id="t71-z"');
            var ci = html.indexOf('<b id="t71-c"');
            String(zi >= 0) + ',' + String(ci >= 0) + ',' + String(zi < ci)
            "#,
        )
        .expect("钉③读回成功");
    assert!(
        ordered.contains("true,true,true"),
        "跨批 InsertBefore 须以批1 柄解析父/ref 并保位（z 在 c 前；修复前父失锚整条 \
         mutation lenient skip、z 永不落地），实测 {ordered:?}"
    );
}
