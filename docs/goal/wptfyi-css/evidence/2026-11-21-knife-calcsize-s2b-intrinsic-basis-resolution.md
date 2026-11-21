# 刀 CALC-SIZE-S2B：calc-size 非 definite 值承载槽＋布局期 basis→px 解析（2026-11-21 14:3x 收账）

## 池

post-S1 残池（css-values calc-size 族，census 源 ZeroWeb-3
results-knife-calcsize-resolved/b1r css-values.json，108F/8 页）：本刀域
＝flex-basis-on-row 19F＋on-column 16F＋height 16F＋width 10F 中
**intrinsic/%/嵌套 indefinite basis 形**（definite-px basis 形已由 s2a 半
刀 ride 面覆盖——本 clone 树含 s2a 在飞 face，双臂同载相消后本电池度量
恰为 intrinsic 半）。box-sizing 15F/typed-om 19F/interpolate 2F 不在域
（预注册 SET-IDENTICAL 面）。

仲裁：NOTES-arbitration-calcsize-s2-overlap.md（14:1x 第三方 fork）——
apply.rs calc-size 臂归本刀（14:10 先声明），s2a（14:12）definite-px
代换半分层让位；页面族分割见认领笔。

## face（patch `knife-s2-calcsize-basis.patch`，md5 918edf623b108b79f3fa03e796b14b84，5 文件）

- **style-system apply.rs**：`calc_size_slot_capture`——definite_sum None
  落点（intrinsic 关键词/`%`/嵌套 indefinite basis），文法复裁合法形
  （calc_size_args_valid 单源）声明原文入承载槽＋受理声明；非 calc-size
  形 None 落原生臂零漂移。`clear_calc_size_slot`——后声明胜出（definite
  match 尾单点＋native 全臂＋none 臂清位）。pub 导出
  `calc_size_parts`/`calc_size_substitute_size`（layout 期单源消费）。
- **computed_style.rs**：`CalcSizeSlots` 七槽（width/height/min/max×2/
  flex_basis）＋`ComputedStyle::calc_size_slots`（anchor_size_slots 同构，
  None 零成本）。
- **layout-engine engine/calc_size.rs**（新文件）：
  `apply_calc_size_basis_resolution` pass——sizing pass 组消费（base
  authority 后位：其门读 CSS FlexBasisValue::Auto，先行会覆写 slot 解析
  basis）；stack 携父 content 双轴长＋父样式（flex 主轴 column 判定，
  flex-basis %/stretch 语境随主轴）；basis 表 `any`=0/`stretch`=父
  content/min/max-content=intrinsic_sizing 既有测量/fit-content=
  min(max,max(min,cb))/`auto`=width 声明 definite px 或 max-content/嵌套
  内层先解/length-percentage 经 CalcContext；sum `size` 叶代换 basis px
  （style-system 单源 ident 边界）→parse_calc→CalcContext（%＝父
  content、em＝本元素字号）→负钳 0 写 taffy 尺寸。kill-switch
  `ZW_CALC_SIZE_BASIS=0`。
- **engine.rs wiring**：SizingTargets.calc_size 预扫描位＋re-run 组并入。
- **高度轴 fail-closed**：min/max-content 高度无测量机器（FIXME 随高度
  intrinsic 刀扩展），height 槽 intrinsic basis 形保现行为。

## 电池（knife-s2-calcsize-basis-battery.sh，log knife-s2-calcsize-battery-1437.log）

14:37 发射（zw-knife-s2calcsize-battery-1437）。判别门：FM
'CALCSIZE-S2 basis-resolve' b0 0/b1 ≥1＋M2 双臂 md5 异＋patch md5 绑定；
对账门：页级 P 计数＋calc-size 靶族 P2F＋css-variables SET-IDENTICAL。
corpora：css-values 主靶＋css-flexbox（flex_basis 写入 blast 哨）＋
css-sizing＋css-variables。同 clone uax14ba 电池相位兼容（其双臂 bin
固化后纯只读 legs，与本刀树变更零交集）。

