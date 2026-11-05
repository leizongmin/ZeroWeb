# R5427 — 纯文本滚动容器行内溢出计入 extent：attachment-353 ✗1.56%→✓0.00% ＋ attachment-350 ✗2.08%→✓0.00%

日期：2026-11-06。上一轮：R5426（content-177 三缺口归因 + CDATA 修复，e637f77e8）。
本轮为 **misc 账本续扫择轻 + 修复轮**（R5426 下一步①），同族两案两刀：paint stored
行盒臂（353）＋layout 标量 span 回退（350）。

## 侦扫与候选定谳（w0 窗 719F，对 R5426 终态 720F 在册抖动漂移）

轻端候选（≤2%）逐一 dump 定谳，六案落账/弃案、一案命中修复：

- **abspos-inline-007（1.40%）＝弃案**：block div 嵌 span（inline CB）内 abspos——
  ZW 整行结构塌缩（黑盒叠字 vs ref 三 156×20 边框盒），BII × abspos × inline-CB
  结构域（BII 深域邻接），非单闸门。
- **ruby-box-generation-001/003/005（1.79/1.83/1.76%）＝弃案**：`<p>` 直子 rb/rt/rbc/rtc
  须匿名 ruby 盒包装（css-ruby-1 #box-fixup）——ZW 无 ruby 盒 fixup，裸元素按纯文本
  排版、annotation 整行丢失。特性域（ruby 盒生成），同 ruby-overhang（R5419⑥）族。
- **ruby-annotation-pairing-001（1.74%）＝弃案**：annotation 横向配对中心化分岔
  （Ahem 块状图，行 1/3 annotation x 错位），ruby 配对布局中域。
- **scope-pseudo-element（1.33%）＝弃案**：@scope 内 ::before/::after/::marker 不
  命中 scoped 规则（ref 蓝 highlight + 红 B/A/M 标记全缺），@scope × 伪元素级联
  特性缺口（style-system 域）。
- **contain-inline-size-bfc-floats-001（1.67%）＝弃案**：fit-content × float 排除带
  （contain:inline-size BFC）——orange inline-block 落位 y/x 双错，R5213/R5419⑦
  shrink-to-fit band-fill 切片邻接深域。
- **background-attachment-353（1.56%）＝修复一**（见下）。
- **background-attachment-350（2.08%）＝修复二**（同族第二案，353 修复后复核发现
  未翻转——见下）。

## 修复一：paint stored 行盒臂（353）

### 归因（dump 几何量取 → 代码路径核对）

`div#target`（overflow:hidden，100×100）内容 `&nbsp;&nbsp;<br>&nbsp;&nbsp;`（Ahem
100px）→ nbsp 不可断行，行盒 200×200 溢出。`background-attachment: local` +
`background-position: center`：定位区应为 scrollable overflow 矩形（200×200）→
图像 (50,50)-(150,150)，窗口内露出图像左上象限（绿 50×50）恰盖 #control 的红块；
窗口其余露出 #control 绿边 → 全绿。ZW 实测图像落 (0,0)（绿象限在盒左上 + 红 L）＝
**定位区回退 padding 盒**。

机制：`scrollable_overflow_extent`（painter/mod.rs，R5365）只遍历子 LayoutBox——
行内内容不是子盒，纯文本滚动容器的行溢出不可见 → extent 缩回 content 盒 →
`local_scrolled_positioning_area` 判「无溢出」返回 None → R5365 fallback padding 盒。
（css-overflow-3 §2.2：line boxes 属 scrollable overflow。）

### 实现（painter/mod.rs scrollable_overflow_extent，kill-switch `ZW_INLINE_OVERFLOW_EXTENT=0`）

extent 顶部新增**本盒直系行内内容臂**：stored `inline_layout` 行盒/片段 span 计入
acc（content-box 系；行 y/height 纵向、fragment x/width 横向）。门与消费方同口径：

- 滚动容器门 `overflow_x/y ∈ {Hidden, Scroll}`（与 paint 子内容平移门 mod.rs:4185
  同口径；Clip 不建滚动容器）；
- stored IFC 有效门 `(inline_layout_width − content_width) < 1px`（与 text.rs
  use_stored 同校验——table/multicol 后处理改宽时存储行不可信）；
- `!is_multicol`（列流存储行坐标已重映射，入 extent 双重计入）；
- kill-switch `ZW_INLINE_OVERFLOW_EXTENT=0`（OnceLock 快照，R5184 口径）回退纯
  子盒遍历。
