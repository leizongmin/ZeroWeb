# R5469 证据档：c5525-fltmrgn-000 归因定谳——fit 门断行新行首词下绕臂落地（floats-132 真修复 +1）＋ test4/test6 残余双缺陷定域

日期：2026-11-08。R5468 下一步①（c5525-fltmrgn-000 ✗3.92% 归因探针）。

## 0. 轮次结论

- **修复落地**：`break_items_into_lines` fit 门断行 fall-through（开新行）处新增「新带放不下本词 → `shift_new_line_below_floats` 下绕臂」（break_lines.rs ~line 875）。R5240 的行首词下绕只在 loop-head 空行臂生效；fit 门断行开出的新行不经 loop-head 重评，行首词原地强放缩短带溢出。本臂补齐姊妹入口，与 loop-head 共用 kill-switch `ZW_FIRST_WORD_FLOAT_FIT=0`。
- **corpus A/B（同二进制 16594）**：ON 690F / OFF 691F。batch 差集 5 案 solo ×3 全 ✓0.00%（insert-block-in-blocks-n-inlines-end-001、table-anonymous-objects-129/149/151 = 噪声家族第五批 movers；tao-129/149 OFF solo 亦 ✓ 双向排除）；**floats-132 真修复**（OFF solo ✗1.33% ×3 恒 → ON solo ✓0.00% ×3 恒，R5240 驱动案全关——残差正是本臂域的 post-break 入口）；c5525 ✗3.92%→**✗2.90%** deterministic（双侧 solo 恒定）。
- **门禁**：内测 725 全绿（Layout 521 + Text 204）；judgment test `r5469_postbreak_word_float_fit_probe` 2 案（c5525 test3 形态下绕锚 + 词放得下不下绕守卫）；clippy/fmt 清。

## 1. 归因方法与五形态矩阵

chromium oracle（`--headless=new --screenshot` per-row ink scan）定谳 test3 形态模型：CSS2 §9.5 rule 8「shortened line box too small to contain any further content → shifted downward until it either fits or there are no more floats present」——float（mt30/ml30，带 margin-box (0,0,75,75)）右带每行仅 1 字位；词 "xx"（2 字位）放不下 → **行位下绕 float 带底 y=75 满宽整行**（"xx x x"），行 4-5 为空带。p 高 = 3×15 + 30（空带）+ 2×15 = 105。

chromium per-row 实测（p1 f3，rel_y 像素行）：`0-44 右带 [75,89] 墨`、`45-74 全空`、`75-89 [0-29]+[45-59]+[75-89]`、`90-104 [0-14]` —— 与模型逐位吻合。

五形态矩阵（ZW probe vs chromium oracle，修复前 → 修复后）：

| 形态 | img margin | 带 (margin-box) | chromium | ZW 修复前 | ZW 修复后 |
|---|---|---|---|---|---|
| T0 | 无 | (0,0,45,45) | 60 | 60 ✓ | 60 ✓ |
| TML | ml30 | (0,0,75,45) | 75 | 75 ✓ | 75 ✓ |
| TMT | mt30 | (0,0,45,75) | 75 | 75 ✓ | 75 ✓ |
| TMN | ml−30 | (0,0,15,45) | 45 | 45 ✓ | 45 ✓ |
| T3 | mt30+ml30 | (0,0,75,75) | 105 | **90 ✗** | **105 ✓** |

只有 T3 触发「词宽 > 带宽」(2 字位词 vs 1 字位带)——下绕臂唯一敏感域，与 corpus 判定吻合（0 真回归）。

机制旁证：T3 修复前 ZW 行结构 `[1,1,1,1,1,2] runs`（90px）——"xx" 强放在 y=45 行的 15px 带溢出、词 6/7 挤到带底后首行；下绕臂落位后 `[1,1,1,3,1]`（75+15=105px，下绕行满宽 3 词）。

## 2. 残余 ✗2.90% 定域（下一步挂账）

> **R5470 勘误（2026-11-09）**：本节 test4/blob3 的「bg-image 原点左移」签名**不成立**——彼时 diff 带分析未剥离布局位移贡献。R5470 定谳 c5525 主体残差 = 布局域双缺陷（taffy 对含 float 子容器 collapse-through + 步 6 高度物化不传播，p3-p6 累计上移 15/60/75/90px），修复后 c5525 ✗2.90%→**✓0.28%**（见 [r5470 证据档](r5470-float-collapse-through-sib-reanchor-2026-11-09.md)）。test6 墨 z 序结论保留为 0.28% 残差域；test4/blob3 发射源重开归因时须先在布局修复后的基线上重取 diff 带。

c5525 = 6 子测试纵排。修复后 diff 带（ZW vs REF 与 ZW vs chromium oracle 双口径一致）全部落在 test4/test6 区，**test1/2/3/5 已全关**：

