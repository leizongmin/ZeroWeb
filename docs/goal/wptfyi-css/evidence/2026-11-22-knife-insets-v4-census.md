# 刀 CSSOM-INSETS-V4 census 证据（2026-11-22，V3 收账后余池失败归类）

> 本地工件（gitignored，ZeroWeb-3 clone）：探针页 `tests/wpt-runner/wpt-data/css/cssom/zz-v4probe-vertical-rl-fixed.html`、
> 探针 JSON `./.acceptance/wptfyi-css/results-knife-insets-v4-census/probe-json-rl-lr.json`、
> 工作底稿 `./.acceptance/wptfyi-css/NOTES-knife-insets-v4-vertical-wm-census.md`。


## 池与签名（V3 r2 battery JSON 实测，`results-knife-cssom-insets-v3/b1/cssom.json`）

gate 两页残 108F（absolute 24 + fixed 84），三族：

1. **fixed vertical-rl：统一 +140 x 平移**（60F＝pct/calc/overconstrained 36 + single-auto 24）。
   LTR 容器 left 族 got=expected+140（150→290/148→288/600→740/596→736），RTL 容器 right 族
   got=expected−140（450→310/446→306/600→460/598→458）＝x 帧整体平移＋回读忠实（echo=used
   值，布局位错非 echo 错；right=cbW−(x+w) 恒等式自洽验证）。
2. **both-auto static 域 36F**（fixed lr/rl 各 12 + absolute lr/rl 各 12）：top static 回读错，
   ltr 容器 127→320（+193）、rtl 容器 173→538（+365）；assert 先 fire top，x 值被掩盖未见。
3. **absolute both-auto 24F**＝族 2 同根（absolute 页 inner vertical-*）。

## 探针矩阵（`zz-v4probe-vertical-rl-fixed.html`，wpt-data/css/cssom/ 本 clone 语料；产物
`results-knife-insets-v4-census/probe-json-rl-lr.json`；runner=b1 face 449c54d4）

| 形 | 断言 | 我方 | chrome/WPT 期望 | 定谳 |
|---|---|---|---|---|
| vertical-rl 直接子，pct left:25% | 150px | **75px**＝25%×300(inline) | 150＝25%×600(physical) | % 轴基走 inline 轴，物理轴未用（cssom resolved-value：top/bottom×CB 高、left/right×CB 宽，物理维度） |
| vertical-rl 直接子，all-auto | static 位 | top echo 320px；t.gBCR=(64,320) rel padding origin | 内容流起点（≈472,64） | used 值解析已触达但 static 落点 vertical 块流轴映射错（block-start 应右缘） |
| vertical-lr 直接子，pct left:25% | 150px | **raw "25%"**（未 absolutize） | 150 | resolved-value 管道整体未触达（比 rl 更基础层） |
| vertical-lr 直接子，all-auto | used 值 | **raw "auto"**；t3.gBCR h=172 异常 | used 值 | 同上，管道未触达 |

## 形态映射未定谳（V4 第一侦查点）

corpus 嵌套形（cb[transform+WM] > abspos[relative] > inflow > testEl）与探针直接子形行为分裂：
- corpus vertical-lr 容器 pct **全过**（150px 正确）＝嵌套链存在时 resolution 通路可达；
- 探针 vertical-lr 直接子 pct raw echo＝通路未触达。
候选分裂点：fixed-CB 分类是否带写向 gate／GEN_RECT/containing_block_padding_rect 的
rect 源在两种 DOM 形态下的可达性／`is_vertical_block_flow()` 门位置。corpus +140 形
（290）是否含轴基错未定谳：双假设 150+140（物理轴基+静态位污染）vs 75+215（inline 轴基+
更大污染）——剥层探针（逐容器移除）可裁决。

## 引擎锚点（abspos.rs 现状）

- R4017 static-y fix 显式竖排排除：`box_node.writing_mode.is_vertical_block_flow()` →
  continue（行底 y 写入语义不适用竖排）；R5224 rtl 行首锚镜像 gate 限 HorizontalTb；
  R5379 混合容器 IFC 锚 y 偏移竖排不适用。＝**竖排 static position 是显式 park 域**，
  V4 需新域臂（R4017-x：竖排块流行进轴＝水平，block-start=右缘[ltr]／左缘[rl]，
  inline-start=top/bottom 按 direction）。
- V3 刚入库的 rtl 过约束选边（rtl_overconstrained_right_anchor）为 HorizontalTb 语义
  写就，cb direction 判定来自 CB rect 附带 direction（V2），竖排 CB direction×写向
  组合未覆盖——V4 改 resolve/选边臂时须回归 V3 恰集（battery 五哨兜底）。

## V4 处方（下轮序）

1. 侦查：形态分裂点定位（fixed-CB 分类 gate + rect 源可达性）→ 剥层探针裁决 +140 构成。
2. 臂 1（若 corpus 形含轴基错）：inset %/calc 轴基物理化——resolve_abspos_inset_px caller
   链 cb 尺寸传递按物理轴（预计小改面、corpus 直接兑现 pct/calc/overconstrained 族）。
3. 臂 2（另刀）：竖排 static position 新域（R4017-x）+ both-auto used 值域——面较大，
   需先设计 vertical 块流 static 累计模型再动刀。
4. 门禁：battery 五哨（cssom 四 insets 页 + 三哨 SET-IDENTICAL）+ product-smoke
   （布局域变更惯例）+ V3 恰集零回退（GRAND P2F=0 硬门自含）。

## 本轮不动项

- 探针页留在 wpt-data/css/cssom/（gitignored 本 clone 语料，不影响对账快照；
  corpus 名 zz- 前缀防上游同名）。
- tally +13647 不动（census 无兑现）。
