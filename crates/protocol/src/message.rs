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
    /// 更新页面 JavaScript 执行策略；不影响用户代理默认动作。
    SetJavascriptEnabled(bool),
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
    /// Browser → compositor：更新 surface 滚动偏移（RFC 4.2 异步滚动切片）。
    CompositorSetScroll {
        /// 页面 surface 的稳定标识。
        surface_id: u64,
        /// 水平滚动偏移（CSS 像素）。
        scroll_x: f32,
        /// 垂直滚动偏移（CSS 像素）。
        scroll_y: f32,
    },
    /// Browser → compositor：注册 Chrome UI 层 surface 元数据（RFC 4.4 切片）。
    CompositorRegisterUiSurface(crate::compositor_types::CompositorUiSurfaceInfo),
    /// Browser → compositor：登记最终窗口 surface（RFC 4.4-S4 Viz 所有权）。
    CompositorRegisterWindowSurface(crate::compositor_types::CompositorWindowSurfaceInfo),
    /// Browser → compositor：提交 Chrome UI 层位图（RFC 4.4-S2）。
    CompositorUiFrame {
        /// UI surface 的稳定标识。
        surface_id: u64,
        /// 宽度（像素）。
        width: u32,
        /// 高度（像素）。
        height: u32,
        /// RGBA 像素（shm 传输时为空）。
        rgba: Vec<u8>,
        /// POSIX shm buffer 名（不含 `zeroweb-cmp-` 前缀）。
        #[serde(default)]
        shm_name: Option<String>,
    },
    /// 拉取 UI surface 最新位图（显示消费方 → 合成器）。
    GetCompositorUiFrame {
        /// UI surface 的稳定标识。
        surface_id: u64,
    },
    /// 拉取 page+UI 合成 present 帧（RFC 4.4-S3 Viz present 切片）。
    GetCompositorPresentFrame {
        /// 输出宽度（像素）。
        width: u32,
        /// 输出高度（像素）。
        height: u32,
        /// 页面 surface 标识。
        page_surface_id: u64,
        /// UI surface 标识。
        ui_surface_id: u64,
    },
    /// 已合成帧数据（合成器 → 显示消费方）：front 缓冲像素。
    ///
    /// 默认内联 `rgba`；Linux shm 传输（默认开）时 `shm_name` 非空且 `rgba` 为空，
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
        /// compositor 侧记录的水平滚动偏移（RFC 4.2；Browser 可选消费）。
        #[serde(default)]
        scroll_x: f32,
        /// compositor 侧记录的垂直滚动偏移（RFC 4.2；Browser 可选消费）。
        #[serde(default)]
        scroll_y: f32,
        /// GPU shared image 描述符（RFC 4.3-S2+）。
        #[serde(default)]
        gpu_image: Option<crate::compositor_types::GpuSharedImageDescriptor>,
        /// present 帧是否为 compositor 权威输出（RFC 4.4-S4）。
        #[serde(default)]
        present_authoritative: bool,
    },

    // ── 存储请求（渲染→浏览器→存储）──
    /// localStorage/sessionStorage 操作。
    StorageOp(StorageOpParams),

    // ── 输入事件（浏览器→渲染）──
    /// 鼠标事件。
    MouseEvent(MouseEventParams),
    /// 键盘事件。
    KeyboardEvent(KeyboardEventParams),
    /// 输入法合成事件。
    ImeEvent(ImeEventParams),
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

    // ── 焦点生命周期（渲染→浏览器）──
    // 纪律：bincode 1 按变体声明位置编码判别值，新变体只能 append 在本枚举**末尾**，
    // 严禁插入中间（否则跨版本 peer 静默错位，见审查 R3254-L6）。
    /// 页面焦点所有者变更（R3254-H1：同步 browser 的 event_targets / 文本控件守卫）。
    FocusOwnerChanged(FocusChangeInfo),
    /// 自动化请求（WebDriver/testdriver → live renderer）。
    AutomationRequest(AutomationRequest),
    /// 自动化响应（live renderer → WebDriver/testdriver）。
    AutomationResponse(AutomationResponse),
    /// IndexedDB 请求（renderer → browser storage owner）。
    IndexedDbRequest(IndexedDbRequestParams),
    /// IndexedDB 响应（browser storage owner → renderer）。
    IndexedDbResponse(IndexedDbResponseParams),
    /// 页面导航开始（renderer → browser storage-key authority）。
    NavigationStarted(NavigationStartedParams),
    /// 页面导航提交（renderer → browser storage-key authority）。
    NavigationCommitted(NavigationCommittedParams),
    /// IndexedDB connection 的 versionchange 通知（browser → renderer）。
    IndexedDbConnectionEvent(IndexedDbConnectionEventParams),
    /// IndexedDB connection event 已在 JS owner 执行（renderer → browser）。
    IndexedDbConnectionEventAck(IndexedDbConnectionEventAckParams),
    /// Service Worker 请求（renderer → browser owner）。
    ServiceWorkerRequest(ServiceWorkerRequestParams),
    /// Service Worker 响应（browser owner → renderer）。
    ServiceWorkerResponse(ServiceWorkerResponseParams),
}

