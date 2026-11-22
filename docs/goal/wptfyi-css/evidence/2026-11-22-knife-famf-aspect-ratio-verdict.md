# 刀 FAM-F 收账：aspect-ratio 几何插值＋applier＋gCS 规范形（2026-11-22 11:2x）

## 面与电池

- face 双文件：`crates/engine/src/animation.rs`（interpolate_property_value
  "aspect-ratio" 臂＋parse_aspect_ratio_decl/interpolate_aspect_ratio 幂律
  r0^(1−t)·r1^t＋apply_single_property applier 臂＋compose_animation_value
  不可加 add 臂（有效值取 underlying）＋registry 准入＋钉测 2 函数 7 案）；
  `crates/engine/src/js_dom_bridge/computed_style.rs`（aspect_ratio_str 翻转
  `x / 1` 规范形，css-values-4 §serialize-a-ratio，auto 前缀保留；存储合并比值
  diverge 性质不变仅格式对齐 Chrome）。语义：css-values-4 §combine-ratio。
- 钉测 dev 实证：`cargo test -p zero-engine --lib aspect_ratio` 2 passed
  （几何插值 4 案＋简并/auto 混离散 3 案；test-guard 包裹）。
- 电池 r3（unit zw-knife-famf-salvage-r3，10:57:48→11:21:00 全 6 腿 rc=0，
  log `knife-famf-salvage-r3-1103.log`，快照 `results-knife-famf-aspect-ratio-pristine-1121/`）：
  - bin：b0 `7c6d3c426127868aa197cbf0af778909`（HEAD 态恒同）/ b1
    `c765707eff1f5435764f70866bc450e9`（face 烙入；DOA 修复后重建，
    旧坏版 b6c897f6 已废）。
  - 换装判别：SYMBOL-FM b1=2（interpolate_aspect_ratio/parse_aspect_ratio_decl
    符号）b0=0；M2 异 md5。
  - ride：`es_module.rs` 他流 WIP 双臂同载（s1/s2 md5 恒
    `9d6449b6216d0b06e8a340e42011a4a0`＝delta 相消）。
  - 语料腿：css-sizing（主判据）/css-values/css-transitions（哨）三对。

## RECONCILE（键级逐案）

- **css-sizing gained=59 lost=0**：
  - aspect-ratio-interpolation.html P 59→113（+54）：CSSA/WAAPI 几何插值车道
    （[0.5]→[2]、[1/2]→[2/1]、auto 1/1→2/1、auto 1/2→auto 2/1、auto→2/1 全 t
    点）＋简并离散＋allow-discrete transitions 全 F→P（CASE 54 行实录）。
  - aspect-ratio/parsing/aspect-ratio-computed.html P 1→6（+5）：serializer
    `x / 1` 规范形翻面。
- **css-values lost=1**：random-computed.tentative.html `Shared globally:
  random(--identifier, a, b)` P 120→119——**flake 改判（红线解除）**，见下。
- **css-transitions 零漂移**（gained=0 lost=0）；calc-interpolation.html 无
  SENTINEL-RISE（恒 F 不升）；PREP 红线三域全持位（float 形静态页零 P→F、
  transitions 哨零漂移、calc 恒 F 不升）。

## 红线触发案 flake 定谳（FAIL-STOP 行→COLLECT，M4-b 先例）

正式 VERDICT 行 FAIL-STOP（lost>0）系于 random-computed 单案。改判证据链
（另一 resume fork 11:23 探针 `probe-json-randcomp/` b0/b1 交替 ×3 单页＋
`probe-json-famf-p2f1/` ×4，本实例独立核验）：

- E1：b0-r2（HEAD bin 零 face）同案 Fail——face 无关失败；
- E2：b1-r1 Fail／b1-r2、b1-r3 Pass——b1 双向翻，无确定性效应；
- E3：snap-knife-text-interp-f2 b0（异刀异 face 历史树）同案 Fail——历史复发；
- E4：机制零交集——CSS random() 共享全局确定性（跨元素等值）与 aspect-ratio
  插值/applier/串化无代码路径重叠。

四证齐＝flake 域双向不入账，−1 不计 tally。

## 页内 P2F 逐案归因（page 级 net 不作免责）

