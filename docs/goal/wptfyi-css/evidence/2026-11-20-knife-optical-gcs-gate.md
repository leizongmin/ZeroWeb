# 刀 optical：font-optical-sizing 三性 + kerning/numeric 写面文法门（2026-11-20）

## 池与 face

池＝ffix2 evidence 残面段三处（NOTES-knife-optical-prep.txt 预备账）：css-fonts
`discrete-no-interpolation.html` optical 14F（allow-discrete 3 + transition-property:all
3 + CSS Animations 4 + WAAPI 4）+ parsing invalid 三页 13F（kerning 2 / optical 2 /
numeric 9）+ numeric-computed 组合形 2F。face＝3 文件（patch
knife-optical-face.patch，生成器 /tmp/knife-optical-face-gen.py 幂等 md5
`8be978fe`）：

1. `crates/engine/src/animation.rs`：`apply_single_property` 补 font-optical-sizing
   应用臂——auto/none keyword 严格解析，失败不动样式 → canonicalize None 回落
   基值（承 ffix2/ffs 配方）。
2. `crates/engine/src/js_dom_bridge/transition_registry.rs`：
   `DISCRETE_TRANSITIONABLE` 15→16 += font-optical-sizing——animation-type:
   discrete（css-values-4 §animations），仅 allow-discrete 可过渡
   （css-transitions-2 §transition-behavior-property）。
3. `crates/style-system/src/property/apply.rs`：`VALIDATED_PROPERTIES` 69→72
   （+= font-kerning/font-optical-sizing/font-variant-numeric）+ 分派臂三段：
   kerning(`auto|normal|none`)/optical(`auto|none`) keyword gate + numeric
   组文法裁决（`normal` 独置；≤1 figure + ≤1 spacing + ≤1 fraction +
   ordinal/slashed-zero 各 ≤1 跨组组合合法，同组重复/normal 混用/未知 token
   invalid）+ 替换函数形放行（store 原文回显豁免同 alternates 形）。
   - https://drafts.csswg.org/css-fonts-4/#font-optical-sizing-prop
   - https://www.w3.org/TR/css-fonts-3/#font-variant-numeric-prop

预备 hunks 先行在树（types.rs `FontOpticalSizingValue` + matcher row +
apply_advanced 级联臂 + gCS 回显），本 patch 不触；b0/b1 双臂同 ride。

## 电池（knife-optical-battery.sh，10:20:33 发射）

- 流程改型（vs ffix2）：face 不预先入树（SH-A1 与本刀共享 animation.rs）——
  b0=树原态构建 → apply → b1 构建；trap 归一方向=face apply。
- b0 `367c1066ee197030754838168f4b8744`（45s）/ b1
  `b17abf3e879fab67a5cd46c747dc2b24`（M2 不等=face 入二进制）。
- RIDE 18 文件（SH-A1 WIP css-parser×3/animation.rs clip 域/engine clip 域 +
  TEV face trap 态 part05.js/es_module/part115 + optical prep）S1/S2/S3 md5
  零扰动。
- 腿：fonts 主靶 + transitions/animations/properties-values-api/typed-om/shapes
  五哨兵 ×{b0,b1}，test-guard 包裹（build 6/14/1800s，leg 4/8/3000s +
  CASE_WALL_S=60），GATE-DONE 10:56:45 rc=0。

## 撞窗事件与三重污染仲裁

corpusexp 探针（R6-TAKEOVER fork，五语料库存 ui/filter-effects/conditional/
borders/flexbox，r4-b1 二进制）BUSY-REFUSED 只认 `zw-(wa3a-r4|corpusexp|tev)`
不认 knife-optical，于 10:21:44–10:26:37 与本电池并行（重叠窗：fonts b0 全程 +
transitions b0 前 2min）。仲裁三证：

1. fonts b0（全程重叠）vs ffix2 b1（干净基线，同 HEAD-ffix2 态无 optical
   face）逐页 P 恒等 **diff pages=0**。
2. transitions b0（部分重叠）vs TEV b1（干净基线，同 TEV face 在）逐页 P
   恒等 **diff pages=0**。
3. 五靶页 b0→b1 delta＝预测−残面（见下），无超额增益/无回归。

→ 探针负载零毒化，账目干净。（反向：探针 ui/filter-effects/conditional/borders
四腿在本窗内跑，其库存账若现 timing 型页 flake 可按本窗仲裁。）

## 收账（10:56:45 RECONCILE）

| 页 | b0→b1 | 判定 |
|---|---|---|
| discrete-no-interpolation.html | P 532→546 | optical 14 行全数 F2P |
| parsing/font-kerning-invalid.html | P 0→2 | keyword gate 拒 "normal auto"/"none, auto" |
| parsing/font-optical-sizing-invalid.html | P 0→2 | 拒 "normal"/"auto none" |
| parsing/font-variant-numeric-invalid.html | P 0→9 | 组文法拒 9 形（normal 混用 4 + 同组重复 3 + auto + ordinal normal） |
| parsing/font-variant-numeric-computed.html | P 9→9 | **2F 未翻（残面）** |

- **TOTAL: gained=27 lost=0 regressed=0 pages_gain=4 pages_loss=0**
- 哨兵：transitions 1051 / animations 840 / properties-values-api 958 /
  typed-om 12024 / shapes 6244——五语料 SET-IDENTICAL。

局部账累计 +7720 → **+7747**。

## 残面

- font-variant-numeric 组合形 computed 2F：gate 已放行（decl 白名单内），
  但组合形 store 原文不落组值级联槽、gCS 回显回落默认——组值级联存储残面
  另刀（numeric-invalid 9 形组文法裁决已兑现，组合形 assert 依赖组值存储+
  回显通道）。
- properties-value 池 `/values` 臂 134F（gCS 读串空，famA）solo 探针页已备
  （zwprobe/values-gcs-probe.html，corpus 树外），刀位待按 NOTES 认领。
