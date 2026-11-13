# R5517 轮次证据：纯文本 float 收缩值 min/max-width 交叉约束钳（CSS2 §10.3.5 × §10.4）

日期：2026-11-14
轮次：R5517（R5516 后继）
择案：css-sizing/box-sizing-content-box-003（/tmp/r5516/fails-r5516.txt 682F 盘点；
css-sizing 目录 triage——root-canvas-001 object 子文档域排除、replaced-intrinsic-001/003
R3855 svg-replaced defer 族排除、intrinsic-ratio-replaced-box-sizing 双子机制排除后择定；
账本 grep 零在册）。
结论：**驱动案 solo ✗（6/6）→ ✓0F（6/6）**。corpus 全量 A/B（16594 vs R5516 轮
682F）：680F，GONE=3（驱动案＋tao-127＋insert-block-in-blocks-n-inlines-middle-003）、
NEW=1（gradient-repaint conic）；双二进制 solo×3 收账——驱动案 base ✗×3/new ✓×3＝
真修，tao-127 base solo 亦绿＝互翻族相位非真修，insert-middle-003 系 R5516 双二进制
已证批噪声相位（上窗 NEW 本窗 GONE 互翻，枚举勘误补记），gradient NEW new solo
全绿＝批跑 storm 相位。**真态 FIXED=1 REGRESSED=∅**。

## 一、取证链

- 案面：ref 腿 `.box-sized { min-width:250px; max-width:350px; height:100%;
  float:left }` 双纯文本 float（"LEFT HALF"/"RIGHT HALF"）于 abspos 容器
  （min-width:500px）应并排填满（各 250）；ZW 旧态两 float 收缩到词宽 ~84/95px
  （probe4 最小形 LAYOUT_DUMP：w=9.8/10.7＝"L"/"R" 词宽），红底外露 ✗~7.8%。
- 机制链：float auto 宽三条 used-width 写入路径中，**仅纯文本臂缺 §10.4 钳**——
  ①R5368 收缩臂（block/replaced/floated 子）与 sizing.rs 钉宽臂经
  `block_min/max_content_width` 测量（block_intrinsic_width_ex 内嵌 R5368 自身
  min-width 地板/max cap，§10.4 序）自带约束；②converter 把 min/max-width 送
  taffy min_size/max_size（taffy 解出的地板宽 250 在场）；③`shrink_pure_text_floats`
  纯文本测量走 `text_content_max_width`/Σ walk（裸文本度量，无 R5368 内嵌语义），
  收缩值 < taffy 宽即写回 → **converter min_size 地板被词宽覆盖**（max cap 侧
  taffy 已钳、raw > 宽不触发写回，故仅 min-floor 面暴露）。
- 旁证：kill-switch `ZW_FLOAT_MINMAX_CLAMP=0` 下锚测试回红（float one=90＝
  9 词符×10px advance，地板丢失）；on 后 250。max-width cap 锚两种状态均绿
  ＝converter/taffy cap 面独立在岗，钳的 cap 臂为防御性对称（写回面 raw<宽
  才可达，converter cap 后不可达绑定态）。

## 二、修复（两处，+24/−4）

1. **intrinsic_sizing.rs `clamp_used_width_cross_constraints`**（新 pub(crate)，
   §10.4 单 owner）：tentative used border-box 宽过自身 definite min/max-width
   交叉约束——先 max cap（Calc 混合式含循环 % 不 cap，同 clamp_box_max_contribution
   R5368 口径）后 min floor（`resolve_min_width_intrinsic_floor`：%/content 关键字
   None 跳过、calc 取 px 部分）；box-sizing:border-box 时 min/max 即 border-box 值
   不再加 frame。block 系内在测量臂已有内嵌语义不走此函数。
2. **float_positioning.rs `shrink_pure_text_floats` 写回点**：收缩值经上函数钳后
   写回（先 cap 后 floor＝§10.4 min 胳 max 序）；content_width 同式回
   `(used−frame)`——无 min/max 时钳恒等、行为逐字节不变（含 kill-switch
   `ZW_FLOAT_MINMAX_CLAMP=0` A/B 门）。垂直臂（R4462/R4463 列宽域）不动
   ＝writing-mode 深域既有避让。

