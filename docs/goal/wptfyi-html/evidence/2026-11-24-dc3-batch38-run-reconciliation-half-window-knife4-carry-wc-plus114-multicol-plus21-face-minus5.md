# DC-3 批三十八收账——刀③/shim③ 首入窗自报逐字兑现（WC +114、multicol +21）、face −5 落 messagechannel/track-cues、窗码图破形＝刀④ 半窗携带新形首例

2026-11-24。run 45345696071（07:08:29Z dispatch，HEAD 6ea86a6ea＝批三十七收账
提交，coverage-only `-f run_wptfyi_coverage=true`；09:23:21Z completed
success，历时 ~2h14m52s＝回历史带 2h12-17m 带内，批三十七 ~2h01m 快边未复
现＝font-mmap 候选不定谳维持）。上一会话 09:19Z 因 API 429 中断于 run 在飞
＋下载已布防点，本会话零损失接续（waiter/autodl 后台进程跨越中断存活）。

## 窗码图（非均匀窗首例＝码刀半窗携带）

55 提取＝**40× 6ea86a6ea（dispatch HEAD，07:08:46-47Z 起跑簇）＋14×
85c872f28（dom 流域刀④，07:12:52-07:17:05Z 晚起簇）＋1×
ee5c29562（aggregate 唯一 09:22:55Z，携 dispatch 后刀④⑤⑥＋css multicol
三刀＋PR #322＋docs 全部，不跑套件＝零套件面影响，批三十一同形）**。

**破形课**：刀④ 85c872f28 于 dispatch 后 **+2m36s**（07:11:05Z）落树，晚起
14 分片 checkout 拉到含刀④的 main＝**批三十八起「均匀窗」前提不再默认成立**
。布防判据更新：dispatch 与他流在途码刀存在竞态，收账时必须按分片 SHA 逐分
片甄别归因（本批实践：14 携刀④分片清单全留档，face 变化分片逐一核对携带
码域）。

**携刀④分片清单**（14）：css-v2-face-a（css-borders）、css-v2-face-c
（css-multicol）、form-encode-sjis、editing-run-3/4/6/7/8/9/10、
service-workers、catch-all-5、catch-all-7、catch-all-11（xhtml）。face 变化
分片核对：css-multicol +21＝shim③ 96ee337b4 自报逐字（shim③ 为 dispatch
HEAD 祖先、两码变体皆含＝归因不受破形影响）；corner-shape +24 所在
css-v2-face-a 携刀④但刀④域＝DSR/attachShadow 与 css 边框插值无涉＝wobble
候选读数照记；editing er0/er1/er2、dom、html-sem（WC）、html-infra、media、
net-api、IndexedDB 各变化分片全 6ea86a6ea 均匀侧＝归因纯净。

**窗内码刀**（对批三十七套件 checkout 62fd84670 全量携带，祖先链 dispatch
前预核对过）：

- **a3e299907 fix(dom) 刀③**（05:18:48Z）＝registries 刀③，自报 WC
  +114P/−31F 六页全限定、effective-global-registry 48F→0F 页全绿。
- **96ee337b4 fix(engine) shim③**（05:58:45Z）＝columns 裸 'auto' 端点豁免
  （纯谓词 `_zgvT==='auto' && kebab==='columns'`），自报 WAAPI 21 键 F2P
  净 +21P。
- **73f4d5dcf fix(engine) 恢复**（06:13:00Z）＝刀③ CE 源面还原（批三十七
  跨流碰撞收尾），恒等性照读窗。

## A/B（基线 /tmp/batch37/run37，167 文件两侧、pair 计数 2,050,863/2,050,503）

套件面总 P **1,948,937→1,948,712（−225）**＝**本窗零 zw_only 差**（套件面
＝aligned 分子＝CI zw_passes 三口径精确闭合）；组行归因：

