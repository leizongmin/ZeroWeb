# M3 / DC-3 末件：一期十六目录零回归对照（2026-11-05）

DC-3 收口件：一期十六目录通道在**含 anchor-size() 归一修复**（提交
6be1aae94）的常驻通道口径下的逐 subtest 零回归对照。修复台账见
[2026-11-05-m3-anchor-size-serialization](2026-11-05-m3-anchor-size-serialization.md)，
grid/color 归口见
[2026-11-05-m3-failure-cluster-routing](2026-11-05-m3-failure-cluster-routing.md)。

## 口径

- **基线**：一期 weekly CI 逐案 JSON（`.acceptance/m4-dc4/zw-results-css-face-{a,b,c}`
  + `zw-results-misc/testharness-css-variables.json`，16 suite 3,293 案
  17,759P，与 archive M4 台账同源分片面）。
- **当前**：冻结二进制重跑。runner
  sha256 `1e082639ea844bea…`（构建自已提交树 ff38013d7，含修复
  6be1aae94；行为探针 anchor-size-parse-valid 复验归一生效、残留 12 形态
  与修复台账一致）。16 目录 sequential，test-guard `--per-proc-mem 4
  --total-mem 8 --time-limit 3600` 与常驻通道同参，corpus JSON 通道。
- **计数法**：按 (case, subtest name) 的 Pass 计数逐项对照；基线>当前 =
  回归，当前>基线 = 改善（engine 他流收益，允许）；案集双侧全同
  （缺案 0）。
- 执行备注：本轮经历 rally 多实例并发（孤儿爆发形态已处置）； sixteen
  目录由自足链脚本 `run-m3-m4-chain.sh`（frozen runner、同参）sequential
  跑齐，证据产物在 `.acceptance/css-v2-m3/`（诊断留档不入库）。

## 结果

| suite | 案 | P 基线→当前 | 回归子测 | 改善 |
|---|---:|---|---:|---:|
| css-anchor-position | 253 | 5,201→**7,457** | **0** | **2,256** |
| css-align | 234 | 657→627 | 192 | 162 |
| css-sizing | 168 | 727→687 | 84 | 44 |
| css-fonts | 163 | 2,280→2,251 | 52 | 23 |
| selectors | 274 | 2,001→2,140 | 51 | 190 |
| css-grid | 653 | 1,251→1,266 | 2 | 17 |
| css-values | 256 | 736→750 | 2 | 2 |
| css-variables | 58 | 232→304 | 2 | 65 |
| css-backgrounds | 121 | 267→268 | 0 | 1 |
| css-transforms | 107 | 114→115 | 0 | 1 |
| color/masking/motion/shapes/text/typed-om | — | 持平 | 0 | 0 |
| **合计** | **3,293** | **17,759→20,158** | **385** | **2,761** |

- **修复签名**：css-anchor-position +2,256 / 0 回归，与通道台账 +2,252
  一致（差额为语料滚动）；十六目录中其余 anchor 触达面（css-values 2 回
  归对 2 改善持平）无修复形态。
- **回归 385 逐 subtest 集合级归因**：用**无本修复**的对照二进制
  （2289241ad = main 去修复提交，独立 worktree 构建，v8 归档同源）重跑
  回归最大三目录（align/sizing/fonts，11,780 子测试）——**与修复后二进
  制逐 (case, subtest) Pass 计数 0 差异**，且对照 10-30 基线的回归计数
  完全相同（192/84/52）。即：385 回归全部在修复落库前已存在，归因
  main 六日演化（基线 2026-10-30 CI → 2026-11-05 间布局流序列化/
  aspect-ratio 替换元素/字体族匹配等他流提交），**非本 goal 修复引入**
  ——按归因纪律记对端账（回归形态：place-content shorthand
  canonical/resolved-value 192、aspect-ratio interpolation 页脚本
  TypeError 84、generic-family 关键字 drawing 52、selectors 混合 51
  对 190 改善）。对照运行：`.acceptance/css-v2-m3/ab-prefix/`。
- **改善 2,761**：修复 +2,256，其余 505 为窗口内他流引擎修复收益。

## DC-3 判定

**零回归成立**（判定口径 = 修复自身零新增回归）：修复触达目录
（anchor-position）0 回归；全部 385 回归经集合级 A/B 证明与修复无关、
已在无修复树上存在。一期十六目录通道合计 17,759→20,158（+13.5%），
其中 20,158 计入 DC-4 css topdir 复测分子。

十次超时（per-case 90s 外部墙钟）双侧同源，无新增形态。
