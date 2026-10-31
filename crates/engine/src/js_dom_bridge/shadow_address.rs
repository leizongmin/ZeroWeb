//! s176（html-semantics M2）shadow 域**定位选择器**寻址。从 js_dom_bridge 主文件
//! 独立成模块（行数治理同 selector_match 先例）。
//!
//! 背景：封装查询门（query 路径显式跳过 ShadowRoot 子树）使 shadow 内容拿不到全局
//! 唯一选择器（`unique_selector_for_node` 对 shadow 内容恒 None），shim 枚举通道
//! （shadowRoot children/childNodes/firstChild 族）此前只能靠 detached 镜像——s173/s175
//! 定谳撤下（选择器身份盗用根修 + 镜像对引擎 light-dismiss/hit-test 不可见零增益）。
//! 本模块提供「不依赖全局唯一性」的复合地址串，让 shadow 内容以**普通 selector proxy**
//! 形态进出既有读写通道（mutations/读回调/matches 全经 [`find_by_selector`] 单漏斗）。
//!
//! `use super::*` 复用父模块 find_by_selector / unique_selector_for_node /
//! try_structural_path_fastpath / element_child_index / json_str（子模块可访祖先私有项）。

use super::*;

/// s176：shadow 域定位选择器——`zwsh<len>:<host>|<local>`。
///
/// host 段 = 轻树宿主的既有唯一选择器（[`unique_selector_for_node`]，宿主是 light
/// 元素，s175 可达门通过）；local 段 = shadow 根子树内唯一（[`shadow_unique_local_selector`]）。
/// `len` 为 host 段字节长，前缀切分使段内字符零约束（host 段含 `|`、`:` 亦无歧义）。
/// 嵌套 shadow（宿主在 shadow 树内）暂不支持：宿主唯一选择器从 doc root 解析会被
/// 封装门剪掉（解析不着）→ 返 None；与深 JSON 序列化不下降嵌套 shadow 的既有边界一致。
/// https://dom.spec.whatwg.org/#shadow-trees
pub(crate) fn shadow_scoped_selector(doc: &Document, host: NodeId, node: NodeId) -> Option<String> {
    let host_sel = unique_selector_for_node(doc, host)?;
    let sr = doc.shadow_root(host)?;
    let local = shadow_unique_local_selector(doc, sr, node)?;
    Some(format!("zwsh{}:{}|{}", host_sel.len(), host_sel, local))
}

/// s176：local 段——在 shadow 根子树内唯一的轻量选择器。阶梯与
/// [`unique_selector_for_node`] 同形，唯一性域收窄为 `query_selector_all(sr, cand)`
/// （同域解析+判重：解析走 [`try_shadow_scoped_fastpath`] 的 shadow 根查询，两侧语义
/// 一致）。阶梯：`#id` → `tag.class`（首 class，[`stable_selector_for_node`] 形态）→
/// 裸 `tag` → 位置链 `tag:nth-child(i) > …`（元素序 1-based，与
/// [`structural_path_selector`] 同约定；文本/注释不计位）。穿入嵌套 ShadowRoot 节点
/// （非元素、不在 sr 查询域）→ 拒识返 None。
fn shadow_unique_local_selector(doc: &Document, sr: NodeId, node: NodeId) -> Option<String> {
    let tag = match &doc.get(node)?.kind {
        NodeKind::Element(e) => e.local_name().to_string(),
        _ => return None,
    };
    let unique_in_sr = |sel: &str| doc.query_selector_all(sr, sel).len() == 1;
    if let Some(id) = doc.get_attribute(node, "id").filter(|v| !v.is_empty()) {
        let sel = format!("#{id}");
        if unique_in_sr(&sel) {
            return Some(sel);
        }
    }
    if let Some(first) = doc
        .get_attribute(node, "class")
        .filter(|v| !v.is_empty())
        .and_then(|cls| cls.split_whitespace().next().map(str::to_string))
    {
        let sel = format!("{tag}.{first}");
        if unique_in_sr(&sel) {
            return Some(sel);
        }
    }
    if unique_in_sr(&tag) {
        return Some(tag);
    }
    // 位置链：自 node 上行至 sr。
    let mut segments: Vec<String> = Vec::new();
    let mut cur = node;
    loop {
        let parent = doc.get(cur)?.parent?;
        let pos = element_child_index(doc, parent, cur)?;
        let seg_tag = match &doc.get(cur)?.kind {
            NodeKind::Element(e) => e.local_name().to_string(),
            _ => return None,
        };
        segments.push(format!("{seg_tag}:nth-child({pos})"));
        if parent == sr {
            break;
        }
        // 嵌套 ShadowRoot 节点：不在 sr 查询域（封装门剪嵌套子树），local 段不可达。
        if matches!(doc.get(parent).map(|n| &n.kind), Some(NodeKind::ShadowRoot(_))) {
            return None;
        }
        cur = parent;
    }
    segments.reverse();
    Some(segments.join(" > "))
}

