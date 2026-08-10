//! IPC 消息类型定义。

use serde::{Deserialize, Serialize};

/// IPC 消息 — 浏览器进程与渲染进程之间的通信协议。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage {
    /// 消息 ID（用于匹配请求/响应）。
    pub id: u64,
    /// 消息类型。
    pub kind: IpcMessageKind,
}

/// IPC 消息类型。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcMessageKind {
    // ── 导航命令（浏览器→渲染）──
    /// 加载 URL。
    Navigate(NavigateParams),
    /// 后退。
    GoBack,
    /// 前进。
    GoForward,
    /// 停止加载。
    StopLoading,
    /// 重新加载。
    Reload,
    /// 直接加载 HTML（浏览器→渲染，zero:// 等内联页面）。
    LoadHtml(LoadHtmlParams),
    /// 调整视口（浏览器→渲染）。
    SetViewport(SetViewportParams),
    /// 更新颜色方案（浏览器→渲染）。
    SetColorScheme(SetColorSchemeParams),
    /// 更新渲染媒体类型（浏览器→渲染，DC-12 @media print；R1993）。
    SetMediaType(SetMediaTypeParams),
    /// 更新页面绘制帧的 Browser IPC 发布模式。
    SetFramePublishMode(FramePublishMode),
    /// 请求 renderer 立即从当前页面状态重新发布一帧。
    RequestFrame,

    // ── 页面事件（渲染→浏览器）──
    /// 页面标题变更。
    TitleChanged(String),
    /// URL 变更。
    UrlChanged(String),
    /// 页面加载完成。
    LoadComplete,
    /// 页面加载失败。
    LoadFailed(String),
    /// 页面绘制快照（渲染→浏览器）。
    ///
    /// 装箱以缩小枚举体积（`PaintSnapshotParams` 含 10+ Vec 字段，
    /// 否则 `IpcMessageKind` 每个值都背上快照栈体积）。serde 透明，不影响 IPC 线格式。
    ViewPainted(Box<crate::paint_snapshot::PaintSnapshotParams>),
    /// 链接命中测试（浏览器→渲染）。
    HitTestLink(HitTestLinkParams),
    /// 链接命中测试结果（渲染→浏览器）。
    HitTestLinkResult(HitTestLinkResultParams),
    /// 元素命中测试（浏览器→渲染）。
    HitTestElement(HitTestLinkParams),
    /// 元素命中测试结果（渲染→浏览器）。
    HitTestElementResult(HitTestElementResultParams),
    /// 图片命中测试（浏览器→渲染）。
    HitTestImage(HitTestLinkParams),
    /// 图片命中测试结果（渲染→浏览器）。
    HitTestImageResult(HitTestLinkResultParams),
    /// DOM 事件派发（浏览器→渲染，同步响应）。
    DispatchDomEvent(DispatchDomEventParams),
    /// DOM 事件派发结果（渲染→浏览器）。
    DispatchDomEventResult(DispatchDomEventResultParams),

    // ── 网络请求（渲染→浏览器→网络）──
    /// 发起网络请求。
    FetchRequest(FetchParams),
    /// 网络响应。
    FetchResponse(FetchResponseParams),

    // ── 图像解码（渲染→image-decoder 进程）──
    /// 解码请求（D1：PNG/JPEG/WebP 在独立进程解码，隔离编解码器漏洞）。
    ImageDecodeRequest(ImageDecodeParams),
    /// 解码结果。
    ImageDecodeResult(ImageDecodeResultParams),

    // ── 合成（渲染→compositor 进程，C2）──
    /// 帧提交（图元快照 → 合成器进程，BackingStore 双缓冲管理）。
    CompositorFrame {
        /// 页面 surface 的稳定标识。
        surface_id: u64,
        /// 提交帧所属的导航世代。
        navigation_epoch: u64,
        /// renderer 为该 surface 生成的单调帧序号。
        frame_id: u64,
        /// 页面绘制快照。
        paint: Box<crate::paint_snapshot::PaintSnapshotParams>,
    },
    /// 帧已合成确认（合成器 → 渲染）。
    CompositorFrameResult {
        /// 页面 surface 的稳定标识。
        surface_id: u64,
        /// 已合成帧所属的导航世代。
        navigation_epoch: u64,
        /// 已合成的 renderer 帧序号。
        frame_id: u64,
    },
    /// 拉取最新已合成帧（显示消费方 → 合成器）。
    GetCompositorFrame {
        /// 待读取页面 surface 的稳定标识。
        surface_id: u64,
        /// 显示消费方当前接受的导航世代。
        navigation_epoch: u64,
        /// 显示消费方当前已知的帧序号。
        frame_id: u64,
    },
    /// 释放指定页面 surface 的 backing store。
    ReleaseCompositorSurface {
        /// 待释放页面 surface 的稳定标识。
        surface_id: u64,
    },
    /// 已合成帧数据（合成器 → 显示消费方）：front 缓冲像素。
    ///
    /// 默认内联 `rgba`；Linux `ZW_COMPOSITOR_SHM=1` 时 `shm_name` 非空且 `rgba` 为空，
    /// 像素在 `/dev/shm/zeroweb-cmp-{shm_name}`（RFC 4.3 S1）。
    CompositorFrameData {
        /// 页面 surface 的稳定标识。
        surface_id: u64,
        /// front 缓冲所属的导航世代。
        navigation_epoch: u64,
        /// front 缓冲的 renderer 帧序号（无帧时为 0）。
        frame_id: u64,
        /// 宽度（像素）。
        width: u32,
        /// 高度（像素）。
        height: u32,
        /// RGBA 像素（width × height × 4；shm 传输时为空）。
        rgba: Vec<u8>,
        /// POSIX shm buffer 名（不含 `zeroweb-cmp-` 前缀）；None = 内联 `rgba`。
        #[serde(default)]
        shm_name: Option<String>,
    },

    // ── 存储请求（渲染→浏览器→存储）──
    /// localStorage/sessionStorage 操作。
    StorageOp(StorageOpParams),

    // ── 输入事件（浏览器→渲染）──
    /// 鼠标事件。
    MouseEvent(MouseEventParams),
    /// 键盘事件。
    KeyboardEvent(KeyboardEventParams),
    /// 滚动事件。
    ScrollEvent(ScrollEventParams),

    // ── 进程管理 ──
    /// 进程心跳。
    Heartbeat,
    /// 进程崩溃通知。
    CrashNotification(String),

    // ── 通用响应 ──
    /// 成功响应。
    Ok,
    /// 错误响应。
    Error(String),
}