interp 页 Compositing add 车道 2P/4F→2P/4F **净零形式位移**（探针
`probe-json-famf-comp/` 本实例 11:29 b0×2 b1×2 单页实证）：

- CSSA at(0) F→P：serializer 形修正＝真增益（b0 got "0.5" float 形）。
- CSSA at(1) P→F：b0 偶然 pass（b0 Compositing 全 t 产出 underlying，恰合
  at(1) 期望 [2/1]）→ b1 raw 插值（got "1 / 1"，端点 0.5→1 未预合成）。
- WAAPI at(1) 双臂恒 P；at(0)/at(0.5) 双臂恒 F。
- 机制：composition registry/pre-compose 块未触达 CSSA 插值路径（WAAPI 侧
  underlying-at-all-t 持位）——**挂账后续刀 FAM-F2**（add 帧预合成触达 CSSA
  路径），PREP 预注册排除域（"Compositing 不入主判据单列"），非红线域。

ASYM css-transitions event-dispatch.tentative.html b0=11/b1=15：时序域案
注册漂移，pass 零差，flake 域记账不折算。

## 判定

- **COLLECT 净 +59**（css-sizing）；tally 本轮 A/B 累计 +13269→**+13328**。
- face 随本笔提交入库（animation.rs＋computed_style.rs）；钉测随面（防
  pin-tests-doa）。
- 遗留：FAM-F2（CSSA composition 触达）；random-computed flake 域不追踪；
  event-dispatch ASYM 时序域不追踪。

## 复证附记（r4，2026-11-22 11:4x，收账 +59→+61 勘误）

上文本笔「遗留：FAM-F2（CSSA composition 触达）」随 b9f612513 域门准入
hunk 落地即过时——11:21 NOTES 处置笔（笔先于行）已把触达修入同 face。
r4 电池复证：

- 电池：unit `zw-knife-famf-salvage-r4`（11:29 发射，GATE-DONE 11:45:22，
  log `knife-famf-salvage-r4-1129.log`）。b0 复用 r3（7c6d3c42，HEAD 态恒
  同，产物零重建）；b1'' `1868cce65c70a0535598020644bce368`（face 含域门
  准入＋compose 臂，SYMBOL-FM b1=2）。r3 b1 产物已快照
  `results-knife-famf-aspect-ratio-r3b1-att/`。
- **css-sizing gained=61 lost=0**：interp 页 P 59→115（+56）＝plain 车道
  +54 同 r3＋Compositing CSSA at(0)/at(0.5) 两条 fail→pass＋at(1) 持 P
  零回退（r3 的 P→F 不复现）＝**CSSA trio 全 3P，预合成端点 0.5/1→2/1
  几何插值与页面期望逐字吻合**。computed 页 +5 同 r3。
- **css-values lost=1**＝random-computed `Shared globally:
  random(--identifier, a, b)` 同案同一性（msg 逐字同 r3；E1-E4 链直用，
  OM-18 排除域「随机页每次重生成」）——红线解除同 r3 判读。
- css-transitions gained=0 lost=0；calc-interpolation 无 SENTINEL-RISE。
  ASYM ×3：event-dispatch 11/15（同 r3）＋animate-with-color-mix 4/5＋
  properties-value-001 1/0——case 注册完备性旗，pass 零差，不折算。
- 判读：机械 VERDICT 行 FAIL-STOP 系于排除域案；**COLLECT 净 +61**
  （tally +13328→+13330）。+59→+61 差额＝域门准入边际收益（CSSA
  at(0.5) 翻面＋at(1) 回持）。
- **FAM-F2 挂账收窄至 WAAPI compositing 车道**：WAAPI trio at(0)/at(0.5)
  双臂恒 F（pre-merge 块未触达 WAAPI 采样路径，underlying-at-all-t 持
  位）；CSSA 面已在本 face 修复，不再挂账。
- fmt 勘误：b9f612513 钉测断言 7687/7712 两处折行违 rustfmt（本附记同
  批提交修正）。存量漂移（animation.rs 1344/2218/5189/5201/5217/6377/
  6386＋computed_style.rs 36/483＋animation/clip_interp.rs）均在刀 hunk
  域外，不动。
