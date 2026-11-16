# CSS Motion Path offset 簇刀——属性模型 + gCS 通道 + WAAPI 插值，A/B 双面账

日期：2026-11-15（十三轮，wptfyi-css；承接 rate-limit 中断轮的 WIP 树态认领收口）
改动面（8 文件，+711/−41，均 Rust）：
`crates/style-system/src/property/{types,registry,inherit,default_impl,apply_advanced,computed_style}.rs`
+ `crates/engine/src/{animation.rs,js_dom_bridge/computed_style.rs}`。

## 机制要点

**目标面**：texp 逐组转化率轮（11-15 七轮）残差分桶中 supports 覆盖门下的
`offset-* 3.3k`（css-motion-1 语料 45 页，刀前全群 0P 量级——offset 属性无
模型：apply 拒、gCS 无臂、known_properties 白名单缺、WAAPI 插值链无
underlying/端点插值）。

**五站全链**（css-motion-1，非继承）：

1. **类型/注册**（types.rs/registry.rs）：`OffsetRotateValue`
   （Auto/Reverse/Angle(f64 deg)/AutoAngle(f64)）+ 四属性入
   `known_properties()` 白名单（CSS.supports `decl_supported` 前置门）+
   initial-value 表借 `Transform(None)` 位（真实重置在 apply_initial_value）。
2. **apply**（apply_advanced.rs）：`offset-path` none|`path("...")` 首片文法
   （ray()/基本形状 bail → 支持性假、级联丢弃，后置刀）；`offset-distance`
   `<length-percentage>`（font 相对长度按所属元素字号绝对化，同
   transform-origin R5193 语义；% 保留渲染期）；`offset-rotate`
   `auto|reverse|<angle>|auto <angle>`（角度归 deg：deg 直读防 rad 往返
   精度损失）；`offset-anchor` auto|`<position>`（复用 parse_origin_xy）。
3. **初始/继承**（inherit.rs/default_impl.rs）：spec 初始 none/0px/auto/auto。
4. **gCS 计算值通道**（engine computed_style.rs）：四属性序列化臂（缺臂即
   gCS 读缺省——individual translate/rotate/scale 先例同教训）；distance
   font 相对长度 apply 期已绝对化直接串化。
5. **WAAPI 插值链**（engine animation.rs）：`underlying_individual_value`
   四臂（声明值序列化，声明缺席落 spec 初始）+ `interpolate_property_value`
   四臂（distance 同单位线性/混合单位离散、rotate 纯角度线性/关键字侧离散、
   anchor position 对同单位线性、path 离散；离散边界 css-values-4 §8.5.6
   ≥0.5 → to）+ `apply_single_property` 写回臂（动画值进 ComputedStyle，
   underlying 回读往返）。

**css-motion-1 参考**：<https://drafts.fxtf.org/motion-1/#offset-path-property>
等四属性节；插值离散边界
<https://drafts.csswg.org/css-values-4/#compression>（§8.5.6）。

## 单测

- style-system `offset_props_tests` 4 钉：path none/path()/ray()/circle()
  门、rotate 四形、distance px/%/em 绝对化、anchor auto/对形、
  path() 校验器负钉。
- engine animation 3 钉：distance 同单位线性 + 混合单位离散、rotate 角度
  线性 + auto 离散、apply 写回 + underlying 序列化往返。
- dev profile：`cargo test -p zero-style-system --lib offset_props` 4 通过；
  `cargo test -p zero-engine --lib test_offset` 5 通过（3 新钉 + 2 既有
  offsetparent 面）。fmt（FMT-SCOPE 八刀文件，--edition 2024 + config-path）
  ✓；`cargo clippy -p zero-style-system -p zero-engine --all-targets` 净
  （零-engine 侧零告警；唯一 warning = script-sandbox `too_many_arguments`
  s419 WIP 在树连坐，peer 域在册）。收口补刀：offset-anchor 插值臂
  format! 嵌套改单 format!（clippy format-in-format）。

## A/B 双面账（干净二进制重放，supersede 前轮 stale-B0 对比）

**前轮事故修正**：rate-limit 中断轮曾以 `results-jsanim-b3`（三轮前 B1
产物）为 B0 对比，得「css-transforms +47 / css-values +7」——法证定谳为
**陈旧基线错账**：+47 transforms 实为 composite-add（十一轮）与 easing-flip
（十二轮）已提交增益（bin-jsanim-b3 建于两刀之前）；对真基线
`results-flip-b1`（十二轮 B1 = 本刀前 HEAD 行为）复比对，transforms/
values/fonts/backgrounds 四 corpus **逐字节零差**，真增益全部落在目标面
css/motion。教训入册 [docs/learnings/bugs/2026-11/2026-11-15-corpus-ab-stale-baseline-results.md](../../../../docs/learnings/bugs/2026-11/2026-11-15-corpus-ab-stale-baseline-results.md)。

**本轮干净重放**：

