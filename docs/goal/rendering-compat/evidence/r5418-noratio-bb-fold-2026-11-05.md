# R5418 证据：no-ratio 替换元素 auto 尺寸注入的 border-box 语义折算——box-sizing-008 ✗1.46%→✓0.00% ＋ box-sizing-009 ✗4.83%→✗2.50% 半修，corpus 真态 712F→711F FIXED=1 REGRESSED=∅（2026-11-05）

## 1. 侦察结论（承 R5417 下一步① misc 账本续扫择轻）

- **组合态漂移重采**：对端 wpt-corpus 流 s284（img sizes=auto 机具三件套 fb2af536e ＋
  语料补缺 87125994a）入库，R5417 窗名单过期；重跑 corpus 全量窗 **712F**——差集 vs
  R5417 ON 窗（711F）＝NEW 3 / GONE 2 全在册抖动族（insert-inline-in-blocks-n-inlines-
  middle-001、border-collapse-dynamic-table-002、table-anon-149 ↔ table-anon-146/151
  互翻），R5417 FIXED=2 intact（flexbox_first-line ✓0.28%、multicol-inherit-002
  ✓0.00%）。组合态真态 712F 持平。
- **轻端三连侦察全弃**（案级 % 排序）：
  ①text-overflow-with-selection（0.06%/294px）dump 定谳**假轻**——test 页渲染
  `…A 654321` vs ref `…56 FEDCBA`，RTL 行内 bidi 视觉序内容级错误被字形稀疏掩成
  小 px 差（bidi 大域，邻接 box-level bidi RFC 待决策）；
  ②letter-spacing-ligatures-005（0.36%/1715px）——test 页 span 边界打断 Arabic
  joining＋letter-spacing 施加（10px 缝可见）vs chrome cursive 保持 join 不施加
  spacing（fuzzy 预算 9px 实证双页应像素同）；跨 span shaping 连续性＋cursive
  豁免＝text/shaping 深域；
  ③outline-028（0.31%/1500px）——float margin/padding 变体网格多错位＝float 外缘
  margin 参排系统性偏差（关联 floats-placement-vertical-001b/c 族，float 域
  R4234 判例区）。
- 择轻落点：box-sizing-008（1.46%/7000px，css-ui 替换元素 sizing 域，R5407/R5408
  判例域经验直接复用）。

## 2. 归因（dump PNG 几何量取 → chromium oracle 双页铁证 → 插桩定谳注入路径）

- **008 几何量取**（REFTEST_DUMP + PIL）：test 页 t01（border-box＋padding-left 20＋
  双 auto）绿块 80 宽 vs ref 页同位 100；t02（border-box＋padding-bottom 20）致
  第 2 行整体上移 20px（test y528 vs ref y548）。t00/t03/t04 恰好无对应轴 padding
  而免疫。
- **chromium oracle**（puppeteer-core getBoundingClientRect）：t01 bb **120×150**
  ＝content 100（intrinsic 不扣 padding）＋padding 20；t02 bb **100×170**＝content
  150（default 不扣）＋padding 20；t03/t04（显式 width:120/height:170）bb 120/170
  ——box-sizing:border-box 只改写**显式声明**尺寸的解释盒；auto 尺寸（固有维/
  default object size）恒 content-box 语义（css-ui-3 §box-sizing＋CSS2 §10.3.2/
  §10.6.2）。
- **注入路径插桩**（TEMP eprintln，用后即删）：5 img 全走 no-ratio SVG 臂
  （w100.svg＝`<svg width="100">` 有宽无高无比 → img_intrinsic_no_ratio 命中
  (Some(100), None)→ no_ratio_fired=true → 2105 门**整体跳过** min/max 约束表
  （跳表理由＝R3800 假比污染 min/max transfer，与 frame 折算无关）→ tree.rs
  既有 R3801 bb 折算臂（apply_replaced_min_max_constraint_table 内，门
  border_box＋css 双 auto＋stored_is_content_intrinsic）**不达** → both-auto
  非 flex 臂直写 content 语义 (100, 150) → taffy BorderBox 扣减 → content 80/130。
- **009 残差（半修定谳）**：修复后 4.83%→2.50%，残差＝第 3 行下移 10px。
  chrome 截图双页 diff **0**＋getBoundingClientRect：t02/t04 bb 120 的行进由
  margin-bottom −10 精确补偿（replaced 元素行内 baseline＝**下 margin edge**，
  CSS2 §10.8.1 系；bb 120−10＝div 100＋10 行进等）。ZW 第 3 行差 10px＝行内
  replaced baseline 取边语义独立缺陷（IFC 行内 baseline 域）——**第二个独立
  语义面，与 box-sizing 臂无关**，挂账另案。

## 3. 修复（tree.rs 单臂 +29/−2 ＋ 常驻测试 4 针）

- `crates/layout-engine/src/tree.rs` no-ratio both-auto 非 flex 注入臂：写入前
  门 `computed.box_sizing == BorderBox`（＋kill-switch）→ `frame_size(computed)`
  折算 bb＝content＋frame（与 R4165 attr 通道折算同源 helper）；显式侧
  （width:120 等 converter bb 语义直写）不触碰；flex both-auto 臂（R5411 主轴/
  交叉轴让位语义）无驱动案不触碰。
- kill-switch `ZW_REPLACED_NORATIO_BB_FOLD=0` 回退（default-on；tree.rs 内联 env
  先例 R5048/R5411 同款）。
