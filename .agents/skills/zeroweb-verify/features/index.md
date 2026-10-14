# 功能地图

范围为 ZeroBrowser 桌面/Android 与 ZeroWebView 嵌入 API 的主要用户任务。
地图基于 2026-10-14 HEAD `e523b1c73b96360a6451948f0b748b7b4657750f`、
[README](../../../../README.md)、实际入口和既有测试建立。证据状态不代表产品完成度。
本次只实跑 F07 的存储生产 API，其他功能及 F07 的 JS/IPC/OPFS/SW 保持 `NOT_RUN`。
随后按共享工作区 `1217776cc9cf179b93aca8b1557f0dab44b8e498` 补充 T8 UI 入口和
未测判据；原存储证据保留其运行版本，未因这次文档补充重跑或刷新。
初始化资产已在 `d0f281d509de3bf4f9dcd7e8dd01dac3464a663b` 提交。
本次按该 HEAD 补充 F06 的 R5117 calc 几何回归、Touch 构造器/属性通道和
R5118 待复核入口，只有文档维护；相关路径仍为 `NOT_RUN`，历史失败未当作本轮结果。

| ID / 用户任务 | 子行为与入口 | 判据 | 来源与操作文档 | 本次状态 |
|---|---|---|---|---|
| F01 打开网站并浏览 | 地址栏键入/建议/加载指示；HTTP 导航、后退、前进、刷新、主页、失败重试 | URL/标题/活动页对应，刷新 epoch 前进，错误可见且可重试 | [导航与标签](navigation.md)：M1/M2、app、tab_smoke | NOT_RUN |
| F02 管理标签与恢复会话 | 新建、切换、关闭、拖拽、后台隔离、退出重开 | 顺序/活动标签/各页 URL 正确，恢复不重放表单或下载 | [导航与标签](navigation.md)：tab_manager、session | NOT_RUN |
| F03 收藏、历史与个性设置 | 书签/文件夹、栏开关、历史搜索/清空、搜索引擎、隐私、主页 | 重启恢复，删除目标生效且其他条目不受影响 | [个人数据](personal-data.md)：M4、pages、shell | NOT_RUN |
| F04 下载与导出文件 | attachment/download 链接、进度、多任务面板、暂停/取消状态、完成、失败、目录打开；Android SAF 导出/取消 | 文件字节一致，失败不记完成，取消不产生导出；状态与真实传输分别核对 | [下载与工具](downloads-tools.md)：M3、T8、fetch_proxy、Android | NOT_RUN |
| F05 查找、缩放与上下文操作 | Ctrl+F 计数/高亮/逐项/Esc、缩放/滚动、右键菜单、选择复制、查看源代码 | 命中/当前项正确，坐标与帧改变，复制内容正确 | [下载与工具](downloads-tools.md)：M1/M3、page_find、input | NOT_RUN |
| F06 网页交互与显示 | 文本编辑/焦点/IME、表单提交/取消、DOM/脚本、Touch 构造器/属性、calc/浮动几何、CSS/字体/图片、Canvas、a11y、clipboard/fullscreen | 可见值/事件/导航副作用正确；状态/几何/像素分别判定，mismatch 须异于 ref | [页面与渲染](page-rendering.md)：WebView、html_compat、calc 定向测试、Touch testharness、R5118 待复核、parity | NOT_RUN |
| F07 保存站点数据与离线使用 | WebStorage/cookies、IndexedDB、CacheStorage、OPFS、SW 生命周期/fetch，重启/同源隔离/写失败 | 重新加载恢复数据和索引/字节，失败不污染旧数据，源隔离 | [站点存储](storage.md)：StorageManager、owner、页面桥 | PASS：仅 7 个持久化 API 测试；其余 NOT_RUN |
| F08 安全浏览与权限 | CSP/CORS、混合内容、HSTS、权限拒绝/变化、来源隔离 | 不允许操作明确被拒，违规事件/错误正确，合法路径可用 | [页面与渲染](page-rendering.md)：SecurityContext、gate 测试 | NOT_RUN |
| F09 播放媒体 | audio/video 播放/暂停/seek/结束、音量/静音、Web Audio、编码支持 | 连续帧/事件正确、seek 位置正确、静音无输出、结束释放资源 | [媒体](media.md)：VideoPlayer、registry、媒体 goals | NOT_RUN |
| F10 嵌入网页 | Rust 构建器、load_html/load_url、CSS 注入、resize、事件、宿主动作 | 实际页面状态和渲染输出变化，加载失败回调，资源可释放 | [嵌入与自动化](embedding-automation.md)：WebView 契约/测试 | NOT_RUN |
| F11 自动化和调试 | WebDriver session/元素/脚本/截图；CDP/DevTools Elements/Console/Network | live 文档动作结果/协议错误正确，session 删除释放 renderer | [嵌入与自动化](embedding-automation.md)：HTTP、CDP、矩阵 | NOT_RUN |
| F12 Android 移动浏览 | 导航/多标签/缩略图、键盘/触摸、返回/旋转/后台、下载通知/SAF、Intent、断连恢复 | 设备真实状态/文件/UID/恢复正确，不重复重放请求 | [Android](android.md)：MainActivity、设备清单/chaos | NOT_RUN |

## 选择范围

按 ID 和子行为选择路径；F01/F02 的 M2 及 F04/F05 的 M3 有共享演示流，复用一次运行，
不重复算作多个独立证据。库 API 通过不自动更新 GUI 状态。全图只有用户明确要求才运行，
逐功能实跑一条路径仍需列明未覆盖分支。网站兼容性不是一张静态页的通过率。

知识导航：[工程方法论](../../zeroweb-guidelines/SKILL.md)、
[项目知识](../../zeroweb-knowledge/SKILL.md)、
[learnings](../../../../docs/learnings/INDEX.md)。知识核验与本地图的功能实跑结果分别记录。
