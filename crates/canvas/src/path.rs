//! 路径表示 — Canvas 2D 路径命令与 Path2D。

/// 路径命令。
#[derive(Debug, Clone)]
pub enum PathCommand {
    /// 移动到指定点。
    MoveTo(f32, f32),
    /// 画线到指定点。
    LineTo(f32, f32),
    /// 二次贝塞尔曲线。
    QuadraticCurveTo(f32, f32, f32, f32),
    /// 三次贝塞尔曲线。
    BezierCurveTo(f32, f32, f32, f32, f32, f32),
    /// 圆弧。
    Arc(f32, f32, f32, f32, f32),
    /// 圆弧切线（控制点1 x, 控制点1 y, 控制点2 x, 控制点2 y, 半径）。
    ArcTo(f32, f32, f32, f32, f32),
    /// 椭圆弧（圆心 x, 圆心 y, 半径 x, 半径 y, 旋转, 起始角, 结束角）。
    Ellipse(f32, f32, f32, f32, f32, f32, f32),
    /// 圆角矩形子路径（x, y, 宽, 高, 圆角半径列表）。
    RoundRect(f32, f32, f32, f32, Vec<f32>),
    /// 闭合路径。
    ClosePath,
}

/// 2D 路径 — 存储 Canvas 路径命令序列。
#[derive(Debug, Clone, Default)]
pub struct Path2D {
    commands: Vec<PathCommand>,
}

impl Path2D {
    /// 创建空路径。
    pub fn new() -> Self {
        Self::default()
    }

