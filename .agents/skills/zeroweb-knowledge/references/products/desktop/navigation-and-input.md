# 标签导航与页面输入

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

定位桌面地址栏、Ctrl+Tab、滚轮、触摸、IME、查找和复制。`BrowserShell` 存浏览器业务状态，`apps/browser/src/tab_manager.rs` 存每个 TabId 的页面后端、快照与待执行动作，二者不是同一个 TabManager。

关键入口：`navigate_to_request`、`handle_key`、`handle_ime`、`GlyphSelection`、`TextInput`。页面默认动作契约见[输入契约](../../runtime/page-runtime/interaction-contract.md)，文档事件见[默认动作](../../scripting/event-loop/events-and-default-actions.md)。产品路径验证见[验证入口](../../../../zeroweb-verify/SKILL.md)；该验证资产是本轮工作区新增，不归为 HEAD 源码，本次未执行页面输入验收。

## Gotchas
- navigate_to_request 只接受无 body 的 GET 与有 body 的 POST，view-source 必须先拆特殊 scheme；把源码查看地址交给普通地址正规化会破坏 URL。 (`apps/browser/src/app.rs`)
- 未提供启动 URL 时，欢迎页通过 record_initial_navigation 进入标签导航栈；on_page_loaded 排除 zero://newtab 的全局访问史，标签后退记录与全局 History 不是同一集合。 (`apps/browser/src/app.rs`, `crates/browser-shell/src/browser.rs`)
- Ctrl+Tab 不只改变 Shell 活跃 ID，还必须调用 on_active_tab_changed、更新地址栏并同步 viewport；历史修复证明缺少地址栏同步会显示上一标签 URL。 (`apps/browser/src/app_input.rs`, `git:fe8819c714519360444854cd7eb92552e4995125`)
- 滚轮增量在桌面入口取反，LineDelta 乘 40 后使用；上下文菜单打开或指针不在页面区时不滚页面，Ctrl/Cmd+wheel 改为缩放。 (`apps/browser/src/app_input.rs`, `apps/browser/src/page_scroll.rs`)
- 触摸长按候选等待 500ms，移动超过物理 8px 就取消；阈值按物理坐标判断，不能在 DPI 换算后继续套原阈值。 (`apps/browser/src/app_input.rs`)
- IME 先服务地址栏，再服务查找栏，最后服务页面；页面空 Commit 仍会发送，用于结束组合态，不能套用 Android 非空文本的拒绝规则。 (`apps/browser/src/app_input.rs`)
- 页面 IME 候选窗在 sync_ime_state 将 CSS caret 加内容原点/DPI、减物理滚动/DPI 得到窗口逻辑位置；该路径没有套 page_render_scale 的 zoom，不能声称已证明非100%页面缩放下候选位置准确。 (`apps/browser/src/app_platform.rs`, `apps/browser/src/app.rs`)
- 地址栏 TextInput 的 cursor/anchor 按 Rust char 编号，页面控件的 TextControlBoundary 使用 utf16_offset；中文、补充平面字符的编辑不能混用字节或两套索引。 (`apps/browser/src/text_input.rs`, `apps/browser/src/page_selection.rs`)
- 页面复制先按 GlyphSource 逻辑来源排序并去除同 cluster 重复，不能按视觉 glyph 顺序串接文本；双向文字与连字会暴露这种误用。 (`apps/browser/src/page_selection.rs`)
- page_find 从绘制 glyph_id 反查字符，不是 DOM 全文查找，复杂 shaping 和连字有明确限制；静态代码还存在 needle 长于 glyph 序列时的切片越界风险，本轮未执行复现。 (`apps/browser/src/page_find.rs`)
- 桌面发布的 TabManager 默认 Some(ProcessTabBackend::try_new())，try_new 返回后端本身；缺 renderer binary 在建标签时报告失败，只有 cfg(test)/test-support 的进程内 worker 路径可供测试使用，旧注释的“缺 binary 回退 worker”不符合当前实现。 (`apps/browser/src/tab_manager.rs`, `apps/browser/src/process_backend.rs`, `apps/browser/Cargo.toml`)
- normalize_url 对无 scheme 的localhost、*.localhost、IPv4回环/私网与IPv6回环采用http，公网或数字端口采用https；authority含空白或方括号残渣回退搜索，裸非回环IPv6也回退，不能只靠冒号判定URL。 (`apps/browser/src/app_platform.rs`, `apps/browser/src/tests.rs`, `git:b54f6748aba86f2995b1f1dc99d4f1fb3b1de8fe`)
- 地址栏只把单斜杠根相对路径接到当前非zero文档，点击链接另用 resolve_clicked_link_url 接文档相对值；guide/intro.html 的链接解析不能复用地址栏主机启发式。 (`apps/browser/src/app_platform.rs`)
- TextInput 的粘贴会删除CR/LF，cut 只有剪贴板写入成功才删选区；双击选词把下划线、连字符、点和斜杠也视为词字符，不等于DOM或Unicode分词。 (`apps/browser/src/text_input.rs`)
- case-insensitive page_find 对 needle 使用 Unicode to_lowercase，对 glyph 字符却只用 to_ascii_lowercase，非 ASCII 大小写不能当作已完整支持。 (`apps/browser/src/page_find.rs`)

