# M0 对账甄别 + 批次清单 + DC-4 数值门（2026-11-04）

DC-1 三件套：①剩余目录逐目录甄别（本文 §2/§3）；②语料 pin 对齐 sync
（§4，`tests/wpt-runner/scripts/goals/41-css-testharness-compat-v2.sh`
首跑）；③DC-4 数值门按决策规则落定（§5，已回填控制面）。

## 1. 数据源与方法

- **testharness 分母面**：coverage 快照 `docs/compat/wptfyi/coverage-2026-11-02.json`
  groups（wpt.fyi aligned stable 三家并集，reftest 文件无子测试不入口径）。
  css topdir 全量 **195,723 子测试 / 35,555 文件**；一期十六目录已入账
  **129,754 子测试 / 17,923 Pass**（9.16%）；未导入池 **74 suites /
  65,969 子测试**（缺口的其余 111,831 属已导入目录内的失败/未覆盖子测试，
  归引擎修复账不入本 goal 扩面）。
- **上游形态**：git trees API（pin `315976933`，认证列目录）递归清单 74
  目录全量 blob（原始清单 `.acceptance/css-v2-m0/listings/`，diagnostics
  不入库）；扩展分布见 §3 表内括号注记。
- **甄别定性依据**：目录域知识（spec 面 → 引擎能力域）+ 一期十六目录实测
  通过率带 + 上游形态（.xht 占比、helper 资产形态）；不做逐目录干跑
  （干跑基线是 M1/M2 每批次的固定动作，届时校验甄别假设）。

## 2. 未导入池构成（8 桶，74 suites / 65,969 子测试）

| 桶 | 目录数 | 子测试池 | 定性 |
|---|---:|---:|---|
| B1 值/属性解析序列化 | 24 | 23,882 | 导入（引擎声明值/序列化强域） |
| B2 规则/条件/级联 | 7 | 9,751（CSS2 按 html 面 ~300 计入门） | 导入 |
| B3 混合布局 | 13 | 19,286 | 导入（解析份额入账；布局探测面失败归口布局账） |
| 挂账 WAAPI 依赖 | 2 | 4,867 | 不导入（timing-animation-compat 已收口域，失败簇归 WAAPI 底座账） |
| 挂账 API 未实现 | 6 | 3,125 | 不导入（typed-om 划界同款：如实标记不进数值门） |
| 挂账 协调面 | 2 | 4,213 | 不导入（cssom-view/geometry 与 rendering-compat 视口/几何语义双域，run-rules §9 碰头前置） |
| 挂账 尾部 | 14 | 845 | 不导入（<400 子测试零簇；batch-3 视产能） |
| 渲染面 / 0 子测试 | 9 | 0 | 无 harness 面可入账（reftest 归 rendering-compat） |

## 3. 逐目录甄别表

分母列 = coverage-2026-11-02；Pass 列 = 当前 zw 通过（未导入目录为 0，
少数例外注记）。

