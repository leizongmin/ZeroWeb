# M3：anchor()/anchor-size() 声明值参数序 canonical 归一（2026-11-05）

一期 css-anchor-position 台账（archive
`2026-10-30-m3-batch3-anchor-position-masking-backgrounds`）挂账 ~2,498 子
测试失败：`e.style.width = "anchor-size(width --foo)"` 写入后读回仍是原序，
WPT 断言期望 canonical 形态 `anchor-size(--foo width)`。本件就地修该单点，
归口台账见
[2026-11-05-m3-failure-cluster-routing](2026-11-05-m3-failure-cluster-routing.md)。

## 根因

engine 对 inline style 声明值按原始字符串直存（polyfill 路径
`merge_style_property`、native 路径 `css_style_declaration` 两个 setter），
值域此前没有任何 anchor 专属代码。css-anchor-1 §3/§4 文法接受锚元素在前的
两种参数序解析，canonical serialization 要求固定 `<anchor-element>`
（`<dashed-ident>`）在前——归一属序列化层，与解析解耦。

## 修复

新增 `zero-css-parser` values 模块 `anchor.rs`，单公开函数
`canonicalize_anchor_functions`：扫描声明值内的 `anchor()`/`anchor-size()`
函数（函数名大小写不敏感，左边界检查防 `--anchor-size(` 误配），括号感知
顶层逗号分块；首块恰两词且末词为 dashed-ident、首词不是时换位为锚元素在
前；逗号后 fallback 块递归处理（嵌套 anchor-size 已覆盖）；函数名、空白
结构与非函数文本原样保留，已 canonical 输入幂等零改动。

接线三处声明值写入路径，存储前归一：

- `engine/js_dom_bridge.rs` `merge_style_property`——polyfill 路径（runner
  testharness 语料实际走的入口）
- `engine/dom_bindings/css_style_declaration.rs` `native_style_set_property_invoke`
  与 `native_style_named_setter`——native V8 路径

自定义属性（`--*`）值按 spec 原样保留语义，三处调用方均显式跳过归一。

## 验证

单测 8 项（换位/canonical 不动/fallback 递归/`anchor()` side 换位含 calc
实参/calc 树内/非函数文本与左边界/大小写/幂等）8/8 通过。

单案 A/B（同二进制口径；`anchor-size-parse-valid.html` 修复前实测复跑，
`anchor-parse-valid.html` 修复前未单测——一期台账只记通道级分母）：

| 用例 | 修复前 | 修复后 |
|---|---|---|
| `anchor-size-parse-valid.html` | 2,869P / 1,436F（实测） | **4,297P / 8F**（+1,428） |
| `anchor-parse-valid.html` | 推算 ~1,062F（挂账簇 ~2,498 − 1,436） | **2,355P / 4F** |

通道对照（`make testharness-css-anchor-position`，253 案）：

| 时点 | Pass | 子测试 | 通过率 |
|---|---:|---:|---:|
| 一期基线（2026-10-30） | 5,205 | 12,811 | 40.6% |
| 修复后（2026-11-05） | **7,457** | 12,820 | **58.2%** |

通道增益 **+2,252**，与挂账 ~2,498 量级相符；差额来自残留 12 失败（见下
节）与语料自然滚动（上游 2026-10-30 → 11-05 分母 +9）。

## 残留（12 子测试，非本修复范围）

anchor-size 8 + anchor-parse 4，两形态，anchor 参数序在这些断言里已
canonical，归 CSSOM/calc 序列化既有账：

1. **calc canonical 化**：`(a + b) / 2` → `0.5 * (a + b)`、项排序
   `10% + anchor(...)`——calc math function 归一未实现。
2. **fallback 数值单位补全**：`anchor-size(--foo width, 0)` → 期望
   `, 0px`——fallback 长度单位归一未实现。

## 回归

- **css-variables 通道**（`--*` 跳过路径直接触及的目录）：一期基线
  239P / 282F / 11T = 44.9%（2026-10-30 二进制）→ 修复后 **304P / 222F /
  11T**（58 案，分母 537，+5 为语料滚动）——**零回归**：timeout 11 持
  平、失败无新增形态；+65 passes 不能单记本修复（期间窗口含其他流的引擎
  提交与语料滚动），回归判据取零回退而非增量。
- **css-anchor-position 通道**即修复目标目录，上表通道对照同时是同目录
  回归：失败集从参数序簇收窄到残留 12，无新增失败形态。
- 十六目录全量零回归对照（DC-3 末件）由归口台账统一安排，在批次 4 干跑
  窗口后执行，分母对一期台账 65,501/17,781 逐 subtest 对照。

## 门禁

- `cargo fmt --all -- --check` 干净；guarded clippy
  `--workspace --all-targets -D warnings` PASS（诊断留档
  `.acceptance/css-v2-m3/clippy-guarded-0130.log`，不入库）。
- `make guarded-test` workspace：**9,100P / 5F**。5 失败全为
  `js_dom_bridge::tests` 的 stylesheet cssText **尾分号序列化**簇（
  `14px; }` vs `14px }` 形态，CSSOM 对端账第 5 轮同名单）——与本修复无
  关：canonicalize 对无 anchor 函数的值恒等返回，且并行布局流 01:38 在
  无本修改的独立 clone 上同报 3727P/5F 同名单（其提交 4234a4f44 文原）。
- 常驻断言：`testharness-css-anchor-position` 通道 target（一期已接线）+
  `values/anchor.rs` 单测与修复同提交。

transcripts 与诊断产物在 `.acceptance/css-v2-m3/`，不入库。
