# DC-3 批四十收账——均匀码窗＝六码刀＋PR #323 全分片携带读数兑现（刀⑦ +18/刀⑧ +15/kc5r87 +19 逐字精确＋cssom 刀 F2P 2 兑现＝吸收形预布修正）＝face 109,404 持位零变化（刀面全在 html/ 外）＝负面三带（dom −931/editing −1,349 停摆带双下缘外穿＋gethtml −1,307 截断加深＋color −130 镜像带）＝run ~2h01m 批三十七快边带复现

2026-11-24。run 45405289405（13:37:06Z dispatch，HEAD 9dc352bfd，coverage-only
`-f run_wptfyi_coverage=true`；15:38:10Z completed success，历时 ~2h01m04s＝
批三十七快边带 ~2h01m 复现）。上一会话 dispatch＋布防后本会话在位等待接续收
账（waiter poll25 15:39:44Z 写 RUN_DONE；autodl 15:41Z 起逐件下载，r1 一件
blob 瞬断 45s 退避后 ok，16:00:26Z flattened=167/done；shas 55 行一次 DONE_
ALL，唯 1 行空提取＝单 job 定向补提取成功）。

## 窗码图（均匀码窗＝54 套件分片两 SHA 全 docs-only 变体）

55 提取＝**43× 9dc352bfd（dispatch HEAD）＋11× a538eb037（本流批四十布防
docs，13:42:24Z push 被晚起分片拉到）＋1× 371be6508c（aggregate 唯一
15:37:19Z 起＝aggregate-only docs-ahead）**。371be6508c 树面含窗外三码刀
（kc5r88 3cd307a7c5＝14:18:41Z、dom 刀⑨ a06ccc8c75＝14:58:05Z、gCS px 锚
a5c66047b7＝15:27:29Z）但 aggregate 不跑套件＝零套件面影响（批三十一/三十
八/三十九同形第四演）。**54 套件分片码面逐字同 9dc352bfd＝均匀码窗**，六刀
＋PR #323 全分片携带＝归因窗纯净。窗外三码刀全数顺延批四十一携带窗。

## 六码刀＋PR #323 读数（全部自报量级兑现）

- **刀⑦ f348bd327：+18 逐字精确**＝CustomElementRegistry.html P 27→45
  （regs 46→46 恒＝19F→1F 页面自报逐字；group 行 custom-elements
  554→572＝+18＋full_pass_files +1）。
- **刀⑧ a9fa36cff：+15 逐字精确**＝custom-elements/registries/
  element-mutation.html P 0→15（regs 15→15 恒＝15F→0F 页全绿；group 行
  registries 2,245→2,260＝+15＋full_pass_files +1）。
- **kc5r87 7342c3fcb：css-overflow +19 逐字精确**＝overflow-shorthand-001
  P 0→16（16 键全对组合页全绿＝F2P 16）＋overflow-valid P 15→18（validity
  构造性增益 +3）；group 行 css-overflow 292→311＝+19＋full_pass_files +2
  ＝自报 F2P 19 精确逐字兑现。**清单外正溢出**＝cssom/overflow-serialization
  +3（4→7，读面家族反查外溢）。
- **kc5r86 4213170a7：css-transitions 净 +3 候选读**＝properties-value-003
  P 37→41（+4）＋properties-value-inherit-003 22→21（−1）＝净 +3 与自报
  F2P 3 精确量级吻合；−1 为 P→F timing 尾巴单窗不定谳。
- **cssom 刀 61a7edb2b：+2 兑现＝F2P 2 精确**＝multicol
  columns-shorthand-reset-wrap.html P 1→3（单键双翻转）。**批三十九吸收形
  预布修正**＝预判「multicol1/multicol2 两键已被 multicol 三刀先行翻转，预期
  ~0」未全中：本窗该刀自有 F2P 2 兑现，批三十九键名甄别勘误（实为
  shorthand-reset-wrap 单键，multicol 三刀 batch39 翻转的 wrap 键非此键）。
- **PR #323 220945b8a：零行为面兑现**（env 门控默认关，无未解释位移）。

## face 与三口径

**html face 109,404→109,404＝持位零变化**（20 组行全零动＝本窗六刀全在
html/ 外：custom-elements/css/dom/editing/shadow-dom；净 +38,308/基线
71,096、验收 ≥152,096 余 42,692 恒）。快照 totals：zw_passes
1,932,775→1,929,060（**−3,715**）＝A/B 套件面总 P **−3,712**（组行 vs 套
件口径差 −3 小项照记）；full_pass 63,662→63,670（+8＝css-overflow +2/
css-masking +2/css-multicol +2/custom-elements +1/registries +1/editing
+1/css-selectors +1/css-color −1/dom-ranges −1）；covered 74,189 恒＋分母
127,622/2,279,447 恒＝**manifest 恒第十二窗零漂移**；pass_rate
84.79%→84.63%。本地旧 cache（a3830cd8f7 分母代 127,454/2,278,580）复现：
**face 109,404＋zw_passes 1,929,060＋full_pass 63,670 三行逐字同＝双证成
立**；护栏未触发。