| 组 | ΔP | 归因 |
|---|---:|---|
| web-components | **+113** | **刀③ a3e299907 自报逐字精确兑现**：registries 六页 +114P（Document +1/Element +2/define-customized-builtins 0→14/define 0→48/scoped-registry-append +1/**effective-global-registry 18→66＝自报 48F→0F 页全绿逐字**、full_pass 文件 +1）；套件 F −33 vs 自报 −31＝+2 差（DSD regs 截断形 −1 等）；regs +80 全落两 define 页注册满额（1→70/1→15）；WC 分片 html-sem 不携刀④＝刀④ 的 WC 域兑现顺延批三十九 |
| css-multicol | +21 | **shim③ 96ee337b4 自报净 +21P 逐字兑现**：columns-interpolation 363→384（regs 384 恒＝F2P 21、P2F 0 手术级）、full_pass 文件 +1（该页全绿）；分片携刀④但 shim③ 两变体皆含＝归因不受破形影响 |
| css-borders | **+24** | corner-shape-interpolation 133→157（regs 478 恒＝**F2P 76/P2F 52 双向点位翻转**＝值级 wobble 形非注册解锁；133 多窗持位破形＋shim③ 谓词不含 corner-shape（非 columns×auto）＝清单外单窗不定谳，批三十九二读；分片携刀④域无涉） |
| css-transitions | +2 | properties-value-003 37→39（column-width F2P ×5/P2F ×3 双向点位＝清单外 wobble；shim③ 谓词不含 column-width）＋event-dispatch regs 30→17 P 恒（wobble 带 22→30→17） |
| css-backgrounds | +1 | background-computed 24→25（批三十六 +1/三十七 −1/三十八 +1＝镜像震荡三连带） |
| css-writing-modes | −1 | text-input-vertical-overflow 9→8（regs 10→9，**−1 二连＝慢截断 drift**，批三十九三读） |
| css-forms/css-pseudo | ±1×2 | checkbox −1/radio +1、backdrop +1（full_pass 文件 +1）/marker −1＝镜像对 wobble |
| css-animations | 0 | event-order regs 1→4 P 恒 wobble |
| dom | **−123** | Range-mutations-dataChange 978→909／replaceData 962→908，regs 同步 979→910/963→909＝注册截断形；分片均匀侧 |
| editing er0/er1/er2 | −63/−34/−158 | multitest ?9001-last 1092→1029（regs −63）/forwarddelete ?7001-last 388→354（regs 470→423 截断三读）/delete ?7001-last 658→500（regs 734→537 第七读截断加深）；变化分片 er0/1/2 全均匀侧 |
| html-infra | −4 | messagechannel.any.worker.js 115→111（regs 137→134＝已知单键负载抖动带截断形） |
| html-rendering/media | −1 | track-cues 1→0（Pass→Fail「chronological order」＝交替带第十六演 F，十六演带 F P F P F F P F P P F 续不追） |
| shadow-dom | −1 | declarative-shadow-dom-opt-in 第八腿 109→108（regs 119→116＝截断伴生；108↔109 震荡带续） |
| IndexedDB | −1 | idbcursor_iterating 1→0（键级 TO＝批三十七 +1 长持位首翻**二读回吐＝timing wobble 定谳**回 0 长位） |
| net-api (xhr) | −1 | send-timeout-events 1→0（批三十七 +1 清单外镜像闭合＝wobble 对） |
| selection | 0 | 48,902 持位（stringifier_editable regs 8→12 P 恒＝注册 wobble；新平台定谳维持五读） |

## face 与快照

- **html face 109,417→109,412（−5）**（构成＝html/infrastructure messagechannel
  −4＋html/semantics track-cues −1；rendering 零变化）。coverage 口径净
  **+38,316**/基线 71,096，验收 ≥152,096 余 **42,684**。
- 快照 coverage-2026-11-24 **同日覆写** 09:23:15Z 落 **c9c584ed0**（weekly
  cron 提交形）；**manifest 恒 e00a7b14e8 零漂移（第十窗持位）**；文件分母
  127,561 恒、子测试分母 2,279,165 恒、covered 74,189 恒；full_pass
  63,652→63,652 恒（css 全过 +2＝multicol columns-interpolation 页全绿＋
  css-pseudo backdrop，IndexedDB/semantics/xhr −1×3 补平）；zw_passes
  1,930,484→1,930,259（−225）。
- 组行闭合（CI 快照 groups diff）：custom-elements/registries +114＋
  css-multicol +21＋css-borders +24＋css-transitions +2＋css-backgrounds +1
  −css-writing-modes 1−dom/ranges 123−editing/run 255−html/infrastructure
  4−html/semantics 1−shadow-dom/declarative 1−IndexedDB 1−xhr 1＝**−225**
  ＝套件面＝分子三口径闭合。
