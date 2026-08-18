---
date: 2026-08-12
modules: zero-engine, zero-webview, zero-renderer
---

# 表单 live value 被全量重载清空

## 问题描述

文本输入已正确绘制，但随后激活 checkbox、radio 或 reset 后提交表单，文本字段会退回 HTML `value` 内容属性中的默认值。

## 根因分析

`SetFormValue` 只更新 `RenderPipeline::form_control_values` retained state，不写回 HTML 内容属性。表单控件 helper 已通过 `apply_dom_mutations_and_render` 更新 live DOM 和绘制结果，但 renderer 随后又调用 `reload_html_after_script`。全量重载会重新解析默认 HTML，并清空 retained form state。

因此，视觉更新看似成功，后续 form-data 构造却只能读到默认内容属性。

## 解决方案

1. 已经执行增量 DOM mutation 的路径只发布现有渲染结果，不再次 full reload。
2. form-data 构造显式接收 selector 到 live value 的覆盖表。
3. reset 通过 `SetFormValue` 把默认值写回 retained state，确保后续提交读取重置后的值。
4. 测试同时断言 live value、HTML default value 和最终提交 URL，避免视觉断言掩盖状态丢失。