## 1437 interim 电池（首跑，face v1）——GATE FAIL 门证

14:37 发射首轮：FM 门过（b0r=0/b1r=2）＋M2 过，判读门
**TARGET-P2F FAIL**（css-values 双臂 diff F2P 15 / P2F 228；interim
results 存 `results-knife-s2-calcsize-basis-interim1437/`）。门按设计
逮住 v1 face 三处缺陷，逆向机制：

1. **interpolate-size 伪通过池翻面（222 P2F）**：min/max 插值四页
   actual/expected 双 target 同带级联基，b0 拒收→两侧同落级联基
   双巧合通过；v1 capture 门级受理后不走 native 臂→expected 侧 gCS
   从级联基翻初始值。sampler/applier 对 calc-size 串写不进——动画
   通道非肇因。
2. **auto basis 轴漏（on-column `calc-size(auto, size)` P2F）**：b0
   flex-basis 回落 height 声明 50px 恰中期望＝P；v1 auto 臂恒读
   width。
3. **% basis auto-container P2F**（height 页 `calc-size(31%, size)`
   auto-container）：b0 回落 content 10px 恰中 expected_auto＝P；v1
   % 臂无 definite 门解出 3.1px。

## v2/v3 修面（battery→mechanism→fix 正当叙事）

- **v2 轴修**（calc_size.rs）：auto 臂读本轴声明（height 槽/column
  flex-basis → s.height，css-flexbox §flex-basis:auto 主轴尺寸属性）；
  % basis × 高度轴 × 父高度 indefinite → fail-closed None（§8 不可解
  % → 整 calc-size indefinite → b0 现行为）；auto-no-decl 与高度轴
  intrinsic 全 fail-closed（保 b0 等价，真高度 intrinsic 测量挂账
  S2c 刀——on-column 残 ~12 形域）。
- **v3 捕获收窄**（apply.rs）：capture 只收 width/height/flex-basis
  （gCS used-value echo 覆盖＋域池恰集）；min/max 退出捕获→
  interpolate 四页字节级回 b0。域池零损失（F2P 15 形全落三属性）。
- 钉测：style-system 5P（s2b pin 收窄断言更新）＋layout-engine 3P。

## v2 电池（重跑，判据轮）

patch `knife-s2-calcsize-basis-v2.patch`（md5 9061b93cf79f195d59d48044f7efc699，
864 行，-R check＋scratch 树 -R 语义验证）。共享树电池排队协议：launcher
zw-knife-s2calcsize-v2-launcher-1510 等 genfam/uax14ba-r4 双 EXIT＋90s
静默复查后发射（uax14ba legs 含 css-values/css-sizing 语料交集，A/B 窗
横跨其 face 剥/挂周期会破坏 ride 相消）。

v2 实绩（15:41-15:57）：**GATE FAIL——F2P 16 / P2F 5**。P2F 逐案归因＝
真实解算 bug：①`auto` basis 无 definite 声明落 max-content 宽度测量；
②高度轴 intrinsic 用宽度测量（size 常数 ≈123.3px，×1.5/×1.2/÷2 三案
全吻合）；③box-sizing 未建模（19px/7px 同得 203px）。预注册「该域 b0
全 F 无 P2F 面」误判（nested/auto 高度形存在 b0 级联基巧合通过）。

## 收账

v2 判据轮 FAIL → v3 修复分支（auto 三分支＋高度轴 fail-closed）由
执行 fork d2452915 预注册后车道孤儿化，本 clone 1849 于 18:53 接手重发
（zw-knife-s2b-v3-relaunch-1849），19:09 **GATE PASS——F2P 16 / P2F 0**
（预注册主数精确兑现，v2 两枚 auto 臂翻转实证）。face 五文件已随
be355a87d 入库；门证据、16 枚 F2P 逐页明细与哨兵账见
[basis-resolution 全量证据](2026-11-21-knife-calcsize-s2-basis-resolution.md)。

[控制面](../master.md)
