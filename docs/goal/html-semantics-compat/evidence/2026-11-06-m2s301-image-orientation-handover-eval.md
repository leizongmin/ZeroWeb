# M2 s301 — 评估销案：css image-orientation 属性渲染面移交 rendering-compat 成立（承 s296/s299/s300 候选；纯评估零源码改动 docs-only）

日期：2026-11-06。切片号 s301（承 s300 下一步候选①）。评估结论：**移交
成立**——css `image-orientation` 属性语义面（解析/计算值/none 渲染路径/
运行时变更重绘）全部落 rendering-compat 域，html-semantics 域内零剩余
钉形；本域 IDL 语义面（naturalWidth 解耦）s296 已收口。纯评估零源码
改动。

## 勘根

**语料盘点**（css/css-images/image-orientation/ 共 35 页）：reftest
通道在册 32 页（b2 全树日志逐页态），**24P/8F**；不在册 3 页——
computed-style×2（testharness 通道页，`getComputedStyle().image-orientation`
断言）+ cursor（`rel=flags interact` 手动交互页，本通道不可自动化）。

**红面单根归因**：8 红页全部同根——css `image-orientation` 属性未解析
+ 渲染路径无条件消费解码已定向位图，凡 `none` 语义（应按存储像素原样
渲染）必红：

| 页面 | solo 差 | 根 |
|---|---|---|
| none.html | 10.94% | none 臂 |
| none-content-images.html | 13.02% | none 臂 |
| none-cross-origin.html | 1.24% | none 臂 |
| none-cross-origin-border-image.sub.html | 1.04% | none 臂 |
| none-cross-origin-svg.html | 3.33% | none 臂 |
| background-image.html | 1.04%（solo 三连确定性，非 flake） | 第二 div 挂 `image-orientation:none`；fuzzy 0-3;0-60 容差未达 |
| from-image-dynamic2.html | 10.94% | 运行时属性变更重绘面 |
| from-image-composited-dynamic2.html | 10.94% | 同上（composited 变体） |

solo 三连仅对 background-image 执行（3×3 恒 ✗1.04% 定谳确定性红，非
s300 flake 池时序形）；none/dynamic 各页差幅远超阈值无需定谳。

**引擎支持面为零**：`grep image-orientation` 于 css-parser/style-system/
engine 全空——属性未入解析器，computed style 恒空；from-image 默认页
24 绿全靠 s296/s300 解码层物理重排（默认行为 ≡ from-image 免费成立），
none/运行时面在现架构下不可达。**移交根因注记**（rendering 流根修
决策点）：解码侧物理重排把方向烘进位图——`none` 需要「未定向位图变体
解码」或「绘制期方向变换」二选一的渲染路径改造，属 render-foundation/
paint 架构面；property 解析与计算值落 css-parser/style-system；运行时
变更重绘失效落 layout/paint 失效面。IDL 侧无波及：naturalWidth/Height
读解码后 ImageData 尺寸，`none` 下仍反映 EXIF 方向（html spec 解耦语义，
s296 契约 3P 已钉），属性解析落地后无需改 IDL 面。

## 销案与移交清单（rendering-compat 域候选输入）

1. reftest 通道 8 红页（上表；单根 `none`×6 + 动态重绘×2）。
2. testharness 通道 computed-style×2 页（css-images 域条目，属性解析
   后自然收编）。
3. cursor 手动页不进自动化面（record only）。
4. 本域后续：语料钉形新增（如 2-value legacy 语法页）再评估，当前
   35 页无 legacy 2-value 面。

## 下一步候选（本域）

1. **the-dialog-element reftest 26 红页簇勘根**——html/semantics
   reftest 通道 117 红页（b2 全树日志聚类：select 41 / **dialog 26** /
   bdi 13 / img 8 / ol 6 / 零星）中本域自有最大簇；dialog open/close/
   ::backdrop 语义在 shim（本域），UA 样式与 top-layer 渲染归属需勘根
   划界（若纯渲染面则同例移交 recording）。
2. select 41 红页簇归属勘根（shim 语义 vs 表单渲染面划界，涉
   rendering 流 form-visual 域碰头面）。
3. bdi 13 页簇（bidi 隔离渲染面，疑渲染域移交候选）。

## 证据

- 数据面：s300-reftest-full-b2.log（在册页态+域分布）、语料目录
  ls/页头核对（computed-style×2 testharness 形、cursor interact 形）、
  engine 三 crate grep 零命中、background-image solo 三连（本轮前台
  跑批未落盘，3×✗1.04% 摘要在本账）。
- 承前账本：s296（解码单漏斗+naturalWidth 契约）/s299/s300（PNG 面
  收口+flake 池协议）。
