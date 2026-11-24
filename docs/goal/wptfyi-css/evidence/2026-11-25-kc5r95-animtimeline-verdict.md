# kc5r95 verdict——animation-timeline 属性＋view()/scroll() 刀（10 触点，inherit_property 不加）（2026-11-25）

## 结论

- **刀面**（指纹 `ZW-KC5R95-ANIMTIMELINE-FACE`，电池代 b1 bin md5
  0ee0d454c54f5635ef25598fda802519＝R95/R94 双指纹并存 strings=1/1；
  b0=kc5r94/bins/r94-fontpalette-runner 8f0f1f01af4ce7b9a01a6fc1d552a2aa
  R95 指纹=0）：
  1. registry.rs 三表：known_properties + "animation-timeline"
     （CSS.supports 属性名面＋`all` 展开清单 +1＝第 5 采样点）＋
     initial_value `OptionalString(Some("auto"))`＋is_inherited **false**
     （is_inherited 表零编辑——css-scroll-animations-1 #animation-timeline
     Inherited: no）。
  2. computed_style.rs 字段 `pub animation_timeline: String`（存
     keyword/--dashed-ident/scroll()/view() 原文）。
  3. default_impl.rs "auto"。
  4. inherit.rs 单臂：apply_initial_value 复位臂（**inherit_property
     不加臂＝非继承属性**，与 font-palette 继承形反向、与 vtn dashed-ident
     同向）。
  5. apply.rs 求值臂：substitution 词放行（oracle 防线——键形 A specified
     含 random 词须放行到 random 折叠层）∥ auto/none 小写归一 ∥ 裸
     dashed-ident（`is_custom_ident_token` r93 资产三度复用）∥ scroll(
     前缀 ∥ view( 前缀**原文 store 零折叠**——长度叶 min>=max r92 钳制
     在 random 层已折叠，apply 层零二次处理，与 font-palette % 叶 calc
     折叠形成对照）。
  6. apply.rs validate_decl_value：白名单 [&str; 110]→[&str; 111] +
     "animation-timeline"＋分派臂（同款五臂）。
  7. matcher/mod.rs is_property_supported 臂（r93 教训①连续第三刀布防
     在先 as-run 零落空）。
  8. engine js_dom_bridge.rs decl_supported 前置臂（委托
     validate_decl_value，r93 教训②连续第三刀）。
  9. engine js_dom_bridge/computed_style.rs gCS 串化臂（:390
     random_computed 表查询属性无关在前自然承接；字段臂收纯形）。
  10. lib.rs `#[used]` 指纹（random_integer_typed 不加＝无 integer 位）。
- **布防预测账 as-run 零修订全兑现**（r94 之后连续第二轮预注册零落空）：

  | 门 | 布防预测 | as-run | 判 |
  |---|---|---|---|
  | G-F2P | =2 精确（:432 键形 A＋:478 键形 B） | **=2** 刀内精确，out-of-registry=0 missing=0 | 兑现 |
  | G-P2F | =0 fail-stop（无 % 形键→表零新挂；简写静态清单零扰动＋animation 族插值页零读字段＋all·supports·view-timeline 关联键 0 三证） | **=0** | 兑现 |
  | 哨兵 | 'Shared by property' Fail 集 2→**1**（color-mix 独持；animation-timeline 移出） | **=1** | 兑现 |
  | held-F | color-mix＋font-palette 键 2（r94 % 保留形承接）＋descriptor 4 键持 F | **6 键全持 F**（held-F-bad=0） | 兑现 |
  | G4 | SET-IDENTICAL：random 页自动豁免 | **非豁免页 diff=0** | 兑现 |
  | G5 | GONE/NEW=0 | **9882/9882 = 0/0** | 兑现 |
  | NET | +2（6743→6745） | **6743→6745 = +2** | 兑现 |

- **探针门 probe-impl 七块九断言门双臂三跑定谳**：run1 E3 Fail＝探针页
  工件面 bug（`parseFloat('view(29713.16…px …')` 返 NaN 使界内断言恒挂；
  引擎串完全正确——draw 29713.17px∈[1,300000]、557864.28px∈[300000,
  600000]）＝改正则 `/^view\(([\d.eE+-]+)px ([\d.eE+-]+)px\)$/` 提双数
  核界；run2 发现 v1 段尾行残留同款 bug 再修。run3 终判 b0 E1-E6 全 F
  （E7 gCS 名单 in 检查恒 P＝冒烟非翻门）＋b1 七块全 P＝六门双翻全过
  （E4 非继承守卫＝父 view(…) 子读 auto；E5 initial＋all→auto；E6 文法
  round-trip 五形 auto/none/--MyTimeline/view(10px)/scroll(root)；E9
  scroll( 前缀收单证）。探针页电池发射前 trap-rm（PROBE-PAGE-CLEAN=0），
  终页归档 probe-impl-final.html。
- **电池双腿** css-values RC=0/0 满量 257 页/9882 case（b0 03:46:52→
  03:49:52、b1 03:49:52→03:52:52，各 3min）＝**judge VERDICT PASS /
  verify VERIFY-PASS**（judge 五门＋verify 恒等式 +2==2−0＋基线 6743
  与 r94 收账逐位钉死）。judge/verify 预注册硬码零 bug 零修订一次性
  通过（连续第二轮）。
- **cascade-precheck 6-run**（b0 98/98/101、b1 97/98/98 分布互叠＝
  r92/r93/r94 连续第四刀同款 HashMap 迭代序 flake 非注册回归；all 展开
  第 5 采样点位移 watch 结案）。
- **门禁**：clippy 两触点包（zero-style-system＋zero-engine
  --all-targets）RC=0 warning/error 计数 0；fmt --check RC=0 全仓零
  diff（r94 登记残面 2 位点既有债已由对端 cf5dc15ad fmt 清偿笔清偿＝
  fmt 清偿笔挂账结案；本刀自有 fmt 债=0）。

## 挂账移交

1. **键 2 % 叶保留形分歧**（font-palette 引擎级域外）——延续。
2. **descriptor 层 4 键** @font-palette-values parser 刀域——延续。
3. **继承×挂表子元素链 FIXME**——延续。
4. **wrapper 族余 1 键**：color 相对色（css-color-5 域最大）＝族末刀
   候选。
5. **DC-3 周账** 12-01 窗口。

## 工件

- `.acceptance/wptfyi-css/kc5r95/`（本地不入库）：NOTES-kc5r95.md
  （§零机制/§一 10 触点/§二预测账/§三探针门/§四移交/§五收账）、
  judge-r95.py／verify-r95.py（预注册硬码）、judge-r95-run.log／
  verify-r95-run.log（复跑存档 RC=0/0）、battery-r95.log（双腿 RC=0/0
  ＋发射四件套）、probe-impl-r95.log（三跑）、cascade-precheck.log
  （6-run）、probe-impl-final.html（探针页终形）、bins/r95-animtimeline-runner
  ＋GENERATIONS.txt（gen1 0ee0d454… 定谳后零重建）、results-k95-{b0,b1}／
  probe-results／cascade-precheck（JSON 结果集）。
