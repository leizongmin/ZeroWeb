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

下方定向 CLI 共用本次证据目录；在项目根执行，目录创建或守卫检查失败时停止：

```sh
VERIFY_ROOT=.agents/skills/zeroweb-verify/.verify-task
mkdir -p "$VERIFY_ROOT"
VERIFY_RUN="$VERIFY_ROOT/$(date +%Y%m%d-%H%M%S)-page"
mkdir "$VERIFY_RUN"
test -x target/test-guard
```

### calc 尺寸与浮动回归

R5117（`50190390123d763e9bdc141a45942810ef9e0ff0`）对应现有
`crates/layout-engine/src/engine/tests/r5117_calc_cb_base_tests.rs`。从项目根执行，
先按主入口 Prepare 准备并确认 `target/test-guard` 可执行。将下面的
`VERIFY_RUN` 设为本次新建的 `.verify-task/<ID>/` 目录，所有日志保留在其中。
编译与运行一起受内存和墙钟守卫；正常默认分支先清除本次 shell 中的开关：

```sh
unset ZW_CALC_CB_BASE
./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- \
  cargo test --locked -p zero-layout-engine --lib r5117_calc_cb_base_tests -- --list \
  > "$VERIFY_RUN/calc-list.log" 2>&1
./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- \
  cargo test --locked -p zero-layout-engine --lib r5117_calc_cb_base_tests -- --nocapture \
  > "$VERIFY_RUN/calc-test.log" 2>&1
```

列表必须包含且运行必须通过两个测试：
`test_r5117_normal_block_calc_percent_base_is_containing_block` 检查300px容器中的
`calc(50% - 10px)` 宽140px；`test_r5117_float_content_box_calc_side_by_side`
检查 content 140px、border-box 150px，两浮动同 y、第二个 x 接第一个右缘，
容差0.5px。这是生产 LayoutEngine 的几何回归，未覆盖窗口像素、height、
absolute/fixed 或父子均 calc。开关比较需另起进程并分别记录环境，
不能在同一进程修改 OnceLock 已读取的变量。
知识边界见 [盒尺寸与定位](../../zeroweb-knowledge/references/page-core/layout/boxes-and-positioning.md)。

### Touch 构造器与属性

来源为 `Makefile` 的 `testharness-touch-events`、
`tests/wpt-runner/src/main.rs` 的同名子命令和
`testharness.rs` 的 `run_touch_events_cases` / `touch_events_case_skipped`。
先确认 Git/网络、Rust/V8 依赖与守卫可用，再按现有导入器准备语料；
导入器会写共享的、被忽略的 `tests/wpt-runner/wpt-data/touch-events/`，
执行前记录已有数据，不与其他导入任务并发。显式包裹准备和构建：

```sh
./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- \
  make fetch-wpt-touch-events > "$VERIFY_RUN/touch-fetch.log" 2>&1
./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- \
  cargo build --locked --release --bin zero-wpt-runner \
  > "$VERIFY_RUN/wpt-build.log" 2>&1
./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- \
  ./target/release/zero-wpt-runner testharness-touch-events --json \
  > "$VERIFY_RUN/touch-results.json" 2> "$VERIFY_RUN/touch-run.log"
```

定向构造器时在子命令后加路径子串 `constructor`；完整通道同时核对实际收集的
文件与子测试，不把过滤运行写成整个 Touch 支持通过。用例集必须非空、全部
子测试为 Pass，确认 Touch/TouchEvent 构造、列表元素身份与 item 越界、
touches/targetTouches/changedTouches 和 ontouch* 曝光的适用断言。
collector 仅收集 HTML，排除 manual、crashtest 和含 test_driver/testdriver.js 的
宿主注入用例；此入口不验证真实单指/多指触摸、pinch、兼容鼠标或 Android 输入。
列表存在也不证明完整 WebIDL 或 getModifierState。
知识见 [事件与默认动作](../../zeroweb-knowledge/references/scripting/event-loop/events-and-default-actions.md)。

### R5118 待复核的像素与盒几何

历史来源为 `3b548ed7804ecf0e050da00557ee8adb654e615e` 的
[渲染目标记录](../../../../docs/goal/rendering-compat.md)。该提交没有业务代码修复。
control-chars-00B 的0/7失败、border-width-applies-to-008 的90/270几何是历史报告，
本轮没有复现。选择这两个案时，先按既有 WPT 准备入口备齐语料与字体，
使用上节受守卫的当前 runner 构建；每案分别保存输出：

```sh
./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- \
  ./target/release/zero-wpt-runner reftest-upstream control-chars-00B --json \
  > "$VERIFY_RUN/control-chars-00B-report.json" 2> "$VERIFY_RUN/control-chars-00B.log"
./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- \
  ./target/release/zero-wpt-runner reftest-upstream border-width-applies-to-008 --json \
  > "$VERIFY_RUN/inline-border-report.json" 2> "$VERIFY_RUN/inline-border.log"
```

该子命令通过 `--json` 输出到 stdout；不用帮助文本中未在此分支接线的 `--output`。
核对非空结果、命中的 case ID 和 match/mismatch 类型；
control-chars-00B 应异于空白 ref，0%像素差不能 PASS。
inline-border 要分别检查装饰盒、行盒和基线，并按上方 parity 入口补 Chrome
同条件生产截图与几何；不得把90误记为规范预期，也不能直接给行高加边框。
runner self-ref/CPU 结果不证明生产 GPU 或 Chrome 一致性。
根因边界见 [文本绘制](../../zeroweb-knowledge/references/page-core/paint/painter.md)、
[行内度量](../../zeroweb-knowledge/references/page-core/layout/inline/metrics-and-writing-modes.md)。

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
定向 CLI 另记录 HEAD、相关差异、实际 argv、开关/字体/语料版本、退出码、
用例/子测试数和断言；依赖缺失记 BLOCKED，执行不符记 FAIL，零测试不能 PASS。
结束后只清理本次可证明归属的临时数据，保留日志/JSON并重新读取；
已有 WPT 缓存、共享 target 和旧报告保持原状。本轮只维护文档，未创建运行实例。
[工程方法论](../../zeroweb-guidelines/SKILL.md)、
[地图](index.md)。2026-10-14 F06/F08 所有路径 `NOT_RUN`，
本轮不宣称 Chrome 一致性、安全门禁或 GUI 交互通过；环境见 [记录](../initialization.md)。