/// 导航参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigateParams {
    /// 目标 URL。
    pub url: String,
    /// 来源页面 URL。
    pub referrer: Option<String>,
    /// 浏览器侧导航世代（`begin_navigation` 递增）；ViewPainted 须携带同值。
    #[serde(default)]
    pub navigation_epoch: u64,
}

/// 内联 HTML 加载参数（浏览器→渲染）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadHtmlParams {
    /// HTML 文档。
    pub html: String,
    /// 可选附加 CSS。
    pub css: Option<String>,
    /// 逻辑页面 URL（用于相对链接解析与 ImageCache 键）。
    pub url: Option<String>,
    /// 浏览器侧导航世代（与 Navigate 一致）。
    #[serde(default)]
    pub navigation_epoch: u64,
}

/// 视口调整参数（浏览器→渲染）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SetViewportParams {
    /// 视口宽度（CSS 逻辑像素）。
    pub width: u32,
    /// 视口高度（CSS 逻辑像素）。
    pub height: u32,
}

/// IPC 颜色方案（对应 `prefers-color-scheme`）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IpcColorScheme {
    /// 亮色。
    Light,
    /// 暗色。
    Dark,
}

/// 颜色方案更新参数（浏览器→渲染）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SetColorSchemeParams {
    /// 用户偏好颜色方案。
    pub scheme: IpcColorScheme,
}

/// IPC 媒体类型（对应 `@media print/screen`；R1993）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum IpcMediaType {
    /// screen。
    Screen,
    /// print。
    Print,
}

/// 媒体类型更新参数（浏览器→渲染，DC-12 @media print）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SetMediaTypeParams {
    /// 渲染媒体类型。
    pub media_type: IpcMediaType,
}

/// 页面绘制帧的 Browser IPC 发布模式。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FramePublishMode {
    /// 兼容路径：renderer 直接发布 `ViewPainted`。
    Legacy,
    /// 合成器主链路：renderer 发布 `CompositorFrame`。
    Compositor,
}

/// 网络请求参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchParams {
    /// 请求 ID。
    pub request_id: u64,
    /// 请求 URL。
    pub url: String,
    /// HTTP 方法。
    pub method: String,
    /// 请求头。
    pub headers: Vec<(String, String)>,
    /// 请求体。
    pub body: Option<Vec<u8>>,
}

/// 网络响应参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchResponseParams {
    /// 对应的请求 ID。
    pub request_id: u64,
    /// HTTP 状态码。
    pub status_code: u16,
    /// 响应头。
    pub headers: Vec<(String, String)>,
    /// 响应体。
    pub body: Vec<u8>,
}

