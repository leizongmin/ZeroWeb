# Ladybird 近期进展：增量计算、进程安全与 Alpha 交付

调研日期：2026-10-20。重点覆盖 2026 年 8、9 月官方月报，并补查公开发布状态。本文补充 [2026-08-06 的历史调研](research-ladybird-browser-2026-08-06.md)，不重述项目历史。

ZeroWeb 源码观察基线：`28aa3ac722bbe6f200bd1e74374b0545356fd4b4`。本次只读核查源码与公开材料，没有构建、运行浏览器或复测性能。

## 来源与阅读边界

| 分类 | 本文使用方式 |
|---|---|
| 一手事实 | Ladybird 官方月报、官网、PR 中作者的实现说明；ZeroWeb 当前源码 |
| 单来源报告 | 性能分数、长期内存数据是 Ladybird 自报，未独立复测 |
| 推理与建议 | 如何借鉴到 ZeroWeb，由两项目材料综合推导，不代表已确认缺陷或收益 |
| 历史材料 | 旧调研仅提供检索背景，旧数字不沿用为当前状态 |

查到的最新完整官方月报是 9 月 30 日。官网仍写 Linux/macOS 首个 Alpha 目标为 2026 年；本次未找到可确认正式 Alpha 已发布的公告。[1][2] GitHub API 查询遇到限流，因此没有对截至 10 月 20 日的全部提交、版本或发布资产做穷尽核查。

## 主要发现

Ladybird 最近两个月的工作集中在真实网页运行成本、安全隔离和交付准备。最值得 ZeroWeb 借鉴的是：为增量计算建立完整计算参考路径；用跨浏览器小操作基准定位异常；按 renderer 已被攻破检查权限；用后台页面和重复导航检验长期资源释放。[1][3][4][5]

新功能也有明显进展：Twitch 等站点的视频格式支持、JavaScript 断点调试、会话恢复、CSS scroll snap、默认启用的 OffscreenCanvas、图片粘贴、macOS 硬件视频解码及独立媒体进程。[1][3] 这些是官方报告的功能进展，不能据此推断所有相关站点或规范用例已通过。

> 来源说明：一手材料 [1][3][4][5]；“最值得借鉴”的排序是作者综合。

## 关键证据核对

| 结论 | 直接证据 | 补充证据或边界 | 采用方式 |
|---|---|---|---|
| 样式引擎改为增量维护，保留无缓存参考计算 | 8 月月报 [3] | PR #11088 的失效传播、继承、CSSOM 等提交说明 [4] | 实现方向可信；不推定普遍更快 |
| 布局缓存具备缓存命中后重算比较的 shadow mode | 8 月月报 [3] | PR #11113 明确记录参考计算抓出的 SVG 分歧与缓存失效测试 [5] | 可直接借鉴验证方法 |
| renderer 曾能滥用 Cookie/HSTS 特权 | 9 月月报 [1] | PR #12113 的网络进程接管与拒绝特权操作说明 [6] | Ladybird 已披露问题；不能直接归因给 ZeroWeb |
| 长期后台导航会保留旧文档 | 9 月月报 [1] | PR #11645 的导航请求生命周期说明 [7] | 采用复现场景；具体内存数字不跨场景合并 |
| 媒体移到独立进程 | 9 月月报 [1] | PR #12181 的代理、时钟、IPC 和权限变更说明 [8] | 作为架构参考，非 ZeroWeb 实现结论 |
| Alpha 尚按 2026 年目标推进 | 官网 [2] | 8 月剩余交付清单与 9 月已落地项目 [1][3] | 记录目标；不宣布已经发布 |

月报和 PR 来自同一项目，不算独立第三方复测。本文用实现细节核对公告，不将这种交叉核对包装为独立性能认证。

## 增量样式与布局：参考计算比重写本身更值得学