/// 焦点变更信息（渲染→浏览器）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusChangeInfo {
    /// 新焦点所有者稳定选择器；None 表示失焦（blur）。
    pub selector: Option<String>,
    /// 焦点是否在可编辑文本控件（input 文本类 / textarea）——browser 侧滚动守卫用。
    pub text_input: bool,
}

/// 自动化请求。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutomationRequest {
    /// 要在 live renderer owner 上执行的操作。
    pub operation: AutomationOperation,
}

/// 自动化操作。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AutomationOperation {
    /// 在当前 live document 中定位第一个元素。
    FindElement {
        /// 定位策略。
        using: AutomationLocatorStrategy,
        /// 定位值。
        value: String,
    },
    /// 激活一个已定位元素。
    ElementClick {
        /// 带文档作用域的元素引用。
        element: AutomationElementRef,
    },
    /// 向一个已定位元素发送文本或特殊键。
    SendKeys {
        /// 带文档作用域的元素引用。
        element: AutomationElementRef,
        /// 按顺序执行的键序列。
        keys: Vec<AutomationKey>,
    },
    /// 查询当前焦点元素。
    GetActiveElement,
    /// 在当前页面脚本上下文执行同步脚本。
    ExecuteScript {
        /// 脚本源码。
        script: String,
        /// 传给脚本的 JSON 兼容参数。
        arguments: Vec<AutomationValue>,
    },
    /// 显式表示适配层尚未支持的命令；renderer 只返回错误，不执行名称内容。
    Unsupported {
        /// 未支持命令名。
        name: String,
    },
}

/// 自动化元素定位策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutomationLocatorStrategy {
    /// CSS selector。
    CssSelector,
}

/// 带 live document 作用域的自动化元素引用。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AutomationElementRef {
    /// 所属导航 epoch。
    pub navigation_epoch: u64,
    /// 所属 document generation。
    pub document_generation: u64,
    /// 当前 document 内的 opaque node handle。
    pub node_handle: u64,
}

/// 自动化发送键。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutomationKey {
    /// 一段 Unicode 文本。
    Text(String),
    /// Tab。
    Tab,
    /// Shift+Tab。
    ShiftTab,
    /// Backspace。
    Backspace,
    /// Enter。
    Enter,
}

/// 自动化响应。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutomationResponse {
    /// renderer 处理请求时的导航 epoch。
    pub navigation_epoch: u64,
    /// renderer 处理请求时的 document generation。
    pub document_generation: u64,
    /// 操作结果或确定性错误。
    pub result: Result<AutomationResult, AutomationError>,
}

/// 自动化成功结果。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AutomationResult {
    /// 无返回值。
    Empty,
    /// 元素引用；`None` 表示当前无焦点元素。
    Element(Option<AutomationElementRef>),
    /// JSON 兼容脚本返回值。
    Value(AutomationValue),
}

