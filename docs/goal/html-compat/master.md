# HTML 行为兼容主控面板

## 当前状态

- 阶段：M4
- 状态：M0-M3 完成，M4 实施中
- 主 fixture：`examples/forms/form-interaction-test.html`
- 完成标准：FR-001、FR-012

## M0 工作包

- [x] `HtmlScenario`/`PageQuery` 测试 helper
- [x] helper 正常与失败诊断测试
- [x] fixture 覆盖文本、焦点、checkbox、radio、button、reset、submit 状态
- [x] renderer 完整语义序列测试
- [x] browser 真实多进程完整交互测试
- [x] scoped 测试、fmt、workspace clippy
- [ ] `make test` GPU 专用测试（本机无 wgpu adapter，交由 GPU/CI 环境）

## 后续里程碑

| 里程碑 | 主题 | 状态 |
|---|---|---|
| M1 | PageNodeRef、pressed target、JS-disabled、输入事件、焦点/激活 | completed |
| M2 | `page-runtime` 共享默认动作核心 | completed |
| M3 | 文本、选择/表单、导航/交互元素族 | completed |
| M4 | live renderer WebDriver 与 WPT testdriver | in_progress |

## 跨线边界

CSS、布局或绘制根因只在本目标记录最小复现，移交对应开发线，不在本线旁路修补。

## M1 工作包

- [x] `PageNodeHandle/PageNodeRef/PageTarget` contract
- [x] renderer → protocol → browser document generation
- [x] hit-test opaque node handle
- [x] press/release 稳定目标与 stale generation 取消
- [x] JavaScript 禁用时 UA 默认动作
- [x] beforeinput/input 与键盘入口统一
- [x] focusability、焦点事件与 label 激活
- [x] checkbox/radio activation rollback
- [x] reset/submit 取消语义完整验收

## M2 工作包

- [x] `html_actions` typed plan/prepare/rollback/commit 核心
- [x] renderer checkedness adapter 使用 shared plan
- [x] renderer 文本 adapter 使用 shared plan
- [x] renderer focus adapter 使用 shared effect
- [x] renderer reset/submit adapter 使用 shared plan/effect
- [x] TabWorker 文本与 focus adapter 使用 shared plan/effect
- [x] TabWorker checkedness 与 reset/submit adapter 使用 shared plan/effect
- [x] 单进程 POST form navigation transport
- [x] ZeroWebView identity-based user-action adapter
- [x] 三执行器默认动作与取消 conformance
- [x] 20 轮确定性短序列重放
- [x] TabWorker 委托共享 WebView coordinator 并删除重复 action 逻辑
- [x] renderer 委托共享 WebView coordinator 并删除重复 action 逻辑

## M2 验证备注

- `form_input` 性能子门禁通过：p95 `0.0295ms`、jank `0`，每次输入 parse/style/layout 为 `0`、paint/publish 为 `1`。
- 整套 `make bench-gate` 的全局比较因平台基线 CPU 不一致不可归因：当前 Xeon 8260 KVM 对比 i5-13500H 基线，102 个跨 crate 指标同步超预算；未修改或放宽基线。

## M3a 工作包

- [x] input/textarea live value 与 default value 分离及 reset
- [x] 非文本 input selection getter/方法适用性
- [x] readonly 与 maxlength 用户编辑约束
- [x] minlength 用户值约束校验与 reset 清理
- [x] caret、点击 hit-test 与 IME rect 共用 paint 边界

## M3b 工作包

- [x] successful controls 文档序与显式 form owner
- [x] disabled fieldset 交互/提交传播与首个 legend 例外
- [x] select/option 用户激活事务与三宿主一致性
- [x] output/defaultValue reset 与 form owner 集成

## M3c 工作包

- [x] a/area 跨文档导航 intent 与 click 取消
- [x] 同文档 fragment/hash 激活
- [x] details/summary 激活与取消
- [x] dialog/popover 目标校验与 top-layer 状态

## 下一步

### M4 工作包

- [x] live renderer automation IPC 与有界 owner loop
- [x] WebDriver live renderer session 与 HTML 交互端点
- [ ] WPT testharness reporter 与 testdriver 动作
- [ ] forms/focus/input-events 选定子集

实施 WPT testharness reporter 与 testdriver 动作。
