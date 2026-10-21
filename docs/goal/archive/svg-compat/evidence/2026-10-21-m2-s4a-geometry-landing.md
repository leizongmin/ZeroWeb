# M2 S4a SVGGeometryElement 几何面——getTotalLength/getPointAtLength + 路径解析 + getNumberOfChars（2026-10-21）

S3 收口后接续，同 session 端到端持有（工作区仅含 S4a 改动基础上起步）。单文件
落地：part03.js（路径解析/弧长取样/形状 mixin + getNumberOfChars）。

## 实现

### 路径数字文法 + 解析器（spec SVG2 paths.html §PathDataBNF/§PathDataErrorHandling）

- **数字文法**：符号/小数点前后数字可省（'5.5.5' → 5.5,.5、'-10-5'、'.5'、
  '1e2'）；**尾随小数点非法**（'23.' → 错误截断——WPT
  number-error-trailing-decimal 五断言定谳，反 SVG2 BNF 字面的
  `digit-sequence "."` 分支，以断言面为准）。
- **命令面**：M/L/H/V/C/S/Q/T/A/Z 全命令 + 参数组隐式重复（m 后续对 = 隐式
  相对 lineto）；错误前缀语义——非法命令字母/参数缺数保留已完整前缀，参数组
  内不完整尾组丢弃、完整组保留，不完整组首命令整命令跳过；arc 旗标严格单字符
  [01]（'2' → 该组丢弃）。

### 弧长取样（getTotalLength/getPointAtLength 靶面数学）

- 折线化机制：直线精确；C 64 细分/Q 48 细分弦长逼近（tolerance 0.1 内）；
  A 端点→中心参数化（spec appendix）+ 半径缩放校正 + 角度 ≤π/32 细分
  （sweep=1 正角方向——屏幕顺时针，半圆案 y=−50 实证）；T/S 平滑控制点
  反射，无前序 Q/C 时控制点 = 当前点（退化直线）。
- **多子路径**：moveto 锚点入折线但不与前点连线计长（spec：moveto 无长度；
  getPointAtLength 距离跨子路径连续，gap 不消耗；等 cum 值二分自然落锚点
  后段）——首轮探针暴露的 cum/pts 错位 + gap 计长两 bug 同点修复。
- getPointAtLength：dist float 校验（非有限 TypeError）→ 截断 [0,total] →
  累积弦长二分 + 线性插值；无有效路径数据 → (0,0)。

### 形状 mixin + getNumberOfChars

- SVGGeometryElement 六形状：rect 2(w+h)（零/负维度 → 0）、circle 2πr、
  ellipse Ramanujan 近似、line 勾股、polygon 闭合环、polyline 开环；points
  解析复用路径数字文法，奇数尾坐标截断。统一 ring→cum 机制供
  getTotalLength/getPointAtLength。
- SVGTextContentElement.getNumberOfChars：textContent UTF-16 长度（文本几何
  族唯一无布局依赖成员）。

## 验证

- 机制探针 6/6 Pass（gitignored wpt-data/svg/s4a-probe.html；三轮修正——
  cum/pts 错位 + gap 计长两实现 bug、探针自身两处算术错、arc sweep=1 方向
  探针反错（实现正确：屏幕顺时针 → 上拱 y=−50））：方环全长/取样/闭合/截断、
  六形状 mixin、错误前缀四族、arc 半圆、数字文法边界、getNumberOfChars、
  float 校验、空 d、动态元素。
- **path/parsing 11 案 126 子测试全绿**（基线 0P/126F）：moveto absolute/
  relative 全表（含多子路径/链式相对 m）、lineto 13、curveto 11、quadratic
  14、arc 12、whitespace 14、number 三案 19、error-handling 12。
- **全 corpus（486 案，探针剔除）：858P/804F/40T/0NR = 50.4%**（含 NR 分母
  口径）——S3（734P/928F/40T/0NR = 43.1%）以来 **+124P/-124F**，Timeout 40
  持平，全绿案 110→125（+15），逐案零 DOWN，分母稳定。**通过率跨过半数线**。
- S4a 靶面实收：path/parsing +116P、types getTotalLength 2 案 +2P（-02
  rect/circle 全绿；-01 d 属性子绿、2 CSS-d 子挂账）、getnumberofchars +4P、
  historical +2P。
- 全 workspace `make test` DC-4 门禁：EXIT=0（结论见 master.md 轮次记录）。

## 卡点与划界

- **CSS d property（getTotalLength-01 path2/path3 两子 + style-flush 4F +
  path-length-interpolation 1F）**：引擎 CSS 未实现 d property 与 computed
  style 暴露——CSS 域缺口非 SVG 文档面单侧可落（FIXME(svg-compat)：CSS d
  落地后 getTotalLength 接 computed style 回落）。
- setPathData/getPathData（path/interfaces 16 案 54F）：S4c 评估项（path data
  解析器本切片已备）。
- 文本几何余族（getComputedTextLength/getStartPositionOfChar/
  getRotationOfChar 等）：字形布局依赖——按 goal 划界 rendering-compat 协同，
  M4 定稿归口。