/// 自动化脚本值。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AutomationValue {
    /// JavaScript null/undefined。
    Null,
    /// 布尔值。
    Bool(bool),
    /// 数值。
    Number(f64),
    /// 字符串。
    String(String),
    /// 数组。
    Array(Vec<AutomationValue>),
    /// 对象键值列表。
    Object(Vec<(String, AutomationValue)>),
}

/// 自动化错误。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutomationError {
    /// 稳定错误分类。
    pub code: AutomationErrorCode,
    /// 面向调用方的错误消息。
    pub message: String,
}

/// 自动化错误分类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutomationErrorCode {
    /// 当前文档中没有匹配元素。
    NoSuchElement,
    /// 元素引用不属于当前 live document，或其节点已被替换。
    StaleElementReference,
    /// 请求参数不合法。
    InvalidArgument,
    /// 操作不在当前支持面内。
    UnsupportedOperation,
    /// 页面脚本执行失败。
    JavascriptError,
    /// 自动化请求超时。
    Timeout,
    /// renderer 内部错误。
    Internal,
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
    /// 设备缩放因子；仅影响 compositor 位图的光栅分辨率，不改变 CSS 视口。
    pub device_scale_factor: f32,
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

/// IndexedDB 同步 host 请求。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedDbRequestParams {
    /// 页面产生的 IndexedDB JSON wire；origin 由 browser 根据 tab URL 推导。
    pub request: String,
}

/// IndexedDB 同步 host 响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedDbResponseParams {
    /// 成功时的 JSON wire。
    pub response: Option<String>,
    /// 失败时的具名错误。
    pub error: Option<String>,
}

/// Browser owner 向 renderer connection 投递的版本变更事件。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexedDbConnectionEventParams {
    /// Renderer realm 内 connection 标识。
    pub connection_id: u64,
    /// Browser owner 分配的 connection request 标识。
    pub request_id: u64,
    /// 变更前版本。
    pub old_version: u64,
    /// 目标版本；删除数据库时为 `None`。
    pub new_version: Option<u64>,
}

/// Renderer 完成 versionchange event dispatch 后的确认。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexedDbConnectionEventAckParams {
    /// Renderer realm 内 connection 标识。
    pub connection_id: u64,
    /// Browser owner 分配的 connection request 标识。
    pub request_id: u64,
}

/// Renderer 发往 browser Service Worker owner 的请求。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceWorkerRequestParams {
    /// 请求操作。
    pub operation: ServiceWorkerOperation,
}

impl ServiceWorkerRequestParams {
    /// Validate untrusted renderer strings before browser-side processing.
    pub fn validate(&self) -> Result<(), &'static str> {
        const MAX_URL_BYTES: usize = 64 * 1024;
        match &self.operation {
            ServiceWorkerOperation::Register {
                script_url,
                scope,
                document_url,
            } => {
                if script_url.is_empty() || document_url.is_empty() {
                    return Err("Service Worker script and document URL are required");
                }
                if script_url.len() > MAX_URL_BYTES
                    || document_url.len() > MAX_URL_BYTES
                    || scope.as_ref().is_some_and(|value| value.len() > MAX_URL_BYTES)
                {
                    return Err("Service Worker URL exceeds the length limit");
                }
                Ok(())
            }
            ServiceWorkerOperation::Snapshot { .. }
            | ServiceWorkerOperation::Unregister { .. }
            | ServiceWorkerOperation::ActivateWaiting { .. }
            | ServiceWorkerOperation::GetRegistrations
            | ServiceWorkerOperation::Controller
            | ServiceWorkerOperation::StateChanges { .. } => Ok(()),
            ServiceWorkerOperation::PostMessage { data_json, .. } => {
                const MAX_MESSAGE_BYTES: usize = 1024 * 1024;
                if data_json.len() > MAX_MESSAGE_BYTES {
                    return Err("Service Worker message exceeds the size limit");
                }
                Ok(())
            }
            ServiceWorkerOperation::ClientMessages { .. } => Ok(()),
            ServiceWorkerOperation::GetRegistration { client_url } => {
                if client_url.is_empty() {
                    return Err("Service Worker client URL is required");
                }
                if client_url.len() > MAX_URL_BYTES {
                    return Err("Service Worker client URL exceeds the length limit");
                }
                Ok(())
            }
        }
    }
}