/// 图像解码请求（D1：image-decoder 独立进程）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageDecodeParams {
    /// 请求 ID（响应中回带，用于匹配）。
    pub request_id: u64,
    /// 图像 MIME（如 image/png、image/jpeg、image/webp；SVG 不进本通道）。
    pub mime: String,
    /// 图像原始字节。
    pub bytes: Vec<u8>,
}

/// 图像解码结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageDecodeResultParams {
    /// 与请求一致的 ID。
    pub request_id: u64,
    /// 解码成功时的宽。
    pub width: u32,
    /// 解码成功时的高。
    pub height: u32,
    /// 解码成功时的 RGBA 像素（width × height × 4）。
    pub rgba: Vec<u8>,
    /// 解码失败信息（成功时为空）。
    pub error: Option<String>,
}

/// 存储操作参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageOpParams {
    /// 存储类型。
    pub storage_type: StorageType,
    /// 操作类型。
    pub operation: StorageOperation,
    /// 键。
    pub key: String,
    /// 值。
    pub value: Option<String>,
    /// 来源。
    pub origin: String,
}

/// 存储类型。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StorageType {
    /// localStorage。
    Local,
    /// sessionStorage。
    Session,
}

/// 存储操作。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StorageOperation {
    /// 读取。
    Get,
    /// 写入。
    Set,
    /// 删除。
    Remove,
    /// 清空。
    Clear,
    /// 获取长度。
    Length,
    /// 按索引获取键名。
    Key,
}

/// 鼠标事件参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseEventParams {
    /// X 坐标。
    pub x: f32,
    /// Y 坐标。
    pub y: f32,
    /// 鼠标按键。
    pub button: u8,
    /// 事件类型。
    pub event_type: MouseEventType,
}

/// 鼠标事件类型。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MouseEventType {
    /// 按下。
    Down,
    /// 释放。
    Up,
    /// 移动。
    Move,
    /// 单击。
    Click,
    /// 双击。
    DblClick,
}

/// 键盘事件参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardEventParams {
    /// 按键值。
    pub key: String,
    /// 物理键码。
    pub code: String,
    /// Ctrl 键是否按下。
    pub ctrl: bool,
    /// Shift 键是否按下。
    pub shift: bool,
    /// Alt 键是否按下。
    pub alt: bool,
    /// Meta 键是否按下。
    pub meta: bool,
    /// 事件类型。
    pub event_type: KeyboardEventType,
}

/// 键盘事件类型。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KeyboardEventType {
    /// 按下。
    Down,
    /// 释放。
    Up,
    /// 输入。
    Press,
}

/// 滚动事件参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollEventParams {
    /// 水平滚动量。
    pub delta_x: f32,
    /// 垂直滚动量。
    pub delta_y: f32,
}

/// 链接命中测试参数（浏览器→渲染）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitTestLinkParams {
    /// 文档坐标 x。
    pub x: f32,
    /// 文档坐标 y。
    pub y: f32,
}

/// 链接命中测试结果（渲染→浏览器）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitTestLinkResultParams {
    /// 命中链接 href；未命中为 `None`。
    pub href: Option<String>,
}

/// 元素命中测试结果（渲染→浏览器）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitTestElementResultParams {
    /// 命中元素标签名（小写）。
    pub tag_name: Option<String>,
    /// `id` 属性。
    pub id: Option<String>,
    /// `class` 属性。
    pub class_name: Option<String>,
    /// 布局盒左上角 X。
    pub x: f32,
    /// 布局盒左上角 Y。
    pub y: f32,
    /// 布局盒宽度。
    pub width: f32,
    /// 布局盒高度。
    pub height: f32,
    /// 用于 JS 事件派发的稳定选择器。
    pub selector: Option<String>,
}

/// DOM 事件派发参数（浏览器→渲染）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchDomEventParams {
    /// 目标选择器；为 `None` 时在 `(x, y)` 处命中测试。
    pub selector: Option<String>,
    /// 文档坐标 x（选择器为空时使用）。
    pub x: f32,
    /// 文档坐标 y。
    pub y: f32,
    /// 事件类型（`click`、`keydown` 等）。
    pub event_type: String,
    /// `KeyboardEvent.key`
    pub key: Option<String>,
    /// `KeyboardEvent.code`
    pub code: Option<String>,
}

/// DOM 事件派发结果（渲染→浏览器）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchDomEventResultParams {
    /// `preventDefault()` 未被调用。
    pub default_allowed: bool,
}
