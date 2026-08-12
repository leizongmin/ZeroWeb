# HTML 行为兼容主控面板

## 当前状态

- 阶段：M2
- 状态：M1 完成，M2 实施中
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
| M2 | `page-runtime` 共享默认动作核心 | in_progress |
| M3 | 文本、选择/表单、导航/交互元素族 | pending |
| M4 | live renderer WebDriver 与 WPT testdriver | pending |

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
- [ ] renderer 委托共享 WebView coordinator 并删除重复 action 逻辑

## 下一步

切换 renderer 到共享 WebView coordinator，完成 M2。
