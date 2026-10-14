# 窗口、浏览器 UI 与平台行为

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

定位桌面启动、DPI、壳层绘制、主题、窗口装饰和呈现问题。`main.rs` 接宿主事件，`BrowserApp` 经 `include!` 组合平台与绘制实现；页面 renderer 生命周期不是窗口宿主职责。

关键入口：`BrowserApp::build_scene`、`init_gpu`、`WindowConfig`、Windows `WM_NCHITTEST`。页面发布与回退见[合成器恢复](../../runtime/compositor/fallback-and-recovery.md)，窗口 API 见[事件宿主](../host-runtime.md)。

验证导航：[桌面功能地图](../../../../zeroweb-verify/features/navigation.md)与[页面绘制地图](../../../../zeroweb-verify/features/page-rendering.md)；本轮仅源码和历史核对，没有窗口/GPU 实跑，验证资产为本轮工作区新增，不属于上面的 HEAD。

## Gotchas
- WindowConfig 以逻辑尺寸创建窗口，首次 RedrawRequested 再将实际 inner_size 除 DPI 设置 app 的逻辑尺寸，同时保留 surface 物理尺寸；build_scene 的图元已转换为物理像素，browser_scene_gpu_scale_factor 固定1.0以避免 GPU 再次应用 DPI。 (`apps/browser/src/main.rs`, `apps/browser/src/app_render.rs`, `apps/browser/src/app_platform.rs`)
- Wayland 路径主动关闭窗口装饰并避开 wgpu 窗口 surface，使用 softbuffer 呈现；不能以 X11/macOS 的 surface 初始化行为判断 Wayland 页面是否可显示。 (`apps/browser/src/main.rs`, `apps/browser/src/app_platform.rs`)
- 失焦后的 GPU 处理分平台：Wayland 销毁 GPU，其他平台 suspend；恢复焦点需把 surface 标为 stale，而不是继续使用失焦前的呈现状态。 (`apps/browser/src/app_platform.rs`)
- macOS 一体化标题栏使用 transparent titlebar 与 fullsize content，不能替换成 titlebar_hidden，否则系统红黄绿按钮消失。 (`crates/host-runtime/src/window.rs`, `apps/browser/src/main.rs`)
- Windows 自绘最大化按钮的命中测试返回 HTMAXBUTTON，系统 Snap Layout 依赖这个返回值；只保留应用内按钮点击并不能保持窗口行为。 (`apps/browser/src/windows_titlebar.rs`)
- 加载条是从 is_loading 起点按时间逼近 85% 的动画，不是网络已下载比例；显示到某个百分比不证明正文、脚本或子资源已完成。 (`apps/browser/src/app_render.rs`)
- persist_user_data 只接 save_settings 与 save_bookmarks；BrowserShell 有 session/history/download 保存 API 不等于桌面退出已经调用它们。 (`apps/browser/src/app.rs`, `crates/browser-shell/src/browser.rs`)
- Windows 最大化区域使用 GetDpiForWindow，按客户端右侧第二个46逻辑像素按钮及40逻辑像素标签栏换算；WM_NCLBUTTONDOWN 的最大化/还原由 ShowWindow 处理，不是普通 MouseInput 的重复动作。 (`apps/browser/src/windows_titlebar.rs`)
- 失焦时普通 RedrawRequested 会清 needs_redraw 跳过绘制，但 smoke capture、各演示流和 ui_driver 显式绕开该条件；自动化成功截图不证明日常失焦绘制行为相同。 (`apps/browser/src/main.rs`)
- should_skip_local_composite_for_owned_present 必须同时满足 owned-present、present-enabled、compositor健康、无本地GPU及present像素就绪；存在本地GPU仍须本地提交，滚动非零也禁用旧整窗present复用。 (`apps/browser/src/app_platform.rs`)
- UI 文本在无 font_id 时绘制为空，测量返回零；有字体时按主字体行度量居中并用回退字体绘字、measure_advance 测宽，不能把菜单文字缺失直接诊断为菜单未创建。 (`apps/browser/src/app_render_ui.rs`)
- 浮动状态栏优先显示悬停链接，其次在 is_loading 且活动标签有 URL 时显示加载主机；host_str 去掉端口和路径，无法解析主机才回退完整 URL，清除 loading 后不再显示加载提示。 (`apps/browser/src/app_render.rs`, `apps/browser/src/tests.rs`)
- 菜单渐入起点在 build_scene 观察 visible 时建立，150ms 内按时间缩放颜色 alpha 并持续请求重绘；关闭在后续 build_scene 清掉起点，不是输入事件本身立即更新动画时钟，不能把透明度过渡当菜单命中区变化。 (`apps/browser/src/app_render.rs`, `apps/browser/src/app.rs`, `apps/browser/src/tests.rs`)

