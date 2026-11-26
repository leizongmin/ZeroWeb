//! s402（html-semantics M2）：shadow 森林 handle 身份跨探测环刷新的重建面。
//!
//! 根因链（zz-probe-shadow-gcs v4 定谳）：shim 序列化对命令式 attachShadow 根按
//! 「列表成员恒收」资格发 `<template shadowrootmode>`（part05 `_zwSrQualifies180`
//! list 臂 + 引擎 serializer.rs ShadowRoot 臂同形），探测环 dump **携声明式标记**；
//! refresh re-parse 经 parser fixup 物化 shadow 树——树不丢，丢的是 **handle 身份**：
//! 序列化产物不带 handle 标记，`__zw_get_computed_style_by_handle` 的
//! `nodes[handle]` 只来自重放窗口 `[applied..count)`（s161 契约），队列 drain 尽
//! （applied == count）后窗口为空 → shadow 树内元素 handle 全部落空 → gCS 返空串。
//! 刷新后新 attach 的 shadow 一个探测周期后同病（`fresh d2=''` 实证）。
//!
//! 修法（by-handle 视图就地重建，零新 wire）：视图基座剥声明式模板 span
//!（[`strip_declarative_shadow_templates`]——剥后 parse 不物化 shadow，交给切片
//! 重放重建），再按「最终归属闭包」过滤共享队列前缀 `[0..applied)` 取 shadow 森林
//! mutation 切片（[`shadow_forest_mutations`]）重放——Create*/attach/append 链在视图
//! doc 重注册全部森林 handle。最终归属口径：dump 已是 shim 终态真相，重放只重建
//! dump 排除掉的森林终态——中间迁移（forest→light 挪回）按最后一次 placement 定
//! 归属，避免重放把已挪回 light 的节点再塞回 shadow。
//!
//! 与 s168 全量重放臂（computed_style_cache 同函数）的关系：s168 臂服务「基座不含
//! 声明式标记」形态（注册期原文，基座缺全部落地效果 → 全量重放）；本模块服务互补
//! 形态「基座含标记但 handle 身份不在窗口」（drain 尽的 dump/engine 形态基座）。

use super::DomMutation;

