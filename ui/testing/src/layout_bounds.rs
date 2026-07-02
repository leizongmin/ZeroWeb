//! Layout bounds snapshot — 把「WidgetId → Rect」列表拍平为确定性字符串（spec FR-016 testing）。
//!
//! 供 DC-2（flex）/ DC-9（invalidation）/ DC-12（adaptive）等布局断言做 golden 对比：
//! 给定一组节点的绝对 rect，序列化为 `id rect=l,t,r,b`（按输入顺序），与黄金串比较。

use zero_ui_core::geometry::Rect;
use zero_ui_core::widget::WidgetId;

/// 生成布局边界快照：每行 `<id> rect=<l>,<t>,<r>,<b>`，按输入顺序（= 声明/查询顺序）。
pub fn snapshot_layout_bounds(bounds: &[(WidgetId, Rect)]) -> String {
    let mut out = String::new();
    for (id, r) in bounds {
        out.push_str(&format!(
            "{} rect={},{},{},{}\n",
            id.0.as_str(),
            r.left(),
            r.top(),
            r.right(),
            r.bottom()
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_ui_core::geometry::Point;

    #[test]
    fn snapshot_serializes_bounds_in_order() {
        let bounds = vec![
            (
                WidgetId::new("toolbar"),
                Rect::from_origin_size(Point::ZERO, zero_ui_core::geometry::Size::new(400.0, 32.0)),
            ),
            (WidgetId::new("address"), Rect::from_ltrb(0.0, 0.0, 360.0, 32.0)),
        ];
        let snap = snapshot_layout_bounds(&bounds);
        assert!(snap.contains("toolbar rect=0,0,400,32"), "got: {snap}");
        assert!(snap.contains("address rect=0,0,360,32"), "got: {snap}");
        // 顺序保留（toolbar 在 address 前）。
        let tb = snap.find("toolbar").unwrap();
        let ad = snap.find("address").unwrap();
        assert!(tb < ad);
    }

    #[test]
    fn empty_bounds_produce_empty_snapshot() {
        assert!(snapshot_layout_bounds(&[]).is_empty());
    }
}