8 月，Ladybird 替换了选择器匹配、样式失效、级联和计算样式保留机制。它把 DOM 变更、状态变化和 CSSOM 编辑表示为有类型的增量事件，路由到依赖这些输入的选择器，输出未改变时停止传播。官方用“持续查询与物化视图”解释这个模型。[3][4]

正确性检查有两层：无缓存计算器作为参考实现；真实浏览会话的输入事件可记录并确定性回放，检查增量输出是否分歧。官方同时承认，新引擎在不少负载上仍比传统方式慢，因此不宜把它描述为全面性能胜出。[3]

布局缓存按 formatting context 类型、输入约束和根状态等建立键，保存可在新位置复用的片段树。shadow mode 在命中缓存后仍重新布局，发现差异即报错。PR 具体记录了失效世代、视口、字体引用、语言变化和动画继承等边界，还说明参考计算抓出了 SVG 变换依赖上一轮结果的问题。[5]

ZeroWeb 已有 `compute_styles_incremental`：重算变更节点的祖先链和子树，遇到 `:has()` 保守退回全量。它已有实质增量能力，但这一入口不同于 Ladybird 的事件依赖路由。参见 `crates/style-system/src/lib.rs:454–540`。ZeroWeb 布局还包含 Taffy 与多个后处理 pass，参考路径必须比较最终输出，不能只验证 Taffy 缓存。[11][12]

**建议（推理）**：先给现有增量路径增加“增量结果对完整计算”的开发期检查，覆盖移动节点、CSSOM 编辑、伪元素、继承字体、语言、容器尺寸、Shadow DOM 与 `:has()`；再根据实测热点决定是否引入更精细的依赖路由。布局检查同时比较几何、溢出、基线、命中测试与最终像素，并单独观测缓存保留内存。

> 来源说明：Ladybird 一手材料 [3][4][5]；ZeroWeb 源码与工程约束 [11][12]；建议为推理，尚未实施或测量。

## 性能与长期资源：从异常小操作追到真实页面

Ladybird 8 月报告的连续基准变化为：Speedometer 2 约 47→64，Speedometer 3 约 2.5→3.9，StyleBench 约 3.5→83。9 月在连续 Linux runner 上分别又报告约 30%、50%、80% 的提升。[1][3] 这些是项目自报的各月口径，本文不跨机器拼接或计算累计收益。

它同时比较单个 DOM/API 操作与其他浏览器的差距：例如 `fillStyle` 颜色解析从约 Chromium 的 90 倍耗时降到约 2 倍，`replaceChildren()` 从 13 倍降到不足 4 倍。[3] 这种方法适合 ZeroWeb：已有 V8 并不意味着 DOM 访问、JS/Rust 桥接、树更新和绘制便宜。

9 月的资源修复覆盖三个实际场景：[1][7][9][10]

- 零延迟 timer 链丢失嵌套层级，未触发规范要求的 4ms 最小延迟；隐藏标签还有 timer 节流问题。
- GitHub 视口外 CI spinner 持续唤醒 renderer/compositor；修复后停止相应动画工作并跳过未变化帧的呈现。
- 新闻页面不断重载，导航请求和待执行回调保留旧文档。月报记录 Politico 四小时约 11GB、El País 二十小时约 15GB；PR 的另一轮 Politico 观察为五小时约 13GB。它们是不同运行记录，不是同一测量的冲突。

ZeroWeb 已有基准与性能门禁，也有后台 renderer 轮询节流测试 `renderer_poll_order_prioritizes_active_tab_and_throttles_background_tabs`。该测试不等于证明页面 timer、动画和 compositor 全链路停止无效工作。[13]

**建议（推理）**：补充跨浏览器操作基准，先测 `getComputedStyle`、`getBoundingClientRect`、`replaceChildren`、列表插删和 Canvas 状态切换；长期资源场景则固定为后台 timer、视口外动画、重复导航与停滞子资源。记录各进程 CPU、RSS 趋势、队列长度、旧文档存活数和实际呈现次数，阈值从同环境基线建立。

