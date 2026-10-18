//! dnd-compat M3：宿主拖放会话状态机（HTML §6.7 drag-and-drop processing model 的 UA 侧骨架）。
//!
//! 纯数据/无 sandbox 依赖——三个消费方共享同一实现，避免语料驱动面与真窗口输入面分叉（防假面）：
//! - 桌面 renderer IPC（browser 仲裁后经 DragInput 消息驱动，s3 接线）；
//! - WPT runner 直接调用（dnd 语料注入通道，s4 接线）；
//! - 外部嵌入宿主经 [`WebView::apply_drag_input`]（本模块胶水）。
//!
//! 事件序（spec https://html.spec.whatwg.org/multipage/dnd.html#drag-and-drop-processing-model）：
//! mousedown 命中可拖源（宿主按 `ElementHit::drag_source` 仲裁后 `arm`）→ 首个超过
//! 阈值的 move 派 `dragstart`（cancelable）→ 未取消则同批派 `drag` + 首目标 `dragenter`/`dragover`
//! → 后续 move 派 `dragover`（目标变化时前置 `dragleave`/`dragenter`，边界事件携带
//! `relatedTarget`——leave = 新目标、enter = 上次边界目标，初始源不算既有目标）→ up 时按最近一次
//! dragenter/over 是否被取消决定 `drop`（cancelable）或 `dragleave`，两分支都收尾 `dragend`
//! （不可取消）→ Esc 走取消步（`dragleave` + `dragend`）。dragstart 被取消 → 会话不启动、
//! 事件流回归点击语义。
//!
//! 取消反馈：宿主派发完一个计划后必须调 [`DragInputState::report_default_prevented`] 回填
//! 「该批最后一个可取消事件的 defaultPrevented」（shim `__zw_dispatch_event` 返 `'prevented'`），
//! 状态机据此决定是否继续（dragstart 取消）或记录 drop 资格（dragover 取消）。

use super::{WebView, WebViewError};
use zero_engine::DomEventDetail;

/// dragstart 触发位移阈值（CSS 逻辑像素）——Armed 后位移小于该值不启动拖拽（点击不误触）。
// FIXME(TBD-3): 初值取小值保语料确定性；corpus 首跑后按 WPT Actions 步长校准。
pub const DRAG_START_THRESHOLD_PX: f32 = 2.0;

/// 指针输入相位（宿主输入面的单入口语义；`Down` 只做武装判定，不派事件——mousedown
/// 本身走既有鼠标管线，避免双派）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragPointerPhase {
    /// mousedown——命中可拖源（`ElementHit::drag_source`）时武装会话。
    Down,
    /// mousemove。
    Move,
    /// mouseup。
    Up,
    /// Esc 等宿主取消（优先级高于 chrome 快捷键，browser 仲裁路由）。
    Cancel,
}

/// 一条待派发 drag* 事件的宿主计划。
#[derive(Debug, Clone, PartialEq)]
pub struct DragEventPlan {
    /// 目标元素唯一选择器（dragstart/drag 恒为源；enter/over/leave/drop 为命中目标）。
    pub selector: String,
    /// 事件类型（dragstart/drag/dragenter/dragover/dragleave/drop/dragend 之一）。
    pub event_type: &'static str,
    /// 视口 CSS 逻辑像素坐标（MouseEvent.clientX/Y 面）。
    pub client_x: f32,
    /// 同上（Y 轴）。
    pub client_y: f32,
    /// 会话 id——同会话全部事件共享同一 DataTransfer 实例（shim `__zwDragSessions[id]`）。
    pub session: u64,
    /// `DragEvent.relatedTarget`——边界事件语义（spec fire a DND event：dragleave =
    /// 正在进入的新目标；dragenter = 上次边界迁移目标）。非边界事件恒 `None`。
    pub related_target: Option<String>,
}

/// 待报告的取消判定点（宿主派发完该批事件后以
/// [`DragInputState::report_default_prevented`] 回填批内**最后一个**可取消事件的结局）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outstanding {
    /// dragstart 已派发——取消 → 会话不启动；通过 → 同批补派 drag + 首目标 enter/over。
    Start,
    /// 批以 dragover 收尾——取消 → 目标拒绝（operation none）；通过 → 记录 drop 资格。
    Over,
}