| 批次 | 目录 | 分母文件 | 分母子测试 | zw Pass |
|---|---|---:|---:|---:|
| B1 | css/cssom | 303 | 4,456 | 0 |
| B1 | css/css-images | 523 | 3,595 | 0 |
| B1 | css/filter-effects | 428 | 3,007 | 0 |
| B1 | css/css-ui | 1,077 | 2,457 | 0 |
| B1 | css/css-borders | 312 | 1,601 | 0 |
| B1 | css/css-logical | 92 | 1,448 | 0 |
| B1 | css/css-text-decor | 374 | 1,376 | 0 |
| B1 | css/css-box | 151 | 1,063 | 0 |
| B1 | css/css-pseudo | 362 | 865 | 0 |
| B1 | css/css-display | 259 | 567 | 0 |
| B1 | css/css-mixins | 57 | 618 | 0 |
| B1 | css/css-syntax | 52 | 444 | 0 |
| B1 | css/fill-stroke | 11 | 409 | 0 |
| B1 | css/css-shadow | 208 | 377 | 0 |
| B1 | css/css-size-adjust | 5 | 215 | 0 |
| B1 | css/css-content | 80 | 253 | 0 |
| B1 | css/css-will-change | 53 | 173 | 0 |
| B1 | css/css-color-adjust | 41 | 171 | 0 |
| B1 | css/css-easing | 15 | 167 | 0 |
| B1 | css/compositing | 81 | 167 | 0 |
| B1 | css/css-counter-styles | 271 | 158 | 0 |
| B1 | css/css-nesting | 47 | 126 | 0 |
| B1 | css/css-color-hdr | 4 | 116 | 0 |
| B1 | css/css-env | 11 | 53 | 0 |
| B2 | css/css-conditional | 468 | 3,258 | 0 |
| B2 | css/mediaqueries | 95 | 1,768 | 0 |
| B2 | css/css-cascade | 164 | 1,750 | 0 |
| B2 | css/CSS2 | 6,389 | 2,548 | 0 |
| B2 | css/css-forms | 26 | 93 | 0 |
| B2 | css/css-link-params | 9 | 93 | 0 |
| B2 | css/css-font-loading | 34 | 241 | 0 |
| B3 | css/css-gaps | 641 | 4,157 | 0 |
| B3 | css/css-flexbox | 1,429 | 4,782 | 0 |
| B3 | css/css-multicol | 723 | 1,614 | 0 |
| B3 | css/css-position | 394 | 1,507 | 0 |
| B3 | css/css-overflow | 940 | 1,200 | 0 |
| B3 | css/css-tables | 353 | 939 | 0 |
| B3 | css/css-lists | 236 | 972 | 0 |
| B3 | css/css-inline | 436 | 857 | 0 |
| B3 | css/css-scroll-snap | 216 | 845 | 0 |
| B3 | css/css-viewport | 145 | 780 | 0 |
| B3 | css/css-break | 1,310 | 717 | 0 |
| B3 | css/css-contain | 593 | 497 | 0 |
| B3 | css/css-writing-modes | 1,257 | 419 | 0 |
| 挂账 | css/css-transitions (WAAPI) | 144 | 3,437 | 0 |
| 挂账 | css/css-animations (WAAPI) | 210 | 1,430 | 0 |
| 挂账 | css/css-properties-values-api (API) | 125 | 1,393 | 0 |
| 挂账 | css/css-view-transitions (API) | 528 | 1,444 | 0 |
| 挂账 | css/css-highlight-api (API) | 135 | 176 | 0 |
| 挂账 | css/css-parser-api (API) | 1 | 71 | 0 |
| 挂账 | css/css-paint-api (API) | 118 | 28 | 0 |
| 挂账 | css/css-layout-api (API) | 153 | 13 | 0 |
| 挂账 | css/cssom-view (协调面) | 250 | 2,638 | 0 |
| 挂账 | css/geometry (协调面) | 31 | 1,575 | 0 |
| 挂账 | css/css-page (尾部) | 263 | 166 | 0 |
| 挂账 | css/css-ruby (尾部) | 179 | 90 | 0 |
| 挂账 | css/css-scroll-anchoring (尾部) | 86 | 98 | 0 |
| 挂账 | css/css-overscroll-behavior (尾部) | 12 | 109 | 0 |
| 挂账 | css/css-scrollbars (尾部) | 57 | 83 | 0 |
| 挂账 | css/css-navigation (尾部) | 26 | 82 | 0 |
| 挂账 | css/css-rhythm (尾部) | 35 | 183 | 0 |
| 挂账 | css/css-device-adapt (尾部) | 7 | 7 | 0 |
| 挂账 | css/css-forced-color-adjust (尾部) | 4 | 14 | 0 |
| 挂账 | css/css-image-animation (尾部) | 23 | 5 | 0 |
| 挂账 | css/css-exclusions (尾部) | 12 | 4 | 0 |
| 挂账 | css/fetching (尾部) | 1 | 4 | 0 |
| 挂账 | css/css-round-display (尾部，coverage 无组) | — | 0 | 0 |
| 挂账 | css/css-speech (尾部，coverage 无组) | — | 0 | 0 |
| 渲染面 | css/CSS1 / WOFF2 / reference / printing / css-namespaces / css-style-attr / css-zoom / css-motion-path / css-gcpm / css-painting | — | 0 | 0 |

合计核验：23,882 + 9,751 + 19,286 + 4,867 + 3,125 + 4,213 + 845 + 0 =
65,969 ✓（与快照未导入池分毫不差）。

**形态要点**：

- `.any.js` / `.worker.js` 形态几乎为零（仅 filter-effects 1 个
  `.any.js`、geometry 2 个 `.worker.js` 均不在导入批）——css 面 harness
  语料是 `.html` 直跑形态，runner 现有 run-face 管道全覆盖，无 window
  变体/worker 通道需求。
- `.xht`（XML 文档）是结构性缺口：CSS2 **10,501 xht**（占其上游 blob
  81%，testharness 分母大头在 xht 面）、css-writing-modes 1,068、
  css-display 235、css-multicol 216。runner 只收 `.html/.htm`（XML 执行
  路径缺位，一期 22 页挂账延续）——CSS2 数值门按 html 面折算 ~300，其余
  xht 面挂账（XML 路径立项前不可达）。
