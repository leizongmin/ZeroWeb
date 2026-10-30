# M3 批次 5：css-grid / css-text 扩面入账——top15 收尾（2026-10-30）

## 结论

M3 扩面批次 5 两目录过 M0 模板入账，**top15 分母目录全部推进完毕（15/15）**：

- **运行面（批次 5 两目录合计）1,007 案 / 10,733 子测试 / 2,351 Pass（21.9%）**。
- **十六目录统一基线**：65,157 子测试 / 17,747 Pass（27.2%）。
- css-grid 通过率 16.2%——**布局探测簇 3,280**（check-layout-th data-*/
  client 尺寸断言，grid 布局引擎未实现的直接账面）+ computed 序列化枚举
  893（repeat(auto-fill/auto-fit) 展开形态不在允许集）+ computed 探测 483
  （grid-template-columns/rows 为主）。
- css-text 通过率 36.6%——断行位置 approx 簇 543（行首行尾禁则/Unicode
  断行算法）+ 泛型 assert_equals 476（二级子测名归因：~390 为
  `Selection.toString()` × text-transform 选区变换文本）+ 布局探测 195。
- 大语料拉取通道升级：jsdelivr/raw 突发限流（403/连接超时）后改走
  **git blobs API**（树清单 SHA→内容直取，认证额度 5,000/h，3,338 文件
  零失败约 9 分钟）——幂等快路径漏差补拉的第三条通道，CI 接线前脚本化
  候选。

## 基线（单进程 `--json`，test-guard 4G/8G/2400s）

| 目录 | 案 | 子测试 | P | F | TO | 通过率 | 墙钟 | 峰值 RSS |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| css-grid | 653 | 7,716 | 1,247 | 6,393 | 76 | 16.2% | 2m49s | 585MB |
| css-text | 354 | 3,017 | 1,104 | 1,912 | 1 | 36.6% | 1m21s | 508MB |

run-face 案数与 runner 谓词预估逐目录一致（653/653、354/354）；两目录均
无 `.any.js`（形态同试点，无变体口径）。css-grid 上游 3,307 html 仅 653
页 testharness 形态（reftest 面归 rendering-compat）；76 个 Timeout 集中
在布局探测页（check-layout-th 断言在未实现布局上等待）。

## 通道接线（M0 模板复用，零脚本改动）

- runner 子命令 `testharness-css-grid` / `testharness-css-text`；Makefile
  全量 + `-one` target ×2；35 号 goal 脚本 DIRS 追加两目录。两目录均带
  `css-` 前缀，分片/parity 脚本 `css-*` 规则零改动接入。

## 语料对账（上游 pin blob hash）

| 目录 | 上游 pin | 本地（首拉前） | 内容漂移 | 处置 |
|---|---:|---:|---|---|
| css/css-grid | 3,386 blobs / 3,359 html | 89 文件 | 缺 3,311 + 陈旧 2（support png） | 幂等快路径跳过 → **git blobs API 定向补拉 3,313，复核 0 mismatch / 0 missing** |
| css/css-text | 2,998 blobs / 2,811 html | 2,994 文件 | 缺 4 + 陈旧 45 | 补拉 25（4 缺 + 21 陈旧页——19 个 `seg-break-transformation-*.tentative.html` + tab-stop-with-float 对 + 2 text-transform 页）；**24 个陈旧为 `WEB_FEATURES.yml`（不在抓取扩展名契约内，剔除记账）**；复核 0 mismatch（契约内）/ 0 missing |

- **CDN 限流实录**：xargs -P 8 并发 jsdelivr 约 40 个请求后即 403
  （raw.githubusercontent 同步连接超时，冷却 45s 不恢复）——两 CDN 对
  突发并发的容忍度远低于顺序拉取。api.github.com 不受限流影响，git
  blobs API（`GET /repos/:o/:r/git/blobs/<sha>`，base64 内容）按树清单
  SHA 直取是当前最稳通道。
- "文件数一致 ≠ 内容一致"第四次实证（css-text 本地 2,994 vs pin 2,998
  仅差 4，blob 对账实有 21 页契约内内容陈旧）。

## 资产引用甄别

真缺补拉 5 个（全部 run-face 页引用，pin 存在，contents API 拉取）：
`/fonts/adobe-fonts/CSSFWOrientationTest.otf`（34KB）、
`/fonts/noto/cjk/NotoSansCJKjp-Regular-subset-halt.otf`（45KB，
text-transform 引用）、`/images/green-100x50.png`、
`/css/support/1x1-lime.png`、`/css/support/blue-100.png`（后两者 grid
页引用；`/fonts/*`、`/images/*` 绝对路径映射 `wpt-data/fonts`、
`wpt-data/images`）。弃拉类：

