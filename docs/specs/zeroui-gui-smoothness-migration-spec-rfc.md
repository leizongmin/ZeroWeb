# ZeroUI GUI 流畅性机制迁移 Spec + RFC

> 状态：实施中
> 版本：v1.0
> 日期：2026-08-12
> 参考实现：工作区同级目录 `../ZeroUI`
> 目标仓库：ZeroWeb

## 0. 执行摘要

本计划把 ZeroUI 已验证的按需帧、分级失效、retained 文本编辑状态、统一焦点/IME 路由、跨帧渲染缓存和性能门禁迁移到 ZeroWeb。迁移对象是机制和可复用实现，不是把网页 DOM 替换成 ZeroUI Widget 树。

实施按 M0～M5 六个可独立回滚的里程碑推进。每个里程碑必须先有自动化测试，再修改生产代码；阶段测试通过后提交并推送。最终验收不依赖人工点击。

首个落地步骤是把当前已完成的表单交互修复资产化为稳定回归测试，随后测量一次按键实际触发的 parse/style/layout/paint/publish 次数。没有测量证据前不做大范围性能重构。

## 1. 背景与问题

ZeroWeb 已具备 live DOM、脏矩形、增量布局、字形缓存、多进程 renderer/compositor 和按需窗口重绘，但页面表单输入仍存在以下结构性问题：

- 文本输入通过 JS shim 写 `value` 属性，作为通用 DOM mutation 处理。
- 通用属性 mutation 会序列化整份 DOM，并可能退回整视口 style/layout/paint。
- 焦点、命中目标、按键、IME 和默认表单行为跨 browser/renderer/JS shim 多处维护。
- 页面帧发布与窗口 redraw 只有布尔级控制，缺少 style/layout/paint/composite/publish 分级失效。
- ZeroUI 中已落地的 GPU 图片纹理缓存、持久资源和批处理优化尚未完整同步到 ZeroWeb。
- 缺少覆盖“点击控件 → 输入 → IME → 按钮动作 → 上屏”的自动性能场景。

## 2. 范围

### 2.1 必须迁移

1. 表单交互自动化回归与阶段耗时/次数观测。
2. 页面表单控件 retained 编辑状态：值、光标、选区、composition、焦点。
3. browser → renderer 的统一文本输入与 IME 协议。
4. renderer 内统一焦点、pointer target、默认行为路由。
5. 页面渲染分级失效与同一事件循环批次的帧合并。
6. input value-only 变更的局部绘制路径；固定几何时不得触发 HTML parse 或全量 layout。
7. ZeroUI 中适用于 ZeroWeb 的 GPU 图片纹理缓存、持久 uniform/绑定资源与安全批处理。
8. 表单输入性能场景、基线、预算门禁和产品级 GUI 验收。

### 2.2 非目标

- 不用 ZeroUI Widget 树替换 HTML DOM/CSSOM/layout tree。
- 不把浏览器页面排版改写为 ZeroUI layout。
- 不在本计划中重做全部 HTML 表单规范；仅补齐迁移所需的焦点、文本编辑、IME 和按钮默认行为。
- 不通过降低文字、图片或视觉质量换取性能数字。
- 不修改 ZeroUI 仓库。

## 3. 需求规格

### FR-001：表单交互端到端自动验证

场景：多个控件连续交互
假设浏览器打开仓内表单示例页
当自动测试依次点击第一个 input、输入 ASCII、点击 textarea、输入文本、点击 checkbox 和 button
那么每个控件均收到正确焦点/事件，值和页面结果符合预期
验证：browser/renderer 集成测试与 GUI smoke 场景

场景：不同缩放下命中一致
假设窗口 scale factor 为 1.0、1.25、1.5、2.0
当测试点击控件可视中心
那么 renderer 命中的 DOM selector 与视觉控件一致
验证：坐标映射参数化测试

### FR-002：retained 表单编辑状态

场景：输入只更新当前控件
假设固定尺寸 input 已聚焦
当插入、删除或替换文本
那么控件值、光标、选区和 DOM property 同步，其他控件状态不变
验证：engine/webview 单元测试

场景：脚本读写保持兼容
假设页面脚本读取或写入 `input.value`
当 retained 状态处于 dirty value 状态
那么 property 语义正确，`defaultValue` 不被错误覆盖
验证：JS DOM bridge 回归测试

### FR-003：中文 IME 与 composition

场景：中文提交
假设页面文本控件已聚焦
当平台发送 preedit 更新并最终 Commit“中文”
那么预编辑状态可见，Commit 作为一个文本批次插入并触发规范要求的 composition/input 事件
验证：IME 路由集成测试

场景：composition 取消
假设存在未提交 preedit
当焦点丢失或平台禁用 IME
那么临时 composition 被清除，未提交文本不写入 value
验证：状态机单元测试

### FR-004：统一焦点与默认行为路由

场景：输入框之间切换
假设第一个输入框已聚焦
当点击第二个输入框或按 Tab
那么旧控件收到 blur/change，新控件收到 focus，后续文本只进入新控件
验证：renderer 状态机测试

