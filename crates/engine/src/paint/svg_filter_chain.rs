//! R4276（filter-effects-1 #typedef-filter-url）：CSS `filter: url(#id)` **非常量链**
//! 的 SourceGraphic 隔离应用（设计见
//! docs/goal/rendering-compat/svg-filter-reference-isolation-design.md 方案 C）。
//!
//! 数据流：painter 主遍前抑制 isolate 元素并旁路绘制其子树图元（FilterIsolate）；
//! pipeline 以 render_full_scene 离屏栅格化 → 本模块把像素 PNG data-URI 包装进
//! `<image filter="url(#id)">` 过 resvg（rasterize_svg_at 既有通路）→ 输出 rgba
//! 回注 canvas_images 通道（占位 ImagePrimitive 的 key 在主遍已发射）。
//!
//! SourceGraphic/SourceAlpha 语义由 resvg 原生承载（image 像素即 filter 输入）。
//! kill-switch：env `ZW_SVG_URL_CHAIN=0`（default-on，painter 收集侧检查）。

use zero_dom::{Document, NodeKind};
use zero_render_foundation::geometry::Rect;

/// filter isolate 结果图的 ImageKey 命名空间高位（canvas ctx_id 自 1 递增，永不
/// 相交）。
pub(crate) const FILTER_KEY_BASE: u64 = 0xF111_7E25_0000_0000;

/// 一个待链应用的 isolate：子树旁路图元 + filter region + 引用信息。
pub(crate) struct FilterIsolate {
    /// 子树旁路图元（painter 临时换仓绘制的产物，坐标已平移至 region 原点）。
    pub primitives: zero_render_foundation::primitive::RenderPrimitives,
    /// filter region（页面绝对坐标，宽高向下取整供栅格化）。
    pub region: Rect,
    /// 被引用的 `<filter>` 元素节点（多引用按声明序顺序链应用）。
    pub filter_node_ids: Vec<zero_dom::NodeId>,
    /// 主遍占位 ImagePrimitive 的 canvas_images key。
    pub key: u64,
}

/// R4276 门禁②：空 `<filter>`（无原语子元素）= 恒等链（resvg 空链输出透明，
/// filter-chained-url-url-001 回归实证）→ 调用方跳过该引用。
pub(crate) fn chain_is_empty(doc: &Document, filter_node_id: zero_dom::NodeId) -> bool {
    doc.child_nodes(filter_node_id)
        .into_iter()
        .filter(|id| matches!(doc.get(*id).map(|n| &n.kind), Some(NodeKind::Element(_))))
        .count()
        == 0
}

/// R4276 门禁④：isolate 元素 DOM 子树内存在 CSS transform / will-change:transform
/// 后代 → region 逃逸语义（filtered 父的 transform 后代可绘出 region 外）首版不
/// 做 → 调用方旁路（filter-region-transformed-composited-child-001 回归实证）。
pub(crate) fn subtree_has_transform(
    doc: &Document,
    styles: &std::collections::HashMap<zero_dom::NodeId, zero_style_system::ComputedStyle>,
    node_id: zero_dom::NodeId,
) -> bool {
    fn walk(
        doc: &Document,
        styles: &std::collections::HashMap<zero_dom::NodeId, zero_style_system::ComputedStyle>,
        id: zero_dom::NodeId,
    ) -> bool {
        if let Some(style) = styles.get(&id)
            && (!matches!(style.transform, zero_css_parser::values::TransformValue::None)
                || style.will_change.iter().any(|w| {
                    matches!(
                        w,
                        zero_style_system::property::types::WillChangeValue::Custom(name)
                            if name.eq_ignore_ascii_case("transform")
                    )
                }))
        {
            return true;
        }
        doc.child_nodes(id).into_iter().any(|child| walk(doc, styles, child))
    }
    // 仅查**后代**：元素自身 transform 在主遍由 TransformPrimitive 作用于占位
    // image 合成（spec 先 filter 后 transform ✓），不触发 region 逃逸。
    doc.child_nodes(node_id)
        .into_iter()
        .any(|child| walk(doc, styles, child))
}

