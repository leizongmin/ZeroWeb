# HTML、XHTML 与 DOM 接口控制面

**2026-11-26 · DC-3 批四十八收账（run 45658084700 concluded
failure，attempt-1 无 rerun）**：**net 分片 SUCCESS**＝batch47
肇事#1 被对端 b3fab8330 修复兑现；**test262-builtins 贴帽击杀
第三演**（6,309,664 KB>6 GiB 帽差 0.29%，三演全 <0.7% 贴帽带
＝确定性）→毒 JSON 二连→aggregate exit 3 拒写快照二连
（third_party 51907→28,831 同数字逐字重演）＝**快照未落，html
face 读数持批四十六 109,414（净 +38,318 余 42,682）**。A/B 健
全面 +1,614（css-mixins 158→476 +318＝k112+k113+k117..k128
十四刀链满账／TT 1,671→1,886 +215＝四刀归零精确兑现／gethtml
3,801→4,186 +385 大回位／css-values +47 逐字；负面带 css-color
−46 镜像/dom-ranges −70 第 24 采样外穿/corner-shape −12）＝待
下批快照落账。**卡点**＝test262-builtins 内存回归归属对端 TT/CSP
链（b3fab8330 洗 workers 面未洗 test262 面；二分已定谳 culprit=
built-ins，s10k25返修~返修三g＋刀㉚ 均未解），本流不硬修。
**门判定（22:44Z）＝CAPKILL_BALLOON 门不开**：adf767f5b 面
~70min cap 击杀（6,297,236 KB）＝s10k25返修+刀㉗+刀㉘ 未解；
**门判定二演（02:45-02:50Z）＝CAPKILL_BALLOON 门不开**：v3 链
run-1 HEAD 重取 a842bf953（返修三g）~72.3min 击杀
6,370,364 KB＝返修三g 未解；**门判定三演（04:22-04:27Z）＝
CAPKILL_BALLOON 门不开**：run-2 HEAD 重取 2ea852d1c（返修三g＋
刀㉚ 在面）73.6min 击杀 6,344,380 KB＝六源同带
6.30-6.43GB（击杀点带 72.3-73.6min 漂移＝中途存活读数不作数，
唯完跑/cap-kill 可判）；workers 腿三演 SET-IDENTICAL；B46 平行
点按判据均未触发＝dispatch 不发＝批四十九窗再顺延＝快照持
109,414。
**门重探四演在飞（05:25Z arm）**：上会话 04:56Z 曾 HEAD 重取
cde8a7bde 面（刀㉛ cbcccab7c 单 engine 刀），在飞 26min 时刀㉜
59d8acde6（fix engine）落树→面过期；本流 05:25Z 停杀该链（五
PID 同 kill＝全树净零孤儿，中途读数不作数＝无判定入账）后按
重探纪律 HEAD 重取 b07f9a77f（刀㉛＋刀㉜＋k140 面）重演＝
soak 自排在飞（B49_GATE_DONE 哨兵；pid 文件持现行 PID）。
**下一步**＝门重探判读（唯认完跑/cap-kill；判据带不变：CLEAN
rc=0 peak<1GiB json COMPLETE＋workers SET-IDENTICAL）→ 双 CLEAN
再 dispatch49.sh（pull→dispatch→RUNID→waiter/autodl/failarch/
shas-gated/finalarch 收账族）；CAPKILL 则守对端 TT/CSP 下笔
再演（刀㉛刀㉜ reactions 簇非 TT/CSP 域，六源贴帽带画像未变）。
**t262 气球目录二分收官（01:29Z）＝culprit `test/built-ins` 定谳**：
五臂逐目录判定 annexB/staging/intl402/language 全 RC_1 完跑峰
0.26/1.77/0.38/0.61GB，唯 built-ins 66min 贴帽击杀（guard 读数
6,434,960 KB→CULPRIT_TOP／BISECT_DONE）；runner 自身 VmHWM 击杀
臂全程 ≤0.61GB 平台＝6GB 单进程在子进程侧＝渲染子进程机制画像
再加证；跨面确定性三源＝8485db7a76 CI 二演 6,309,664＋探针 A
6,338,488@67min＋adf767f5b 本轮＝s10k25返修+刀㉗+刀㉘ 未解；
证据随 blocker 归对端 TT/CSP 链（嫌疑域 part12-trusted-types/
part14-trusted-sinks 已两刀未中）
（[批四十九二分证据](evidence/2026-11-26-dc3-batch49-t262-balloon-bisect-culprit-builtins.md)）。
刀㉙ be7080635（engine TT eval 门惰性装置）＋返修三/四/三f/三g
落树＝TT/engine 域码笔→门按 drift 甄别重开：v3 门链 soak 于
BISECT_DONE 后自动开启、HEAD 重取至 a842bf953（返修三g，docs-only
a5b2f8d0e 不在探面），build rc=0 全量探针在飞（冻结判形不变）
→02:45Z 判定 CAPKILL_BALLOON 见上门判定二演。
（[批四十八证据](evidence/2026-11-26-dc3-batch48-run-reconciliation-net-fixed-test262-cap-third-play-snapshot-refused-again.md)）。
批四十七收账（破窗＝attempt-1 双分片内存败 net V8 OOM＋
test262-builtins 贴帽击杀，0 字节毒 JSON→环比门闩锁拒写 exit 3
＝rerun ×2 确定性同败＝本地 workers RSS 二分定谳肇刀 9f4486302
（对端 dom 流 k20 补丁滞留形），E1/E2 反向 apply 收窄到
part14-trusted-sinks.js 单文件面＝gethtml 对照实验洗清截断簇＝
早期 A/B 刀面正面 +598＝对端修复笔 b3fab8330 七测 CLEAN
（175 files 696P SET-IDENTICAL）开门＝批四十八发射，发射头漂移
8485db7a76 三笔新码笔八测同 CLEAN 洗清＝run 45655664157 被
concurrency 取代以 45658084700 记账；五课落档含在途 log 须
concluded 后 rerun 前提取）
（[批四十七证据](evidence/2026-11-25-dc3-batch47-run-reconciliation-broken-window-double-memory-fail-culprit-knife-9f4486302-local-workers-bisect-latch-two-plays-snapshot-not-landed.md)）。
批四十六收账（五码刀全分片首跑窗＋rerun 自愈落账首演＝k18+k18b
dom/events 66→132 满仓 EXACT（k16 +11→k17 +55→k18 +66 三刀链
+132 收官＝对端刀⑱交叉互证闭环）＋k107 合面 +68＋k108 +3 逐字＋
k109 合面 +72（css-mixins 六键 +52 领衔 dashed-function-eval
0→30＋css-values random-in/sibling +20）＝刀面 +209 全 html/ 域
外＝face 109,425→109,414（−11＝semantics +1 dialog-overlay/
modal-dialog 回位双键对冲 rendering −12）＝净 +38,318 余 42,682
＝A/B/zw_passes/组行口径差 0 三连＝attempt-1 test262-language＋
aggregate 双失败→rerun --failed attempt-2 自愈＝aggregate 同
attempt 重评估自跑落快照 78a4588cf＝本地聚合兜底变双证＝gethtml
3,450→3,798 八读 +348）见
[批四十六证据](evidence/2026-11-25-dc3-batch46-run-reconciliation-five-knife-window-k18-full-66-132-k107-plus68-k109-plus72-ab-minus184-gap-zero-face-minus11-aggregate-self-heal.md)。
批四十五收账（十二码刀全分片首跑窗＝k106 ζ 5fb707c0e MQ/CQ 条件
CSSOM 表面化双域 +114 首读兑现＝mediaqueries +90＋css-conditional
+24＋k17 44ba2a03d 激活链全分片首跑＝dom/events +55＝k16 +11→k17
+55 双刀链 +66 收官残 66F＝k18 域＝k103 γ2＋k105 ε random 150→154
满仓九读＋k104 δ corner −2 churn 九读＝A/B +490＝口径差 0 二连＝
face −3 全落 onblur/dialog 水位带＝gethtml +278 七读回位＝editing
四读出带收窄＝ranges 第 22 采样外穿＝快照 168142d04 同日第二覆写
＝布防第十五窗 shas 后台常规形课）见
[批四十五证据](evidence/2026-11-25-dc3-batch45-run-reconciliation-twelve-knife-window-k106z-dual-domain-plus114-k17-chain-66-verbatim-ab-plus490-gap-zero-face-minus3-water-band.md)。
批四十四收账（双 docs 分层＋k17 码刀 2 分片窄落窗＝刀⑮ TT +180 逐
字精确（Node-multiple-arguments 0→90×2 全帧内零帧损）＝k16
dom/events +11 逐字＝k99/k100/k102 random 三刀首跑合读 +10＝
k103/k104/k105 仅入 aggregate 零窗面＋k17b 完成后 +90s 落树未跑＝
A/B −1,077＝口径差 0 双向零差归零＝face +39 五页全列零刀面
（rendering +25 svg-inline 回满＋infra +10 messagechannel 回满＋
semantics +4＝onblur 5/dialog 3/track-cues-sorted 1 首过换员形）＝
editing 93,006 出带四读＝gethtml −564 六读再截＝css-color +70 七读
完美反对称封环＝flexbox −110 镜像回吐＝dom/ranges 42,185 第 21 采
样回带＝合成 Fail 四读 94 三连恒＝run 1h57m3s 快边新纪首破＝布防第
十四窗＋收账侧基线路径课留档）见
[批四十四证据](evidence/2026-11-25-dc3-batch44-run-reconciliation-k17-narrow-landing-tt180-verbatim-gap-zero-face-plus39-env-only.md)。
批四十三收账（均匀码窗 docs 分层携带变体首演＝13 分片 docs-ahead 码
面全同＝san 三刀合面 +122（刀⑬ ~+63 候选＋刀⑫ 链 +23＋刀⑭ +20 逐
字）＝刀⑬ 直触 html face（syntax +24）＝kc5r94 font-palette 五键
+65 首读＝A/B +2,146 单窗最大面＝口径差 +8 帧构成小形＝face −12＝
rendering/infra 截断 −35 抵消刀面 +25＝editing 94,088 满仓回带＋
ranges 41,837 第 20 采样＋gethtml +639 五读大回位＝TT 十一读 1,491
＝合成 Fail 三读＝布防第十三窗）见
[批四十三证据](evidence/2026-11-24-dc3-batch43-run-reconciliation-seven-knife-uniform-window-ab-plus2146-record-face-minus12-truncation-offset.md)。
批四十二收账（半窗携带第四演＝携码超集首演＝cf5 fmt +9m 三分片 2398 ⊂
cf5 ⊂ 7197 晚起分片亦带全五刀＝五刀全分片窗实锤＝san 双刀合面 +169
逐字精确（刀⑩ +140 三 tentative 页／刀⑪ +28 自报差 +2）＝口径差 +140
帧构成形首演（tentative 页三家分母帧外＝suite +169 vs 组行 +29）＝
kc5r93 合面 +39（vtn +20 差 −1＋ident() +19）＋kc5r92 math-depth +4
首读＋kc5r91 +5 差 +4＝face −2＝onblur 水位带二读＝gethtml 四读 +135
反转回位＝editing 截断搬家 er0↔er2＝dom/ranges 41,675 二次外穿＝TT
十读破形二连 1,488＝合成 Fail 新法 103=103 定义存档＝布防第十二窗）
见
[批四十二证据](evidence/2026-11-24-dc3-batch42-run-reconciliation-five-knife-full-carry-san-double-verbatim-frame-gap-140.md)。
批四十一收账（半窗携带第三演＝刀⑩ +5m28s 落树 6 晚起分片携带其套件分
片 baseline＝face 零现顺延批四十二＝携码刀套件分片归因新课＋刀⑨
domparsing +48P/−48F 逐字精确 1,264→1,312＋css-values 四刀合面 +37 vs
自报 +38 差 −1＝face −1＝script_009＋A/B −1,984 口径差 0 精确闭合＝
gethtml −1,726 三读加深＋editing 93,057 外穿二连＋dom/ranges 42,025
回带＋TT 九读破形 1,483＝布防第十一窗）见
[批四十一证据](evidence/2026-11-24-dc3-batch41-run-reconciliation-half-window-knife10-carry-domknife9-verbatim-face-minus1.md)。
批四十收账（均匀码窗 54 套件分片逐字同 9dc352bfd＝六刀＋PR #323 读数
全兑现（刀⑦ +18/刀⑧ +15/kc5r87 +19 逐字精确＋cssom 刀 F2P 2 兑现＝吸
收形键名甄别修正）＝face 109,404 持位零变化＝A/B −3,712 三口径闭合＝
停摆带第 17 采样双下缘外穿＝manifest 恒第十二窗＝布防第十窗）见
[批四十证据](evidence/2026-11-24-dc3-batch40-run-reconciliation-uniform-code-window-six-knives-face-hold.md)。
批三十七收账（五码刀均匀窗＝R2 css 动画面 +829 大解锁＋刀②续 WC +20
自报逐字＋B-cut multicol +16；face −9＝img-sizes-auto 新负形首现＋
svg-inline 回吐；A/B +1,146 resolve 闭合；布防第七窗＋跨流碰撞第三起还
原收尾＋run ~2h01m 新快边；窗外三码刀批三十八预布）见
[批三十七证据](evidence/2026-11-24-dc3-batch37-run-reconciliation-r2-mass-unlock-wc-verbatim-face-minus9-img-sizes.md)。
批三十六收账（三码刀窗＝columns 刀 +230 自报逐字精确＋d4bd1cec1 刀②归
因窗第一读零 face＝face 109,426 持位零变化（净 +38,330）＝A/B +611 精确
闭合＝停摆形第 13 采样 regs+P 双升新变体＋svg-inline 收缩新形首现＝快照
同日覆写落 9ed72ec31＝布防第六窗＋批间复制 exec 位丢失新课＝6a8df8cee
刀②续＋B-cut 对＋font-mmap 批三十七预布）见
[批三十六证据](evidence/2026-11-24-dc3-batch36-run-reconciliation-columns-knife-230-verbatim-face-hold.md)。
批三十五收账（三码刀均匀窗＝72957ce1e→domparsing +170＋587250e6e→
sanitizer +80＋7f8df21ce→web-components +90 自报逐字，face −1 全落
track-cues 十三演 flake 带＝43ca50c9d columns 刀唯 aggregate 捡到套件
未及）见
[批三十五证据](evidence/2026-11-24-dc3-batch35-run-reconciliation-triple-knife-uniform-window-face-minus1-track-cues.md)。
批三十四收账（零码窗 54 分片 60b3dc45d＝face −3 全落 flake 带＝dialog
第六演破形转震荡带＋onblur 5P→3P＋双引擎刀 72957ce1e/587250e6e 唯
aggregate 捡到＝停摆形第 11 采样交替互补＋editing +108 二读 delete 632
持位＝caret 定谳强化＋css 负腿二读不定谳＋A/B −376 两口径精确闭合＋
布防全绿第四窗）见
[批三十四证据](evidence/2026-11-24-dc3-batch34-run-reconciliation-zero-code-window-face-minus3-dialog-break-shape.md)。
批三十三收账（双码刀均匀窗＝be78d24cf＋a2295242d 全分片携带＝
a2295242d 双正向兑现 domparsing +923＋dom/ranges +403 registration
首次满额（批二十四移交对端 dom/ −350 修复兑现）＋san +3＋syntax +4＋
editing +108 单窗不定谳＝be78d24cf 嫌疑 css 新负腿两形首现＋face +2
键级构成 dialog 第五演兑现＋mapped-attribute 写入体域；A/B +1,457＝
coverage 分子 +528 两口径各自闭合（tentative 不入 aligned 分子）；
DP①② 五窗零变化 watch 以码因关闭；分母漂移 +588 另报不入账；布防
课两条＝A/B SIGPIPE 丢失须 `> file`＋shas.sh 缺 cd 假完成须自带 cd＋
行数验收）见
[批三十三证据](evidence/2026-11-24-dc3-batch33-run-reconciliation-dual-knife-window-domparsing-plus923-face-plus2.md)。
批三十二收账（face +3 全落 flake 带＝track-cues 第十演回 P＋onblur
五连 3P 破形跳回 5P＝5P↔3P 水位震荡形；零新码窗＋全分片携带
a3932b528＝TT 面首码归因读四面全零动＝编译适配零行为面定谳；A/B
−180 组行闭合；停摆形第九演 editing +258 尖峰证伪；css −49 非镜像
新腿；shas.sh 收紧 fallback 首演零误抓；布防全绿二窗）见
[批三十二证据](evidence/2026-11-24-dc3-batch32-run-reconciliation-face-plus3-flake-keys-and-tt-compile-fix-zero-face-read.md)。
批三十一收账（face 持位零变化 109,425＝双 SHA 窗真码刀 a3932b528
落全部 54 套件分片起跑后唯 aggregate 捡到＝套件面零码归因维持；A/B
+298 组行五项闭合＝css 带精确镜像回位＋停摆形第八演双边外穿；负向
两形第五窗零回位；egress 503 逐件韧形下载课＋cache-key 误抓第三演
＋布防全绿一窗）见
[批三十一证据](evidence/2026-11-23-dc3-batch31-run-reconciliation-face-hold-and-code-knife-outside-suite-shards.md)。
批二十八收账（DP①② 双向账 domparsing +17／CE +24 双精确＋san +8
清单外正溢出＝组行 +49 vs 自报 +41；停摆带外上穿 +306）见
[批二十八证据](evidence/2026-11-23-dc3-batch28-run-reconciliation-dp12-verbatim-and-stall-band-breakout.md)；
批二十七收账（TT 两刀双向账 +114＝③③b +88 精确＋③⑤ +11＋交互窗
+15；负向两形移交；face −4 全落 flake 带）见
[批二十七证据](evidence/2026-11-23-dc3-batch27-run-reconciliation-tt-dual-knife-verbatim-and-negative-spillover.md)；
批二十六收账（③b②/③③a 兑现 tt +87＋CE·SD 第二窗持位＋停摆形三窗
交替震荡 timing 定谳＋track-cues 勘误）见
[批二十六证据](evidence/2026-11-23-dc3-batch26-run-reconciliation-tt-knife-second-window-and-stall-oscillation.md)；
批二十五（TT/Sanitizer 刀簇双向账＋CE·SD 香窗移交）见
[批二十五证据](evidence/2026-11-23-dc3-batch25-run-reconciliation-tt-cluster-both-faces-and-ce-sd-spillover.md)；
批二十四（per-global +3 闭环＋编辑刀首窗＋dom 碰撞移交）与 TO 面
根因定谳见
[批二十四证据](evidence/2026-11-23-dc3-batch24-run-reconciliation-perglobal-editing-knife-and-dom-collision.md)、
[根因定谳证据](evidence/2026-11-23-dc3-batch24-speculative-to-face-rootcause-probes.md)；
批二十二/二十三见
[批二十二证据](evidence/2026-11-23-dc3-batch22-run-reconciliation-three-fixes-and-rt-spillover.md)、
[批二十三证据](evidence/2026-11-23-dc3-batch23-navigator-object-support-closure.md)。
批一 html/dom +1,888；批二 rendering 通道 +2,551；批三 window 变体源通道 +11；
批四 webappapis +1,537；批五 browsers 簇 +40；批六 semantics 补簇 +1,955 + 未引
号 variant 解析修复；批七 offscreen 主线程面 +1,062（对账 +1,046）；批八 html5lib
驱动三件套 +118（1,049 键零回归）；批九 `testharness-html-infra` 新通道
html/infrastructure + 五小簇 +623（193 键分母全量对平）；批十 inline 脚本
`pipe=sub` 臂补 `{{GET[name]}}` 注入（infra 套件 A/B Pass 550→665＝scheme +20/
history +6/eventsource#url +3，对批九基线净 +42，其中 +13 为 messagechannel
.any.worker.js 键级截断恢复抖动带，零回归）；批十一 syntax template 子树定向腿
+42（19 键/329 子测试全执行，19/19 键计数精确对平，22 键宽估修正为 19 键）；
批十二 speculative-parsing 族新通道（127 键分母新入账，chrome 过 118）：
126 执行键（负案 popup skip 1）**50 Pass / 76 TO、墙钟 ~15 min、A/B 两轮逐键
零漂移**——无扫描器下 pass 面为引擎 loader 怪癖（iframe/嵌套 write 管道
`document.write('<plaintext>')` kill 不生效＋template 取数不惰性，探针实证），
leg-2 engine 修后此面重排（[批十二证据](evidence/2026-11-22-dc2-batch12-syntax-speculative-parsing.md)）。
批十九跑道勘误二新增共享面决策项：语料 pin 315976933 后上游漂移面
15,429S（七簇整目录、canvas wide-gamut 族、forms 新形等）——WPT_REV
bump 属 goals/lib.sh 共享面（五 goal 脚本共用），bump 与否、何时 bump
待用户或跨流碰头拍板，本流不单方面动。
批十二勘误（批十三重跑实测）：route_speculative_stash take 臂漏设 body →
JS fetch text() 恒空 → 7 键假 Pass，重跑 126 执行键 **43 Pass / 83 TO**
（A/B 两轮零漂移），净记账 **50→43**，7 键全 document-write 形（真实头
序列比较生效后两 put 头序列不等 → TO 0 Fail；嵌套 write 管道头序列差为
engine 碰头输入新增一形）；批十三 query-encoding 族四端点备料腿
（resource.py/blank.py/stash.py/css-tmpl.py 上游 1:1 + green-2x2.png 与
media 5 资产补档）：infra 全量 A/B 132 键 before 665/352/151 → after
**674/376/138 净 +9 P**，18 变更键全 query-encoding 面、其余 114 键
（含 messagechannel 抖动键）逐键零漂移（xhr +3P/eventsource +3P/loading
+3P/css 8 TO→F——端点上线前 404 挂起 TO、上线后 poll 收敛断言面曝光，
css 余 97 TO 待 stylesheet 载入链碰头刀）
（[批十三证据](evidence/2026-11-22-dc2-batch13-query-encoding-py-fixtures.md)）。
批十四备料核查（无码笔，DC-3 对账前）：批九 pin_missing 6 键 gh api 按目录
实证 pin 315976933 内不存在（collections/scripting/shared-array-buffers/
obsolete 四目录列表均无）＝上游漂移 bucket（同批五 13 键处理），pin 内不可
拉取、随周跑另报；TO 面 83 键勘误后构成＝document-write 45 + page-load 38
（7 翻转键全 dw 形，page-load 面持 38 不动；余量按批十二证据 img×text/plain
载入不完成＋头序列不等两形归因）。
批十四 CI 缺档腿（无 Rust 码笔）：11-21 快照 infra zw_passes 82 对本地 674
的差定谳＝分片语料缺 9 支撑文件（sab/utils/get-host-info/dispatcher/
subset-tests-by-key/IndexedDB support/structured-clone battery 三件；本地由
其他 goal 通道带入、CI 分片作业只跑本 goal fetch）——缺档形＝load support
os error 2（messagechannel/SAB 大簇）＋ subsetTestByKey 未定义 0 注册 TO
（query-encoding 5 页 × 16 include ＝ 80 键全灭）；三先期假设（限流洞/
截断/端点缺位）全证伪（fetch 失败 0、墙钟 106s 帽内、检出批九码）。62 脚本
+10 fetch_raw 补档（含本地同缺的 wpt-logo svg）；本地同码复跑 131/132 键
逐键零漂移（唯一漂移键 messagechannel.any.worker.js＝批十已知抖动带）。
select/options-length-too-large.html 的 0 注册 TO 本地/CI 同形（三份全量
产物＋过滤单跑核对均同一 Timeout，初稿「速度敏感嫌疑」表述勘误）＝engine
select 慢路径域稳定 TO，归 page-load TO 面候选形。补档后另派 coverage-only
周跑取新快照做 DC-3 对账
（[批十四证据](evidence/2026-11-22-dc2-batch14-ci-corpus-support-files.md)）。
批十五 CI 分片三通道缺档定谳（无 Rust 码笔）：控制点 44972755575 快照
infra 82 逐字复现批十四缺档形＝批十四「待实证」兑现；控制快照组基线
（zw_passes 1,914,204/2,276,004＝84.1%，infra 82、webappapis 3、browsers
217、semantics 8,508、rendering 2,793、syntax 2,747、xhtml 25,606、dom
59,634）。根因＝CI fetch 步只拉分片 `fetch:` 清单、套件直跑不经 Makefile
依赖——① syntax-dom 缺批四/五/六三 fetch 目标（run 实测 webappapis 语料
1 文件、补簇全缺＝三通道快照挂账全此形）；② speculative 缺 /common/utils.js
（catch-all-4 实测 126 F/0 P/0 TO「token is not defined」，本地同码 43 P/
83 TO）；③ 57/58/59 支撑文件闭包 14 文件（16 fetch_raw 位点，计分域实跑页
引用扫描）。方法论勘误：朴素「本地语料−CI 键」差集超计（rendering 556/
dom 259 误判缺档），按枚举规则模拟键域后 rendering 152=152 全对齐＝零行动，
56 误改已回退；py-routes 引用非缺档。验证：bash -n ×4＋GITHUB_TOKEN 幂等
重跑失败 0＋discovery 矩阵 syntax-dom 携全 5 目标、53 分片不变。
44983026495（批十五前码）取消（缺四通道必中间态；infra 通道输入同码＝
批十四验证力全额承接），改派 44985566993（cde99ad26，04:33Z）
（[批十五证据](evidence/2026-11-22-dc2-batch15-ci-shard-corpus-channels.md)）。
批十六 DC-3 对账收账（44985566993 五分片产物逐键 A/B，无 Rust 码笔）：
infra 82→674（132 键与批十三本地 after 逐项相同＝批十四兑现）、semantics
8,032→9,991（批六 231 delta 键逐键零漂移＝+1,959 face 兑现）、browsers
44→84（批五 +40 逐键兑现；agent-clusters ~55／open-features 13／
on-new-window 零注册 TO 与本地同形＝行为面挂账不重计）、syntax
2,748→2,791（speculative 勘误后 CI 首跑 43P/83TO 同形＋html5lib/template
基底零漂移）、xhtml 25,606／dom 59,644／rendering 2,795／CE·SD 11,361
持位；唯 webappapis 3→438 距本地 face 1,537 缺 1,099 定谳三形：
event-handler 8 页运行期 `fetch("/interfaces/html.idl")` 失败零注册
（−919 P，批十五静态 `<script src>` 闭包扫描盲区）、structured-clone.any.js
缺 common/sab.js（−148 P，os error 2；62 脚本不在本分片清单）、
base64.any.js 运行期 `fetch_json` 跨 topdir 引用
`fetch/data-urls/resources/base64.json` CI 缺档 → atob 节 94 子测试不注册
（−32 P；勘误：双 scope 初判有误）。批十六 fix＝57 脚本两 `fetch_raw`
补档（bash -n＋GITHUB_TOKEN 幂等重跑 FAILED_FETCHES=0）＋勘误补档
base64.json（b29107e7e），run 44996021189 在飞预期 face 438→~1,505
（base64 +32 随下批 run →~1,537）。另有同根因搭车面：html/dom
idlharness.https 三查询键 CI 零注册（A/B delta=0 初判未现），本地定向
单跑实证三键注册 5,822 子测试 1,397P，批十六 run 预期 +~1,394。邻居观察：
dom/ranges −13＝2 键子测试注册抖动形，归 wptfyi-dom 账本
（[批十六证据](evidence/2026-11-22-dc3-batch16-reconciliation-and-closure.md)）。
DC-3 preflight 三腿收账：syntax 腿与批八 after 字节级全同（1049 键
67,387/10,656/286 四项逐一相同）；infra 腿唯一漂移= messagechannel.any.worker.js
单键负载抖动（Pass 102→29 键级截断，键集不变）；rendering 腿对批二基线共享键
漂移 -62（legend-display -56、fieldset-display -11、video-aspect-ratio -1，
aspect-ratio 族 +6 回冲；+2 新键为批三 fetch 带入非漂移），归因渲染流域
（[批十证据](evidence/2026-11-22-dc2-batch10-query-encoding-getsub.md) +
[批十一证据](evidence/2026-11-22-dc2-batch11-syntax-template-leg.md)）。
DC-3 周跑接入预检完成：discovery 矩阵实证五通道落位（html-infra+rendering 入
catch-all-3、实体矩阵随零头打包漂至 catch-all-9、template 腿随 syntax-dom 手调
分片），修正 syntax-dom 预算截断 3600→13,200s（CI 11-20 快照实证 html/syntax
仅 40/439 文件入账；本地 93 min 实测）；批十二后矩阵重跑实证 speculative
通道零 CI 编辑入 **catch-all-4**（伴 import-maps，本地 ~15 min 对 5400s 预算
充裕）、实体矩阵漂至 catch-all-10（编号不锁）
（[接入预检证据](evidence/2026-11-22-dc3-weekly-integration-preflight.md)）。