/// 拖放会话状态机内部态。
#[derive(Debug, Default)]
enum DragInner {
    /// 无会话；此态下 report 抵达一律静默忽略。
    #[default]
    Idle,
    /// mousedown 武装（源已锁定），位移未过阈值；Up → 静默回 Idle（点击流接管）。
    Armed {
        source: String,
        x: f32,
        y: f32,
        session: u64,
    },
    /// dragstart 已派发，待 `report_default_prevented` 判定（坐标 = 触发 move 的指针位，
    /// 补派 drag 同位——spec 同一指针事件内的连续派发）。`target` = 该 move 的命中落点：
    /// 接受后按此做**初始目标更新**（落点在源外时接受批内直接完成源→落点边界迁移——
    /// UA 逐 move 处理模型，Chrome/corpus 口径）。
    StartDispatched {
        source: String,
        x: f32,
        y: f32,
        session: u64,
        target: Option<String>,
    },
    /// 会话进行中。`current_target` = 最近一次 dragenter 的新目标（离开全部元素 → None）；
    /// `over_accepted` = 最近一次 dragenter/over 未被取消（drop 资格判据；None = 尚未报告）；
    /// `boundary_prev` = 上次**更新期边界迁移**进入的目标（初始接受批的源自身不算——
    /// dragenter.relatedTarget 语义，corpus 口径首个真目标 related 为 null）。
    Dragging {
        source: String,
        current_target: Option<String>,
        session: u64,
        over_accepted: Option<bool>,
        boundary_prev: Option<String>,
    },
}

/// 宿主拖放会话状态机——见模块文档事件序。默认 Idle。
#[derive(Debug, Default)]
pub struct DragInputState {
    inner: DragInner,
    /// 下一个会话 id（从 1 起；0 留给 shim 侧缺省兜底）。
    next_session: u64,
    /// 当前待回填的取消判定点（None 时 report 静默）。
    outstanding: Option<Outstanding>,
}

impl DragInputState {
    /// 新建 Idle 状态机。
    pub fn new() -> Self {
        Self {
            next_session: 1,
            ..Default::default()
        }
    }

    /// 是否有活跃会话（Armed/StartDispatched/Dragging）——宿主仲裁面：true 时抑制
    /// 既有 scroll-drag 手势、click 合成与 chrome 快捷键旁路。
    pub fn is_active(&self) -> bool {
        !matches!(self.inner, DragInner::Idle)
    }

    /// mousedown 武装会话（宿主已按 `drag_source` 仲裁；仅 Idle 态生效）。
    pub fn arm(&mut self, source: String, x: f32, y: f32) {
        if self.is_active() {
            return; // 防御：会话进行中不重武装（browser 仲裁保证不发生）
        }
        let session = self.next_session;
        self.next_session += 1;
        self.inner = DragInner::Armed { source, x, y, session };
    }