/// Service Worker owner operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceWorkerOperation {
    /// Register a fetched/evaluated worker for the current document.
    Register {
        /// Script URL, relative or absolute.
        script_url: String,
        /// Optional scope URL.
        scope: Option<String>,
        /// Renderer document URL; browser validates it against tab authority.
        document_url: String,
    },
    /// Read one registration version snapshot.
    Snapshot {
        /// Browser-assigned registration version ID.
        registration_id: u64,
    },
    /// Remove one registration version.
    Unregister {
        /// Browser-assigned registration version ID.
        registration_id: u64,
    },
    /// Activate a waiting replacement.
    ActivateWaiting {
        /// Browser-assigned registration version ID.
        registration_id: u64,
    },
    /// Find the registration whose scope contains one client URL.
    GetRegistration {
        /// Absolute or document-relative client URL.
        client_url: String,
    },
    /// List the current document origin's registrations.
    GetRegistrations,
    /// Read lifecycle states after a renderer-owned sequence cursor.
    StateChanges {
        /// Browser-assigned registration version ID.
        registration_id: u64,
        /// Number of state changes already observed by this renderer.
        after_sequence: u64,
    },
    /// Read the active controller for the committed document.
    Controller,
    /// Dispatch a JSON-compatible message to one worker version.
    PostMessage {
        /// Browser-assigned registration version ID.
        registration_id: u64,
        /// Serialized structured payload.
        data_json: String,
    },
    /// Read worker messages addressed to the committed document.
    ClientMessages {
        /// Browser-assigned registration version ID.
        registration_id: u64,
        /// Number of completed message-event batches already observed by this renderer.
        after_sequence: u64,
    },
}

/// Browser Service Worker owner response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceWorkerResponseParams {
    /// Typed operation result or stable error.
    pub result: Result<ServiceWorkerResult, ServiceWorkerError>,
}

/// Successful Service Worker owner result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceWorkerResult {
    /// Registration started and script evaluation succeeded.
    Registered {
        /// Browser-assigned registration version ID.
        registration_id: u64,
    },
    /// Current registration snapshot.
    Snapshot(ServiceWorkerSnapshot),
    /// Boolean operation result.
    Boolean(bool),
    /// Operation completed without a value.
    Empty,
    /// Optional registration snapshot.
    OptionalSnapshot(Option<ServiceWorkerSnapshot>),
    /// Registration snapshots for one origin.
    Snapshots(Vec<ServiceWorkerSnapshot>),
    /// Ordered lifecycle states after a renderer-owned cursor.
    StateChanges(ServiceWorkerStateChanges),
    /// Worker messages addressed to one committed document.
    ClientMessages(ServiceWorkerClientMessages),
}

/// Immutable worker-to-client message log suffix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceWorkerClientMessages {
    /// Total number of completed message-event batches at response time.
    pub latest_sequence: u64,
    /// JSON-compatible structured payloads after the request cursor.
    pub data_json: Vec<String>,
}

/// Pure-value registration snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceWorkerSnapshot {
    /// Registration version ID.
    pub registration_id: u64,
    /// Normalized script URL.
    pub script_url: String,
    /// Normalized scope URL.
    pub scope: String,
    /// Current worker lifecycle state.
    pub state: ServiceWorkerStateWire,
}

/// IPC-safe Service Worker lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceWorkerStateWire {
    /// Script is evaluating or install event is running.
    Installing,
    /// Install fulfilled; version is waiting.
    Installed,
    /// Activate event is running.
    Activating,
    /// Version is active.
    Activated,
    /// Version failed or was replaced/unregistered.
    Redundant,
}

/// Immutable lifecycle log suffix for one Service Worker version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceWorkerStateChanges {
    /// Total number of recorded changes at response time.
    pub latest_sequence: u64,
    /// States strictly after the request cursor, in transition order.
    pub states: Vec<ServiceWorkerStateWire>,
    /// Whether this version requested control of matching clients.
    pub claim_clients: bool,
}