**卡点** 解锁 UA 样式面 ≈4,400 的刀位＝engine shim/bridge 同任务 DOM 可见性
（js_dom_bridge 深面＝零-web 流核心域且其 WIP 正在这些文件），本 goal 流不单方面
改，待跨流碰头/用户点名；批八新增同域刀位：Node.normalize 上移共享 Node 原型
（js_dom_shim 面，363 F）、document open/write 管道挂起（engine 域，110 键 TO；
批十新证 base-href 面「This document is an XML document」doc.write XML 拒绝
同域）；批九新增刀位：带查询串样式表载入链挂起（engine stylesheet fetch/加载
管域，?include=css 105 TO）、HTMLAllCollection 可调用语义（40 F）、
structured clone/transfer 深面（47 F）——engine/JS 域待碰头。批十
query-encoding 刀位分解后余量归属：非特殊 scheme 文档编码越界 ~54 F +
`'href' in elm` 原型面 + search setter（js_dom_shim/bridge 面待碰头）、UA 样式
background 属性 8×3 F（rendering 流域）。批十三已把 resource.py/blank.py/
stash.py/css-tmpl.py 四端点备料落地（infra 面 +9 P；css 面 8 TO→F 后余
97 TO + 8 F 落 CSSOM 门；收益余量待 iframe/doc.write/CSSOM 碰头刀兑现）。rendering 腿 -62 漂移待渲染流域定向定谳与修复（不入本流回归账）。
批十二新增 engine 刀位（speculative-parsing 战役图，待碰头）：预加载扫描器
刀（leg-2 主刀，兑现上限 +118）、iframe/嵌套 write 管道
`document.write('<plaintext>')` kill 不生效（与批八 write 挂起同域不同形）、
template 内容取数不惰性（批十一 223 F owner-document 面的取数侧同源）、
非图片响应 img 载入不完成 → 页面 load 不触发（img 管道域）、嵌套 write
管道两 put 头序列不等（批十三勘误后 7 键 TO 形，勘误前被 take 空体遮蔽）。

