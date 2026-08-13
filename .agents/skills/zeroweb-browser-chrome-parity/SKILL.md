---
name: "zeroweb-browser-chrome-parity"
description: "对比 ZeroWeb 与 Chrome 的真实点击、页面状态、事件、几何和生产 GPU 帧。用户要求 Chrome 一致性或交互渲染验收时使用。"
---

# ZeroWeb Chrome 一致性验收

本 Skill 用于为页面或交互场景生成可审计的 Chrome-vs-ZeroWeb 证据。视觉一致性和行为一致性是两个独立门禁；只有两者都经过规定的生产路径，才能报告“完整一致”。

## 硬性规则

1. 不得用 `element.click()` 充当点击证据。Chrome 必须执行 `mouse.move`、`mouse.down`、`mouse.up`；ZeroWeb 必须走浏览器输入链路或 WebDriver 自动化链路。
2. 不得把 CPU、engine-direct、纯 headless 或单元测试截图描述为生产窗口 GPU 证据。
3. 两端必须使用同一场景、viewport、DPR、locale、颜色主题、字体环境和 scrollbar 策略。
4. 每个稳定检查点分别比较状态、事件序列、几何和像素，不得把四项平均成单一分数。
5. 缺少必需产物时判定失败。只有用户明确要求诊断模式或仅静态模式时才能降级。
6. 全图和控件区域像素阈值都是严格小于关系。`3%` 表示结果必须 `<3%`。
7. 验收结束时必须向用户直接展示同一检查点的 Chrome 和 ZeroWeb 两张全帧截图，不得只提供报告路径或文字结论。
8. 字体由其他目标负责时，可对文本控件区域声明 `glyphMaskInsetPx`，但全图仍须未遮罩比较，报告也必须保留区域的 `unmasked` 原始结果。checkbox、radio 等非字形控件不得使用该遮罩。

## 必读资料

创建场景、实现 ZeroWeb 证据生产器或解释报告前，完整阅读 [references/evidence-contract.md](references/evidence-contract.md)。

## 平台支持

核心脚本使用 Node.js ESM，不依赖 Bash、GNU `timeout` 或 `realpath`，支持：

- Windows 10/11：PowerShell、CMD 或其他能启动 Node.js 的终端。
- Linux：X11 或 Wayland；无人值守 GUI 验收可使用 Xvfb。
- macOS：原生窗口环境。

跨平台不等于无环境要求。完整生产验收仍要求：

- Node.js 20+；
- 仓库 Rust 工具链和 `test-guard`；
- Chrome/Chromium 127，或场景指定的版本；
- 可用的窗口系统和 GPU adapter；
- Windows 首次构建前配置 `RUSTY_V8_ARCHIVE`；
- Linux/macOS 首次构建前执行 `make setup-rusty-v8`。

Chrome 自动探测顺序见 `capture-chrome.mjs`。也可在所有平台显式设置 `PUPPETEER_EXECUTABLE_PATH`。生产视觉验收优先连接预启动的 GUI Chrome：`ORACLE_CDP_URL=http://127.0.0.1:9222`。

## 执行流程

1. 选择证据目录。默认使用仓库内已被 `.gitignore` 忽略的 `.acceptance/chrome-parity/<scenario-name>-<timestamp>/`，一次性或 CI 诊断也可使用系统临时目录。不得把截图写入受版本控制的源码或文档目录。

2. 准备仓库依赖：

   ```bash
   cd tests/wpt-runner/scripts
   npm ci
   ```

   构建命令须通过仓库 `test-guard` 包裹。Linux/macOS 示例：

   ```bash
   ./target/test-guard --time-limit 900 -- cargo build --release \
     -p zero-wpt-runner -p zero-browser -p zero-renderer \
     -p zero-compositor -p zero-webdriver
   ```

   Windows 使用仓库在该平台提供的等价 `test-guard` 入口，不得裸跑长时间构建。

3. 复制 [templates/form-interaction.scenario.json](templates/form-interaction.scenario.json)，只修改页面 URL、观察目标和动作。

4. 校验场景：

   ```bash
   node .agents/skills/zeroweb-browser-chrome-parity/scripts/validate-scenario.mjs <scenario.json>
   ```

5. 采集 Chrome 证据。优先使用 CDP 连接预启动的 GUI Chrome：

   ```bash
   node .agents/skills/zeroweb-browser-chrome-parity/scripts/capture-chrome.mjs \
     --scenario <scenario.json> \
     --out <evidence-dir>/chrome
   ```

   未设置 `ORACLE_CDP_URL` 时脚本会启动 headless Chrome。该结果可用于行为诊断，但不满足生产视觉门禁。

