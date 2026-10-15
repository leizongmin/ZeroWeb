---
date: 2026-10-16
modules: crates/style-system, crates/layout-engine
---

# 命名 Grid 轨道与单命名区域简写不可丢弃

## 问题

Windows 上用 ZeroWeb release 与 Chrome 对照 MDN Web 首页时，标题原本落在 x=16 的左栏。Chrome 在 x=291.5，宽 768px。MDN 的响应式 Grid 使用 `[line-name …]` 轨道边界、`rem` 长度及 `grid-area: header` 命名区域。

## 根因

`grid-template-columns` 校验器把合法的方括号命名线当成无效轨道 token，并不接受 `rem`，因此整条列定义回落为 `none`。布局转换也把边界命名当轨道处理。另一个级联入口中，`grid-area: header` 只展开 `grid-row-start`，其余三条线变成 `auto`；直接属性解析测试覆盖了该值，但级联简写展开测试没有。

## 修复

- 将非空 `[<custom-ident> …]` 识别为轨道边界标注，允许其中 CSS Grid 规定的名称；它不创建轨道。
- 转换轨道时跳过边界标注，保留真正的 `minmax()` 与间距轨道；长度通过样式系统转换 `em`、`rem` 和 viewport 单位。
- `grid-area` 单个命名区域展开为 row/column 的 start/end 四条命名线；单个数字仍只设置 row-start。

## 回归

样式解析测试覆盖 MDN 的 5 轨道模板，布局转换测试覆盖命名标注不增加轨道及 `15rem=240px`、`2rem=32px`，简写测试覆盖四条名称。Windows 原生 `zero-style-system` 与 `zero-layout-engine` 全包测试分别通过；Windows release 浏览器与同机 Chrome 的 MDN 页面实测也确认三栏与标题几何恢复。

完整页面仍有差异：MDN 的菜单使用 JS shim 当前不参与生产布局的 Shadow DOM；关闭态下菜单面板仍可见。不要据此声称整页 Chrome parity 已通过。
