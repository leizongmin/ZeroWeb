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
}
