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

/// 钉④（T71 返修 DF-1 根修）：死 handle 的 ledger 补注册不得回锚到已被其他
/// 现役柄持久持有的节点。不变量：`persistent_handle_nodes` 注册表是节点所有权
/// 唯一权威——同一节点不得被两个 handle 同时持久锚定；所有权冲突候选退
/// lenient skip（宁漏勿错），死引用写保持不可见基线（同柄自 reclaim 不受影响，
/// 钉①②已钉）。缺陷形（修复前）：现役认领守卫读 `selector_handle_map`，而
/// drain 路径从不 merge 该表（守卫输入恒空恒放行），`rebind_handle_node` 只查
/// selector 唯一性不查节点归属——死柄 `__t71g` 经 ledger 旧 selector 回锚到活柄
/// `__t71l` 的节点，双柄同锚，后续经 `__t71g` 的写静默改写他人现役节点（DF
/// 定向探针 PROBE1-OUT 实证：data-via="ghost" 落 successor 节点，
/// review/t71-df-targeted/df1-probe-run3-nocapture.log）。规范语义：DOM §4.2.2
/// concept-node-pre-insert——插入宿主子树后身份归属随子树唯一，
/// https://dom.spec.whatwg.org/#concept-node-pre-insert。
/// 红先证：bench/results/t71-repair/nail-red.log（本钉 FAIL——活柄写与死柄写
/// 同时落同一节点）；修复后 nail-green.log 全绿。
#[test]
#[serial_test::serial]
fn t71_dead_handle_ledger_never_reanchors_node_claimed_by_live_handle() {
    let mut wv = wv();
    wv.load_html(PAGE, None);

    // 批1：建死柄子树（div.t71-ghost，当时唯一 → ledger 记 __t71g 的锚点）。
    apply_batch(
        &mut wv,
        &[
            DomMutation::CreateElement {
                handle: "__t71g".into(),
                tag: "div".into(),
            },
            DomMutation::SetAttrOnHandle {
                handle: "__t71g".into(),
                name: "class".into(),
                value: "t71-ghost".into(),
            },
            DomMutation::AppendChild {
                parent_selector: "#wrap".into(),
                child_handle: "__t71g".into(),
            },
        ],
    );

    // 批2：指名移除（页面真实意图：该节点废弃；evict 双向清表 + 换代——
    // ledger 残项保留，selector 从 doc 消失）。
    apply_batch(
        &mut wv,
        &[DomMutation::RemoveHandle {
            handle: "__t71g".into(),
        }],
    );

    // 批3：新元素恰好占住同 selector（活柄 __t71l，data-owner 标记主人）。
    apply_batch(
        &mut wv,
        &[
            DomMutation::CreateElement {
                handle: "__t71l".into(),
                tag: "div".into(),
            },
            DomMutation::SetAttrOnHandle {
                handle: "__t71l".into(),
                name: "class".into(),
                value: "t71-ghost".into(),
            },
            DomMutation::SetAttrOnHandle {
                handle: "__t71l".into(),
                name: "data-owner".into(),
                value: "l".into(),
            },
            DomMutation::AppendChild {
                parent_selector: "#wrap".into(),
                child_handle: "__t71l".into(),
            },
        ],
    );

    // 批4：分别经死柄 __t71g 与活柄 __t71l 写。
    apply_batch(
        &mut wv,
        &[
            DomMutation::SetAttrOnHandle {
                handle: "__t71g".into(),
                name: "data-ghost".into(),
                value: "1".into(),
            },
            DomMutation::SetAttrOnHandle {
                handle: "__t71l".into(),
                name: "data-live".into(),
                value: "1".into(),
            },
        ],
    );

    let out = wv
        .execute_script(
            r#"
            var card = document.querySelector('div.t71-ghost');
            String(card ? card.outerHTML : 'MISSING')
            "#,
        )
        .expect("钉④读回成功");
    assert!(
        out.contains("data-owner=\"l\"") && out.contains("data-live=\"1\"") && !out.contains("data-ghost"),
        "死 handle 的 ledger 补注册不得回锚到现役柄持有的节点（注册表唯一所有权，\
         双柄同锚禁止）：活柄 __t71l 写须落地（data-live=\"1\"），死柄 __t71g 写须保持 \
         lenient skip 基线（节点上不得出现 data-ghost——修复前守卫在 drain 路径恒失效，\
         死柄经 ledger 回锚活柄节点、写静默劫持他人现役节点），实测 {out:?}"
    );
}