> 来源说明：性能和内存数值为 Ladybird 单来源自报 [1][3][7]；实现补充 [9][10]；ZeroWeb 当前能力 [13]；测试建议为推理。

## 进程安全：Rust 不能替代权限边界

9 月，Ladybird 与 Trail of Bits 在 OpenAI 赞助的 Patch the Planet 项目中进行了一周高强度安全检查，发现了完整利用链和大量待修问题。其威胁模型明确假定恶意页面可以控制 renderer，其他进程必须把 renderer 消息当作不可信请求。[1]

具体修复包括：网络进程接管 HTTP 响应和 WebSocket 握手的 Cookie 处理；禁止普通 renderer 使用读取、替换 HttpOnly Cookie 等特权接口；HSTS 写入不再由 renderer 申请；私密会话归属由可信端记录，而不相信请求携带的私密模式标志。[6] Linux 还调整了 Landlock 生效时机，避免 GPU 驱动线程先创建而逃过限制，并默认拒绝在无 Landlock 时启动相应 helper。[1][14]

跨站 iframe 进程隔离已能运行，web test runner 默认覆盖此路径，但 9 月月报明确说正常浏览仍以顶层隔离为主，嵌入 frame 留在包含页面的进程。这是“测试默认”与“产品默认”不同的例子。[1]

ZeroWeb 的 `WsHost::connect` 接收页面来源和 cookie 参数，`ws_bridge.rs` 包含 HTTP-facing Cookie 与 shim Cookie 合并，`WsEvent::SetCookie` 会将握手响应投递回 shim；这些位置应成为审计入口。[15] 这仅证明存在跨层数据路径，不证明存在跨站读取、HttpOnly 泄露或可利用漏洞。是否越权还要追踪真实进程中的存储所有权、身份校验和消息权限。

**建议（推理）**：优先审计 Cookie/HSTS/权限/存储 IPC 的可信身份归属，检查恶意 renderer 能否伪造 origin、URL、会话或自动化身份；核对 helper 的 Linux/macOS 实际沙箱策略与线程启动顺序。安全验收应注入非法进程请求，验证拒绝及资源终止，而不能仅靠 `document.cookie` 的页面级测试。

> 来源说明：Ladybird 一手材料 [1][6][14]；ZeroWeb 源码 [15]；审计优先级为推理。本次没有验证 ZeroWeb 安全漏洞。

## 合成、媒体与站点功能：选具体链路借鉴

8 月开始，Ladybird 把可合成的 opacity/transform 动画作为已解析关键帧与时间参数交给 compositor；页面 JS 查询样式或动画状态时按需采样，有限动画在结束时唤醒主线程派发事件。9 月扩展了部分 background-color/filter 情况，并默认启用页面与 worker 的 OffscreenCanvas 2D/WebGL/WebGL2。[1][3][16]

**对 ZeroWeb 的建议（推理）**：以“页面主线程被长 JS 阻塞时，动画与 worker canvas 是否继续呈现”为验收场景，同时检查 `getComputedStyle`、`currentTime`、结束事件和命中测试，避免只有画面运动而页面语义滞后。ZeroWeb 已有 compositor，新增价值在减少对 renderer 主线程的依赖及保持查询一致。

9 月的媒体变化包括 macOS VideoToolbox 硬件解码、解码帧免像素复制进入 compositor，以及独立媒体进程。代理保存脚本同步可读状态，时钟通过共享内存读取，其余状态由事件更新；媒体进程负责 demux、解码和播放。[1][8] ZeroWeb 当前架构文档仍将 `zero-media` 描述为进程内底座。[11]

**对 ZeroWeb 的建议（推理）**：如果长视频实测显示 CPU、拷贝或 decoder 隔离是瓶颈，再评估平台解码与媒体进程。借鉴重点是媒体时钟、状态代理、错误归类和崩溃降级，不能仅增加进程后就认为完成隔离。

