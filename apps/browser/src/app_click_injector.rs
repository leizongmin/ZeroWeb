//! M2-S3c DC-3 点击注入器（goal：compositor-animation-continuity，RFC §2.3）：
//! `ZW_INJECT_CLICKS="x,y@t[;…]"` 环境变量门控的定时左键注入——页面 load 完成后
//! 武装时钟，到期点经与真实输入同一的 `handle_mouse_move`/`handle_mouse_click`
//! 路径下发（命中修正后的 hit_test 由该路径消费，DC-3 场景 c/d 的触发器）。
//!
//! 未设 env / 解析失败 = 禁用（零行为）。坐标为窗口物理像素，与既有鼠标输入
//! handler 同一口径；时刻为页面 load 完成起算的毫秒。

use std::time::Instant;

/// 单个注入点：窗口物理像素坐标 + load 起算毫秒。
struct ClickAt {
    x: f64,
    y: f64,
    at_ms: f64,
}

/// env 门控点击注入器。禁用态 `schedule` 为空，所有方法零行为。
pub struct ClickInjector {
    schedule: Vec<ClickAt>,
    /// 武装时刻（最近一次活跃 tab 页面 load 完成）；`None` = 未武装。
    t0: Option<Instant>,
    next: usize,
}

impl ClickInjector {
    /// 从 `ZW_INJECT_CLICKS` 读取注入计划。格式 `x,y@t[;…]`（逗号/`@`/`;` 分隔，
    /// 数值均为有限十进制数）。env 未设 = 禁用；**非法条目整体禁用**（信任边界：
    /// env 是外部输入，部分生效的计划会掩盖配置错误——失败要显眼，warn + 拒绝）。
    pub fn from_env() -> Self {
        let Ok(spec) = std::env::var("ZW_INJECT_CLICKS") else {
            return Self::disabled();
        };
        match Self::parse(&spec) {
            Some(schedule) => Self {
                schedule,
                t0: None,
                next: 0,
            },
            None => {
                tracing::warn!("ZW_INJECT_CLICKS 解析失败，点击注入禁用 spec={spec:?}");
                Self::disabled()
            }
        }
    }

    /// 解析 `x,y@t[;…]`。任一段非法（缺 `@`/分量、非数、负值、非有限）返回 None。
    fn parse(spec: &str) -> Option<Vec<ClickAt>> {
        let mut schedule = Vec::new();
        for part in spec.split(';') {
            let (coords, at_ms) = part.split_once('@')?;
            let mut fields = coords.split(',');
            let x = fields.next()?.parse::<f64>().ok()?;
            let y = fields.next()?.parse::<f64>().ok()?;
            let at_ms = at_ms.parse::<f64>().ok()?;
            if fields.next().is_some()
                || !x.is_finite()
                || !y.is_finite()
                || !at_ms.is_finite()
                || x < 0.0
                || y < 0.0
                || at_ms < 0.0
            {
                return None;
            }
            schedule.push(ClickAt { x, y, at_ms });
        }
        schedule.sort_by(|a, b| a.at_ms.total_cmp(&b.at_ms));
        Some(schedule)
    }

    /// 禁用态（env 未设 / 解析失败）。
    fn disabled() -> Self {
        Self {
            schedule: Vec::new(),
            t0: None,
            next: 0,
        }
    }

    /// 武装时钟（活跃 tab 页面 load 完成时调用；重复 load 重新从 0 播放）。
    pub fn arm(&mut self) {
        self.arm_at(Instant::now());
    }

    /// 指定时刻武装（测试注入可控 t0）。
    fn arm_at(&mut self, t0: Instant) {
        self.t0 = Some(t0);
        self.next = 0;
    }

    /// 计划条目数（测试断言用）。
    #[cfg(test)]
    fn len(&self) -> usize {
        self.schedule.len()
    }

    /// 是否还有待注入点（武装中且计划未耗尽）——主循环据此保持连续帧。
    pub fn armed(&self) -> bool {
        self.t0.is_some() && self.next < self.schedule.len()
    }

    /// 取出所有已到期的注入点，返回最后一个的坐标（同一帧到期多点时，前面的
    /// 点瞬时被取代——注入计划按可区分间隔编写）。
    pub fn due(&mut self, now: Instant) -> Option<(f64, f64)> {
        let t0 = self.t0?;
        let elapsed_ms = now.duration_since(t0).as_secs_f64() * 1000.0;
        let mut due = None;
        while self.next < self.schedule.len() && self.schedule[self.next].at_ms <= elapsed_ms {
            let click = &self.schedule[self.next];
            due = Some((click.x, click.y));
            self.next += 1;
        }
        due
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn parse_accepts_multiple_points_sorted_by_time() {
        let injector = ClickInjector::parse("150,250@120;100,200@50").expect("合法计划");
        assert_eq!(injector.len(), 2);
        assert_eq!((injector[0].x, injector[0].y, injector[0].at_ms), (100.0, 200.0, 50.0));
        assert_eq!((injector[1].x, injector[1].y, injector[1].at_ms), (150.0, 250.0, 120.0));
    }

    #[test]
    fn parse_rejects_malformed_specs() {
        for spec in [
            "",
            "100,200",
            "100,200@",
            "abc,200@50",
            "100,200@-5",
            "100,200@50;x,y@1",
            "nan,200@50",
        ] {
            assert!(ClickInjector::parse(spec).is_none(), "应拒绝 {spec:?}");
        }
    }

    #[test]
    fn due_emits_points_after_arm_and_exhausts() {
        let mut injector = ClickInjector {
            schedule: ClickInjector::parse("100,200@50;150,250@120").unwrap(),
            t0: None,
            next: 0,
        };
        assert!(!injector.armed(), "未武装零行为");
        let t0 = Instant::now();
        injector.arm_at(t0);
        assert!(injector.armed());
        assert_eq!(injector.due(t0 + Duration::from_millis(49)), None, "未到期");
        assert_eq!(
            injector.due(t0 + Duration::from_millis(60)),
            Some((100.0, 200.0)),
            "首点到"
        );
        assert_eq!(injector.due(t0 + Duration::from_millis(60)), None, "同刻不重复");
        assert_eq!(
            injector.due(t0 + Duration::from_millis(200)),
            Some((150.0, 250.0)),
            "次点到"
        );
        assert!(!injector.armed(), "计划耗尽");
    }
}
