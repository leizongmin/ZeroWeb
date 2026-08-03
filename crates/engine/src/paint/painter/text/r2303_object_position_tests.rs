use super::compute_object_fit_rect;
use zero_style_system::property::types::{BackgroundPositionComputedValue as Bp, ObjectFitComputedValue};

fn cover() -> ObjectFitComputedValue {
    ObjectFitComputedValue::Cover
}

#[test]
fn center_is_byte_identical_to_old_centering() {
    // 100×50 intrinsic, 80×80 container, Cover → scale=1.6 → img 160×80.
    // Center: x = (80-160)/2 = -40, y = (80-80)/2 = 0。
    let (x, y, w, h) = compute_object_fit_rect(&cover(), &Bp::Center, 80.0, 80.0, 100.0, 50.0, 0.0, 0.0);
    assert_eq!((x.round(), y.round(), w.round(), h.round()), (-40.0, 0.0, 160.0, 80.0));
}

#[test]
fn top_left_pins_image_to_origin() {
    // object-position: 0% 0%（左上角）→ offset (0, 0)，非居中。
    let pos = Bp::TwoValue(Box::new(Bp::Percent(0.0)), Box::new(Bp::Percent(0.0)));
    let (x, y, w, h) = compute_object_fit_rect(&cover(), &pos, 80.0, 80.0, 100.0, 50.0, 0.0, 0.0);
    assert_eq!((x, y), (0.0, 0.0), "top-left → 0,0 (was -40,0 when centered)");
    assert_eq!((w.round(), h.round()), (160.0, 80.0));
}

#[test]
fn bottom_right_pins_to_far_edge() {
    // object-position: 100% 100%（右下角）→ offset = container - img。
    let pos = Bp::TwoValue(Box::new(Bp::Percent(100.0)), Box::new(Bp::Percent(100.0)));
    let (x, y, _, _) = compute_object_fit_rect(&cover(), &pos, 80.0, 80.0, 100.0, 50.0, 0.0, 0.0);
    // x = 80-160 = -80；y = 80-80 = 0
    assert_eq!((x, y), (-80.0, 0.0));
}

#[test]
fn fill_ignores_position() {
    // Fill 拉伸填满，position 不影响。
    let pos = Bp::TwoValue(Box::new(Bp::Percent(0.0)), Box::new(Bp::Percent(0.0)));
    let (x, y, w, h) = compute_object_fit_rect(&ObjectFitComputedValue::Fill, &pos, 80.0, 60.0, 100.0, 50.0, 5.0, 7.0);
    assert_eq!((x, y, w, h), (5.0, 7.0, 80.0, 60.0));
}

#[test]
fn length_offset_applied() {
    // object-position: 10px 20px → 固定偏移。
    let pos = Bp::TwoValue(Box::new(Bp::Length(10.0)), Box::new(Bp::Length(20.0)));
    let (x, y, _, _) = compute_object_fit_rect(&cover(), &pos, 80.0, 80.0, 100.0, 50.0, 0.0, 0.0);
    assert_eq!((x, y), (10.0, 20.0));
}
