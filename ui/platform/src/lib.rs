//! # zero-ui-platform
//!
//! 平台服务（spec §8.4.1 `zero-ui-platform` / FR-016 / IF-010 `PlatformServices` / §8.4.1B
//! 拖拽·剪贴板·file picker·通知 走 platform service、§8.8 platform service mock 测）。
//!
//! 全部为 trait（可 mock），不向 widgets 暴露具体后端；组件产生 intent（drag / open file /
//! copy），由 runtime 经 [`PlatformServices`] 调用对应服务执行（不污染 widgets，spec §8.4.1B）。

use std::cell::RefCell;

// ── 剪贴板 ─────────────────────────────────────────────────────────────────────

/// 剪贴板服务（IF-010 `ClipboardService`）。
pub trait ClipboardService {
    fn get_text(&self) -> Option<String>;
    fn set_text(&self, text: &str);
    /// 是否可用（某些平台/无障碍模式下可能禁用）。
    fn available(&self) -> bool {
        true
    }
}

/// 内存剪贴板（测试 + headless）。trait 方法 `&self`，内部 `RefCell`。
#[derive(Debug, Default)]
pub struct InMemoryClipboard {
    content: RefCell<Option<String>>,
}

impl InMemoryClipboard {
    pub fn new() -> InMemoryClipboard {
        InMemoryClipboard::default()
    }
}

impl ClipboardService for InMemoryClipboard {
    fn get_text(&self) -> Option<String> {
        self.content.borrow().clone()
    }
    fn set_text(&self, text: &str) {
        *self.content.borrow_mut() = Some(text.to_string());
    }
}

// ── 文件选择器 ─────────────────────────────────────────────────────────────────

/// 文件选择结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickedFile {
    pub path: String,
    pub display_name: String,
}

/// 文件选择器（IF-010 `FilePickerService`）。
pub trait FilePickerService {
    fn pick_open(&self) -> Option<PickedFile>;
    fn pick_save(&self, suggested: &str) -> Option<PickedFile>;
}

/// 内存文件选择器：`pick_open` 按队列返回预设结果（模拟用户依次选文件）；
/// `pick_save` 记录最后一次建议名并返回固定结果。
#[derive(Debug, Default)]
pub struct InMemoryFilePicker {
    open_queue: RefCell<Vec<PickedFile>>,
    last_save_suggested: RefCell<Option<String>>,
}

impl InMemoryFilePicker {
    pub fn new() -> InMemoryFilePicker {
        InMemoryFilePicker::default()
    }

    /// 预设 `pick_open` 将依次返回的文件（先入先出）。
    pub fn enqueue_open(&self, file: PickedFile) -> &Self {
        self.open_queue.borrow_mut().push(file);
        self
    }

    /// 最近一次 `pick_save` 收到的建议名（断言用）。
    pub fn last_save_suggested(&self) -> Option<String> {
        self.last_save_suggested.borrow().clone()
    }
}

impl FilePickerService for InMemoryFilePicker {
    fn pick_open(&self) -> Option<PickedFile> {
        let mut q = self.open_queue.borrow_mut();
        if q.is_empty() { None } else { Some(q.remove(0)) }
    }
    fn pick_save(&self, suggested: &str) -> Option<PickedFile> {
        *self.last_save_suggested.borrow_mut() = Some(suggested.to_string());
        Some(PickedFile {
            path: format!("/tmp/{suggested}"),
            display_name: suggested.to_string(),
        })
    }
}

// ── 拖放（drag / drop）──────────────────────────────────────────────────────

/// 拖拽项（§8.4.1B tab/link/download/file）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DragItem {
    /// MIME / 类型标识（如 `text/uri-list`、`application/x-tab`、`text/plain`）。
    pub mime: String,
    /// 拖拽负载数据（URL / 文本 / tab id 序列化等）。
    pub data: String,
    /// 可选展示标签（拖拽幽灵上显示）。
    pub label: Option<String>,
}

