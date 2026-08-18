---
date: 2026-08-12
modules: zero-engine 渲染管线, renderer 表单输入
---

# 表单当前值更新应走 paint-only 路径

## 问题描述

文本框每输入一个字符都会记录 `value` 属性变更。旧分类器把它当作普通属性变化；第一次优化虽跳过样式和布局，却仍会序列化整棵 DOM 并重扫图片子资源，连续输入依然存在与页面规模相关的卡顿。

## 根因分析

输入框的外部尺寸由控件样式和 UA 尺寸规则决定，当前值只参与控件内部文字绘制。更关键的是，HTML 的 IDL `input.value` 当前值与 `value` 内容属性不是同一个状态；用户编辑和 `.value = ...` 不应改写内容属性，因此 `[value]` 选择器也不应随输入变化。把两者混为一个 `SetAttr` mutation 同时造成了错误语义和昂贵的整页快照。

## 解决方案

新增独立的 `SetFormValue` 通道，将 input/textarea 当前值保存为页面级 retained 状态，绘制器直接读取该状态。纯当前值批次不修改内容 DOM、不生成 HTML 快照、不刷新图片子资源，并复用 computed styles 与 layout；`setAttribute("value", ...)` 继续走普通 DOM 属性路径。通过 `parse_count/style_count/layout_count/paint_count` 自动断言快速路径为 `0/0/0/1`，同时断言 `[value]` 内容属性选择器保持不变。

## 如何避免

新增高频交互状态时先判断它是 IDL/retained 状态还是内容 DOM，再定义最高失效级别。性能测试必须覆盖序列化、子资源扫描、style、layout、paint 全链路；不要仅以“没有重新解析 HTML”作为增量渲染已经完成的判断。
