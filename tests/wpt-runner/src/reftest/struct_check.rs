//! DC-13 struct-check 布局结构验证（R1707 从 reftest.rs 抽离，reftest.rs 减负）。
//!
//! 检测布局树结构性退化：兄弟盒重叠 / 塌缩容器 / 文本串联 / 替换元素塌缩 /
//! 按 class 计数盒与行数。函数经 `pub use struct_check::*` 在 reftest 模块根再导出，
//! 调用方（main.rs product-smoke gate）仍用 `reftest::<fn>` 路径访问，字节等价。
//! 纯重定位——函数体用全限定路径，无 use 声明依赖。

/// DC-13 line 322-326：检测布局树中**同父兄弟盒** border-box 重叠（产品可见排版退化回归门）。
///
/// 正常流（block/flex/grid/float）同父兄弟盒不应重叠；重叠面积 > 阈值（忽略负 margin 等微叠）
/// 表示布局 breakage（abspos 错位、margin 折叠异常、IFC/匿名盒错排等）。返回问题描述列表
/// （空 = 结构通过）。与像素 diff 门禁互补：像素 diff 量化整体差距，本检查定位**结构性**
/// 退化（即使像素差小，兄弟盒重叠也是用户可见 bug）。`labels` 为 node_id→tag.class（诊断用）。
pub fn check_sibling_overlaps(
    root: &zero_layout_engine::types::LayoutBox,
    labels: &std::collections::HashMap<zero_dom::NodeId, String>,
    paint_skip: &std::collections::HashSet<zero_dom::NodeId>,
) -> Vec<String> {
    // 忽略 < 100px² 的微小叠（负 margin、亚像素边界、匿名盒 layout artifact）；仅报显著
    // 结构重叠。50→100：morning 残余 50px² 匿名盒重叠为亚像素噪声（~7×7px），真重叠
    //（wintertc 55936 / 测试 2500）远超此阈值。
    const MIN_OVERLAP_PX: f32 = 100.0;
    let mut issues = Vec::new();
    fn label_of(
        b: &zero_layout_engine::types::LayoutBox,
        labels: &std::collections::HashMap<zero_dom::NodeId, String>,
    ) -> String {
        b.node_id
            .and_then(|id| labels.get(&id).cloned())
            .unwrap_or_else(|| "(anon)".to_string())
    }
    fn walk(
        b: &zero_layout_engine::types::LayoutBox,
        off_x: f32,
        off_y: f32,
        labels: &std::collections::HashMap<zero_dom::NodeId, String>,
        paint_skip: &std::collections::HashSet<zero_dom::NodeId>,
        issues: &mut Vec<String>,
    ) {
        let abs_x = off_x + b.x;
        let abs_y = off_y + b.y;
        let child_off_x = abs_x + b.padding_left + b.border_left;
        let child_off_y = abs_y + b.padding_top + b.border_top;
        let n = b.children.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let (ci, cj) = (&b.children[i], &b.children[j]);
                // 跳过无可见尺寸的盒（纯结构包裹 / 零尺寸匿名盒）
                if ci.width < 2.0 || ci.height < 2.0 || cj.width < 2.0 || cj.height < 2.0 {
                    continue;
                }
                // R1504：跳过 positioned（abspos/fixed/relative）或 float 盒对——它们**按设计**重叠
                //（relative offset / abspos 定位 / float exclusion 都是合法叠加，reftest red/green
                // 覆盖、z-order、paint-order 测试大量用之）。flag 这些是噪声非 bug。仅报**普通块流**
                // 兄弟的非预期重叠（如 R1492 长高重叠）。须任一为 positioned/float 即跳过该对。
                if ci.is_absolute
                    || ci.is_fixed
                    || ci.is_relative
                    || ci.is_sticky
                    || !matches!(ci.float, zero_css_parser::values::FloatValue::None)
                    || cj.is_absolute
                    || cj.is_fixed
                    || cj.is_relative
                    || cj.is_sticky
                    || !matches!(cj.float, zero_css_parser::values::FloatValue::None)
                {
                    continue;
                }
                // R2198：跳过 paint_skip orphan box 对——orphan 是 hit-test proxy（paint-skip，
                // 非视觉盒），其几何为父 IFC 片段并集（multi-line inline 元素并集盒会与同行
                // sibling 边界重叠，如 morning 窄屏 CC 许可 `<a>` 跨行），不代表视觉重叠。
                // orphan 文本/背景已由父 IFC 片段绘制（R639 part2）。任一为 paint_skip 即跳过。
                if ci.node_id.is_some_and(|id| paint_skip.contains(&id))
                    || cj.node_id.is_some_and(|id| paint_skip.contains(&id))
                {
                    continue;
                }
                let (ov, ov_h) = rect_overlap_area(
                    (child_off_x + ci.x, child_off_y + ci.y, ci.width, ci.height),
                    (child_off_x + cj.x, child_off_y + cj.y, cj.width, cj.height),
                );
                // R1503：跳过「宽而薄」的亚像素 sliver——重叠高 ≤ 2px 但面积超阈（如 morning @320
                // 相邻行 `<code>` 1px×149=149px²，IFC baseline/line-height 舍入噪声）非真结构重叠。
                // 真 overlap（如 article/disqus 108px 高）远超此。area 阈值单独会被宽 sliver 绕过。
                if ov > MIN_OVERLAP_PX && ov_h > 2.0 {
                    issues.push(format!(
                        "sibling overlap {:.0}px²: [{}] @({:.0},{:.0},{:.0}x{:.0}) & [{}] @({:.0},{:.0},{:.0}x{:.0})",
                        ov,
                        label_of(ci, labels),
                        child_off_x + ci.x,
                        child_off_y + ci.y,
                        ci.width,
                        ci.height,
                        label_of(cj, labels),
                        child_off_x + cj.x,
                        child_off_y + cj.y,
                        cj.width,
                        cj.height
                    ));
                }
            }
        }
        for child in &b.children {
            walk(child, child_off_x, child_off_y, labels, paint_skip, issues);
        }
    }
    walk(root, 0.0, 0.0, labels, paint_skip, &mut issues);
    issues
}