impl DragItem {
    pub fn new(mime: &str, data: &str) -> DragItem {
        DragItem {
            mime: mime.to_string(),
            data: data.to_string(),
            label: None,
        }
    }
    pub fn labeled(mut self, label: &str) -> DragItem {
        self.label = Some(label.to_string());
        self
    }
}

/// 拖拽结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragOutcome {
    /// 拖拽完成，目标接收（复制语义）。
    Copied,
    /// 拖拽完成，源移除（移动语义）。
    Moved,
    /// 取消 / 目标不支持。
    Cancelled,
}

/// 拖放服务（IF-010 `DragDropService`）。
///
/// 组件产生 drag intent → runtime 调 [`DragDropService::begin_drag`] 执行 OS 拖拽；
/// drop 目标组件经 [`DragDropService::staged`] / [`DragDropService::receive`] 取回 payload
/// 决定接受与否（spec §8.4.1B：不污染 widgets）。
pub trait DragDropService {
    /// 开始一次拖拽；返回结果（mock 可立即返回，真实后端异步）。
    fn begin_drag(&self, item: &DragItem) -> DragOutcome;

    /// 当前暂存的拖拽项（drop 目标读取以决定是否接受该 MIME）。
    fn staged(&self) -> Option<DragItem>;

    /// 目标是否接受该 MIME（用于 drag-over 反馈）。
    fn accepts_mime(&self, mime: &str) -> bool;

    /// 接收当前暂存项（drop 落入）；返回取回的数据，无暂存则 None。
    fn receive(&self) -> Option<DragItem>;
}

/// 内存拖放服务（测试 + headless）：记录 begin_drag；暂存当前项供 receive。
#[derive(Debug, Default)]
pub struct InMemoryDragDrop {
    staged: RefCell<Option<DragItem>>,
    begins: RefCell<Vec<DragItem>>,
    accepted_mimes: Vec<String>,
}

impl InMemoryDragDrop {
    pub fn new() -> InMemoryDragDrop {
        InMemoryDragDrop::default()
    }

    /// 声明接受的 MIME 列表（决定 `accepts_mime` / `begin_drag` 结果）。
    pub fn accepting(mut self, mimes: &[&str]) -> InMemoryDragDrop {
        self.accepted_mimes = mimes.iter().map(|s| s.to_string()).collect();
        self
    }

    /// 历史 begin_drag 调用（断言用）。
    pub fn begun(&self) -> Vec<DragItem> {
        self.begins.borrow().clone()
    }
}

impl DragDropService for InMemoryDragDrop {
    fn begin_drag(&self, item: &DragItem) -> DragOutcome {
        self.begins.borrow_mut().push(item.clone());
        if self.accepts_mime(&item.mime) {
            *self.staged.borrow_mut() = Some(item.clone());
            DragOutcome::Copied
        } else {
            DragOutcome::Cancelled
        }
    }
    fn staged(&self) -> Option<DragItem> {
        self.staged.borrow().clone()
    }
    fn accepts_mime(&self, mime: &str) -> bool {
        self.accepted_mimes.iter().any(|m| m == mime)
    }
    fn receive(&self) -> Option<DragItem> {
        self.staged.borrow_mut().take()
    }
}

// ── 通知 ───────────────────────────────────────────────────────────────────────

/// 通知内容（IF-010）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    pub title: String,
    pub body: Option<String>,
}

impl Notification {
    pub fn new(title: &str) -> Notification {
        Notification {
            title: title.to_string(),
            body: None,
        }
    }
    pub fn body(mut self, body: &str) -> Notification {
        self.body = Some(body.to_string());
        self
    }
}

/// 通知服务（IF-010 `NotificationService`）。
pub trait NotificationService {
    fn notify(&self, n: &Notification);
}

/// 内存通知服务（测试 + headless）：记录所有 notify 调用。
#[derive(Debug, Default)]
pub struct InMemoryNotifications {
    sent: RefCell<Vec<Notification>>,
}

impl InMemoryNotifications {
    pub fn new() -> InMemoryNotifications {
        InMemoryNotifications::default()
    }
    pub fn sent(&self) -> Vec<Notification> {
        self.sent.borrow().clone()
    }
}

