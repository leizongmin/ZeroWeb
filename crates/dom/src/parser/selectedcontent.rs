//! selectedcontent 解析期初始化（s324，html-semantics M2 切片）。
//!
//! spec：HTML Standard #the-selectedcontent-element（whatwg/html #9799 谱系）——
//! `<selectedcontent>` 反映所属 select 当前选中 option 的内容快照。
//!
//! 文档解析完成后对每个 select 元素运行一次初始化：按**解析期 selectedness 模型**
//! （dirtiness 恒 false，selectedness 全由 `selected` 内容属性 + setting 回落推导）
//! 找出选中 option，把其子节点深克隆进该 select 的每个 **enabled** selectedcontent
//! 后代（replace-all 语义：先清空后追加克隆）；无选中 option 时清空。
//!
//! 时点等价性：本引擎内联脚本随整篇解析完成后执行（无 mid-parse 脚本观察面），
//! 解析后单次重建与 spec 的逐 option pop「maybe clone an option into selectedcontent」
//! 增量时点不可区分。html5ever 0.38 的 TreeSink pop/maybe_clone 钩对隐式闭合
//! （无 `</option>`）不触达（html5ever#712，且 `</select>` 直落 `pop_until` 旁路
//! `sink.pop`），后处理单遍绕开该缺口。
//!
//! spec 差异（corpus 判据优先，见 s323/s324 evidence 账本）：spec「update a select's
//! selectedcontent」只更新首个 enabled selectedcontent（get a select's enabled
//! selectedcontent 取首项）并另有「clear non-primary」清非首项；语料
//! （selectedcontent.html test2 / mutations）钉**全部** enabled selectedcontent 同步
//! 保持 → 本模块与 shim 运行期一致按全量 enabled selectedcontent 施克隆。
//! multiple select：语料（mutations test6 末态 html）钉全体 enabled selectedcontent
//! **清空**（非 spec「null 不施」），与 shim 运行期同判据。
//!
//! enabled 判定（selectedcontent post-connection steps 镜像）：selectedcontent 的
//! 祖先链上出现 option / selectedcontent 祖先，或第二个 select 祖先 → disabled（跳过）。

use super::HTML_NAMESPACE;
use crate::document::Document;
use crate::node::{NodeData, NodeKind};

/// 文档解析后运行：对每个 select 元素按解析期模型初始化其 enabled selectedcontent
/// 后代。全文档无 selectedcontent 元素时零工作（热路径零回归门）。
pub(crate) fn initialize_selects(doc: &mut Document) {
    let root = doc.root();
    let mut has_selectedcontent = false;
    for id in doc.collect_descendants(root) {
        if is_html_tag(doc, id, "selectedcontent") {
            has_selectedcontent = true;
            break;
        }
    }
    if !has_selectedcontent {
        return;
    }

    // collect_descendants 树序——select 按出现顺序处理；嵌套 select 在新解析规则下
    // 内层 select 起始标签闭合外层（外层已处理过，内层以自身子树重复处理，幂等）。
    let all = doc.collect_descendants(root);
    for id in all {
        if !is_html_tag(doc, id, "select") {
            continue;
        }
        // multiple：语料判据（mutations test6 末态 html）钉全体 enabled sc **清空**
        // （Chromium update 面清除初始内容）——非 spec「null 不施」；selected=None
        // 走清空面，与 shim 运行期同判据。
        let selected = if has_attribute(doc, id, "multiple") {
            None
        } else {
            selected_option(doc, id)
        };
        for sc in enabled_selectedcontent_descendants(doc, id) {
            match selected {
                Some(opt) => replace_children_with_clones_of(doc, opt, sc),
                None => clear_children(doc, sc),
            }
        }
    }
}

/// 判定 `id` 是否为 HTML 命名空间、给定 local name 的元素。
fn is_html_tag(doc: &Document, id: crate::node::NodeId, local: &str) -> bool {
    matches!(doc.get(id), Some(NodeData { kind: NodeKind::Element(elem), .. })
        if elem.namespace() == HTML_NAMESPACE && elem.local_name() == local)
}

/// 属性存在性（HTML 文档属性名 case-insensitive 已由 `has_attribute` 处理）。
fn has_attribute(doc: &Document, id: crate::node::NodeId, name: &str) -> bool {
    matches!(doc.get(id), Some(NodeData { kind: NodeKind::Element(elem), .. })
        if elem.has_attribute(name))
}

/// spec「the list of options」收集（get the list of options 步镜像）：树序遍历
/// select 后代；option 收集；遇 select/hr/option/datalist 或嵌套 optgroup（祖先链上
/// 已有 optgroup）时**跳过其子树**。
/// https://html.spec.whatwg.org/multipage/form-elements.html#concept-select-options
fn list_of_options(doc: &Document, select: crate::node::NodeId) -> Vec<crate::node::NodeId> {
    let mut out = Vec::new();
    // 根 select 自身不整枝（skip 规则只作用于后代——嵌套 select/hr/option/datalist）。
    let children = doc.child_nodes(select);
    for child in children {
        walk_list_of_options(doc, child, false, &mut out);
    }
    out
}