- 本地旧 cache（a3830cd8f7 分母代）face 行双证：html **109,412**（den
  160,124）、zw_passes **1,930,259**＝与 CI 逐字同值（分子与分母版本解耦维
  持）；**护栏未触发**（coverage exit 0）。

## 观察清单例行读

- **R2 二读**：css 动画面域 +829 全域持位（sizing/values/align/transform/
  transitions 零 P 变化）；**contain-intrinsic-size-interpolation −49
  P2F 候选持位＝真形定谳**（非 timing wobble，归渲染流域双向照例行）；
  **img-sizes-auto 0 持位＋svg-inline 408 持位＝批三十七新负形定谳**（归
  渲染流域双向照例行）。
- TT 1,482 六读持位；domparsing 1,264／sanitizer 414 四读持位；CE·SD
  idl-setters 225P/2,685F/2,910regs **第十四窗逐字持位**＋reactions/Node、
  shadow-root-clonable、property-reflection 族逐字＝73f4d5dcf 恢复恒等性
  照读兑现（零行为面）；DSD-opt-in 第八腿见上；selection 五读持位。
- per-global 3P/3F 持位；webappapis 1,540 恒；dat 170 键 shapes 恒（TO
  148/Fail 19/Fail-Pass 3）；variant nameset DISTINCT 恒；interestfor 恒；
  test262/csp 恒值＝**负向两形第十二窗零回位**；**合成 Fail 集恒
  103=103**（19 套件分布同法互证）零新增零消失。
- **停摆带第 15 采样**：dom/ranges 42,292→42,169（regs 42,583→42,460）
  带内＝批三十六 regs+P 双升新变体**连续两窗未复现**（回吐截断形候选收
  敛）；editing/run 95,358→95,103（regs 96,547→96,240）**下缘外穿 −27＝
  带暂记更新 95,103~95,442**。

## 布防第八窗（一课）

pipeline：waiter poll28 09:24:56Z completed success→RUN_DONE；autodl
marker 09:25:12Z，r1 首件 blob 瞬断一败（egress 形）r2 补齐 167/167、
flattened=167 done 09:37:25Z、autodl.out 余无错误＝韧性下载器第三窗连续
零事故。**新布防课＝刀④ 半窗携带**（详见窗码图）：dispatch 后他流码刀
+2m36s 落树即破均匀窗——后续批间 dispatch 前后窗语义以「分片 SHA 逐分片
甄别」为准，不再默认均匀；本批 14/54 携带清单已留档为破形归因样板。
shas.sh 55 行验收通过（40+14+1）；aggregate ee5c29562 本地对象库在册
（先 fetch 判课未触发）。批间脚本族 cp -p＋exec 位＋路径三课连续三窗零复
发。跨流碰撞：刀④⑤⑥＋css multicol 三刀对 wptfyi-html 文件域零触碰
（第四起未发生）。

## 批三十九预布

窗外已落码六刀全祖先链待全分片携带窗读数：**刀④ 85c872f28**（自报 WC
+6P/−6F 四页全限定，本批 WC 分片未携＝首读顺延）＋**刀⑤ f909812e0**（自
报 WC **+2,436P/−2,775F** fixture 族八页全限定＝大刀）＋**刀⑥ 0d138a405**
（自报 WC +31 面：define 22F→0F＋接口页 8 面外溢）＋css multicol 三刀
**4a3bef038**（电池 F2P 98/P2F 0）/**3829434b7**（F2P 14/P2F 0）/
**e134036a5**（F2P 6/P2F 0）＋PR #322 perf hit_test（timing 归因干扰变量）
＋c9c584ed0 快照 chore（非行为）。观察清单承接：corner-shape +24 二读（定
谳窗）、text-input-vertical-overflow 三读、停摆带第 16 采样（editing/run
新带 95,103~95,442/dom-ranges 42,022~42,677）、TT 七读、domparsing/san 五
读、CE·SD 第十五窗＋DSD 第九腿、selection 六读、负向两形第十三窗、合成
Fail 扫描、track-cues 十七演、R2 三读（img-sizes-auto/contain-intrinsic
真形持位读）、刀④⑤⑥⑥ multicol 三刀归因双向扫（WC 大面读数＋溢出照例行）。
