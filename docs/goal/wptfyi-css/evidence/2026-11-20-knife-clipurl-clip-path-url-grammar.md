# 刀 CLIP-URL · clip-path `<clip-source>` = `<url>` 文法面（2026-11-20）

## Claim 与 census 入口

- claim 笔：NOTES-knife-clipshape-prep.md（46b68290 resume 主链续，探针 lane 认领）。
- census 入口：master.md 前笔 CLIP-SHAPE 探针 census（results-clipshape-probe-184828，
  masking 883F）——失败值枚举定谳刀位 = url() 文法面（66 supports 失败全数
  url() 形；xywh/rect 解析经代码核实已存在，改判不追）。

## Ground-truth 探针（face 后复跑，zwprobe3 镜像 + CS-1c b1 代 binary）

- 产物：zwprobe3/probe2.json（BIN=results-knife-cs1c/bin/b1-runner——该代含
  本 face 终形，见 NOTES-clipurl-fork-contamination-cs1c-window.md 时序）。
- PROBE-SUP 矩阵：url("#cp")/url(/clip-source)/url("/clip-source")/url("abc")/
  url("") 全翻 true；`url(#cp) border-box` 保 false（css-masking-1 CR 文法
  `[ <basic-shape> || <geometry-box> ] | <url>`：url 独立臂不与 geometry-box
  组合，语料 clip-path-valid/invalid 双页均无 url+box 用例，零冲突）。
- PROBE-COMPUTED：设 url("#cp") → gcs.clipPath = `url("#cp")`（face 前 none）。
- PROBE-WRITE：三形 echo 保真（face 前已经 loose write 臂成立，face 后不回退）。
- 探针页 4 subtest 报 Fail 系页面 assert_unreached 报告机制（含 border-box
  全真断言），非 face 缺陷；语料预期全数匹配。

## Face（7 文件）

- css-parser `parse_clip_path` url 独立臂（置于盒剥离前拒 url+box 组合）：
  引号形/非引号形/空引号串受理保原文；`url()` 空参/非引号含空白/引号不配对/
  反斜杠拒收；suffix `)` 剥除与八函数同款（v1 缺该步被单测定谳后修复——
  url(abc) inner 恒含 `)` 全拒）。
- `ClipPathValue::Url(String)` 变体（types.rs，enum doc 同步去「不支持 url()」）。
- engine computed `clip_path_to_css` Url 臂：`url(<原文>)` 回显（computed
  value: as specified；离散语料 `url("abc")` 带引号钉面）。
- 域外拒收三臂（复用泄漏防护）：shape-outside 写面门（apply.rs）+ shape-outside
  apply 槽（apply_advanced.rs）+ border-shape 段解析（parse_border_shape）——
  shape-outside 文法 url <image> 形未建模（上行 FIXME 同口径）、css-borders-4
  border-shape 文法无 <url> 臂。
- 穷尽臂两处：clip_value_has_calc（Url 无结构化 calc → false）、
  border_shape_vertices（Url 非几何 → 空形）。
- 单测：clip_path_url_forms（受理 6 形 + 拒收 6 形 + border-shape 拒收），
  css-parser clip_path 族 69 passed。
- supports 自动翻真：@supports matcher（supports.rs:186）+ CSS.supports apply
  oracle 走同一 parse_clip_path；animation 离散翻转走 interpolate_clip_path
  parse 失败 → 既有 None→t≥0.5 flip（url↔url/url↔basic-shape 均离散，正确）。

## A/B 电池（knife-clipurl-battery.sh，results-knife-clipurl）

- b0 md5 1a4bb8aaea02590b6be41537edadcb76 / b1 md5 0ce6721d1b7c3f366682c5b0ca844d45；
  FACE-MARKER b0=0 b1=1（`#[used]` static 'CLIPURL clip-path-url-grammar-arm'）；
  M2 异 md5 ✓；RIDE 2 文件（es_module.rs 对端 WIP + inheritance.rs CS-1c
  face transit）S1/S3 零扰动 ✓；8 腿 19:25:26→19:31:52；零 ASYM。
- RECONCILE：**gained=48 lost=14**
  - masking clip-path-interpolation-001.html P 358→394（**+36**）
  - masking clip-path/animations/clip-path-interpolation-discrete.html P 135→147
    （**+12**）——两页恰为 census 预测 url 臂页，supports 前置门翻开 + 离散
    expect 字面（引号保真）双双兑现。
  - shapes/typed-om 零位移（哨兵零回归）。
  - borders corner-shape-interpolation.html −14：**CS-1c corner hunk
    （apply_advanced @@3352 logical_alias_physical_slot）ride 效应**——该 hunk
    在本刀 patch 内（同文件双刀），b0 剥除/b1 重放产生臂间差；非本刀账，
    归 CS-1c lane（其收账已采认，见 NOTES 回执）。

## 双电池确定性互证

CS-1c 电池（19:10 窗，b0 parse=v1 臂间差形态）同测两页 +48（001 +36 /
discrete +12），与本刀（19:28 窗，b0 无 face）逐页相等——独立双窗双二进制
同 delta = deterministic 双证（其收账已按 NOTES 归因约定让渡该 +48 入本刀）。

## 交叉验证残差

borders b1 页级比对（本刀 b1 vs CS-1c b1，回执点 4）：恰 1 页不等——
corner-shape-interpolation 140 vs 123：本窗 HEAD 含 OM-4（19:12 落库，
part10-typed-om.js cursor reify），其窗早于 OM-4；17 点差归 OM 流，非臂间差。

## 残面池（不入本刀）

- shape() 插值 218F（css-shapes-2 shape-function 插值刀，重刀）。
- clip-path composition/computed 族残面（calc 化 inset 跨形对、clip-composition
  72F——calc 面整刀域）。
- offset-path url("abc")（css-motion offset-path 文法，独立 parse 通道，5F 级）。

## 归因修正（11-20 19:38 终证推翻笔）

上文「borders corner-shape-interpolation −14＝CS-1c corner hunk ride 效应」
被 NOTES-clipurl-fork-contamination-cs1c-window.md 回执点 6 终证推翻：子测级
比对示该页两 b1 间同语义断言双向翻转（69 F→P / 52 P→F，如
corner-top-right-shape from [notch] to [square] at (0) 同断言两侧各现），
且本刀 b0 对 CS-1c b0 同向漂移 +20——该页 P 值在行为等价构建间摆幅 ≥17
（probe2/3 已实证的 corner 槽 HashMap 迭代序跨进程非确定性）。−14 落噪声带
内，不可归因任何 face hunk；按 flake 域口径双向不入账不记回归。CS-1c 证据
页 −11 同判（待其 b2 孤立臂终谳）。masking +48 两页归因不受影响。

## 长账

本刀局部账 +48（masking 两页）；周账 101,673→101,721/199,018。
