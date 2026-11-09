# R5428 — 视口滚动 root-scroller：'html' 滚动记录导出＋paint 收藏原点平移——FIXED=5（overflow-clip-margin-007/012、document-element-overflow-hidden-scroll、clip-text-on-body-scroll、transform-fixed-bg-002）＋伪绿转化真绿两案（fixed-z-index-blend、contain-layout-021），账面 OFF 718F→ON2 711F

日期：2026-11-06。上一轮：R5427（纯文本滚动容器行内溢出计入 extent，FIXED=2 真态 717F）。
本轮为 **misc 账本续扫择轻 + 修复轮**：css-overflow/css-position 滚动驱动族五案 + 伪绿
转化两案，五缝修复（shim 导出／painter 字段／resolve 抽取／R4353 root 臂／收藏原点平移）。

## 侦扫与候选（w0 窗 720F）

misc 轻端候选 dump 定谳命中同根五案：`overflow-clip-margin-007/012`、
`document-element-overflow-hidden-scroll`、`clip-text-on-body-scroll`、
`transform-fixed-bg-002`——案页共同形＝`onload = function(){ documentElement.scrollTop
= window.innerHeight * N }` 后断言视口滚后画面。

## 归因（机制链两段拆分）

**handler 一直有跑**（非缺口）：shim R2932 通用 on* IDL accessor（part06）把 `onload
= fn` 转成 `_globalAddEventListener('load', fn)` 入 html 三合一 key（tgt:'win'）；
runner `__zw_dispatch_event('html','load',null)` 泛型缺省 bubbles:true → win 虚站在
bubbles 路径触发。REFTEST 探针实证 handler 执行。

**真缺口两处**：

1. **shim 不导出窗口滚动槽**：`documentElement.scrollTop` setter 经 M2-S7（part05，
   HTML 标签门）委托 `globalThis.scrollTo` → `_winScroll` 槽（part01）；而
   `__zw_dump_scroll_offsets` 只导出 `_scrollOffsets`（per-element 记录），
   `_winScroll` 不回流 → paint 侧拿不到视口滚动值。
2. **paint 侧无视口滚动概念**：旧滤显式丢弃 `html`/`body` 记录（R4353 期「canvas
   渲染无视口滚动概念」旧行为）——即使记录回流也不消费。

规范定位：css-overflow-3 §2.2/§2.3 根元素可滚溢出传播至 viewport（滚动根元素＝滚动
视口）；cssom-view-1 §11.2 根元素 scrollTop/scrollLeft 即视口滚动偏移，且 client 面
按**视口尺寸**量取而非根盒 auto 高（根盒 auto 高＝文档高，按盒面钳会归零）。reftest
canvas 即视口窗（dump PNG 800×600 viewport-sized）：视口滚动＝文档子树整体平移，
画布背景保持视口锚定（CSS2 §14.2 canvas background）。

**时序约束**（缝 5 动因）：根 scope `collect_positioned_descendants` /
`collect_floats_descendants` 在子盒递归 R4353 块**之前**执行 → 钳制计算须前置，
收藏原点须随视口滚平移，否则被根 scope 认领的 positioned 后代全部绘在未滚位。

## 实现（五缝，painter 侧单一 kill-switch `ZW_ROOT_VIEWPORT_SCROLL=0`）

1. **shim 导出**（part01.js `__zw_dump_scroll_offsets` 尾）：`_winScroll` 非零时
   追加 `{selector:'html', scroll_top, scroll_left}` 记录（零偏移跳过，同 sel 条目
   口径）。
2. **painter 字段**：`viewport_scroll: Option<(f32,f32)>` ＋
   `viewport_scroll_node: Option<NodeId>`（Default 同步）。
3. **resolve 抽取**（paint()）：kill-switch 门内 for 循环抽取首个 'html' 记录入
   `viewport_scroll`（不入 per-element map——根盒重复消费双计）；既有 html/body
   滤保留。kill-switch 为 OnceLock env 快照（R5184 口径），回退 R4353 旧行为。
4. **root-scroller 钳制**（收藏前置计算，is_scope 之后 collect 之前）：
   `is_root_scroller`＝viewport_scroll 命中且盒 node 匹配；钳制值按
   `reversed_scroll_axes`＋`scrollable_overflow_extent`（R5365）与**视口 client
   面**逐轴钳（正/反向轴分支）；收藏原点 `(abs_x,abs_y)` 平移
   `collect_positioned_descendants` 与 `collect_floats_descendants` 两调用。
5. **R4353 子盒递归 root 臂**：复用前置 `is_root_scroller`——`root_scroll.or()`
   接入、filter 放行、client 面分支取 `(viewport_w, viewport_h)`（替代盒面
   border/padding 扣除）。

FIXME（挂账）：fixed CB 域分析未建模——css-contain-1 §3.1 contain:layout/transform
祖先使 fixed CB 转移（随文档滚，contain-layout-021 语义正确）；真 viewport CB 应
视口锚定不随滚，本实现一律随文档平移。fixed-z-index-blend 的 .blend/.background
无背景面像素无差，待像素可辨驱动案再分域。

