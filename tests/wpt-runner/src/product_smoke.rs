//! Product-smoke 的 Chrome 几何与局部像素门禁。

use std::collections::HashMap;

use serde::Deserialize;
use zero_layout_engine::types::LayoutBox;
use zero_render_foundation::surface::FrameBuffer;

#[derive(Debug, Deserialize)]
pub struct GeometryOracle {
    pub rects: HashMap<String, [f64; 4]>,
}

#[derive(Debug, Clone)]
pub struct RegionGate {
    pub id: String,
    pub max_diff_pct: f64,
}

pub fn parse_region_gate(value: &str) -> Option<RegionGate> {
    let (id, threshold) = value.rsplit_once(':')?;
    Some(RegionGate {
        id: id.to_string(),
        max_diff_pct: threshold.parse().ok()?,
    })
}

pub fn full_diff_passes(actual_pct: f64, threshold_pct: f64) -> bool {
    actual_pct < threshold_pct
}

pub fn load_geometry_oracle(path: &str) -> Result<GeometryOracle, String> {
    let data = std::fs::read_to_string(path).map_err(|error| format!("read {path}: {error}"))?;
    serde_json::from_str(&data).map_err(|error| format!("parse {path}: {error}"))
}

pub fn collect_layout_rects(root: &LayoutBox, html: &str) -> HashMap<String, [f64; 4]> {
    let doc = zero_dom::parse_html(html);
    let mut ids = HashMap::new();
    let mut nodes = vec![doc.root()];
    while let Some(node_id) = nodes.pop() {
        if let Some(id) = doc.get_attribute(node_id, "id") {
            ids.insert(node_id, id);
        }
        let mut child = doc.first_child(node_id);
        while let Some(child_id) = child {
            nodes.push(child_id);
            child = doc.next_sibling(child_id);
        }
    }
    let mut rects = HashMap::new();
    fn walk(
        layout: &LayoutBox,
        offset_x: f32,
        offset_y: f32,
        ids: &HashMap<zero_dom::NodeId, String>,
        rects: &mut HashMap<String, [f64; 4]>,
    ) {
        let x = offset_x + layout.x;
        let y = offset_y + layout.y;
        if let Some(id) = layout.node_id.and_then(|node_id| ids.get(&node_id)) {
            rects.insert(
                id.clone(),
                [x as f64, y as f64, layout.width as f64, layout.height as f64],
            );
        }
        let child_x = x + layout.border_left + layout.padding_left;
        let child_y = y + layout.border_top + layout.padding_top;
        for child in &layout.children {
            walk(child, child_x, child_y, ids, rects);
        }
    }
    walk(root, 0.0, 0.0, &ids, &mut rects);
    rects
}

pub fn geometry_diff(id: &str, oracle: &GeometryOracle, actual: &HashMap<String, [f64; 4]>) -> Result<f64, String> {
    let expected = oracle
        .rects
        .get(id)
        .ok_or_else(|| format!("Chrome geometry missing #{id}"))?;
    let actual = actual
        .get(id)
        .ok_or_else(|| format!("ZeroWeb geometry missing #{id}"))?;
    Ok(expected
        .iter()
        .zip(actual)
        .map(|(expected, actual)| (expected - actual).abs())
        .fold(0.0, f64::max))
}

pub fn region_diff_pct(
    first: &FrameBuffer,
    second: &FrameBuffer,
    first_rect: [f64; 4],
    second_rect: [f64; 4],
    channel_threshold: u8,
    pixel_radius: usize,
) -> f64 {
    let first_stride = first.width as usize;
    let second_stride = second.width as usize;
    let first_x = first_rect[0].floor().max(0.0) as usize;
    let first_y = first_rect[1].floor().max(0.0) as usize;
    let second_x = second_rect[0].floor().max(0.0) as usize;
    let second_y = second_rect[1].floor().max(0.0) as usize;
    let first_width = ((first_rect[0] + first_rect[2]).ceil().max(0.0) as usize).saturating_sub(first_x);
    let first_height = ((first_rect[1] + first_rect[3]).ceil().max(0.0) as usize).saturating_sub(first_y);
    let second_width = ((second_rect[0] + second_rect[2]).ceil().max(0.0) as usize).saturating_sub(second_x);
    let second_height = ((second_rect[1] + second_rect[3]).ceil().max(0.0) as usize).saturating_sub(second_y);
    let width = first_width.min(second_width);
    let height = first_height.min(second_height);
    if width == 0 || height == 0 {
        return 100.0;
    }
    let mut different = 0usize;
    let total = width * height;
    for local_y in 0..height {
        for local_x in 0..width {
            let x = first_x + local_x;
            let y = first_y + local_y;
            if x >= first.width as usize || y >= first.height as usize {
                different += 1;
                continue;
            }
            let first_index = (y * first_stride + x) * 4;
            if !pixel_matches(
                first,
                second,
                first_index,
                second_x + local_x,
                second_y + local_y,
                channel_threshold,
                pixel_radius,
                second.width as usize,
                second.height as usize,
                second_stride,
            ) {
                different += 1;
            }
        }
    }
    100.0 * different as f64 / total as f64
}

pub fn full_diff_pixels(
    first: &FrameBuffer,
    second: &FrameBuffer,
    channel_threshold: u8,
    pixel_radius: usize,
) -> usize {
    let width = first.width.min(second.width) as usize;
    let height = first.height.min(second.height) as usize;
    let first_stride = first.width as usize;
    let second_stride = second.width as usize;
    let mut different = 0;
    for y in 0..height {
        for x in 0..width {
            let first_index = (y * first_stride + x) * 4;
            if !pixel_matches(
                first,
                second,
                first_index,
                x,
                y,
                channel_threshold,
                pixel_radius,
                width,
                height,
                second_stride,
            ) {
                different += 1;
            }
        }
    }
    different
}

#[allow(clippy::too_many_arguments)]
fn pixel_matches(
    first: &FrameBuffer,
    second: &FrameBuffer,
    first_index: usize,
    x: usize,
    y: usize,
    channel_threshold: u8,
    pixel_radius: usize,
    width: usize,
    height: usize,
    second_stride: usize,
) -> bool {
    let x0 = x.saturating_sub(pixel_radius);
    let y0 = y.saturating_sub(pixel_radius);
    let x1 = (x + pixel_radius + 1).min(width);
    let y1 = (y + pixel_radius + 1).min(height);
    (y0..y1).any(|candidate_y| {
        (x0..x1).any(|candidate_x| {
            let second_index = (candidate_y * second_stride + candidate_x) * 4;
            (0..4).all(|channel| {
                first.data[first_index + channel].abs_diff(second.data[second_index + channel]) <= channel_threshold
            })
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_region_gate_from_last_colon() {
        let gate = parse_region_gate("submit:10").unwrap();
        assert_eq!(gate.id, "submit");
        assert_eq!(gate.max_diff_pct, 10.0);
        assert!(parse_region_gate("missing").is_none());
    }

    #[test]
    fn full_diff_threshold_is_strict() {
        assert!(full_diff_passes(4.99, 5.0));
        assert!(!full_diff_passes(5.0, 5.0));
    }
}
