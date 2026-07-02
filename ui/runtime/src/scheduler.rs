//! 调度器 — 延迟回调 / 动画 tick（spec §8.4.1 `scheduler.rs`）。
//!
//! 使用整数 tick（纳秒或毫秒由调用方约定），便于 `ui/testing::fake_clock` 确定性驱动。

/// 调度任务 id。
pub type TaskId = u64;

/// 基于整数 tick 的调度器。
#[derive(Debug, Default)]
pub struct Scheduler {
    tasks: Vec<(TaskId, u64)>, // (id, due_tick)
    next_id: TaskId,
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler::default()
    }

    /// 安排一个在 `due_tick` 到期的任务，返回其 id。
    pub fn schedule(&mut self, due_tick: u64) -> TaskId {
        let id = self.next_id;
        self.next_id += 1;
        self.tasks.push((id, due_tick));
        self.tasks.sort_by_key(|(_, t)| *t);
        id
    }

    /// 取消任务。
    pub fn cancel(&mut self, id: TaskId) {
        self.tasks.retain(|(tid, _)| *tid != id);
    }

    /// 推进到 `now`，返回所有到期（due <= now）任务 id（按到期顺序），并从队列移除。
    pub fn drain_due(&mut self, now: u64) -> Vec<TaskId> {
        let mut due = Vec::new();
        let mut rest = Vec::new();
        for (id, t) in self.tasks.drain(..) {
            if t <= now {
                due.push(id);
            } else {
                rest.push((id, t));
            }
        }
        self.tasks = rest;
        due
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drain_due_in_order() {
        let mut s = Scheduler::new();
        let a = s.schedule(100);
        let b = s.schedule(50);
        let c = s.schedule(150);
        let due = s.drain_due(100);
        // 50 与 100 到期，按到期顺序。
        assert_eq!(due, vec![b, a]);
        assert_eq!(s.len(), 1);
        let due2 = s.drain_due(200);
        assert_eq!(due2, vec![c]);
    }

    #[test]
    fn cancel_removes_task() {
        let mut s = Scheduler::new();
        let a = s.schedule(10);
        s.cancel(a);
        assert!(s.drain_due(100).is_empty());
    }
}