/// DF-2 钉测的 tracing 域内捕获订阅（`with_default` 线程局部作用域，无全局订阅
/// 安装——进程级全局资源跨用例串行由 #[serial] 保证）。只收集事件 message 字段。
#[derive(Default, Clone)]
struct RolloverWarnCapture {
    lines: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
}

impl RolloverWarnCapture {
    fn snapshot(&self) -> Vec<String> {
        self.lines.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
}

struct MessageOnlyVisitor(String);

impl tracing::field::Visit for MessageOnlyVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0 = format!("{value:?}");
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.0 = value.to_string();
        }
    }
}

impl tracing::Subscriber for RolloverWarnCapture {
    fn enabled(&self, _metadata: &tracing::Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, _attrs: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        tracing::span::Id::from_u64(1)
    }

    fn record(&self, _span: &tracing::span::Id, _values: &tracing::span::Record<'_>) {}

    fn record_follows_from(&self, _span: &tracing::span::Id, _follows: &tracing::span::Id) {}

    fn event(&self, event: &tracing::Event<'_>) {
        let mut visitor = MessageOnlyVisitor(String::new());
        event.record(&mut visitor);
        if !visitor.0.is_empty() {
            self.lines.lock().unwrap_or_else(|e| e.into_inner()).push(visitor.0);
        }
    }

    fn enter(&self, _span: &tracing::span::Id) {}

    fn exit(&self, _span: &tracing::span::Id) {}
}

/// 钉⑤（T71 返修 DF-2 计数语义）：换代掉项聚合 warn 只报真正无落点的掉项——
/// 本批合法移除（RemoveHandle 指名移除等已知合法落点）计入豁免、不计入掉项。
/// 缺陷形（修复前）：remove_child 只摘链不灭 slotmap 槽，apply 尾 retain 保留
/// detached 条目 → pre_registry 含被合法移除的柄，换代后必掉项 → 任何移除过
/// 注册柄的批必发一条 rollover dropped warn（T69 §6.3 换代掉项信号被正常 DOM
/// 移除噪声化，活体 feed 每批移除卡片即恒响）。真掉项正控：本批 mutation 面
/// 之外的失锚（SetInnerHtml 祖先内容拭除——柄节点被替换丢弃，不在裁决书列举的
/// Remove 家族豁免面内）仍必须计入掉项——豁免不得钝化真实信号。豁免集取
/// Remove/RemoveHandle 最小裁决列举面：attached 节点由 apply 尾 emission 以
/// `unique_selector_for_node` 造位置化唯一 selector 自愈重锚（形 B 若用"重复
/// selector 失唯一"形，柄会被自愈救援回锚原节点、dropped=0——GREEN 实跑
/// nail-green-attempt1.log 证伪该形，故真掉项须取拭除形）。
/// 红先证：bench/results/t71-repair/nail-red.log（形 A warn 必发——修复前合法
/// 移除被计入掉项，即 DF-2 行为修正的先红；形 B 为既有信号保真正控，非行为
/// 修正面，无独立先红要求）。
#[test]
#[serial_test::serial]
fn t71_rollover_dropped_warn_excludes_legal_removal_counts() {
    // 形 A：纯合法移除——零 warn（豁免语义；修复前必发 dropped 1 噪声）。
    {
        let mut wv = wv();
        wv.load_html(PAGE, None);
        apply_batch(
            &mut wv,
            &[
                DomMutation::CreateElement {
                    handle: "__t71a".into(),
                    tag: "div".into(),
                },
                DomMutation::SetAttrOnHandle {
                    handle: "__t71a".into(),
                    name: "class".into(),
                    value: "t71-legal".into(),
                },
                DomMutation::AppendChild {
                    parent_selector: "#wrap".into(),
                    child_handle: "__t71a".into(),
                },
            ],
        );
        let capture = RolloverWarnCapture::default();
        let lines = tracing::subscriber::with_default(capture.clone(), || {
            apply_batch(
                &mut wv,
                &[DomMutation::RemoveHandle {
                    handle: "__t71a".into(),
                }],
            );
            capture.snapshot()
        });
        assert!(
            !lines.iter().any(|l| l.contains("rollover")),
            "本批合法移除（RemoveHandle）不得计入换代掉项——修复前 pre_registry 经 \
             apply 尾 retain 保留 detached 条目、换代后必发 rollover dropped 噪声 warn，\
             捕获到 {lines:?}"
        );
    }

    // 形 B：合法移除 + 真掉项混批——dropped 恰 1（真掉项）、exempt 恰 1（合法移除）。
    {
        let mut wv = wv();
        wv.load_html(PAGE, None);
        apply_batch(
            &mut wv,
            &[
                // 合法移除柄（div#t71-legal-b）。
                DomMutation::CreateElement {
                    handle: "__t71b".into(),
                    tag: "div".into(),
                },
                DomMutation::SetAttrOnHandle {
                    handle: "__t71b".into(),
                    name: "id".into(),
                    value: "t71-legal-b".into(),
                },
                DomMutation::AppendChild {
                    parent_selector: "#wrap".into(),
                    child_handle: "__t71b".into(),
                },
                // 宿主容器（#t71-host，内含真掉项柄的节点）。
                DomMutation::InsertAdjacentHtml {
                    selector: "#wrap".into(),
                    position: "beforeend".into(),
                    html: "<div id=\"t71-host\"></div>".into(),
                },
                // 真掉项柄（div#t71-dup-c 挂 #t71-host——下批被 SetInnerHtml 拭除）。
                DomMutation::CreateElement {
                    handle: "__t71c".into(),
                    tag: "div".into(),
                },
                DomMutation::SetAttrOnHandle {
                    handle: "__t71c".into(),
                    name: "id".into(),
                    value: "t71-dup-c".into(),
                },
                DomMutation::AppendChild {
                    parent_selector: "#t71-host".into(),
                    child_handle: "__t71c".into(),
                },
            ],
        );
        let capture = RolloverWarnCapture::default();
        let lines = tracing::subscriber::with_default(capture.clone(), || {
            apply_batch(
                &mut wv,
                &[
                    DomMutation::RemoveHandle {
                        handle: "__t71b".into(),
                    },
                    // 祖先内容拭除：__t71c 节点随 #t71-host 子树替换被丢弃——
                    // 非 Remove 家族豁免面，须计入真掉项。
                    DomMutation::SetInnerHtml {
                        selector: "#t71-host".into(),
                        html: "<p>replaced</p>".into(),
                    },
                ],
            );
            capture.snapshot()
        });
        let line = lines.iter().find(|l| l.contains("rollover")).unwrap_or_else(|| {
            panic!(
                "真掉项（SetInnerHtml 祖先拭除，无豁免落点）必须计入换代掉项 warn——\
                     豁免不得钝化真实信号，捕获 {lines:?}"
            )
        });
        assert!(
            line.contains("dropped 1 entries") && line.contains("legal-removal exempt 1"),
            "计数语义恰集：合法移除 __t71b 豁免（exempt 1）、真掉项 __t71c 计入\
            （dropped 1）——修复前两者都计入掉项（dropped 2 entries），实测 {line:?}"
        );
    }
}
