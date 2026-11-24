# DC-3 批三十七收账——engine R2 css 动画面 +829 大解锁、刀②续 WC +20 自报逐字、face −9 落 img-sizes-auto 新负形

2026-11-24。run 45326847882（04:30:19Z dispatch，HEAD 62fd84670＝批三十六收账
提交 rebase 后，coverage-only `-f run_wptfyi_coverage=true`；06:31:40Z completed
success，历时 ~2h01m21s＝历史带（2h12-17m）下缘外新快边，font-mmap perf 候选
观察不定谳）。上一会话于 06:15Z 因 API 429 中断于 run 在飞点，本会话零损失接续。

## 窗码图

55 提取＝**40× 62fd84670（dispatch HEAD）＋10× 411be250a（批三十七布防
docs，+4m01s）＋4× 21987fec4（rendering-compat R6120 docs，+7m39s）＋1×
656f8669（aggregate 唯一，~06:31Z 起跑）**：411be250a/21987fec4 对前序互
diff 均 docs-only（master.md／rendering-compat.md +7）＝等码归并＝**54 套件
分片全均匀 62fd84670**；656f8669 唯 aggregate 捡到（其 checkout 携带
dispatch 后三码刀，不跑套件＝零套件面影响，批三十一同形）。

**窗内码刀（对批三十六套件 checkout ef0df1118 全量携带，五刀全祖先链预核对过）**：

- **3e8cdfc12 fix(engine) 刀 R2**（04:23:18Z）＝canonicalize 往返写面补臂
  （LP 直写臂 ×5：border-*-radius ×4/column-gap；text-decoration-color 色
  槽；transform-origin/box-shadow 委托 apply_advanced_property_value；
  interpolate_lp_token_strict 纯 % 分支）＋锚定保留读（登记失败不再毒化
  from 锚）＋追刀 style_change_advances_anchor＝本窗主刀，css 动画面大解锁。
- **6a8df8cee 刀②续**（03:52:15Z）＝registries/shadow-dom 五件修复，自报
  WC +20P/−20F 六页全限定。
- **45efa2584+cd65c9f47 columns B-cut 对**（03:13/03:27Z）＝column-height
  inherit 臂，自报 F2P=13 超门＋CSS Animations 3。
- **05d32ab5c perf(font) mmap 驻留**（03:37:02Z，PR #321）＝timing 归因
  干扰变量，无 face 签名预期。
- 9ed72ec31 快照 chore＝非行为。

## A/B（基线 /tmp/batch36/run36，对计数 150,582 两侧互证后采信）

套件面总 P **1,947,791→1,948,937（+1,146）**，组行归因：

| 组 | ΔP | 归因 |
|---|---:|---|
| css-sizing | **+485** | R2 锚保留起动＋纯 %＋applier 委托：min/max-width/height composition 16/24/16/19→52、interpolation +38~+98、contain-intrinsic-size computed +7/valid +3、stretch 0→4；**负溢出候选：contain-intrinsic-size-interpolation 432→383（−49 全 P→F，regs 恒，全部 auto Npx from-to 形）＝R2 锚链 P2F 候选单窗不定谳** |
| css-backgrounds | +403 | R2 box-shadow 委托臂（interpolation +240/composition +65）＋border-*-radius 直写臂（interpolation +98）；background-computed −1＝批三十六清单外 +1 二读回吐＝wobble 对闭合 |
| css-values | +229 | R2 纯 % 分支：calc-size interpolate-size 族 composition 0→4/6/8/16、interpolation +37~+80；random-computed +1 清单外；interpolate-size-which-value −1 |
| css-transitions | +89 | R2 自报 F2P=93（1049→1142）；CI +89＝−4 环境差（event-dispatch regs 15→30 注册扩张 P 恒 wobble＋TO −1）＝自报 vs CI 环境差照例行如实记 |
| css-align | +87 | R2 column-gap 直写臂：column-gap-interpolation 74→142、composition 0→16、gap-animation-00x 0→1×3 |
| css-transforms | +46 | R2 transform-origin 委托臂：transform-origin-interpolation 10→56 |
| css-multicol | +16 | B-cut 对自报逐字：columns-interpolation 350→363＝**F2P 13 超门翻转收账形兑现**＋multicol-gap-animation-00x 0→1×3＝CSS Animations 3 |
| web-components | **+20** | **6a8df8cee 刀②续自报逐字精确兑现**：六页全限定 registries/shadow-dom 簇（effective-global-registry 10→18＝自报 56F→48F 逐字／append +3／attachShadow +2／declarative-basic +3／reference-target-basics +3／clonable +1），F −20 镜像 P2F=0；刀②首读（批三十六）零 face，续刀兑现 |
| editing er0/er1/er2 | +8/−18/+107 | 停摆带交替：er2 delete 551→658（regs 596→734＝截断回吐＋regs+P 双升）；er1 forwarddelete 406→388（regs 491→470＝截断形首读） |
| html-rendering | **−9** | **img-sizes-auto 12→0 新负形首现**（12 全部 `- contain-intrinsic-size` 子测试 P→F，expected "" got "none"/"30px 15px"）＋svg-inline 405→408（regs 639→648 回扩＝批三十六收缩形回吐＝wobble 定谳）；face 唯一负构成 |
| 小项 | +2/−2 | idbcursor_iterating 0→1 长持位后首翻单窗不定谳；send-timeout-events 0→1；font-size-animation −1；text-input-vertical-overflow −1（regs 截断）；css-pseudo/css-forms ±1 镜像对；css-animations P 恒（event-dispatch regs 32→22、event-order regs 4→1 wobble） |

