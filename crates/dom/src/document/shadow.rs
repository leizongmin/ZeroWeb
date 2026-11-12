//! Document Shadow DOM 操作 —— 拆自 `mod.rs`（rule 5 单文件 <2000 行，R3164）。
//!
//! 本模块为 [`super::Document`] 的 Shadow DOM 面（attach_shadow / shadow_root / slot 分配 /
//! shadow 子树查询）。作为 `document` 模块的**子模块**，可访问 [`super::Document`] 的私有字段
//!（`nodes` / `shadow_roots` / `slot_assignments` / `manual_slot_assignments`）与 `mod.rs` 的私有查询助手
//!（`collect_slot_elements` / `find_first_matching_shadow` / `collect_matching_shadow`）——Rust 隐私规则：
//! 私有项对定义模块及其后代可见，故无需任何可见性改动（行为不变重组）。

use crate::node::{NodeData, NodeId, NodeKind, ShadowRootData, ShadowRootMode, SlotAssignment};

use super::Document;
use super::DomError;

/// valid shadow host 名（词汇法）：valid custom element name 或基本元素名单。
///
/// 45 号 M3 前名单只含基本元素；连字符宿主（`inner-host` 等 custom element 名
/// 形态）同为 valid shadow host（命令式 attachShadow shim 侧同款 ASCII 词汇法——
/// 小写字母开头、含连字符、`[a-z0-9._-]` 字符集；HTML spec 保留名不算 valid）。
/// 词汇法判定即可，无需自定义元素注册面。
/// https://dom.spec.whatwg.org/#valid-shadow-host-name
/// https://html.spec.whatwg.org/multipage/custom-elements.html#valid-custom-element-name
fn is_valid_shadow_host_name(name: &str) -> bool {
    // 基本元素名单（dom.spec whatwg #valid-shadow-host-name）。
    const BASIC_VALID_SHADOW_HOST_NAMES: &[&str] = &[
        "article",
        "aside",
        "blockquote",
        "body",
        "div",
        "footer",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "header",
        "main",
        "nav",
        "p",
        "section",
        "span",
    ];
    if BASIC_VALID_SHADOW_HOST_NAMES.contains(&name) {
        return true;
    }
    is_valid_custom_element_name_ascii(name)
}

/// valid custom element name 的 ASCII 词汇法（镜像 shim `_attachShadow` 校验形态）：
/// `[a-z]` 开头、`[a-z0-9._-]` 组成、含至少一个连字符（非首字符），且不在 HTML
/// spec 保留名排除表内（PCENChar 的非 ASCII 段不在此列，shim 同口径只放 ASCII）。
fn is_valid_custom_element_name_ascii(name: &str) -> bool {
    // HTML spec #valid-custom-element-name 排除的保留名。
    // https://html.spec.whatwg.org/multipage/custom-elements.html#valid-custom-element-name
    const RESERVED_CUSTOM_ELEMENT_NAMES: &[&str] = &[
        "annotation-xml",
        "color-profile",
        "font-face",
        "font-face-src",
        "font-face-uri",
        "font-face-format",
        "font-face-name",
        "missing-glyph",
    ];
    if RESERVED_CUSTOM_ELEMENT_NAMES.contains(&name) {
        return false;
    }
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {}
        _ => return false,
    }
    let rest = chars.as_str();
    if !rest
        .bytes()
        .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_'))
    {
        return false;
    }
    rest.contains('-')
}