场景：按钮与选择控件
假设页面包含 submit/reset/button/checkbox/radio
当点击控件且事件未 preventDefault
那么只执行一次对应默认行为
验证：renderer 多进程路径回归测试

### FR-005：分级失效与帧合并

场景：value-only 输入
假设 input 几何和影响布局的样式不变
当一次事件批次更新 value/caret/composition
那么最多发布一帧，只标记 paint/publish，不执行 HTML parse 或全量 layout
验证：管线计数断言

场景：布局相关脚本回调
假设 input 事件监听器修改 width 或 DOM 结构
当回调完成
那么失效升级为 style/layout/paint/publish，结果与全量渲染一致
验证：增量与全量快照对照测试

### FR-006：渲染资源跨帧复用

场景：重复页面帧
假设连续帧引用相同图片和字形
当 GPU renderer 上屏
那么相同图片不重复创建 texture/bind group，字形/shape 缓存保持有效
验证：缓存命中计数与 GPU 单元测试

场景：同 key 内容变化
假设图片 key 相同但像素内容变化
当下一帧上屏
那么缓存以尺寸和内容摘要区分，不复用旧纹理
验证：缓存失效单元测试

## 4. 非功能需求

- NFR-001：表单输入到帧发布 p95 必须在固定机器基线基础上满足 `baseline * 1.10 + 2ms`。
- NFR-002：20ms 卡顿帧比例不得高于 `min(baseline + 0.02, 0.05)`。
- NFR-003：固定尺寸 input 的 value-only 输入每批 `parse_count = 0`、`full_layout_count = 0`、`publish_count <= 1`。
- NFR-004：所有渲染/布局变更必须通过相关单测、`make test`、scoped reftest、product smoke 与 clippy。
- NFR-005：CPU/GPU、单进程/多进程和不同 scale factor 的行为不得分叉。
- NFR-006：公共 API 具备文档；规范行为代码附对应 WHATWG/UI Events 规范链接。

## 5. 技术设计

### 5.1 选定方案

采用“页面 retained 控件状态 + 分级失效 + 合并帧”的增量方案。

拒绝以下替代方案：

- 每次输入继续写 HTML 属性并优化全量管线：无法消除序列化和全局重算。
- 直接嵌入 ZeroUI TextInput 覆盖网页 input：会破坏 DOM/CSS/JS 语义和进程边界。
- 只移植 GPU 优化：能降低上屏成本，但不能解决焦点、IME 和每键全量布局。

### 5.2 状态模型

每个可编辑表单控件维护稳定 node/handle 对应的 `FormControlState`：

- `value`
- `selection_start` / `selection_end`
- `composition_text` / `composition_range`
- `dirty_value`
- `focused`
- `revision`

状态属于页面运行时，与 live DOM 同生命周期；HTML 序列化仅用于快照、调试和兼容边界，不再作为每次按键的真值存储。

### 5.3 失效模型

采用可组合标志：

- `NEEDS_STYLE`
- `NEEDS_LAYOUT`
- `NEEDS_PAINT`
- `NEEDS_COMPOSITE`
- `NEEDS_PUBLISH`
- `NEEDS_HIT_TEST`

失效只允许升级。layout 蕴含 paint/hit-test/publish；paint 蕴含 publish；仅 caret/composition/value 绘制变化通常为 paint/publish。

### 5.4 输入到上屏流程

```text
winit key/IME/pointer
  → browser 坐标归一化与 Input/IME IPC
  → renderer 统一 focus/pointer/default-action 路由
  → FormControlState 原地更新
  → 同批 JS event + mutation 收集
  → 计算最高失效级别
  → 最多一次 style/layout/paint
  → 最多一次 frame publish
  → compositor/browser 请求一次 redraw
```

### 5.5 GPU 迁移边界

从 ZeroUI `render-foundation` 有选择地迁移：

- image texture/bind group cache，key 包含 image key、尺寸和内容摘要。
- renderer 生命周期内持久 uniform buffer/bind group。
- 保持 painter order 的 vertex-buffer 合并与相邻同材质批处理。
- present-only 性能测量路径与视觉 readback 路径严格分离。

## 6. 实施里程碑

| 里程碑 | 状态 | 自动验证 |
|---|---|---|
| M0 | 已完成并推送 | 真实 renderer 多控件交互、IME Commit、1.0～2.0 DPI、CPU/GPU 快照、全工作区测试与 clippy |
| M1 | 已完成并推送 | retained input/textarea 值与选区、IDL/内容属性分离、Unicode 编辑、`0/0/0/1` paint-only、无整页序列化/图片重扫 |
| M2 | 已完成，待阶段提交 | 单一 focus owner/pointer target、IME 全生命周期、候选窗锚点、单/多进程一致性、真实 renderer 自动回归 |
| M3～M5 | 待实施 | 按下列顺序推进 |

### M0：交互正确性基线

- 固化当前点击、第二输入框、按钮、坐标和中文 Commit 修复。
- 建立多进程自动回归；示例页保留在 `examples/forms/`。
- 验证现有改动后提交并推送。

### M1：观测与 retained 编辑状态

