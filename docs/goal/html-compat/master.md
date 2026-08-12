# HTML 行为兼容主控面板

## 当前状态

- 阶段：M0
- 状态：功能完成，GPU 门禁待 CI
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
| M1 | PageNodeRef、pressed target、JS-disabled、输入事件、焦点/激活 | pending |
| M2 | `page-runtime` 共享默认动作核心 | pending |
| M3 | 文本、选择/表单、导航/交互元素族 | pending |
| M4 | live renderer WebDriver 与 WPT testdriver | pending |

## 跨线边界

CSS、布局或绘制根因只在本目标记录最小复现，移交对应开发线，不在本线旁路修补。

## 下一步

提交 M0 阶段进展并进入 M1 `PageNodeRef` spike；同时观察 CI 的 GPU 门禁结果。