/// DC-13 struct-check 扩展（R1575）：检测「塌缩容器」——有显著高度子内容但自身高度
/// 近 0 的**真实元素**盒（非匿名）。这是 layout grow 失败的强信号（容器未随内容长高，
/// R1492 谱系 / IFC 高度未回填等）。仅 flag 显著 case（子 > 20px 且父 < 2px）以降低
/// 误报（合法 height:0 + 高子元素在产品 fixture 罕见）。
pub fn check_collapsed_containers(
    root: &zero_layout_engine::types::LayoutBox,
    labels: &std::collections::HashMap<zero_dom::NodeId, String>,
) -> Vec<String> {
    const MIN_CHILD_H: f32 = 20.0;
    const MAX_PARENT_H: f32 = 2.0;
    let mut issues = Vec::new();
    fn walk(
        b: &zero_layout_engine::types::LayoutBox,
        labels: &std::collections::HashMap<zero_dom::NodeId, String>,
        issues: &mut Vec<String>,
    ) {
        // 仅真实元素盒（labels 含该 node_id；匿名盒 label_of 返回 "(anon)"）
        let is_real = b.node_id.is_some_and(|id| labels.contains_key(&id));
        if is_real && b.height < MAX_PARENT_H {
            // 找最高**流内**子盒（排除 abspos/fixed——脱离流，父盒正确不随其长高；
            // position-absolute-* 测试的父盒 h=0 是正确行为，非 bug）。
            let max_child_h = b
                .children
                .iter()
                .filter(|c| !c.is_absolute && !c.is_fixed)
                .map(|c| c.height)
                .fold(0.0f32, f32::max);
            if max_child_h > MIN_CHILD_H {
                let label = b
                    .node_id
                    .and_then(|id| labels.get(&id).cloned())
                    .unwrap_or_else(|| "(anon)".to_string());
                issues.push(format!(
                    "collapsed container: [{label}] h={:.0} < in-flow child h={:.0} (parent failed to grow around content)",
                    b.height, max_child_h
                ));
            }
        }
        for child in &b.children {
            walk(child, labels, issues);
        }
    }
    walk(root, labels, &mut issues);
    issues
}