- 增加 input 事件各阶段计数和耗时。
- 新增 `FormControlState`，先支持 input/textarea 的 value/caret/selection。
- JS getter/setter 与绘制读取 retained 状态。
- 固定尺寸 value-only 输入不再序列化整 DOM 或全量 layout。

### M2：统一 focus/pointer/IME

- renderer 建立单一 focus owner 与 pointer target。
- browser 只负责平台事件和坐标归一化，不推测表单默认行为。
- 完成 preedit/commit/cancel、composition 事件和 IME caret rect。
- 覆盖 Tab、blur/change、selection 和多控件连续交互。

完成记录：

- `PageInteractionState` 统一 keyboard/IME focus owner 与最新 pointer target，按钮焦点不再被文本控件状态覆盖。
- browser/renderer 协议显式传递 Enabled/Preedit/Commit/Disabled；临时 preedit 仅 paint，不写入 `value`。
- winit 页面态启用 IME，并根据文本控件点击位置、滚动和 DPI 设置候选窗锚点。
- 单进程 worker 与多进程 renderer 共用 composition 生命周期语义。
- `scripts/test-form-interaction.ps1` 先构建真实 renderer，再自动验证 1.0/1.25/1.5/2.0 DPI 下两个文本控件、按钮、preedit 与中文 Commit。

### M3：分级失效与帧合并

- 在 engine/webview/page-runtime/protocol 中落地失效契约。
- 一次平台事件及其 JS 回调合并为一个渲染事务。
- value/caret 局部 paint；布局相关 mutation 自动升级。
- compositor/browser 合并过期页面帧和重复 redraw。

### M4：GPU 缓存与批处理

- 回移并适配 ZeroUI 图片纹理缓存。
- 回移持久 GPU 绑定资源和安全 vertex batching。
- 保持 CPU/GPU 像素一致和图片内容变化正确失效。

### M5：性能门禁与最终验收

- 新增 `form_input` 性能报告和平台基线。
- 纳入 perf gate、GUI smoke、product smoke 和 CI。
- 完成全工作区 fmt/clippy/test、scoped reftest 和性能对比。
- 把根因、迁移经验和后续边界记录到 `docs/learnings/`。

## 7. 提交与推送策略

每个里程碑至少一个独立提交，格式建议：

1. `test(gui): lock form interaction baseline`
2. `perf(input): retain form editing state`
3. `fix(input): unify focus pointer and ime routing`
4. `perf(render): add staged invalidation and frame coalescing`
5. `perf(gpu): cache image resources and batch uploads`
6. `test(perf): gate form input smoothness`

每次推送前执行：

1. `git pull --rebase origin codex/zeroui-gui-migration`（远端分支存在后）。
2. 按阶段运行受管测试和质量门禁。
3. 调用 pre-commit 安全门禁。
4. 推送当前分支，禁止 force push。

## 8. 验证矩阵

| 维度 | 覆盖 |
|---|---|
| 控件 | input、textarea、button、submit、reset、checkbox、radio |
| 输入 | ASCII、UTF-8、Backspace、Enter、Tab、IME preedit/commit/cancel |
| 状态 | focus、blur、change、selection、caret、composition |
| 进程 | single-process、browser→renderer→compositor |
| 渲染 | CPU、GPU、脏矩形、全量兜底 |
| 缩放 | 1.0、1.25、1.5、2.0 |
| 性能 | input-to-publish p50/p95/max、jank、parse/style/layout/paint/publish 计数 |

## 9. 回滚策略

- 每个里程碑独立提交，可按里程碑 revert。
- retained 状态迁移期间保留全量渲染兜底；任何增量/全量结果不一致时升级失效，不静默显示旧帧。
- GPU 缓存可按组件禁用用于定位，但不能成为长期绕过测试的开关。
- 不删除旧路径，直到对应行为、视觉和性能门禁全部通过；删除动作单独提交。

## 10. 风险

- HTML input property 与 attribute 语义不同；必须通过现有 `defaultValue`/dirty value 测试约束。
- JS 事件回调可任意修改 DOM/CSS，失效分类必须保守升级。
- 坐标跨 DPI、页面滚动和 chrome inset，必须统一在边界处转换一次。
- IME preedit 平台行为不同，状态机不能只验证 Commit。
- GPU 缓存必须包含内容摘要并设置有界淘汰，防止旧纹理和无界内存增长。

## 11. Spec Lint

| 检查 | 裁决 |
|---|---|
| 每条功能需求有自动验证场景 | Pass |
| Requirement、Decision、Assumption 边界明确 | Pass |
| 实现来源和模块边界明确 | Pass |
| 包含迁移顺序、回滚点和提交策略 | Pass |
| 不依赖人工验收才能判定完成 | Pass |

## 12. 实施合同

- 必须按 M0→M5 顺序推进；提前动作仅限为后续阶段补测试夹具。
- 每阶段测试未通过不得提交为“完成”。
- 不覆盖或丢弃进入本任务前已有的工作区修改。
- 不为性能数字跳过视觉路径、降低质量或放宽既有门禁。
- 阶段性进展及时提交并推送到 `codex/zeroui-gui-migration`。
- 最终完成标准是 M0～M5 全部通过验证矩阵和仓库质量门禁。
