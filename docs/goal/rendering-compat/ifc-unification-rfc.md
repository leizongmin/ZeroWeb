# IFC 水平推进统一化专项（R109 Phase A——GB-20261007 批复立项）

**状态**: Active（勘察期）
**批复**: 2026-10-07 用户对话渠道批复「做」——对本流待决策清单两项：① R3878 user-gated 的 IFC horizontal advance unification（Phase A/R109）解封立项（本文件）② font-wall 结构修复重议立案（见 master.md 同日批复记录，第一切片 = R1512 证据复审勘察轮，尚未到代码）
**依据**: R3878-I plateau-guard 定谳 + R4114-N 勘察轮全量试验（净 −125pp 证伪单点门开法）
**工作域**: 渲染流（layout-engine R109 匿名片段盒 / inline 流管线）

---

## 1. 使命

把「block 容器内 inline 级内容（文本 / span 等原子 / 替换元素）的水平推进与行内排布」统一到**单一 IFC 行内流路径**：inline 元素序列与直排文本在匿名片段 IFC 内按行盒流动，而非各自成为独立 taffy 盒平铺。目标域收口制，不设固定通过率数字。

## 2. 背景与证据锚点

- **R3878-I**（plateau-guard 定谳）：多 inline orphan bg 丢失域两轮单点修复净负（0/−6、0/−14 全 revert；R2197 backfill height 静默 R2160 part2 信号、0<h≤1.5fs 死区）——coherent fix = IFC horizontal advance unification（Phase A/R109），当时 user-gated 挂账至今。
- **R4114-N**（2026-09-07 勘察轮，0 net code）：`block_container_has_mixed_content` 要求直接非空白文本子才触发 R109 拆分 → **span 包裹混排不拆分**（insert-inline-in-blocks-n-inlines ×9 停 36-40%）；env-gated 试验（has_text |= inline element）全量 16815 XOR **净负 −125pp**（improved 639 < worsened 764）后按纪律回退。定性：单开判定门把 span 全部块化盒子化，方向性证伪。
- **同谱系**：R3800 inline-svg paint、R207 stored IFC 守卫；R4112-F/R4111-N 已在匿名片段 % 高域修通 CB 穿透（本专项的水平轴对应物）。
- **教训在案**：insert-block-in-blocks 族的「全绿」经试验揭穿为 test/ref 双页同平铺互恰假绿（R3870 教训）——统一化落地会先揭假绿再翻红，A/B 判读须按 ID 级 XOR 全量口径，不接受「族内全绿」表象。

## 3. 目标形态

span/inline 元素加入行盒与文本**同行流动**（行内流排布），行盒几何由统一 IFC 推进逻辑产出；`block_container_has_mixed_content` 的触发面扩为「inline 内容 = 直排文本 ∪ inline-level 元素子」，扩入的子走同一行内流管线而非独立盒。

## 4. 已知驱动簇（收益面）与在册保护（回归面）

**收益面**（R4114-N 试验实测）：
- insert-inline-in-blocks-n-inlines ×9：36-40% → 7.5（结构翻正）
- svg-feimage ×2 / translate-attribute-in-svg / backdrop-filter-image-size 等试验翻绿面
- 多 inline orphan bg 丢失域（R3878 死区族）
- CSS2 ≥4% 157 案重聚类余账中的混排域

**在册保护**（R4114-N 回归面，切片验收的必查清单）：
- insert-block-in-blocks-n-inlines ×9（试验中 0→14-17，假绿揭穿后真实语义面）
- inline-replaced-width-008/009（0→21.9/12.5）
- contain-size-multicol-002（7.2→34.7）、border-bottom-width-061（0→11）
- R207 stored IFC 守卫案、R3800 inline-svg paint 面

## 5. 切片纪律

- **切片 0（勘察，0 net code）**：R109 拆分触发面普查（现有 `has_mixed_content` 全调用点）+ 行内流排布目标形态定义（对照 chrome dump 的行盒/原子几何），产出落 evidence/。
- 每步 = env-gated default-off 试验 → 全量 ID 级 XOR A/B → 净收益 > 0 才 default-on；净 0/净负按净值纪律回退，不挂账硬扛。
- 门禁：make test 全绿、make reftest + make reftest-upstream 零回归、product-smoke 逐字节恒值、legacy struct PASS、§4 在册保护案逐案核对。
- **与 layout-perf-pullback 专项协调**：本专项动布局热路径，bench A/B 度量窗错峰；落地的每切片做 bench-gate 定向复核（block/flex/incremental 不劣化）。
- 工作面：本流（layout-engine），按 run-rules §9 与兄弟流不重叠承诺。

## 6. 进度记录

- 2026-10-07：立项（GB-20261007 批复「做」）。下一步：渲染流下一轮 rally 轮从切片 0（触发面普查 + 目标形态定义）开始，勘察产出经 A/B 复核后定第一刀。