## 常驻测试（r5428_viewport_scroll.rs，4 针全绿）

★'html' 记录抽取为 viewport_scroll 不入 per-element map 且平移子树／★client 面按
视口量取钳制（滚动 300 钳到可滚范围 100——按根盒 auto 高会钳 0 不平移）／元素
scroller 路径不变（R4353 回归卫：overflow:hidden 内红块滚入窗裁剩 50 高）／★定位
flush 随视口平移（abspos top:600 滚 100 后 y=500，收藏原点平移臂）。观测口径注记：
overflow 裁剪 rewrite 对 fill 按 clip rect 钳制边（非真交），断言取平移后可分辨
几何（子块部分露出裁剪窗）。

## 验证

### 伪绿转化链（guidelines #33 自洽≠合规实证）

`fixed-z-index-blend`（test 侧 `window.scrollBy(0,4000)`）与 `contain-layout-021`
（`documentElement.scrollTop += 200`，fixed CB＝html contain:layout）修复前双侧未滚
自洽伪绿；缝 1-4 激活滚动后 ON1 窗**诚实败**（test 侧全白＝定位 flush 未平移）；
缝 5 收藏原点平移后 ON2 双双 ✓0.00% **真绿**。

### corpus 全量窗（16606 案，同二进制 kill-switch A/B）

- w0 基线（修复前二进制）**720F**——含 4 例在册互翻族额外败（insert-block-in-blocks
  -n-inlines-middle-002、table-anonymous-objects-147、intrinsic-percent-replaced
  -dynamic-002/007；对照 OFF 窗 718F 互翻 table-anonymous-objects-145/150）。
- 缝 1-4 ON1 窗 **714F**：007/012/document-element/clip-text-on-body-scroll 四案翻
  绿；fixed-z-index-blend/contain-layout-021 诚实败（见上）。
- 缝 5 ON2 终态窗 **711F**：transform-fixed-bg-002＋fixed-z-index-blend＋
  contain-layout-021 翻真绿。
- **终判 A/B（OFF 718F vs ON2 711F，同二进制仅 kill-switch 翻转，按案名比对）**：
  F2P 8＝**FIXED 5**（007/012/document-element/clip-text-on-body-scroll/
  transform-fixed-bg-002）＋table-anonymous-objects-145/150/154 抖动绿漂；P2F 1＝
  scroll-positioned-multiple-background-images 抖动红漂。四 mover **solo ≥3× 双向
  全绿洗清**（guidelines #35），**真态 FIXED=5 REGRESSED=∅，账面 718F→711F 净 −7
  （＝FIXED 5＋抖动净漂 −2）**；对 R5427 账面真态 717F 差 1＝在册 ±1 抖动带。
- 五 FIXED 全部 **solo ≥3× 确定性 ON✓/OFF✗**（单案精确回滚 R3858 口径）。

## 门禁

- fmt `--check` clean；clippy zero-engine all-targets 0 error（首版 3 lint：2×
  collapsible_if 合并 let-chains＋1× unnecessary_lazy_evaluations 改 `.or()`）。
- engine lib 腿（test-guard 包裹）**3758P/5F**＝js_dom_bridge 在册五案（p56/p99s/
  r3276/r2810/r2809）同名单逐字再现（对端观察项维持）；本轮 4 新针全绿。
- make reftest（curated 725）**725/725**（521/521 Layout＋204/204 Text）。
- product-smoke welcome **17.23% ≤20% PASS**（逐字节同值）＋struct PASS 全绿＋
  form-input-perf-gate PASS。
- bench-gate 定向 zero-engine（静窗，load 0.75 起单跑）：**GATE PASS 19 指标
  （NEW=7）全部预算内**——medium/first_paint_wall_ms 327.9（预算 362.35）、
  morning 155.29（预算 241.51）、peak_rss 180.36（预算 359.02）。

## 清理

target/reftest-dump PNG 已清（overflow-clip-margin-007/012、fixed-z-index-blend、
contain-layout-021 取证 dump）；/tmp 窗口取证文件（r5428_w0/on/off/on2*.txt、
off_names/on2_names、s_*.txt 中转）按 /tmp 常规。

## 挂账（新增一条 + 维持）

- ⑲fixed CB 域分析未建模（css-contain-1 §3.1 containment/transform 祖先 CB 转移随
  文档滚 vs 真 viewport CB 视口锚定不随滚；contain-layout-021 语义正确侧，
  fixed-z-index-blend 无背景面像素无差；待像素可辨驱动案再分域）

R5427 五条（⑭-⑱）＋R5426 三条＋R5425 四域＋R5424/R5423/R5422/R5421 全部维持；
**待用户决策清单**①-⑤维持（无新增）。c64-uri-000 三缺口基建评估维持；对端
js_dom_bridge 五案观察维持（本轮 lib 腿同名单第 N 轮再现）。