impl NotificationService for InMemoryNotifications {
    fn notify(&self, n: &Notification) {
        self.sent.borrow_mut().push(n.clone());
    }
}

// ── 平台 back 手势 / 硬件 back（DC-15 移动端，spec §8.4.1B / §8.8）──────────────

/// app 级 back handler 标识（如「关闭地址栏菜单」「关闭下载面板」）。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BackHandlerId(String);

impl BackHandlerId {
    pub fn new(id: &str) -> BackHandlerId {
        BackHandlerId(id.to_string())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 平台 back 意图仲裁结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackResult {
    /// 已注册 handler 消耗了 back（携带其 id；宿主据此执行该 handler 的动作，如关闭对应浮层）。
    Handled(BackHandlerId),
    /// 无已注册 handler：宿主执行默认 back（典型：navigator.pop；栈底则退出应用）。
    DefaultBack,
}

/// 平台 back 手势 / 硬件 back 仲裁（DC-15 移动端，spec §8.4.1B / §8.8）。
///
/// Android 硬件 back / iOS edge-swipe-back / HarmonyOS 返回：平台投递 back 意图时，先咨询
/// 已注册的 app 级 back handler（弹层 / 菜单打开时注册、关闭时注销）；无 handler 时返回
/// [`BackResult::DefaultBack`]，由宿主决定（navigator.pop / 退出）。
///
/// handler 以 **LIFO 栈**管理（最近注册的最先响应），匹配 Android `OnBackPressedDispatcher`
/// 与 Flutter `WillPopScope` 语义。trait 方法 `&self`（内部可变性），可作 trait object。
pub trait BackNavigationService {
    /// 注册一个 app 级 back handler（弹层 / 菜单打开时）。重复 id 入栈不合并（每注册一次响应一次）。
    fn push_handler(&self, id: BackHandlerId);
    /// 注销最近注册的 handler（弹层 / 菜单关闭时）；栈空返回 None。
    fn pop_handler(&self) -> Option<BackHandlerId>;
    /// 当前是否有已注册 handler。
    fn has_handler(&self) -> bool;
    /// 平台 back 意图到达：有 handler → 消耗栈顶并返回 [`BackResult::Handled`]；无 → [`BackResult::DefaultBack`]。
    fn on_platform_back(&self) -> BackResult;
}

/// 内存 back 导航服务（测试 + headless）：LIFO 栈记录 handler。
#[derive(Debug, Default)]
pub struct InMemoryBackNavigation {
    handlers: RefCell<Vec<BackHandlerId>>,
}

impl InMemoryBackNavigation {
    pub fn new() -> InMemoryBackNavigation {
        InMemoryBackNavigation::default()
    }
    /// 当前栈快照（断言用，最在栈顶 = 末尾）。
    pub fn handlers(&self) -> Vec<BackHandlerId> {
        self.handlers.borrow().clone()
    }
}

impl BackNavigationService for InMemoryBackNavigation {
    fn push_handler(&self, id: BackHandlerId) {
        self.handlers.borrow_mut().push(id);
    }
    fn pop_handler(&self) -> Option<BackHandlerId> {
        self.handlers.borrow_mut().pop()
    }
    fn has_handler(&self) -> bool {
        !self.handlers.borrow().is_empty()
    }
    fn on_platform_back(&self) -> BackResult {
        match self.handlers.borrow_mut().pop() {
            Some(id) => BackResult::Handled(id),
            None => BackResult::DefaultBack,
        }
    }
}

// ── 触觉反馈 / haptics（DC-15 移动端触摸，spec §8.8）──────────────────────────

/// 触觉反馈强度 / 类型（匹配 Android `VibrationEffect` / iOS `UIImpactFeedbackGenerator`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HapticKind {
    /// 轻触反馈（按钮点击）。
    Light,
    /// 中等反馈（开关切换）。
    Medium,
    /// 强反馈（长按 / 拖拽起手）。
    Heavy,
    /// 选择变化反馈（滚动拾取）。
    Selection,
}

/// 触觉反馈服务（DC-15 移动端）。组件产生 haptic intent → runtime 经此服务触发设备震动，
/// 不污染 widgets。桌面端通常为 no-op。
pub trait HapticFeedbackService {
    fn tap(&self, kind: HapticKind);
}

/// 内存触觉反馈服务（测试 + headless）：记录所有 tap 调用。
#[derive(Debug, Default)]
pub struct InMemoryHaptics {
    taps: RefCell<Vec<HapticKind>>,
}

impl InMemoryHaptics {
    pub fn new() -> InMemoryHaptics {
        InMemoryHaptics::default()
    }
    /// 历史 tap 调用（断言用）。
    pub fn taps(&self) -> Vec<HapticKind> {
        self.taps.borrow().clone()
    }
}

impl HapticFeedbackService for InMemoryHaptics {
    fn tap(&self, kind: HapticKind) {
        self.taps.borrow_mut().push(kind);
    }
}

// ── 平台服务聚合（IF-010 PlatformServices）────────────────────────────────

/// 平台服务聚合（IF-010 `PlatformServices`）。
///
/// 宿主持**一个** `PlatformServices`；widgets / runtime 经它访问剪贴板 / 文件选择 / 拖放 / 通知 /
/// back 手势 / 触觉反馈，不直接依赖具体后端（spec §6.2：widgets 不直接访问 platform API）。
pub struct PlatformServices {
    clipboard: Box<dyn ClipboardService>,
    file_picker: Box<dyn FilePickerService>,
    drag_drop: Box<dyn DragDropService>,
    notifications: Box<dyn NotificationService>,
    back_navigation: Box<dyn BackNavigationService>,
    haptics: Box<dyn HapticFeedbackService>,
}

impl PlatformServices {
    /// 用四类核心服务构造；back 导航 + 触觉反馈默认为内存实现（移动端宿主经
    /// [`with_back_navigation`](PlatformServices::with_back_navigation) /
    /// [`with_haptics`](PlatformServices::with_haptics) 注入真实后端）。
    pub fn new(
        clipboard: Box<dyn ClipboardService>,
        file_picker: Box<dyn FilePickerService>,
        drag_drop: Box<dyn DragDropService>,
        notifications: Box<dyn NotificationService>,
    ) -> PlatformServices {
        PlatformServices {
            clipboard,
            file_picker,
            drag_drop,
            notifications,
            back_navigation: Box::new(InMemoryBackNavigation::new()),
            haptics: Box::new(InMemoryHaptics::new()),
        }
    }