impl Document {
    /// 为宿主元素附加 ShadowRoot（全参形态，s180/gethtml Batch 2）。
    ///
    /// 与 [`Self::attach_shadow`] 相同的校验与挂接，另写入四个状态位：
    /// `delegates_focus` / `clonable` / `serializable`（命令式来自 attachShadow
    /// init，声明式来自三个 `shadowroot*` 布尔属性存在性）、`slot_assignment`
    ///（命令式来自 init `slotAssignment` 枚举；声明式来自
    /// `shadowrootslotassignment` 枚举，缺失/invalid 默认 Named）与 `declarative`
    ///（声明式物化产物标记——序列化/getHTML 回写模板形态的资格）。
    // s180/gethtml 全参形态沿用（canvas/path.rs 等既有先例）——参数均为独立
    // 状态位，收敛成 struct 反而模糊调用面。
    #[allow(clippy::too_many_arguments)]
    pub fn attach_shadow_full(
        &mut self,
        host: NodeId,
        mode: ShadowRootMode,
        delegates_focus: bool,
        clonable: bool,
        serializable: bool,
        slot_assignment: SlotAssignment,
        declarative: bool,
    ) -> Result<NodeId, DomError> {
        // 验证宿主是元素节点
        let is_element = self
            .nodes
            .get(host)
            .map(|n| matches!(n.kind, NodeKind::Element(_)))
            .unwrap_or(false);
        if !is_element {
            return Err(DomError::NotAnElement);
        }

        // 验证宿主没有已有 ShadowRoot
        if self.shadow_roots.contains_key(&host) {
            return Err(DomError::AlreadyHasShadowRoot);
        }

        // 创建 ShadowRoot 节点（状态位随创建写入，宿主引用稍后回填）
        let mut shadow_data = ShadowRootData::new(mode);
        shadow_data.delegates_focus = delegates_focus;
        shadow_data.clonable = clonable;
        shadow_data.serializable = serializable;
        shadow_data.slot_assignment = slot_assignment;
        shadow_data.declarative = declarative;
        let shadow_id = self.nodes.insert(NodeData::new(NodeKind::ShadowRoot(shadow_data)));

        // 设置宿主引用
        if let Some(NodeKind::ShadowRoot(data)) = self.nodes.get_mut(shadow_id).map(|n| &mut n.kind) {
            data.host = Some(host);
        }

        // 将 ShadowRoot 作为宿主的子节点追加
        // （ShadowRoot 存在于宿主的子列表中，但封装边界阻止外部查询穿透）
        // children 结构变化 → 位置索引全清（失效契约见 mod.rs invalidate_query_pos_index）。
        self.invalidate_query_pos_index();
        if let Some(host_data) = self.nodes.get_mut(host) {
            host_data.children.push(shadow_id);
        }
        if let Some(shadow_data) = self.nodes.get_mut(shadow_id) {
            shadow_data.parent = Some(host);
        }

        // 注册映射
        self.shadow_roots.insert(host, shadow_id);

        Ok(shadow_id)
    }

    /// 为宿主元素附加 ShadowRoot（仅 mode，其余状态位全 false——既有命令式
    /// attachShadow 调用面的兼容入口）。
    pub fn attach_shadow(&mut self, host: NodeId, mode: ShadowRootMode) -> Result<NodeId, DomError> {
        self.attach_shadow_full(host, mode, false, false, false, SlotAssignment::Named, false)
    }

    /// 获取宿主元素附加的 ShadowRoot，如果没有则返回 `None`。
    pub fn shadow_root(&self, host: NodeId) -> Option<NodeId> {
        self.shadow_roots.get(&host).copied()
    }