    /// 指针步进/抬升/取消——生成按序派发的 drag* 事件计划（见模块文档事件序）。
    /// `target` 为 `(x, y)` 命中元素选择器（hit-test 无命中 → `None`）。
    pub fn on_pointer(
        &mut self,
        phase: DragPointerPhase,
        x: f32,
        y: f32,
        target: Option<String>,
    ) -> Vec<DragEventPlan> {
        // mem::take 取出内部态消除「匹配借用 vs 赋值」冲突；不替换态的分支回收原值。
        let inner = std::mem::take(&mut self.inner);
        match (inner, phase) {
            (idle @ DragInner::Idle, _) => {
                self.inner = idle;
                Vec::new()
            }
            (active, DragPointerPhase::Down) => {
                // 会话中忽略重复 Down（武装仅 Idle 态生效）。
                self.inner = active;
                Vec::new()
            }
            (
                DragInner::Armed {
                    source,
                    x: sx,
                    y: sy,
                    session,
                },
                DragPointerPhase::Move,
            ) => {
                let dx = x - sx;
                let dy = y - sy;
                if dx * dx + dy * dy < DRAG_START_THRESHOLD_PX * DRAG_START_THRESHOLD_PX {
                    self.inner = DragInner::Armed {
                        source,
                        x: sx,
                        y: sy,
                        session,
                    };
                    return Vec::new(); // 未过阈值——静默
                }
                self.inner = DragInner::StartDispatched {
                    source: source.clone(),
                    x,
                    y,
                    session,
                    target,
                };
                self.outstanding = Some(Outstanding::Start);
                vec![DragEventPlan {
                    selector: source,
                    event_type: "dragstart",
                    client_x: x,
                    client_y: y,
                    session,
                    related_target: None,
                }]
            }
            (
                pending @ DragInner::StartDispatched { .. },
                DragPointerPhase::Move | DragPointerPhase::Up | DragPointerPhase::Cancel,
            ) => {
                // dragstart 未判定前后续指针事件不产出（宿主按序派发，不会到达此分支）。
                self.inner = pending;
                Vec::new()
            }
            (armed @ DragInner::Armed { .. }, DragPointerPhase::Up | DragPointerPhase::Cancel) => {
                let _ = armed; // 未启动拖拽——up 交还点击流，取消步无事可做
                self.inner = DragInner::Idle;
                self.outstanding = None;
                Vec::new()
            }
            (
                DragInner::Dragging {
                    source,
                    current_target,
                    session,
                    over_accepted,
                    boundary_prev,
                },
                DragPointerPhase::Move,
            ) => {
                if target.as_ref() == current_target.as_ref() {
                    // 同目标：持续派 dragover（spec 连续派发；逐 move 派发保证语料确定性——TBD-1
                    // 周期性重派待 corpus 证据定）。悬空（无目标）无事可派。over_accepted 保留
                    // 上一批判定值（新报告抵达前它仍是最新已知状态）。
                    self.inner = DragInner::Dragging {
                        source: source.clone(),
                        current_target: current_target.clone(),
                        session,
                        over_accepted,
                        boundary_prev,
                    };
                    match current_target {
                        Some(sel) => {
                            self.outstanding = Some(Outstanding::Over);
                            vec![DragEventPlan {
                                selector: sel,
                                event_type: "dragover",
                                client_x: x,
                                client_y: y,
                                session,
                                related_target: None,
                            }]
                        }
                        None => Vec::new(),
                    }
                } else {
                    // 目标变化：旧目标 dragleave（不可取消；related = 正在进入的新目标，spec
                    // fire a DND event）→ 新目标 dragenter（related = 上次边界迁移目标；初始
                    // 接受批的源自身不算既有拖拽目标，首个真目标 related 为 null——corpus
                    // `events/dragenter-dragleave-related-target` 口径）+ dragover。
                    let mut plans = Vec::new();
                    if let Some(old) = current_target.clone() {
                        plans.push(DragEventPlan {
                            selector: old,
                            event_type: "dragleave",
                            client_x: x,
                            client_y: y,
                            session,
                            related_target: target.clone(),
                        });
                    }
                    if let Some(new) = target.clone() {
                        plans.push(DragEventPlan {
                            selector: new.clone(),
                            event_type: "dragenter",
                            client_x: x,
                            client_y: y,
                            session,
                            related_target: boundary_prev.clone(),
                        });
                        plans.push(DragEventPlan {
                            selector: new,
                            event_type: "dragover",
                            client_x: x,
                            client_y: y,
                            session,
                            related_target: None,
                        });
                        self.outstanding = Some(Outstanding::Over);
                    } else {
                        // 离开全部元素：drop 资格清零；边界链清零（再入目标按首入语义 related=null）。
                        self.outstanding = None;
                    }
                    self.inner = DragInner::Dragging {
                        current_target: target.clone(),
                        source,
                        session,
                        over_accepted: None,
                        boundary_prev: target,
                    };
                    plans
                }
            }
            (
                DragInner::Dragging {
                    source,
                    current_target,
                    session,
                    over_accepted,
                    ..
                },
                DragPointerPhase::Up,
            ) => {
                self.inner = DragInner::Idle;
                self.outstanding = None; // dragend 不可取消；drop 结局不驱动后续事件
                match (current_target, over_accepted) {
                    (Some(target), Some(true)) => vec![
                        DragEventPlan {
                            selector: target,
                            event_type: "drop",
                            client_x: x,
                            client_y: y,
                            session,
                            related_target: None,
                        },
                        DragEventPlan {
                            selector: source,
                            event_type: "dragend",
                            client_x: x,
                            client_y: y,
                            session,
                            related_target: None,
                        },
                    ],
                    (Some(target), _) => vec![
                        // 目标拒绝（dragover/enter 被取消或未报告）——operation none，不 drop。
                        // spec 释放步 dragleave 无 specific related target → null。
                        DragEventPlan {
                            selector: target,
                            event_type: "dragleave",
                            client_x: x,
                            client_y: y,
                            session,
                            related_target: None,
                        },
                        DragEventPlan {
                            selector: source,
                            event_type: "dragend",
                            client_x: x,
                            client_y: y,
                            session,
                            related_target: None,
                        },
                    ],
                    (None, _) => vec![DragEventPlan {
                        selector: source,
                        event_type: "dragend",
                        client_x: x,
                        client_y: y,
                        session,
                        related_target: None,
                    }],
                }
            }
            (
                DragInner::Dragging {
                    source,
                    current_target,
                    session,
                    ..
                },
                DragPointerPhase::Cancel,
            ) => {
                // spec 取消步：对当前目标派 dragleave（无 specific related target），
                // 随后源上派 dragend（不可取消）。
                self.inner = DragInner::Idle;
                self.outstanding = None;
                let mut plans = Vec::new();
                if let Some(target) = current_target {
                    plans.push(DragEventPlan {
                        selector: target,
                        event_type: "dragleave",
                        client_x: x,
                        client_y: y,
                        session,
                        related_target: None,
                    });
                }
                plans.push(DragEventPlan {
                    selector: source,
                    event_type: "dragend",
                    client_x: x,
                    client_y: y,
                    session,
                    related_target: None,
                });
                plans
            }
        }
    }

