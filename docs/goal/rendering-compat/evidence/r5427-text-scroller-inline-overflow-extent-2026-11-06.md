# R5427 — 纯文本滚动容器行内溢出计入 extent：background-attachment-353 ✗1.56%→✓0.00%

日期：2026-11-06。上一轮：R5426（content-177 三缺口归因 + CDATA 修复，e637f77e8）。
本轮为 **misc 账本续扫择轻 + 修复轮**（R5426 下一步①）。

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
- **background-attachment-353（1.56%）＝本轮修复**（见下）。

## 归因（dump 几何量取 → 代码路径核对）

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

## 修复（painter/mod.rs scrollable_overflow_extent 单臂 +31/−0）

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

## 常驻测试（r5365_scroll_overflow_extent.rs 追加 5 针）

★纯文本滚动容器 extent 200×200（旧行为 (0,0,0,0)）／宽度失配不消费（回退重跑
IFC 同语义）／visible 容器不入臂／multicol 跳过／行内臂 × 子盒遍历并集。
**5/5 绿**（test-guard 包裹）。

## 验证

- 单案 ON ✓0.00% / OFF ✗1.56% 精确回滚（R3858 口径）；ON solo 3×3 全绿稳定。
- attachment-local 族（R4350/R4353/R4354/R5365 全修复面）：ON/OFF 全名单逐案同值，
  唯 353 翻绿；attachment-local-clipping-color-6 维持 ✗4.08% 在账（R4354③
  content-edge 圆残差，独立子缺口）。
- corpus 全量窗（同二进制，16606）：base **719F** → ON **719F**；movers＝
  −{353, table-anonymous-objects-147, border-collapse-dynamic-column-002}＋
  {table-anonymous-objects-148/150/151}；movers 全员 solo ≥3× **全绿**＝在册
  table-anon/border-collapse-dynamic 互翻抖动族洗清（guidelines #35）。**真态
  FIXED={background-attachment-353} REGRESSED=∅，719F→718F**。

## 门禁

- fmt `--check` rc=0；clippy zero-engine + zero-layout-engine all-targets
  `-D warnings` rc=0。
- make reftest（curated 725）**725/725**（558 真通过 + 54 近似，同 R5426 分布）。
- 受影响腿（test-guard，no-fail-fast）：**6251P/5F**＝js_dom_bridge 在册五案
  （p56/p99s/r3276/r2810/r2809，与 R5426 同名单逐字同——对端第 N 轮再现观察）。
- product-smoke welcome **17.23% ≤20% PASS**（与 R5419-R5426 逐字节同值）＋struct
  PASS；product-smoke-legacy **42/42 struct PASS**；form-input-perf-gate PASS。
- bench-gate 定向 zero-engine：待静窗补跑（本轮 paint 路径单点臂，R5419 先例
  「受影响 crate 定向跑」）。

## 清理

target/reftest-dump PNG 已清（含上会话遗留 01:52 旧 dump）；/tmp 窗口取证文件
（r5427_w0_full.txt 等）按 /tmp 常规。

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
