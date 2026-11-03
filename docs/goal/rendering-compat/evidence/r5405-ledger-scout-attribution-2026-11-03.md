# R5405 — 账本侦察归因轮（五域定谳全落特性级，corpus 真态 724F 不变，无代码变更）

日期：2026-11-03
性质：归因轮（R5403 同型，0 net code）
范围：控制面 R5404 下一步①侦察候选五域（css-conditional / css-transforms / writing-modes / counter / ruby）+ ②完成标准评估

## 1. css-conditional 14F 全归因（本轮主战场，探针链定谳）

### 1.1 cq 尺寸静态声明推导缺口（定谳，4 案同根）

驱动案：`no-layout-containment-abspos-dynamic.html`（✗2.08%，红可见绿缺失）。

探针链（临时探针 8 个，全部用后即删）：

1. **converter 复核**：R5153 container-type containment 消费点仅尺寸/min_size 臂，abspos
   CB 解析不受 container-type 影响——假设 (b)「abspos 被截留 0 高容器」弱化。
2. **静态/纯突变探针双败**（去 script / 去 offsetTop）——排除动态重估与脚本时序域，
   失败是静态配置本身。
3. **no-query abspos 探针 ✓0.00%**——直接 abspos 在同款 container-type 子树内绘制完好，
   containment/绘制层无罪，假设 (b) 彻底出局。
4. **规则匹配探针双败**（`(width = 100px)` 等式与 `(width >= 100px)` 比较两种形式，
   查询仅设 background）——@container 规则匹配本身失败，与运算符形式无关
   （R4126 等式臂在案且判例案绿）。
5. **DOM 手术三探针全败**（去 height:0 / container-type 改样式表声明 / 去
   position:relative）——这三者都不是触发点。
6. **通过案对照**：multiple-conditions-001（R4126 判例，✓）四个容器全部带**显式内联
   `width: 50/100/175/200px` 声明**；探针容器宽度 auto（布局期自父级拉伸）。
7. **代码定谳**：`container_entry_for`（style-system/src/lib.rs:3833）尺寸轴只做
   **静态声明推导**——`style.width` 非 `LengthValue::Px` → `None` → unknown →
   尺寸条件按规范行为恒 false。auto/stretch/percent 宽度容器全走此路径。
8. **确认探针**：探针容器补显式 `width: 100px` → **✓0.00%**。定谳。

**归因（特性级）**：容器查询尺寸解析是「声明即尺寸」静态推导；真实语义要求
「布局后容器尺寸 → 重跑条件组匹配」两阶段反馈（style↔layout 循环），是渲染管线级
特性缺口，非轻量修复。波及：no-layout-containment-abspos-dynamic、
no-layout-containment-fixedpos-dynamic、flex-basis-with-container-type（100cqh 同缺）、
container-units-sharing-via-rule-node（cq 单位同缺）共 4 案。

### 1.2 其余 10 案归属

- **canvas-as-container-001..004（4 案，@2.08%）**：初判特性级——`:focus-within` ×
  canvas 容器化 × double-rAF 脚本聚焦三特性交叉，未深归因。
- **multicol-inside-container（✗2.08%）**：diff 量纲 = 恰一 100×100 方块 = 绿 200 宽
  未分栏 vs ref 分栏，即 **column-count 分栏未实现域**（multicol 深域）。
- **custom-layout-container-001.https**：custom layout API 未实现域。
- **dialog-backdrop-create（100%）**：dialog/backdrop 域（backdrop-root 族挂账邻域）。
- **inline-size-bfc-floats（7.67%）**：floats × inline-size 容器深交叉（floats-031
  深结构挂账同域邻接）。
- **at-media-dynamic-001**：iframe 嵌套 + 动态 @media 重估，多特性域。

## 2. css-transforms 85F 分布归因（全深域，无轻量候选）

- animation/ 族 18 案：transform interpolation/slerp/keyframes——WAAPI 动画中间值族
  （R5403 已特性级挂账同族）。