    /// 覆盖默认的内存 back 导航服务（移动端宿主注入平台 back 后端）。
    pub fn with_back_navigation(mut self, back: Box<dyn BackNavigationService>) -> PlatformServices {
        self.back_navigation = back;
        self
    }

    /// 覆盖默认的内存触觉反馈服务（移动端宿主注入平台震动后端）。
    pub fn with_haptics(mut self, haptics: Box<dyn HapticFeedbackService>) -> PlatformServices {
        self.haptics = haptics;
        self
    }

    pub fn clipboard(&self) -> &dyn ClipboardService {
        &*self.clipboard
    }
    pub fn file_picker(&self) -> &dyn FilePickerService {
        &*self.file_picker
    }
    pub fn drag_drop(&self) -> &dyn DragDropService {
        &*self.drag_drop
    }
    pub fn notifications(&self) -> &dyn NotificationService {
        &*self.notifications
    }
    pub fn back_navigation(&self) -> &dyn BackNavigationService {
        &*self.back_navigation
    }
    pub fn haptics(&self) -> &dyn HapticFeedbackService {
        &*self.haptics
    }
}

/// 全部服务用内存 mock 的 [`PlatformServices`]（测试 + headless）。
pub fn in_memory_platform_services() -> PlatformServices {
    PlatformServices::new(
        Box::new(InMemoryClipboard::new()),
        Box::new(InMemoryFilePicker::new()),
        Box::new(InMemoryDragDrop::new().accepting(&["text/uri-list", "text/plain"])),
        Box::new(InMemoryNotifications::new()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipboard_round_trip_via_trait_object() {
        let cb: Box<dyn ClipboardService> = Box::new(InMemoryClipboard::new());
        assert!(cb.get_text().is_none());
        cb.set_text("https://zero.example");
        assert_eq!(cb.get_text().as_deref(), Some("https://zero.example"));
        assert!(cb.available(), "default available = true");
    }

    #[test]
    fn file_picker_queue_and_save() {
        let fp = InMemoryFilePicker::new();
        fp.enqueue_open(PickedFile {
            path: "/a/b.txt".into(),
            display_name: "b.txt".into(),
        });
        let picked = fp.pick_open().unwrap();
        assert_eq!(picked.display_name, "b.txt");
        // 队列空 → None。
        assert!(fp.pick_open().is_none());
        // pick_save 记录建议名并返回固定结果。
        let saved = fp.pick_save("report.pdf").unwrap();
        assert_eq!(saved.display_name, "report.pdf");
        assert_eq!(fp.last_save_suggested().as_deref(), Some("report.pdf"));
    }

    #[test]
    fn drag_drop_intent_routes_through_service_not_widget() {
        // §8.4.1B：组件产生 drag intent → runtime 调 begin_drag；widget 不碰 OS。
        let dd = InMemoryDragDrop::new().accepting(&["text/uri-list"]);
        // 接受的 MIME → Copied + 暂存。
        let link = DragItem::new("text/uri-list", "https://zero.example").labeled("Zero");
        assert_eq!(dd.begin_drag(&link), DragOutcome::Copied);
        assert_eq!(dd.begun().len(), 1);
        assert_eq!(dd.staged().unwrap().data, "https://zero.example");
        // drop 目标 receive 取回。
        assert_eq!(dd.receive().unwrap().mime, "text/uri-list");
        assert!(dd.staged().is_none(), "staged consumed after receive");
        // 不接受的 MIME → Cancelled，不暂存。
        let tab = DragItem::new("application/x-tab", "tab.2");
        assert_eq!(dd.begin_drag(&tab), DragOutcome::Cancelled);
        assert!(dd.staged().is_none());
        assert!(!dd.accepts_mime("application/x-tab"));
    }

    #[test]
    fn notifications_recorded() {
        let n = InMemoryNotifications::new();
        n.notify(&Notification::new("Download complete").body("zero.zip"));
        n.notify(&Notification::new("Pasted"));
        assert_eq!(n.sent().len(), 2);
        assert_eq!(n.sent()[0].title, "Download complete");
        assert_eq!(n.sent()[0].body.as_deref(), Some("zero.zip"));
    }

    #[test]
    fn platform_services_aggregates_all_four() {
        // IF-010：一个 PlatformServices 暴露四类服务。
        let ps = in_memory_platform_services();
        ps.clipboard().set_text("copied");
        assert_eq!(ps.clipboard().get_text().as_deref(), Some("copied"));
        ps.notifications().notify(&Notification::new("hi"));
        // drag_drop 默认接受 text/plain。
        assert!(ps.drag_drop().accepts_mime("text/plain"));
        let item = DragItem::new("text/plain", "hello");
        assert_eq!(ps.drag_drop().begin_drag(&item), DragOutcome::Copied);
        assert_eq!(ps.drag_drop().receive().unwrap().data, "hello");
        // file_picker 默认空队列。
        assert!(ps.file_picker().pick_open().is_none());
    }

    #[test]
    fn custom_backends_can_be_injected() {
        // 宿主可注入自定义后端（非内存 mock），验证 PlatformServices 不绑死实现。
        struct StubClip;
        impl ClipboardService for StubClip {
            fn get_text(&self) -> Option<String> {
                Some("stub".into())
            }
            fn set_text(&self, _: &str) {}
        }
        let ps = PlatformServices::new(
            Box::new(StubClip),
            Box::new(InMemoryFilePicker::new()),
            Box::new(InMemoryDragDrop::new()),
            Box::new(InMemoryNotifications::new()),
        );
        assert_eq!(ps.clipboard().get_text().as_deref(), Some("stub"));
    }

    #[test]
    fn back_navigation_lifo_arbitrates_platform_back() {
        // DC-15 平台 back：弹层/菜单打开时 push handler；平台 back 到达时 LIFO 消耗。
        let bn = InMemoryBackNavigation::new();
        // 无 handler → DefaultBack（宿主 navigator.pop / 退出）。
        assert!(!bn.has_handler());
        assert_eq!(bn.on_platform_back(), BackResult::DefaultBack);

        // 打开菜单 → push；再打开下载面板 → push（LIFO：下载面板在上）。
        bn.push_handler(BackHandlerId::new("menu"));
        bn.push_handler(BackHandlerId::new("downloads"));
        assert!(bn.has_handler());
        assert_eq!(
            bn.handlers(),
            vec![BackHandlerId::new("menu"), BackHandlerId::new("downloads")]
        );

        // 平台 back → 消耗栈顶「downloads」。
        assert_eq!(
            bn.on_platform_back(),
            BackResult::Handled(BackHandlerId::new("downloads"))
        );
        // 再 back → 消耗「menu」。
        assert_eq!(bn.on_platform_back(), BackResult::Handled(BackHandlerId::new("menu")));
        // 栈空 → DefaultBack。
        assert_eq!(bn.on_platform_back(), BackResult::DefaultBack);
    }

    #[test]
    fn back_navigation_explicit_pop_matches_handler_lifecycle() {
        // 弹层显式关闭（非 back 触发）→ pop_handler 注销。
        let bn = InMemoryBackNavigation::new();
        bn.push_handler(BackHandlerId::new("menu"));
        assert_eq!(bn.pop_handler(), Some(BackHandlerId::new("menu")));
        assert!(bn.pop_handler().is_none());
    }

    #[test]
    fn haptics_record_taps() {
        let h = InMemoryHaptics::new();
        h.tap(HapticKind::Light);
        h.tap(HapticKind::Selection);
        h.tap(HapticKind::Heavy);
        assert_eq!(
            h.taps(),
            vec![HapticKind::Light, HapticKind::Selection, HapticKind::Heavy]
        );
    }

    #[test]
    fn platform_services_exposes_back_and_haptics_by_default() {
        // PlatformServices::new 默认注入内存 back + haptics；移动端宿主可经 with_* 覆盖。
        let ps = in_memory_platform_services();
        // 默认 back：无 handler → DefaultBack。
        assert_eq!(ps.back_navigation().on_platform_back(), BackResult::DefaultBack);
        ps.back_navigation().push_handler(BackHandlerId::new("overlay"));
        assert_eq!(
            ps.back_navigation().on_platform_back(),
            BackResult::Handled(BackHandlerId::new("overlay"))
        );
        // 默认 haptics 可记录。
        ps.haptics().tap(HapticKind::Medium);
    }

    #[test]
    fn platform_services_with_back_navigation_overrides_default() {
        // 注入一个忽略 push、恒返回 DefaultBack 的后端；与默认 in-memory（push 后返回 Handled）
        // 行为不同，借以证明 with_back_navigation 确实覆盖了默认实现。
        struct NoopBack;
        impl BackNavigationService for NoopBack {
            fn push_handler(&self, _: BackHandlerId) {}
            fn pop_handler(&self) -> Option<BackHandlerId> {
                None
            }
            fn has_handler(&self) -> bool {
                false
            }
            fn on_platform_back(&self) -> BackResult {
                BackResult::DefaultBack
            }
        }
        let ps = in_memory_platform_services().with_back_navigation(Box::new(NoopBack));
        ps.back_navigation().push_handler(BackHandlerId::new("x"));
        // 默认 in-memory 会返回 Handled("x")；注入的 NoopBack 忽略 push → DefaultBack。
        assert_eq!(ps.back_navigation().on_platform_back(), BackResult::DefaultBack);
    }
}