**下一步** DC-3 收口两步：① 批四十九窗——**dispatch 门＝双探针
RSS 对照链双 CLEAN**（batch48 门盲区课：batch47 只测 workers 面
开门，batch48 破在未测的 test262-builtins 面）。**碰头信号记入
（2026-11-26 03:15Z 后）**：另一恢复会话已于 02:56 在同 clone 先
起 test262 A/B 探针对（探针 A＝p8 二进制面 8485db7a76 跑
test262-builtins〔T262_PROBE_DONE 哨兵〕；判据＝完整 JSON＋
RSS<6 GB），本流同刻起的两点对照链已主动停杀让路
（`t262-aborted-chain-*` 存档；**同 clone 串行纪律＝重测试探针
互斥，并发跑互相污染 RSS 测量并换装 target/release 二进制破坏
对方面序**）。**勘误（19:32Z）＝「探针 B 在飞」误判**：03:16
观察到的第二条链实为本流停杀链的孤儿 b46 探针（kill 只杀 bash
包装、test-guard/time/runner 子树存活 PPID=1，仍写
`t262-aborted-chain-b46.json`；与探针 A 并发＝其 RSS 读数带
污染仅作旁证）；`t262-probeB-run.sh` 从未起跑、
`T262B_PROBE_DONE` 永不出现。两点对照链
（/tmp/batch48/t262-probe-chain.sh，b46 基线 vs head 同仪器同跑）
**本流已武装串行门链 v2**（`/tmp/batch48/b49-gate-chain.sh`
nohup 在飞，终哨兵 `B49_GATE_DONE`：等探针 A 哨兵〔记录不强制〕＋
test262 runner 进程连续 10 分钟静默→两点链 b46bcfacb/head→
workers 复测 `workers-retest.json`＋SET-IDENTICAL 比对
`workers-retest-compare.txt` 全自动串行；4h 等待帽＝
WAIT_TIMEOUT 形）；**对端会话见此记入勿再起探针/两点链**
（checkout 换树＋换装二进制互相破坏对方面序），门判读与
dispatch 由本流后续轮次承接。判据细则：a) **test262-builtins
两点对照**（release 二进制＋test-guard `--per-proc-mem 6
--total-mem 8 --time-limit 9000`＋`/usr/bin/time -v`＝CI 同形；
b46bcfacb 基线 vs 现行 main head；CLEAN 判据＝完整 JSON＋head
peak RSS ≈ 基线档——**本地 RSS 低档不能单独证 CI CLEAN**〔workers
本地 4.5 GB 档 vs CI 6.3 GB 击杀档，本地/CI 差 ~1.8 GB〕，单点
被杀或 RSS 高档＝门不开）；b) **workers 复测**（s10k25返修/s10k26
再触 part03/04/06/14 面＝9f4486302/b3fab8330 同域须重测；CLEAN
判据＝175 files 696P 与 b46 SET-IDENTICAL，唯一差异＝已知
wobble 页 TO↔F）。双 CLEAN→dispatch（coverage-only 带
`-f run_wptfyi_coverage=true`，dispatch 前 pull 一次）＝**批四十九
窗口读**：k129 d06dbbca6 css-mixins +4（batch48 未表达首读）＋
s10k25返修/s10k26 TT/DOM 面＋dom-ranges 第 25 采样（第 24 采样
−70 外穿负向，s10k25返修 R522c 原序恢复回带候选）；test262 面
内存回归仍归属对端 dom 流（TT/CSP 链，9f4486302 part14 面首候
选），本流不硬修。/tmp/batch48/ 脚本族自 batch47 `cp -p`＋sed 同
换（RUN/A/B 路径三处）＋**目标目录已有内容先 ls＋diff 再覆盖＝批四
十六课⑤**＋**脚本启动前 bash -n＋test -x 双验＝exec 位丢失二演课**；
A/B 基线照旧 /tmp/batch46/run46/。**窗语义按批三十八~四十八
课执行**：dispatch 后他流在途码刀可随时落树破均匀窗（先例六演），
收账必须 shas.sh 55 行逐分片甄别归因（**shas.sh 后台 nohup 跑为常
规形**）；**rerun 形预案＝attempt-1 有失败先归档失败 log 再 rerun
（批四十六课①）；rerun --failed 后 aggregate 同 attempt 重评估自
跑落快照为预期（批四十六课③）；completed 后 aggregate 仍缺账→本
地聚合复现兜底（fetch --refresh＋coverage 同脚本，快照手动落）**；
携码谱系先 merge-base 判 docs-only 超集/码超集/窄落/分歧携带，归因
前先查各刀自身套件分片所在 SHA；aggregate 捎到的窗外后落码 SHA 先
fetch 再判 docs-only 与码刀（批三十七课）；shas 空行行数验收＋定向
补提取；RUN_DONE 后验收 dl.log 完成形（flattened=167/done）＋
autodl.out 无错误。**收账侧基线两课**＝A/B 基线 runNN 目录在上一批
/tmp/batchNN 树下（非本批树），复量前先 ls 验基线文件存在，防 glob
空集伪 0→N 大面误读；快照组行基线取上一批快照 commit（`git show
<prev>:docs/compat/wptfyi/coverage-<日>.json` 形），防两窗净差误读
（批四十五实证）。观察清单（批四十八承接口）：**trusted-types 持位
读**（1,886 新基＋HTMLScriptElement-internal-slot 9→1 regs 收缩
负面读二读）＋dom/ranges 第 25 采样（41,632 外穿负向回带候选＋
replaceChild −4＝s10k25返修 R522c 原序恢复回带候选）＋css-mixins
（k129 +4 未表达首读＋476 新基恒读）＋css-values（6,717 新基持
位＋random 154 满仓持位）＋**dom/events 十六读**（593 恒读续）＋
gethtml 十读（4,186 距 4,688 差 502）＋editing 九读（93,749 出带
新高位持位读）＋face 八读（rendering 2,781/infra 635＋svg-inline
408 回满二读＋onblur 5P 破带持位/dialog-overlay 2→1 带读）＋
color 十一读（hsl regs 3,523 镜像带回摆候选）＋corner-shape 十二
读＋getAnimations 十一读 wobble＋san 七读＋domparsing 1,313＋
CE·SD 二十四窗＋DSD 十七腿＋selection 十五读＋track-cues 交替
带二读＋合成 Fail 七读（104 基，+1 抖动）＋R2 归渲染流域照例行
＋负向两形第十九窗（test262/csp 恒值）＋dat 170 键/per-global
照旧；快照侧照旧（coverage-<UTC 日> 同日覆写；CI aggregate
aligned manifest＝分母漂移常态另报不入账，本地旧 cache 复现 face
行做双证，护栏命中 GUARD_EXIT 时以 artifact 收账）。

**门链 v2 击杀勘误＋v3 隔离门链武装（2026-11-26 20:52-20:58Z）**：
v2 于 20:52Z 被本流击杀——其两点链脚本对**共享树** `git checkout`
b46bcfacb（detached）恰逢对端流 commit/起腿窗；reflog 实证零实害
（对端 commit 全部 20:33:44Z 落库，本流 detached 窗 20:50:58-20:52:29
仅 91s、对端无树依托操作），但形态禁用：**v3 起门链对共享树零
checkout、零 target/release 换装**。跨 clone 澄清：对端 s766 流实际
执行 clone＝ZeroWeb-2（/proc/2328511/exe 直证），其腿面序与本 clone
二进制无关＝「换装破坏对端腿」危险度重评为零（同 clone 探针互斥
纪律仍对本 clone 内会话有效）；其 wait-and-run v3.4 窗判据＝按 comm
全局数 cargo/rustc/rustup/clippy/zero-wpt-runner/test-guard＋load1<3
三查 45s 浸泡、无击杀逻辑——**本流协调形＝v3 启动浸泡镜像同判据**
（绝不在对端腿飞行中起跑；本流在飞时对端守望只见重进程持闭窗＝
延迟不污染，其 9000s deadline 可能超时一次属预期、对端轮次自会
重臂）。v3 形态＝每点 `git worktree add --detach`＋CARGO_TARGET_DIR
隔离构建→二进制 cp 出→worktree remove；探针 cwd＝主树
（third_party/test262 未跟踪语料只在主树）；输出名全带 `-v3` 后缀
零覆写（`t262-head-v3.*`/`t262-b46-v3.*`/`workers-retest-v3.*`）；
终哨兵沿用 `B49_GATE_DONE`＋新增 `B49_GATE_V3_SUMMARY` 判词。
**判据修订（范围收敛非放宽）**：两点链改 HEAD 点优先、b46 平行点
仅 AMBIGUOUS（peak 1-5.5GiB 带）触发——仪器对齐已锚定（探针 A＝
p8 面 cap 击杀 6,338,488KB vs CI 6,309,664KB 差 0.46%；孤儿 b46
对照 ~554MB@77min 外杀未完整），悬而未决者唯现行 main 是否仍
balloon；判读带 CLEAN（rc=0＋peak<1GiB＋JSON 可析）/
BALLOON_RISK（rc=0＋peak≥5.5GiB＝CI 余量 ~12% 不开门）/
CAPKILL_BALLOON（rc≠0＋peak≥cap）/FAIL_RC_n（构建 11/12/13 或
他形）。workers 腿照旧 175 files 696P SET-IDENTICAL
（b3f-compare.py）。s10k27（678e17076）＋k131（9ef9851b4）已落
main＝head 点实测此新面（dispatch 即携带之，属窗语义预期）。
**重臂勘误（21:02Z）**：v3 首臂 20:58Z 实测面 a3d2f3be8；浸泡刚过
（21:00:09Z）对端即推入 s10k28 刀㉗ a09bdd83c（part12-trusted-types/
part14-trusted-sinks＝气球嫌疑域本体）＋clippy 解堵 d10adbfcd（零
语义）→21:01Z 击杀重臂至 dispatch 真面 97239d176（陈旧面探针两向
都无法裁决：CLEAN 证不了新面、BALLOON 冤枉可能已修的新面＝门判
准必须对准 dispatch 携带面）；击杀时 worktree add 未落盘＝零残留
实证（无注册项、无部分目录、无 cargo/rustc 存活）。

**三臂/四臂勘误（21:09-21:21Z）**：三臂＝build_point 首行 `local
sha=$1 tag=$2 wt=…/wt-$tag …` 同一 local 语句**先全量展开后赋值**
＝`$tag` 未绑定即崩（set -u 语义，bash -n 测不出；nohup log 双行
报错实证，首臂死于浸泡窗未触及同一雷）＝拆分 local 声明修复
（差分复现确证）。四臂首试 build rc=101＝rusty_v8 预编译静态库在
worktree 内解析树根 `.cargo/rusty_v8/archive`（gitignored 符号链
接→~/.cache blob v150.2.0＝lock 同版）不携带→build.rs:937
NotFound panic＝`RUSTY_V8_ARCHIVE` 指回主树修复。四臂 21:21Z 构建
过（1m04s 增量复用）→**探针在飞**（zw-run-head 131,142,720 B，
face a53dcd0af）＝本流首次真正到达 head 点实测。**可复用课＝
无人值守 bash 脚本 bash -n 之外须 smoke 首个函数调用**（set -u
展开序雷）＋**worktree 隔离构建须携带 gitignored 本地构建资产**
（rusty_v8 archive 形）。门判读仍由后续轮次承接：查
B49_GATE_DONE→SUMMARY 判词→双 CLEAN 即 dispatch49.sh；balloon
则记 dom 流 blocker。

**追刀一次到位＋冻结判形（21:31-21:44Z）**：四臂探针在飞 9min 时
对端再落 s10k28 刀㉘＋刀㉘-B（part14-trusted-sinks/csp.rs/
webview.rs＝气球嫌疑域再进两刀）＋k132 css（style-system 域）→
击杀重臂至 adf767f5b（s10k28 批收尾＝自然间歇点）**并冻结：此后
对端再推刀不再追，判读按漂移集域甄别**——docs-only 恒等洗清；
css/style-system 域码刀与 test262 face 无关（testharness-test262
纯 JS builtins 不触 CSS）恒等洗清；**TT/CSP/engine/script-sandbox
域码刀＝face 已变→门不开、下轮重探**（误开是危险向＝CI 重演
batch48 毒 JSON 拒快照；误关只延迟一天）。**杀链教训＝探针二进制
副本 comm=zw-run-head＋/usr/bin/time 不在既有 kill/soak 模式**→
孤儿对 2403182/84（PPID=1）实证后补杀，soak clean_check 模式扩
zw-run-head|time（自残留同堵）。五臂 21:34Z 构建过（58.78s 增量）
→**探针在飞 face adf767f5b**（zw-run-head 131,163,056 B≠四臂
131,142,720 B＝新刀已入二进制旁证）；四臂 9min 部分 json/time＝
我方中止非面行为＝无证据价值由五臂覆写。balloon 击杀点预计
~22:41Z（探针 A 67min 形）、CLEAN 完跑 ~23:30-24:05Z＋workers
~15min→终哨兵最迟 ~00:20Z。门判读由后续轮次承接。

**批四十九门判定＝CAPKILL_BALLOON 门不开（2026-11-26 22:44-22:49Z）**：
五臂探针 adf767f5b 面跑 ~70min 被 cap 击杀（`单进程内存超限
(6297236 KB > 6291456 KB)`；探针 A 6,338,488／batch48 CI
6,309,664 同带 <1% 离散＝确定性再现；击杀点 22:41:08Z 仍存活
1:07:10→22:44:40 rc=124＝~70min 形，探针 A 67min 同形）＝
**s10k25返修＋s10k27 刀㉗＋s10k28 刀㉘/-B 均未解** test262 气球＝
blocker 仍归对端 TT/CSP 链（b3fab8330 只洗 workers 面），本流
不硬修。workers 腿 **SET-IDENTICAL**（175 files 696P 零
other_diff 零 wobble＝json 与基线字节同尺寸 106,832 B；RSS
4,439,008 KB＝已知 4.5GB 档；4:20 完跑）＝workers face 与 b46
恒等。脚本两课落档：① cap 击杀时 guard 连 time 一起 SIGKILL＝
time -v 报告不落盘，SUMMARY 标签 FAIL_RC_124 实为
CAPKILL_BALLOON——peak_of 已加 guard 消息读数源（差分验证
t262=6,297,236/workers=4,439,008）；② workers rc=1＝套件含非
Pass 即 exit 1 非错误。dispatch 不发＝批四十九窗顺延，快照持
109,414（净 +38,318 余 42,682）。飞书已报卡点。**下一步＝守对端
TT/CSP 修复笔落树（重探判据＝气球击杀 ~70min 形须消失、完跑压回
9000s 帽内）→落树后重跑 /tmp/batch48/b49-gate-chain-v3.sh
（HEAD 重取＝冻结判形继续适用，drift 域甄别不变）→双 CLEAN 再
dispatch49.sh。**
门判读与 dispatch 仍由本流后续轮次承接：查 `B49_GATE_DONE`→读
SUMMARY→双 CLEAN 跑 /tmp/batch49/dispatch49.sh（pull --rebase→
dispatch→RUNID.txt）→按 RUNID 起 waiter/autodl/failarch/
shas-gated/finalarch 收账族；balloon 则记 dom 流 blocker
（test262 内存回归归属对端＝s10k25返修后首实测）并继续守其返修。
② 目标级前置账按批四十六快照重排：净 +38,318（109,414/71,096）、
余量 42,682；**拉取与接线两杠杆均耗尽**（勘误二 pin 可拉未计分 0；勘误三五簇
甄别＋窗腿姊妹键校正后真扫描域缺口 ≈487S＋窗腿真残差 ~400S 全在
导航/iframe 行为域，接线批取消），批二十唯一新增可回收杠杆（html5lib
dat +118P）批二十二兑现 +98，残差 −20 归 document.write 刀位；
余量路径收敛两杠杆：① pin bump（上游漂移
15,429S 杠杆，WPT_REV 共享面跨流决策**待拍板**）；② 行为面刀位（UA
样式面 ≈4,400、popovers 余 ~2,540（批二十二 RT 溢出已兑 +204）、
rendering ≈9,164、scripting-1 blob:
URL 模块图与 module-map 缓存族、safe-passing MessagePort instanceof
接口对象缺位 ~150–200S＋structuredclone iframe 内容门 117S、worker 面挂账
dom idlharness.any.worker 808S＋webappapis 589S）维持待碰头
（[前置账证据](evidence/2026-11-22-dc3-goal-accounting-and-corpus-runway.md)
＋[批二十七证据](evidence/2026-11-23-dc3-batch27-run-reconciliation-tt-dual-knife-verbatim-and-negative-spillover.md)）。
批十二 leg-2 备料
顺序：engine 扫描器刀待碰头排期。TO 面 83 键根因定谳已收口（批二十四
备料腿：script/stylesheet 车道探针补齐，三车道同形——非标准响应资源载入
永不完成 → load 链阻断；逐键归因 script 37/img 26/stylesheet 12/preload 8
＋四键 dw 形独有 TO 非对称观察＝engine 碰头输入）
（[根因定谳证据](evidence/2026-11-23-dc3-batch24-speculative-to-face-rootcause-probes.md)）。
批九余量挂账续推：service/sharedworker 生成形通道
（messagechannel 274 子测试最大单项＝engine worker lane 域待碰头）。
批十刀位余量：.py fixture 备料已批十三落地 + shim 编码越界刀（待碰头）。
刀位储备
首选仍为批六战役图——popovers 2,744 F+T/66 键（engine popover 语义域）、
command-and-commandfor 318/23 键、interestfor 112/28 键；批八挂起面两刀
（normalize / document.write）与批七战役图（text 65 / createImageBitmap
门 ~33）、批五（origin/api 序列化族 215、open-features 特性串族 135）、
批四（navigator.protocol 244 / event-handler 反射面 ~455 / atob 70）续挂；
批二十三新增小刀：clientInformation 缺 navigator 别名（js_dom_shim 面，
per-global 三测 assert_implements 红，~3S 上限）。
engine 刀（同任务可见性）与 UA 样式值差刀按碰头结论排期。syntax 套件周跑
墙钟 +80 min（148 页级 TO 各 ~30s）随两刀落地消解。