- FIXME：后代盒自身的行内行溢出传递未建模（无驱动案；R5365 子盒遍历只覆盖
  child-box 几何）。

消费面（全为滚动语境，非滚动 local 维持 padding 盒）：①`local_scrolled_positioning_area`
（本轮驱动）；②`normalized_scroll_offset`（脚本滚动钳——纯文本滚动容器此前钳 0，
现为真范围）；③paint 子内容平移钳（mod.rs:4193）。`local_scrolled_paint_extension`
（R4354 绘制区扩展）维持 children-fold 口径不动（353 期望像素不受其影响——绘制区
= 窗口即可；避免双点同改，见 guidelines #36 反例教训，留待有驱动案再统一）。

## 修复二：layout 标量 span ＋ paint 回退消费（350）

### 归因（353 修复后复核）

350 同族案形（overflow:hidden 100×100 盒 + nbsp 长行 200×200 + local +
`position: bottom right`，期望图像底缘移到 y=200 出窗），353 修复后**未翻转**
仍 ✗2.08%。差异点：350 无 `font-family: Ahem`（默认字体）→ 非 pure-Ahem 容器 →
`inline_layout` = None（R84/R355 store-gate 拒存，非 Ahem 字体度量失真域）→
paint 行内臂 stored 门不过。

### 实现（两文件）

1. **layout 侧**（`inline_finalization.rs compute_final_inline_layouts`，store-gate
   return 之前——R5166/R5227 先例区）：IFC 运行后聚合 `inline_ctx.lines` 存标量
   `LayoutBox.inline_overflow_extent = Some((max_right, max_bottom))`（content-box
   系；行 y/height 纵向、run x+relpos shift+width 横向，位移烘焙与 stored 面
   line 2186 同口径）。**所有文本容器都写**（含非 stored）；竖排书写模式不入
   （R5227 同门，行盒坐标已重映射）；空行集不存；FIXME direction:rtl/竖排负向
   span 未建模。只存标量、不落 `inline_layout`——那会翻转 paint use_stored 路径
   （R5227 oof_static_line_bottom 同款理由）。存面恒写（O(行×run) 常数小），消费端
   受 `ZW_INLINE_OVERFLOW_EXTENT` 总门。`types/mod.rs` 加字段（Default None）＋
   `engine.rs` LayoutBox 字面量初始化同步。
2. **paint 侧**（`scrollable_overflow_extent` 臂内二选一）：stored 行有效取全 span
   （含 min 侧）；否则回退消费标量 span（min 侧恒原点）。优先级针防止同盒双计。

## 常驻测试（r5365_scroll_overflow_extent.rs 追加 8 针）

★纯文本滚动容器 extent 200×200（旧行为 (0,0,0,0)）／宽度失配不消费（回退重跑
IFC 同语义）／visible 容器不入臂／multicol 跳过／行内臂 × 子盒遍历并集／
★非 stored 标量 span 回退（旧行为恒 (0,0,0,0)，350 形）／stored 行优先标量不叠加
（999 不穿透）／stored 失配回退标量（不再弃整臂）。
**8/8 绿**（test-guard 包裹，cargo test -p zero-engine --lib r5427）。

## 验证

### 修复一（353）

- 单案 ON ✓0.00% / OFF ✗1.56% 精确回滚（R3858 口径）；ON solo 3×3 全绿稳定。
- attachment-local 族（R4350/R4353/R4354/R5365 全修复面）：ON/OFF 全名单逐案同值，
  唯 353 翻绿；attachment-local-clipping-color-6 维持 ✗4.08% 在账（R4354③
  content-edge 圆残差，独立子缺口）。

### 修复二（350）

- 单案 ON ✓0.00% / OFF ✗2.08% 精确回滚（R3858 口径）；353 ON 维持 ✓0.00%
  （stored 路径优先臂不受回退改造影响）。
- background-attachment 全集 61 案 ON/OFF 逐案对比：差异恰 = {350, 353} F2P；
  fixed-block-002/fixed-inline-002（100%）＋color-6（4.08%）两态同值在账
  （attachment:fixed 域＋R4354③ 残差，非本轮扰动）。

### corpus 全量窗（16606 案，同二进制）

