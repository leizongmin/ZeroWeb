# g44 色标文法位 + style-attr 写面拒收刀

2026-11-15。driving: css-images gradient/color-stops-parsing 60F（10 invalid 形 × 6
渐变函数；valid 102 全过）。断言双面：`div.setAttribute("style", …)` 写 +
`div.style.getPropertyValue("background-image")` / gCS 读——invalid 渐变声明被
写面原文落库回显伪成功（g30 同型），叠加 parse 侧 hint 文法位宽松。

## 1. triage（两通道三形）

- **写面通道缺口**：g43 拒写哨兵只接 `_zwNormalizeDeclValue`（setProperty/IDL
  set 面，part06 setProp + part03 writeDecl）；本页走 `setAttribute("style", …)`
  内容属性通道——handle 世界（活跃文档 createElement → `_wrapHandle` → part04
  `_makeProxy` setAttribute）与物化世界（detached doc → part03 `_zwMElSetAttribute`）
  双世界均原文落库，`_styleProxy.readRaw`（R3199 replay）回显原文 → inline 断言
  首跳即败（computed 断言未达）。
- **parse hint 文法位宽松**（5/10 形 parse 误接受）：`black, 25%`（尾 hint）、
  `black, , white`（空段）、`black, white, 75%`（尾 hint）、
  `black, 25%, 50%, white`（连续 hint）、`,black, white`（空段）——旧
  `parse_color_stops` 空段静默跳过、hint 只要有前导色标即消费。余 5 形
  （`""`/`black, invalid`/`black, 25% 50%, white`/`black 10% 25% 50%, white`/
  `0%, black, white`）parse 已拒但被写面回显掩盖。
- **radial 首参误派发**：`radial-gradient(0%, black, white)` 首参单百分比命中
  size-only 派发当 ellipse 尺寸消费（css-images-4 `<size>`：circle 单 `<length>`
  不受 %、ellipse 需双值）→ 渐变伪接受。

## 2. 实现

- **parse 色标文法位**（css-parser `values/parse_transform.rs`）：
  `parse_color_stops` 空段返 None；hint 位约束（首位色标前/连续两 hint/尾悬挂
  均拒收，css-images-4 §color-stop-syntax hint 仅夹两色标间）；单色标表合法
  （`linear-gradient(black)` 页面 valid 面钉，css-images-4 放宽非 {2,}）；radial
  派发单 token 纯百分比不入 head（落回色标段由 hint 位约束拒收）。
- **写面逐声明拒收**（engine shim）：part06 新增 `_zwStyleAttrDeclFilter`——
  `_zwSplitDeclSegs`（引号/注释感知）分段，background-image 声明值经 g43 同源
  判定器（`__zw_gradient_normalize_decl` 'null' 哨兵）拒收丢弃；幸存声明原文
  保形（非 Blink parse-reserialize——T52 mini-impl 边界既录口径不变）；`!important`
  尾缀先剥离再判（混入误杀合法声明）；非 background-image 属性不介入；
  无 'gradient' 子串快门（拒收集全为渐变前缀层，热路径零开销）；回调未注册
  零漂移。双世界接线：part04 `_makeProxy` setAttribute（handle 世界，语料驱动
  面）+ part03 `_zwMElSetAttribute`（物化世界，同型回显缺口一并收口）。
- session 事故记录：首录 corpus 净 ±0——刀口先落 part03 物化世界（活跃文档实为
  handle 世界 `_wrapHandle`，part03:27348 createElement 是 detached-doc 工厂）；
  补 part04 臂后 +60 兑现。A/B 判读以逐页 diff 为准（totals 恒同≠变更生效）。

## 3. 单测与门禁

- css-parser **3,223P/0F**（+1 钉 `test_g44_color_stops_parsing_page_forms`：
  页面 10 invalid 形 × 6 函数 canon 全 None、16 valid 代表形全 Some、500-stop
  stop/hint 交替长表接受——页面 parse 期望逐形镜像）。
- engine lib **3,991P/0F**；fmt `--all --check` clean；定向 clippy
  css-parser+engine `--all-targets -D warnings` 零告警。

## 4. 受控 A/B（B0 = results-g43-b1 g43 提交态；B1 = results-g44-b1）

| corpus | B0（g43-b1） | B1（g44-b1） | 净 |
|---|---|---|---|
| css-images | 2881P/334F | 2941P/274F | **+60P/−60F** |
| css-values | 4041P/1546F/6T | 4041P/1546F/6T | **±0 零回归** |

- 逐页（ab-g42.py，案数恒同 3215/5593）：color-stops-parsing **F 60→0 全页清零**
  （B1 页内 162/162 Pass），其余全 corpus 零漂移。**REGRESSED=0，grand net +60。**
- css-images corpus 两刀累计（g42 基线 1439P → g43 2881P → g44 2941P）。

## 5. 残面挂账

- image-set-parsing 114F + image-set-computed 36F（6b 指派 C+D 刀）。
- gradient-position-invalid 残 5F、conic-calc-angle-invalid 残 3F。
- solo 直跑 color-stops-parsing/image-set-parsing 零输出 exit=1 异常（corpus
  路径计数正常）在案未解。
- bridge replay 通道（style_from_mutations_lw fold 消费）invalid 拒收未接
  （runner 页面走 shim 通道无语料驱动，扩面 defer）。
- handle 世界 IDL `proxy.style = …` set trap 通道未验（本页未驱动；style IDL
  setter 走 setAttribute 臂已被本刀覆盖，TypedOM 面另查）。
