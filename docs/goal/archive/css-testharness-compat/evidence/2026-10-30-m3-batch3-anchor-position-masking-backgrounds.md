# M3 批次 3：css-anchor-position / css-masking / css-backgrounds 扩面入账（2026-10-30）

## 结论

M3 扩面批次 3 三目录过 M0 模板入账（top15 分母目录推进至 12/15）：

- **运行面（批次 3 三目录合计）435 案 / 14,440 子测试 / 5,593 Pass（38.7%）**。
- **十二目录统一基线**：53,293 子测试 / 15,176 Pass（28.5%）。
- css-anchor-position 通过率 40.6% 显著高于 typed-om——基础解析面已通，
  失败集中 `anchor-size()` **序列化参数序颠倒**（~2,500，ident 与 dimension
  交换输出）与 **锚定几何未实现**（offset 断言按静态布局回退 ~1,305）。
- css-masking / css-backgrounds 中小体量（61/121 run-face 案），失败主簇
  computed-style 探测（mask-*/background-* 长属性族）+ canonical 序列化。

## 基线（单进程 `--json`，test-guard 4G/8G/2400s）

| 目录 | 案 | 子测试 | P | F | TO | 通过率 | 墙钟 | 峰值 RSS |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| css-anchor-position | 253 | 12,811 | 5,205 | 7,602 | 4 | 40.6% | 1m12s | 487MB |
| css-masking | 61 | 544 | 121 | 422 | 1 | 22.2% | 19s | 346MB |
| css-backgrounds | 121 | 1,085 | 267 | 818 | 0 | 24.6% | 16s | 169MB |

run-face 案数与 runner 谓词预估逐目录一致（253/61/121）；三目录上游均无
`.any.js`（形态同试点两目录，无变体口径）。批内墙钟与批次 2 同受兄弟流
负载影响，计数维度不受影响。

## 通道接线（M0 模板复用，零脚本改动）

- runner 子命令 `testharness-css-anchor-position` /
  `testharness-css-masking` / `testharness-css-backgrounds`；Makefile 全量
  + `-one` target ×3；35 号 goal 脚本 DIRS 追加三目录。分片/parity 脚本
  经 `css-*` 规则映射零改动接入。

## 语料对账（上游 pin blob hash）

| 目录 | 上游 pin | 本地 | 内容漂移 | 处置 |
|---|---:|---:|---:|---|
| css/css-anchor-position | 653 blobs / 647 html | 648 文件 | **0 mismatch** | 深挖全量首拉（原空目录）；缺口 = 4 .yml 元数据 + 1 未引用 png |
| css/css-masking | 821 blobs / 644 html | 805 文件 | **0 mismatch** | 深挖全量首拉（原空目录）；缺口 = 12 .yml + 5 png（引用甄别见下） |
| css/css-backgrounds | 1,520 blobs / 1,168 html | 1,520 文件 | **1 页陈旧 → FORCE 单页重拉后一致** | 精选仓存量 1,520 文件与 pin 恰好同数但 `background-clip-color-ref.html` 内容漂移——批次 1 "文件数一致 ≠ 内容一致" 再证 |

- **GitHub API 限流**：未认证 60 req/h 下深挖列树 403（35 号脚本按设计
  优雅降级跳过）；`GITHUB_TOKEN=$(gh auth token)` 认证重跑后首拉完成。
  CI 侧 weekly 已带 token，本地 rally 轮须沿用认证环境变量。
- **缺口资产甄别**：真缺补拉 19 个（masking resources/support png 族 +
  css/support 共享 png + css-images swatch-red + images/blue-100x100.svg
  ——runner 绝对路径 `/images/*` 映射到 `wpt-data/images/*`）+
  `background-clip-color-ref.html` 陈旧页重拉。**误报弃拉**（上游树无此
  文件 / 模板串伪影）：`css/support/space_border_box.png` 族 3 png
  （`background_repeat_space_border_box.htm` 动态拼名，上游 404）、
  masking `support/green.png` 族 5 png（子目录页模板串，上游无）、
  `${IMAGE_URI}`/`invalid:`/`about:blank` 等负例与模板占位。

## 失败分簇（message 归一化：数字→N、串→'…'）

- **css-anchor-position（7,602 F / 108 簇）**：
  - `anchor-size()` 序列化参数序颠倒 **~2,500**（期望
    `anchor-size(--foo width)` 实得 `anchor-size(width --foo)`——custom
    ident 与 dimension 参数交换；canonical serialization 簇 2,498 全部
    此形态）——引擎可修单点，归 CSSOM 序列化域挂账；
  - 锚定几何未实现 **~1,305**（`assert_array_equals` offset 四元组按
    静态布局回退——anchor()/position-area 实际布局面缺口）；
  - position-area computed 探测 **632** + 泛型 assert_equals 1,707
    （inset/accepted-valued 解析长尾）。
  归口：**anchor positioning 序列化单点可修 + 定位布局面挂账**。
- **css-masking（422 F / 51 簇）**：computed 探测族（mask-image 48 +
  mask 32 + mask-repeat 23 + clip-path 22）+ 序列化族（169）+ 页脚本
  TypeError 21。归口：**mask-* 长属性 computed 值面**。
- **css-backgrounds（818 F / 83 簇）**：泛型序列化 228 + 探测族
  （background-image 49 + background-position 37 + position-x/y 48）+
  assert_true 59。归口：**background-* 长属性 computed/序列化长尾**。

## parity（分片 ≡ 单进程恒等判据）

全语料 parity（`make testharness-css-parity`，FILTER=目录前缀取全量）：

| 目录 | 案（single=sharded） | 判定 |
|---|---|---|
| css-anchor-position | 253=253 | **IDENTICAL** |
| css-masking | 61=61 | **IDENTICAL** |
| css-backgrounds | 121=121 | **IDENTICAL** |

DC-2 通道契约对三新目录成立；批次 2/3 连续六目录 parity 无一状态残留。

## 验证矩阵

| 项 | 结果 |
| --- | --- |
| 语料对账（git blob hash vs 上游 pin） | 三目录 0 mismatch（backgrounds 陈旧 1 页重拉后一致） |
| 缺口资产 | 真缺 19 个补拉 + 1 陈旧页；误报类甄别留档 |
| 单进程基线 ×3 目录 | 见基线表 |
| run-face 案数 vs 谓词预估 | 253/61/121 逐目录一致 |
| parity 全语料 | 见上节 |
| fmt / clippy（-p zero-wpt-runner --all-targets -D warnings） | 干净 |
| runner bin 单测 | 265/265（4G/proc 阈值下两连杀于 4.24GB 边际溢出——兄弟流负载 + 套件并行调度噪声，6G/proc 复跑 16.5s 全绿；本批未触碰 runtime_path 测试域） |

## 边界与遗留

- **anchor-size 序列化**：~2,500 子测试的参数序单点是批内最大可修簇，
  归 CSSOM 序列化域专项评估（修则同窗回归 rendering-compat reftest 门）。
- **WAAPI 归口**：masking/anchor-position 的 animations/ 子目录页依赖
  `/web-animations/testcommon.js`（wpt-data 无此目录——timing-animation
  goal 语料布局不同），失败按 goal 划界归口 WAAPI 底座，不入本账分母
  之外的补拉范围。
- **M3 剩余**：top15 其余 3 目录——css-grid（15,841 分母 / 上游 3,359
  html）与 css-text（5,714 / 2,811 html）重量级独立轮次 +
  css-transforms（5,516 / 1,238 html）；motion（4,959 / 148 html）可随
  transform 轮顺带。