- base（w0，pre-353-fix）719F → 353-fix ON 719F → 350-fix ON **719F**。
- 350-fix 窗 movers（vs 353-fix ON 窗）：−{350, border-collapse-dynamic-column-002,
  table-anonymous-objects-147} ＋{insert-inline-in-blocks-n-inlines-end-002 21.52%,
  border-collapse-dynamic-column-001, table-anonymous-objects-145,
  run-in-table-between-003}；**四 mover 全员 solo ≥3× 全绿**＝在册互翻抖动族
  （table-anon/border-collapse-dynamic）＋同形抖动（insert-002/run-in-003 solo
  恒 ✓0.00%），guidelines #35 洗清。
- **真态：FIXED={background-attachment-353, background-attachment-350}
  REGRESSED=∅，719F→717F**（两修累计；各窗账面 719F 系抖动族逐窗 ±1 漂移）。

## 门禁

- fmt `--check` rc=0（首版 painter 测试文件两处超宽 rustfmt 修复）；clippy
  zero-engine + zero-layout-engine all-targets `-D warnings` rc=0。
- make reftest（curated 725）**725/725**（558 真通过 + 54 近似，同 R5426 分布）。
- 受影响腿（test-guard，no-fail-fast，cargo test -p zero-engine -p
  zero-layout-engine）：**6254P/5F**＝js_dom_bridge 在册五案
  （p56/p99s/r3276/r2810/r2809，与 R5426 同名单逐字同——对端第 N 轮再现观察；
  6254 = 6251 + 本轮 3 新针）。
- product-smoke welcome **17.23% ≤20% PASS**（与 R5419-R5426 逐字节同值）＋struct
  PASS；product-smoke-legacy **42/42 struct PASS**；form-input-perf-gate PASS。
- bench-gate 定向 zero-engine（负载沉降后，15min 均值 27→静窗）：**GATE PASS 19
  指标（NEW=7）**——welcome/medium/morning parse/style/layout/paint/total 全
  PASS；first_paint_wall_ms welcome 15.21／medium 325.89（预算 362.35 内，R5424
  同档漂移形）／morning 148.16 全 PASS；peak_rss 187.43 预算内。7 个 zero-engine
  微基准 NEW 维持 R5424 状态（capture 随待用户决策⑤挂起）。
- bench-gate（修复二后复跑，终态二进制）：首跑 GATE FAIL 2 指标（medium/layout_ms
  217.27、medium/first_paint_wall_ms 446.13）＝编译尾窗污染定谳（R5419 先科同型：
  首跑含 bench 编译余波，负载 5min≈17）；静窗复跑（1min=3.1）**GATE PASS 19
  指标（NEW=7）**——medium/layout_ms 156.15（预算 181.9 内，基线 123.39 同 R5424
  漂移形）、first_paint_wall_ms welcome 15.66／medium 327.45（预算 362.35 内）／
  morning 153.18 全 PASS；peak_rss 180.86 预算内。两跑同二进制，FAIL 跑非代码
  回归（guidelines #35 窗口污染定谳）。

## 清理

target/reftest-dump PNG 已清（含上会话遗留 01:52 旧 dump）；/tmp 窗口取证文件
（r5427_w0_full.txt、r5427b_on_window.txt 等）按 /tmp 常规。

## 挂账（新增五条 + 维持）

- ⑭abspos-inline-007：BII × abspos × inline-CB 结构塌缩（1.40%，BII 深域邻接）
- ⑮ruby 盒生成 fixup 特性缺失（css-ruby-1 #box-fixup；ruby-box-generation-001/003/005
  1.76-1.83%，与 ruby-overhang ⑥ 同族）
- ⑯ruby annotation 配对布局中域（ruby-annotation-pairing-001 1.74%）
- ⑰@scope × 伪元素级联特性缺口（scope-pseudo-element 1.33%；scope-implicit-print
  族 6 案在 2.08 大簇待甄别）
- ⑱fit-content × float 排除带（contain-inline-size-bfc-floats-001 1.67%，R5213
  shrink-to-fit band-fill 切片邻接）

R5426 三条（⑪abs 盒直系文本 display 门／⑫abs branch STF 折行分岔／⑬ruby abs CB
几何）＋R5425 四域＋R5424/R5423/R5422/R5421 全部维持；**待用户决策清单**①-⑤维持
（无新增）。c64-uri-000 三缺口基建评估维持（本轮未动：runner load_linked_stylesheets
absolutize + 外链 CSS 图片预取属 runner 基建轮，影响面 css1 全族 + 全语料外链案，
维持「先评估再动」挂账）。