**resolve 归属闭合（批二十四法）**：套件面 +1,146 vs aligned 分子
zw_passes +1,048，差 −98＝zw_only 键 border-radius-interpolation +98 与
idbcursor_iterating +1（不在 union/分片映射）；editing 三键分片归组计入
（+97）＝分子账 +1,047 ≈ CI +1,048（±1 组行噪声）＝两口径各自闭合。

## face 与快照

- **html face 109,426→109,417（−9）**（构成＝img-sizes-auto −12＋svg-inline
  +3；semantics/media 零变化）。coverage 口径净 **+38,321**/基线 71,096，
  验收 ≥152,096 余 **42,679**。
- 快照 coverage-2026-11-24 **同日覆写** 06:29:51Z 落 **92d5fce57**；
  **manifest 恒 e00a7b14e8 零漂移**（第九窗持位）；CI 分母 2,279,165 恒、
  covered 74,189 恒；full_pass 63,643→63,652（+9）；zw_passes
  1,929,436→1,930,484（+1,048＝aligned 分子）；护栏未触发。
- 本地旧 cache（a3830cd8f7 分母代）face 行双证：html **109,417**（den
  160,124）、zw_passes **1,930,484**＝与 CI 逐字同值（分子与分母版本解耦
  维持）。

## 观察清单例行读

TT 1,482 五读持位；domparsing 1,264／sanitizer 414 三读持位；selection
48,902 四读持位＝**新平台定谳**（+44 四窗稳定）；CE·SD idl-setters
225P/2,685F/2,910regs 第十三窗逐字持位；web-animations 526 持位；track-cues
第十五演零变化＝P 持位；dialog/onblur/semantics 零变化；per-global 3P/3F、
webappapis 1,540、dat 170 键 shapes 恒、variant DISTINCT、interestfor 恒；
test262/csp 恒值＝**负向两形第十一窗零回位**；停摆带第 14 采样
dom/ranges 42,608→42,292（regs 42,899→42,583）带内＝**批三十六 regs+P 双升
新变体未复现**（回吐截断形）＝单窗形不定谳续；editing/run 95,261→95,358
带内；**合成 Fail 集恒 103=103**（同法两窗互证）零新增零消失。

## 布防第七窗全绿与跨流碰撞第三起

- 布防全绿：批三十六两课（exec 位 `cp -p`、绝对路径启动）落地首演零复发；
  waiter poll25 自动写 RUN_DONE；autodl 06:33:39Z marker→54/54 首轮全 ok
  零 fail；flattened=167 done；autodl.out 零错误。
- **shas 新观察形**：aggregate SHA 656f8669 不在本地对象库＝他流 push 未
  fetch——「对象存在性检查」遇不在对象库须先 `git fetch` 再判，与
  cache-key 误抓形（对象根本不存在）区分。
- **跨流碰撞第三起**：96ee337b4（css 流）wholesale 落库碰撞误删他流面
  （本流批三十六 evidence 两文件＋rendering-compat＋dom 刀③源面）——对端
  73f4d5dcf 还原源面、6bccb8472 接岗轮还原 9 路径 docs（含本流两文件）；
  本会话自还原 commit 被 rebase 判「补丁内容已在上游」丢弃＝零损失。

## 窗外码刀与下窗承接口（批三十八观察清单）

三码刀均落全部分片起跑后＝唯 aggregate 捡到＝批三十八全分片携带预布：
**a3e299907 dom registries 刀③**（05:18:48Z，自报 WC +114P/−31F、
effective-global-registry 48F→0F 页全绿＝dom 流域按域双向扫溢出照例行）＋
**96ee337b4 shim③ columns 裸 auto 豁免**（05:58:45Z，自报 WAAPI 21 键 F2P
净 +21P）＋73f4d5dcf 恢复（CE 源面恒等性照读）。承接口：R2 二读（+829 域
持位＋contain-intrinsic-size-interpolation −49 P2F 候选定谳窗＋
img-sizes-auto 定谳窗：回位＝timing、持位/加深＝真形归渲染流域溢出照例行）；
B-cut 二读（multicol 1,267 水位）；刀②续二读（WC 17,688 水位）；刀③首入窗
双向扫；domparsing/san 四读；TT 六读；CE·SD 第十四窗；selection 新平台例行；
停摆带第 15 采样；er1 forwarddelete 截断二读；idbcursor 二读；track-cues
十六演；负向两形十二窗；合成 Fail 扫描；快照侧照旧（coverage 同日覆写；
本地旧 cache face 行双证；护栏命中 GUARD_EXIT 时以 artifact 收账）。