6. 用仓库命令生成 ZeroWeb 证据。命令会收到：

   ```text
   PARITY_SCENARIO
   PARITY_OUTPUT_DIR
   PARITY_REPO_ROOT
   ```

   命令必须写出 `<evidence-dir>/zeroweb/manifest.json`。完整验收的 manifest 必须声明：

   ```json
   {
     "engine": "zeroweb",
     "capturePath": "production-window-gpu",
     "inputPath": "browser-pointer"
   }
   ```

   本仓内置生产器：

   ```text
   ZEROWEB_EVIDENCE_COMMAND=["target/release/zero-browser","--renderer=gpu","--scale=1","--parity-scenario","${PARITY_SCENARIO}","--parity-output-dir","${PARITY_OUTPUT_DIR}"]
   ```

7. 比较证据：

   ```bash
   node .agents/skills/zeroweb-browser-chrome-parity/scripts/compare-evidence.mjs \
     --scenario <scenario.json> \
     --chrome <evidence-dir>/chrome \
     --zeroweb <evidence-dir>/zeroweb \
     --out <evidence-dir>/report.json \
     --require-production
   ```

8. 一键编排使用跨平台 Node 入口。将 ZeroWeb 生产器命令写成 JSON 字符串数组，避免 shell quoting 的平台差异：

   ```text
   ZEROWEB_EVIDENCE_COMMAND=["cargo","run","--release","--bin","zero-parity-producer"]
   ```

   然后执行：

   ```bash
   node .agents/skills/zeroweb-browser-chrome-parity/scripts/run-parity.mjs \
     <scenario.json> <evidence-dir>
   ```

9. 展示验收截图。优先选择最终检查点；若验收提前失败，选择最后一个两端均有截图的检查点。使用 `view_image` 分别展示 `<evidence-dir>/chrome/<step-id>.png` 和 `<evidence-dir>/zeroweb/<step-id>.png`，并在最终回复中以内联图片并排或相邻呈现，清楚标注 `Chrome` 与 `ZeroWeb`。同时给出两张图片及 `report.json` 的绝对路径，便于用户打开原图核对。

   两张图必须来自同一场景、同一检查点、同一 viewport 和 DPR。不得用 diff 图、控件裁剪图或其他检查点截图代替任一全帧截图。

## ZeroWeb 适配器优先级

1. 生产浏览器窗口 + 多进程 renderer/compositor + 浏览器指针输入 + wgpu present + 严格 GPU readback。
2. `zero-webdriver` 仅用于行为诊断。它持有 live renderer，但不是最终浏览器窗口 compositor 路径。
3. Rust `HtmlScenario` 测试仅用于快速回归定位。

如果第 1 条无法逐步输出证据，必须报告缺少产品自动化能力并停止完整验收。不得把第 2 条的行为证据与第 1 条无关的初始截图拼接成“端到端通过”。

## 稳定帧条件

每个动作后等待所有适用条件：

- 页面完成导航和加载；
- 页面状态断言成立；
- snapshot 或 frame sequence 已前进；
- 连续两次采样帧一致；
- 字体和图片完成加载。

使用有上限的轮询并保留诊断信息。不得只依赖固定 sleep。

## 完整通过标准

- Chrome `capturePath` 为 `chrome-cdp-gui`；
- ZeroWeb `capturePath` 为 `production-window-gpu`；
- ZeroWeb `inputPath` 为 `browser-pointer`；
- 每个检查点的可观察状态一致；
- 归一化事件序列一致；
- 所有观察矩形偏差不超过几何阈值；
- 每张全帧 diff 严格小于 `maxDiffPercent`；
- 每个控件区域 diff 严格小于 `maxRegionDiffPercent`；
- 不缺少任何必需产物或检查点。

## 失败定位

- 状态/事件不一致但几何一致：检查默认动作、焦点所有权、事件取消和 retained form state。
- 几何不一致但状态接近：检查布局、UA 样式、viewport、DPR 和 scrollbar 占位。
- 几何一致但像素不一致：检查边框、绘制顺序、控件 appearance、抗锯齿和字体区域。
- headless 通过但产品窗口失败：检查 compositor IPC、surface 初始化、scale 换算、GPU fallback 和 readback 来源。
- 仅非 1 DPR 点击失败：检查 document → content → physical 坐标换算。

评测样例见 [evals/evals.json](evals/evals.json)。