真实站点修复也能提供定向用例：GitHub 文件行依赖 `content-visibility`/`contain-intrinsic-size`；Slate 编辑器依赖 composition 事件；模块预加载需要正确 CORS mode；Shopify sticky header 使用 scroll-state query。[1] 8 月新增声明式 JSON 站点规则，处理 UA sniffing；9 月允许按站点暴露部分实验接口。[1][3]

**对 ZeroWeb 的建议（推理）**：把这些站点缩减成独立 fixture，再验证真实点击、事件、几何与生产 GPU 输出。若引入站点规则，应区分 UA 分流与引擎缺陷，记录规则理由、范围和移除条件；不能让例外规则取代通用修复。

> 来源说明：Ladybird 一手材料 [1][3][8][16]；ZeroWeb 架构 [11]；所有落地建议为推理，未跑功能验收。

## Rust、交付与协作取舍

7 月迁移样式和布局之后，8 月 CSS 解析、计算样式和绘制管线进一步由 Rust 承担。解析迁移按语法族启用 Rust/C++ A/B 比较，序列化输出零分歧后删除旧路径；两语言字符串统一表示以减少复制。[3] 对已经以 Rust 为主的 ZeroWeb，价值主要是迁移和重构时的差分验证方法，不构成更换语言或自研 JS 引擎的理由。

Alpha 的工作开始集中到签名安装包、自动更新、首次运行、下载入口和无终端崩溃报告。9 月新增本地 crash report、用户选择发送的崩溃页面、自动更新过滤列表，并把桌面 GUI 收敛为 Qt。[1][3] 本次未确认安装签名和自动更新已全部完成。

6 月已公布停止接收公众 PR，仅维护者引入代码；仍接受 bug、缩减用例、网站测试、安全报告与技术反馈。官方理由包括 AI 降低伪装贡献成本，以及维护者承担理解和长期维护责任。[17] 这是旧调研时间范围内的既有政策，不是 8、9 月新增事件。

**对 ZeroWeb 的建议（推理）**：保留 V8/QuickJS 和稳定 ZeroWebView 嵌入边界；为下一次对外体验明确安装、更新、崩溃恢复和反馈闭环。协作上借鉴“代码责任与可复现证据”，无需照搬关闭公众 PR，也无需照搬 Qt 单前端或缩减 Android/Windows 目标。

> 来源说明：Ladybird 一手材料 [1][3][17]；ZeroWeb 项目边界 [11]；路线建议为作者综合。

## 建议优先级与交付判据

下表是建议，不是已授权实施计划。成本、收益尚未测量。

| 顺序 | 建议 | 可检查产物 | 判据 |
|---|---|---|---|
| 优先 | 按 renderer 已被攻破审计特权边界 | 消息权限表、实际调用链、非法请求用例 | 服务端绑定身份；越权操作被拒绝；普通页面不可使用自动化特权 |
| 优先 | 给现有增量样式与布局建立参考计算检查 | 变更序列、完整计算结果、差异报告 | 计算样式与最终几何/像素符合预期；缓存资源有界 |
| 优先 | 将资源测试扩到后台与重复导航 | 各进程 CPU/RSS、旧文档与队列趋势 | 无持续无效呈现；旧文档可释放；结果按同环境基线判断 |
| 随后 | 增加跨浏览器 DOM/桥接微基准 | 同页面、同操作的耗时与调用计数 | 找出数量级异常；优化后保留行为正确性 |
| 小范围试验 | 合成动画与 worker canvas | 主线程阻塞 fixture、画面与脚本状态证据 | 动画继续，查询与事件仍正确 |
| 按瓶颈推进 | 平台媒体解码与进程隔离 | 长视频 CPU/拷贝数据、故障注入结果 | 解码崩溃转为播放错误；renderer 权限减少 |
| 对外体验前 | 安装、更新与崩溃反馈闭环 | 可安装产物、更新与恢复演示 | 普通用户无需开发终端完成反馈和恢复 |

