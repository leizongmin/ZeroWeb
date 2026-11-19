# 刀 animation-range 族（css-animations/parsing，2026-11-19 12:4x 认领）

**认领**：NOTES 12:4x 条目；residue-survey 候选 #2。typed-om 域让位
togrid face B/C（其 evidence 明文全占）。a3 刀#2（keyframes-name-valid +
animation 简写）同语料不同页避让。

## 账（pristine css-animations.b1.json）

| 页 | P/F | 主形态 |
|---|---|---|
| animation-range-shorthand.html | 22/111 | 简写展开+双向序列化 |
| animation-range-start-computed.html | 0/30 | gCS 文本 |
| animation-range-end-computed.html | 0/29 | gCS 文本 |
| animation-range-start-valid.html | 21/5 | canonical 序列化 |
| animation-range-end-valid.html | 20/4 | canonical 序列化 |
| animation-range-start-invalid.html | 0/11 | 拒收 |
| animation-range-end-invalid.html | 0/14 | 拒收 |
| animation-range-shorthand-end-zero-offset.html | 3/3 | 边角形 |

style-system 全零支持（grep 无 animation-range）；part10 仅 list-valued
两行既有（'animation-range-end':1 @1861，只读不动）。

## 断言机制（parsing-testcommon.js）

- test_valid_value：纯 CSD——`style[prop]=v → getPropertyValue` 须等于
  canonical 串；再 set(readback) 往返一致。**无 typed-om 无 computed**。
- test_invalid_value：set 后 getPropertyValue 须 ''（拒收丢声明）。
- test_computed_value：set 指定值 → `getComputedStyle().getPropertyValue`
  等于期望（计算值语义）。

## 文法（scroll-animations-1 §view-timelines-ranges）

- 长手 `animation-range-start/end`：
  `normal | [ <timeline-range-name> <length-percentage>? ]#`
  名集 cover/contain/entry/exit/entry-crossing/exit-crossing。
  名必须在百分数**前**（'50% exit' 拒）；normal 互斥独占（'normal 10px' 拒）；
  单段内一名一数（'contain contain'、'cover 50% enter 50%' 拒）；
  'none' 拒。
- 简写 `animation-range`：`<start> <end>?` 逗号列表；单值=start、end
  默认 normal；'normal normal'→'normal'；'normal 100px' 拒。

## canonical 序列化规则（valid 页实证）

指定侧（CSD readback）：
1. `0` → `0px`（单位补全）
2. start 侧省略 0%（'cover 0%'→'cover'）；end 侧省略 100%
   （'contain 100%'→'contain'）
3. calc percent-first 排序：'calc(10px + 10%)'→'calc(10% + 10px)'；
   乘积同样（'calc(sign(100em - 1px) * 1%)'→'calc(1% * sign(100em - 1px))'）
4. calc 同族简化保壳：'calc(10% - 10%)'→'calc(0%)'（简写页指定形）
5. 逗号列表逐段规范化

computed 侧（gCS）：
1. em→px（'entry 1em'→'entry 10px'）
2. calc 同族折叠去壳（'calc(41% + 1%)'→'42%'；'calc(1em + 10px)'→'20px'）；
   百分+px 混族保 calc（'calc(10% + 10px)' 原样）
3. 省略规则同指定侧（'COVER 0%'→'cover'，大小写规范化）
4. 'entry calc(10% - 10%) entry calc(50% + 50%)' computed→'entry'
   （start 0% 省 + end 100% 省）

简写重组：逐段 start/end 配对，end 为 normal 默认时省略（'10% normal'→
'10%'）；start/end 皆默认→'normal'。

## 接线图（animation-direction 先例全链）

1. css-parser values：枚举 + parse fn（AnimationDirectionValue 先例
   parse_basic.rs:1247；新文件或 parse_misc.rs 域）
2. style-system types.rs DeclaredValue 变体（AnimationDirectionList 先例
   @1784；types.rs 已 2451 行超 2000 限——新增类型考虑 types/ 子目录或
   css-parser 侧承载，避免加剧）
3. registry.rs 初始值（@196 邻域）
4. inherit.rs 继承臂（@1447 先例）
5. matcher/supports.rs CSS.supports 臂（@71 先例）
6. apply.rs validate_decl_value 写门臂（@2243；apply.rs 4877 行）
7. engine js_dom_bridge/computed_style.rs gCS 文本臂（@1213
   enum_list_to_css 先例；简写重组 @1228 animation_shorthand_to_css 先例）