- B0c = `bin-offset-b0c/zero-wpt-runner`：stash 八刀文件后于 HEAD(c7b7e9574)
  重建（树载 peer es_module s419 WIP——与全部在库快照同基座；CSS corpus 页
  纯 script 无模块路径）。B1c = `bin-offset-b1c/zero-wpt-runner`：WIP 树
  重建。md5 对拍 fddac4a0 ≠ 2680ef20（刀已入 B1c）；38.09s Finished 为
  温 target 增量（engine 全量重编 + 链接，非复用 rlib——哈希不同为证）。
  体积 127MB 与历史快照 950MB 差异 = debug 打包形态，行为等价由「B0c
  motion 结果与 results-flip-b1 motion 逐页恒等（45 页 P105/F4850 全同）」
  实证。
- 双腿十腿批（zw-offab2.service，22:30–22:52）：motion/css-transforms/
  css-values/css-backgrounds/css-fonts × {b0c,b1c}，test-guard 包裹，
  transcript 唯一名，JSON 分目录 `results-offset-{b0c,b1c}-clean`。
- 白名单触面护栏双腿（zw-offab2b.service，22:52–22:58）：css-typed-om +
  cssom（CSS.supports `decl_supported` 全局门触面——known_properties 增
  四串为纯增量，护栏实证零外溢）。

**结果（B0c → B1c，七 corpus）**：

| corpus | gained | lost | regressed | pages_gain |
|---|---|---|---|---|
| motion | +363 | 0 | 0 | 16 |
| css-values | +1 | 0 | 0 | 1 |
| css-transforms | 0 | 0 | 0 | 0 |
| css-backgrounds | 0 | 0 | 0 | 0 |
| css-fonts | 0 | 0 | 0 | 0 |
| css-typed-om | 0 | 0 | 0 | 0 |
| cssom | 0 | 0 | 0 | 0 |
| **合计** | **+364** | **0** | **0** | **17** |

- motion +363 明细：interpolation 族 311（path-001 109/rotate-interp 82/
  anchor-interp 44/distance-interp 26/path-007 24/path-004 12/
  path-003 8/path-002 6/distance-composition 12/rotate-composition 10）+
  parsing/computed 25（anchor 10/path 7/distance 6/rotate 2）+ inheritance 4
  + supports-calc 1。
- values +1 = random-computed `Property offset-distance value
  'random(10%, 30%)'`（属性白名单 + apply % 槽红利，刀面内副产物）。

**与 z2 存量互证**：`results-offset-b1-z2`（前轮 bin-offset-b1 19:07 快照，
19:12–20:03 产）对 `results-flip-b1` 同窗口逐 corpus 数字与本轮双腿一致
（motion +363/values +1/其余 0）——两代二进制行为收敛。

## 残面桶（motion 45 页余 4,487F，下一刀候选）

- ray()/基本形状文法族 ~1,830：interp-005/006/008（1,626）+ parsing/
  computed ray 形（204）——`parse_offset_path_fn` 首片 bail 的后置面。
- offset-position 属性缺面 ~386（interpolation 312 + composition 56 +
  parsing）——同型属性模型刀。
- 已解锁页内插值中点/混合形残余 ~1,891：rotate auto+angle 混合端点
  （rotate-interp 余 458）、path() 命令级兼容插值（离散化设计，
  path-001/003/004/007 余 687）、distance %/混合单位（余 166）、
  anchor 混合单位对（余 268）等。
- coord-box 形 144；composition add/accumulate 140；math 函数角度 96
  （rotate/ray-angle-interpolation-math-functions 各 48）。

## 门禁（make test 拆腿形态，zw-offset-mkt3.service，23:01–23:21）

全局 fmt 步按 composite-add/flip 先例拆出（peer es_module s419 WIP fmt 漂移
阻断），本刀八文件 FMT-SCOPE 复核 ✓（--edition 2024 + config-path）。分腿账：

| 腿 | rc | 红项 |
|---|---|---|
| companion-build | 0 | |
| workspace-test | 101 | es_module 6 WIP 红（peer s419 在途：TLA/defer s269/s337/s360/s361 等），cargo fail-fast 截停后续 crate |
| quickjs-clippy | 101 | `too_many_arguments(8/7)` es_module.rs:2217（peer WIP，-D warnings 连坐，在册） |
| renderer-bin | 0 | |
| renderer-lib | 101 | worker-fetch renderer 形（在册：left "undefined" right "ERR"） |
| browser-bin | 101 | tab worker-fetch + default_actions（在册） |
| browser-tests | 101 | 同对（在册，741P/2F） |
| gpu-adapter / gpu-consistency | 0/0 | |
| quickjs-workspace | 101 | image_batch_e2e 负载形（在册伪红，solo ×3 定谳；775P/1F） |
| quickjs-renderer-bin | 0 | |
| quickjs-renderer-lib | 101 | worker-fetch quickjs 形（在册，369P/1F） |

**本刀域零红**：engine 3 钉在 workspace 腿日志全绿
（`test_offset_distance_interpolates`/`test_offset_rotate_interp_and_discrete`/
`test_offset_apply_roundtrip_underlying`）；style-system 4 钉因 fail-fast 截停
未及，门禁后补定向腿 RC=0 全绿
（`cargo test -p zero-style-system --lib offset_props`，test-guard 包裹，
`/tmp/zw-offset-sysleg.log`）。