/// DC-13 line 325「不同 sibling card/link/shortcut 的文本不串联」：检测容器把**块级子元素**
/// 的文本错误地吸收进自身 IFC 的回归（R109 inline-ownership 失效谱系）。
///
/// 信号原理：`store_font_sizes_from_ifc`（layout 主路径多处调用）把每个 box 自身 IFC 处理过的
/// 文本节点 ID 存入该 box 的 `text_node_font_sizes`/`text_node_line_heights` 映射。正常布局下，
/// grid/flex/block 容器（如 welcome 的 `.cards`/`.shortcuts`/`.links`）的文本由各 block 子盒各自
/// 的 IFC 渲染，**容器自身的 text_node 映射为空**。当 R109 inline-ownership 退化时，父容器经
/// `text_content()` 收集整棵 inline 子树文本并跑一套 IFC，把多个 sibling block 子元素的文本拼到
/// 同一 IFC——此时容器的 text_node 映射会含**子元素子树**的文本节点 ID（用户可见的「卡片/链接/
/// 快捷键文本串联」退化）。
///
/// 判定规则（三条件全满足才 flag，降低误报）：
/// 1. 容器有 ≥2 个**真实元素**（labels 含其 node_id）、in-flow、高度 ≥ `MIN_CHILD_H` 的子盒
///    （= block-level sibling，排除 inline span / 匿名盒 / 单子容器）。
/// 2. 容器自身 `text_node_line_heights` 含 ≥1 个**非空白**文本节点 ID（确有吸收的子元素文本，
///    由 `non_ws_text_nodes` 过滤空白/换行节点）。
/// 3. 容器的 DOM 元素**无直接文本子节点**（`!has_direct_text`）——有直接文本的容器（如
///    `<div>intro<p>..</p></div>` 块中行 / 合法 block-in-inline）合法拥有自身 IFC，不 flag。
///
/// 与 [`check_sibling_overlaps`]（border-box 重叠）互补：后者抓几何重叠，本检查抓**文本归属**
/// 串联（盒未必重叠，但文本被错位拼到容器自身 IFC）。`has_direct_text`/`non_ws_text_nodes`
/// 由 [`collect_concat_dom_info`] 从 DOM 预构建。
pub fn check_text_concatenation(
    root: &zero_layout_engine::types::LayoutBox,
    labels: &std::collections::HashMap<zero_dom::NodeId, String>,
    has_direct_text: &std::collections::HashSet<zero_dom::NodeId>,
    non_ws_text_nodes: &std::collections::HashSet<zero_dom::NodeId>,
) -> Vec<String> {
    const MIN_BLOCK_CHILDREN: usize = 2;
    const MIN_CHILD_H: f32 = 4.0;
    let mut issues = Vec::new();
    fn walk(
        b: &zero_layout_engine::types::LayoutBox,
        labels: &std::collections::HashMap<zero_dom::NodeId, String>,
        has_direct_text: &std::collections::HashSet<zero_dom::NodeId>,
        non_ws_text_nodes: &std::collections::HashSet<zero_dom::NodeId>,
        issues: &mut Vec<String>,
    ) {
        // 条件 1：≥2 个真实元素、in-flow、显著高度的 block-level 子盒。
        let block_children = b
            .children
            .iter()
            .filter(|c| {
                !c.is_absolute
                    && !c.is_fixed
                    && c.height >= MIN_CHILD_H
                    && c.node_id.is_some_and(|id| labels.contains_key(&id))
            })
            .count();
        if block_children >= MIN_BLOCK_CHILDREN
            && let Some(bid) = b.node_id
            && !has_direct_text.contains(&bid)
        {
            // R1652：跳过 table-internal 容器（tr/td/th/tbody/thead/tfoot/caption/col/colgroup）。
            // 本检查针对 flex/grid/block 的 R109 inline-ownership 串联（sibling 文本错位拼到共享
            // IFC）；table 单元格**合法**拥有自身文本（cell 内容由 td IFC 处理，tr 的
            // text_node_line_heights 会含子 cell 文本属正常 table 布局，非串联 bug）。legacy-html
            // fixture 19-testpage-minimal 的 `<tr>` 误报即此（LAYOUT_DUMP 表格几何正确）。
            let is_table_internal = labels.get(&bid).is_some_and(|label| {
                let tag = label.split('.').next().unwrap_or("");
                matches!(
                    tag,
                    "tr" | "td" | "th" | "tbody" | "thead" | "tfoot" | "caption" | "col" | "colgroup"
                )
            });
            // 条件 2：容器自身 text_node 映射含 ≥1 个非空白文本节点（吸收的子元素文本）。
            // table-internal 跳过（合法 table 文本归属，非串联）。
            if !is_table_internal {
                let absorbed: usize = b
                    .text_node_line_heights
                    .keys()
                    .filter(|id| non_ws_text_nodes.contains(id))
                    .count();
                if absorbed >= 1 {
                    let label = labels.get(&bid).cloned().unwrap_or_else(|| "(unlabeled)".to_string());
                    issues.push(format!(
                        "text concatenation: [{label}] ran an IFC absorbing {absorbed} non-whitespace \
                         text node(s) from across {block_children} block children (sibling text merged \
                         into shared IFC — R109 inline-ownership regression)",
                    ));
                }
            }
        }
        for child in &b.children {
            walk(child, labels, has_direct_text, non_ws_text_nodes, issues);
        }
    }
    walk(root, labels, has_direct_text, non_ws_text_nodes, &mut issues);
    issues
}

