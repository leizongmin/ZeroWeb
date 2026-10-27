# R5299 证据：单片段整盒 slice 旗标 2×2 全量仲裁 + 行裂解段表 + selection 带域 once 绘制

**日期**: 2026-10-27
**轮次**: R5299
**驱动案**: R5120/R5121 谱系 kill-switch `ZW_INLINE_SINGLE_FRAG_BOX_BORDER` 转正评估（R5297 挂账②单片段/orphan 转正独立 sweep）
**语料**: 16591 案，旗标全关基线 = R5298 交付态 15733P/**858F**；**旗标维持默认 OFF，corpus 交付态不变**

## 2×2 旗标全量矩阵（失败集归一化 diff）

| 组合 | 新红 | 翻绿 | 定谳 |
|---|---|---|---|
| `ZW_PARA_BIDI=1` 单开 | 0 | 0 | A/B 干净，随时可独立启用 |
| `ZW_INLINE_SINGLE_FRAG_BOX_BORDER=1` 单开 | 9 | 3 | 9 = 8 真新红 + content-none-select-1（并行 runner flake，隔离复跑 ✓0.00） |
| 双开（combo） | 8 | 4 | 真实翻转面 = single_frag 本体；bidi-004 仅 combo 翻绿（para_bidi 叠加效应，阈带页） |

- **翻绿（4）**：border-width-applies-to-008（基线 ✗15.53 → ✓0.00）、line-height-125（✓0.22）、hanging-punctuation-inline-bound-001（基线 ✗10.27 → ✓）、bidi-004（combo）。
- **真新红（8）**：bidi-001/003/005/006/010a、active-selection-012、selection-contenteditable-011、backdrop-filter-inline-positioning（前 7 本轮修/定性，见下；末者 = 翻转阻塞②）。

## 伪通过发现（本轮核心定性）

5 个 bidi 页（bidi-text/ 001/003/010a + CSS2 根 005/006）在基线是**伪通过**：引擎文本布局与 chromium 已分叉（RLO 不跨 span 边界传播、无跨 item L2 重排、字形级原位反转），判别差 ~0.71% 藏在 1.00% reftest 阈值之下；旗标开的 border 像素把判别推到 1.01–1.37% 才显红。LAYOUT_DUMP 实证 bidi-001 的 span 是**单个合并 fragment（184px，字形级原位重排）**——paint 侧段表切分路径在当前布局域下处于休眠/防御态，真实修复在布局域 bidi item 裂解（R5121 FIXME 谱系「per-fragment 完整 level 栈（parity 近似收口）」）。

> 取证教训：product-smoke 渲染管线的 bidi 输出与 reftest 判定管线**不一致**（旗标开 PNG 字序明显错误而 reftest 判 ✓0.71%）——bidi 域 PNG 证据以 reftest 直跑（逐旗标隔离）为权威，product-smoke 不可代表。

## 修复（旗标开路径，text.rs）

1. **selection 带域 + once 绘制**（active-selection-012 ✗→✓0.00、selection-contenteditable-011 ✓0.00）：①选区 fill 从逐词域改 owner 行带域（`r4332_whole_box && !multi_line` 时 x 域 = band ± pad），逐词域在词缝露元素底色（ref 全带绿 vs test 词缝红）；②**once-per-(owner,line,segment) 外层守卫**（`r5299_band_drawn` HashSet）——chromium 层序是全 bg 层→全字形层，宏按片段交错 bg→glyphs，逐片段带矩形会盖住先前片段字形（selection-012 修 x 域后反而 1.96%→2.86% 的 PNG 取证定谳），带矩形每段只画一次。
2. **行裂解段表** `owner_line_segment_extent_map`（css-writing-modes-3 #bidi-box-model slice 语义）：行连续域在「盒裂解缝」处分段——缝内楔入**非子孙** owner 的片段即切段；缝内仅子孙片段（子 inline 穴）或空缝（词 run 间隙）保持连续；段附 `(has_first, has_last)` 逻辑首/末标记（边归属按逻辑序非 x 序），lead/trail 外延、垂直边、bg 端头全部改读段标记，替代裸 `is_first/is_last`。单行以外（multi_line）回退原值。
3. **backdrop-filter 门**：`backdrop_filter` 非空的 owner 保留基线口径——effects.rs apply_backdrop_filter（R4296/R4297 行盒+外延口径）滤区取**盒级 rect**，与整盒内容区锚几何不同域，合成语义未实现前强制同域会滤镜区/描绘盒错位（同 box-decoration-break: clone / bg-image owner 先例）。
4. 段表在旗标关时返回空表（两处构建点均门在 `single_frag_box_border_enabled()`），自然回退原 band/`is_first/is_last` 口径——**旗标关零行为变化**。

## 翻转阻塞（旗标转正前置，挂账）

1. **布局域 bidi item 裂解**（R5121 谱系）：解开 5 个伪通过页，否则旗标开即红。
2. **backdrop-filter inline 滤区/整盒几何统一（合成语义）**：门只护 test 侧，ref 侧同款 span（无 backdrop-filter、有 bg+border+pad）走整盒，跨页几何失配仍在（✗1.90%）；须 effects.rs 盒级滤区与 text.rs 内容区锚统一。
3. **per-(font,size) 度量缓存**：`fallback_line_metrics_for_paint` 每调用 `Face::parse`，整盒路径放大调用频次，转正前置。

## 旗标关惰性证据

- 逐页隔离复跑旗标关 = 基线判定逐字一致：bidi-001 ✓0.71、active-selection-012 ✓0.00、border-width-applies-to-008 ✗15.53。
- 构造性：全部新代码路径在 `whole_box ∧ 旗标 ∧ 段命中` 门内；multi_line 与 legacy 路径回退 `is_first/is_last` 原值；段表构建旗标关短路为空表。

## 常驻测试（+7，全绿）

- `crates/engine/src/paint/tests/r5299_inline_slice_segment_tests.rs`（4）：外来楔入切段+逻辑首末标记、子孙穴合并、空缝合并、子孙判定 walk。
- `crates/engine/tests/r5299_single_frag_slice_kill_switch.rs`（3，独立进程——R3858 教训：env 门旗标进程内 set_var 与并行单测互踩）：带域 once 单 fill、选区 fill 与 bg 同带域、backdrop owner 保留词域口径。

## 门禁

- make reftest **716/716**（557 真通过/47 近似/0 failed）；`cargo test -p zero-engine --release --lib` **3548P/0F**；fmt 0 diff；clippy（zero-engine --all-targets）0 告警；product-smoke **15.50%** ≤ 20% + struct-check PASS（welcome 同基线值持平）。
- bench-gate 定向 `zero-engine,zero-layout-engine` 首跑 GATE FAIL 8/25（对端 regression-guard 连轴 testharness sweep 全程并行，loadavg 峰 14.8）：
  ①zero-layout-engine 微基准 4 FAIL（deep_nesting 402µs / 基线 215µs / 预算 290µs、flex_1000、incremental、wide_tree）——**本轮该 crate 零 diff**，当前机器比记录基线慢 1.87×，R5212「记录基线陈旧环境漂移」判例同构；
  ②页级 medium/morning 墙钟 FAIL（medium layout_ms 296 vs 基线 123）——diff 不含任何 layout 改动，layout_ms 2.4× 只能来自并行负载；同跑内 welcome 墙钟全 PASS、morning/medium 差异化失败 = 单跑内非均匀噪声签名；
  ③**改动 crate zero-engine 微基准 7 指标全 NEW（首次建档）/PASS，零失败**。
  处置：R5298 四窗口噪声签名（FAIL 集 1→8→5→2 非定）延续，按 R5210/R5212 环境噪声判例交付，**零 relax 零基线改动**；R5298 挂账的 zero-layout-engine 空闲复测继续挂账（对端为连轴 sweep 序列本机静窗短期不可得；layout-engine 微基准记录基线重录属用户决策）。

## 挂账

翻转阻塞三项（见上）+ R5298 遗留（容器 strut < legacy 域 first-line-line-height-003、ruby anon 容器继承缺口 = 对端流碰头信号维持）+ para_bidi parity 近似收口 + 858F 账本续扫择轻。