8. style-system shorthand/transition.rs animation 简写重组（@224/@307
   mk() 先例）——animation-range 简写在此或兄弟模块
9. parse.rs parse_animation_range_*（parse.rs 1686 行，+臂后 <2000）

## 风险与避让

- togrid face B/C 在飞（part10 12:39 M）；若扩散到 parse.rs/css-parser
  并发窗→碰头信号记 master.md 错峰。
- calc percent-first 排序/简化复用既有 calc 机制（css-parser calc 流
  已落库 parse_layout.rs face A）；fold 复用 serialize_math 既有函数。
- computed em→px 需字号因子（computed_style 现有 em 解析路径复用，
  font-size 16px 默认）。
- P2F 哨兵：typed-om/pva/masking/transitions 四语料（togrid 电池同面）。

## 计划

1. 深读接线点（transition.rs/computed_style.rs/inherit.rs/supports.rs/
   apply.rs 门/css-parser parse_basic 模板）
2. css-parser 类型+parse（含 canonical 序列化函数）
3. style-system 接线（types/registry/inherit/supports/apply 门/shorthand）
4. engine computed 文本臂
5. 钉测（style-system + engine lib，togrid 三钉模式）
6. 探针 zz-ar-* → test-guard 构建 → 单页验证
7. A/B 电池（css-animations b0/b1 + 四哨兵）→ evidence 回填 → pathspec
   commit → guard → master.md

## face A 探针定谳（13:05，bin md5 3295cee6）

实现面收敛为三件（比接线图 9 站更薄——文法/canonical 单源 css-parser，
style-system 只加 validate 门）：

1. `crates/css-parser/src/values/parse_animation_range.rs`（新）——
   `parse_animation_range_side`（split_top_level_commas + normal 独占门 +
   逐段 1/2 token 分派）、`canonical_offset`（calc→fold_math_specified
   Length 槽；sign 树乘积 narrow 兜底 reorder_percent_product_front——
   FIXME 通用 percent-first 乘积排序挂账 css-parser calc canonical 域；
   裸数补 px；`1em` 数/单位拆分状态机——'e' 贪指数会吞 'em'）、
   `animation_range_side_to_css`（侧别省略：start 0%/end 100%）、
   `fold_animation_range_specified`
2. serialize_math.rs fold_decl_specified 加 animation-range-start/end 臂
   （语料驱动注释）
3. apply.rs VALIDATED_PROPERTIES 66→68 + 写门分派臂（css-wide 早放行于
   门上游不受影响；var() 走既有 substitution 早放行）

钉测 `css-parser/src/values/tests/animation_range.rs` 5 枚：start_valid
（23 案）、start_calc（3 案含 sign 重排）、end_valid（18 案含 end 100%
省略）、invalid_reject（14 形双侧）、fold_none_on_invalid——5 passed
0 failed（test-guard 包裹 cargo test）。

探针腿（test-guard 构建 release bin + css-animations 全语料单腿 +
pristine b1 逐页 diff）：

| 页 | pristine | face A bin | Δ |
|---|---|---|---|
| animation-range-start-valid | 21P/5F | 26P/0F | +5 |
| animation-range-end-valid | 20P/4F | 24P/0F | +4 |
| animation-range-start-invalid | 0P/11F | 11P/0F | +11 |
| animation-range-end-invalid | 0P/14F | 14P/0F | +14 |

**TOTAL gained=34 lost=0 regressed=0**——四长手页全数归绿，shorthand/
computed 页零扰动（face B/C 域），全语料零 P2F。发射遇 togrid face B
在飞编译红（parse.rs:351 E0631，13:01 落）扫入→NOTES 碰头条目+
retry-on-green 守望（systemd zw-arfa-probe），其 13:0x 自愈后 attempt 1
即绿（13:05:18，构建 13:03:51 DONE）；bin provenance=HEAD+RIDE WIP
（es_module/part115 惰性面，face A 探针账用）+togrid face B 已自愈态
（grid 域与 css-animations 语料不相交，diff 表零扰动实证）。

face B 设计（已侦察，未实现）：ComputedStyle 存 canonical 指定文本
`Vec<String>` ×2，gCS 臂逐段 parse_animation_range_side→偏移
parse_length→`length_to_css(lv, font_size_px)`（computed_style.rs:2193
已含 % 保留/calc 同族折叠去壳/相对单位 resolve 全口径）→变换后重判
range_offset_elides；六站接线照 animation-direction（default/inherit/
apply_advanced/gCS 臂）。face C：shorthand 展开+读回重组待 face B 后。
