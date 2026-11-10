# DC-3 全量对账预备（只读盘点，未动码面与测量号）

2026-11-11 凌晨。本文件是 DC-3 的执行预备：账面现状、周跑 pipeline 运行手册、
验收核对清单、缺口→产出地图。全部数字来自已发布快照与本地记录轮证据，无新测量。

## 1. 账面现状

| 时点 | 口径 | css 通过 / 分母 | 来源 |
|---|---|---|---|
| 11-07 发布快照 | 同口径全量 | 29,392 / 196,331 | [coverage-2026-11-07](../../../compat/wptfyi/coverage-2026-11-07.md) |
| 11-09 发布快照 | 同口径全量 | 29,958 / 196,461 | [coverage-2026-11-09](../../../compat/wptfyi/coverage-2026-11-09.md) |
| 11-10 发布快照 | 同口径全量 | **52,488 / 196,597（26.70%）** | [coverage-2026-11-10](../../../compat/wptfyi/coverage-2026-11-10.md)，run 42801058132 |

11-10 快照 head 3bfccdfa3 的内容态含 Typed OM API 面、color 管线前段批与 13
语料扩面（transitions 185 / om-view 545 / geometry 209 / animations 225 与
master.md 扩面批数字逐目录吻合）；**不含**其后五批（rebase 重写使哈希祖先检测
失效，以内容数字对账为准）：

| 已落库未入快照批 | 本地记录轮增量 | 快照 vs 本地 |
|---|---|---|
| values serialize（8b5fc9ebd 等价提交） | values 1,548→2,213（净 +665，v5d 256 页口径） | 周跑 985 → 本地 2,213（**+1,228**） |
| color currentcolor ctx / light-dark / shadow dark（d1dd30e7a、639bed48a、6dd926911） | color 三批净 +26×2 | 周跑 10,756 → 本地 10,808（**+52**） |
| grid CSSOM 声明面（17f30543c） | 锚页 13→94（+81） | 周跑含旧态 |
| typed-om `.any.js` 变体通道 | +43（v5b 9,872） | 周跑 9,829 |

粗期望：终 HEAD 周跑 css ≈ 53.9k（52,488 + ~1,400）。**验收线 182,392 → 尚余
~128.5k**，主缺口是未跑语料（未覆盖子测试 144,109）与已覆盖失败面的引擎能力。

## 2. 在途三链与测量号（本轮多实例仲裁，zeroweb-2-fe 定谳）

- v11 = zeroweb-2-70：fill/stroke gCS 读出面 + color currentColor→inherit（对
  v10 基线 10,808P 预期净 +3）；脚本 `.acceptance/wptfyi-css/run-v11.sh`。
- v12 = image 链（light-dark image 上下文 31F 挂账），落库后跑。
- v13 = zeroweb-2-fe：grid G2a（grid-line 双分量模型）css-grid 分片。
- 周跑（DC-3 全量对账）自 **v14** 起报号，待三链落定后执行。

## 3. 周跑 pipeline 运行手册

入口 [weekly.yml](../../../../.github/workflows/weekly.yml) `wptfyi-coverage`
矩阵 + `wptfyi-coverage-aggregate`，全自动：

1. **前置**：三链及后续批次全部 push main；`gh run list --workflow=weekly.yml
   --status=in-progress` 确认无在跑实例——concurrency `weekly-build` 是
   cancel-in-progress，dispatch 会砍旧 run 尾巴。
2. **dispatch**：`gh workflow run weekly.yml --ref main -f run_wptfyi_coverage=true`
   （只跑 coverage 矩阵，跳过 build matrix）。job checkout `ref: main`。
3. **执行**：discover-shards 动态分片矩阵；每分片 build zero-wpt-runner + 定向
   fetch 语料（41/35 号 pin 树逐文件对账 sync）+ test-guard 包裹跑（per-proc 6G
   / total 8G / per-suite time_limit；job 级硬顶 360min）。上轮全程 ~7h
   （10:39Z 起、17:44Z 快照生成）。
4. **产物**：aggregate job merge 各分片 JSON → `wptfyi_gap.py fetch + coverage`
   → `docs/compat/wptfyi/coverage-<date>.md/.json` 自动提交回 main；artifact
   `wptfyi-coverage` 留 14 天、`zw-results-<shard>` 留 3 天。
5. 分片容错契约：单套件 fetch 失败/runner 无 JSON 只废该套件面（记 0，同未跑
   态）+ 分片红，不连带兄弟套件；个别分片红不吞快照。

## 4. 验收核对清单（映射 wptfyi-90-plan 共同验收）

- [ ] **同 revision**：快照头部三家 aligned sha + ZW 语料 pin（315976933 /
  04067ce9）入账；上游漂移（分母变化 196,597 → 终值）另列。
- [ ] **分片截断检查**：终跑 per-suite 数对本地最新记录轮核对——values 应达
  serialize 批水平（~2.2k，若仍在 ~985 级即语料陈旧/分片截断信号）、color
  ≥10,808、typed-om ≥9,872、grid 含 G2a 面；差异先归因页集口径再比较。
- [ ] **绝对线**：css 通过 ≥182,392。
- [ ] **邻域无可归因回退**：以 coverage-2026-11-10.json `groups` 逐目录
  zw_passes 为基线取证，css 内不降（已知重写面单列），相邻 topdir
  （html/editing/svg/…）无本目标可归因回退。
- [ ] **环境受限失败**单列（`zw_env_fails`），不计引擎缺口。
- [ ] 每批「新增执行/失败转通过/已有通过回退/超时/未执行」五分账入 master.md。