- CSS2 上游 blob 12,902（html 818 + xht 10,501 + css 222 + 资产 1,361）
  ——数值分母 6,389 文件是 wpt.fyi 口径（含 xht），导入批只拉 html/css
  扩展面（sync 脚本扩展分流自动限定）。

## 4. 语料 pin 对齐 sync（DC-1 第二件）

- 脚本：`tests/wpt-runner/scripts/goals/41-css-testharness-compat-v2.sh`
  （45 项 DIRS = B1 24 + B2 7 + B3 13 计 44 批次目录 + css/support；`git trees recursive`
  逐目录清单 → 本地缺失并行拉取（jsdelivr→raw 双源回退，SYNC_PAR=6）→
  blob hash 对账不符定向重拉（旧文件替换成功前不删，#31）→ missing=0
  逐目录对账）。
- 与 35 号差异：v2 池 24 目录已含一期 rendering 拉取的 reftest 存量页，
  幂等快路径会整体漏拉——sync 模式逐文件对账解决；空目录同一路径全量
  首拉。
- 首跑记录（2026-11-04 启动，2026-11-05 终局）：见 §6 跑批台账
  （终局轮日志 `.acceptance/css-v2-m0/sync-20261105-000117.log`，
  diagnostics 不入库；台账表已录入本文件）。
- 幂等性：重跑只补缺失/不符文件；`resources/check-layout-th.js` 根级
  helper 与 35 号同款点拉。

## 5. DC-4 数值门决策（决策规则分支判定 + 公式）

**分支判定**：未导入池 65,969 < 100k → **降级为按簇收尾**：只导入
B1/B2/B3（高密度 + 导入即得/引擎可修簇），WAAPI/API 未实现/协调面/尾部
挂账具名，富余产能转 rendering-compat 协同面（§7）。

**门公式**（同 ≥100k 分支的「地址池 × 一期同类簇通过率」口径，一期实测
率先行、导入批干跑校验在后）：

| 批次 | 入门池 | 一期同类簇率（快照口径实测带） | 门增益 |
|---|---:|---|---:|
| B1 值/序列化 | 23,882 | 7%（parsing/序列化簇带 4.3%–50%，取下缘） | 1,672 |
| B2 规则/级联 | 7,503（CSS2 折 html 面 300） | 7% | 525 |
| B3 混合布局 | 19,286 | 2%（grid 7.9% / motion 2.1% / transforms 2.1% / masking 1.9%，取下缘） | 386 |
| **合计** | | | **2,583** |

**DC-4 数值门：css topdir 通过子测试 ≥ 20,500**（9.16% → ≥ 10.47%，
按 coverage-2026-11-02 分母 195,723 计；M4 复测时分母随上游滚动则同式
重算：门 = 基线 17,923 + 2,583 的绝对子测试数不随分母变）。

- 全量头条不回落（DC-4 另一半判据）维持既有口径。
- 一期十六目录零回归由 DC-3 分母逐 subtest 对照保障，不在门公式内。
- **校准风险与前置校验**：门增益按一期率先行取下缘；M1 首批（B1 前 6
  目录）干跑基线若实测率 < 3%（B1），触发门复评——复评证据落 evidence
  并在控制面留痕，不允许静默改门。

## 6. M0 跑批台账（2026-11-04～05，终态）

- 上游清单：74 目录 trees 全量（`.acceptance/css-v2-m0/listings/`，
  `summary.tsv` 汇总；diagnostics 不入库，计数已录 §3 表）。
- 语料 sync 终态：**45 目录全部 missing=0**（`== sync 结果：45 目录
  missing=0，0 目录有缺失 ==`，`resources/check-layout-th.js` 点拉在册）。
  扩展过滤面（html/htm/css/headers/js/json/xml/xhtml/svg）在册 blob
  **18,031**；本轮新拉 **6,963**（B1 837 + B2 203 + B3 5,923，前三
  css-break 1,368 / css-flexbox 1,212 / css-gaps 788）；blob-hash 不符
  定向重拉 **33**（css-text-decor 16、css-flexbox 10、css-content/
  css-cascade 各 3、css-pseudo 1）——一期 rendering 存量页随上游滚动
  过期，替换前旧文件保留（#31）。
- 执行形态：幂等续拉跨三轮（前轮中断落盘文件自动对账通过，B1 前六目录
  新拉 0 属正常）；jsdelivr CDN 403 突发由 raw.githubusercontent 权威
  回退承接，零目录失败。日志 `.acceptance/css-v2-m0/sync-20261105-000117.log`
  （diagnostics 不入库）。