[执行契约](../wptfyi-html.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | 完成 | 分片缺失核对＋实体矩阵试跑 25,606/25,606 全过：[dc1 证据](evidence/2026-11-20-dc1-baseline-and-xhtml-entity-trial.md) + [逐文件 JSON](evidence/2026-11-20-dc1-xhtml-entity-trial.json) |
| DC-2 按簇收敛 | 进行中 | dom 批一 +1,888 零回归：[dc2 证据](evidence/2026-11-20-dc2-dom-batch1-variants-idlharness.md)；rendering 批二 +2,551 通道落地＋根因归因修正：[批二证据](evidence/2026-11-20-dc2-rendering-batch2-channel.md) + [逐文件 JSON](evidence/2026-11-20-dc2-rendering-batch2.json)；批三 window 变体通道 +11：[批三证据](evidence/2026-11-20-dc2-batch3-window-forms-channel.md) + [逐文件 JSON](evidence/2026-11-20-dc2-batch3-window-forms.json)；批四 webappapis +1,537：[批四证据](evidence/2026-11-20-dc2-batch4-webappapis-channel.md) + [逐文件 JSON](evidence/2026-11-20-dc2-batch4-webappapis.json)；批五 browsers window/origin +40：[批五证据](evidence/2026-11-21-dc2-batch5-browsers-window.md) + [逐文件 JSON](evidence/2026-11-21-dc2-batch5-browsers-window.json)；批六 semantics 补簇 +1,955＋variant 解析修复：[批六证据](evidence/2026-11-21-dc2-batch6-semantics-complement.md) + [逐文件 JSON](evidence/2026-11-21-dc2-batch6-semantics-complement.json)；批七 canvas offscreen 主线程面 +1,062（对账口径 +1,046，覆盖键 +1,252）：[批七证据](evidence/2026-11-21-dc2-batch7-canvas-offscreen-main-faces.md) + [逐文件 JSON](evidence/2026-11-21-dc2-batch7-canvas-offscreen-main-faces.json)；批八 html5lib 170 变体键 +118（对账 +118，覆盖键 +170，零回归）：[批八证据](evidence/2026-11-21-dc2-batch8-syntax-html5lib-dat.md) + [逐文件 JSON](evidence/2026-11-21-dc2-batch8-syntax-html5lib-dat.json)；批九 infrastructure+五小簇 132 键 +623（覆盖键 +132，193 键分母对平）：[批九证据](evidence/2026-11-21-dc2-batch9-infra-small-clusters.md) + [逐文件 JSON](evidence/2026-11-21-dc2-batch9-infra-small-clusters.json)；批十 {{GET}} 注入刀 infra 净 +42（对批九基线，含抖动带 +13；A/B changed 键 12 零回归）：[批十证据](evidence/2026-11-22-dc2-batch10-query-encoding-getsub.md) + [逐文件 JSON](evidence/2026-11-22-dc2-batch10-query-encoding-getsub.json)；批十一 template 子树 19 键 +42（计数精确对平）：[批十一证据](evidence/2026-11-22-dc2-batch11-syntax-template-leg.md) + [逐文件 JSON](evidence/2026-11-22-dc2-batch11-syntax-template-leg.json)；批十二 speculative-parsing 新通道 127 键分母新入账（chrome 118）126 执行键 +50 Pass / 76 TO（A/B 零漂移，loader 怪癖面归属见证据）：[批十二证据](evidence/2026-11-22-dc2-batch12-syntax-speculative-parsing.md) + [逐文件 JSON](evidence/2026-11-22-dc2-batch12-syntax-speculative-parsing.json)；批十三 query-encoding 四端点备料腿 infra 净 +9 P（132 键全量 A/B 665→674，18 变更键全 query-encoding 面、其余零漂移，定向三轮互证）＋批十二 speculative 勘误 50→43（take body 缺设致 7 假 Pass，重跑 43/83 零漂移）：[批十三证据](evidence/2026-11-22-dc2-batch13-query-encoding-py-fixtures.md) + [逐文件 JSON](evidence/2026-11-22-dc2-batch13-query-encoding-py-fixtures.json)；批十四 CI 缺档定谳＋62 脚本支撑文件补档（无 Rust 码笔，本地 131/132 键零漂移）：[批十四证据](evidence/2026-11-22-dc2-batch14-ci-corpus-support-files.md) + [逐文件 JSON](evidence/2026-11-22-dc2-batch14-ci-corpus-support-files.json)；批十五 CI 分片三通道缺档补档（syntax-dom 三 fetch 目标＋speculative utils.js＋57/58/59 闭包 14 文件；rendering/dom/syntax 全对齐零行动，56 误改回退）：[批十五证据](evidence/2026-11-22-dc2-batch15-ci-shard-corpus-channels.md) + [逐文件 JSON](evidence/2026-11-22-dc2-batch15-ci-shard-corpus-channels.json)；批十六 DC-3 对账收账＋CI 专有缺档两文件补档（webappapis 438/1,537 缺 1,099 定谳＝html.idl 运行期 fetch −919＋sab.js −148＋base64 fetch_json 跨 topdir 缺档 −32（勘误：双 scope 初判有误，补档 b29107e7e）；infra/semantics/browsers/syntax 四通道 face 逐键兑现）：[批十六证据](evidence/2026-11-22-dc3-batch16-reconciliation-and-closure.md) + [逐文件 JSON](evidence/2026-11-22-dc3-batch16-reconciliation-and-closure.json)；余簇缺口（11-20 快照）：rendering ≈9,164 / syntax 余量=pin 缺失 26 + 驱动变体 3 + 无扫描链 43-19(template 批十一落地)=24 键（speculative-parsing 127 键此前不在 syntax 余量账内，批十二新入账）（webappapis 3,672 已批四接入余 126 键挂账见批四证据；browsers window/origin 通道分母 234 键已批五接入余 102 键挂账见批五证据；semantics 补簇 428 键已批六接入余 197 键挂账见批六证据；canvas 4,240 已批七接入余 pin 缺失 135 键 + manual 门 469 键挂账见批七证据；syntax parsing+fragments 262 键已批八接入余 72 键挂账见批八证据；infrastructure+五小簇 193 键已批九接入余 61 键挂账=内容门 42 + lane 缺位 4 + .xhtml 1 + pin 缺失 6 + 记账 8，见批九证据） |
| DC-3 全量对账 | 净 +38,318/余 42,682；跑道两杠杆（pin bump 拍板/行为刀位）维持 | 验收线 152,096 未到（当前 109,414，净 +38,318）；45556963936（批四十六收账，快照 coverage-2026-11-25 同日第三覆写 09:34:26Z 落 78a4588cf＝rerun 自愈落账首演，manifest 恒 056c528ccb 第十八窗零漂移）：html 面 109,425→109,414＝**−11＝净 +38,318**（构成＝semantics +1 dialog 回位双键对冲 rendering −12 svg-inline 408→396 截断＝**刀面 +209 全在 html/ 域外＝face 零刀面四连**）；窗码图＝五码刀全分片首跑窗＋attempt-1 双失败 rerun 自愈首演（attempt-1＝53 shard 绿＋test262-language 败 log 被清空＋aggregate Merge digest-mismatch 瞬态；rerun --failed attempt-2＝aggregate 同 attempt 重评估自跑落快照＝run success；completed 后 rerun API 全锁＝本地聚合复现兜底变双证三证）；窗内码刀读数＝k18+k18b dom/events 66→132 满仓 EXACT（k16 +11→k17 +55→k18 +66 三刀链收官＝对端刀⑱交叉互证闭环）＋k107 合面 +68＋k108 +3 逐字＋k109 合面 +72（css-mixins 六键 +52＋random-in/sibling +20）；三口径＝A/B −184 vs zw_passes −184 vs 组行 −184＝口径差 0 三连；负面带＝editing/run 92,998 −445 出带加深二读＋dom/ranges 41,702 第 23 采样外穿二连＋css-color −185 九读镜像带＋getAnimations 17→1 wobble＋corner-shape +9 churn 十读；正面＝gethtml 3,450→3,798 八读 +348 三读连续正向；full_pass 63,690→63,700（+10）＋covered 恒＋本地三证＋护栏未触发＋合成 Fail 六读 103=103＝GONE/NEW 0 五连＋布防第十六窗六课（rerun 前归档 log/aggregate digest-mismatch 瞬态/config 回显甄别/rerun --failed 语义/shas 三段式/cp 先查目标/ab.py 去重口径）：[批四十六证据](evidence/2026-11-25-dc3-batch46-run-reconciliation-five-knife-window-k18-full-66-132-k107-plus68-k109-plus72-ab-minus184-gap-zero-face-minus11-aggregate-self-heal.md) + [逐文件 JSON](evidence/2026-11-25-dc3-batch46-run-reconciliation-five-knife-window-k18-full-66-132-k107-plus68-k109-plus72-ab-minus184-gap-zero-face-minus11-aggregate-self-heal.json)；45536569566（批四十五收账，快照 coverage-2026-11-25 同日第二覆写 05:17:22Z 落 168142d04，manifest 恒 056c528ccb 第十七窗零漂移）：html 面 109,428→109,425＝**−3＝净 +38,329**（构成＝semantics −3＝onblur 5→3 水位带四读回 3＋dialog 3→2 episode 四读＝交替带单窗回吐不定谳；rendering 2,781/infra 635 新基恒＝**刀面 +169 全在 html/ 域外＝face 零刀面三连**）；**窗码图＝十二码刀全分片首跑窗**（54× dea1943b3 dispatch HEAD 自推零易主五连＋1× 2542e606 唯 aggregate docs-ahead 第九演；谱系 dea1943b3 ⊂ 2542e606＝窗外实列 docs＋码刀五笔 k107/k108/k18/k18b/k109 零窗面全部顺延；**布防侧普查时点课**＝k106 ζ 落树在批四十四普查之后、dispatch 前 22m＝预布漏计＝收账普查须贴近 dispatch 时点重验）；**窗内码刀读数＝k106 ζ 5fb707c0e MQ/CQ 条件 CSSOM 表面化双域 +114 首读兑现**（mediaqueries +90＝113→203 16 页均匀开面 prefers-* 族＋css-conditional +24＝at-container-parsing +15＋at-container-serialization 0→9 D6 锚逐字）＋**k17 44ba2a03d 激活链全分片首跑＝dom/events +55 逐字**（Event-dispatch-single-activation 11→66＝k16 +11→k17 +55 双刀链 +66 收官形，残 66F 恰为 k18 域＝对端刀⑱交叉互证闭环）＋k103 γ2＋k105 ε＝random-computed 150→154＝满仓收官九读＋k104 δ corner −2 churn 九读＋k17b 零 suite 面；**三口径＝A/B +490 vs zw_passes +490＝口径差 0 双向零差二连**；负面带＝editing/run 93,443 四读仍出带收窄（er0 +123＋er2 +639 回＋er1 −289 截＝trio +437）＋dom/ranges 41,800 双缘外穿第 22 采样（dataChange −201＋replaceData −184＝−385 账归 wptfyi-dom）＋transitions −7＋corner-shape −2；正面＝gethtml 3,172→3,450 七读回位 +278（距 4,688 差 1,238）；full_pass 63,689→63,690（+1＝random 满仓首过）＋covered 74,189 恒＋分母恒＋本地三行逐字同双证三连＋护栏未触发＋合成 Fail 五读 94=94＝GONE/NEW 0 四连＋run 2h20m1s 慢边回带内＋布防第十五窗一小课（waiter 29 轮一次成功＋autodl ~8m 慢边二演零事故＋shas 零空行五连〔前台 5min 墙钟截断→nohup 后台重跑补齐＝后台跑为常规形〕＋在位等待零跨会话损失）：[批四十五证据](evidence/2026-11-25-dc3-batch45-run-reconciliation-twelve-knife-window-k106z-dual-domain-plus114-k17-chain-66-verbatim-ab-plus490-gap-zero-face-minus3-water-band.md) + [逐文件 JSON](evidence/2026-11-25-dc3-batch45-run-reconciliation-twelve-knife-window-k106z-dual-domain-plus114-k17-chain-66-verbatim-ab-plus490-gap-zero-face-minus3-water-band.json)；45517085385（批四十四收账，快照 coverage-2026-11-25 新日首写 02:15:38Z 落 fdb8fd978，manifest 恒 056c528ccb 第十六窗零漂移）：html 面 109,389→109,428＝**+39＝净 +38,332**（构成五页全列＝rendering +25 svg-inline 383→408 regs 648 回满＋infra +10 messagechannel.any.worker 105→115 regs 137 回满＋semantics +4＝onblur 3→5 水位带三读回上带＋dialog 2→3 episode 三读回位＋track-cues 族总 2→3＝sorted-before-dispatch 0→1 首过换员形；**零刀面**＝刀⑮ TT/k16 events 均不在 html/* 路径）；**窗码图＝双 docs 分层＋k17 码刀 2 分片窄落窗**（40× 92abb4b5a dispatch HEAD 自推零易主四连＋12× 421f9b074 s746 docs-only +2m＋2× 44ba2a03d k17 +9m editing-run-10/catch-all-2 窄落＝批三十八刀④窄域特例＝零 suite 面兑现〔目标域 wptfyi-dom 电池不在 coverage 套件集〕＋1× 6df72d978 k106 布防 docs 唯 aggregate 第八演；谱系 92ab ⊂ 421f ⊂ 44ba ⊂ 6df7；**k103/k104/k105 仅入 aggregate 零窗面顺延＋k17b 完成后 +90s 落树未跑顺延**）；**窗外码刀读数＝刀⑮ TT +180 逐字精确**（Node-multiple-arguments 0→90×2＝自报 90+90 全清；TT 1,491→1,671；TT 页全帧内零帧损；ab44.json 键级 quirk 留档＝直接工件复量权威）＋**k16 dom/events +11 逐字**（Event-dispatch-single-activation 0→11＝自报 132F→121F）＋k99/k100/k102 random 电池三刀首跑合读 +10（random-computed 140→150 五读连涨 127→132→140→150 页级不定谳）；**三口径＝A/B −1,077 vs zw_passes −1,077＝口径差 0 双向零差归零**（+140→+8→0）；负面三带＝editing/run 93,006 下缘外穿四读（er0 −345＋er2 −855 截＋er1 +118＝trio −1,082 截断搬家五演）＋gethtml −564 六读再截（3,736→3,172 高幅震荡带确立距 4,688 差 1,516）＋flexbox −110 position-absolute-013 镜像回吐归渲染流账；正面＝css-color +70 镜像带七读完美反对称封环（−136/+136/−130/+130/−70/+70）＋dom/ranges 42,185 回带内第 21 采样（dataChange +149＋replaceData +199）＋corner-shape +12 churn 八读；full_pass 63,684→63,689（+5＝TT +2＋semantics +2＋color/web-animations +1×2−editing −1）＋covered 74,189 恒＋分母恒＋本地三行逐字同双证＋护栏未触发＋合成 Fail 四读 94=94=94 同提取器三连恒（历史 103 系含续行宽度读数口径注记留档）＋run 1h57m3s 快边新纪首破＋布防第十四窗零新课（waiter 25 轮一次成功＋autodl ~6m52s 慢边一演零事故＋shas 零空行四连＋在位等待零跨会话损失＋收账侧基线路径课留档）：[批四十四证据](evidence/2026-11-25-dc3-batch44-run-reconciliation-k17-narrow-landing-tt180-verbatim-gap-zero-face-plus39-env-only.md) + [逐文件 JSON](evidence/2026-11-25-dc3-batch44-run-reconciliation-k17-narrow-landing-tt180-verbatim-gap-zero-face-plus39-env-only.json)；45494445571（批四十三收账，快照 coverage-2026-11-24 同日第五覆写 23:53:41Z 落 9a1976811，manifest 恒 056c528ccb 第十五窗零漂移）：html 面 109,401→109,389＝−12（构成＝刀面 +25＝html/syntax +24 刀⑬ html5lib math/svg/foreign-fragment 各 +8＋webappapis +1 刀⑭ parseHTML 面−环境截断 −35＝rendering −25 svg-inline regs 648→599＋infra −10 messagechannel.worker regs 137→127 截断搬家入 face 域首演−semantics −2 modal-dialog episode；rendering 零变化行不适用）；**窗码图＝均匀码窗 docs 分层携带变体首演**（41× 77fbff6a3 dispatch HEAD 自推零易主三连＋13× 5c03a1220 R6510 docs-only +4m 宽域拉到＝谱系 77fb ⊂ 5c03 ⊂ 3377 docs-only 超集码面全同归因零混淆＋1× 3377ef780 唯 aggregate 同形第七演）；**窗外七码刀读数兑现＝A/B 总 P +2,146 单窗最大面**＝san 三刀合面 +122（P 584→706＝F 172→50 深收敛；basic-filtering +64＋parseHTML +20 刀⑭逐字＋sethtml 链 +23＝刀⑫；javascript-url +10＋names +3＋html5lib +2＝刀⑬ 余量 ~+63 候选；自报可归和 59）＋TT 交叉兑现（刀⑫ 面 +3 差 −1）＋kc5r94 font-palette 五键全开 +65 首读大面＋kc5r95 animations net +14 内（getAnimations 1→17 六读回位＋event-dispatch 截断对冲）＋kc5r96 color −70 内淹没＋cf5 fmt 二读零面；**三口径＝A/B +2,146 vs zw_passes +2,138 口径差 +8 帧构成小形二读**；正面三带＝editing/run 94,088 回带（er0 multitest +453 满仓 TO 清＋er2 +543＋er1 +116 三截全回）＋dom/ranges 41,837 回带内第 20 采样（dataChange +92＋replaceData +70）＋gethtml +639 五读大回位（3,097→3,736 距 4,688 差 951＝DSD 108→109 十三腿交替带回）；负面＝css-color −70 镜像带六读（regs 3,753→3,684 截断形首见＝kc5r96 同域淹没）＋corner-shape −10 churn 六读；full_pass 63,679→63,684（+5）＋covered 74,189 恒＋分母恒＋本地三行逐字同双证＋护栏未触发＋合成 Fail 三读 103=103 恒＋flexbox position-absolute-013 0→110 regs 1→432 大面回位归渲染流账＋run 2h7m20s 回带内＋布防第十三窗零新课（waiter 27 轮一次成功＋autodl ~2m21s 快下载三连＋shas 零空行三连＋在位等待零跨会话损失）：[批四十三证据](evidence/2026-11-24-dc3-batch43-run-reconciliation-seven-knife-uniform-window-ab-plus2146-record-face-minus12-truncation-offset.md) + [逐文件 JSON](evidence/2026-11-24-dc3-batch43-run-reconciliation-seven-knife-uniform-window-ab-plus2146-record-face-minus12-truncation-offset.json)；45468215603（批四十二收账，快照 coverage-2026-11-24 同日第四覆写 21:16:03Z 落 8862fa3ae，manifest 恒 056c528ccb 第十四窗零漂移）：html 面 109,403→109,401＝−2（构成＝html/semantics −2＝checkable-active-onblur 5→3 onblur 3↔5 水位带二读＋webappapis 1,541 恒＝script_009 二读恒＋emptyish 二读恒 5＝两形关闭；rendering 零变化）；**窗码图＝半窗携带第四演＝携码超集首演**（51× 2398ca8a6 dispatch HEAD＝批四十一收账自推零易主二连＋3× cf5dc15ad cargo fmt 纯形刀 +9m 三晚起分片〔css-face-a＋test262-builtins＋catch-all-1〕＝谱系 2398 ⊂ cf5 ⊂ 7197 晚起分片亦带全五刀＝**五刀全分片窗实锤**＋1× 7197fb154 s741 docs 唯 aggregate 同形第六演）；**五码刀读数兑现＝san 双刀合面 +169 逐字精确**（刀⑩ +143＋刀⑪ +26 自报和精确＝P 415→584/F 341→172/subs 757 恒；刀⑩ 域 +140＝sethtml-tt 129＋immutable +5＋parsehtmlunsafe-tt +6；刀⑪ 域 +28＝config +18＋modifiers +10 自报差 +2）＋**口径差 +140 帧构成形首演**（刀⑩ 三 tentative 页三家分母帧外＝suite +169 vs 组行 sanitizer-api 368→397 仅 +29＝非漂移，恒偏移 ~18.4k 两端相消不再成立首窗）＋kc5r93 合面 +39（vtn +20 自报 21 差 −1＋css-values ident() 家族 +19）＋kc5r92 math-depth +4 首读兑现＋kc5r91 +5 自报 +1 差 +4＝五刀合面 suite 级 +217；A/B 总 P **−79 vs zw_passes −219 口径差 +140**（帧构成）；负面带＝editing 92,976 下缘外穿三读（er0 multitest 全回 +696＋er2 delete −611＋er1 −166＝截断 er0↔er2/er1 搬家环境形再证）＋dom/ranges 41,675 二次外穿第 19 采样（dataChange −233＋replaceData −117＝双缘带再证）＋corner-shape −4 churn 五读；正面 gethtml 四读 **+135 反转回位**（2,962→3,097 regs 2,963→3,098 截断缓和 135 条回录＝距 4,688 全量远缓和非回位；web-components 组行 +135 全由 gethtml＝DSD 恒 108）＋TT 十读破形二连 1,483→1,488（时序候选，刀⑫ TT+4 下窗入归因分离注意）＋domparsing 持位 1,312→1,313；full_pass 63,674→63,679（+5）＋covered 74,189 恒＋分母恒＋本地三行逐字同双证＋护栏未触发＋合成 Fail 新法二读 103=103 恒（ab.py 提取器定义存档结清批四十勘误候选）＋run 2h1m1s 快边二演＋布防第十二窗零新课（waiter 25 轮一次成功＋autodl ~2m24s 快下载新快边二演＋shas 零空行二连＋在位等待零跨会话损失）：[批四十二证据](evidence/2026-11-24-dc3-batch42-run-reconciliation-five-knife-full-carry-san-double-verbatim-frame-gap-140.md) + [逐文件 JSON](evidence/2026-11-24-dc3-batch42-run-reconciliation-five-knife-full-carry-san-double-verbatim-frame-gap-140.json)；45437133238（批四十一收账，快照 coverage-2026-11-24 同日第三覆写 18:40:40Z 落 60d1db094，manifest 恒 056c528ccb 第十三窗零漂移）：html 面 109,404→109,403＝−1（构成＝html/webappapis −1＝html-syntax script_009 1→0 document.write 刀位残差域抖动＋html/semantics 净 0＝onblur 3→5 水位带回 5P＋2 与 emptyish-script-elements 7→5 −2 对消单窗不定谳；rendering 零变化）；**窗码图＝半窗携带第三演**（48× 4dc925d64 dispatch HEAD＝批四十收账自推零易主＋6× c5818cb57 刀⑩ sanitizer dispatch 后 +5m28s 落树被 6 晚起分片拉到〔editing-run ×3＋forms-clip＋test262-misc＋syntax-dom〕＋1× 88fea2a93 R6430 docs 唯 aggregate docs-ahead 树含窗外五码刀不跑套件；**刀⑩ 自身套件分片 catch-all-8 baseline 码面＝刀⑩ face 本窗零现顺延批四十二**＝携码刀套件分片归因新课）；**窗外四码刀全分片携带读数兑现＝刀⑨ domparsing +48P/−48F 逐字精确**（1,264→1,312 subs 恒，四键 +11/+2/+17/+18，分片 baseline 归因纯）＋css-values 四刀合面 +37 vs 自报和 +38 差 −1（kc5r88 +29＋gCS +1＋kc5r90 +8＝random 族六键全正向 7/21/2/1/3/3，单键合翻转候选，full_pass_files +4）；A/B 总 P **−1,984＝zw_passes 口径差 0 精确闭合**（批四十 −3 差消解）；负面三带＝gethtml −1,726 截断三读加深（−913→−1,307→−1,726 单调加深候选，分片 baseline＝环境形实锤）＋editing −705（93,057）下缘外穿二连（er0 −1,071 深截＋er1 −45＋er2 +411 回位）＋css-animations −15 getAnimations 17→1 四读回位；正面 dom/ranges +247（42,025）停摆带第 18 采样回带内＝双缘带定谳强化＋color +130 镜像带四读对称＋position-absolute-013 二读回位关闭＋interpolation 族宽带零位移持位；full_pass 63,670→63,674（+4）＋covered/分母恒＋本地三行逐字同双证＋护栏未触发；观察清单（TT 九读破形 1,483＝script-enforcement-003 +1/san 415 持位刀⑩⑪未携正确零现/CE·SD 十七窗持位/DSD 第十一腿 108/selection 八读 48,902/track-cues 十九演 3P/负向两形十五窗/合成 Fail 报告扫描 100=100＋批四十 103 口径勘误候选）＋run ~2h14m24s 回带内＋布防第十一窗零新课（快下载 ~4.5m 新快边候选＋shas 零空行零复发＋在位等待零跨会话损失）：[批四十一证据](evidence/2026-11-24-dc3-batch41-run-reconciliation-half-window-knife10-carry-domknife9-verbatim-face-minus1.md) + [逐文件 JSON](evidence/2026-11-24-dc3-batch41-run-reconciliation-half-window-knife10-carry-domknife9-verbatim-face-minus1.json)；45405289405（批四十收账，快照 coverage-2026-11-24 同日覆写 15:37:38Z 落 c1e98b9c7c，manifest 恒 056c528ccb 第十二窗零漂移）：html 面 109,404→109,404＝**持位零变化**（20 组行全零动＝六刀全在 html/ 外；rendering 零变化）；**窗码图＝均匀码窗**（43× 9dc352bfd dispatch HEAD＋11× a538eb037 本流布防 docs 晚起分片拉到＋1× 371be6508c aggregate docs-ahead 树含窗外三码刀不跑套件＝54 套件分片码面逐字同＝归因窗纯净）；**窗外六刀读数全兑现**＝刀⑦ +18 逐字精确（CustomElementRegistry 27→45）＋刀⑧ +15 逐字精确（element-mutation 0→15 页全绿）＋kc5r87 +19 逐字精确（shorthand-001 0→16＋overflow-valid 15→18＋清单外溢出 cssom/overflow-serialization +3）＋kc5r86 净 +3 候选读（properties-value-003 +4/inherit-003 −1 单窗不定谳）＋cssom 刀 +2 F2P 精确（columns-shorthand-reset-wrap 1→3＝**批三十九吸收形预布修正**＝键名甄别勘误）＋PR #323 零行为面兑现；A/B 总 P **−3,712**（zw_passes −3,715＝口径差 −3 照记）；负面三带＝dom/ranges −931 **停摆带第 17 采样下缘外穿**（dataChange 1211→716＋replaceData 1146→710＝带暂记 41,778~42,709）＋editing −1,349 下缘外穿深落（er0 −202/er1 −408/er2 −739＝带暂记 93,762~95,442）＋gethtml −1,307 截断二读加深非回归＋color −130 镜像带三读＋flexbox −110 position-absolute-013 regs 432→1 新截断形候选单窗不定谳；正面 ~+50 interpolation 族宽带＋getAnimations 1→17 回位＋corner-shape 135→141 震荡带；full_pass 63,662→63,670（+8）＋covered 74,189 恒＋分母恒＋本地三行逐字同双证＋护栏未触发；观察清单全持位（TT 八读 1,482/domparsing 六读 1,264 刀⑨未携正确持位/san 415/CE·SD 十六窗回位 2,910+2,910/DSD 第十腿 109/selection 七读 48,902/track-cues 十八演 3P/closedby-complex −12 二读未复现关闭/负向两形十四窗/合成 Fail 103=103 页级 GONE 0/NEW 0）＋run ~2h01m04s 批三十七快边带复现＋布防第十窗（blob 瞬断退避零损失＋shas 空行定向补提取＋在位等待零跨会话损失＋nohup cd 坑记档）；窗外三码刀 kc5r88/刀⑨/gCS px 锚全数顺延批四十一：[批四十证据](evidence/2026-11-24-dc3-batch40-run-reconciliation-uniform-code-window-six-knives-face-hold.md) + [逐文件 JSON](evidence/2026-11-24-dc3-batch40-run-reconciliation-uniform-code-window-six-knives-face-hold.json)；45369836111（批三十九收账，快照 coverage-2026-11-24 同日覆写 12:12:30Z 落 e98b3a243，manifest 换代 e00a7b14e8→056c528ccb 第十一窗＝分母漂移 +61 文件/+282 子测试另报不入账）：html 面 109,412→109,404＝−8（构成＝html/infrastructure messagechannel +4 键级抖动带恢复＋html/semantics −14＝onblur 5→3 水位震荡＋dialog-popover-closedby-complex 24→12 新面 −12 单窗不定谳＋dialog-focus −1＋track-cues 十七演回 P＋webappapis +2；rendering 零变化）；**窗码图＝半窗携带第二演**（50× a6581f10a dispatch HEAD＋4× 61a7edb2b cssom 刀 dispatch 后 +6m44s 落树被晚起 4 分片拉到＋1× 1e81edff5 唯 aggregate docs-ahead 不跑套件；cssom 刀 face 吸收形＝其 multicol F2P 2 两键已被 multicol 三刀先行翻转，批四十携带窗预期读 ~0）；**窗外六刀全分片携带窗读数＝刀⑤ 大刀 DSR 域自报逐字兑现**（property-reflection-idl-setters 225→2,910＝+2,685 恒等纯翻面＝批二十五 CE·SD −2,772 溢出面回摆定谳闭合＋attachment +68＋fixture 族五页＋sanitizer +1＋setHTMLUnsafe.html +2 直接页 face）＋刀⑥ +31 逐字精确（define 48→70＝22F→0F＋define-customized-builtins +1＋CustomElementRegistry +8 接口页外溢）＋刀④ 面 Element-customCR +2/ShadowRoot-init-customCR +1＋交互窗＋multicol 三刀 +118＝自报 98+14+6 逐字精确（column-count +37/column-width +37/discrete +42/wrap 键 ×2）；gethtml −913（regs −912）＝刀⑤⑥ 页预算截断 CI 环境差如实记（自报 −371；非回归）；A/B 总 P **+2,516＝零 zw_only 差三口径精确闭合**；负面 wobble 带（css-animations −18＝getAnimations 17→1 批三十三形复现＋corner-shape −22＝批三十八 +24 二读回位定谳强化＋css-transitions −2 镜像回吐＋backgrounds −1 四连）＋dom/ranges **+540＝停摆带第 16 采样上缘外穿新高**（regs+P 双升 +539＝带暂记 42,022~42,709）＋editing/run +8 带内回位；full_pass 63,652→63,662＋covered 恒＋zw_passes +2,516＋本地 face 109,404 双证＋护栏未触发；观察清单全持位（TT 七读/domparsing 五读/selection 六读/DSD 第九腿 108/负向两形十三窗/合成 Fail 103=103 GONE 0/NEW 0）；PR #322 perf hit_test 首入窗＝timing 干扰变量照记（run ~2h16m48s 回带内）＋布防第九窗零新课（poll26 网络抖动手动补 RUN_DONE 批三十课幂等承接＋韧性下载第四窗零事故＋aggregate 先 fetch 再判 docs-only 首实战确认）：[批三十九证据](evidence/2026-11-24-dc3-batch39-run-reconciliation-wc-dsr-mass-recovery-cesd-plus2685-multicol-plus118-face-minus8.md) + [逐文件 JSON](evidence/2026-11-24-dc3-batch39-run-reconciliation-wc-dsr-mass-recovery-cesd-plus2685-multicol-plus118-face-minus8.json)；45345696071（批三十八收账，快照 coverage-2026-11-24 同日覆写 09:23:15Z 落 c9c584ed0，manifest 恒 e00a7b14e8 零漂移第十窗）：html 面 109,417→109,412＝−5（构成＝html/infrastructure messagechannel −4 键级抖动带截断＋html/semantics track-cues −1 十六演交替带 F；rendering 零变化）；**窗码图破形＝刀④ 半窗携带新形首例**（40× 6ea86a6ea＋14× 85c872f28＝dom 流刀④ dispatch 后 +2m36s 落树被晚起 14 分片拉到＋1× ee5c29562 唯 aggregate；携刀④分片清单全留档＝均匀窗前提不再默认成立，按分片 SHA 逐分片甄别归因）；**窗内三码刀＝刀③ WC 六页 +114P 自报逐字精确（effective-global-registry 18→66＝48F→0F 页全绿逐字）＋shim③ multicol columns-interpolation +21 自报逐字（regs 恒 F2P 21/P2F 0、页全绿）＋73f4d5dcf 恢复恒等性照读兑现**；A/B 总 P −225＝零 zw_only 差三口径精确闭合（负面全落停摆带截断形 dom/ranges −123＋editing/run −255＝regs −307 加深；清单外 wobble corner-shape +24＝F2P 76/P2F 52 双向点位值级形单窗不定谳＋transitions +2/backgrounds +1 镜像带）；定谳两项＝idbcursor +1 回吐 timing wobble＋xhr send-timeout 镜像闭合；R2 二读兑现＝css +829 域持位＋contain-intrinsic-size-interpolation −49 与 img-sizes-auto 真形定谳（归渲染流域照例行）；full_pass 63,652 恒＋covered/分母恒＋zw_passes −225＋本地 face 109,412 双证＋护栏未触发；停摆带第 15 采样 dom/ranges 带内（双升新变体连续两窗未复现）＋editing/run 下缘外穿 −27＝带暂记 95,103~95,442；观察清单全持位（TT 六读/domparsing·san 四读/selection 五读/CE·SD 十四窗逐字/负向两形十二窗/合成 Fail 103=103/text-input-vertical-overflow −1 二连慢截断 watch）；run ~2h14m52s 回带内（批三十七快边未复现）＋布防第八窗一课＝刀④ 半窗携带＋韧性下载第三窗零事故＋脚本三课三窗零复发＋跨流第四起未发生：[批三十八证据](evidence/2026-11-24-dc3-batch38-run-reconciliation-half-window-knife4-carry-wc-plus114-multicol-plus21-face-minus5.md) + [逐文件 JSON](evidence/2026-11-24-dc3-batch38-run-reconciliation-half-window-knife4-carry-wc-plus114-multicol-plus21-face-minus5.json)；45326847882（批三十七收账，快照 coverage-2026-11-24 同日覆写 06:29:51Z 落 92d5fce57，manifest 恒 e00a7b14e8 零漂移第九窗）：html 面 109,426→109,417＝−9（构成＝img-sizes-auto 12→0 新负形首现〔12 全 contain-intrinsic-size 子测试 expected '' got 'none'〕＋svg-inline +3＝批三十六收缩形回吐 wobble 定谳）；**五码刀均匀窗＝R2 css 动画面 +829 大解锁＋刀②续 WC +20 自报逐字＋B-cut multicol +16 逐字**（3e8cdfc12 自报 transitions F2P=93 vs CI +89 环境差如实记；6a8df8cee 六页全限定 effective-global-registry 10→18＝56F→48F 逐字；45efa2584+cd65c9f47 columns-interpolation +13 超门收账形兑现）；A/B 总 P +1,146＝resolve 闭合（zw_only border-radius-interpolation +98＋idbcursor +1 不入 aligned 分子＝分子账 +1,047≈CI +1,048 两口径各自闭合）＋停摆带第 14 采样 dom/ranges −316 带内（批三十六 regs+P 双升新变体未复现＝回吐截断形）＋er2 delete +107 截断回吐/er1 forwarddelete −18 截断首读＋R2 域负溢出候选 contain-intrinsic-size-interpolation −49 全 P→F 单窗不定谳＋小项 ±2（idbcursor_iterating 长持位后首翻）；full_pass 63,643→63,652＋covered 恒＋zw_passes +1,048＋本地 face 109,417 双证＋护栏未触发；观察清单全持位（TT 五读/domparsing·san 三读/selection 四读定谳新平台/CE·SD 十三窗逐字/负向两形十一窗/合成 Fail 103=103 同法互证）；布防第七窗全绿＋shas 新形（aggregate SHA 不在本地对象库＝先 fetch 再判与 cache-key 误抓区分）＋跨流碰撞第三起（96ee337b4 wholesale 误删批三十六 evidence 两文件等，对端 73f4d5dcf/6bccb8472 还原收尾零损失）＋run ~2h01m 新快边（font-mmap 候选观察）；窗外三码刀 a3e299907/96ee337b4/73f4d5dcf 唯 aggregate 捡到＝批三十八全分片预布：[批三十七证据](evidence/2026-11-24-dc3-batch37-run-reconciliation-r2-mass-unlock-wc-verbatim-face-minus9-img-sizes.md) + [逐文件 JSON](evidence/2026-11-24-dc3-batch37-run-reconciliation-r2-mass-unlock-wc-verbatim-face-minus9-img-sizes.json)；45304091946（批三十六收账，快照 coverage-2026-11-24 同日覆写 03:35:19Z 落 9ed72ec31，manifest 恒 e00a7b14e8 零漂移）：html 面 109,426→109,426＝**持位零变化**（构成＝semantics +3＝onblur 5P 回位＋track-cues 十四演 F→P；rendering −3＝svg-inline regs 收缩新形首现＝净 0）；**三码刀窗＝columns 刀 +230 自报逐字精确兑现**（43ca50c9d+3fc122daa 全分片首携：multicol 三键 +198/+18/+14＝F 镜像 P2F=0 subs 恒）＋**d4bd1cec1 刀②归因窗第一读零 face**（registries 域零键变化＋双向扫溢出无新形＝单窗不定谳）；A/B 总 P +611 精确闭合＝停摆形第 13 采样 dom/ranges +417（**regs+P 双升注册完成度新变体**带上缘内 42,608/42,677）＋editing er0 +49/er2 −75（delete 626→551 截断第四读加深）＋css wobble 尾巴（corner-shape −12/event-dispatch regs 19→32 P 恒）＋columns 清单外 +1（css-backgrounds 不定谳）；full_pass 63,640→63,643＋covered 恒＋zw_passes 1,928,825→1,929,436＝A/B 精确闭合＋本地旧 cache face 109,426 双证＋护栏未触发；观察清单全持位（TT 1,482 四读/domparsing·san 二读/selection +44 三读定谳强化/CE·SD 十二窗逐字/负向两形十窗零回位/合成 Fail 61=61）；**布防第六窗＋新布防课＝批间脚本复制 exec 位丢失**（retrydl.sh 失 x 位＝autodl「权限不够」断，上一会话 429 前手动 bash 补跑 167/167 无损＝修复 cp -p/chmod +x＋dl.log 完成形验收）；6a8df8cee 刀②续＋columns B-cut 对＋05d32ab5c font-mmap 批三十七预布：[批三十六证据](evidence/2026-11-24-dc3-batch36-run-reconciliation-columns-knife-230-verbatim-face-hold.md) + [逐文件 JSON](evidence/2026-11-24-dc3-batch36-run-reconciliation-columns-knife-230-verbatim-face-hold.json)；45280793303（批三十五收账，快照 coverage-2026-11-24 新日首写 00:31:38Z 落 3842330c7，manifest 063eb7cae2→e00a7b14e8 换代＝第八窗漂移另报）：html 面 109,427→109,426＝**−1 全落 flake 带**（track-cues Pass→Fail 交替带第十三演＝face 唯一构成；dialog 1/1＋onblur 3P 持位零变化键）；**三码刀均匀窗＝批三十四预布归因读兑现＋新识别刀① 7f8df21ce**（72957ce1e→domparsing +170 subs 恒 F→P／587250e6e→sanitizer +80 regs 扩张 +98／7f8df21ce→web-components +90 自报逐字精确 15 键全 registries/*＝刀域全在 html/ 外 face 零扰自洽；43ca50c9d columns 刀 dispatch 后 27m40s 落唯 aggregate 零套件面影响）；A/B 总 P **+425**＝清单内 +317＋清单外 +108＝分子 +253 差 −64 子测试名级交集差两口径各自闭合；停摆形第 12 采样交替互补（dom +87/editing −63 双带内刀域无关）＝delete 第三读注册截断伴生非码回吐；css 负腿三读回位兑现定谳（getAnimations 17＋event-order regs 4 精确回位＝be78d24cf 嫌疑卸除）；selection +44 单窗不定谳；full_pass 63,642→63,640＋covered 恒＋本地 face 109,426 双证＋护栏未触发＋布防全绿第五窗（waiter cd 课落地＋retrydl 第二轮补齐）＋d4bd1cec1 刀② 00:34Z 落＝批三十六全分片携带预布：[批三十五证据](evidence/2026-11-24-dc3-batch35-run-reconciliation-triple-knife-uniform-window-face-minus1-track-cues.md) + [逐文件 JSON](evidence/2026-11-24-dc3-batch35-run-reconciliation-triple-knife-uniform-window-face-minus1-track-cues.json)；45253935330（批三十四收账，快照 coverage-2026-11-23 同日覆写 22:08:26Z 落 5558ca516，manifest 恒 063eb7cae2 零漂移第七窗分母恒）：html 面 109,430→109,427＝**−3 全落 flake 带**（dialog-overlay 2/2→1/1＝dialog 回稳候选第六演破形转震荡带例行＋onblur 5P→3P 水位震荡例行）；**零码窗**＝54 分片全 60b3dc45d 对批三十三 docs-only＋双引擎刀 72957ce1e（domparsing +118P 自报）＋587250e6e（sanitizer +52P 自报）dispatch 后落唯 aggregate 捡到（批三十一同形）＝归因读顺延批三十五；停摆形第 11 采样交替互补（dom/ranges −424＝42,104 带内＋editing +36＝95,350 带内＝timing 形续证，dom/ 归 wptfyi-dom 账本）；editing +108 二读＝delete 632 持位零回吐＝a2295242d caret 域正向定谳强化；css 负腿二读混合形不定谳维持（corner-shape +13 回位 timing 向＋getAnimations 持 1 不回位＋注册点 wobble 双向）；selection regs 12→8→12 镜像对收口 timing 定谳；负向两形第八窗零回位＋TT 三读 1,482 持位＋CE·SD 第十窗持位＋DSD-opt-in 第五腿 109→108＋track-cues 第十二演三连 P＋color 第六窗零键变化＋domparsing/san 1,094/334 持位＋合成 103=103；A/B 总 P −376＝coverage 分子 −376 精确闭合（零 tentative 差）＝full_pass 63,641→63,642＋covered/分母恒＋本地 face 行 109,427 双证＋护栏未触发＋布防全绿第四窗（run 时长 ~2h12-16m 记档）：[批三十四证据](evidence/2026-11-24-dc3-batch34-run-reconciliation-zero-code-window-face-minus3-dialog-break-shape.md) + [逐文件 JSON](evidence/2026-11-24-dc3-batch34-run-reconciliation-zero-code-window-face-minus3-dialog-break-shape.json)；45229462020（批三十三收账，快照 coverage-2026-11-23 同日覆写 19:28:07Z 落 9d41d5fb9，manifest 恒 063eb7cae2 零漂移第六窗）：html 面 109,428→109,430＝**+2＝净 +38,334**（构成＝dialog-overlay 1/2→2/2＝**dialog 回稳候选第五演兑现为 face** 与本地三连 2P 首次同值＋mapped-attribute-adopt-001 0→1＝写入体域）；**双码刀均匀窗**（be78d24cf renderer 图片取回出临界路径＋a2295242d engine 位置方法族＋stream*HTML 写入体，均 dispatch 前落树全分片携带，55 提取 40×c251c1212＋14×bda7fa14b＋1×1f8635665 祖先链互验＋docs-only 互 diff）；**a2295242d 双正向兑现**＝domparsing 171→1,094（+923，23 键 tentative stream-*/positional-* F→P，对端自报 +297 vs CI +923 环境差如实记）＋dom/ranges +403（Range-mutations 两键 **registration 首次满额**＝批二十四移交对端 dom/ −350 直写形**修复兑现**）＋san +3＋syntax +4＋editing +108（caret 刀候选 vs timing 单窗不定谳）；**be78d24cf 嫌疑面＝css 新负腿两形首现**（getAnimations 17→1＋corner-shape −17＋event-dispatch 注册收缩×2 单窗不定谳；color +49＝−49 逐键镜像对）；A/B 套件面总 P **+1,457** 组行闭合（domparsing +923/dom +403/editing +108/html +5/css +15/san +3/selection·web-animations·shadow-dom 零）＝coverage 分子 +528 两口径各自闭合（tentative 不入 aligned 分子，domparsing 组 138 恒）；full_pass 63,638→63,641；分母漂移 +588 另报不入账＋本地 face 行 109,430 双证＋护栏未触发；负向两形第七窗零回位；TT 二读 1,482 持位；CE·SD 第九窗持位＋DSD-opt-in 第四腿 109 持位；**DP①② 五窗零变化 watch 以码因关闭**；onblur 5P 带读持位；track-cues 第十一演 Pass 持位（键级验证）；dat 170 键字节同＝**write-driver 110 键仍 TO（stream*HTML 刀不解锁 write-driver 形，批十二 loader 怪癖域预期内）**；合成 103=103；布防全绿第三窗＋**布防课两条：A/B 报告 SIGPIPE 丢失（必须 `> file` 重定向）＋shas.sh 缺 cd 假完成（DONE_ALL 非成功证明，脚本自带 cd＋行数验收）**：[批三十三证据](evidence/2026-11-24-dc3-batch33-run-reconciliation-dual-knife-window-domparsing-plus923-face-plus2.md) + [逐文件 JSON](evidence/2026-11-24-dc3-batch33-run-reconciliation-dual-knife-window-domparsing-plus923-face-plus2.json)；45198754756（批三十二收账，快照 coverage-2026-11-23 同日覆写，manifest 恒 063eb7cae2 零漂移第五窗）：html 面 109,425→109,428＝**+3 全落 flake 带**（track-cues Fail→Pass＝交替带第十演两连 F 后回 P 仅记不追＋onblur 3P→5P＝**五连 3P 后六演破形＝回稳判定破形＝5P↔3P 水位震荡形降级例行读数**）；**零新码窗＋全分片基线携带 a3932b528＝TT 面首码归因读四面全零动＝编译适配零行为面定谳**（tt 1,482＋test262 51,907＋csp 199＋CE·SD 第八窗）；55 提取全 CONTAINED 零误抓（40×bf7e188ab＋14×f4908b6f5＋1×5c1890fb14 docs-ahead）＝**shas.sh 收紧 checkout 行锚 fallback 首演有效**；A/B 套件面总 P **−180**＝组行六项闭合（editing −236/css −50/web-animations −1/shadow-dom +1/html +3/dom +103）＝zw_passes 1,928,600→1,928,420＋分母恒 2,279,168＋full_pass 63,638＋本地 face 行双证＋护栏未触发；**停摆形第九演＝editing +258 尖峰证伪再演**（95,442→95,206 带内）＋dom +103 带内＝timing 形续证；css −50 新震荡腿（color −49 非镜像＝四窗 ±34/−136/+136/−49 扩幅震荡 watch）；DSD-opt-in 108↔109 震荡对第三腿；负向两形第六窗零回位（对端监测第三读顺延批三十三）；DP①② 第五演持位；观察清单全持位（合成 103=103 逐套件同）；布防全绿第二窗（autodl＋retrydl 主路径零事故＋瞬时失败退避补齐）：[批三十二证据](evidence/2026-11-24-dc3-batch32-run-reconciliation-face-plus3-flake-keys-and-tt-compile-fix-zero-face-read.md) + [逐文件 JSON](evidence/2026-11-24-dc3-batch32-run-reconciliation-face-plus3-flake-keys-and-tt-compile-fix-zero-face-read.json)；45169842737（批三十一收账，快照 coverage-2026-11-23 同日覆写，manifest 恒 063eb7cae2 零漂移第四窗）：html 面 109,425→109,425＝**face 持位零变化**（track-cues 第九演 media 套件零变化＝b30 Fail 持位，两连 F 仅记不追）；**双 SHA 窗＝窗内真码刀 a3932b528（script-sandbox TT 跳板 rquickjs 0.7 编译适配）落在全部 54 套件分片起跑之后，唯 aggregate 捡到（只聚合不跑套件）＝套件面零码归因维持**；批三十二预布＝TT 面首个码归因读数到期；A/B 总 P +298＝组行五项闭合（dom −118/shadow-dom −1/web-animations +2/css +157/editing +258）＝zw_passes 1,928,302→1,928,600＋分母恒＋full_pass 63,636→63,639＋本地 face 行双证＋护栏未触发；**css 带精确镜像回位**（color-computed-hsl +136 与批三十 −136 同键逐字节镜像对＋borders +18＝对称震荡定谳强化）；**停摆形第八演双边外穿**（editing +258 上缘外穿 95,442＋dom −118 下缘再穿 42,022＝带暂记 42,022~42,677/95,130~95,442）；负向两形第五窗零回位（对端监测第二读未落行为修复）；CE·SD 第七窗持位；DP①② 第四演持位；onblur 3P/4F 五连（回稳判定候选升级）；观察清单全持位（合成 103=103 逐套件同）；下载窗新形＝**egress 503 账号带宽滚动窗**（整包式无续传全弃→逐 artifact 韧形下载器 167/167）＋**cache-key 误抓第三演**（对象存在性捕获＋定向补提取）；布防全绿一窗（批二十九/三十两课落地首演）：[批三十一证据](evidence/2026-11-23-dc3-batch31-run-reconciliation-face-hold-and-code-knife-outside-suite-shards.md) + [逐文件 JSON](evidence/2026-11-23-dc3-batch31-run-reconciliation-face-hold-and-code-knife-outside-suite-shards.json)；45145268804（批三十收账，快照 coverage-2026-11-23 同日覆写，manifest 恒 063eb7cae2 零漂移第三窗）：html 面 109,426→109,425＝−1 唯一构成 track-cues Pass→Fail＝交替带第八演（F P F P F F P F 带内续不追）；**零码刀窗全变化零码归因**（码图同 da82f2226，55 分片三 SHA 全对象存在性验证）；**负向两形第四窗零回位＝对端修复兑现监测通道首读持位**（对端未修）；CE·SD 第六窗持位；DP①② 第三演持位；停摆形第七演 dom/ranges −96 下缘轻微外穿 −33（实测带暂记 42,140~42,677 双侧扩缘）＋editing −29 带内；onblur 3P/4F 四连持位；shadow-dom +1＝DSD-opt-in 回批二十七水位；css −140 震荡带（color −136 regs 收缩扩幅 watch）；观察清单全持位；快照 −268 组行闭合＋本地 face 行双证＋护栏未触发；**布防课二**＝批间 sed 复制残留字面量子目录名 run29（产物落错目录＋A/B 全零伪回归 −1,945,824 识别为缺档形，改名重跑恢复）＋waiter API 抖动 RUN_DONE 手动补写：[批三十证据](evidence/2026-11-23-dc3-batch30-run-reconciliation-zero-code-window-and-autodl-dirname-lesson.md) + [逐文件 JSON](evidence/2026-11-23-dc3-batch30-run-reconciliation-zero-code-window-and-autodl-dirname-lesson.json)；45124287002（批二十九收账，快照 coverage-2026-11-23 同日覆写，manifest 恒 063eb7cae2）：html 面 109,425→109,426＝+1 唯一构成 track-cues Fail→Pass；**零码刀窗全变化零码归因**；**批二十八勘误**（track-cues「Pass 持位」误＝artifact 实 Fail，未下键验键＝批二十六课复发，七演交替带 F P F P F F P）；**负向两形第三窗零回位（零码窗下 dom −441 对照）＝timing 假说弱化、码形主导证据加强**（③③b／③③a+③⑤ 主嫌证据链已移交对端）；CE·SD 第五窗持位；DP①② 第二演持位（+49 组行读数稳定）；停摆形第六演 dom/ranges −441 回撤带内（批二十八带外上穿单窗尖峰证伪）＋editing −20 带内；css +16 镜像震荡带（color +34 精确回位）；观察清单全持位；快照 −444 组行闭合＋本地 face 行双证＋护栏未触发；布防课 autodl 等待窗击穿＝RUN_DONE 后幂等重跑成功：[批二十九证据](evidence/2026-11-23-dc3-batch29-run-reconciliation-zero-code-window-and-negative-third-window.md) + [逐文件 JSON](evidence/2026-11-23-dc3-batch29-run-reconciliation-zero-code-window-and-negative-third-window.json)；45109059524（批二十八收账，快照 coverage-2026-11-23 同日覆写，manifest 恒 063eb7cae2 零漂移）：html 面 109,424→109,425＝+1 唯一构成 dialog 回位（flake 交替带第二演）；**DP①② 双向账**＝唯一窗码刀 4c1816d14 兑现组行 +49 vs 自报 +41（domparsing +17 逐字精确＋CE +24 组行级精确/套件 +23 regs 口径差＋sanitizer +8 清单外正溢出＝对端净零自报冲突如实记）；tt 1,482 持位；**负向两形第二窗零回位零加深＝判读窗顺延**；CE·SD 第四窗持位；停摆形第五演 dom/ranges +306 **带外上穿**（dataChange 新高 1,179）＋editing −125 带内；观察清单全持位（合成 Fail 103=103 两侧）；快照 +201 组行闭合＋本地 face 行双证＋护栏未触发：[批二十八证据](evidence/2026-11-23-dc3-batch28-run-reconciliation-dp12-verbatim-and-stall-band-breakout.md) + [逐文件 JSON](evidence/2026-11-23-dc3-batch28-run-reconciliation-dp12-verbatim-and-stall-band-breakout.json)；45091003913（批二十七收账，快照 coverage-2026-11-23 同日覆写 7542fd567）：html 面 109,428→109,424＝−4 全落 html/semantics flake 带（onblur 5P→3P＋track-cues 交替带第五演＋dialog 3→2 新形）＝零窗口码归因；**TT 两刀双向账**＝tt 套件 1,368→1,482（+114：③③b +88 与自报逐键精确同值＋③⑤ 名单域 +11＋交互窗 +15）组行 1,343→1,447 两口径闭合＋san 323 零键变化；**负向扫溢出两形移交对端**（test262 −13 Function 构造器域＝③③b collateral 嫌疑／csp −4 timer-eval 签名＋页级 Timeout＝码/timing 并存单窗不定谳）；CE·SD 第三窗持位；停摆形第四演带内（dom/ranges 42,371/editing 95,358）；分母漂移 a3830cd8f7→063eb7cae2 另报不入账＋本地复现 face 行双证＋快照 +301 闭合＋护栏未触发：[批二十七证据](evidence/2026-11-23-dc3-batch27-run-reconciliation-tt-dual-knife-verbatim-and-negative-spillover.md) + [逐文件 JSON](evidence/2026-11-23-dc3-batch27-run-reconciliation-tt-dual-knife-verbatim-and-negative-spillover.json)；45077609991（批二十六收账，快照 coverage-2026-11-23 新日首写）：html 面 109,427→109,428＝③b②＋③③a 兑现 tt +87（组行同值清单内全落第二演：block-string 族＋setTimeout/setInterval +4）＋san 323 持位＋**CE·SD −2,772 持位第二窗**（idl-setters 225P/2,685F 注册不变全同值＝不修不加深；③③b 081d40b4a 落窗外归批二十七）＋**停摆形第三采样交替互补**（dom/ranges −247 vs editing/run +223 完全回位≈批二十四水位＝三窗零码关联 timing 形定谳强化）＋**track-cues 勘误**（批二十五「持 Pass 第二连演」误＝artifact 实 Fail，组行 +1＝onblur +2 与 track-cues −1 净和；四演交替带非收敛不再追）＝face +1 唯一构成；观察清单全持位＋css 带镜像震荡（−14→+14/+7→−9）；快照 +73 组行闭合＋本地复现逐字同＋护栏未触发：[批二十六证据](evidence/2026-11-23-dc3-batch26-run-reconciliation-tt-knife-second-window-and-stall-oscillation.md) + [逐文件 JSON](evidence/2026-11-23-dc3-batch26-run-reconciliation-tt-knife-second-window-and-stall-oscillation.json)；45063183178（批二十五收账，快照 442d0e6a1 同日覆写）：html 面 109,408→109,427＝TT/Sanitizer 刀簇正向面兑现（Sanitizer interface object 曝光 16 F→P＋setHTML 家族 +4＝html/dom +18；tt 套件 +338 组行同值＝③b① 全落清单内、san +263 组行 +242 口径各自闭合）＋**CE·SD −2,772 溢出**（property-reflection-idl-setters 2,910→225 P→F 整面翻红＝对端看门狗盲区，coverage 通道唯一探测器，移交对端定谳修复）＋dom/ranges +120 无修复回位（批二十四 4fb594667 归因降级＝停摆形 timing 敏感）＋editing/run −220 同款停摆形（timing vs 码两假说并存）；观察清单全持位；快照总差 −2,286 组行闭合＋护栏未拦截总回落（wptfyi-data 域观察）：[批二十五证据](evidence/2026-11-23-dc3-batch25-run-reconciliation-tt-cluster-both-faces-and-ce-sd-spillover.md) + [逐文件 JSON](evidence/2026-11-23-dc3-batch25-run-reconciliation-tt-cluster-both-faces-and-ce-sd-spillover.json)；45049069340（批二十三码）收账（快照 coverage-2026-11-22）：html 面 109,404→109,408＝per-global +3 精确兑现（第 8 例 CI 闭环，webappapis 1,537→1,540，残 3F navigator 别名刀位挂账）＋track-cues +1 带外回摆（flaky 候选）；观察清单四项持位判定（write TO 恒定成立不再追/interestfor −3/variant nameset 逐字同/合成 Fail 扫描）；邻账 edf8ce921 TT 刀兑现 tt +912/san +21（组行差＝清单外三键 zw_only 分母契约非异常）＋4fb594667 编辑刀首窗 editing +96 正向、dom/ −350 对端碰撞归 wptfyi-dom 待定谳（消失子测试＝nodeValue/textContent/data/replaceData 直写形，本地 FILTER 复现页级 TO）；快照组差 +632＝1,930,093→1,930,725 闭合：[批二十四证据](evidence/2026-11-23-dc3-batch24-run-reconciliation-perglobal-editing-knife-and-dom-collision.md) + [逐文件 JSON](evidence/2026-11-23-dc3-batch24-run-reconciliation-perglobal-editing-knife-and-dom-collision.json)；批二十四备料＝TO 面 83 键根因定谳收口（script/stylesheet 车道探针三车道同形＋逐键归因＋四键 dw 非）：[根因定谳证据](evidence/2026-11-23-dc3-batch24-speculative-to-face-rootcause-probes.md) + [逐文件 JSON](evidence/2026-11-23-dc3-batch24-speculative-to-face-rootcause-probes.json)；批二十三备料＝per-global 支撑闭包（object-association.js 全局缺档＝隔离律第 8 例全局变体）57 脚本补拉，本地实跑 1 Fail→6 注册/3P/3F，+3P 批二十四兑现：[批二十三证据](evidence/2026-11-23-dc3-batch23-navigator-object-support-closure.md) + [逐文件 JSON](evidence/2026-11-23-dc3-batch23-navigator-object-support-closure.json)；45028417732（批二十一码）收账（快照 c22c3c0de）：html 面 112,102→109,404＝过滤纠偏 −3,002 精确兑现（三键 48/1,003/426＝1,477P/5,822 注册与本地探针逐字同、同值检测互异，第 7 例闭环）＋non-object +4 兑现（第 6 例闭环）＋dat +98/预期 +118（url 驱动兑现、write/write_single 110 键 CI 全 TO＝document.write 刀位挂账 −20）＋对端 ec4ef28dc RT 切片 2 溢出 +204（popover-invoking-attribute 0P→204P＋command/interest 反射 +3−interestfor −3 反转红归 RT 流域）＋onblur flaky −1 带内回摆；邻账 dom/ +222（5e217bdba 兑现）、web-components RT 三键注册全开 +8,640（CE·SD 邻账）、san/tt 首批 39P/31P（829c56fb1 兑现）：[批二十二证据](evidence/2026-11-23-dc3-batch22-run-reconciliation-three-fixes-and-rt-spillover.md) + [逐文件 JSON](evidence/2026-11-23-dc3-batch22-run-reconciliation-three-fixes-and-rt-spillover.json)；45017894086（批十九码）收账（快照 c7725c4de）：html 面 107,392→112,102＝dom +4,479（三查询键兑现但揭全页未过滤虚增＝subset-tests-by-key.js syntax-dom 分片恒缺，隔离律第 7 例，40 脚本补拉）＋semantics +231（批十八 14 键 230P＋onblur +1；−4＝non-object 4 json 缺档第 6 例，59 脚本补拉）＋其余十一组持位；CE·SD 组持位（+66P reference-target tentative 形不入 aligned face）：[批二十一证据](evidence/2026-11-22-dc3-batch21-run-reconciliation-and-subset-key-shard-gap.md) + [逐文件 JSON](evidence/2026-11-22-dc3-batch21-run-reconciliation-and-subset-key-shard-gap.json)；45008050513（批十七码）收账：webappapis 586→1,537 键级兑现（+919 event-handler＋+32 base64）＋syntax +43/infra +6 回位＋semantics −2 flaky；dom 三键落空定谳＝idl_test setup 各 1 Fail（Error fetching /interfaces/dom.idl，分片语料隔离律第 4 例）＝40 脚本补拉兑现（挪走实跑 cmp）：[批十九证据](evidence/2026-11-22-dc3-batch19-batch17-run-reconciliation-and-domidl-shard-gap.md) + [逐文件 JSON](evidence/2026-11-22-dc3-batch19-batch17-run-reconciliation-and-domidl-shard-gap.json)；批二十备料＝五簇甄别＋窗腿姊妹键校正（接线杠杆取消），揭出 html5lib 170 键 CI dat 404＝隔离律第 5 例，ci-shards.yml 补 fetch-wpt-html-syntax-dat：[跑道勘误三](evidence/2026-11-22-dc3-goal-accounting-and-corpus-runway.md) + [清点 JSON](evidence/2026-11-22-dc3-batch20-scan-domain-lever-census.json)；批十六码 44996021189 收账（html 面 106,394 净 +35,298＝sab.js +148 兑现；瞬态网络窗 syntax −43/import-maps −37 下批自愈回位兑现）：sab.js +148 兑现（webappapis 438→586），计划缺口 −2,313＝WebIDLParser 生成路由 404（上游非 Git blob，57/59 改真身落别名）＋base64 +32 预期内未含；瞬态网络窗 syntax −43／import-maps −37（下批自愈）；批十七 run 在飞承接 +919＋+1,394＋+32＋回位：[批十七证据](evidence/2026-11-22-dc3-batch17-webidlparser-alias-and-rerun.md) + [逐文件 JSON](evidence/2026-11-22-dc3-batch17-webidlparser-alias-and-rerun.json)；44985566993（批十五码）五分片逐键收账＋aggregate 快照（全量 1,914,204→1,917,413 +3,209）逐组对齐：infra 82→635／webappapis 3→438／browsers 217→257／semantics 8,508→10,467／syntax 2,747→2,790／xhtml·dom·rendering 持位，唯 webappapis 距本地 face 1,099＝html.idl −919＋sab.js −148＋base64 fetch_json 缺档 −32，57 脚本三 fetch_raw 落码：[批十六证据](evidence/2026-11-22-dc3-batch16-reconciliation-and-closure.md) + [逐文件 JSON](evidence/2026-11-22-dc3-batch16-reconciliation-and-closure.json)；目标级前置账（余量主序 dom 12,818/semantics 12,022/rendering 8,924/syntax 6,223/canvas 3,436/webappapis ~2,170/browsers 3,070/七簇 2,131＋语料跑道缺失 2,768 文件/14,413 子测试）：[前置账证据](evidence/2026-11-22-dc3-goal-accounting-and-corpus-runway.md) + [逐文件 JSON](evidence/2026-11-22-dc3-goal-accounting-and-corpus-runway.json)；控制点 44972755575（infra=82 形）与 44985566993 预期形记档见[批十五证据](evidence/2026-11-22-dc2-batch15-ci-shard-corpus-channels.md)；接入预检五通道落位＋syntax-dom 预算修正 3600→13,200s：[接入预检证据](evidence/2026-11-22-dc3-weekly-integration-preflight.md)；实体矩阵随打包漂至 catch-all-9/10（编号不锁），rendering 通道入 catch-all-3 已首跑入账（2,793 持位）；批十二后 discovery 重跑（147 套件）：speculative 通道入 catch-all-4 兑现 43 P |

碰撞记录：2026-11-20 rendering 解锁刀（engine shim 同任务 DOM 可见性）与零-web 流
js_dom_bridge 域重叠且其在途 WIP 在同文件——本流不动刀，记待碰头；UA 样式表值差
刀与 rendering 流域重叠，同待碰头。2026-11-21 批八战役图新增两刀同性质待碰头：
Node.normalize 上移共享 Node 原型（js_dom_shim 面，363 F）、document open/write
管道挂起（engine 域，110 键 TO）。2026-11-21 批九：workspace clippy `-D warnings`
门禁被渲染流域提交破坏（f25940399 css-parser doc_lazy_continuation 缩进、
21e0ea83d/22787ab21 style-system computed_style 缺字段文档注释 + apply let-else、
layout-engine too-many-arguments ×2 需签名重构）——渲染流域，待该流或碰头修复；
本批门禁降级为 scoped `cargo clippy -p zero-wpt-runner --all-targets`（本批文件
零警告；批十复扫同样 25 条既有警告全在渲染流域文件，runner 本域零警告），
workspace 门禁恢复前各流提交均需自行 scoped 核验本流域文件。
2026-11-22 凌晨：wptfyi-css 流 stale 工作树提交 ee2ad7855 连带删除批九全部交付
（模块/接线/Makefile target/fetch 脚本/两份 evidence/master.md 回退批八态，非有意
回滚，同 a8de9c124 自记事故型第二起）——已整批还原（f76250c81，blob 与原提交
逐一相同）+ 批十一准备提交 rebased（a62d221df）；跨流 stale 回写已两起，建议
碰头议工作树钉扎约定。同窗口 DC-3 preflight 三腿（本地 test-guard 链）发现
跨流 engine 漂移形：rendering 套件对批二基线**共享键 -62 Pass**（legend-display
-56、fieldset-display -11、video-aspect-ratio -1、aspect-ratio 族 +6 回冲；
批十证据含逐键 A/B），漂移键集中 computed display/UA 呈现域——批三后渲染流
engine 提交 3f05d6acf/21e0ea83d/22787ab21/f25940399 在途（11-21 19:33-22:00 入
HEAD，23:42 runner 二进制内嵌），归因渲染流域待该流定向定谳，不入本流回归账；
syntax 套件字节级零漂移（批八 after 四项 totals 逐一相同）、infra 套件仅
messagechannel.any.worker.js 单键负载抖动（Pass 102→29 键级超时截断，键集
不变，批十后跑 115 恢复带 +13 抖动带宽）——两套件无漂移面。2026-11-22 批十二
战役图新增 engine 刀位三形（探针实证，待碰头）：iframe/嵌套 write 管道
`document.write('<plaintext>')` kill 不生效（批八 write 挂起同域不同形）、
template 内容取数不惰性（template 资源照常 fetch）、非图片响应 img 载入永不
完成 → 页面 load 不触发（76 TO 主构成）——speculative 50 Pass 面依赖前两形，
engine 修后此面重排；本流不动 engine，记碰头输入。2026-11-22 批十七：
resources/WebIDLParser.js 恒 404 事故两课——① goal 脚本补档的幂等重跑在
本地在盘时逐文件跳过，FAILED_FETCHES=0 对新增 fetch_raw 零验证力，补档
验证必须在挪走目标文件后实跑；② 上游「文件存在」必须四路核对
（jsdelivr/raw/contents API/git trees），wptserve 生成路由形
（WebIDLParser=webidl2.js 的服务端别名）不落 Git，fetch_raw 永远失败——
57/59 改真身落别名，归档 99-webgl-compat.sh:39 同款休眠记档。同日
07:06–07:08Z CI 网络劣化窗一次（git trees 超时＋git clone 连接失败，
catch-all-4 双套件语料空转），优雅降级设计路径自愈，下批 run 回位。
2026-11-22 批十九：A/B delta=0 初判再坐实一课——「零注册」必须下分片
产物核对注册形（本轮真形＝三键各 1 子测试 idl_test setup Fail，非
tests:0 TO=0）；共享支撑件「本地在册」前提必查带入脚本是否在本分片
fetch 清单（dom.idl＝99-webgl 脚本带入，分片隔离律第 4 例）。
2026-11-22 批二十：隔离律第 5 例变体形——语料 fetch 目标只在 Makefile
套件依赖链（testharness-html-syntax → fetch-wpt-html-syntax-dat）而
分片 fetch 步直跑清单不经 Makefile 套件依赖，批八 dat 语料 CI 恒缺
（170 键各 1 synthetic Fail `fetch <file>.dat: 404` 隐身于 syntax
2,790 持位面内）；「A/B 持位≠语料在册」——对账时 synthentic Fail
message 扫描（fetch 404 形）应入对账例行。同轮甄别教训：窗腿分母键
（.any.html/.window.html）与 zw 键（.any.js/.window.js）同测异名，
未计分面核算前必须做姊妹键残差校正，否则虚增（本轮 195 键 1,101S
名义残差大头实为已计分）。
2026-11-22 批二十一：隔离律第 6/7 例＋新虚增形两课——① 支撑闭包枚举
漏「源文件自身 import 闭包」（non-object.any.js 4 json：批十八按静态
引用枚举 20 文件，漏源文件 import 的 null/true/false/string.json，
CI 形 4 Fail `Failed to fetch module` 隐身为键内低 Pass 不报错）；
② 过滤助手缺档是**静默虚增**形（subset-tests-by-key.js 只有 62 脚本在
清单，syntax-dom 分片恒缺 → 三 variant 键各跑全页不报错不 TO，
face +4,479 中 ~3,002 为重复计入——「A/B 持位≠过滤生效」，本地探针
复现过滤形（三键不相交）与 CI 全页形（三键逐字节同）比对即定位；
CI 三键同值＝未过滤指纹，对账例行应加「variant 键族同值检测」）。
修过滤缺档 = face 回吐非增益，记账纠偏照实入账不粉饰。
2026-11-23 批二十二：对端 engine 刀首次在 html 面内溢出实证——批十六只
预登记 dom/ 面观察，本轮 ec4ef28dc（RT 切片 2，批二十一 run checkout 之后
落 main）对 html/semantics 补簇溢出 +204（含 interestfor −3 反转红）。
两 run A/B 必须先核对分片 checkout 窗口内落 main 的跨流码刀再归因，
正负位移都照实入账并标注对端 commit；同 run 另证 dat 语料到位后 CI 实跑
首证 write 驱动 TO 形——「语料修复兑现」预期须按驱动形态拆分（url 驱动
兑现 +98、write 驱动落 engine 刀位 −20），不按键数一刀切预估。
2026-11-23 批二十三：隔离律第 8 例全局缺档变体两课——① 支撑闭包扫描的
盲区＝页面从未实跑：静态 <script src> 扫描（批十五）与运行期引用扫描
（批十六）都以「页面跑过并留下信号」为前提，从未运行的页面两法皆不可见，
唯一可见通道是合成 Fail 例行扫描（批二十教训）的 os error 2 形——对账
例行必须保留该扫描，不因前两法已入册而裁撤；② 「本地在册」前提的对称
面＝「本地也缺档」：批十九教训查分片 fetch 清单之前，先确认文件本地是否
真的在册（本轮 object-association.js 两处皆无，修复位＝语料 owner 脚本
而非分片清单）。
2026-11-23 批二十四：对端刀首入窗双向扫溢出＋快照两口径各自闭合两课——
① 4fb594667 编辑路径刀首入 weekly 窗：本域正向 +96（editing 面）同时
dom/ 面溢出负位移 −350（Range-mutations 两键 P→None 未注册＋页级 TO，
消失子测试恰为该刀改写的 nodeValue/textContent/data/replaceData 直写形，
本地当前树 FILTER 复现页级 TO）——「对端刀首入 run 窗按域双向扫溢出、
正负位移都入账」与批二十二课并列入对账例行；机制证据链（逐键 flips＋
本地复跑）移交 wptfyi-dom 对端刀作者定谳，本流不动 engine 不修。
② 快照组行账与套件产物账首次出现可感差（tt +887 vs +912、san 持 39
vs +21）——定谳＝eval-csp-tt ×2＋sethtml-tentative 三键在 wpt.fyi
aligned stable 清单外（resolve None 入 zw_only），两口径皆为真：套件账
证刀兑现全额、组行账按并集清单口径记账；对账遇组行差先查 resolve 归属
再谈管线异常（本地 wptfyi_gap.py coverage 同 cache 重跑复现即定谳）。
2026-11-23 批二十五：对端刀簇双向扫溢出第二演＋停摆形归因降级课——①
TT/Sanitizer 刀簇（abfa08966/6346b9981/efb757973）正负两向同时入账：正向
tt +338／san +263／Sanitizer IDL 面 +16 兑现；负向 CE·SD −2,772
（property-reflection-idl-setters 2,910→225 P→F 整面翻红，注册不变）——
**对端自家 runner 全桥 fail=3 持位＝该溢出对其看门狗不可见**，coverage
通道为其唯一探测器（探测不对称第二实证），证据链移交对端定谳修复。
② 批二十四 dom/ −350 的 4fb594667 归因**降级为「嫌疑形＋timing 不稳定
并存」**：批二十五窗内无任何 dom 侧修复落码而 dom/ranges +120 部分回位
（同款停摆形再现 editing/run −220）＝停摆形对负载/时序敏感、非确定码
行为——**对端刀后停摆形归因必须等下一窗回位证据再定谳，单窗形不定谳**
入对账纪律（与批二十二「先核窗内跨流码刀再归因」并列）。
③ 快照环比护栏未拦截 −2,286 总回落（快照照常回写 main）＝护栏判定域
观察记档，归 wptfyi-data 域待碰头。
2026-11-23 批二十六：停摆形三窗定谳＋组行反推课——① 停摆形三窗交替
震荡零码关联（b24 dom −350/editing +96 → b25 dom +120/editing −220 →
b26 dom −247/editing +223 完全回位）＝**timing 形定谳强化**：停摆形归
负载/时序带非确定码行为，4fb594667 机制归因维持降级；后续读数以多窗
水位带为准（dom/ranges 42,173~42,420、editing/run 95,130~95,353），
单窗回位/加深不作修复证据。② **组行反推课（勘误入账）**：批二十五把
html/semantics 组行 +1 误读为 track-cues「持 Pass 第二连演」——artifact
实为 Fail，组行 +1 实为 onblur +2 与 track-cues −1 净和；四演真形
Fail→Pass→Fail→Pass 交替带。「组行 +1 不能反推单键 Pass，单键判定
必须下 artifact 验键」与批十九/A/B delta=0 初判课同族入对账例行。
③ CE·SD −2,772 持位第二窗（③b②/③③a 不修不加深全同值）＝溢出
归因面稳定，对端修复读数通道不变。
2026-11-23 批二十七：分片 checkout 逐 job 实测课＋交互窗归账课——①
分片 checkout 必须**逐 job** 从 log `git log -1 --format=%H` 提取：API
`head_sha`（workflow_dispatch 时点）≠ 分片实际 checkout（jobs `ref:
main` 在 job start 时各自解析）；本窗 dispatch 后 4s 落码＋55 分片
跨 8 分钟启动＝60 分片双 SHA 共存（45 dc81c2168＋10 956c94cef），
docs-only 领先分片按「等码性判定」（逐 commit 验零码差）归并同码窗
后才可全窗归因。② CI aggregate 每次新拉 aligned manifest＝分母漂移
常态（本窗 +593），本地**旧 cache** 复现 face 行做双证（分子与分母
版本解耦）。③ 两刀同窗时「自报之和 ≠ 实测」的差值先按域分组再定
交互窗（本窗 tt +114＝③③b +88 精确＋③⑤ 名单域 +11＋交互窗 +15），
交互域（default-policy 族）单刀自报均不含＝如实记交互窗，不冒充
未知漂移；负向扫溢出（test262 −13/csp −4）继续「单窗不定谳、下窗
回位/加深再判」纪律（批二十五课续）。
2026-11-23 批二十八：cache key 误抓复发＋对端自报环境差课——①
SHA 提取 fallback grep 40-hex 再次抓到 Cargo.lock cache key
（b8a594ea5，不在本地对象库）＝批二十七课复发实证：**提取的 40-hex
必须做对象存在性＋祖先链检查**（`git merge-base --is-ancestor <码刀>
<sha>`），不在对象库即非 checkout SHA；job name 经 `--jq --arg` 提取
全空（gh CLI 无 --arg flag）不影响 SHA 判定。② 对端自报与 CI 读数
环境差两形如实记不深挖：sanitizer-custom-elements-constructor 对端
报净零、CI +8P（清单外正溢出）；upgrade-criteria 对端报 4F→页级
TO、CI regs 14→11（3 消失＋1 F→TO）＝「自报 vs CI 冲突先记环境/语料
差，连续两窗同形再定谳」。③ 负向两形第二窗零回位零加深＝「判读窗
顺延」首次演（不定谳期不自动定谳，顺延须显式记窗号）。
2026-11-23 批二十九：单键验键复发＋布防等待窗课——① 批二十八把
track-cues「零变化」直接写成「Pass 持位第六演」，artifact 实为 Fail
持位——**「零变化＝上一窗状态持位」仍须下键验 status**：上一窗的
判定本身可能是错的（本例批二十八未验键、沿用了批二十六预测），
勘误随下一批 evidence 落账（批二十六组行反推课同族第二演）。
② /tmp/batch29/autodl.sh 等待循环 60×120s＝2h 上限被 run 实际时长
~2h16m 击穿，marker 出现前 16 分钟误判 timeout 退出＝**布防脚本
等待窗必须 ≥ run 时长历史上界 + 余量**（批二十七/二十八 ~2h20m 为
参考上界，取 4h+ 安全）；marker 已在时幂等重跑即成功，无产物损失。
2026-11-23 批三十：批间 sed 复制子目录名残留课两形——① 批二十九→
三十布防脚本复制用 `s|/tmp/batch29|/tmp/batch30|g` 只替换了路径前缀，
autodl.sh 内**字面量子目录名 run29-raw/run29 未替换**＝批三十产物
下载落 /tmp/batch30/run29/，ab30.py 以不存在的 run30/ 为 new dir →
glob 空 → 全套件 subs=0＝伪回归 −1,945,824——识别形：**P/F/TO 三值
全零＝缺档形非真回归**（真回归至少一侧非零）；**批间 sed 复制必须
同时替换子目录名字面量**（与路径前缀并列两条 s 命令）。② **A/B
首跑前必须预验证 new dir 存在且 subs 总量与基线同量级**（本轮两侧
各 150,582 互证后才采信 A/B）；waiter 另遇 gh API 网络抖动两拍
（error connecting to api.github.com）＝收账时人工核对 run 终态后
手动补写 RUN_DONE，autodl 幂等承接无损失，网络抖动不重试升级。
2026-11-23 批三十一：egress 503 整包下载课＋cache-key 误抓第三演两
条——① `gh run download` 整包式（--pattern 全量）先遇连接重置、
重试遇 **HTTP 503 egress over the account limit**（账号级带宽滚动
窗，多流并行下载挤占）：整包式无断点续传、一件失败全弃重来的形状
在限流窗内反复烧配额；**逐 artifact 韧形下载器**（已提取跳过＋45s
退避＋多轮循环）让重试只补缺口、滚动窗恢复即收齐（167/167）＝
批三十二起逐件下载为主路径。② fallback grep 40-hex 再次抓到
Cargo.lock cache key（b8a594ea5，批二十八同值）＝对象存在性检查
当场捕获（批二十八课第三演实证）；补提取用定向 `git log -1` 上下文
grep 取真身——**fallback 应收紧到 checkout 行锚**（HEAD is now at/
git log -1 邻近行），批三十二 shas.sh 已照此修。同窗布防全绿一窗
（批二十九等待窗课＋批三十子目录名课双双落地首演零复发）。
2026-11-24 批三十三：工具链两课——① A/B 报告 SIGPIPE 丢失：ab33.py
首跑经 `| head` 截断管道，python 在 json.dump 前被 SIGPIPE 杀死＝
分析 stdout 在、报告 JSON 缺（本次重跑补齐零损失）；**长输出必须
`> file` 重定向后再读，不在管道内截断**。② shas.sh 缺 cd 假完成：
batch32/batch33 两代脚本均无 `cd` 到仓库，batch32 靠调用方 cwd 侥幸
成立，batch33 从 /tmp/batch33 调用即 gh 全报错＋shas.txt 空＋仍打
DONE_ALL＝**脚本必须自带 cd＋调用方验收输出行数（期望 55）再续，
DONE_ALL 非成功证明**（批三十子目录名复制课同族第三演，批三十四
脚本族已照修：自带 cd＋行数验收＋A/B 重定向）。同窗双码刀窗范式：
**dispatch 前先核对 main 码刀（祖先链＋docs-only 互 diff 判均匀窗），
dispatch 后收账前再 pull 复核零第三刀**（be78d24cf＋a2295242d 双刀
全分片携带＝归因窗均匀，避免批三十一「刀落分片起跑后唯 aggregate
捡到」的零归因形）。同窗布防下载侧全绿第三窗（retrydl 54 artifact
首轮全 ok 零失败）。
2026-11-24 批三十七窗：跨流 wholesale 碰撞第三起——96ee337b4（css 流
shim③ 刀）落库时 wholesale 卷入他流面删除：本流批三十六 evidence 两文件、
rendering-compat.md、dom 刀③ a3e299907 源面/钉测/evidence 均被删；对端
73f4d5dcf 还原 CE 源面（7 文件 PURE-STALE 逐位恒等直证＋part03 合成 blob）、
6bccb8472 接岗轮还原 9 路径 docs 含本流两文件；本会话独立自还原 commit
78b745e81 被 pull --rebase 判「补丁内容已在上游」丢弃＝零净损。与 11-22
ee2ad7855 stale 回写（第二起）、a8de9c124 自记事故（首起）同族——wholesale
落库前须 `git status` 核对暂存面不含他流域文件，工作树钉扎约定建议
（11-22 已提，第三起强化）。