    /// 移动到指定点。
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.commands.push(PathCommand::MoveTo(x, y));
    }

    /// 画线到指定点。
    pub fn line_to(&mut self, x: f32, y: f32) {
        self.commands.push(PathCommand::LineTo(x, y));
    }

    /// 闭合路径。
    pub fn close_path(&mut self) {
        self.commands.push(PathCommand::ClosePath);
    }

    /// 添加圆弧。
    pub fn arc(&mut self, x: f32, y: f32, radius: f32, start: f32, end: f32) {
        self.commands.push(PathCommand::Arc(x, y, radius, start, end));
    }

    /// 添加圆弧切线（arcTo）。
    pub fn arc_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, radius: f32) {
        self.commands.push(PathCommand::ArcTo(x1, y1, x2, y2, radius));
    }

    /// 添加二次贝塞尔曲线。
    pub fn quadratic_curve_to(&mut self, cpx: f32, cpy: f32, x: f32, y: f32) {
        self.commands.push(PathCommand::QuadraticCurveTo(cpx, cpy, x, y));
    }

    /// 添加三次贝塞尔曲线。
    pub fn bezier_curve_to(&mut self, cp1x: f32, cp1y: f32, cp2x: f32, cp2y: f32, x: f32, y: f32) {
        self.commands
            .push(PathCommand::BezierCurveTo(cp1x, cp1y, cp2x, cp2y, x, y));
    }

    /// 添加椭圆弧。
    #[allow(clippy::too_many_arguments)]
    pub fn ellipse(
        &mut self,
        cx: f32,
        cy: f32,
        radius_x: f32,
        radius_y: f32,
        rotation: f32,
        start_angle: f32,
        end_angle: f32,
    ) {
        self.commands.push(PathCommand::Ellipse(
            cx,
            cy,
            radius_x,
            radius_y,
            rotation,
            start_angle,
            end_angle,
        ));
    }

    /// 添加矩形子路径（四个 line_to + close）。
    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.commands.push(PathCommand::MoveTo(x, y));
        self.commands.push(PathCommand::LineTo(x + w, y));
        self.commands.push(PathCommand::LineTo(x + w, y + h));
        self.commands.push(PathCommand::LineTo(x, y + h));
        self.commands.push(PathCommand::ClosePath);
    }

    /// 添加圆角矩形子路径。
    pub fn round_rect(&mut self, x: f32, y: f32, w: f32, h: f32, radii: Vec<f32>) {
        self.commands.push(PathCommand::RoundRect(x, y, w, h, radii));
    }

    /// 返回路径命令数量。
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// 返回路径命令列表。
    pub fn commands(&self) -> &[PathCommand] {
        &self.commands
    }

    /// 返回路径命令列表的可变引用。
    pub fn commands_mut(&mut self) -> &mut Vec<PathCommand> {
        &mut self.commands
    }

    /// 路径是否为空。
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// 清空路径。
    pub fn clear(&mut self) {
        self.commands.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_new() {
        let p = Path2D::new();
        assert!(p.is_empty());
        assert!(p.commands().is_empty());
    }

    #[test]
    fn test_path_move_to() {
        let mut p = Path2D::new();
        p.move_to(10.0, 20.0);
        assert_eq!(p.commands().len(), 1);
        assert!(matches!(p.commands()[0], PathCommand::MoveTo(10.0, 20.0)));
    }

    #[test]
    fn test_path_line_to() {
        let mut p = Path2D::new();
        p.move_to(0.0, 0.0);
        p.line_to(100.0, 50.0);
        assert_eq!(p.commands().len(), 2);
        assert!(matches!(p.commands()[1], PathCommand::LineTo(100.0, 50.0)));
    }

    #[test]
    fn test_path_close() {
        let mut p = Path2D::new();
        p.move_to(0.0, 0.0);
        p.line_to(10.0, 0.0);
        p.close_path();
        assert!(matches!(p.commands().last(), Some(PathCommand::ClosePath)));
    }

    #[test]
    fn test_path_arc() {
        let mut p = Path2D::new();
        p.arc(50.0, 50.0, 25.0, 0.0, std::f32::consts::PI);
        assert_eq!(p.commands().len(), 1);
        assert!(matches!(p.commands()[0], PathCommand::Arc(50.0, 50.0, 25.0, 0.0, _)));
    }

    #[test]
    fn test_path_rect() {
        let mut p = Path2D::new();
        p.rect(10.0, 20.0, 100.0, 50.0);
        // rect = MoveTo + 3x LineTo + ClosePath
        assert_eq!(p.commands().len(), 5);
        assert!(matches!(p.commands()[0], PathCommand::MoveTo(10.0, 20.0)));
        assert!(matches!(p.commands()[4], PathCommand::ClosePath));
    }

    #[test]
    fn test_path_clear() {
        let mut p = Path2D::new();
        p.move_to(0.0, 0.0);
        p.line_to(10.0, 10.0);
        assert!(!p.is_empty());
        p.clear();
        assert!(p.is_empty());
    }

    #[test]
    fn test_path_quadratic_curve_to() {
        let mut p = Path2D::new();
        p.move_to(0.0, 0.0);
        p.commands_mut()
            .push(PathCommand::QuadraticCurveTo(10.0, 20.0, 30.0, 40.0));
        assert_eq!(p.commands().len(), 2);
        assert!(matches!(
            p.commands()[1],
            PathCommand::QuadraticCurveTo(10.0, 20.0, 30.0, 40.0)
        ));
    }

    #[test]
    fn test_path_bezier_curve_to() {
        let mut p = Path2D::new();
        p.move_to(0.0, 0.0);
        p.commands_mut()
            .push(PathCommand::BezierCurveTo(1.0, 2.0, 3.0, 4.0, 5.0, 6.0));
        assert_eq!(p.commands().len(), 2);
        assert!(matches!(
            p.commands()[1],
            PathCommand::BezierCurveTo(1.0, 2.0, 3.0, 4.0, 5.0, 6.0)
        ));
    }

    #[test]
    fn test_path_commands_mut_modification() {
        let mut p = Path2D::new();
        p.move_to(0.0, 0.0);
        p.commands_mut().push(PathCommand::LineTo(5.0, 5.0));
        p.commands_mut().remove(0);
        assert_eq!(p.commands().len(), 1);
    }

    #[test]
    fn test_path_default_trait() {
        let p = Path2D::default();
        assert!(p.is_empty());
    }

    #[test]
    fn test_path_clone_equality() {
        let mut p = Path2D::new();
        p.move_to(1.0, 2.0);
        p.line_to(3.0, 4.0);
        let cloned = p.clone();
        assert_eq!(p.commands().len(), cloned.commands().len());
    }

    #[test]
    fn test_path_multiple_rects() {
        let mut p = Path2D::new();
        p.rect(0.0, 0.0, 10.0, 10.0);
        p.rect(20.0, 20.0, 10.0, 10.0);
        assert_eq!(p.commands().len(), 10); // 2 × 5
    }
}
