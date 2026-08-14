# ZeroWeb Chrome 一致性证据契约

## 场景结构

场景使用 JSON：

```json
{
  "version": 1,
  "name": "form-interaction",
  "url": "file://${REPO_ROOT}/examples/forms/form-interaction-test.html?__zero_test_state=1",
  "viewport": { "width": 800, "height": 720, "dpr": 1 },
  "environment": {
    "locale": "en-US",
    "colorScheme": "light",
    "reducedMotion": "no-preference"
  },
  "thresholds": {
    "maxDiffPercent": 3,
    "maxRegionDiffPercent": 5,
    "channelDiff": 8,
    "pixelRadius": 1,
    "maxGeometryDiffPx": 2
  },
  "observe": {
    "selectors": ["#name", "#note"],
    "stateExpression": "JSON.parse(document.querySelector('#test-state').textContent)",
    "eventTypes": ["mousedown", "focus", "mouseup", "click", "input", "change"]
  },
  "steps": [
    { "id": "initial", "action": { "type": "snapshot" } },
    { "id": "focus-name", "action": { "type": "click", "selector": "#name" } },
    { "id": "type-name", "action": { "type": "type", "text": "abc" } },
    { "id": "tab", "action": { "type": "key", "key": "Tab" } }
  ]
}
```

脚本会展开 `${REPO_ROOT}`。每个步骤 ID 必须唯一，并且可安全用作文件名。

Chrome 端支持以下动作：

- `snapshot`
- `click`，需要 `selector`
- `type`，需要 `text`
- `key`，需要 Puppeteer 支持的 `key`
- `wait`，需要 `milliseconds`，仅用于诊断

页面专用的 `stateExpression` 必须返回可 JSON 序列化的数据。只记录 Web 可观察行为，不加入引擎私有字段。

`observe.selectors` 使用页面 `querySelector` 语义，允许 ID、class、属性、组合器等 CSS selector，不限于 `#id`。Chrome 和 ZeroWeb 都在各自 live document 的页面脚本上下文执行同一 `stateExpression` 并读取同一组 selector；页面不需要写入 `#test-state`、修改 title 或暴露其他验收专用通道。表达式报错、结果不可序列化或 selector 对应几何缺失时必须保留明确诊断，不得回退为解析 HTML 字符串猜测状态。

`stateExpression` 的语法、运行时异常和 JSON 可序列化性只能在真实页面上下文中判定，因此静态 validator 只检查它是非空字符串，采集器负责在失败时输出诊断。`click` 的 selector 不必出现在 `observe.selectors`；两端均在动作发生时从当前 live document 单独解析点击目标。

生产采集当前明确支持 `locale: "en-US"`、`reducedMotion: "no-preference"`，以及 `colorScheme: "light" | "dark"`。validator 会拒绝 ZeroWeb 尚不能真实应用的环境值，避免 Chrome 单边模拟后产生伪一致性结论。

## Manifest 结构

每个引擎写出 `manifest.json`：

```json
{
  "schemaVersion": 1,
  "scenario": "form-interaction",
  "engine": "chrome",
  "engineVersion": "Chrome/127.0.0.0",
  "capturePath": "chrome-cdp-gui",
  "inputPath": "browser-pointer",
  "viewport": { "width": 800, "height": 720, "dpr": 1 },
  "steps": [
    {
      "id": "initial",
      "action": { "type": "snapshot" },
      "screenshot": "initial.png",
      "state": {},
      "events": [],
      "geometry": {
        "#name": {
          "x": 118,
          "y": 201,
          "width": 193,
          "height": 40
        }
      }
    }
  ]
}
```

文件路径相对于 manifest 所在目录。证据生产器可以增加字段，但不得改变必需字段的语义。

## 事件归一化

比较前只保留：

```json
{
  "type": "click",
  "target": "#subscribe",
  "defaultPrevented": false
}
```

不比较时间戳。可以附加 value 和 checked 快照用于诊断，但 canonical state 比较仍是权威结果。

采集器使用 capture phase，以观察 focus 和不冒泡事件；在 microtask 中更新 `defaultPrevented`，确保后续 listener 的取消结果可见。不得通过程序化 dispatch 制造事件日志。

