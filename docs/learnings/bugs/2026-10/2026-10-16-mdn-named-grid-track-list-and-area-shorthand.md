---
date: 2026-10-16
modules: crates/style-system, crates/layout-engine
---

# 命名 Grid 轨道与单命名区域简写不可丢弃

## 问题

Windows 上用 ZeroWeb release 与 Chrome 对照 MDN Web 首页时，标题原本落在 x=16 的左栏。Chrome 在 x=291.5，宽 768px。MDN 的响应式 Grid 使用 `[line-name …]` 轨道边界、`rem` 长度及 `grid-area: header` 命名区域。

## 根因

`grid-template-columns` 校验器把合法的方括号命名线当成无效轨道 token，并不接受 `rem`，因此整条列定义回落为 `none`。布局转换先把边界命名当轨道处理；首轮修复虽跳过了这些 token，却也丢掉了命名线信息。子项引用 `grid-column-start: content-start` 时，剩余 `GridLineValue::Name` 又被转成 `auto`，因此显式命名线仍不能放置子项。另一个级联入口中，`grid-area: header` 只展开 `grid-row-start`，其余三条线变成 `auto`；直接属性解析测试覆盖了该值，但级联简写展开测试没有。

## 修复

- 将非空 `[<custom-ident> …]` 识别为轨道边界标注，允许其中 CSS Grid 规定的名称；它不创建轨道。
- `[none]` 是合法线名；校验器只排除 Grid 线名语法禁止的 `auto`、`span`，以及 `<custom-ident>` 禁止的 CSS-wide / `default`。
- 将轨道尺寸和同一显式网格线上的名称一起转换：尺寸进入 Taffy track list，边界名进入平行的 `grid_template_*_names` 列表；固定 repeat 展开边界名，auto-fill/auto-fit 名称保留在 repetition 中。子项命名线保留为 Taffy `NamedLine`，由其 resolver 按容器线名放置。
- 保留真正的 `minmax()` 与间距轨道；长度通过样式系统转换 `em`、`rem` 和 viewport 单位。
- `grid-area` 单个命名区域展开为 row/column 的 start/end 四条命名线；单个数字仍只设置 row-start。

## 回归

样式解析测试覆盖 MDN 的 5 轨道模板；布局回归检查五条轨道各自的 `minmax()`/`fr`/`rem` 尺寸、六组命名线边界、固定与自动 repeat 命名线，并用 Taffy 实际布局验证子项按显式 named line 放到目标列。`grid-area` 简写测试覆盖四条名称。Windows 原生 `zero-style-system` 2413/2413、`zero-layout-engine` 1913 通过且 2 项 ignored，affected-crate strict clippy 和 release browser/peer 构建通过。

完整页面仍有差异：MDN 的菜单使用 JS shim 当前不参与生产布局的 Shadow DOM；关闭态下菜单面板仍可见，H1 位置差 7.5px，全帧差 10.01%。完整 Chrome parity 未通过，也没有证明本修复带来加载速度收益。