/// 共享队列前缀 `[0..applied)` 的 shadow 森林 mutation 切片（顺序保留，供
/// `__zw_get_computed_style_by_handle` 视图重建重放——重注册森林 handle）。
///
/// 三遍算法（O(n)，n = 队列长度）：
/// 1. 种子 + 最后归属表：AttachShadow/AttachShadowOnHandle 的 `root_handle` 无条件
///    入种子；handle 型 placement 边（append/insert/fragment flatten）按队列序
///    覆盖写 `child → parent`——最后一次 placement 定终态；light 锚 placement
///    （selector 父）与 RemoveHandle 记「离林」。
/// 2. 成员闭包：handle ∈ 森林 ⇔ 种子，或 parent ∈ 森林（记忆化下行，环防御返假
///    ——DOM 父链无环，防御病态队列）。
/// 3. 顺序过滤：种子恒收；Create* 按其 handle 成员收；anchor 变体按其 handle 成员
///    收；placement 边按 child 终态成员收（父成员随之隐含）；sel 子 move-in
///    （AppendChildByHandleSel）按 parent 成员收。
///
/// 已知近似（钉账）：sel 子 move-in 重放时 `child_selector` 在剥模板基座查不到
/// （该子终态在 shadow 内 → dump 把它序列化进模板 → 剥除带走）→ lenient miss，
/// 视图缺该 sel 子——其无 handle（parsed 元素），gCS by-handle 面不受影响。
///
/// light 宿主命令式 shadow（createElement 宿主 append 进 light 后 attachShadow）：
/// 宿主 create 不重放（dump 已含，重放双份）→ AttachShadowOnHandle 发射会撞
/// unknown host handle Err 中止切片尾部。整树 prune（根不入种子 → 成员闭包落选
/// → 子树 mutation 全弃）——重放零 Err，后继森林链不受截断波及；该子树视图缺
/// （与今日行为同形，lenient 不劣化）。
pub fn shadow_forest_mutations(m: &[DomMutation], applied: usize) -> Vec<DomMutation> {
    let end = applied.min(m.len());
    // 遍 1：种子 + 最后归属（后写覆盖前写 = 终态口径）。
    let mut seeds: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut parent_of: std::collections::HashMap<String, Placement> = std::collections::HashMap::new();
    for mutation in &m[..end] {
        match mutation {
            DomMutation::AttachShadow { root_handle, .. } | DomMutation::AttachShadowOnHandle { root_handle, .. } => {
                seeds.insert(root_handle.clone());
            }
            // handle 锚 placement：child 归属 parent（森林候选链）。
            DomMutation::AppendChildByHandle {
                parent_handle,
                child_handle,
            }
            | DomMutation::AppendChildByHandlePath {
                parent_handle,
                child_handle,
                ..
            }
            | DomMutation::InsertBeforeByHandle {
                parent_handle,
                child_handle,
                ..
            }
            | DomMutation::InsertBeforeByHandleHandle {
                parent_handle,
                child_handle,
                ..
            }
            | DomMutation::AppendFragmentChildrenByHandle {
                parent_handle,
                fragment_handle: child_handle,
            }
            | DomMutation::InsertFragmentBeforeByHandle {
                parent_handle,
                fragment_handle: child_handle,
                ..
            }
            | DomMutation::InsertFragmentChildrenBeforeHandle {
                parent_handle,
                fragment_handle: child_handle,
                ..
            } => {
                parent_of.insert(child_handle.clone(), Placement::Handle(parent_handle.clone()));
            }
            // light 锚 placement：child 离林（真 detached/removed 同口径——终态不在
            // 森林即可，detached 与 light 对视图重建无差别）。
            DomMutation::AppendChild { child_handle, .. }
            | DomMutation::AppendChildBySelPath { child_handle, .. }
            | DomMutation::InsertBefore { child_handle, .. }
            | DomMutation::InsertAdjacentElement { child_handle, .. }
            | DomMutation::RemoveHandle { handle: child_handle } => {
                parent_of.insert(child_handle.clone(), Placement::Detached);
            }
            _ => {}
        }
    }
    // 遍 2：成员闭包（记忆化 + 在途环防御）。
    fn in_forest(
        h: &str,
        seeds: &std::collections::HashSet<String>,
        parent_of: &std::collections::HashMap<String, Placement>,
        visiting: &mut std::collections::HashSet<String>,
    ) -> bool {
        if seeds.contains(h) {
            return true;
        }
        match parent_of.get(h) {
            Some(Placement::Handle(p)) => {
                if !visiting.insert(h.to_string()) {
                    return false;
                }
                let r = in_forest(p, seeds, parent_of, visiting);
                visiting.remove(h);
                r
            }
            _ => false,
        }
    }
    // 遍 2b：light 宿主命令式 shadow 整树 prune——AttachShadowOnHandle 宿主终态
    // 离林（create 不重放，attach 发射必撞 unknown host Err）→ 根出种子重算闭包，
    // 迭代至不动点（嵌套 forest 内宿主随外层 prune 逐层落选）。
    loop {
        let mut changed = false;
        let hosts: Vec<(String, String)> = m[..end]
            .iter()
            .filter_map(|mutation| match mutation {
                DomMutation::AttachShadowOnHandle {
                    host_handle,
                    root_handle,
                    ..
                } => Some((host_handle.clone(), root_handle.clone())),
                _ => None,
            })
            .collect();
        for (host, root) in hosts {
            if seeds.contains(&root) && !in_forest(&host, &seeds, &parent_of, &mut std::collections::HashSet::new()) {
                seeds.remove(&root);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    // 遍 3：顺序过滤发射。AttachShadowOnHandle 宿主准入：宿主 create 本身被保留
    //（终态在林）才发射——light 宿主形态已在遍 2b 整树 prune（子树 mutation 不入
    // 切片），此处 kept_create 门为重放序一致性兜底。AttachShadow（selector 宿主）
    // 经 find_by_selector 在剥模板基座解析，恒可发射。
    let kept_create: std::collections::HashSet<String> = m[..end]
        .iter()
        .filter_map(|mutation| match mutation {
            DomMutation::CreateElement { handle, .. }
            | DomMutation::CreateElementNS { handle, .. }
            | DomMutation::CreateTextNode { handle, .. }
            | DomMutation::CreateComment { handle, .. }
            | DomMutation::CreateProcessingInstruction { handle, .. }
            | DomMutation::CreateDocumentFragment { handle } => {
                if in_forest(handle, &seeds, &parent_of, &mut std::collections::HashSet::new()) {
                    Some(handle.clone())
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect();
    let mut out = Vec::new();
    for mutation in &m[..end] {
        let keep = match mutation {
            DomMutation::AttachShadow { .. } => true,
            DomMutation::AttachShadowOnHandle { host_handle, .. } => kept_create.contains(host_handle),
            DomMutation::CreateElement { handle, .. }
            | DomMutation::CreateElementNS { handle, .. }
            | DomMutation::CreateTextNode { handle, .. }
            | DomMutation::CreateComment { handle, .. }
            | DomMutation::CreateProcessingInstruction { handle, .. }
            | DomMutation::CreateDocumentFragment { handle }
            | DomMutation::SetAttrOnHandle { handle, .. }
            | DomMutation::RemoveAttrOnHandle { handle, .. }
            | DomMutation::SetTextOnHandle { handle, .. }
            | DomMutation::SetInnerHtmlOnHandle { handle, .. }
            | DomMutation::SetInnerHtmlOnHandleUnsafe { handle, .. }
            | DomMutation::SetStyleOnHandle { handle, .. }
            | DomMutation::RemoveStyleOnHandle { handle, .. }
            | DomMutation::RemoveHandle { handle } => {
                in_forest(handle, &seeds, &parent_of, &mut std::collections::HashSet::new())
            }
            // slot_ref 句柄优先失配回落选择器（45 号 M3）——'@handle' 形态剥前缀判。
            DomMutation::SlotAssign { slot_ref, .. } => in_forest(
                slot_ref.strip_prefix('@').unwrap_or(slot_ref),
                &seeds,
                &parent_of,
                &mut std::collections::HashSet::new(),
            ),
            DomMutation::AppendChildByHandle {
                parent_handle,
                child_handle,
            }
            | DomMutation::AppendChildByHandlePath {
                parent_handle,
                child_handle,
                ..
            }
            | DomMutation::InsertBeforeByHandle {
                parent_handle,
                child_handle,
                ..
            }
            | DomMutation::InsertBeforeByHandleHandle {
                parent_handle,
                child_handle,
                ..
            }
            | DomMutation::AppendFragmentChildrenByHandle {
                parent_handle,
                fragment_handle: child_handle,
            }
            | DomMutation::InsertFragmentBeforeByHandle {
                parent_handle,
                fragment_handle: child_handle,
                ..
            }
            | DomMutation::InsertFragmentChildrenBeforeHandle {
                parent_handle,
                fragment_handle: child_handle,
                ..
            } => {
                let _ = parent_handle;
                in_forest(child_handle, &seeds, &parent_of, &mut std::collections::HashSet::new())
            }
            // sel 子 move-in：child 无 handle，按父成员收（重放 miss lenient，见头注）。
            DomMutation::AppendChildByHandleSel { parent_handle, .. } => {
                in_forest(parent_handle, &seeds, &parent_of, &mut std::collections::HashSet::new())
            }
            _ => false,
        };
        if keep {
            out.push(mutation.clone());
        }
    }
    out
}

/// 遍 1 的 placement 归属值（Handle = handle 锚父链候选；Detached = light 锚父 /
/// 移除——终态离林）。
enum Placement {
    Handle(String),
    Detached,
}

/// 剥除串内全部 `<template …shadowrootmode…>…</template>` span（深度配平嵌套
/// template）。供 by-handle 视图重建：剥后 parse 不物化 shadow 树，森林交给
/// [`shadow_forest_mutations`] 切片重放重建（handle 身份随 Create*/attach 链回归）。
///
/// 已知近似（钉账，与 shim `_zwStripEchoHtml180` FIXME 同族）：开标签判按
/// `shadowrootmode` 子串（属性值含该词的病态串误剥）；`<template` 字面出现在
/// 注释/属性值内会扰动深度配平。两发射源（shim part05 / 引擎 serializer）产物
/// 形态固定，语料面无钉例。
pub fn strip_declarative_shadow_templates(html: &str) -> String {
    if !html.contains("shadowrootmode") {
        return html.to_string();
    }
    let bytes = html.as_bytes();
    let mut out = String::with_capacity(html.len());
    let mut i = 0;
    while i < bytes.len() {
        // 找下一个 template 开标签起点（`</template` 以 `/` 隔开不会撞 `<template`）。
        match html[i..].find("<template") {
            Some(off) => {
                let tag_start = i + off;
                out.push_str(&html[i..tag_start]);
                let Some(gt_rel) = html[tag_start..].find('>') else {
                    // 开标签未闭合（病态串）——余量原样透传。
                    out.push_str(&html[tag_start..]);
                    return out;
                };
                let tag_end = tag_start + gt_rel;
                let open_tag = &html[tag_start..=tag_end];
                if !open_tag.contains("shadowrootmode") {
                    // 普通 template（页面源声明模板等）——原样保留，跳过其开标签。
                    out.push_str(open_tag);
                    i = tag_end + 1;
                    continue;
                }
                // 深度配平扫描本 span 的闭 tag（嵌套 template +1/-1）。
                let mut depth = 1usize;
                let mut j = tag_end + 1;
                let mut span_end = None;
                while j < bytes.len() {
                    if html[j..].starts_with("<template") {
                        depth += 1;
                        j += "<template".len();
                    } else if html[j..].starts_with("</template>") {
                        depth -= 1;
                        j += "</template>".len();
                        if depth == 0 {
                            span_end = Some(j);
                            break;
                        }
                    } else {
                        j += 1;
                    }
                }
                match span_end {
                    Some(end) => {
                        // 整 span 剥除（含 shadow 内容——由切片重放重建）。
                        i = end;
                    }
                    None => {
                        // 未配平（病态串）——开标签原样保留，不吞余量。
                        out.push_str(open_tag);
                        i = tag_end + 1;
                    }
                }
            }
            None => {
                out.push_str(&html[i..]);
                return out;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attach(host_sel: &str, root: &str) -> DomMutation {
        DomMutation::AttachShadow {
            host_selector: host_sel.into(),
            root_handle: root.into(),
            mode: "open".into(),
            delegates_focus: false,
            clonable: false,
            serializable: false,
            slot_assignment: false,
        }
    }

    fn create(h: &str) -> DomMutation {
        DomMutation::CreateElement {
            handle: h.into(),
            tag: "div".into(),
        }
    }

    fn append(parent: &str, child: &str) -> DomMutation {
        DomMutation::AppendChildByHandle {
            parent_handle: parent.into(),
            child_handle: child.into(),
        }
    }

    fn set_attr(h: &str) -> DomMutation {
        DomMutation::SetAttrOnHandle {
            handle: h.into(),
            name: "id".into(),
            value: "x".into(),
        }
    }

    fn light_append(parent_sel: &str, child: &str) -> DomMutation {
        DomMutation::AppendChild {
            parent_selector: parent_sel.into(),
            child_handle: child.into(),
        }
    }

    #[test]
    fn s402_forest_slice_light_interleave() {
        // light/light/forest 链/light 混排——切片只收森林链且顺序保留。
        let m = vec![
            light_append("#a", "__n0"),
            set_attr("__n0"),
            attach("#host", "__n1"),
            create("__n2"),
            append("__n1", "__n2"),
            set_attr("__n2"),
            light_append("#b", "__n3"),
            set_attr("__n3"),
        ];
        let slice = shadow_forest_mutations(&m, m.len());
        assert_eq!(slice.len(), 4);
        assert!(matches!(slice[0], DomMutation::AttachShadow { ref root_handle, .. } if root_handle == "__n1"));
        assert!(matches!(slice[1], DomMutation::CreateElement { ref handle, .. } if handle == "__n2"));
        assert!(
            matches!(slice[2], DomMutation::AppendChildByHandle { ref parent_handle, ref child_handle }
            if parent_handle == "__n1" && child_handle == "__n2")
        );
        assert!(matches!(slice[3], DomMutation::SetAttrOnHandle { ref handle, .. } if handle == "__n2"));
    }

    #[test]
    fn s402_forest_slice_final_placement_move_back() {
        // forest → light 挪回：child 终态离林，建链与 attr 全部不重放。
        let m = vec![
            attach("#host", "__n1"),
            create("__n2"),
            append("__n1", "__n2"),
            set_attr("__n2"),
            light_append("#b", "__n2"),
        ];
        let slice = shadow_forest_mutations(&m, m.len());
        assert_eq!(slice.len(), 1);
        assert!(matches!(slice[0], DomMutation::AttachShadow { ref root_handle, .. } if root_handle == "__n1"));
    }

    #[test]
    fn s402_forest_slice_light_then_forest_move_in() {
        // light 创建 → 挪入森林：终态在林，append 边与后续 attr 重放（create 在林外
        // 发生——视图基座 dump 无该节点，append 重放经由 create 注册的 handle 落林）。
        let m = vec![
            create("__n2"),
            set_attr("__n2"),
            attach("#host", "__n1"),
            append("__n1", "__n2"),
            set_attr("__n2"),
        ];
        let slice = shadow_forest_mutations(&m, m.len());
        // anchor 变体按**终态**成员收——离林期的首 set_attr 也保留（create 在队列序
        // 上恒先行，重放依赖序成立，attr 覆盖至终值）。create 终态在林 → 重放（提供
        // handle 注册源）。
        assert_eq!(slice.len(), 5);
        assert!(matches!(slice[0], DomMutation::CreateElement { ref handle, .. } if handle == "__n2"));
        assert!(matches!(slice[2], DomMutation::AttachShadow { .. }));
    }

    #[test]
    fn s402_forest_slice_applied_prefix_only() {
        // applied 截断：窗口外（pending 尾）不入切片——pending 由重放窗口覆盖。
        let m = vec![attach("#host", "__n1"), create("__n2"), append("__n1", "__n2")];
        let slice = shadow_forest_mutations(&m, 1);
        assert_eq!(slice.len(), 1);
        assert!(matches!(slice[0], DomMutation::AttachShadow { .. }));
    }

    #[test]
    fn s402_forest_slice_empty_without_shadow() {
        let m = vec![create("__n0"), light_append("#a", "__n0"), set_attr("__n0")];
        assert!(shadow_forest_mutations(&m, m.len()).is_empty());
    }

    #[test]
    fn s402_forest_slice_light_host_imperative_pruned() {
        // light 宿主命令式 shadow（createElement 宿主 append 进 body 后 attach）：
        // 整树 prune——子树 mutation 全弃，后继静态宿主森林链不受 Err 截断波及。
        let m = vec![
            attach("#host", "__n1"),
            create("__n2"),
            append("__n1", "__n2"),
            // host2：createElement → light 放置 → attachShadow。
            create("__n3"),
            light_append("#b", "__n3"),
            DomMutation::AttachShadowOnHandle {
                host_handle: "__n3".into(),
                root_handle: "__n4".into(),
                mode: "open".into(),
                delegates_focus: false,
                clonable: false,
                serializable: false,
                slot_assignment: false,
            },
            create("__n5"),
            append("__n4", "__n5"),
            // 后继静态宿主链——prune 后必须完整保留（无 unknown handle Err 源）。
            attach("#host3", "__n6"),
            create("__n7"),
            append("__n6", "__n7"),
        ];
        let slice = shadow_forest_mutations(&m, m.len());
        // 保留面 = 首链 3 + 尾链 3；host2 子树 3 笔全弃。
        assert_eq!(slice.len(), 6);
        assert!(slice.iter().all(|mm| !matches!(
            mm,
            DomMutation::CreateElement { handle, .. } if handle == "__n3" || handle == "__n5"
        )));
        assert!(matches!(slice[3], DomMutation::AttachShadow { ref root_handle, .. } if root_handle == "__n6"));
        assert!(
            matches!(slice[5], DomMutation::AppendChildByHandle { ref parent_handle, ref child_handle }
            if parent_handle == "__n6" && child_handle == "__n7")
        );
    }

    #[test]
    fn s402_strip_removes_shadow_template_keeps_light_template() {
        let html = r#"<body><template id="t">plain</template><div id="host"><template shadowrootmode="open"><link rel="stylesheet"><div class="green"><slot></slot></div></template>host text</div></body>"#;
        let stripped = strip_declarative_shadow_templates(html);
        assert!(!stripped.contains("shadowrootmode"));
        assert!(!stripped.contains("<slot>"));
        assert!(stripped.contains(r#"<template id="t">plain</template>"#));
        assert!(stripped.contains("host text"));
    }

    #[test]
    fn s402_strip_nested_templates_depth_matched() {
        // shadow 模板内嵌普通 template——深度配平整 span 剥除。
        let html = r#"<div id="h"><template shadowrootmode="open"><template id="inner">x</template><div>y</div></template>light</div>"#;
        let stripped = strip_declarative_shadow_templates(html);
        assert!(!stripped.contains("shadowrootmode"));
        assert!(!stripped.contains("<div>y</div>"));
        assert!(stripped.contains("light"));
    }

    #[test]
    fn s402_strip_no_marker_passthrough() {
        let html = r#"<div><template id="t">x</template></div>"#;
        assert_eq!(strip_declarative_shadow_templates(html), html);
    }

    #[test]
    fn k139_slot_assign_path_ref_plain_and_innerhtml() {
        // changing-002 test 2/3 链（wptfyi-css k139）：sel-form 宿主 manual shadow
        // root → template clone 物化 slot/p（handle 世界，CreateElement+append 上传
        // 注册）→ slot.assign 的双 miss 节点（plain Text ／ sel 路径 innerHTML 解析
        // 代理产物——无 handle 无 selector）以路径形引用上传
        // `"@path:<anchorRef>|<idx.chain>"`。断言：路径解析命中视图树节点（文本与
        // 元素同通道）→ 手动表落表 → :has-slotted 翻转；清空回落。修复前形态：
        // 三级解析全 miss → shim 侧 skipSl 整 slot 不同步 → 手动表空 → gCS blue。
        use crate::js_dom_bridge::apply_dom_mutations_full;
        use std::collections::HashMap;
        let mut doc = zero_dom::parse_html("<body><my-element id=test>Test</my-element></body>");
        let mut persistent_nodes: HashMap<String, zero_dom::NodeId> = HashMap::new();
        let head = |slot_assignment: bool| -> Vec<DomMutation> {
            vec![
                DomMutation::AttachShadow {
                    host_selector: "#test".into(),
                    root_handle: "__sr0".into(),
                    mode: "open".into(),
                    delegates_focus: false,
                    clonable: false,
                    serializable: false,
                    slot_assignment,
                },
                DomMutation::CreateElement {
                    handle: "__n1".into(),
                    tag: "slot".into(),
                },
                append("__sr0", "__n1"),
                DomMutation::CreateElement {
                    handle: "__n2".into(),
                    tag: "p".into(),
                },
                DomMutation::SetAttrOnHandle {
                    handle: "__n2".into(),
                    name: "id".into(),
                    value: "target".into(),
                },
                append("__sr0", "__n2"),
            ]
        };
        let slot = |doc: &zero_dom::Document| {
            doc.query_selector_including_shadow("slot")
                .expect("shadow 树内 slot 应可达")
        };
        // 场景 1（test 2 形）：assign(plain Text)——静态文本子路径引用。
        let m1 = head(true);
        let mut m1 = m1;
        m1.push(DomMutation::SlotAssign {
            slot_ref: "__n1".into(),
            nodes: vec!["@path:#test|0".into()],
        });
        apply_dom_mutations_full(&mut doc, &m1, None, Some(&mut persistent_nodes), None, None)
            .expect("plain Text 路径引用应可解析");
        let s = slot(&doc);
        assert!(
            doc.is_has_slotted_element(s),
            "manual 分配 plain Text 后 :has-slotted 应命中（活过滤 parent==host）"
        );
        let text = doc.get_assigned_nodes(s)[0];
        assert!(
            matches!(
                doc.get(text).map(|n| &n.kind),
                Some(zero_dom::NodeKind::Text(t)) if t.content == "Test"
            ),
            "手动表应恰含静态文本子"
        );
        // 清空回落（test 2 尾段形态）。
        apply_dom_mutations_full(
            &mut doc,
            &[DomMutation::SlotAssign {
                slot_ref: "__n1".into(),
                nodes: vec![],
            }],
            None,
            Some(&mut persistent_nodes),
            None,
            None,
        )
        .expect("空列表清表应可应用");
        assert!(
            !doc.is_has_slotted_element(s),
            "assign() 清空后 :has-slotted 应回落假"
        );
        // 场景 2（test 3 形）：sel 路径 innerHTML 物化 div → assign(div)——同链路径
        // 引用按重放后视图树定位（innerHTML 已入队列，视图树 test 子 = div）。
        let m2 = vec![DomMutation::SetInnerHtml {
            selector: "#test".into(),
            html: "<div></div>".into(),
        }];
        apply_dom_mutations_full(&mut doc, &m2, None, Some(&mut persistent_nodes), None, None)
            .expect("innerHTML 应用应成功");
        apply_dom_mutations_full(
            &mut doc,
            &[DomMutation::SlotAssign {
                slot_ref: "__n1".into(),
                nodes: vec!["@path:#test|0".into()],
            }],
            None,
            Some(&mut persistent_nodes),
            None,
            None,
        )
        .expect("innerHTML 物化 div 的路径引用应可解析");
        assert!(
            doc.is_has_slotted_element(s),
            "manual 分配 innerHTML 物化 div 后 :has-slotted 应命中"
        );
        let div = doc.get_assigned_nodes(s)[0];
        assert!(
            matches!(
                doc.get(div).map(|n| &n.kind),
                Some(zero_dom::NodeKind::Element(e)) if e.local_name() == "div"
            ),
            "路径 [0] 应命中 innerHTML 物化的新子（旧 Text 已被整体替换）"
        );
    }

    #[test]
    fn k140_island_bind_sr_container_write_through() {
        // shadow-reassign-dynamic-003 链（wptfyi-css k140）：SR 容器 innerHTML 物化
        // slot（引擎 attached）＋shim 岛铸 slot handle（CreateElement detached 岛，
        // t8r `_wcRebuildAsHandle` 产物形）→ BindHandleByPath 重绑 → 岛 handle 跨批
        // 属性写落在引擎真 slot（persistent_nodes 回填保重绑）；岛 handle 作
        // SlotAssign slot_ref 可解析（k139 岛形挂账消除面）。修复前形态：bind 缺位
        // → __n1 恒指 detached 岛 → 属性写岛不可观测（query miss）。
        use crate::js_dom_bridge::apply_dom_mutations_full;
        use std::collections::HashMap;
        let mut doc =
            zero_dom::parse_html("<body><div id=host><span id=slotted>x</span></div></body>");
        let mut persistent_nodes: HashMap<String, zero_dom::NodeId> = HashMap::new();
        let m1 = vec![
            DomMutation::AttachShadow {
                host_selector: "#host".into(),
                root_handle: "__sr0".into(),
                mode: "open".into(),
                delegates_focus: false,
                clonable: false,
                serializable: false,
                slot_assignment: true,
            },
            // 真实 shim 消费序：岛铸（CreateElement）→ 宿主 SetInnerHtml 物化 →
            // bind journal 重绑（同批按序 apply）。
            DomMutation::CreateElement {
                handle: "__n1".into(),
                tag: "slot".into(),
            },
            DomMutation::SetInnerHtmlOnHandle {
                handle: "__sr0".into(),
                html: "<slot name=\"nomatch\" style=\"color:green\"></slot><slot style=\"color:red\"></slot>".into(),
            },
            DomMutation::BindHandleByPath {
                anchor_handle: "__sr0".into(),
                path: vec![0],
                handle: "__n1".into(),
            },
        ];
        apply_dom_mutations_full(&mut doc, &m1, None, Some(&mut persistent_nodes), None, None)
            .expect("SR 容器岛 bind 批应可应用");
        // 批 2：岛 handle 属性写穿透到 attached 真 slot（重绑经 persistent_nodes
        // 跨批保持；引擎解析 slot 带 name/style，岛铸 slot 裸建——命中节点带
        // name="nomatch" 即证为物化真节点而非岛）。
        apply_dom_mutations_full(
            &mut doc,
            &[DomMutation::SetAttrOnHandle {
                handle: "__n1".into(),
                name: "data-probe".into(),
                value: "k140".into(),
            }],
            None,
            Some(&mut persistent_nodes),
            None,
            None,
        )
        .expect("岛 handle 跨批属性写应可应用");
        let probe = doc
            .query_selector_including_shadow("slot[data-probe=k140]")
            .expect("重绑后属性写应命中引擎真 slot（岛写不可观测＝miss）");
        assert!(
            matches!(
                doc.get(probe).map(|n| &n.kind),
                Some(zero_dom::NodeKind::Element(e)) if e.local_name() == "slot"
            ),
            "命中节点应为 slot 元素"
        );
        // 批 3：岛 handle 作 slot_ref 上传——分配 light-DOM span（路径引用滤 SR
        // 位次，#host 引擎子 = [SR, span]）。
        apply_dom_mutations_full(
            &mut doc,
            &[DomMutation::SlotAssign {
                slot_ref: "__n1".into(),
                nodes: vec!["@path:#host|0".into()],
            }],
            None,
            Some(&mut persistent_nodes),
            None,
            None,
        )
        .expect("岛 handle 作 slot_ref 应可解析");
        let assigned = doc.get_assigned_nodes(probe);
        assert_eq!(assigned.len(), 1, "手动表应恰含一个 slottable");
        assert!(
            matches!(
                doc.get(assigned[0]).map(|n| &n.kind),
                Some(zero_dom::NodeKind::Element(e)) if e.local_name() == "span"
            ),
            "分配应命中 light-DOM span（路径 [0] 滤 SR 后）"
        );
    }

    #[test]
    fn k140_island_bind_filters_shadowroot_position() {
        // 宿主容器 bind 的 SR 位次过滤（k140 引擎臂口径）：引擎模型 SR 挂宿主
        // children（child_nodes 含 SR）而 shim 视图 childNodes 无 SR——path "0"
        // 须命中物化 div 而非 SR。修复前形态（原生序 [`walk_child_path`]）：
        // path[0] 命中 SR → 岛 handle 绑到 SR → 属性写落 SR，查询 miss。
        use crate::js_dom_bridge::apply_dom_mutations_full;
        use std::collections::HashMap;
        let mut doc = zero_dom::parse_html("<body></body>");
        let mut persistent_nodes: HashMap<String, zero_dom::NodeId> = HashMap::new();
        let m1 = vec![
            DomMutation::CreateElement {
                handle: "__host".into(),
                tag: "div".into(),
            },
            DomMutation::SetAttrOnHandle {
                handle: "__host".into(),
                name: "id".into(),
                value: "host".into(),
            },
            DomMutation::AppendChild {
                parent_selector: "body".into(),
                child_handle: "__host".into(),
            },
            DomMutation::AttachShadowOnHandle {
                host_handle: "__host".into(),
                root_handle: "__sr0".into(),
                mode: "open".into(),
                delegates_focus: false,
                clonable: false,
                serializable: false,
                slot_assignment: false,
            },
            // 引擎物化后宿主子 = [SR, div]；shim 视图子 = [div]（无 SR）。
            DomMutation::SetInnerHtmlOnHandle {
                handle: "__host".into(),
                html: "<div class=\"inner\"></div>".into(),
            },
            DomMutation::CreateElement {
                handle: "__n9".into(),
                tag: "div".into(),
            },
            DomMutation::BindHandleByPath {
                anchor_handle: "__host".into(),
                path: vec![0],
                handle: "__n9".into(),
            },
            DomMutation::SetAttrOnHandle {
                handle: "__n9".into(),
                name: "id".into(),
                value: "probe".into(),
            },
        ];
        apply_dom_mutations_full(&mut doc, &m1, None, Some(&mut persistent_nodes), None, None)
            .expect("宿主容器岛 bind 批应可应用");
        let probe = doc
            .query_selector_including_shadow("#host div#probe")
            .expect("path[0] 应命中物化 div（SR 未过滤时写落 SR，查询 miss）");
        assert!(
            !matches!(
                doc.get(probe).map(|n| &n.kind),
                Some(zero_dom::NodeKind::ShadowRoot(_))
            ),
            "重绑目标不得是 ShadowRoot 节点"
        );
    }

    #[test]
    fn k140_island_bind_deep_path_element_container() {
        // 元素容器深层 bind（无 SR 零漂移面——r80 clone 既有通道回归守卫）：
        // 锚 handle → "0,1" 下行命中深层 attached 节点；同批序岛铸→物化→重绑→
        // 写穿。修复前岛 handle 写落 detached 岛。
        use crate::js_dom_bridge::apply_dom_mutations_full;
        use std::collections::HashMap;
        let mut doc = zero_dom::parse_html("<body></body>");
        let mut persistent_nodes: HashMap<String, zero_dom::NodeId> = HashMap::new();
        let m1 = vec![
            DomMutation::CreateElement {
                handle: "__box".into(),
                tag: "div".into(),
            },
            DomMutation::AppendChild {
                parent_selector: "body".into(),
                child_handle: "__box".into(),
            },
            DomMutation::CreateElement {
                handle: "__nDeep".into(),
                tag: "b".into(),
            },
            DomMutation::SetInnerHtmlOnHandle {
                handle: "__box".into(),
                html: "<div><i></i><b></b></div>".into(),
            },
            DomMutation::BindHandleByPath {
                anchor_handle: "__box".into(),
                path: vec![0, 1],
                handle: "__nDeep".into(),
            },
            DomMutation::SetAttrOnHandle {
                handle: "__nDeep".into(),
                name: "data-probe".into(),
                value: "deep".into(),
            },
        ];
        apply_dom_mutations_full(&mut doc, &m1, None, Some(&mut persistent_nodes), None, None)
            .expect("元素容器深层岛 bind 批应可应用");
        let probe = doc
            .query_selector_including_shadow("div b[data-probe=deep]")
            .expect("深路径 [0,1] 应命中深层物化 b（岛写不可观测＝miss）");
        assert!(
            matches!(
                doc.get(probe).map(|n| &n.kind),
                Some(zero_dom::NodeKind::Element(e)) if e.local_name() == "b"
            ),
            "命中节点应为深层 b 元素"
        );
    }

    #[test]
    fn k140_island_bind_reslot_after_name_removal() {
        // shadow-reassign-dynamic-003 mutation 层复刻：SR 容器 innerHTML 物化双 slot
        // + 岛 bind + 批 2 removeAttribute(name) → 批末 resolve_slots_all_hosts 应
        // 把 light-DOM span 重解析到首个（现 unnamed）slot。钉 mutation 层引擎链
        // 完整（bind 重绑 attached 节点 + 属性写穿透 + 批末重解析）；slotted 内容
        // computed style 的失效消费面（name 移除→分派变化→级联刷新）是独立挂账，
        // 不在本钉范围。
        use crate::js_dom_bridge::apply_dom_mutations_full;
        use std::collections::HashMap;
        let mut doc =
            zero_dom::parse_html("<body><div id=host><span id=slotted>x</span></div></body>");
        let mut persistent_nodes: HashMap<String, zero_dom::NodeId> = HashMap::new();
        let m1 = vec![
            DomMutation::AttachShadow {
                host_selector: "#host".into(),
                root_handle: "__sr0".into(),
                mode: "open".into(),
                delegates_focus: false,
                clonable: false,
                serializable: false,
                slot_assignment: false,
            },
            DomMutation::CreateElement {
                handle: "__n1".into(),
                tag: "slot".into(),
            },
            DomMutation::SetInnerHtmlOnHandle {
                handle: "__sr0".into(),
                html: "<slot name=\"nomatch\" style=\"color:green\"></slot><slot style=\"color:red\"></slot>".into(),
            },
            DomMutation::BindHandleByPath {
                anchor_handle: "__sr0".into(),
                path: vec![0],
                handle: "__n1".into(),
            },
        ];
        apply_dom_mutations_full(&mut doc, &m1, None, Some(&mut persistent_nodes), None, None)
            .expect("批 1 应可应用");
        apply_dom_mutations_full(
            &mut doc,
            &[DomMutation::RemoveAttrOnHandle {
                handle: "__n1".into(),
                name: "name".into(),
            }],
            None,
            Some(&mut persistent_nodes),
            None,
            None,
        )
        .expect("批 2 removeAttr 应可应用");
        let slot1 = doc
            .query_selector_including_shadow("slot")
            .expect("shadow 树首 slot 应可达");
        let slot1_name = doc
            .get(slot1)
            .map(|n| match &n.kind {
                zero_dom::NodeKind::Element(e) => e.get_attribute("name").unwrap_or_default(),
                _ => "?".into(),
            })
            .unwrap_or_default();
        assert!(slot1_name.is_empty(), "批 2 后首 slot 的 name 应已移除，实际 {slot1_name:?}");
        let assigned = doc.get_assigned_nodes(slot1);
        assert_eq!(assigned.len(), 1, "re-slot 后首 slot 应自动分得 span");
    }
}