## Architecture
- build_scene 分开普通 fills/glyphs 和 overlay fills/glyphs，上下文菜单等浮层必须在主体图元之后呈现，壳层阴影也有独立集合。 (`apps/browser/src/app_render.rs`)
- compositor 状态 Starting 或 Healthy 时禁止浏览器消费 legacy 页面图元；这个控制条件不等同于“已经拿到第一帧”，启动期间不能擅自拼回旧页面。 (`apps/browser/src/app_render.rs`)
- 书签栏只有设置开启且存在根级书签时占高度，页面内容原点随其可见性变化；命中与 IME 定位不能固定减一个工具栏常量。 (`apps/browser/src/app.rs`, `apps/browser/src/app_render.rs`)
- 页面缩放只改变页面 device-scale×zoom 换算和 CSS 视口，不缩放浏览器 chrome；content_logical_size 对内容物理尺寸除该倍率后 floor，避免背景超出内容区底边。 (`apps/browser/src/app.rs`)

## Decisions
- 菜单在存储原点时按整份 items 高度贴右缘向左、贴下缘向上翻转，渲染与命中消费同一坐标；separator 紧凑高度不能用统一行高替代，翻转不是绘制时单独裁剪。 (`apps/browser/src/app_input_context_menus.rs`, `apps/browser/src/app.rs`, `apps/browser/src/tests.rs`, `git:2c6616aa8d504e737e2a921340bedbbab99da1f7`)
- 主菜单 Zoom 行按横向等分命中子动作，键盘进入行时无 hovered_segment 不执行动作；段 hover 变化必须置 needs_redraw，否则同一行移动时旧高亮不刷新。 (`apps/browser/src/app_input_context_menus.rs`, `apps/browser/src/app_input.rs`, `apps/browser/src/app_render.rs`, `git:c4a86a14d5f688b6e45af5e9f6b7ffffbb68c048`)
- 右键上下文按地址栏编辑、已有页面选区、图片、链接、页面空白顺序选择；图片包在链接中仍优先图片菜单，未实现的 save_link/save_image/copy_image/print 被禁用而不是展示可用。 (`apps/browser/src/app_input_context_menus.rs`)

## Patterns
- 平台与渲染分文件通过 include! 留在 app 模块作用域，增加方法时应核对 app.rs 的包含关系，不能把文件名当成独立 Rust 模块边界。 (`apps/browser/src/app.rs`, `apps/browser/src/app_platform.rs`, `apps/browser/src/app_render.rs`)
- CLI 选择、event-loop 驱动和 app_smoke_state 只读状态面在多次窗口 smoke 变更中共同接线；仅新增一个配置或状态 getter 不会让 release 浏览器自动执行该流程。 (`apps/browser/src/main.rs`, `apps/browser/src/app_smoke_state.rs`, `git:fe8819c714519360444854cd7eb92552e4995125`, `git:a62669a2df1bf643da096e7cc70cf1bab2e3f8be`, `git:ee54787fbcddd44bd71f09d793c2d31d4fd0cb7c`)

## Testing Strategy
- 菜单边缘回归经 handle_mouse_click 验证存储原点，scale=1 的页面菜单按7×32+2×10=244高和240宽翻转；测试没有证明更高 DPI 或 GPU 像素正确，不能省略实际场景坐标核对。 (`apps/browser/src/tests.rs`, `apps/browser/src/app_input_context_menus.rs`)
- Zoom 段重绘回归先打开菜单、进入首段，再清 needs_redraw 并移到另一段，确保跨段是唯一重绘来源；仅首次进入行的断言会被菜单打开或行 hover 的重绘掩盖。 (`apps/browser/src/tests.rs`, `apps/browser/src/app_input.rs`)