## 5. 缺口→产出地图（11-10 json，90 子组全 144,109 缺口分桶）

按产出特征分三类（全表见 coverage-2026-11-10.json `groups`，此处列大于 800 的面）：

**A 类 · 语义面（CSSOM/解析/级联——现有 shim+engine 通路可直接收，DC-2 先例
单批 1k+）**：
css-values 余 ~8.2k（math functions computed/序列化已收一批）、css-fonts 6,347
（font-face/descriptor 解析面）、selectors 4,118、cssom 2,502、css-transitions
3,254（过渡插值门）、css-conditional 1,424、css-animations 1,206、cssom-view
2,115、css-properties-values-api 1,287（registerProperty 门）、geometry 1,366、
css-borders 1,457、css-text-decor 1,214、css-logical 1,286、css-cascade 1,633、
CSS2 1,626、highlight-api 231、font-loading 236、nesting 112、mixins 527、
syntax 319、mediaqueries 1,673、size-adjust 209。
A 类合计 ~34k，其中无门依赖的解析/CSSOM 面约 20k 是近期最高性价比批次来源。

A 类首个候选——css-values 残余 3,380F 分桶（results-v10/css-values.json 逐案
名字归类，供下一批直接取用）：`abs()/sign()`+signed-zero used-value 等价
~250F（computed 数学管线延伸，specified 常量折叠已有）、`random()`/`calc-mix()`
tentative 新函数 ~325F、`attr()` 全类型 ~103F（值上下文取 DOM 属性）、
line-break ch 单元 ~194F 与 viewport-units ~84F（布局 used-value 面，归 B 类
策略处理）、minmax/round-mod-rem invalid 收尾 ~200F。

A 类次候选——css-fonts 残余 2,751F 分桶（.acceptance/css-v2-m3/sixteen/json
/css-fonts.json，11-05，163 页 2,251P/2,747F；周跑 2,345P 同量级）：font-family
custom-ident 语法（数字开头 ident 如 `0simple` 须拒，含 font 简写联动与 setter
面）~608F、font 简写全语法 computed（normal 关键字/行高 calc/system fonts）~615F
（font-valid + font-computed）、@font-feature-values 规则建模（CSSRule type 14）
~294F、font 简写序列化阻止门（font-variant 等置 initial/inherit 后 gCS font 返
''）~107F、@font-face 描述符面（CSSOM 遍历 `reading 'style'` 崩 = rule 面缺
type 5 + weight auto）~88F、size-adjust/format-specifiers/palette-values 等
长尾 ~500F。前三桶均为解析/CSSOM 语法面，预估可收 1.2–1.5k。

A 类第三候选——selectors 残余 2,116F 分桶（同目录 selectors.json，11-05，274 页
2,140P/2,110F；周跑 2,148P 同量级）：`:has()` invalidation set 机制
（invalidation/*-in-* 族 is/not/has 位置面）~970F——属样式失效集引擎架构面，
工程量大于纯语法批；heading.html 182F（待归类）；attribute-case CSSOM `i`
修饰符 ~50F、parse-anplusb An+B 微语法边界 ~48F——两个小语法批 ~100F 可速收。

A 类第四候选——css-anchor-position 残余 6,213F 分桶（同目录
css-anchor-position.json，11-05，253 页 7,457P/5,359F；周跑 7,455P 同量级）：
`position-area` 语法解析 1,800F + computed 序列化 633F（解析/序列化双面，
A 类高产出）；`@position-try` CSSOM 规则建模 265F；writing-modes 交互
1,296F 与位置落地几何（余量）——归 B 类布局面。A 类可收约 2,400–2,700。

**B 类 · 布局面（需引擎布局/绘制能力，按 grid G2 先例小步收敛）**：
css-grid 14,510、css-masking 6,278、css-shapes 5,836、css-backgrounds 5,739、
css-sizing 5,679、css-anchor-position 6,213（已 54%，剩余半是布局定位）、
css-transforms 5,367、css/motion 4,854、css-text 4,667、css-flexbox 3,895、
css-gaps 3,801、css-images 2,720、css-ui 2,199、filter-effects 2,822、multicol
1,518、position 1,357、overflow 1,040、box 869、lists 848、tables 612、pseudo
805、inline 751、scroll-snap 695、break 694、viewport 710、contain 405。
B 类合计 ~85k，首轮按 10–20% 收敛率估 8–17k，是长周期主战场。

**C 类 · 零通过 API 门面（需新引擎基建，暂缓）**：fill-stroke 409、rhythm 183、
ruby 99、forms 93、navigation 82、parser-api 71、paint-api 28、layout-api 13、
device-adapt 7、exclusions 4、image-animation 5、fetching 4、forced-color-adjust
14。合计 ~950，paint/layout/parser API 需要 worklet 型基建，先挂账。

**结构判断**：A+B+C 全收也只有 ~120k < 128.5k 缺口，且 B 类实际收敛率必然低于
100%（DC-1 定谳：运行面断言失败 ~139.5k 中 Chrome 同面过 95%）。**182,392 线
需要在 A 类全收之外，把 B 类布局面（grid/flexbox/anchor/sizing 等）推进到
Chrome 同面 60%+ 水平**——即数十个布局能力批。这超出 DC-3 单轮可达范围，属于
本 goal 与渲染流的长期交汇面；DC-3 的「同口径净增 153,000」按此路径分批逼近，
每批留五分账与不降核对。

## 6. 本轮边界

只读盘点：未改任何码面、未占用 v11–v13 测量号、未动 master.md（三链落定后由
验证轮统一入账）。周跑执行（v14）待三链落库、合并树 fmt/clippy/make test 全量
验证通过后进行。