- 修正留痕：首版脚本 B3 块漏 `css/css-lists`（§3 表有、脚本 12 目录≠13），
  终局轮前补入并整目录首拉 350 blob；「44 目录（含 css/support）」口径
  同步修正为 **44 批次目录 + css/support = 45 项**。
- 逐目录 missing=0 台账（列 = 上游过滤面 blob / 终局轮新拉 / 定向重拉 /
  missing；持久证据，不依赖 diagnostics 日志）：

| 目录 | 上游 | 新拉 | 重拉 | missing |
|---|---:|---:|---:|---:|
| css/cssom | 241 | 0 | 0 | 0 |
| css/css-images | 768 | 0 | 0 | 0 |
| css/filter-effects | 606 | 0 | 0 | 0 |
| css/css-ui | 1561 | 0 | 0 | 0 |
| css/css-borders | 250 | 0 | 0 | 0 |
| css/css-logical | 101 | 0 | 0 | 0 |
| css/css-text-decor | 615 | 0 | 16 | 0 |
| css/css-box | 179 | 3 | 0 | 0 |
| css/css-pseudo | 579 | 2 | 1 | 0 |
| css/css-display | 157 | 157 | 0 | 0 |
| css/css-mixins | 50 | 50 | 0 | 0 |
| css/css-syntax | 70 | 70 | 0 | 0 |
| css/fill-stroke | 12 | 12 | 0 | 0 |
| css/css-shadow | 225 | 225 | 0 | 0 |
| css/css-size-adjust | 5 | 5 | 0 | 0 |
| css/css-content | 132 | 0 | 3 | 0 |
| css/css-will-change | 69 | 69 | 0 | 0 |
| css/css-color-adjust | 57 | 57 | 0 | 0 |
| css/css-easing | 15 | 15 | 0 | 0 |
| css/compositing | 156 | 156 | 0 | 0 |
| css/css-counter-styles | 496 | 2 | 0 | 0 |
| css/css-nesting | 54 | 0 | 0 | 0 |
| css/css-color-hdr | 4 | 4 | 0 | 0 |
| css/css-env | 10 | 10 | 0 | 0 |
| css/css-conditional | 449 | 7 | 0 | 0 |
| css/mediaqueries | 108 | 108 | 0 | 0 |
| css/css-cascade | 162 | 1 | 3 | 0 |
| css/CSS2 | 1153 | 7 | 0 | 0 |
| css/css-forms | 36 | 36 | 0 | 0 |
| css/css-link-params | 4 | 4 | 0 | 0 |
| css/css-font-loading | 40 | 40 | 0 | 0 |
| css/css-gaps | 788 | 788 | 0 | 0 |
| css/css-flexbox | 1956 | 1212 | 10 | 0 |
| css/css-multicol | 725 | 269 | 0 | 0 |
| css/css-position | 487 | 353 | 0 | 0 |
| css/css-overflow | 1106 | 9 | 0 | 0 |
| css/css-tables | 419 | 238 | 0 | 0 |
| css/css-lists | 350 | 350 | 0 | 0 |
| css/css-inline | 378 | 378 | 0 | 0 |
| css/css-scroll-snap | 221 | 221 | 0 | 0 |
| css/css-viewport | 224 | 224 | 0 | 0 |
| css/css-break | 1368 | 1368 | 0 | 0 |
| css/css-contain | 811 | 0 | 0 | 0 |
| css/css-writing-modes | 814 | 513 | 0 | 0 |
| css/support | 20 | 0 | 0 | 0 |

## 7. 布局簇地址池测量 → layout goal 分档触发

划界预置分档：「grid/flexbox 布局探测簇若定性为布局引擎缺口……M0 实测
布局簇地址池 > 10k 子测试时独立 layout goal 立项」。

- **实测**：布局耦合目录（B3 全部 13 目录 + 一期已入账 grid 探测簇
  3,280）子测试池 **22,566**；扣解析份额（按 30% 上限折算）后布局探测
  地址池下限仍 **> 15k > 10k** → **触发条件成立**。
- 归口动作：B3 照常导入入账（本 goal 修到序列化/声明值层为止，探测面
  失败不就地修）；独立 layout goal 立项建议与测算数字已 Lark 通报用户，
  goal 文档待用户拍板后立项（不在本 goal 控制面内自建）。
- B3 探测面失败簇的账本归口默认记 rendering-compat 控制面（一期 grid
  探测簇同款），M1 干跑分簇后按实际簇形修正。