- preserve3d-and-* / transform3d-sorting / backface / perspective / 3d-rendering-context
  等 ~50 案：3D 渲染上下文（preserve-3d 扁平化/平面排序/backface）深域。
- .xht 2 案：在册 .xht XML 解析模式族。
- 余 ~15 案零散：transform-table（表格×变换）、transform-fixed-bg（固定背景×变换）、
  transform-box mutation（动态）、transform-iframe 等，各自独立深域。

## 3. css-writing-modes 32F / css-ruby 19F / css-counter-styles 全绿

- **css-writing-modes 32F**：31 案 .xht（在册 XML 解析模式族）+ available-size-022
  （正交流 orthogonal-flow 自动尺寸 × contain:layout size × vh 约束，深域）。
- **css-ruby 19F**：ruby 盒模型域（CSS Ruby Annex：annotation pairing / overhang /
  box generation / inlinize；ZW 有 ruby 元素 inline 特判但无 ruby 对盒模型）。
- **css-counter-styles 240/240 全绿**——从候选清单剔除。

## 4. 全量残差分布（本轮全量跑：725F 单跑窗口，真态 724F ± 在册抖动族；可信 10365）

按目录失败数（本轮归因 150 案标 ✦）：

| 目录 | F | 状态 |
|---|---|---|
| css-multicol | 103 | 深域（R5134–R5139 多轮碎片化工程在案） |
| css-overflow | 64 | 未系统侦察（下轮候选） |
| css-transforms | 85 | ✦ 本轮归族 |
| css-flexbox | 40 | 部分收口（R5401 等），余未侦察（下轮候选） |
| css-backgrounds | 37 | 部分归因，余未侦察（下轮候选） |
| css-writing-modes | 32 | ✦ 本轮归族（31 .xht + 1 深域） |
| css-sizing | 32 | 未系统侦察（下轮候选） |
| css-pseudo | 31 | first-line 行高位已收口 1，::first-letter/::marker 族挂账在案 |
| css-ruby | 19 | ✦ 本轮归因（ruby 盒模型域） |
| css-text | 19 | letter-spacing 域多轮收口，余未侦察 |
| css-conditional | 14 | ✦ 本轮全归因 |
| css-cascade | 15 | ✦ R5403 已归因（@scope/revert-layer/layer×shadow） |
| css-contain | 20 | R5153 邻域，余未侦察 |
| css-display | 13 | 未系统侦察 |
| css-color | 13 | color-6 切片已归 13，另有余量待核 |
| css-ui / css-grid / css-nesting / css-position / css-tables / css-borders / css-fonts / css-content / css-text-decor | 42 | 零散 |

## 5. ②完成标准评估

控制面 Done Criteria（2026-10-17 用户修订）：**DC-2~5 通过率目标 100%（范围内零失败）**。
当前真态 724F（95.6% 单跑窗口）远未达标；「侦察全部落特性级」只表明本轮无轻量候选，
**不触发 DONE**。执行协议不变：特性级归因按协议记档跳过（待用户决策/特性级清单），
续扫其余未侦察域择轻。

## 6. 挂账增量（特性级，待用户决策/特性窗）

新增：①**cq 尺寸静态声明推导缺口**（4 案，两阶段 style↔layout 反馈管线级）
②canvas-as-container ×4（初判三特性交叉，未深归因）③multicol-inside-container
（分栏未实现）④custom-layout-container-001（API 未实现域）⑤dialog-backdrop-create
（dialog/backdrop 域）⑥inline-size-bfc-floats（floats×cq 深交叉）⑦at-media-dynamic-001
（iframe+动态重估多特性）⑧css-ruby 19F（ruby 盒模型域）⑨css-writing-modes
available-size-022（正交流自动尺寸深域）⑩css-transforms 85F 归族（WAAPI 18 + 3D 渲染
上下文 ~50 + .xht 2 + 深零散）。

维持：R5404 及以前挂账全量、对端账全量。