WPT 指标应分别保留固定快照进展和上游新增测试的贡献。Ladybird 9 月通过子测试从 2,088,677 增至 2,109,072，新增 20,395，其中约一半来自上游新增测试。[1] 这不是 20,395 个既有失败被修复。ZeroWeb 的 reftest、自一致性、Chromium oracle、Web API subtest 与 Ladybird 综合 WPT 计数分母不同，不能据相近百分比推断两者成熟度相同。[11][12][13]

质量核对：主要公告已对照相关 PR 作者说明；性能自报与推理已区分；保留 Alpha、产品默认隔离和新样式引擎负载限制；未用旧调研数字冒充当前数据。未独立复测 Ladybird，未做 ZeroWeb 功能或安全验收。

> 来源说明：优先级与判据为作者综合；指标事实 [1]；本项目度量边界 [11][12][13]。

## 参考资料

| 编号 | 来源 | 类型与用途 |
|---|---|---|
| 1 | [Ladybird September 2026](https://ladybird.org/newsletter/2026-09-30/) | 官方月报，安全、媒体、资源、WPT、交付 |
| 2 | [Ladybird 官网](https://ladybird.org/) | 官方状态与平台目标 |
| 3 | [Ladybird August 2026](https://ladybird.org/newsletter/2026-08-31/) | 官方月报，性能、样式、布局、Rust、Alpha |
| 4 | [PR #11088](https://github.com/LadybirdBrowser/ladybird/pull/11088) | 作者提交说明，增量样式边界 |
| 5 | [PR #11113](https://github.com/LadybirdBrowser/ladybird/pull/11113) | 作者提交说明，布局缓存、shadow oracle |
| 6 | [PR #12113](https://github.com/LadybirdBrowser/ladybird/pull/12113) | 作者提交说明，Cookie/HSTS/会话权限 |
| 7 | [PR #11645](https://github.com/LadybirdBrowser/ladybird/pull/11645) | 作者说明，旧文档与导航请求释放 |
| 8 | [PR #12181](https://github.com/LadybirdBrowser/ladybird/pull/12181) | 作者说明，媒体 IPC 与生命周期 |
| 9 | [PR #11612](https://github.com/LadybirdBrowser/ladybird/pull/11612) | 隐藏页面 timer 节流 |
| 10 | [PR #11715](https://github.com/LadybirdBrowser/ladybird/pull/11715) | 空闲动画与呈现 |
| 11 | [ZeroWeb 架构](../architecture.md) | 当前本地架构材料；历史数字保留各自版本 |
| 12 | [ZeroWeb 工程方法论](../../.agents/skills/zeroweb-guidelines/SKILL.md) | 缓存、双路径、最终 pass 与像素验证边界 |
| 13 | [性能与 oracle 知识](../../.agents/skills/zeroweb-knowledge/references/engineering/testing/performance-and-oracles.md)、[process_backend.rs](../../apps/browser/src/process_backend.rs) | 门禁与后台轮询测试源码 |
| 14 | [PR #12110](https://github.com/LadybirdBrowser/ladybird/pull/12110) | 沙箱、文件描述符和 GPU 初始化 |
| 15 | [ws_bridge.rs](../../crates/engine/src/ws_bridge.rs)、[style-system/lib.rs](../../crates/style-system/src/lib.rs) | 当前源码，Cookie 路径与增量入口 |
| 16 | [PR #12211](https://github.com/LadybirdBrowser/ladybird/pull/12211) | OffscreenCanvas 页面与 worker 渲染 |
| 17 | [Changing How We Develop Ladybird](https://ladybird.org/posts/changing-how-we-develop-ladybird/) | 官方协作政策，2026-06-05 |

正文以编号对应本表。GitHub 页面包含机器人评论；技术结论优先取月报和作者说明，不把机器人建议当已实现功能。部分网页提取为相关片段，未宣称完整源码审查。
