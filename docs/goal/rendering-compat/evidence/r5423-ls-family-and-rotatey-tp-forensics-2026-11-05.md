# R5423 — letter-spacing 族弃案归因 + rotateY/3D 函数簇深域定谳（TP 区域清白毁背衬）

日期：2026-11-05。上一轮：R5422（backface 3D 渲染上下文矩阵合成，914d6e00b）。
本轮为**归因轮**（零代码改动）：misc 账本续扫两族候选全落弃案/深域，取证链完整落账。

## 基线

本轮 w0 全量窗 fail 名单 **711F**（R5422 终态 708F，3 案在册抖动族漂移带内）。
letter-spacing 族红案全名单 = letter-spacing-203/206/ligatures-005（005 见 R5418④）；
3D 函数簇红案 8 案 = rotateY.html、transformed-rotateY-1、
transform-3d-rotateY-stair-above/below-001、transform3d-matrix3d-003/004/005、
transforms-rotateY-degree-60。

## ① letter-spacing 族弃案归因（三联合机制）

- **005（Arabic joining 打断）**：联合要求两臂——(a) ZW shaping 门
  `text.rs:3538-3544`：`letter_spacing == 0.0 && frag_letter_spacing == 0.0 &&
  word_spacing == 0.0` 才走 shaped 路径，任一非零整体关 shaping → Arabic 逐字符
  isolated 形（连写形式丢失）；(b) css-text-3 §7.2.1 cursive 豁免缺失：不能在不
  断连写的前提下展开 cursive 文本的 UA **不得**在同 script 字母对之间施加
  letter-spacing（Arabic = cursive）。ZW 逐字符路径对同 script 对照加 gap。
  cursive-001/002 的 ✓ 为容差宽纵伪绿（严格容差下同样会红）。
- **203（closest-mutual-ancestor）**：css-text-3 letter-spacing 2024+ 模型——两字
  符间间距由**最近公共祖先**元素的 ls 决定：`<p class="ls1"><span class="ls0">A</span><span class="ls0">B</span></p>`
  边界间距取 p 的 ls1（span 自身 ls0 不跨界）。ZW 按字符所属 fragment 逐个施 ls，
  无收集期边界归属计算 + 行装配 + paint 协调的联合改造不可能表达。
- **206（intrinsic sizing with ls）**：固有尺寸测量需计入 ls，属 min-content
  测量模型面。
- **弃案理由**：三案任一修复都是 core inline/收集模型改造，爆炸半径覆盖
  letter-spacing/bidi/text 全语料族，非轻量。修复方向（三联合取舍）记待用户决策。

## ② rotateY/3D 函数簇深域定谳（机制链完整）

取证链（全部实证，探针用后即删）：

1. **REFTEST_DEBUG 图元清单**：corpus 原案 test 侧发射 green(100,100,100,100) +
   white(100,100,100,100) **未前缩**两 fill；ref 侧为手绘 25px 条正确。
2. **直连 Painter 单测探针**：v5/v10 两配置图元层一致——双 fill 全尺寸 +
   **TP 正确发射**：`rect=(0,0,100,100) origin=(50,50) m=(a=0.5, b=0, c=0, d=1,
   tx=25, ty=0)` = rotateY(60°) 精确中心投影（scaleX(0.5)，左 0→25、右 100→75）。
3. **布局/样式层排除**：v5（全类规则）vs v10（元素规则偏移）ComputedStyle 与
   LayoutBox Debug 逐字节一致——分歧不在 style/layout 层。
4. **像素定谳（PNG bbox 量取）**：v10 dump 红块 bbox=(25,0,74,99) 精确 50×100
   条 = `apply_transform_post`（cpu/mod.rs:1493）**形变本身正确执行**；
   GREEN bbox=None = 背衬绿盒被**销毁**。
5. **机制闭环**：TP 后处理语义 = 区域快照 → **整区清白** → 逆采样。TP 区域 =
   元素 rect，但 rect 内含 z 下方背衬（绿盒）→ 清白连带销毁背衬 → 逆采样后区域
   除 50px 条外全白 → 白页 vs ref「绿条+白条+绿条」差 5000px = 1.04%（与 corpus
   差值精确吻合，white-on-white 同理）。
6. **det≈0 分支**（rotateY.html / transformed-rotateY-1 的 90° 消失案）：
   `apply_transform_post` det<1e-10 在清白**之前**早退 → 变换静默丢弃、元素全尺寸
   渲染——「正面消失」语义错误（正确 = 零宽不可见）。

### 根因层级与轻修路径排查（为何不轻修）

根因 = **TP 后处理模型把「元素自身渲染」与「区域合成物」混同**。正确语义需元素
级隔离（元素自身图元离屏 tile → 形变 → 合成回背衬之上）= render-foundation
`TransformPrimitive` 契约扩展（own-primitives 快照字段）+ CPU/GPU 双光栅路径 +
ops/serialization 联动 = 跨 crate 架构改动，与「3D 渲染上下文 ~50」深域同族。

轻修备选逐一排除：

- **直绘 quad 扩展到 RotateY/Y 轴 rotate3d**：`axis_exact_3d`（painter/mod.rs:5145）
  现仅 RotateX/X 轴 rotate3d；RotateY 被 R5173 注释**刻意排除**——
  backdrop-filter-3d-transform-perspective 配对 test 侧 .card 被
  `backdrop_filter.is_empty()` 门锁 TP、ref 侧同几何实心盒直绘 → AA/边缘相位
  拆对面（首轮 2 案 P2F）。加 dirty-region 门（背衬在区域内才转直绘）**无效**：
  该配对双侧均为 card-on-backdrop 脏区域，门会让 ref 直绘而 test 仍锁 TP → 拆对
  复发。
- **det≈0 消失案轻量子集**（rotateY(90) 直绘零宽 quad，无 AA 面）：corpus 有
  9 文件含 rotateY(90)/rotate3d-Y-90，其中 perspective-transforms-equivalence
  **test+ref 双侧**同用（现双侧同错自洽伪绿，单侧改语义即 P2F）+ animation 冻结
  案（t=0 语义）。净收益 +2F、P2F 风险 0..3 案未量化，不值得单独开 A/B。
- **matrix3d-003/004/005**：透视分量（m14/m24=-0.005、scaleZ=2）非仿射，直绘
  quad 不可表，归 R5174 透视折叠/平面排序深域。

## ③ guidelines #33 实证与技法（本轮副产物）

早期探针多对 0.00% 通过全部为 blank-vs-blank 伪绿（第二面「0-fill 丢盒」实为
探针页漏写 `<div>` 标记的空文档正确行为——三臂注释实验后 force-dump 揭穿）。
技法沉淀：`docs/learnings/patterns/2026-11/2026-11-05-reftest-passing-page-primitive-forensics.md`
（REFTEST_DEBUG 图元清单 / black-ref 强拆 / 严格 fuzzy meta / 直连单测补印
transforms / PNG bbox 量取）。

## ④ 清理记录

39 个 zz* 探针页/参考页删除（css-transforms 目录 grep zz = 0）；两个临时诊断
测试（r5423_rotatey_regress_tests.rs / r5423_rotatey_paint_probe.rs）及 mod.rs
注册还原；target/reftest-dump/*.png 清理。零代码改动 → corpus 无 A/B 义务。