/// kill-switch（default-on）。painter 收集侧与 pipeline 应用侧共用。
pub(crate) fn url_chain_enabled() -> bool {
    std::env::var("ZW_SVG_URL_CHAIN").as_deref() != Ok("0")
}

/// PNG RGBA 编码（engine 直依赖 png crate；与 render-foundation 解码侧同 crate 族）。
pub(crate) fn encode_png_rgba(rgba: &[u8], width: u32, height: u32) -> Option<Vec<u8>> {
    let mut buf = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut buf, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().ok()?;
        writer.write_image_data(rgba).ok()?;
    }
    Some(buf)
}

/// 标准 base64（RFC 4648 §4，含 '=' padding）。仅本模块 data-URI 使用，避免为
/// 单一用途新增工作区依赖。
pub(crate) fn base64_encode(data: &[u8]) -> String {
    const TBL: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(TBL[(n >> 18) as usize & 63] as char);
        out.push(TBL[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            TBL[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TBL[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

/// 构造链应用包装 SVG：`<filter>` 原样序列化进 defs，元素像素作 data-URI image
/// 并挂 `filter="url(#id)"`——SourceGraphic = image 像素（含 alpha），其余原语
/// 语义全部由 resvg 原生执行。
pub(crate) fn build_wrapper_svg(
    doc: &Document,
    filter_node_id: zero_dom::NodeId,
    region: &Rect,
    rgba: &[u8],
) -> Option<String> {
    let node = doc.get(filter_node_id)?;
    let NodeKind::Element(elem) = &node.kind else {
        return None;
    };
    if elem.local_name() != "filter" {
        return None;
    }
    let id = elem.id.clone()?;
    let chain_xml = doc.outer_html(filter_node_id);
    let w = region.size.width.ceil().max(1.0) as u32;
    let h = region.size.height.ceil().max(1.0) as u32;
    let png = encode_png_rgba(rgba, w, h)?;
    Some(format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}">{chain_xml}<image x="0" y="0" width="{w}" height="{h}" filter="url(#{id})" href="data:image/png;base64,{}"/></svg>"#,
        base64_encode(&png)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// base64 已知向量（RFC 4648 测试组）。
    #[test]
    fn base64_rfc4648_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    /// R4276 门禁②：空 `<filter>`（无原语子元素）= 恒等链 → chain_is_empty 命中；
    /// 有原语子元素 → 非空。文本/注释子节点不计数。
    #[test]
    fn chain_is_empty_gates() {
        let doc = zero_dom::parse_html(
            r#"<html><body><svg width="0" height="0">
                <filter id="empty"></filter>
                <filter id="full"><feGaussianBlur stdDeviation="1"/></filter>
                <filter id="whitespace"> <!-- 注释 --> </filter>
            </svg></body></html>"#,
        );
        let empty = doc.get_element_by_id("empty").unwrap();
        let full = doc.get_element_by_id("full").unwrap();
        let ws = doc.get_element_by_id("whitespace").unwrap();
        assert!(chain_is_empty(&doc, empty), "无子元素 = 恒等链");
        assert!(!chain_is_empty(&doc, full), "有原语子元素非空");
        assert!(chain_is_empty(&doc, ws), "仅注释/空白 = 恒等链");
    }

    /// 包装 SVG：filter 原样进 defs、image 挂 filter 引用与 data URI。
    #[test]
    fn wrapper_svg_carries_chain_and_image() {
        let doc = zero_dom::parse_html(
            r#"<html><body><svg width="0" height="0"><filter id="alpha">
                <feColorMatrix in="SourceAlpha"/></filter></svg></body></html>"#,
        );
        let fid = doc.get_element_by_id("alpha").expect("filter");
        let region = Rect::new(8.0, 8.0, 100.0, 100.0);
        let rgba = [255u8, 0, 0, 255].repeat(100 * 100);
        let svg = build_wrapper_svg(&doc, fid, &region, &rgba).expect("wrapper");
        assert!(svg.contains(r#"width="100" height="100""#), "{svg}");
        assert!(svg.contains("<feColorMatrix"), "原链 XML 保留: {svg}");
        assert!(svg.contains(r#"filter="url(#alpha)""#), "{svg}");
        assert!(svg.contains("data:image/png;base64,"), "{svg}");
    }
}