## 三、锚测试（r5517_float_minmax_clamp_tests，2 针）

①驱动案最小形：abspos 容器 min-width:500px＋双纯文本 float
min-width:250px/max-width:350px 应各 250 并排（two.x = one.x+250）；红态由
kill-switch off 值差算术确立（90 词宽 vs 250）。②max cap＋box-sizing frame
两态：content-box cap 入内容盒（border-box=64）、border-box cap 即 border-box
（宽 50/内容 36）。文本度量接固定 10px/字符 advance source（r1242 同款接线；
engine 测试不装 shaping/resolver 恒 Legacy＝锚点空转教训，R5509 挂账③
`<style>` 须解析后传 compute_styles 同批防复发）。

## 四、验收

- lib：**2506P/0F/2I**（2504 基线＋2 新锚）。
- solo：驱动案 **✓0F**（旧 ✗6/6；EXIT=0）。
- corpus 全量 A/B（16594 vs /tmp/r5516 轮 682F）：**680F**——GONE=3＝驱动案＋tao-127
  （在册互翻族，base solo×3 全绿＝批跑相位）＋insert-middle-003（R5516 双二进制已证
  批噪声相位，上窗 NEW 本窗 GONE＝互翻枚举勘误补记）；NEW=1＝gradient-repaint
  conic（在册噪声族，new solo×3 全绿＝storm 相位）。**真态 FIXED=1 REGRESSED=∅**。
- make reftest：**725/725**（521 layout＋204 text）。
- make test：**3919P＋7F＝js_dom_bridge 对端在册族恰集**（img/svg settle 2＋fetch 2
  ＋websocket 2＋iframe-sw 1，R5513/R5516 同集先例；改动面零涉网络桥面；workspace
  腿止于 engine 包，layout-engine lib 单包复验 **2506P/0F/2I**＝R5516 同款；fmt 门禁
  净；guarded-test 24G，155 套件，/tmp/r5517/make-test2.log 在案）。
- fmt 净＋clippy `-p zero-layout-engine --all-targets -D warnings` 净。
- product-smoke：**6/6 struct PASS**＋welcome chromium oracle **17.23%**
  （R5501 基线同值）＋form-input-perf-gate PASS。
- 定向 bench-gate（zero-layout-engine，R5516 挂账顺带收账）：**GATE PASS 25 指标
  NEW=0**（静窗 load 1.05 触发；/tmp/r5517/bench-gate2.log 在案。插曲：target-disk-guard
  全量清 target/ 97GB 超阈＝R5515 已知坑复现，bench profile 全量重编后测量）。

## 五、挂账与下一步

- **% min/max-width on float used 域**：钳只认 definite 长度（% 经 resolver
  None 跳过）——与 R5506 挂账①「% max-width used 域钳缺口」同族维持，不另开账。
- 防御性 cap 臂（converter cap 后写回面不可达绑定态）维持现形，不删
  （§10.4 单 owner 完整性；converter 面演变时自动接管）。
- R5516 残差（cell B 幽灵列 9.6px 亚阈在册）与 taffy-local 隔离盒末子
  margin-bottom 深域记档全维持。
- 下一步 R5518：①corpus FAIL 盘点续择轻量（680F 清单 /tmp/r5517/
  fails-r5517.txt，避开行进新域/在册深域/待决策面/动态域/tao 互翻族）②维持项
  ＋对端挂账观察＋重腿静窗纪律维持。

## 六、本轮动作与卫生

- 临时诊断单测 3 枚（body 直下/abspos 嵌套/锚 A 原样 dump）用后删净；
  探针页 probe4.html 用后留 /tmp（wpt-data gitignored，不入仓）。
- 对端 ZeroWeb-3 storm 期（load 135→31）轻门禁先行（fmt/clippy/product-smoke
  单页渲染），make test/bench-gate 等窗收账——单重腿串行无撞窗。