fn walk_list_of_options(
    doc: &Document,
    node: crate::node::NodeId,
    in_optgroup: bool,
    out: &mut Vec<crate::node::NodeId>,
) {
    let Some(NodeData {
        kind: NodeKind::Element(elem),
        children,
        ..
    }) = doc.get(node)
    else {
        return;
    };
    if elem.namespace() != HTML_NAMESPACE {
        // 非 HTML 命名空间子树不产 option、也不设跳过边界（spec 按 HTML 元素名判定）。
        for &child in children {
            walk_list_of_options(doc, child, in_optgroup, out);
        }
        return;
    }
    let local = elem.local_name();
    let is_option = local == "option";
    let skip_descendants =
        local == "select" || local == "hr" || is_option || local == "datalist" || (local == "optgroup" && in_optgroup);
    if is_option {
        out.push(node);
    }
    if skip_descendants {
        return;
    }
    let child_in_optgroup = in_optgroup || local == "optgroup";
    let children = children.clone();
    for child in children {
        walk_list_of_options(doc, child, child_in_optgroup, out);
    }
}

/// 解析期选中 option（selectedness setting 算法解析面）：
/// 1. 列表存在 `selected` 属性 option → 树序**最后一项**胜（setting 步 2 all-but-last
///    清假的净效果；disabled 不影响 selectedness，照选中克隆）。
/// 2. 无属性选中且 display size 1（非 multiple 缺省 1）→ 首个非 disabled option
///    （setting 步 1 回落）。
/// 3. 都不满足 → None（selectedcontent 清空）。
///
/// https://html.spec.whatwg.org/multipage/form-elements.html#selectedness-setting-algorithm
fn selected_option(doc: &Document, select: crate::node::NodeId) -> Option<crate::node::NodeId> {
    let list = list_of_options(doc, select);
    if list.is_empty() {
        return None;
    }
    if let Some(&last) = list.iter().rev().find(|&&opt| has_attribute(doc, opt, "selected")) {
        return Some(last);
    }
    if display_size(doc, select) != 1 {
        return None;
    }
    list.into_iter().find(|&opt| !is_option_disabled(doc, opt))
}

/// display size（spec：size 属性按非负整数解析且须 >0；无效/缺省 → 非 multiple 为 1）。
fn display_size(doc: &Document, select: crate::node::NodeId) -> usize {
    let Some(NodeData {
        kind: NodeKind::Element(elem),
        ..
    }) = doc.get(select)
    else {
        return 1;
    };
    match elem.get_attribute_str("size") {
        Some(v) => match v.trim().parse::<usize>() {
            Ok(n) if n > 0 => n,
            _ => 1,
        },
        None => 1,
    }
}

/// option disabled 判定（spec option disabled 算法解析面）：自身 `disabled` 属性，
/// 或祖先链（逆树序）上 optgroup 带 `disabled`；select/hr/datalist/option 祖先截断。
/// https://html.spec.whatwg.org/multipage/form-elements.html#concept-option-disabled
fn is_option_disabled(doc: &Document, option: crate::node::NodeId) -> bool {
    if has_attribute(doc, option, "disabled") {
        return true;
    }
    let mut cur = doc.parent_node(option);
    while let Some(id) = cur {
        match doc.get(id) {
            Some(NodeData {
                kind: NodeKind::Element(elem),
                ..
            }) if elem.namespace() == HTML_NAMESPACE => match elem.local_name() {
                "select" | "hr" | "datalist" | "option" => return false,
                "optgroup" if elem.has_attribute("disabled") => return true,
                _ => {}
            },
            _ => {}
        }
        cur = doc.parent_node(id);
    }
    false
}

/// select 的 enabled selectedcontent 后代（树序全量——spec 差异见模块文档）。
/// disabled 判定（selectedcontent post-connection steps 镜像）：祖先链上第二个
/// select、或 option / selectedcontent 祖先。
fn enabled_selectedcontent_descendants(doc: &Document, select: crate::node::NodeId) -> Vec<crate::node::NodeId> {
    let mut out = Vec::new();
    for id in doc.collect_descendants(select) {
        if is_html_tag(doc, id, "selectedcontent") && !is_selectedcontent_disabled(doc, id) {
            out.push(id);
        }
    }
    out
}

fn is_selectedcontent_disabled(doc: &Document, sc: crate::node::NodeId) -> bool {
    let mut cur = doc.parent_node(sc);
    let mut select_seen = false;
    while let Some(id) = cur {
        if is_html_tag(doc, id, "select") {
            if select_seen {
                return true; // 第二个 select 祖先
            }
            select_seen = true;
        } else if is_html_tag(doc, id, "option") || is_html_tag(doc, id, "selectedcontent") {
            return true;
        }
        cur = doc.parent_node(id);
    }
    false
}

/// 「clone an option into a selectedcontent」解析面：先清空 sc 现有子，再逐子
/// 深克隆（`Document::clone_node`）追加。克隆节点 id 不入 id_map（与运行期 cloneNode
/// 同口径，调用方克隆后自行赋新 id——解析期克隆持重复 id，语料按位置查询不按 id）。
/// https://html.spec.whatwg.org/multipage/form-elements.html#clone-an-option-into-a-selectedcontent
fn replace_children_with_clones_of(doc: &mut Document, option: crate::node::NodeId, sc: crate::node::NodeId) {
    clear_children(doc, sc);
    let children = doc.child_nodes(option);
    for child in children {
        let clone = doc.clone_node(child, true);
        if let Some(nd) = doc.get_mut(sc) {
            nd.children.push(clone);
        }
        if let Some(cd) = doc.get_mut(clone) {
            cd.parent = Some(sc);
        }
    }
}

/// 「clear a selectedcontent」解析面：清空全部子节点并断开父链。
fn clear_children(doc: &mut Document, sc: crate::node::NodeId) {
    let old = match doc.get_mut(sc) {
        Some(nd) => std::mem::take(&mut nd.children),
        None => return,
    };
    for child in old {
        if let Some(cd) = doc.get_mut(child) {
            cd.parent = None;
        }
    }
}