- 常驻测试 `crates/engine/tests/r5418_replaced_noratio_bb_fold.rs` 4 针（R5411
  runner-map 注入法，字体无关）：★border-box＋padding-left 20→bb 120（旧径
  100 红）／★border-box＋padding-bottom 20→bb 170（旧径 150 红）／显式
  width:120 不重复折算守卫（重复计 140 红）／content-box 路径不污染守卫
  （双重计 140 红）。首版 SRC 带 width attr 走了 attr 直写臂（R2172
  extract_svg_data_uri_size 通道）——改无属性 data URI 对齐 corpus 真形后 4/4
  绿（注入点差异侦察副产品：attr 单值通道 1435 系 s283/R2172 域既有 content
  直写臂，R4165 已折垂直轴，水平轴＋单值臂缺口无驱动案不动）。

## 4. 单案 + corpus A/B（同二进制 kill-switch 双窗）

- 单案 008：ON ✓0.00% / OFF ✗1.46% 精确回滚（R3858 口径极性双向）。
- 单案 009：ON ✗2.50%（半修）／OFF ✗4.83%。
- corpus 全量（16594 案）：ON **711F** / OFF **710F**。差集：FIXED＝{008}
  ＋table-anon-153（OFF 红 ON 绿）；表观 REGRESSED＝table-anon-130/146/154
  （在册抖动族）。**单案隔离双侧 3×3**：153/130/146/154 ON/OFF 各 3 跑
  **24/24 全绿**＝抖动洗清（guidelines #35）。**真态 FIXED={box-sizing-008}
  REGRESSED=∅，712F→711F**。
- 修复域哨兵全绿：replaced-elements-{all-auto,height-20,max-height-20,
  max-width-40,min-height-20,min-height-40,min-width-40,min-width-80,width-40}
  （no-ratio 臂驱动案 9 案）＋intrinsic-percent-replaced-017/031＋dynamic 族
  ＋flex-minimum-width-flex-items-015＋flex-svg-no-intrinsic-column-001 全
  ✓0.00%（R5407/R5408/R5411/R5412 修复 intact；R3801 假比污染案 min-width-80
  intact）。

## 5. 门禁

- fmt 全 workspace FMT-CLEAN（首版仅 fmt layout-engine 漏新测试文件，make test
  fmt 门拦下后 --all 补齐——门禁有效性实证）。
- clippy zero-layout-engine＋zero-engine all-targets `-D warnings` rc=0。
- make reftest 本地集 **725/725（100%）**。
- make test workspace 腿 3727P/5F＝js_dom_bridge CSSOM 对端账第 5 轮同名单
  （p56_adopted/p99s_rule_style/r3276_add_rule/r2809_write/r2810_rule_style
  逐名同上轮，本轮零改动该域）中止，余腿 Makefile 原命令逐腿补跑 9 腿：
  ✓renderer-bin（3s）/✓renderer-lib/✓gpu1/✓gpu2/✓qjs-rb；红腿四：
  ①browser-bin 742P/1F＝default_actions **第 7 轮在册同名单**；
  ②browser-tests t28p37_menu_fade_pump_completes_without_input（新面孔，
  跑窗 loadavg 1min 峰 300＝对端 clone ZeroWeb r5391 重腿窗）——静窗 solo
  1/1 绿＝**负载假红定谳**（同文件 t28p37_menu_open… 2026-11-01 M2 slice3
  负载 flaky 先科，sibling 族）；
  ③qjs-ws image_batch_e2e（在册）——三轮定谳矩阵：主腿负载窗红（line 317
  count 断言）/静窗 bin 缺失红（line 233 workspace_bin 伪象，bench-gate/
  smoke/legs 多配置 target 争用置换，R5416 碰头同型）→补建 bin 后静窗全腿
  红（**39 vs 36 超取**，line 317）而 solo 单测静窗 ×2 绿——**leg 内并发
  形态稳态红**（超取计数随并发扰动漂移 36/39/42），升级从「负载敏感观察」
  为「leg 并发超取稳态」＝对端账（网络取图计数 fixture 域，本轮 tree.rs
  布局臂域隔离零交集）；
  ④qjs-rl mutation_arm **第 5 观察在册**。
- product-smoke 主腿 struct PASS（0 issues＋sibling-overlap＋count）rc=0
  ＋legacy rc=0（trend-only 字体墙基线数据非 pass/fail 面）。
- bench-gate 定向 `ZERO_WEB_BENCH_CRATES=zero-layout-engine` 静窗
  （loadavg 2.40）**GATE PASS 25 指标 NEW=0 全预算内**（tree.rs 布局热路径
  门控臂 BorderBox 短路分支零 measurable 回归）。

## 6. 环境账

- 对端 wpt-corpus 流 s285（testharness-html-semantics）重腿窗（loadavg 1min
  峰 32.5）与本轮 corpus 双窗错峰：ON/OFF 窗均在其腿间歇启动；table-anon
  mover 单案隔离 24 跑全程无外来 cargo/make 进程。
- 对端 clone（本仓同级目录 `../ZeroWeb`，r5391 nested-clip 测试链接）重腿
  窗 loadavg 1min 峰 300.95 与本轮余腿 9 腿重叠——browser-tests t28p37 红即该
  窗产物（静窗 solo 绿定谳）；qjs-ws 三轮矩阵跨窗（负载/静窗）均有红，与负载
  窗无单值因果＝并发形态稳态定性依据。
- target 多配置争用置换：bench-gate（磁盘守卫清 incremental 14GB）＋smoke＋
  quickjs legs 交错后 target/debug/{zero-browser,zero-renderer} 二进制消失
  （R5416 standalone 缺失碰头同型），qjs-ws 静窗复跑首轮 line 233 workspace_bin
  panic＝缺 bin 伪象非行为红；守卫补建 37.5s 后终验。
- R5417 轮 pkill 自杀先科复用：本轮重腿前 ps 静窗纪律全程维持。
