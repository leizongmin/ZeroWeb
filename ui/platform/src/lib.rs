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

// ── 平台服务聚合（IF-010 PlatformServices）────────────────────────────────

/// 平台服务聚合（IF-010 `PlatformServices`）。
///
/// 宿主持**一个** `PlatformServices`；widgets / runtime 经它访问剪贴板 / 文件选择 / 拖放 / 通知，
/// 不直接依赖具体后端（spec §6.2：widgets 不直接访问 platform API）。
pub struct PlatformServices {
    clipboard: Box<dyn ClipboardService>,
    file_picker: Box<dyn FilePickerService>,
    drag_drop: Box<dyn DragDropService>,
    notifications: Box<dyn NotificationService>,
}

impl PlatformServices {
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
        }
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
}