- `../interpolation/resources/interpolation-test.js`（1 页）——解析目标
  不在 pin 树（transforms 同款上游破损引用，typed-om comparisons.js
  先例）；
- `/resources/testdriver*.js`——runner 装配层置空注入，无需落盘；
- reftest 域字体引用（`/fonts/noto/*` 族 10+、`/fonts/CanvasTest*` 等
  ——reftest 页渲染资产，rendering-compat 划界）；
- 误报：meta author 邮箱（grid 7 / text 5）、`url(dummy)` 负例字面量。

## 失败分簇（message 归一化：数字→N、串→'…'）

- **css-grid（6,393 F / 385 簇）**：
  - 布局探测 **3,280**（check-layout-th `data-expected-*`/`data-offset-*`
    + client 尺寸）——grid 布局引擎未实现，值按非 grid 布局回退；76 个
    Timeout 同源（断言等待）。归口：**grid 布局面挂账**（engine 专项，
    体量为 top15 各目录之最）；
  - computed 序列化枚举 **893**（gridTemplateColumns 731 +
    gridTemplateRows 151 + 其他 11）——`repeat(auto-fill/auto-fit)`
    computed 展开形态不在断言允许集；
  - computed 探测 **483**（grid-template-columns 241 + rows 137 +
    auto-columns/rows 各 26 + grid-row/column 各 ~20 + grid-lanes 14）；
  - canonical 序列化 **375** + 泛型 assert_equals 1,062 + assert_true 210。
- **css-text（1,912 F / 61 簇）**：
  - 断行位置 approx **543**（行首行尾禁则 / Unicode UAX #14 断行——
    精确断行位置与引擎当前折行策略差异）；
  - 泛型 assert_equals **476**（二级归因：~390 为 `Selection.toString()`
    × text-transform 选区变换文本——选区 API 与文本变换交互面）；
  - 泛型 assert_true **331**（Unicode 空白/可断字符判定）；
  - 布局探测 **195** + canonical 序列化 78（hyphenate-limit-chars 为主）+
    computed 探测长尾（text-autospace 32 / white-space 28 / text-wrap 25
    / text-spacing 16 / tab-size·text-fit·text-indent·text-transform·
    letter-spacing 各 ~12）。

归口：**grid 布局引擎（3,280+76）与断行/选区文本面（543+390）为两目录
最大可修域**，均 engine 专项挂账；CSSOM 解析/序列化长尾（~1,700）延续
既有归口。

## parity（分片 ≡ 单进程恒等判据）

| 目录 | 案（single=sharded） | 判定 |
|---|---|---|
| css-grid | 653=653 | **IDENTICAL** |
| css-text | 354=354 | **IDENTICAL** |

DC-2 通道契约对两新目录成立；批次 2–5 连续十目录 parity 无一状态残留。

## 验证矩阵

| 项 | 结果 |
| --- | --- |
| 语料对账（git blob hash vs 上游 pin） | grid 3,386 blobs 0/0；text 2,974 契约内对齐 + 24 WEB_FEATURES.yml 契约外记账 |
| 缺口资产 | 真缺 5 个补拉；弃拉类留档（上游破损/置空注入/reftest 字体/误报） |
| 单进程基线 ×2 目录 | 见基线表 |
| run-face 案数 vs 谓词预估 | 653/653、354/354 逐目录一致 |
| parity 全语料 | 见上节 |
| fmt / clippy（-p zero-wpt-runner --all-targets -D warnings） | 干净 |
| runner bin 单测 | 267 passed / 0 failed（test-guard 6G/12G，15.2s） |

## 边界与遗留

- **top15 收口**：15/15 目录全部入账（css-values/selectors/css-variables/
  color/align/sizing/typed-om/fonts/shapes/anchor-position/masking/
  backgrounds/transforms/motion/grid/text）。css topdir 其余 74 个二级
  目录（coverage 分母 ~24%）不在 top15，M4 后按需评估。
- **M4（DC-4）**：coverage 复测收口现在可做——十六目录 run-face 面已定，
  weekly misc 分片接线（fetch 入口 + 时间预算）与 wpt-suites.csv 汇总行
  是剩余实质。
- **拉取通道债务**：git blobs API 补拉须脚本化进 35 号脚本（本地 rally
  轮已三写三改；CI 接线前收口，fonts 批次 2 遗留项合并处理）。