    /// s168：全部 ShadowRoot 节点（创建序 = slotmap 插入序，稳定）。供 engine handle
    /// 视图的 shadow 包含回落查询（gCS by handle 是对象身份通道——页面持元素对象
    /// 访问 shadow 内容合法，不受 light DOM 查询封装约束）。
    pub fn shadow_root_ids(&self) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter(|(_, n)| matches!(n.kind, NodeKind::ShadowRoot(_)))
            .map(|(id, _)| id)
            .collect()
    }

    /// s168：shadow 包含的选择器查询——light 域先查（复用封装版 [`Self::query_selector`]），
    /// miss 后逐 shadow root 子树查询（rooting 在 shadow root 上，封装门不剪根自身）。
    /// 嵌套 shadow（shadow 树内再 attach）不在本遍历面（FIXME 级联切片）。
    pub fn query_selector_including_shadow(&self, selector: &str) -> Option<NodeId> {
        if let Some(hit) = self.query_selector(self.root(), selector) {
            return Some(hit);
        }
        for sid in self.shadow_root_ids() {
            if let Some(hit) = self.query_selector(sid, selector) {
                return Some(hit);
            }
        }
        None
    }

    /// s168：声明式 Shadow DOM 重建——`<template shadowrootmode=open|closed>`（本仓
    /// 序列化面产物；页面直写的声明式形态同样适用）解析后还原为 ShadowRoot 节点：
    /// mode 属性合法时在宿主（template 的父元素）attach_shadow、contents 子移入
    /// shadow 树、摘除 template 元素。属性缺省/非法 → 保留普通 template（spec：
    /// 无有效 shadowrootmode 按普通 template 处理）。
    ///
    /// M3 清单 1（attachment 170F 勘根）：候选枚举自 [`Document::get_elements_by_tag_name`]
    /// 改为本函数私有遍历——**下探 template contents**（嵌套声明族 + 外层 template
    /// 包裹声明式模板案的枚举盲区：contents 内声明式模板按 spec 解析期即 attach，
    /// [`collect_by_tag_name`](Document::collect_by_tag_name) 只走 children 边
    /// 不可达）。shadow 树内层不枚举：声明式 attach 仅解析期生效（spec），已挂载
    /// shadow 树中的 template 无 attach 面，跳过即 spec 行为。
    /// https://html.spec.whatwg.org/multipage/scripting.html#the-template-element
    /// https://dom.spec.whatwg.org/#valid-shadow-host-name
    pub fn materialize_declarative_shadow_roots(&mut self) {
        let mut templates = Vec::new();
        self.collect_declarative_template_candidates(self.root, &mut templates);
        for tid in templates {
            // s173 配套值还原（declarative_shadow 扩面）：解析前预变换把 open/close
            // 值改写为首字母大写以出 html5ever 0.29.1 精确匹配集；被本函数**消费**的
            // template 随物化移除（值不可观测），但被**跳过**的 template（contents
            // 缺失/宿主非法/attach Err/模式非法）以普通元素存活——改写值经
            // getAttribute/序列化可观测，须还原源码小写形态（WPT opt-in 断言
            // `getAttribute === 'open'`；查询路径 serialization→re-parse 场
            // `query_selector('template')` 命中跳过模板实证）。还原先行于模式匹配：
            // 消费路径随后摘除 template，还原无观测面。
            if let Some(v) = self.get_attribute(tid, "shadowrootmode") {
                let restored = match v.as_str() {
                    "Open" => Some("open"),
                    "Close" => Some("close"),
                    _ => None,
                };
                if let Some(restored) = restored {
                    self.set_attribute(tid, "shadowrootmode", restored);
                }
            }
            let Some(mode_str) = self
                .get_attribute(tid, "shadowrootmode")
                .map(|s| s.to_ascii_lowercase())
            else {
                continue;
            };
            let mode = match mode_str.as_str() {
                "open" => ShadowRootMode::Open,
                "closed" => ShadowRootMode::Closed,
                _ => continue,
            };
            let Some(host) = self.parent_node(tid) else {
                continue;
            };
            if !matches!(self.nodes.get(host).map(|n| &n.kind), Some(NodeKind::Element(_))) {
                continue;
            }
            // M3 簇 C（attachment safelist）：声明式物化仅在 valid shadow host 名单
            // 元素上生效——名单外宿主（void/table 上下文等）的 template 保留普通
            // 形态，其 shadowrootmode 值已被上方 s173 还原为源码小写（WPT
            // attachment.html disallowed 臂：template 保留 + getAttribute 断言面）。
            // 45 号 M3（css-scoping-shadow-slotted-nested）：custom element 名宿主
            //（连字符）同为 valid shadow host——词汇法判定即可，无需注册面。
            // https://dom.spec.whatwg.org/#valid-shadow-host-name
            let host_valid = self
                .nodes
                .get(host)
                .map(|n| match &n.kind {
                    NodeKind::Element(elem) => is_valid_shadow_host_name(elem.local_name()),
                    _ => false,
                })
                .unwrap_or(false);
            if !host_valid {
                continue;
            }
            let Some(frag) = self.template_contents(tid) else {
                continue;
            };
            // s180/gethtml（M3 Batch 2）：三个 shadowroot* 按 HTML 布尔属性语义
            // 取存在性（存在即真，值不参与），物化产物标 declarative=true——
            // 序列化回写与 getHTML({serializableShadowRoots}) 资格标记；
            // shadowrootreferencetarget 按值捕获（见下）。
            let delegates_focus = self.get_attribute(tid, "shadowrootdelegatesfocus").is_some();
            let clonable = self.get_attribute(tid, "shadowrootclonable").is_some();
            let serializable = self.get_attribute(tid, "shadowrootserializable").is_some();
            // M3 手动臂：`shadowrootslotassignment` 枚举捕获（named/manual；缺失与
            // invalid value default 均 Named）。html5ever 0.29.1 对该属性无值匹配
            // 缺陷（s173 预变换仅 shadowrootmode 面），原值存活可直读。
            // https://html.spec.whatwg.org/multipage/scripting.html#the-template-element
            let slot_assignment = match self
                .get_attribute(tid, "shadowrootslotassignment")
                .as_deref()
                .map(str::to_ascii_lowercase)
                .as_deref()
            {
                Some("manual") => SlotAssignment::Manual,
                _ => SlotAssignment::Named,
            };
            // M3 簇 C（referenceTarget）：`shadowrootreferencetarget` 值原样捕获（空值属性
            // =空串；属性序无关），序列化按 spec 序回写。
            // https://dom.spec.whatwg.org/#shadowroot-reference-target
            let reference_target = self
                .get_attribute(tid, "shadowrootreferencetarget")
                .map(|s| s.to_string());
            let Ok(root) = self.attach_shadow_full(
                host,
                mode,
                delegates_focus,
                clonable,
                serializable,
                slot_assignment,
                true,
            ) else {
                continue;
            };
            if let Some(target) = reference_target
                && let Some(NodeKind::ShadowRoot(data)) = self.nodes.get_mut(root).map(|n| &mut n.kind)
            {
                data.reference_target = Some(target);
            }
            let kids = self.nodes.get(frag).map(|n| n.children.clone()).unwrap_or_default();
            for k in kids {
                let _ = self.append_child(root, k);
            }
            let _ = self.remove_child(host, tid);
            // s176（html-semantics M2）：物化后即解析该宿主的 slot 分配——布局树
            // shadow 分支经 `get_assigned_nodes` 消费分配表替换 slot；此前
            // `resolve_slots` 无任何调用点，分配恒空 → slot 全走回退内容 → 被分配的
            // light 子树（含嵌套宿主链）无布局盒（gBCR 零 rect、命中测试不可达——
            // closedby-complex shadow 点击坐标塌 (0,0) 根因）。
            // 45 号 M3 R3 变异应用臂已收口命令式面：attachShadow 与宿主 childList/
            // slot 属性变异经 `apply_dom_mutations_full` 批末统一重解析
            //（[`Self::resolve_slots_all_hosts`]）——此处保留物化时点即解析（解析
            // 幂等，物化批与变异批语义一致）。
            self.resolve_slots(host);
        }
        // ZeroWeb 内部通道（M3 手动臂）：`data-zw-manual-assigned` marker 携带
        // `assign()` 分配关系过 serialize→re-parse 边界（spec 无声明式对影——
        // manually assigned nodes 不落序列化产物，见
        // [`Self::manual_assigned_slot_marker`]），物化把 slot 归位 shadow 树后
        // 在此还原手动表并摘除 marker。
        self.restore_manual_slot_assignments();
    }

    /// 声明式物化候选枚举——镜像 [`Document::collect_by_tag_name`](Document::collect_by_tag_name)
    /// 的 children 边遍历 + shadow 树跳过，额外下探 template contents（`shadow.rs`
    /// 模块文档注：contents 是惰性文档片段，不属 children 边）。快照先行于物化：
    /// 消费路径会摘除 template/移动子树，枚举一次完成，NodeId 稳定不受重挂影响。
    fn collect_declarative_template_candidates(&self, id: NodeId, result: &mut Vec<NodeId>) {
        let node_data = match self.nodes.get(id) {
            Some(n) => n,
            None => return,
        };
        let is_template =
            matches!(&node_data.kind, NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("template"));
        if is_template {
            result.push(id);
        }
        for &child in &node_data.children {
            if self.is_shadow_root_node(child) {
                continue;
            }
            self.collect_declarative_template_candidates(child, result);
        }
        if is_template
            && let Some(frag) = self.template_contents(id)
            && let Some(frag_node) = self.nodes.get(frag)
        {
            for &child in &frag_node.children {
                if self.is_shadow_root_node(child) {
                    continue;
                }
                self.collect_declarative_template_candidates(child, result);
            }
        }
    }

    /// 获取 ShadowRoot 的封装模式。
    ///
    /// 如果节点不是 ShadowRoot，返回 `None`。
    pub fn get_shadow_root_mode(&self, shadow_root: NodeId) -> Option<ShadowRootMode> {
        self.nodes.get(shadow_root).and_then(|n| match &n.kind {
            NodeKind::ShadowRoot(data) => Some(data.mode),
            _ => None,
        })
    }

    /// 获取 slot 元素已分配的节点列表。
    ///
    /// 查找分配给指定 slot 的 light DOM 节点。
    /// `slot_name` 对应 slot 元素的 `name` 属性值。
    pub fn assigned_nodes(&self, slot_element: NodeId, slot_name: &str) -> Vec<NodeId> {
        self.slot_assignments
            .get(&(slot_element, slot_name.to_string()))
            .cloned()
            .unwrap_or_default()
    }

    /// 将 light DOM 节点分配给指定的 slot。
    ///
    /// `slot_element` 是 `<slot>` 元素的 NodeId，
    /// `slot_name` 是 slot 的 `name` 属性值。
    pub fn assign_slot(&mut self, slot_element: NodeId, slot_name: &str, node: NodeId) {
        self.slot_assignments
            .entry((slot_element, slot_name.to_string()))
            .or_default()
            .push(node);
    }

    /// 解析 Shadow DOM slot 分配。
    ///
    /// 遍历宿主元素（`host_node_id`）的 shadow 树中所有 `<slot>` 元素，
    /// 将宿主 light DOM 子节点中匹配的元素分配到对应 slot：
    ///
    /// - 有 `slot="name"` 属性的子元素分配到 `<slot name="name">`
    /// - 没有 `slot` 属性的子元素分配到默认 slot（`<slot>` 无 name 属性）
    /// - 如果某个 slot 没有匹配的 light DOM 子节点，使用 slot 自身的子节点作为回退内容
    ///
    /// 该方法会清除并重新计算该宿主的全部 slot 分配。
    ///
    /// `slotAssignment: 'manual'` 的 shadow root 整树跳过（spec：manual 模式
    /// light 子节点不参与自动分配，分配面是 `assign()` 的手动列表——
    /// [`Self::set_manual_assigned_nodes`] 镜像通道；这里若照常重算会把手动
    /// 分配清成自动匹配结果）。
    /// https://dom.spec.whatwg.org/#dom-shadowroot-slotassignment
    pub fn resolve_slots(&mut self, host_node_id: NodeId) {
        // 获取 shadow root
        let shadow_id = match self.shadow_roots.get(&host_node_id).copied() {
            Some(id) => id,
            None => return,
        };

        // 45 号 M3 手动臂：manual root 不做自动分配（跳过清表与重算两步）。
        if matches!(
            self.nodes.get(shadow_id).map(|n| &n.kind),
            Some(NodeKind::ShadowRoot(d)) if d.slot_assignment == SlotAssignment::Manual
        ) {
            return;
        }

        // 收集 shadow 树中所有 <slot> 元素：(NodeId, Option<name>)
        let slot_elements = self.collect_slot_elements(shadow_id);
        if slot_elements.is_empty() {
            return;
        }

        // 清除这些 slot 元素的旧分配
        for &(slot_id, ref slot_name_opt) in &slot_elements {
            let name = slot_name_opt.as_deref().unwrap_or("");
            self.slot_assignments.remove(&(slot_id, name.to_string()));
        }

        // 收集宿主的 light DOM 子节点（排除 ShadowRoot 自身）
        let host_children = self
            .nodes
            .get(host_node_id)
            .map(|n| n.children.clone())
            .unwrap_or_default();
        let light_children: Vec<NodeId> = host_children
            .iter()
            .copied()
            .filter(|&id| !matches!(self.nodes.get(id).map(|n| &n.kind), Some(NodeKind::ShadowRoot(_))))
            .collect();

        // 对每个 light DOM 子节点，确定它分配到哪个 slot
        for &child_id in &light_children {
            let slot_attr = self.nodes.get(child_id).and_then(|n| match &n.kind {
                NodeKind::Element(elem) => elem.get_attribute("slot"),
                _ => None,
            });

            let matched_slot = if let Some(ref slot_name) = slot_attr {
                // 有 slot 属性：找匹配 name 的 slot
                slot_elements
                    .iter()
                    .find(|(_, name_opt)| name_opt.as_deref() == Some(slot_name.as_str()))
            } else {
                // 没有 slot 属性：分配到默认 slot（无 name 属性的 slot）
                slot_elements.iter().find(|(_, name_opt)| name_opt.is_none())
            };

            if let Some(&(slot_id, ref name_opt)) = matched_slot {
                let name = name_opt.as_deref().unwrap_or("").to_string();
                self.slot_assignments.entry((slot_id, name)).or_default().push(child_id);
            }
        }
    }

    /// 45 号 shadow-dom-rendering M3 R3（变异应用臂）：重解析全部 shadow host 的
    /// slot 分配——供 engine 变异应用批次（`apply_dom_mutations_full`）批末统一
    /// 挂接。命令式 attachShadow、宿主 childList 变异（appendChild/insertBefore/
    /// removeChild/innerHTML）与 slot/name 属性改名此前都不触发 [`Self::resolve_slots`]
    ///（仅声明式物化时点解析一次，见 [`Self::materialize_declarative_shadow_roots`]
    /// FIXME——已收口），动态场景分配过期：slot 走回退内容、被分配 light 子树无
    /// 布局盒。批次粒度全量重解析而非增量跟踪：每文档 shadow host 数量极小，
    /// [`Self::resolve_slots`] 每宿主 O(子数+slot 数) 幂等；无 shadow 页
    /// `shadow_roots` 空 O(1) 快返。嵌套 shadow 宿主在 `shadow_roots` 表内逐层
    /// 各自解析（与逐宿主调用语义一致）。
    /// https://dom.spec.whatwg.org/#assigning-slots-and-slots
    pub fn resolve_slots_all_hosts(&mut self) {
        if self.shadow_roots.is_empty() {
            return;
        }
        let hosts: Vec<NodeId> = self.shadow_roots.keys().copied().collect();
        for host in hosts {
            self.resolve_slots(host);
        }
    }

    /// 45 号 shadow-dom-rendering M3 手动臂：写入 slot 的 `assign()` 手动分配
    /// 原始列表（`HTMLSlotElement.assign()` 的渲染镜像通道——JS shim 侧
    /// `__zwManualAssigned` 语义面归 42 号，本表只服务渲染消费）。assign 是
    /// **整体替换**语义（spec assign slottables：列表清旧写新，跨 slot 抢占由
    /// shim 侧 victim 臂同步清对方列表后各自调本方法）。
    ///
    /// 读取面 [`Self::get_assigned_nodes`] 对 manual root 的 slot 活过滤
    /// parent==host，故这里存原始列表即可（assign 后节点移出宿主自动失配）。
    /// https://dom.spec.whatwg.org/#dom-slotable-assignedslot
    pub fn set_manual_assigned_nodes(&mut self, slot_node_id: NodeId, nodes: Vec<NodeId>) {
        if nodes.is_empty() {
            self.manual_slot_assignments.remove(&slot_node_id);
        } else {
            self.manual_slot_assignments.insert(slot_node_id, nodes);
        }
    }

    /// slot 元素所属 shadow root 及其宿主——从 slot 沿 parent 链上溯到
    /// [`NodeKind::ShadowRoot`] 边界（slot 恒在 shadow 树内或 light DOM；
    /// light slot 无 shadow 上下文返 `None`，走既有自动分配表路径）。
    fn slot_shadow_context(&self, slot_node_id: NodeId) -> Option<(NodeId, NodeId)> {
        let mut cur = self.nodes.get(slot_node_id)?.parent;
        while let Some(id) = cur {
            match &self.nodes.get(id)?.kind {
                NodeKind::ShadowRoot(d) => {
                    return d.host.map(|host| (id, host));
                }
                _ => cur = self.nodes.get(id)?.parent,
            }
        }
        None
    }

    /// ZeroWeb 内部序列化通道（M3 手动臂）：slot 的 `assign()` 分配关系序列化
    /// 形态——`data-zw-manual-assigned="#id1 #id2 …"`（render 顺序，空格分隔的
    /// `#id` 引用）。
    ///
    /// spec 无声明式对影：manually assigned nodes 只活在 `assign()` 运行时
    ///（spec 明言该集合不可从脚本直接观测、可以弱引用实现），序列化产物不含
    /// 分配关系。ZeroWeb 的 query_selector 渲染管线在 serialize→re-parse 边界
    ///（快照/变异应用→`render_html` 重解析）重建 Document，NodeId 键控的手动表
    /// 无法越过该边界；本通道以 marker 属性把分配关系写进序列化产物，parse 侧
    /// 物化后由 [`Self::restore_manual_slot_assignments`] 还原并摘除。
    ///
    /// 降级策略：任一被分配节点无 `id`（或非元素/已移除）→ 返 `None` 不写
    /// marker，re-parse 后该 slot 走回退内容——与真实浏览器行为一致
    ///（assign 状态本就非持久化面）。FIXME(45-M3+)：无 id 节点的稳定引用方案
    ///（树位路径编码）。
    ///
    /// 触发门：slot 所属 shadow root 为 [`SlotAssignment::Manual`] 且手动表
    /// 非空——named root 的 assign() 列表 inert，不产生序列化痕迹。
    pub(crate) fn manual_assigned_slot_marker(&self, slot_node_id: NodeId) -> Option<String> {
        let (shadow_root, _) = self.slot_shadow_context(slot_node_id)?;
        if !matches!(
            self.nodes.get(shadow_root).map(|n| &n.kind),
            Some(NodeKind::ShadowRoot(d)) if d.slot_assignment == SlotAssignment::Manual
        ) {
            return None;
        }
        let nodes = self.manual_slot_assignments.get(&slot_node_id)?;
        if nodes.is_empty() {
            return None;
        }
        let mut refs = Vec::with_capacity(nodes.len());
        for &n in nodes {
            let id = match self.nodes.get(n).map(|nd| &nd.kind) {
                Some(NodeKind::Element(e)) => e.get_attribute("id"),
                _ => return None,
            };
            match id {
                Some(v) if !v.is_empty() => refs.push(format!("#{v}")),
                _ => return None,
            }
        }
        Some(refs.join(" "))
    }

    /// [`Self::manual_assigned_slot_marker`] 的 parse 侧对偶：扫全树（children
    /// 边含 shadow 树——ShadowRoot 挂在宿主 children；template contents 不在
    /// children 边，天然不达）摘 `data-zw-manual-assigned` marker 并还原手动表。
    ///
    /// 引用解析走 [`Self::get_element_by_id`]（light DOM O(1) 索引面；跨 shadow
    /// 树引用命中封装边界返 `None`——与降级策略同路）。任一引用不可解析（006
    /// hosts 4/5 场景：assign 后节点被 remove）→ 放弃整个 marker，slot 走回退
    /// 内容；部分解析不做（spec assign 是整体替换语义，残表比空表更失真）。
    /// 摘除先行于建表：marker 不留存活属性面（re-serialize 不再回写、快照
    /// 等价性不破）。
    pub(crate) fn restore_manual_slot_assignments(&mut self) {
        let mut slots = Vec::new();
        self.collect_manual_slot_marker_candidates(self.root, &mut slots);
        for slot in slots {
            let Some(marker) = self.get_attribute(slot, "data-zw-manual-assigned") else {
                continue;
            };
            self.remove_attribute(slot, "data-zw-manual-assigned");
            let manual_root = self
                .slot_shadow_context(slot)
                .and_then(|(shadow_root, _)| {
                    matches!(
                        self.nodes.get(shadow_root).map(|n| &n.kind),
                        Some(NodeKind::ShadowRoot(d)) if d.slot_assignment == SlotAssignment::Manual
                    )
                    .then_some(())
                })
                .is_some();
            if !manual_root {
                continue;
            }
            let mut resolved = Vec::new();
            for r in marker.split_ascii_whitespace() {
                match r.strip_prefix('#').and_then(|id| self.get_element_by_id(id)) {
                    Some(n) => resolved.push(n),
                    None => {
                        resolved.clear();
                        break;
                    }
                }
            }
            if !resolved.is_empty() {
                self.set_manual_assigned_nodes(slot, resolved);
            }
        }
    }

    /// children 边全树枚举带 `data-zw-manual-assigned` marker 的 slot 元素
    ///（含 shadow 树内；NodeId 先收集后处理，摘除/建表不干扰遍历）。
    fn collect_manual_slot_marker_candidates(&self, id: NodeId, result: &mut Vec<NodeId>) {
        let Some(node_data) = self.nodes.get(id) else {
            return;
        };
        if matches!(&node_data.kind, NodeKind::Element(e) if e.local_name() == "slot")
            && self.get_attribute(id, "data-zw-manual-assigned").is_some()
        {
            result.push(id);
        }
        for &child in &node_data.children {
            self.collect_manual_slot_marker_candidates(child, result);
        }
    }

    /// 获取 slot 元素已分配的节点列表。
    ///
    /// 在调用 `resolve_slots` 之后使用此方法获取分配结果。
    /// 如果 slot 有已分配的 light DOM 节点，返回这些节点；
    /// 如果没有已分配的节点（即使用回退内容），返回空列表。
    ///
    /// `slotAssignment: 'manual'` shadow root 内的 slot 走手动表
    ///（[`Self::set_manual_assigned_nodes`] 原始列表）并活过滤 parent==host
    ///（spec assign slottables 可见面；shim 侧 assignedNodes 的 matchedM 同口径
    /// ——assign 后移出宿主的节点不再算已分配，slot 显示回退内容）。manual 表
    /// 命中仍以所属 shadow root 的分配策略为门（spec find slotables 的 manual
    /// 分支——named root 的 slot 上 assign() 列表 inert，与 shim 读取面同口径；
    /// 表未命中的 manual root slot 恒走自动表路径，[`Self::resolve_slots`] 对
    /// manual host 不写表，结果同为空 = 回退内容）。
    ///
    /// `slot_node_id` 是 `<slot>` 元素的 NodeId。
    pub fn get_assigned_nodes(&self, slot_node_id: NodeId) -> Vec<NodeId> {
        // 手动分配面：先探手动表（哈希命中即走；未命中不付 parent 链上溯——
        // named 页读取路径零新增成本）。
        if let Some(nodes) = self.manual_slot_assignments.get(&slot_node_id)
            && let Some((shadow_root, host)) = self.slot_shadow_context(slot_node_id)
            && matches!(
                self.nodes.get(shadow_root).map(|n| &n.kind),
                Some(NodeKind::ShadowRoot(d)) if d.slot_assignment == SlotAssignment::Manual
            )
        {
            let result: Vec<NodeId> = nodes
                .iter()
                .copied()
                .filter(|&n| self.nodes.get(n).and_then(|nd| nd.parent) == Some(host))
                .collect();
            return result;
        }

        // 获取 slot 的 name 属性
        let slot_name = self
            .nodes
            .get(slot_node_id)
            .and_then(|n| match &n.kind {
                NodeKind::Element(elem) => elem.get_attribute("name"),
                _ => None,
            })
            .unwrap_or_default();

        self.slot_assignments
            .get(&(slot_node_id, slot_name))
            .cloned()
            .unwrap_or_default()
    }

    /// 45 号 shadow-dom-rendering M2：反查节点被分配到的 slot 元素。
    ///
    /// [`Self::get_assigned_nodes`] 的对偶——样式/匹配侧从 slotted 元素出发定位其
    /// flat tree 父（`::slotted` 段收集的级联源定位）。分配表空（非 shadow 页常态）
    /// 快返 `None` 零开销；shadow 页每元素一次 O(slot 数 × 分配数) 反扫（表极小，
    /// 每元素样式计算一次可接受）。FIXME(45-M3+)：动态分配接线后若成热点，建
    /// 节点 → slot 反向索引。
    ///
    /// 45 号 M3 手动臂：manual root 的 slot 分配活在手动表（`assign()` 镜像，
    /// [`Self::set_manual_assigned_nodes`]），与自动表同扫；手动条目按
    /// parent==host 活过滤（与 [`Self::get_assigned_nodes`] 读侧同口径——原始
    /// 列表节点被移出宿主后不再算已分配）。
    pub fn assigned_slot(&self, node: NodeId) -> Option<NodeId> {
        if self.slot_assignments.is_empty() && self.manual_slot_assignments.is_empty() {
            return None;
        }
        if let Some(((slot, _), _)) = self
            .slot_assignments
            .iter()
            .find(|(_, assigned)| assigned.contains(&node))
        {
            return Some(*slot);
        }
        self.manual_slot_assignments
            .iter()
            .find(|(_, assigned)| assigned.contains(&node))
            .and_then(|(&slot, _)| {
                // 与 [`Self::get_assigned_nodes`] 同门：manual 表命中仍以所属
                // shadow root 分配策略为准（named root 的 assign() 列表 inert）。
                let (shadow_root, host) = self.slot_shadow_context(slot)?;
                if !matches!(
                    self.nodes.get(shadow_root).map(|n| &n.kind),
                    Some(NodeKind::ShadowRoot(d)) if d.slot_assignment == SlotAssignment::Manual
                ) {
                    return None;
                }
                (self.nodes.get(node).and_then(|n| n.parent) == Some(host)).then_some(slot)
            })
    }

    /// 在 ShadowRoot 子树中查找第一个匹配选择器的元素。
    ///
    /// 类似 `query_selector`，但范围限定在 shadow DOM 树内，
    /// 不会穿透到嵌套的 ShadowRoot 边界。
    pub fn query_selector_shadow(&self, shadow_root: NodeId, selector: &str) -> Option<NodeId> {
        let parsed = self.cached_simple_selector(selector)?;
        self.find_first_matching_shadow(shadow_root, &parsed)
    }

    /// 在 ShadowRoot 子树中查找所有匹配选择器的元素。
    ///
    /// 类似 `query_selector_all`，但范围限定在 shadow DOM 树内，
    /// 不会穿透到嵌套的 ShadowRoot 边界。
    pub fn query_selector_all_shadow(&self, shadow_root: NodeId, selector: &str) -> Vec<NodeId> {
        let parsed = match self.cached_simple_selector(selector) {
            Some(s) => s,
            None => return vec![],
        };
        let mut result = Vec::new();
        self.collect_matching_shadow(shadow_root, &parsed, &mut result);
        result
    }
}