/// s176：shadow 域定位选择器快速通道——识别 `zwsh<len>:<host>|<local>` 并两段解析
/// （host 走轻树既有解析：structural fastpath + 根查询；local 在 shadow 根子树查询，
/// 与 [`shadow_unique_local_selector`] 的唯一性域同语义）。三层返回：`None` = 非本
/// grammar（调用方走后续通道）；`Some(None)` = 识别但不可解析（host 失配/shadow 缺
/// 失/local 失配）——**不回落通用引擎**（compound 串非合法 CSS，通用解析误匹配）；
/// `Some(Some(id))` = 命中。
pub(crate) fn try_shadow_scoped_fastpath(doc: &Document, selector: &str) -> Option<Option<NodeId>> {
    let rest = selector.strip_prefix("zwsh")?;
    let colon = rest.find(':')?;
    let len: usize = rest[..colon].parse().ok()?;
    let after = &rest[colon + 1..];
    if after.len() < len + 1 {
        return Some(None);
    }
    let (host_sel, local) = after.split_at(len);
    let local = local.strip_prefix('|')?;
    let host = match try_structural_path_fastpath(doc, host_sel.trim())
        .or_else(|| doc.query_selector(doc.root(), host_sel.trim()))
    {
        Some(h) => h,
        None => return Some(None),
    };
    let Some(sr) = doc.shadow_root(host) else {
        return Some(None);
    };
    let local = local.trim();
    // 位置链段（`tag:nth-child(k)`）：CSS 求值经 `parent_element_node`（跳过
    // ShadowRoot 非元素父），SR 直下元素的位次被算进宿主域——失真。改按元素序索引
    // 直查（与 try_structural_path_fastpath 同范式；minting 端 element_child_index
    // 同计数口径，两侧对称）。
    if local.contains(":nth-child(") {
        return Some(resolve_local_positional(doc, sr, local));
    }
    Some(doc.query_selector(sr, local))
}

/// s176：位置链 local 段的元素序索引解析——自 sr 按段下行，每段取第 k 个元素子并
/// 校验 tag（`:nth-child` 元素序 1-based 口径，文本/注释不计位）。
fn resolve_local_positional(doc: &Document, sr: NodeId, local: &str) -> Option<NodeId> {
    let mut cur = sr;
    for seg in local.split(" > ") {
        let seg = seg.trim();
        let paren = seg.rfind(":nth-child(")?;
        let tag = &seg[..paren];
        let pos: usize = seg[paren + ":nth-child(".len()..].strip_suffix(')')?.parse().ok()?;
        if tag.is_empty() || pos == 0 {
            return None;
        }
        let nd = doc.get(cur)?;
        let mut idx = 0usize;
        let mut hit = None;
        for &c in nd.children.iter() {
            if let Some(NodeKind::Element(e)) = doc.get(c).map(|n| &n.kind) {
                idx += 1;
                if idx == pos {
                    if e.local_name().eq_ignore_ascii_case(tag) {
                        hit = Some(c);
                    }
                    break;
                }
            }
        }
        cur = hit?;
    }
    Some(cur)
}

