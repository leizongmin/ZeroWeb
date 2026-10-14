# 真实 UI 驱动、DevTools 与自动化兼容边界

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

定位 JSONL 真实窗口驱动、生产 GPU 截图、Chrome parity 场景、DevTools frontend 资源服务与 Playwright 命令面。`ui_driver` 调 BrowserApp 输入，`parity_smoke` 观测状态/事件/几何，headless `devtools_serve` 服务外供静态 bundle，三者并非同一种验收。

关键入口：`UiDriver::tick`、`ParitySmokeConfig::load`、`render_full_scene_gpu_capture`、DevTools 路径校验。协议语义见[CDP/BiDi](cdp-and-bidi.md)，证据组织见[工程验收](../../engineering/testing/performance-and-oracles.md)。

验证导航：[自动化地图](../../../../zeroweb-verify/features/embedding-automation.md)与[Chrome parity 原入口](../../../../zeroweb-browser-chrome-parity/SKILL.md)；本轮只读实现，未生成截图或执行流程，zeroweb-verify 为工作区新增，不归上述 HEAD。

## Gotchas
- UiDriver 的 click 发按下再释放，move/key/type 走 BrowserApp 与窗口事件相同的处理方法，shot 用生产场景 GPU 离屏读回；DOM element.click() 或 WebDriver CPU screenshot 不能替代这一条输入/截图链。 (`apps/browser/src/ui_driver.rs`, `apps/browser/src/parity_smoke.rs`)
- wait_idle 只等待 any_tab_loading 为 false 后静默 settle，或到 deadline 即放行；它不断言页面达到空闲，不能作为后台 JS/网络全部完成的成功条件。 (`apps/browser/src/ui_driver.rs`)
- UiDriver 整体期限 600 秒，超时/步骤失败也尝试落 result.json；写报告为 best-effort，需实际读取产物，不能只凭退出日志判断已有完整证据。 (`apps/browser/src/ui_driver.rs`)
- assert_url 默认 contains，prefix 模式才区分错误 scheme 前缀；assert_nav/status/zoom 读 app_smoke_state，与壳层渲染同源，不是从屏幕 OCR 推断状态。 (`apps/browser/src/ui_driver.rs`, `apps/browser/src/app_smoke_state.rs`)
- shot 文件名拒绝 ..、斜杠和反斜杠，输出只能落到配置目录内；自定义场景不能利用文件名向外写截图。 (`apps/browser/src/ui_driver.rs`)
- parity 场景只支持 version=1、非空步骤/状态表达式、en-US、no-preference reduced motion、light/dark 和非空 DPR=1 viewport；改变这些条件不能仍称为当前采集器支持的环境。 (`apps/browser/src/parity_smoke.rs`)
- DevTools bundle 必须外供，未配置返回 503，配置目录内缺文件返回 404；发现响应的 devtools:// URL 只是入口形态，不能证明 frontend 可加载。 (`apps/browser/src/headless/devtools_serve.rs`, `apps/browser/src/headless/discovery.rs`)
- DevTools 静态路径同时做词法过滤与 canonical 根约束，防 .. 与 symlink 逃逸；只验证请求字符串前缀不足以保护 bundle 目录。 (`apps/browser/src/headless/devtools_serve.rs`)
- Playwright 直连 CDP_ENDPOINT_URL 时跳过捕获代理，只产 steps-report；没有 capture.jsonl 不等于执行失败，也不能用直连结果声称捕获了全部协议流量。 (`tests/playwright-matrix/README.md`, `tests/playwright-matrix/scripts/capture-core-flow.mjs`)
- scripts/rally-devtools.sh 启动的是长期目标推进，不是一次 DevTools smoke；阅读它或链接它不授权启动无人值守网站/调试优化任务。 (`scripts/rally-devtools.sh`)
- UiDriver::type 逐字符调用 handle_key，未经过 IME preedit/commit；字符能输入不证明候选框、组合文字或 InputConnection 正确，IME 验收需要对应宿主入口。 (`apps/browser/src/ui_driver.rs`, `apps/browser/src/app_input.rs`)
- UiDriver 创建时开始计算600秒 deadline，配置只拒绝空路径，不拒绝已有输出目录；shot 的 File::create 会覆盖同名图，所以新运行应隔离输出，路径安全不等于旧证据安全。 (`apps/browser/src/ui_driver.rs`)

## Architecture
- parity manifest 分别保留每步截图、区域、live 状态、事件、几何、activeElement 和 URL；像素相同不能替代输入事件或页面状态一致的判据。 (`apps/browser/src/parity_smoke.rs`)
- 捕获代理改写发现响应的 webSocketDebuggerUrl 指向自身，避免 WebSocket 绕过观测；命令矩阵以记录的 wire 流量生成，不由支持方法名列表推断。 (`tests/playwright-matrix/README.md`, `tests/playwright-matrix/scripts/cdp-capture-proxy.mjs`)

## Dependencies
- playwright-core 1.63.0 和 ws 8.21.0 在该测试工程固定，Node 属于测试依赖不进入产品构建；不同客户端版本的握手/初始化命令差异不能直接归成产品回归。 (`tests/playwright-matrix/README.md`, `tests/playwright-matrix/package.json`)

## Testing Strategy
- UiDriver 单测覆盖路径参数与步骤解析，不驱动窗口或验证截图；DevTools resolver 单测覆盖入口、转义穿越、非法百分号与缺文件，不能证明外供 bundle 的面板已加载或协议语义完备。 (`apps/browser/src/ui_driver.rs`, `apps/browser/src/headless/devtools_serve.rs`)
- Playwright 核心流逐步骤捕获错误后继续，因此生成 steps-report 不表示整流通过；必须读取每步 ok/error，顶层异常也尝试落 fatal 报告，不能只按是否存在报告判断成功。 (`tests/playwright-matrix/scripts/capture-core-flow.mjs`)