    /// 回填上一批计划中**最后一个可取消事件**的 `defaultPrevented`
    /// （shim `__zw_dispatch_event` 返 `'prevented'` → `true`）。返回追补事件计划
    ///（仅 dragstart 通过时返 `drag` + 初始目标更新批——落点在源上为
    /// `dragenter`/`dragover`@source，在源外为源→落点边界迁移三连，其余返空）。
    pub fn report_default_prevented(&mut self, prevented: bool) -> Vec<DragEventPlan> {
        let outstanding = self.outstanding.take();
        let inner = std::mem::take(&mut self.inner);
        match (outstanding, inner) {
            (
                Some(Outstanding::Start),
                DragInner::StartDispatched {
                    source,
                    x,
                    y,
                    session,
                    target,
                },
            ) => {
                if prevented {
                    // dragstart 被取消 → 拖拽操作不启动（spec），事件流回归点击语义。
                    self.inner = DragInner::Idle;
                    Vec::new()
                } else {
                    // 接受批：drag（源，不可取消）+ **初始目标更新**——落点仍在源上 →
                    // enter/over@source（指针未离开按下点的常规起拖）；落点已在源外
                    //（起拖 move 一次到位）→ 接受批内完成源→落点边界迁移（UA 逐 move
                    // 处理模型：dragstart 后 UA 立即按当前指针位置跑目标更新步——
                    // Chrome/corpus 口径，enter(新).related=null：源不算既有拖拽目标）。
                    // 批以 over 收尾 → 待报告（悬空落点 dragleave 收尾，无待判定）。
                    let away = matches!(&target, Some(t) if *t != source);
                    self.inner = DragInner::Dragging {
                        current_target: match &target {
                            Some(t) if *t != source => target.clone(),
                            Some(_) => Some(source.clone()),
                            None => None,
                        },
                        boundary_prev: if away { target.clone() } else { None },
                        source: source.clone(),
                        session,
                        over_accepted: None,
                    };
                    let mut batch = vec![DragEventPlan {
                        selector: source.clone(),
                        event_type: "drag",
                        client_x: x,
                        client_y: y,
                        session,
                        related_target: None,
                    }];
                    match target {
                        Some(t) if t != source => {
                            batch.push(DragEventPlan {
                                selector: source.clone(),
                                event_type: "dragleave",
                                client_x: x,
                                client_y: y,
                                session,
                                related_target: Some(t.clone()),
                            });
                            batch.push(DragEventPlan {
                                selector: t.clone(),
                                event_type: "dragenter",
                                client_x: x,
                                client_y: y,
                                session,
                                related_target: None,
                            });
                            batch.push(DragEventPlan {
                                selector: t,
                                event_type: "dragover",
                                client_x: x,
                                client_y: y,
                                session,
                                related_target: None,
                            });
                            self.outstanding = Some(Outstanding::Over);
                        }
                        Some(sel) => {
                            batch.push(DragEventPlan {
                                selector: sel,
                                event_type: "dragenter",
                                client_x: x,
                                client_y: y,
                                session,
                                related_target: None,
                            });
                            batch.push(DragEventPlan {
                                selector: source,
                                event_type: "dragover",
                                client_x: x,
                                client_y: y,
                                session,
                                related_target: None,
                            });
                            self.outstanding = Some(Outstanding::Over);
                        }
                        None => {
                            // 悬空落点（指针已离源、无新目标）：源 dragleave（related=null），
                            // current_target 归 None；批无尾 over → 无待判定。
                            batch.push(DragEventPlan {
                                selector: source,
                                event_type: "dragleave",
                                client_x: x,
                                client_y: y,
                                session,
                                related_target: None,
                            });
                            self.outstanding = None;
                        }
                    }
                    batch
                }
            }
            (
                Some(Outstanding::Over),
                DragInner::Dragging {
                    source,
                    current_target,
                    session,
                    boundary_prev,
                    ..
                },
            ) => {
                let over_accepted = Some(!prevented);
                self.inner = DragInner::Dragging {
                    source,
                    current_target,
                    session,
                    over_accepted,
                    boundary_prev,
                };
                Vec::new()
            }
            (outstanding, inner) => {
                // 无待判定（Idle/批尾不可取消/批间未报告）——静默，回收原态不丢状态。
                let _ = outstanding;
                self.inner = inner;
                Vec::new()
            }
        }
    }
}