## 负面三带与停摆带第 17 采样

- **dom/ranges −931＝停摆带第 17 采样下缘外穿**（Range-mutations-dataChange
  1211→716＋replaceData 1146→710；带暂记 **41,778~42,709**，批三十六双升
  变体未再现＝回吐截断形）。
- **editing/run −1,349＝下缘外穿深落**（er0 −202/er1 −408/er2 −739＝
  delete?7001-last 586→20 深截；带暂记 **93,762~95,442**）。双腿同窗深负＋
  run ~2h01m 快边并现＝负载/时序带续证（批二十六定谳维持）。
- **gethtml −1,307（5995→4688）＝截断二读加深**（批三十九 −913；CI 页预算
  截断环境差形随负载浮动，非回归——本窗 WC 刀净 +33 全数兑现）。
- **css-color −130＝镜像带**（color-computed-hsl 3753→3623；批三十 −136/批
  三十一 +136 同键带三读，TO 0→1 伴生）。
- **css-flexbox −110＝新截断形候选**（abspos/position-absolute-013
  P 110→0、regs 432→1＝页级注册塌缩；单窗不定谳，二读批四十一）。
- **清单外正向带 ~+50 单窗不定谳**＝animation interpolation 族宽 +1 带
  （css-masking +12 七键/css-motion +5/css-shapes +3/css-fonts +3/css-
  backgrounds +3/css-ui +2/css-sizing +1/writing-modes +1）＋
  **getAnimations 1→17 回位**（+16＝批三十三形交替带三读兑现）＋
  corner-shape 135→141（+6＝157↔135↔141 震荡带三读）。与 run 快边并现＝
  timing 正向带候选，与刀域（anim/transitions）重叠不切分。

## 观察清单（批三十九承接口逐项）

TT 八读 **1,482 持位**；domparsing 六读 **1,264 持位**（刀⑨窗外未携＝正确
持位）；san 持位读 **415**；CE·SD 第十六窗 **回位持位读兑现**＝
property-reflection-idl-setters 2,910＋imperative-setup 2,910 双持位；DSD
第十腿 108→**109**（108↔109 交替带续）；selection 七读 **48,902 持位**；
track-cues 十八演 **3P 持位**（十键逐键同值＝十七演回 P 后首演持位）；onblur
6P/dialog 键面逐键同值＝水位带持位；dialog-popover-closedby-complex 12 持
位＝**−12 新面二读未复现＝单窗不定谳关闭**；img-sizes-auto 0 持位＋
contain-intrinsic-size-interpolation +1 带读（归渲染流域照例行）；负向两形
第十四窗 test262/csp **恒值持位**（2 页 message 文本变体：blob-url/
repeated-imports fetch module URL 抖动＝页级键集 GONE 0/NEW 0）；dat 170
键/per-global 3F 挂账/webappapis 1,542 持位；**合成 Fail 103=103**（82 页
级键集逐字同）。快照侧照旧（coverage-2026-11-24 同日覆写 15:37:38Z 落
c1e98b9c7c zero-web-bot；manifest 恒）。

## 布防窗记事

韧性下载第四窗＋1 瞬断（blob r1 form-encode-euckr 45s 退避后 ok＝批三十一课
续零损失）；shas 55 行一次 DONE_ALL＋1 空行定向补提取（批三十三课行数验收执
行有效）；waiter poll26 内 RUN_DONE 正常；本会话在位等待收账＝零跨会话损失。
布防脚本自纠一坑记档（批四十布防会话）：多进程 nohup 启动时 `cd X && nohup A
& nohup B` 把 cd 后台化致 B 落错 cwd 静默失败＝绝对路径调用＋启动后 ps 核对
（本窗已按此执行）。

## 批四十一预布（窗外码刀＋快照 chore 携带窗）

- **kc5r88 3cd307a7c5**（自报 css F2P 26 精确+3 增益/P2F 0/净 +29＝
  random()/random-item() 族三刀）。
- **dom 刀⑨ a06ccc8c75**（自报 domparsing 净 +48P/−48F 三目标页全收敛＝
  domparsing 1,264→~1,312 首读）。
- **gCS px 锚 a5c66047b7**（css random 消flake＝F→P NET +1 per kc5r89
  收账）。
- **kc5r90 10417f5e0**（收账 fetch 新见＝自报 NET +8＝random-item 选位
  感知 var 预写＋注册 custom 替换放行落表，a100de9a1 收账）。
- 快照 chore c1e98b9c7c（非行为）＋批四十一布防窗 docs（非行为）。
- 归因双向扫照例行；gethtml 三读＋position-absolute-013 二读＋color 四读＋
  getAnimations/corner-shape 四读＋停摆带第 18 采样（新带
  41,778~42,709/93,762~95,442 回位读）＋TT 九读/domparsing 刀⑨首读/CE·SD
  第十七窗/DSD 第十一腿/track-cues 十九演/负向两形第十五窗照旧。
