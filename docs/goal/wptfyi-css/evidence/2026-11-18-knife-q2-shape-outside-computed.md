# 刀 Q2：shape-outside computed 三修（format_num 6 位有效 + calc Ex/Ch 静态叶 + ch 兜底 1em）（2026-11-18）

## 靶面

css-shapes 腿 shape-outside computed 域三类 168F（刀 Q p1 收口后残面，
[Q p1 evidence](2026-11-18-knife-q-shape-outside-specified.md) 探针实测分类：
精度 ~88F、calc 非 Px 静态叶 ~40F、ch 度量 ~40F）。页面运行时经
parsing-utils `buildPositionTests(..., 'lengthUnit + computed', validUnits)`
生成 unit × position 组合，期望串为 Chromium oracle 6 位有效数字形。

## 机制（3 文件，fv1/A3r 在飞件零交集）

**Fix C｜format_num 千分位→6 位有效数字**（engine `computed_style.rs`
`format_num`，129 调用点单源——animation.rs 插值串化与
registered_properties.rs 同函数复用）：Chromium gCS 数值串 = 6 位有效数字
去尾零（CSSOM serialize decimal 精度口径；`50pt`→`66.6667px`、
`1pt`→`1.33333px`——旧千分位 `66.667px`/`1.333px` 与 oracle 串不等）。
≤3 位小数形旧新同串（`19.2px`/`16px`/`0.5%`）；6 位有效舍入同时保留既有
f64 噪声抑制（`19.200000000000003`→`19.2px`）。≥1e6 数量级保留全整数位
（语料无驱动，FIXME: %.6g 科学计数口径挂账）。

**Fix A｜calc 静态叶 Ex/Ch 折 px**（engine `calc_computed_to_css` 的
CalcContext 构造补 `x_height = 0.8×font_size`、`ch_width = 1.0×font_size`）：
桥层 ctx 未填两度量 → `resolve_length_to_px` Ex/Ch 叶 None → 整串 `''`
（css-values-4 §5.1.1 ex=x-height、§5.1.2 ch='0' advance，Ahem 两度量分别
0.8em/1em；语料 `calc(70ex)`→`896px` 驱动）。度量兜底口径与 style-system
`resolve_length_inner`（ex 0.8 已有、ch 1.0 本刀）对齐。

**Fix B｜ch 兜底 0.5em→1em**（style-system `computed.rs`
`resolve_length_inner` Ch 臂）：css-values-4 §5.1.2 1ch = '0' 字形 advance，
无度量上下文按 Ahem-oriented 口径（语料字体 '0' advance = 1em，
`50ch`@16px → `800px`；旧 0.5em 兜底恰 ×0.5 系统性偏小——Q p1 探针
50ch 800→400 定谳）。ex 兜底 0.8 已对不动；Rch/rch 根相对兜底不在本刀
靶面（无语料驱动，维持 0.5 待池）。

## 门禁

- fmt：两面 rustfmt --check rc=0。
- style-system lib 全量 **2566/0**（7 处 ch=0.5 旧真钉测改判：computed.rs
  `test_resolve_ch` 16→32、`test_resolve_ch_approximation` 40→80、
  property/tests/ui.rs 5 处 2ch@16px 断言 16→32）。
- engine lib 全量复跑 **4184/0**（ENGELIB-RC=0，12:14 同窗：对端 A3r WIP
  编译稳态后共树共跑；总数 4181→4184 含对端新增 3 钉测。首跑
  4177 passed/4 failed——3 处 format_num 旧形钉测改判
  `circle(at 1889.764px 50%)`→`1889.76px` ×2、aspect-ratio `1.778`→`1.77778`
  （Chrome aspect-ratio 数值形 6 位有效）＋ 1 处窗内瞬态插桩，见下方法证）。
- 定向 clippy（`-p zero-engine --lib -D warnings`）两连跑定谳：首跑在
  `zero-script-sandbox` es_module.rs:2211 too_many_arguments 截停（fv1 WIP
  在案 lint，路径依赖被连带 lint）；带单项 `-A clippy::too_many_arguments`
  重放后 engine lib 仅余 2 处在案 unnecessary_cast（animation.rs:2452/2453
  `*p/*v as f64`，2d3e71d19 提交态原 lint、行号随对端 A3r 重构漂移，归属
  对方域）——本刀两面（engine computed_style.rs / style-system
  computed.rs）零 lint，clippy 清偿待 fv1 落账（挂账池既有项）。

## 法证｜首跑第 4F 为窗内瞬态插桩，非 HEAD 泄漏

engine lib 首跑 4F 中的 `tmp_probe_ts_lerp_exact_orange`
（`panic!("PROBE-0.3-exact: …")` 取值插桩）经三重 grep 定谳：HEAD
`53d13b0eb`、A3 机制提交 `5f2fb94b0`、当前工作树三态均无此名——系共享树
窗口内对端 A3r 开发期瞬态插桩（TEMP-DBG 形），非入库泄漏，零提交义务。
当前 animation.rs 工作树 diff 663 行全量归属对端 A3r WIP（NOTES
`A3r 残面刀归属声明` 一致），本刀不触碰。

## A/B 定谳（七语料 × 双侧十四腿，配方逐字同 knife-a3-ab-run；fonts 对照腿
监看 format_num 全局变更对 font computed 串化的位移）