无 `id` 的事件目标记录稳定的 `tag:nth-of-type(n)` DOM 路径，以区分页面中多个同标签元素；有 `id` 时仍使用 `#id`。

## 点击语义

Chrome：

1. 解析 selector。
2. 获取可见 bounding box。
3. 未指定归一化 offset 时取中心点。
4. 执行 `mouse.move`、`mouse.down`、`mouse.up`。
5. 校验事件目标或动作后的状态。

ZeroWeb：

1. 在当前 document generation 中解析目标。
2. 在 renderer hit-test 区域内寻找点击点。
3. 把 document 坐标转换为 page-content 物理坐标。
4. 执行浏览器 mouse move、press、release。
5. 校验实际 target selector。
6. 等待新 compositor 帧后再截图。

WebDriver `ElementClick` 可用于行为诊断，但必须声明 `inputPath: "webdriver-element"`，不能满足完整生产输入门禁。

## 像素比较

标准比较器：

```bash
zero-wpt-runner compare-png \
  <zeroweb.png> <chrome.png> \
  --max-diff 3 \
  --channel-diff 8 \
  --pixel-radius 1
```

全图和控件区域阈值均为严格小于。

字体由其他 goal 负责时，仍保留未遮罩的全图结果，同时增加排除 glyph mask 的布局/控件报告。不得丢弃全图结果。

场景可为文本控件声明内部字形遮罩：

```json
{
  "observe": {
    "selectors": ["#name", "#submit"],
    "glyphMaskInsetPx": {
      "#name": 3,
      "#submit": 3
    }
  }
}
```

值表示从区域四边保留的像素宽度；比较器仅把其余内部像素统一为白色。selector 必须同时出现在 `observe.selectors`，值必须为非负整数。该能力只用于字体字形由其他目标负责的文本控件，不得用于 checkbox、radio 等需要完整 native appearance 证据的控件。

遮罩后的结果仍使用 `maxRegionDiffPercent` 判定，并在区域报告中记录 `glyphMaskInsetPx`。同一区域未遮罩的原始结果保存在 `unmasked` 字段；全图 `pixels` 结果始终不遮罩。

## 生产证据边界

完整 Chrome 一致性要求：

```text
Chrome GUI CDP
以及
ZeroBrowser 真实窗口
-> 多进程 zero-renderer
-> compositor
-> wgpu present
-> 严格 GPU readback
```

以下路径仅能用于诊断：

- engine-direct framebuffer
- CPU raster screenshot
- 不含浏览器窗口合成的 ZeroWeb headless GPU
- `zero-webdriver` live renderer
- Rust 单元或集成测试快照

这些仍是有价值的底层回归测试，但必须如实标注 `capturePath` 和 `inputPath`。

## 跨平台命令契约

一键编排器读取 `ZEROWEB_EVIDENCE_COMMAND`，其值必须是 JSON 字符串数组：

```json
["cargo", "run", "--release", "--bin", "zero-parity-producer"]
```

禁止传 shell 命令字符串。JSON argv 不依赖 Bash、PowerShell 或 CMD quoting，可在 Windows、Linux、macOS 使用。
参数中的 `${PARITY_SCENARIO}`、`${PARITY_OUTPUT_DIR}` 和 `${PARITY_REPO_ROOT}` 会在启动前替换为绝对路径。

生产器会收到：

```text
PARITY_SCENARIO=<场景绝对路径>
PARITY_OUTPUT_DIR=<ZeroWeb 证据目录绝对路径>
PARITY_REPO_ROOT=<仓库根目录绝对路径>
```

路径由 Node.js `path.resolve()` 生成。生产器必须使用这些环境变量，不得假设系统临时目录、用户主目录、盘符或路径分隔符。

## 产物布局

```text
evidence/
├── chrome/
│   ├── manifest.json
│   └── <step-id>.png
├── zeroweb/
│   ├── manifest.json
│   └── <step-id>.png
└── report.json
```

任一门禁失败时保留完整 evidence 目录。
