# DC-2 contrast-color() computed 刀（css-color-5 §contrast-color）

- 日期：2026-11-11 晚
- 流：zeroweb-2-ba（color 长尾桶 5）
- 代码：36de2a6f6（part11-computed-color.js +42）
- 验证链：g7（unit zw-g7-contrast-2118，树含 contrast WIP，HEAD=b68481d28）

## 实现（三件）

1. `_zwCCParseColorValue` contrast-color parse 臂：恰一 `<color>` 实参，
   多参/空参拒；mix 侧/relative origin 位置内参可解析（嵌套 contrast-color
   递归到通用解析）。
2. `_zwCCResolve` contrast 臂：内色解析 → sRGB 钳位 → WCAG 相对亮度
   （sRGB 线性化 0.04045/12.92、2.4 幂）→ L≤0.17913 选白否则黑；判据对齐
   engine paint/color.rs `contrast_color_of`。结果不携内色 alpha
   （`color(srgb 1 0 1 / 0.5)` 期望无 alpha rgb() 形，R5107 同口径）。
3. `__zwComputedColor` 顶层零漂移门：基础形内参（不含 color-mix/light-dark/
   contrast-color 嵌套/相对 from/calc()）保宿主 R5107 路径（关键字/系统色/
   currentcolor 宿主已正确解析——覆盖前同路径）；建模域内参落通用解析由
   kind 'contrast' 臂接管。sign()/abs() 形在前置臂先行送宿主。

## g7 账（css-color 语料 66 页，fd2 基线 10,948）

- **10,948 → 10,962 净 +14、零回归**：
  - 真收割：color-computed-contrast-color-function.html **4F→0F 全清**
    （`contrast-color(white/black/pink)`、`color(srgb 1 0 1 / 0.5)`、
    `lab(0.2 0.5 0.2)`、`color(srgb ±10 …)`、嵌套
    `contrast-color(contrast-color(pink))`、currentcolor、calc 通道内参、
    color-mix 侧内参——期望均为 any-of 白/黑，WPT 页宽松断言）
  - 账面锚：zz-caprobe-interp.html 8P/0F + zz-caprobe-real.html 2P/0F
    （本地探针页首入账，.gitignore 域内不入库，非真实缺口）
- 单跑页账：computed 页 17P/0F；valid 页 16P/1F；
  contrast-color-function-calc-container.html 0P/2F。

## 残余 3F（分面挂账）

- valid 页 1F：写面 canonicalization——
  `contrast-color(color(srgb calc(0.5) calc(1 + 1 / 1) 1 / .5))` 声明应规范化
  `calc(2)`/`0.5`（declared face 折叠域，非 computed）。
- calc-container 2F：① computed `contrast-color(color(srgb calc(1 +
  (sign(20cqw - 10px) * 1)) …))` 得 rgba(0,0,0,0) 透明回退——cqw 容器宽
  缺面（color 长尾桶 1 同根因，容器感知查找落地后随收）；② 写面
  `calc(1 + (sign(20cqw - 10px) * 0.5))` canonical 序 `(0.5 * sign(…))`
  操作数序 + `.5`→`0.5`（declared canonicalization）。

## 归属与并发注记

桶 5 认领与实现、证据、提交均本流（zeroweb-2-ba）。g7 跑期间他流两笔
engine 提交（f415be99c inline format、92e80fc64 s404 object）同 clone
落库——与 css-color 域不相交，语料零回归互证；共享树索引竞态一例（本流
git add 被并发索引操作吞掉一次，重 add 后提交，见 master.md 协作段）。