**电池归属（12:36 双发竞态仲裁，NOTES-knife-q2-race-arbitration.txt 在档）**：
本刀电池由链 C scope `zeroweb-dev-guard-q2c-123902`（12:39 发射，日志
knife-q2-battery-r2-1240.log）独跑收账——发射方身份不改变账目归属（认领
凭 CLAIM+判据+法证链），shapes b0/b1 JSON 单一终态写者（12:39/12:40，145
页 parse-ok）。bins 为 12:14/12:15 双建（b0 无 Q2 面、b1 有，fv1/A3r WIP +
刀 A3b 原始守卫形双测**双侧同态**）——A3b R2 收窄（bezier-only 守卫）晚于
双建窗，未入本电池 bins；其 P→F 修复面（after-change steps −3、
currentcolor linear −2）在本电池两侧对称出现、ΔP 内对消，不属本刀账。

**十四腿收账（13:12 GATE-DONE 零 LEG-FAIL，results-knife-q2，链 C 单写者）**：

| 语料 | b0 | b1 | ΔP | F→P | P→F | only-b0 | only-b1 |
|---|---|---|---|---|---|---|---|
| **css-shapes（靶）** | 5677P/1837F/1T | 5915P/1599F/1T | **+238** | +238 | 0 | 0 | 0 |
| css-transitions | 820P/696F/17T | 同 | 0 | 0 | 0 | 0 | 0 |
| css-animations | 516P/781F/6T | 516P/780F/7T | 0 | 0 | 0 | 1 | 1 |
| css-backgrounds | 3430P/2816F | 3431P/2815F | +1 | +1 | 0 | 0 | 0 |
| css-masking | 4651P/1704F | 同 | 0 | 0 | 0 | 0 | 0 |
| css-typed-om | 11072P/1287F | 11074P/1285F | +2 | +2 | 0 | 0 | 0 |
| css-fonts | 5739P/2502F/4T | 同 | 0 | 0 | 0 | 0 | 0 |
| **TOTAL** | 31905P | 32146P | **+241** | +241 | **0** | — | — |

**认领口径 +238**（=CLAIM 判据声明的 shapes 靶腿全账，页集合双侧恰等零
lost；不因 favourable 搭载扩认领范围）。靶页级 F→P 分解（三类根因覆盖面
超 p1 探针预估 168F——同三根因打中 p1 未列页）：
ellipse-004 +63、circle-004 +63（Fix C 精度）、inset-001 +46、ellipse-005
+30、polygon-004 +18（Fix A calc 静态叶）、inset-003 +6（精度+calc）、
circle-005 +4、shape-margin-001/002 +4（Fix B ch）、inset-004 +2、
shape-arguments-000 +2。

控制腿：transitions/masking/fonts 三腿 SET-IDENTICAL（fonts 对照腿实证
Fix C 全局 6-sig-fig 对 font computed 串化**零位移**）；animations 页级
Timeout 形对称 ±1（Element-getAnimations async 形摆动既有注记）。
favourable 搭载 +3 分账：typed-om ×2＝animation-delay/transition-delay
秒值串化——gCS 走 format_num 单源，旧千分位截断丢精度
（`assert_approx_equals expected -0.00314 but got -0.003`）、新 6 位有效
保留；solo 双 bin b0 29P/2F → b1 30P/1F 恒、电池+solo 双观测同向
deterministic（探针档案 q2-probe-typed-om-delay/）——机制关联成立，
**按 CLAIM 判据范围（shapes 主账）不扩认领，挂 favourable 备注**
（后续刀可显式认领）。backgrounds +1＝background-computed.html
bg-position-x calc 多层串化（fv1 serialize 域跨跑双向漂移已知页，A2/A3
先例第三次）——solo b0 26P/13F == b1 26P/13F 恒等不重现，窗噪不认领。

**全账零 P→F 零 lost——判据（shapes 主账 + 六腿零 REG 零 lost + 钉测
常驻）全绿。**

## 残面池（不认领，挂观测）

- **css-shapes 腿残 1599F**（b1 计）：本刀三类靶面外（box/value 解析面、
  shape-* 相邻属性、reference-box 组合面），维持 Q p1 池口径，后续刀另行
  侦查分类。
- **background-computed calc 多层串化漂移家族**：A2（bg-position-x）、A3
  （bg-position-y）、本轮（bg-position-x）三次出现，fv1 serialize.rs 域
  固有非确定性，跨跑双向漂移不认领。
- **layout 本地 ch=0.5 换算**（layout-engine `table_borders.rs` / inline
  finalization）与本刀 style-system ch=1.0 兜底口径不一致——无语料驱动，
  挂池待驱动页出现再统一。
- **`format_num` ≥1e6 数量级科学计数口径** FIXME（%.6g 对齐 Chromium 待
  语料驱动）。
- **`Rch`/`rch` 根相对 ch 兜底**维持 0.5（本刀不动，css-values-4 根字体
  口径待驱动）。
- **engine clippy 在案 3 lint**（script-sandbox es_module too_many_arguments
  归 fv1、animation.rs:2452/2453 unnecessary_cast 归 2d3e71d19）清偿待对端
  落账。
- **animations 腿页级 Timeout 形 ±1 对称摆动**（Element-getAnimations
  async 形）既有注记维持。