1. **test6 绘制序缺陷（定谳，挂账深域）**：mr−30/mb−30 → img border box 与 3 行文本重叠。ZW 行几何**逐位正确**（TEMP IFC dump：L1 runs x=15/45/75、L2 x=0w30/45/75、L3 x=0w15，与 pattern-6 要求的墨位一致），但 img `pattern-grg-rrg-rgg.png` 的红格在墨位**露出**（ZW 像素 R vs chromium 全绿）→ **文字墨绘于 float 内容之下**。CSS2.1 App E：行内级内容（文字墨）应绘于非定位 float 之上；chromium 的墨覆盖 img 红格正是全绿的关键。swatch-green img（test2/3/5）全绿，历来无案暴露此序。域 = engine/render-foundation 图元 z 序（float vs 行内墨），非行断域。
2. **test4 背景图元绘制域异常（像素签名定谳，域 = paint 管线，行断域已排除）**：p.test4（margin 3em 1em 1em 3em，content 60×60 @ 页 (56,293)）。隔离子集探针（width 60 + img mt/ml −30）**h=60 4 行正确**，且 diff 单元格局与其多种 ink/带假设均不自洽——L1 [15,30) 露红可由 pattern-4 绘制原点左移签名解释（pattern-4 60×60 的 G 格 [30,45) 在 ZW 露红＝该处必须被 pattern 绿像素直接覆盖，任何 ink 假设都救不了），即 **bg-image 原点计算或 float 相关 bg 细分异常**，非行断/带问题。另见条 3 红矩形。**勘误链**：先误判 block 位移（实为 margin 3em 正确值 x=56/w=60）→ 再误判 L1 带未生效（与 pattern 原点异常单元格不自洽）——单元格推理多假设耦合不可靠，以源头归因为准。
3. **blob3＝test4 左 margin 区红矩形（页 x[26,56) y[323,352]，30×30）**：该区应只由 div bg（pattern-0.png 120×555 全绿）承载，ZW 露红、chromium 全绿；x 恰止于 p content 左缘 56、y 恰为 p-rel [30,60)——与 img（p-rel (−30,−30)-(15,15)，页 [26,71)×[263,308]）不相交。发射源待源头归因（候选：float 相关 bg 细分/裁剪矩形错位、p bg-color 泄漏进 margin 带）。

test4/blob3 与 test6 分域：test4 两异常仅现于「负 margin-left float + p 大 margin-left」组合子测试（test5/6 的 pattern 原点无异常），归因时先做最小页二分（去 img / 去 p ml / 去 mt−30）。

pattern PNG 语义备查：p bg = red + pattern PNG（红=墨须覆盖、绿=须空）；img（swatch-green / pattern-grg）为真绘内容且**文字墨须绘其上**方得全绿（REF 页 = 纯绿 div，test 页须逐位复现）。

## 3. 验证矩阵

- corpus ON 690F（Duration 73.1s）/ OFF 691F（58.4s），差集 movers solo ×3 全 ✓（§0）。
- floats-132 solo：OFF ×3 恒 ✗1.33%、ON ×3 恒 ✓0.00%（定谳真修复）；c5525 solo ON ×3 恒 ✗2.90%、OFF ✗3.92%。
- 内测 `make reftest` 725/725 全绿；judgment test 2/2 ✓；clippy --all-targets 清、fmt 清。
- `make test` 全量（test-guard 包裹）：在 zero-engine lib 套件中止（make 错误 101，字母序 zero-engine 先于 zero-layout-engine）——中止前合计 **9329P/7F**，唯一 FAILED 套件 = zero-engine lib（3816P/7F），7F 全落 `js_dom_bridge`（img/svg settle、fetch filtered matrix、iframe SW dispatch、readable stream、websocket ×2）；`git stash` 干净树复跑同 7 案同红（filtered 口径 1383P/7F）→ **pre-existing 实锤**（控制面在册「对端 js_dom_bridge 红腿观察」，非本轮引入）。layout-engine 全腿由定向 guarded 运行单独覆盖（下条）。
- layout-engine 全腿（test-guard compile-first，42 套件）：lib **2446P/0F/2I**（含本轮新增 judgment test 2 案）＋ 集成二进制/doc-tests **101P/0F**，合计 **2547P/0F/2I** 全绿。
- `make product-smoke` 全腿 PASS：welcome vs chromium oracle **17.23%**（与 R5465/R5466/R5467 同值零漂移）＋ 6 站 struct-check 全 PASS ＋ form-input-perf-gate PASS。
- bench-gate 定向 zero-layout-engine **GATE PASS 25 指标（NEW=0）**。

## 4. 下一步（R5470 候选）

1. c5525 残余双缺陷（均 paint 域）：test4/blob3 先做最小页二分（去 img / 去 p ml / 去 mt−30）定位 bg-image 原点偏移与 margin 带红矩形发射源（可能轻量，源码在 engine/render-foundation bg 绘制路径）；test6 绘制序（文字墨 vs float 内容 z 序，深域，开案前先查图元发射序）。
2. fsa 度量单 owner 切片（R5468 遗留，证据完备可直接开案）。
3. 缺陷 3（UA p margin 逻辑化）维持待用户决策⑥。