/// [`WebView::apply_drag_input`] 的应用结果——宿主仲裁与测试断言面。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DragApplyReport {
    /// 本次输入是否派发了 drag* 事件（true → 宿主抑制 click 合成 / scroll-drag 等既有手势）。
    pub consumed: bool,
    /// 最后派发的可取消事件是否被页面取消（dragstart 取消 = 会话未启动；dragover 取消 = 目标拒绝）。
    pub prevented: bool,
    /// 拖放会话是否处于拖拽中（dragstart 接受后至 dragend 前）——此窗口真实浏览器
    /// 抑制鼠标/指针事件面（mousemove/mouseup 被 drop/dragend 替代）；runner 注入与
    /// 宿主仲裁据此并行维护鼠标面（dnd-compat M3 RFC §5.6）。
    pub session_active: bool,
}

impl WebView {
    /// 拖放输入单入口：命中查询 → 状态机推进 → 按序派发计划（`dispatch_dom_event_inner`
    /// 同一宿主通道，坐标/会话 id 经 detail 透传）→ 回填取消判定直至批空。
    ///
    /// 命中：`Down` 按 [`zero_engine::ElementHit::drag_source`]（最近可拖祖先或自身）仲裁武装；
    /// `Move`/`Up` 以命中选择器为当前目标；hit-test 缓存未注入 / 无命中 → `None`（悬空）。
    /// `Cancel` 不查命中（Esc 取消当前会话）。
    pub fn apply_drag_input(
        &mut self,
        phase: DragPointerPhase,
        x: f32,
        y: f32,
    ) -> Result<DragApplyReport, WebViewError> {
        let hit = self.hit_test_element(x, y);
        let target = hit.as_ref().map(|h| h.selector.clone());
        let mut state = std::mem::take(&mut self.drag_input);
        let mut batch = match phase {
            DragPointerPhase::Down => {
                // 武装源 = 最近可拖祖先或自身的选择器（dragstart 派至该源而非命中元素）。
                if let Some(source) = hit.as_ref().and_then(|h| h.drag_source.clone()) {
                    state.arm(source, x, y);
                }
                Vec::new()
            }
            DragPointerPhase::Move => state.on_pointer(phase, x, y, target.clone()),
            DragPointerPhase::Up => state.on_pointer(phase, x, y, target),
            DragPointerPhase::Cancel => state.on_pointer(phase, x, y, None),
        };
        let mut consumed = false;
        let mut prevented = false;
        let mut guard = 0;
        while !batch.is_empty() {
            guard += 1;
            if guard > 4 {
                break; // 防御：正常至多两批（dragstart 批 + 补派批）；状态机不变式破坏时截断
            }
            let mut last_prevented = false;
            for plan in &batch {
                let detail = DomEventDetail {
                    client_x: Some(plan.client_x),
                    client_y: Some(plan.client_y),
                    data_transfer_id: Some(plan.session),
                    related_target: plan.related_target.clone(),
                    ..Default::default()
                };
                last_prevented =
                    self.dispatch_dom_event_inner(&plan.selector, plan.event_type, Some(&detail))? == "prevented";
            }
            consumed = true;
            prevented = last_prevented;
            batch = state.report_default_prevented(last_prevented);
        }
        self.drag_input = state;
        let session_active = matches!(self.drag_input.inner, DragInner::Dragging { .. });
        Ok(DragApplyReport {
            consumed,
            prevented,
            session_active,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 武装后小幅移动（< [`DRAG_START_THRESHOLD_PX`]）不产出事件，过阈值派 dragstart。
    #[test]
    fn threshold_quiets_jitter_then_starts() {
        let mut s = DragInputState::new();
        s.arm("div#src".into(), 10.0, 10.0);
        assert!(s.is_active());
        assert_eq!(
            s.on_pointer(DragPointerPhase::Move, 11.0, 10.0, Some("div#src".into())),
            Vec::new()
        );
        let batch = s.on_pointer(DragPointerPhase::Move, 40.0, 30.0, Some("div#src".into()));
        assert_eq!(batch.len(), 1);
        assert_eq!(batch[0].event_type, "dragstart");
        assert_eq!(batch[0].selector, "div#src");
        assert_eq!(batch[0].session, 1);
    }

    /// dragstart 通过 → 补派 drag + 首目标（源自身）enter/over；后续同目标 move 仅派 dragover。
    #[test]
    fn start_accepted_then_over_per_move() {
        let mut s = DragInputState::new();
        s.arm("div#src".into(), 0.0, 0.0);
        assert_eq!(
            s.on_pointer(DragPointerPhase::Move, 30.0, 30.0, Some("div#src".into()))
                .len(),
            1
        );
        let follow = s.report_default_prevented(false);
        assert_eq!(
            follow.iter().map(|p| p.event_type).collect::<Vec<_>>(),
            ["drag", "dragenter", "dragover"]
        );
        assert!(follow.iter().all(|p| p.selector == "div#src" && p.session == 1));
        // 同目标 move → 仅 dragover。
        let over = s.on_pointer(DragPointerPhase::Move, 45.0, 35.0, Some("div#src".into()));
        assert_eq!(over.len(), 1);
        assert_eq!(over[0].event_type, "dragover");
    }

    /// dragstart 被取消 → 会话不启动（Idle），后续 move 不再产出。
    #[test]
    fn start_refused_returns_to_idle() {
        let mut s = DragInputState::new();
        s.arm("div#src".into(), 0.0, 0.0);
        assert_eq!(
            s.on_pointer(DragPointerPhase::Move, 30.0, 30.0, Some("div#src".into()))
                .len(),
            1
        );
        assert_eq!(s.report_default_prevented(true), Vec::new());
        assert!(!s.is_active());
        assert_eq!(
            s.on_pointer(DragPointerPhase::Move, 60.0, 60.0, Some("div#t".into())),
            Vec::new()
        );
    }

    /// 目标变化 → [dragleave(旧), dragenter(新), dragover(新)]；离开全部元素 → [dragleave]。
    #[test]
    fn target_change_dispatches_boundary_sequence() {
        let mut s = DragInputState::new();
        s.arm("div#src".into(), 0.0, 0.0);
        assert_eq!(
            s.on_pointer(DragPointerPhase::Move, 30.0, 30.0, Some("div#src".into()))
                .len(),
            1
        );
        assert_eq!(s.report_default_prevented(false).len(), 3);
        let change = s.on_pointer(DragPointerPhase::Move, 60.0, 60.0, Some("div#dst".into()));
        assert_eq!(
            change
                .iter()
                .map(|p| (p.event_type, p.selector.as_str()))
                .collect::<Vec<_>>(),
            [
                ("dragleave", "div#src"),
                ("dragenter", "div#dst"),
                ("dragover", "div#dst")
            ]
        );
        let off = s.on_pointer(DragPointerPhase::Move, 90.0, 90.0, None);
        assert_eq!(off.len(), 1);
        assert_eq!(off[0].event_type, "dragleave");
    }

    /// 最近 dragover 通过 → up 派 [drop, dragend]；被拒/未报告 → [dragleave, dragend]。
    #[test]
    fn release_gates_drop_on_last_over() {
        let mut accepted = DragInputState::new();
        accepted.arm("div#src".into(), 0.0, 0.0);
        assert_eq!(
            accepted
                .on_pointer(DragPointerPhase::Move, 30.0, 30.0, Some("div#src".into()))
                .len(),
            1
        );
        assert_eq!(accepted.report_default_prevented(false).len(), 3);
        assert_eq!(
            accepted
                .on_pointer(DragPointerPhase::Move, 60.0, 60.0, Some("div#dst".into()))
                .len(),
            3
        );
        accepted.report_default_prevented(false);
        let drop_batch = accepted.on_pointer(DragPointerPhase::Up, 60.0, 60.0, Some("div#dst".into()));
        assert_eq!(
            drop_batch
                .iter()
                .map(|p| (p.event_type, p.selector.as_str()))
                .collect::<Vec<_>>(),
            [("drop", "div#dst"), ("dragend", "div#src")]
        );
        assert!(!accepted.is_active());

        let mut refused = DragInputState::new();
        refused.arm("div#src".into(), 0.0, 0.0);
        assert_eq!(
            refused
                .on_pointer(DragPointerPhase::Move, 30.0, 30.0, Some("div#src".into()))
                .len(),
            1
        );
        assert_eq!(refused.report_default_prevented(false).len(), 3);
        assert_eq!(
            refused
                .on_pointer(DragPointerPhase::Move, 60.0, 60.0, Some("div#dst".into()))
                .len(),
            3
        );
        refused.report_default_prevented(true); // 目标拒绝
        let leave_batch = refused.on_pointer(DragPointerPhase::Up, 60.0, 60.0, Some("div#dst".into()));
        assert_eq!(
            leave_batch
                .iter()
                .map(|p| (p.event_type, p.selector.as_str()))
                .collect::<Vec<_>>(),
            [("dragleave", "div#dst"), ("dragend", "div#src")]
        );
    }

    /// Esc 取消进行中的会话 → [dragleave(当前目标), dragend(源)]；Armed 态取消静默。
    #[test]
    fn cancel_ends_session_with_leave_and_dragend() {
        let mut s = DragInputState::new();
        s.arm("div#src".into(), 0.0, 0.0);
        assert_eq!(
            s.on_pointer(DragPointerPhase::Cancel, 0.0, 0.0, Some("div#src".into())),
            Vec::new()
        );
        assert!(!s.is_active());
        s.arm("div#src".into(), 0.0, 0.0);
        assert_eq!(
            s.on_pointer(DragPointerPhase::Move, 30.0, 30.0, Some("div#src".into()))
                .len(),
            1
        );
        assert_eq!(s.report_default_prevented(false).len(), 3);
        let cancel = s.on_pointer(DragPointerPhase::Cancel, 35.0, 35.0, Some("div#src".into()));
        assert_eq!(
            cancel
                .iter()
                .map(|p| (p.event_type, p.selector.as_str()))
                .collect::<Vec<_>>(),
            [("dragleave", "div#src"), ("dragend", "div#src")]
        );
        assert!(!s.is_active());
    }

    /// Armed 态 Up（未过阈值）静默回 Idle——点击流不受拖拽状态机影响。
    #[test]
    fn armed_release_without_drag_is_quiet() {
        let mut s = DragInputState::new();
        s.arm("div#src".into(), 0.0, 0.0);
        assert_eq!(
            s.on_pointer(DragPointerPhase::Up, 1.0, 0.0, Some("div#src".into())),
            Vec::new()
        );
        assert!(!s.is_active());
    }

    /// 会话 id 递增——上一会话 dragend 后新 arm 分配新 id（DataTransfer 新实例语义）。
    #[test]
    fn session_ids_increment_across_sessions() {
        let mut s = DragInputState::new();
        s.arm("div#a".into(), 0.0, 0.0);
        let first = s.on_pointer(DragPointerPhase::Move, 30.0, 30.0, Some("div#a".into()));
        assert_eq!(s.report_default_prevented(false).len(), 3);
        assert_eq!(
            s.on_pointer(DragPointerPhase::Up, 30.0, 30.0, Some("div#a".into()))
                .len(),
            2
        );
        s.arm("div#a".into(), 0.0, 0.0);
        let second = s.on_pointer(DragPointerPhase::Move, 30.0, 30.0, Some("div#a".into()));
        assert_eq!(first[0].session + 1, second[0].session);
    }

    /// 无待判定时 report 静默（Idle / 批尾不可取消两态）。
    #[test]
    fn stray_report_is_ignored() {
        let mut s = DragInputState::new();
        assert_eq!(s.report_default_prevented(true), Vec::new());
        s.arm("div#src".into(), 0.0, 0.0);
        assert_eq!(
            s.on_pointer(DragPointerPhase::Move, 30.0, 30.0, Some("div#src".into()))
                .len(),
            1
        );
        s.report_default_prevented(false);
        // 补派批报告后再多余的 report 返空计划（不产出事件）；over_accepted 的批次序
        // 契约由宿主保证——每批派发后恰好报告一次（apply_drag_input 内建）。
        assert_eq!(s.report_default_prevented(true), Vec::new());
    }

    /// 边界事件 relatedTarget（spec fire a DND event：dragleave.related = 正在进入的
    /// 新目标、dragenter.related = 上次边界迁移目标；初始接受批不建立边界链——首个真
    /// 目标 related 为 null，corpus `events/dragenter-dragleave-related-target` 口径）。
    #[test]
    fn boundary_events_carry_related_target() {
        let mut s = DragInputState::new();
        s.arm("div#src".into(), 0.0, 0.0);
        assert_eq!(
            s.on_pointer(DragPointerPhase::Move, 30.0, 30.0, Some("div#src".into()))
                .len(),
            1
        );
        // 初始接受批（源自身 drag/enter/over）：related 全 null。
        let accept = s.report_default_prevented(false);
        assert!(accept.iter().all(|p| p.related_target.is_none()));

        // 首真目标 A：leave(源).related = A；enter(A).related = null（源不算既有目标）。
        let to_a = s.on_pointer(DragPointerPhase::Move, 60.0, 60.0, Some("div#a".into()));
        assert_eq!(
            to_a.iter()
                .map(|p| (p.event_type, p.selector.as_str(), p.related_target.as_deref()))
                .collect::<Vec<_>>(),
            [
                ("dragleave", "div#src", Some("div#a")),
                ("dragenter", "div#a", None),
                ("dragover", "div#a", None),
            ]
        );
        s.report_default_prevented(false);

        // A → B：enter(B).related = A；leave(A).related = B。
        let to_b = s.on_pointer(DragPointerPhase::Move, 90.0, 90.0, Some("div#b".into()));
        assert_eq!(
            to_b.iter()
                .map(|p| (p.event_type, p.selector.as_str(), p.related_target.as_deref()))
                .collect::<Vec<_>>(),
            [
                ("dragleave", "div#a", Some("div#b")),
                ("dragenter", "div#b", Some("div#a")),
                ("dragover", "div#b", None),
            ]
        );

        // 离开全部元素：leave.related = null，边界链清零。
        let off = s.on_pointer(DragPointerPhase::Move, 120.0, 120.0, None);
        assert_eq!(off.len(), 1);
        assert_eq!(off[0].event_type, "dragleave");
        assert!(off[0].related_target.is_none());

        // 再入 A：按首入语义 related = null。
        let back = s.on_pointer(DragPointerPhase::Move, 60.0, 60.0, Some("div#a".into()));
        let enter_back = back.iter().find(|p| p.event_type == "dragenter").expect("enter");
        assert!(enter_back.related_target.is_none(), "再入目标 related=null");
    }

    /// 起拖 move 落点已在源外（一次到位手势）→ 接受批内完成源→落点边界迁移
    ///（UA 逐 move 处理模型——dragstart 后 UA 立即按当前指针位置跑目标更新步；
    /// corpus `events/dragenter-dragleave-related-target`：enter(新).related=null，
    /// 后续 drop 落在该目标）。
    #[test]
    fn initiating_move_away_from_source_transitions_in_accept_batch() {
        let mut s = DragInputState::new();
        s.arm("div#src".into(), 0.0, 0.0);
        let start = s.on_pointer(DragPointerPhase::Move, 60.0, 60.0, Some("div#a".into()));
        assert_eq!(start.len(), 1);
        assert_eq!(start[0].event_type, "dragstart");
        let accept = s.report_default_prevented(false);
        assert_eq!(
            accept
                .iter()
                .map(|p| (p.event_type, p.selector.as_str(), p.related_target.as_deref()))
                .collect::<Vec<_>>(),
            [
                ("drag", "div#src", None),
                ("dragleave", "div#src", Some("div#a")),
                ("dragenter", "div#a", None),
                ("dragover", "div#a", None),
            ]
        );
        // 批宿主协议：接受批以 over 收尾 → 恰好一次回报（over 通过）。
        assert_eq!(s.report_default_prevented(false), Vec::new());
        // 释放：drop 落在初始更新后的目标 A（over 通过）。
        let up = s.on_pointer(DragPointerPhase::Up, 60.0, 60.0, Some("div#a".into()));
        assert_eq!(
            up.iter()
                .map(|p| (p.event_type, p.selector.as_str()))
                .collect::<Vec<_>>(),
            [("drop", "div#a"), ("dragend", "div#src")]
        );
    }

    /// 起拖 move 命中为空（悬空落点）→ 接受批仅 drag，无待判定。
    #[test]
    fn initiating_move_dangling_target_only_drags() {
        let mut s = DragInputState::new();
        s.arm("div#src".into(), 0.0, 0.0);
        assert_eq!(s.on_pointer(DragPointerPhase::Move, 60.0, 60.0, None).len(), 1);
        let accept = s.report_default_prevented(false);
        assert_eq!(
            accept
                .iter()
                .map(|p| (p.event_type, p.selector.as_str(), p.related_target.as_deref()))
                .collect::<Vec<_>>(),
            [("drag", "div#src", None), ("dragleave", "div#src", None)]
        );
        // 后续 move 进入目标 A：正常边界迁移（current_target 已悬空 → 无 leave）。
        let to_a = s.on_pointer(DragPointerPhase::Move, 90.0, 90.0, Some("div#a".into()));
        assert_eq!(
            to_a.iter().map(|p| p.event_type).collect::<Vec<_>>(),
            ["dragenter", "dragover"]
        );
    }
}