/// DC-13 [`check_text_concatenation`] 辅助：单次 DOM 遍历产出两个集合。
///
/// - `has_direct_text`：有非空白**直接**文本子节点的元素 NodeId（合法拥有自身 IFC 的容器，
///   如 `<p>text</p>`、`<div>intro<div>b</div></div>` 外层 div），用于条件 3 排除。
/// - `non_ws_text_nodes`：内容非空白的文本节点 NodeId，用于条件 2 过滤空白/换行节点
///   （避免 HTML 源码缩进空白触发误报）。
pub fn collect_concat_dom_info(
    html: &str,
) -> (
    std::collections::HashSet<zero_dom::NodeId>,
    std::collections::HashSet<zero_dom::NodeId>,
) {
    use std::collections::HashSet;
    use zero_dom::{NodeKind, parse_html};
    let doc = parse_html(html);
    let mut has_direct_text: HashSet<zero_dom::NodeId> = HashSet::new();
    let mut non_ws_text_nodes: HashSet<zero_dom::NodeId> = HashSet::new();
    let mut queue = vec![doc.root()];
    while let Some(id) = queue.pop() {
        if let Some(node) = doc.get(id) {
            match &node.kind {
                NodeKind::Element(_) => {
                    // 检查直接子节点中是否有非空白 Text。
                    let mut child = doc.first_child(id);
                    let mut has_text = false;
                    while let Some(c) = child {
                        if let Some(cnode) = doc.get(c)
                            && let NodeKind::Text(t) = &cnode.kind
                            && !t.content.trim().is_empty()
                        {
                            has_text = true;
                        }
                        queue.push(c);
                        child = doc.next_sibling(c);
                    }
                    if has_text {
                        has_direct_text.insert(id);
                    }
                }
                NodeKind::Text(t) => {
                    if !t.content.trim().is_empty() {
                        non_ws_text_nodes.insert(id);
                    }
                    let mut child = doc.first_child(id);
                    while let Some(c) = child {
                        queue.push(c);
                        child = doc.next_sibling(c);
                    }
                }
                _ => {
                    let mut child = doc.first_child(id);
                    while let Some(c) = child {
                        queue.push(c);
                        child = doc.next_sibling(c);
                    }
                }
            }
        }
    }
    (has_direct_text, non_ws_text_nodes)
}

/// DC-13 line 327「参与方 Logo 网格中 SVG/PNG Logo 可见且不会退化为短横/alt glyph」：
/// 检测**塌缩的替换元素**（`<img>`/logo）——`is_replaced` 盒 width<2 或 height<2，即固有尺寸
/// 未解析/图片未参与布局致盒塌缩到近 0（R1578b inline>inline-IMG 固有尺寸谱系：img 塌缩→
/// 容器塌缩→logo 不可见）。返回问题描述列表（空 = 通过）。
///
/// **opt-in 检查**：仅在 `--check-img-visibility` 启用时跑（非通用 struct-check）——因为部分
/// fixture 含**故意缺失**的图片（如 morning 的 `images/cc_unavailable.png` 测 alt 回退），
/// 通用 gate 会误报。仅对「所有图片都应可见」的 fixture（如 wintertc 14 个 logo 全有 asset）
/// 启用。仅报告真实元素盒（labels 含 node_id，排除匿名盒）。
pub fn check_replaced_collapse(
    root: &zero_layout_engine::types::LayoutBox,
    labels: &std::collections::HashMap<zero_dom::NodeId, String>,
) -> Vec<String> {
    const MIN_REPLACED_PX: f32 = 2.0;
    let mut issues = Vec::new();
    fn walk(
        b: &zero_layout_engine::types::LayoutBox,
        labels: &std::collections::HashMap<zero_dom::NodeId, String>,
        issues: &mut Vec<String>,
    ) {
        if b.is_replaced
            && let Some(id) = b.node_id
            && let Some(label) = labels.get(&id)
            && (b.width < MIN_REPLACED_PX || b.height < MIN_REPLACED_PX)
        {
            issues.push(format!(
                "collapsed replaced element: [{label}] size={:.0}x{:.0} (img/logo failed to get \
                 intrinsic size — R1578b spectrum; degrades to invisible/alt glyph)",
                b.width, b.height
            ));
        }
        for child in &b.children {
            walk(child, labels, issues);
        }
    }
    walk(root, labels, &mut issues);
    issues
}

