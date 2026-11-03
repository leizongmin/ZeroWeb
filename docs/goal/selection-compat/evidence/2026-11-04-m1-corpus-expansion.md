# M1 / DC-1 语料扩面 — selection 全量深拉核对（2026-11-04）

## 结论

DC-1 成立：selection/ 语料在 pin rev `315976933870b34d6ea30e3f6643403edae678ba`
全量就位（2026-10-30 深拉落盘，本轮逐 blob 哈希核对），外加 8 个 pin 漂移
文件对齐上游 HEAD `43f66fcb72f7ef1b1cacdfc97e789ba5e34f92fb`（2026-11-03）。

## 语料盘点（对上游 pin 树逐 blob 核对）

- 上游 pin 树 `selection/` 共 195 blob：182 .html + 2 .htm + 7 .js + 4 .yml
  （git trees API `?recursive=1`，truncated=false）。
- 本地 `tests/wpt-runner/wpt-data/selection/` 191 个非空文件（182 .html +
  2 .htm + 7 .js）**git blob SHA 逐一比对 0 mismatch**（`sha1("blob <len>\0" + content)`）。
- 缺 4 个 .yml（META.yml / WEB_FEATURES.yml ×3）——工具元数据，fetch 脚本
  扩展名分流不拉，与运行无关。

## pin 漂移对齐（coverage universe 驱动）

coverage-2026-10-31 快照 universe（wpt.fyi aligned stable 三家并集）selection
行 194 文件，其中 **14 项在 pin 树不存在**（上游 pin 后新增）。照 net-api
M2-S12 先例逐文件对齐 HEAD 并记录 rev（`fetch-selection-subset.sh`
PIN_DRIFT_CASES 块，升级重拉时按当期 master 校正）：

| universe 路径 | 三家子测试权重 | 处置 |
|---|---:|---|
| collapse-common-ancestor-1/2/3.html | 0/0/0 | 已拉（HEAD） |
| document-selectionchange.html | 5 | 已拉（HEAD） |
| longpress-selection.html | 1 | 已拉（HEAD） |
| selection-range-painting-after-splitText.html（+?splitAt=1..8 变体 6 项） | 全 0 | 已拉（HEAD） |
| crashtests/selection-modify-line-from-optgroup-...html | 0 | 已拉（HEAD） |
| shadow-dom/tentative/Selection-deleteFromDocument-around-shadow.html | 33 | 已拉（HEAD） |
| idlharness.window.html | 112 | **不拉**——pin 树内已有 idlharness.window.js，runner 按 clipboard/timing/net-api/compression idlharness 先例记账排除（依赖 /resources/WebIDLParser.js 与 /interfaces/*.idl build 资产，语料不含） |

pin 漂移面合计子测试权重 151，其中 idlharness 112 记账排除、可清零面 39
（deleteFromDocument-around-shadow 33 + document-selectionchange 5 +
longpress 1）；其余 12 项 0 权重（不进分母子测试）。

## universe ↔ 语料映射

- 194 universe 文件中 180 项在语料有 base 文件；14 项 pin 漂移（上表）。
- 23 个 `?query` 变体 universe 项（8 个 base 文件）：7 个 base 文件在 pin 树
  且均声明 `<meta name="variant">`（runner case_variants js-dom R329 承载）；
  6 个 `?splitAt` 变体属 pin 漂移（base 文件已按 HEAD 补齐，变体由 meta
  声明承载）。
- 語料 34 个 crashtests .html、12 个 `*-ref.html`、1 个 `dir-manual.html`
  均不在 universe 或 0 权重——runner 通道按 skip 规则排除（见 M1 基线
  evidence 导入清单）。

## runner 通道扩面（DC-1 配套）

`tests/wpt-runner/src/testharness.rs`：

- `SELECTION_TEST_SUBDIRS`：`["selection", "selection/textcontrols", "editing",
  "editing/other"]` → `["selection", "editing", "editing/other"]`——`selection/`
  根改**递归扫描**（二期扩面），`editing/` 两目录保持一期 flat 口径（锚面
  相对路径不漂移，逐 subtest 零回归对照基础不变）。
- 新增 `selection_case_skipped`（`*-manual.html` / `*-ref.html` /
  `selection/crashtests/` 前缀，uievents 通道同款先例）、
  `collect_selection_dir_cases`（递归，`collect_uievents_dir_cases` 同构）、
  `collect_selection_flat_dir_cases`（一期 flat 逻辑抽出）与
  `push_selection_case_results`（单案执行 + variant meta 展开，两收集器共享）。
- clippy 全绿（`make guarded-clippy`）；递归发现、子目录用例、variant 展开
  均经 FILTER 抽验（collapse-common ×3、shadow-dom/tentative、
  textControl 变体族 4 变体全跑）。
