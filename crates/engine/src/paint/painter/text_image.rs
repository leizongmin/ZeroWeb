use zero_style_system::{BackgroundPositionComputedValue, ObjectFitComputedValue};

/// 获取 `<img>` 元素的固有尺寸。
///
/// 优先使用解码后的真实尺寸；若图片尚未解码，再回退到 HTML `width`/`height` 属性，
/// 最后使用调用方提供的回退尺寸。
pub(super) fn get_img_intrinsic_size(
    node: &zero_dom::NodeData,
    decoded_size: Option<(f32, f32)>,
    fallback_w: f32,
    fallback_h: f32,
) -> (f32, f32) {
    if let Some((w, h)) = decoded_size
        && w > 0.0
        && h > 0.0
    {
        return (w, h);
    }

    let elem = match &node.kind {
        zero_dom::NodeKind::Element(e) => e,
        _ => return (fallback_w, fallback_h),
    };
    let w = elem
        .get_attribute("width")
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(fallback_w);
    let h = elem
        .get_attribute("height")
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(fallback_h);
    (w.max(1.0), h.max(1.0))
}

/// 根据 `object-fit` + `object-position` 计算图片在容器内的绘制矩形。
pub(super) fn compute_object_fit_rect(
    fit: &ObjectFitComputedValue,
    position: &BackgroundPositionComputedValue,
    container_w: f32,
    container_h: f32,
    intrinsic_w: f32,
    intrinsic_h: f32,
    content_x: f32,
    content_y: f32,
) -> (f32, f32, f32, f32) {
    match fit {
        ObjectFitComputedValue::Fill => (content_x, content_y, container_w, container_h),
        ObjectFitComputedValue::Contain => {
            let scale = (container_w / intrinsic_w).min(container_h / intrinsic_h);
            let w = intrinsic_w * scale;
            let h = intrinsic_h * scale;
            let (px, py) = super::effects::resolve_background_position(position, container_w, container_h, w, h);
            (content_x + px, content_y + py, w, h)
        }
        ObjectFitComputedValue::Cover => {
            let scale = (container_w / intrinsic_w).max(container_h / intrinsic_h);
            let w = intrinsic_w * scale;
            let h = intrinsic_h * scale;
            let (px, py) = super::effects::resolve_background_position(position, container_w, container_h, w, h);
            (content_x + px, content_y + py, w, h)
        }
        ObjectFitComputedValue::None => {
            let (px, py) = super::effects::resolve_background_position(
                position,
                container_w,
                container_h,
                intrinsic_w,
                intrinsic_h,
            );
            (content_x + px, content_y + py, intrinsic_w, intrinsic_h)
        }
        ObjectFitComputedValue::ScaleDown => {
            let none_w = intrinsic_w;
            let contain_scale = (container_w / intrinsic_w).min(container_h / intrinsic_h);
            let contain_w = intrinsic_w * contain_scale;
            if none_w <= contain_w {
                let (px, py) = super::effects::resolve_background_position(
                    position,
                    container_w,
                    container_h,
                    intrinsic_w,
                    intrinsic_h,
                );
                (content_x + px, content_y + py, intrinsic_w, intrinsic_h)
            } else {
                let w = contain_w;
                let h = intrinsic_h * contain_scale;
                let (px, py) = super::effects::resolve_background_position(position, container_w, container_h, w, h);
                (content_x + px, content_y + py, w, h)
            }
        }
    }
}