/// DC-13 line 322：统计布局树中带指定 class 的盒数（结构计数断言用）。
///
/// `labels`（[`collect_dom_labels`] 产出）格式为 `tag.class1.class2`；本函数按 `.` 拆分后
/// 跳过 tag、**精确匹配** class（`.card` 不误匹配 `.card-sub`，避免子串假阳性）。用于
/// 检测结构塌缩（如 welcome 须有 4 个 `.card`，丢失/塌缩会被检出）。
pub fn count_boxes_by_class(
    root: &zero_layout_engine::types::LayoutBox,
    labels: &std::collections::HashMap<zero_dom::NodeId, String>,
    class: &str,
) -> usize {
    let mut count = 0usize;
    fn walk(
        b: &zero_layout_engine::types::LayoutBox,
        class: &str,
        labels: &std::collections::HashMap<zero_dom::NodeId, String>,
        count: &mut usize,
    ) {
        if let Some(id) = b.node_id
            && let Some(label) = labels.get(&id)
        {
            // label = "tag.class1.class2"；跳过首段 tag，精确匹配 class token。
            if label.split('.').skip(1).any(|c| c == class) {
                *count += 1;
            }
        }
        for child in &b.children {
            walk(child, class, labels, count);
        }
    }
    walk(root, class, labels, &mut count);
    count
}

/// DC-13 line 323/324：估算带指定 class 的首个盒的**行数**（行数断言用，如「标题不拆行」/
///「tagline 保持 2 行」）。
///
/// 行数 = `content_height / per_line_height`，其中 `per_line_height` 取该盒 IFC 存储的
/// `text_node_line_heights`（store_font_sizes_from_ifc 存的**单行**片段高度，= 1 行高度）。
/// content_height = 行盒高度之和 = 行数 × 单行高度，故比值即行数。无存储度量（如非 IFC owner
/// 或无文本）返回 None（调用方跳过）。`.round()` 容忍亚像素；阈值 ±0.5 判定。
pub fn count_lines_for_class(
    root: &zero_layout_engine::types::LayoutBox,
    labels: &std::collections::HashMap<zero_dom::NodeId, String>,
    class: &str,
) -> Option<usize> {
    fn find<'a>(
        b: &'a zero_layout_engine::types::LayoutBox,
        class: &str,
        labels: &std::collections::HashMap<zero_dom::NodeId, String>,
    ) -> Option<&'a zero_layout_engine::types::LayoutBox> {
        if let Some(id) = b.node_id
            && let Some(label) = labels.get(&id)
            && label.split('.').skip(1).any(|c| c == class)
        {
            return Some(b);
        }
        for child in &b.children {
            if let Some(found) = find(child, class, labels) {
                return Some(found);
            }
        }
        None
    }
    let b = find(root, class, labels)?;
    // 取首个 text_node_line_heights 作单行高度（同一 IFC 内文本行高一致）。
    let per_line = b.text_node_line_heights.values().next().copied()?;
    if per_line <= 0.0 {
        return None;
    }
    let content_h = b.content_height.max(0.0);
    Some((content_h / per_line).round() as usize)
}

/// 两轴对齐矩形 `(x, y, w, h)` 的交集面积（无交集返回 0）。
/// 返回 (重叠面积, 重叠高度)。重叠高度供调用方过滤「宽而薄」的亚像素 sliver（R1503）。
fn rect_overlap_area(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)) -> (f32, f32) {
    let (ax, ay, aw, ah) = a;
    let (bx, by, bw, bh) = b;
    let w = ((ax + aw).min(bx + bw) - ax.max(bx)).max(0.0);
    let h = ((ay + ah).min(by + bh) - ay.max(by)).max(0.0);
    (w * h, h)
}
