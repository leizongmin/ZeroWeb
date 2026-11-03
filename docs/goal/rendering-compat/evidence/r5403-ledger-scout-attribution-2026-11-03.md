# R5403：账本续扫归因轮——203 / backdrop-root / color-6 三向侦察（2026-11-03）

范围：R5402 落账后按下一步清单三向侦察。全部归因落**特性级/族级缺口**，按协议记档
跳过，无代码变更（corpus 真态 725F 不变）。证据采集：案面 + 控制盒推导 + REFTEST_DUMP
PNG 三件套 + chromium oracle 探针（/tmp 一次性探针，用后即删）。

## ① letter-spacing-203（9.94%）——closest mutual ancestor ls 归属模型

案面 assert："Letter spacing between two characters is given by their closest mutual
ancestor"。控制盒唯一吻合推导：盒 7（A|B 共同祖先 = 无 ls 外层 span → 0，"AAAB" 连排）
+ 盒 8（共同祖先 span.ls3 → A|B 三隙）——非左右值平均、非左元素值。

ZW 现状：run 按 inline 元素边界划分（collect_items.rs:494-534），`letter_spacing` =
所属元素 computed 值，R4231 advance 含词尾 ls → 跨 run 边界间距 = **左 run 的 ls**。
10 对盒中 8 盒依赖 mutual-ancestor 模型，无法子集化（盒 9/10 空格边界同域）。

修复面：TextRun 边界归属字段 + collect_items 父链 mutual-ancestor 计算 +
break_lines 推进/fit 门/R5402 原谅项联动 + paint fragment 推进同步（#3 度量单一
owner）+ balance_breaks 消费点——**特性级**，牵动 letter-spacing 全家族绿案基线。

## ② backdrop-root 深评——FilterPrimitive 取样基线缺口（特性级）

四案红：backdrop-root-{filter 0.96/mask 1.48/mix-blend-mode 1.48/opacity 1.00}。
案面语义（filter-effects-2 #BackdropRoot）：opacity<1 / filter / mask / mix-blend-mode
/ will-change 指向这些的元素建立 backdrop root，backdrop-filter 取样止于最近 root。
opacity 案 PNG 形态：右侧半盒被 invert 成红（ZW 取样穿透到页面底）vs 应显青。

ZW 现状：`apply_backdrop_filter`（effects.rs:1898）只投递 FilterPrimitive（rect +
filters）；CPU 光栅（render-foundation cpu/effects.rs `apply_filter`）对主 FB 的 rect
**就地取样**——无基线/层边界概念。isolate 机制（svg_filter_chain）仅覆盖 filter
url() 的 SourceGraphic 隔离，非 backdrop root。

修复面：图元模型加基线快照图元（收集阶段插 BaselineMark + 光栅阶段基线判定）+
CPU/GPU 双路径同步（#6 共享等价断言）+ paint 热路径快照内存（#11）——**特性级**。

## ③ color-6 切片侦察——红案归因四个族级缺口

css-color 红案 13 案全归因，无轻量修复点：

| 缺口族 | 案 | 证据 |
|---|---|---|
| **.xht XML 解析模式** | t425-hsla-basic-a 8.92% | PNG：ZW 渲染 head 内 `/*…*/` 注释文本（HTML5 in-head 非空白字符挪 body 规范行为）；chromium oracle 探针（file:// .xht → application/xhtml+xml → XML 解析）页面干净 head 隐藏。hsla 数学与 alpha 合成本身正确（手算 (102,255,102) 两 div 同色）。runner 对 .xht 无 XML 分支（wpt_file_loader.rs 同 .html 处理） |
| **WAAPI 动画中间值** | color-mix-basic-001 19.41% | PNG：ZW test 页色带自洽渲染（color-mix 计算在案），**ref 页全白**——ref 用 `element.animate({backgroundColor})` + pause + currentTime 取动画中间态色（CSS Color 5：动画插值 ≡ color-mix），ZW 无 WAAPI，脚本在 animate 处抛错中断 |
| **opacity 原子合成** | opacity-overlapping-letters 3.75%；同域 color-layers-no-blend-mode 40.10%、filters-under-will-change-opacity 4.69% | PNG：test "XXXXX" 交替深条（per-glyph alpha 叠 1−0.5ⁿ）、ref "X X X" 近似正确带 → 双页互异。根因铁证：`apply_opacity_to_new_primitives`（paint/clip.rs:783）= per-primitive alpha 衰减（`glyph.color.a ×= opacity`），无重叠时与原子层数学等价，重叠内容分叉。修复 = 图元模型层合成（LayerPush/Pop）+ CPU/GPU 双路径——**特性级** |
| **复合特性域** | contrast-color-style-query 100%（contrast-color + style query）、contrast-color-interpolation 2.08%（动画插值）、currentcolor-003 4.28%（::first-line × box-decoration-break:clone × 七属性 currentcolor）、tagged/untagged-images ×4 + at-color-profile（ICC 色彩管理） | 案面复合特性叠加，各自挂账 |

## 结论

三向侦察全部落特性级/族级缺口，与 floats-031 / R5397 两案 / R5213 同级记档。下轮
账本续扫应转向未侦察域（避开本轮四面缺口与既有深结构挂账）。