/// s176：元素所在**树作用域**根——父链遇 ShadowRoot 节点即该 shadow 根，否则 doc
/// root。供 matches 族匹配集求值域（spec：matches 按元素 root 的 tree 求值——shadow
/// 内容的匹配集从 doc root 求会被封装门剪空，compound 定位寻址进 shadow 后匹配集
/// 必须换域）。
/// https://dom.spec.whatwg.org/#dom-element-matches
pub(crate) fn shadow_scope_root(doc: &Document, node: NodeId) -> NodeId {
    let mut cur = node;
    loop {
        if cur == doc.root() {
            return doc.root();
        }
        let nd = doc.get(cur);
        match nd {
            None => return doc.root(),
            Some(n) => {
                if let NodeKind::ShadowRoot(_) = &n.kind {
                    return cur;
                }
            }
        }
        match doc.get(cur).and_then(|n| n.parent) {
            Some(p) => cur = p,
            None => return doc.root(),
        }
    }
}

/// s176：child_nodes 条目的 shadow 内容寻址——父链上溯首遇 ShadowRoot 节点取其宿主
/// 铸 compound 定位选择器（[`node_entry_json`] 元素条目 `s` 字段的 shadow 回落）。
/// 宿主自身在 shadow 树内（嵌套 shadow，唯一选择器从 doc root 解析不着）→ None，
/// 条目照旧省略。文本/注释无 tag，调用方分支先行。
pub(crate) fn shadow_content_entry_selector(doc: &Document, node: NodeId) -> Option<String> {
    let mut cur = node;
    loop {
        if cur == doc.root() {
            return None;
        }
        let parent = doc.get(cur)?.parent?;
        if let Some(NodeKind::ShadowRoot(d)) = doc.get(parent).map(|n| &n.kind) {
            let host = d.host?;
            // 嵌套 shadow 守卫：宿主 parent 链穿 ShadowRoot → 地址从 doc root 解析不着。
            let mut h = host;
            loop {
                if h == doc.root() {
                    break;
                }
                let hp = doc.get(h)?.parent?;
                if matches!(doc.get(hp).map(|n| &n.kind), Some(NodeKind::ShadowRoot(_))) {
                    return None;
                }
                h = hp;
            }
            return shadow_scoped_selector(doc, host, node);
        }
        cur = parent;
    }
}

/// s176：声明式 shadow 深条目（带 sel）——形态同主文件 [`deep_entry_json`]，元素条
/// 目增 `"sel"` 字段（[`shadow_scoped_selector`] 地址串；取不到 local 段的元素——嵌
/// 套 shadow 内容——省略 sel，shim 内容构建只代理带 sel 条目）。镜像构建消费同一
/// 回调、只读 k/tag/attrs/children，多余字段零影响。
pub(crate) fn ds_scoped_entry(doc: &Document, id: NodeId, host: NodeId) -> Option<String> {
    let node = doc.get(id)?;
    match &node.kind {
        NodeKind::Text(t) => Some(format!("{{\"k\":\"T\",\"v\":{}}}", json_str(&t.content))),
        NodeKind::Comment(c) => Some(format!("{{\"k\":\"C\",\"v\":{}}}", json_str(&c.content))),
        NodeKind::Element(e) => {
            let tag = json_str(e.local_name());
            let sel_field = match shadow_scoped_selector(doc, host, id) {
                Some(s) => format!("\"sel\":{},", json_str(&s)),
                None => String::new(),
            };
            // 属性发限定名（prefix:local）——同 deep_entry_json 口径（foreign context
            // 片段 prefix 保全，2026-10-28 encoding-legacy-mb M1 对账修复）。
            let attrs: Vec<String> = e
                .attributes
                .iter()
                .map(|a| {
                    let qname = match a.name.prefix.as_deref() {
                        Some(p) if !p.is_empty() => format!("{}:{}", p, a.name.local.as_ref()),
                        _ => a.name.local.to_string(),
                    };
                    format!("{{\"n\":{},\"v\":{}}}", json_str(&qname), json_str(&a.value))
                })
                .collect();
            let kids_src = match doc.template_contents_children(id) {
                tpl_kids if !tpl_kids.is_empty() => tpl_kids,
                _ => doc.child_nodes(id),
            };
            let kids: Vec<String> = kids_src.iter().filter_map(|&k| ds_scoped_entry(doc, k, host)).collect();
            Some(format!(
                "{{\"k\":\"E\",{}\"tag\":{},\"attrs\":[{}],\"children\":[{}]}}",
                sel_field,
                tag,
                attrs.join(","),
                kids.join(",")
            ))
        }
        _ => None,
    }
}