/// Service Worker owner error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceWorkerError {
    /// Stable error category.
    pub code: ServiceWorkerErrorCode,
    /// Diagnostic safe for renderer exposure.
    pub message: String,
}

/// Stable Service Worker IPC error category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceWorkerErrorCode {
    /// URL, scope, security context, or request shape is invalid.
    InvalidArgument,
    /// Requested registration does not exist.
    NotFound,
    /// Operation conflicts with current lifecycle state.
    InvalidState,
    /// Script fetch failed.
    Network,
    /// Script compile/evaluation or lifecycle event failed.
    Script,
    /// Browser owner reached a resource limit.
    Capacity,
    /// Browser owner failed internally.
    Internal,
    /// Registration violates secure-context, origin, or scope-path policy.
    Security,
}

/// Renderer 页面导航开始信号。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavigationStartedParams {
    /// 待加载文档 URL。
    pub url: String,
    /// Browser 分配或 renderer 递增的导航世代。
    pub navigation_epoch: u64,
}

/// Renderer 页面导航提交信号。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavigationCommittedParams {
    /// 已提交文档 URL，Browser 必须与 pending navigation 逐项匹配。
    pub url: String,
    /// 已提交文档的导航世代。
    pub navigation_epoch: u64,
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

/// 输入法事件参数。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImeEventParams {
    /// 输入法生命周期阶段。
    pub event_type: ImeEventType,
    /// Preedit 或 Commit 文本；Enabled/Disabled 为空。
    pub text: String,
    /// Preedit 光标/选区起点（UTF-8 字节偏移，沿用 winit 契约）。
    pub cursor_start: Option<usize>,
    /// Preedit 光标/选区终点（UTF-8 字节偏移，沿用 winit 契约）。
    pub cursor_end: Option<usize>,
}

/// 输入法生命周期阶段。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImeEventType {
    /// 平台输入法已启用。
    Enabled,
    /// 合成文本更新，尚未写入控件值。
    Preedit,
    /// 合成文本提交为一次编辑批次。
    Commit,
    /// 输入法已禁用；未提交合成文本必须取消。
    Disabled,
}

/// 滚动事件参数。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScrollEventParams {
    /// 水平滚动量。
    pub delta_x: f32,
    /// 垂直滚动量。
    pub delta_y: f32,
    /// 滚轮发生处的光标视口坐标 x（物理像素，相对 WebView 内容区）。
    ///
    /// R3298（元素滚动 RFC S1，非破坏性扩展）：默认 `0.0` 以向后兼容旧发送端。
    /// renderer 侧 `handle_scroll_event` 用此 + `cursor_y` 命中可滚动祖先容器；
    /// 缺省 `0.0` 时退化为既有文档级滚动路径（S0/R3294 行为不变）。
    /// 元素级滚动视觉（命中可滚动容器 + 容器内偏移）依赖 S3 layout 几何暴露（渲染流域）。
    pub cursor_x: f32,
    /// 滚轮发生处的光标视口坐标 y（物理像素，相对 WebView 内容区）。见 `cursor_x`。
    pub cursor_y: f32,
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
    /// 所属导航 epoch。
    #[serde(default)]
    pub navigation_epoch: u64,
    /// 所属 Document 世代。
    #[serde(default)]
    pub document_generation: u64,
    /// 当前 Document 内的 opaque DOM 节点句柄。
    #[serde(default)]
    pub node_handle: Option<u64>,
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
    /// Shift 键是否按下（Tab 反向焦点导航等默认行为）。
    #[serde(default)]
    pub shift: bool,
    /// 指针定位得到的文本控件 UTF-16 selection 起点。
    #[serde(default)]
    pub selection_start: Option<u32>,
    /// 指针定位得到的文本控件 UTF-16 selection 终点。
    #[serde(default)]
    pub selection_end: Option<u32>,
}

/// DOM 事件派发结果（渲染→浏览器）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchDomEventResultParams {
    /// `preventDefault()` 未被调用。
    pub default_allowed: bool,
}
