# F06/F08 网页交互、显示与安全

## 用户任务与预期

输入文本、焦点/IME/编辑、表单提交和取消，运行脚本/DOM，显示 CSS、字体、图片、Canvas，
使用无障碍/Clipboard/Fullscreen；安全策略拒绝不允许的资源和权限请求。
预期来自 [HTML 行为规格](../../../../docs/specs/html-behavior-compatibility-spec-rfc.md)、
页面 API 契约和既有安全 gate，不以当前错误结果反向改写预期。
来源：`crates/webview/src/tests/user_actions.rs`、
`tests/integration/src/html_compat.rs`、`crates/webview/src/tests/{csp_gate,mixed_content_gate}.rs`、
`apps/browser/src/parity_smoke.rs`、`crates/engine/src`。

## 准备与驱动

真实网站/GUI 功能按 [parity 入口](../../zeroweb-browser-chrome-parity/SKILL.md) 和
[证据契约](../../zeroweb-browser-chrome-parity/references/evidence-contract.md) 执行，
复用场景和生产窗口采集器。本轮没有创建场景或采集截图。
先 Doctor：当前四产品进程和执行体、Node/Chrome、窗口/GPU、同 viewport/DPR/locale/
主题/字体/scrollbar、本次 profile 和本地 HTTP fixture；失败不采集。
现有模板是
[表单场景](../../zeroweb-browser-chrome-parity/templates/form-interaction.scenario.json)。
场景放本次 `.verify-task/`，先校验，再按原入口的 JSON argv 设置生产器并运行
`run-parity.mjs`，外层用既有内存/墙钟守卫。Chrome 真实 mouse move/down/up，
ZeroWeb 浏览器输入；禁止 `element.click()` 代替真实输入。

低成本 API 诊断可定向执行
`zero-webview --lib tests::user_actions::text_focus_and_stale_actions_use_scoped_identity`，
用既有守卫保护编译与运行。它调用生产 dispatch_user_action，
检查插入 A、取消 X 后值仍为 A、beforeinput/input 序列；不验证窗口 hit-test 或 IME。
完整函数名执行前用受守卫的 `--list` 确认，0 tests 不能 PASS。

## 判据与失败恢复

每步独立比较可见值/checked、取消/提交导航、事件序列、DOM/焦点、几何及像素。
稳定帧与字体/图片加载按 parity 原规则，不固定 sleep 冒充稳定。
视觉通过不能替代语义，CPU/headless/单测不是生产窗口 GPU。
安全分支用本地受控 HTTP/HTTPS 和 CSP fixture，检查资源被阻止、违规事件与合法资源
恢复；不访问未授权线上身份，不默认授予 clipboard 或系统权限。
缺工具、字体或安全上下文记 BLOCKED；不改 manifest 冒充生产身份。

## 证据、清理与导航

保留同检查点双端全帧、manifest、状态/事件/几何和 report.json；完整 parity 按原规则展示
Chrome/ZeroWeb 两张全帧。只关闭本次 browser/Chrome/server、删除本次 profile/fixture，
保留证据并重读；失败恢复重建本次实例再 Doctor。
[工程方法论](../../zeroweb-guidelines/SKILL.md)、
[地图](index.md)。2026-10-14 F06/F08 所有路径 `NOT_RUN`，
本轮不宣称 Chrome 一致性、安全门禁或 GUI 交互通过；环境见 [记录](../initialization.md)。