## Architecture
- 新导航会清滚动、选区、favicon 与查找状态，页面后端快照按 TabId 保存；切标签后的视口和输入必须定位到当前标签，而不是复用前一个页面全局缓存。 (`apps/browser/src/app.rs`, `apps/browser/src/tab_manager.rs`)
- 页面滚动键默认动作挂在 keydown 回执之后，允许页面 preventDefault 阻止滚动，聚焦文本控件时也有守卫；提前在宿主直接滚动会绕开页面取消语义。 (`apps/browser/src/app_input_keys.rs`, `apps/browser/src/tab_manager.rs`)
- hover 命中直接读主线程 TabSnapshot 的 hit_test，不同步查询繁忙 renderer；mousedown 缓存带 navigation_epoch/document_generation 的 PageTarget，release遇到过期文档目标直接取消，缓存缺失则转发真实坐标给renderer命中。 (`apps/browser/src/tab_manager.rs`)
- 页面键事件有已知selector时指向该目标，多进程无selector时保留renderer焦点目标，进程内无目标才回退body；resolve_dispatch一次性移除派发记录，仅default_allowed时入队默认动作。 (`apps/browser/src/tab_manager.rs`)
- 页面窗口坐标换算由 page_doc_point 加物理滚动、减内容原点并除DPI×zoom；文本控件caret选取paint提供的同node_handle、同一行最近UTF-16边界，不能从字宽估算替代真实边界缓存。 (`apps/browser/src/app_input.rs`, `apps/browser/src/page_selection.rs`, `apps/browser/src/tab_manager.rs`)
- Overlay滚动条不缩减CSS视口，横纵需求独立计算；图片文档宽度用clip交集，根背景不计入可滚动高度，避免垂直滚动条引入水平溢出或背景撑出空白滚动区。 (`apps/browser/src/page_scroll.rs`)

## Decisions
- 启动URL等真实窗口尺寸已知后由ensure_startup_tab消费一次，先建默认标签runtime再走navigate_to；不能在BrowserApp构造时用默认视口直接装载CLI页面。 (`apps/browser/src/main.rs`, `apps/browser/src/app.rs`, `git:b5cabf3e54158b2541db5f00c861311a0208d055`)
- 页面视觉滚动先更新宿主状态，JS scroll 通知为fire-and-forget且JS禁用时no-op；这是尽力补页面可观察事件，不把通知失败转换成视觉滚动失败。 (`apps/browser/src/app_input.rs`, `apps/browser/src/tab_manager.rs`)

## Conventions
- Alt+Left/Right、F11、F5 和 Ctrl/Cmd 强制刷新是浏览器级早返回，页面输入转发在地址栏、查找栏、菜单及修饰键条件后决定；添加快捷键需按既有优先级判断，不统一下发所有键。 (`apps/browser/src/app_input_keys.rs`)
