# Missing-Asset 导入 Turnkey Handoff（R590 hand-off，R596 mechanism-verified）

> **目的**：为并行 code agent 提供一条「复制即可执行」的 missing-asset 导入流程，消除反复 stall 的摩擦（session 1 曾把文件误落 repo-root `css/` 而非 wpt-data，session 2-3 仅落 css/support/ 2 文件即停）。本 doc 由 doc-maintenance agent 产出（R597），资产清单经 R590 静态扫描 + R596 重扫实证（排除 intentional broken-image + 已落盘的 css/support/60x60-green/red.png）。

## 背景（一句话）

R590 发现 **63 个 WPT test 文件引用 ≥1 缺失 support 资产**（writing-modes 38 / css-fonts 11 / css-flexbox 9 / css-text 4 / css-multicol 1），导致 DC-14 双症状：① **false-pass**（test+ref 双侧同破无图→0.00% 假通过，污染 selfsource）② **real divergence**（缺图致 test/ref 发散）。**主要价值 = DC-14 metric credibility**（消除 false-pass）+ 次要 modest yield。R596 实证机制可行：资产落 `tests/wpt-runner/wpt-data/` 即被 ZeroWeb 正常渲染（flex-minimum-width-flex-items-007 落 60x60-green.png 后 0.00% PASS，非 "image not found" 退化）。

## ⚠️ 关键路径约束

**所有资产必须落到 `tests/wpt-runner/wpt-data/<wpt_rel_path>`**（runner 的 `wpt_data_dir`），**不是** repo-root `css/`。例：`css/css-writing-modes/support/swatch-yellow.png` → `tests/wpt-runner/wpt-data/css/css-writing-modes/support/swatch-yellow.png`。session 1 stall 根因 = 误落 repo-root `css/`。

## Turnkey 资产清单（23 个，按目录）

上游基准 URL = `https://raw.githubusercontent.com/web-platform-tests/wpt/master/`。每行：`<wpt_rel_path>` `[refcount]` → 落 `tests/wpt-runner/wpt-data/<wpt_rel_path>`。

### css-writing-modes（38 test 文件，ROI 最高，优先）
```
css/css-writing-modes/support/swatch-yellow.png                          [20]
css/css-writing-modes/support/blue-yellow-206w-165h.png                  [18]
css/css-writing-modes/support/test-bl.png                                [10]
css/css-writing-modes/support/test-br.png                                [10]
css/css-writing-modes/support/test-tl.png                                [10]
css/css-writing-modes/support/test-tr.png                                [10]
css/css-writing-modes/support/ortho-htb-alongside-vrl-floats-002-exp-res.png [4]
css/css-writing-modes/support/pass-cdts-float-contiguous.png             [4]
css/css-writing-modes/support/block-flow-direction-025-exp-res.png       [2]
css/css-writing-modes/support/pass-cdts-horiz-rule.png                   [2]
css/css-writing-modes/support/block-flow-direction-066-exp-res.png       [1]
css/css-writing-modes/support/pass-cdts-clearance-calculations.png       [1]
```
（注：R592 核验 session 1 已 fetch 过 swatch-yellow/blue-yellow/test-tl/tr/bl/br 等为有效 PNG，但落错路径已删；swatch-blue.png R590 列出但当前重扫未命中 test 引用——可顺带补齐防 latent。）

### css-fonts（11 test）
```
css/css-fonts/support/css/font-variant-features.css                      [5]
css/css-fonts/support/css/variation-sequences.css                        [3]
css/css-fonts/support/font-weight-bolder-001-ref.png                     [1]
css/css-fonts/support/font-weight-lighter-001-ref.png                    [1]
css/css-fonts/support/font-weight-normal-001-ref.png                     [1]
```

### css-flexbox（9 test）
```
css/css-flexbox/support/test-style.css                                   [6]
css/css-flexbox/support/flexbox.css                                      [1]
css/css-flexbox/support/large-green-rectangle.svg                        [1]
```

### css-multicol（1 test）
```
css/css-multicol/support/swatch-lime.png                                 [5]
```

### css/support（顶层共享；60x60-green/red.png 已落，补残余）
```
css/support/red-rect.svg                                                 [1]
```

### images/（顶层，`/images/` 绝对引用）
```
images/blue.png                                                          [3]   → tests/wpt-runner/wpt-data/images/blue.png
```

## 执行流程（code agent，一条命令循环）

```sh
WPT=https://raw.githubusercontent.com/web-platform-tests/wpt/master
DST=tests/wpt-runner/wpt-data
# 对上面 23 行每条 <rel>：
mkdir -p "$DST/$(dirname <rel>)"
curl -fsSL "$WPT/<rel>" -o "$DST/<rel>"
file "$DST/<rel>"   # 须为 PNG image / ASCII text(CSS) / SVG，非 HTML 404 页
```
（建议：写个 loop 一次性 fetch 全 23 条 + 逐条 `file` 校验非 HTML。）

## 验证（scoped reftest，test-guard 包裹）

落盘后对高 refcount 案抽查（须用 `./target/test-guard -- cargo run --bin zero-wpt-runner -- reftest-upstream <substring>`，禁止裸跑）：
- writing-modes：`percent-margin-vlr-005`（blue-yellow，R590 实测 20.91% REAL divergence）/ `inline-replaced-vrl-004`（test-tl/bl，R590 实测 0.00% FALSE PASS，导入后应变 real divergence=selfsource 下降，DC-14 更诚实）
- css-flexbox：`flex-minimum-width-flex-items-007`（60x60-green，R596 实证 0.00% PASS）

## A/B 方法论（量化 R590 预测）

1. **selfsource before/after**：对受影响 63 test 文件，导入前后各跑 scoped reftest。预期 **selfsource 通过数下降**（false-pass 案变 real divergence）——这是 DC-14 honesty 提升，非回归。
2. **oracle 重抓**：⚠️ oracle PNG（`tests/wpt-runner/oracle-shots/`，2026-06-21 抓）从同一 wpt-data 渲染→chromium oracle **同样**缺图。导入后须用 `tests/wpt-runner/scripts/capture-oracle-per-dir.mjs` **重抓受影响 case 的 oracle PNG**，否则 oracle 数字不动（两侧都从缺图变有图，但 oracle 仍是旧抓取）。
3. **门禁**：导入涉及 wpt-data fixture 变更（非渲染代码），但仍建议 `make test` + `make product-smoke MAX_DIFF=22` 守回归（fixture 变更理论上不改渲染代码，但 img/css 资产补齐会改某些 test 的渲染输出）。

## 状态

- R590：hand-off 提出（63 test / 24 资产）。
- R592：de-risk（fetch 的资产为有效 PNG，非 404）。
- R594：stall #1（abandoned，repo-root css/ staging 删除）。
- R596：mechanism proof（资产落 wpt-data 即生效）+ stall #2（仅落 css/support/ 2 文件即停）。
- 当前（R597）：stall #3 持续，本 turnkey handoff 产出以消除摩擦。
