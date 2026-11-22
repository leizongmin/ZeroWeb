# s638 收账证据——非平凡窗＝mpreload 谱第三键 origin-only 单腿摆动（落 leg2＝先行位 2 连止）＝谱内摆动第 3 演＝同文件族键位轮换第三键（stable 8516 持位＝canon 链断第 3 轮 f063fa74＝disagree=1 GONE=1＝谱内摆动非码移）＋leg1 norm c735d371 整腿复现链第 41 连（leg2 c9bf9480 断一键形＝s636 镜像位形）＋runner 候选五十七持位形兑现（pre=mid=post 4fe4ccc6 三点同值＝新锚第 1 轮＝leg1 Compiling ×6 重编产物同值形）＋b638"不动"结果面持位第 12 轮（触点 ×0）＋'No such' 归零第 33 演＋同 session 等窗收账闭环

日期：2026-11-22（三臂起动 19:29:25；候窗 6m00s＝clean-soaked；窗开 19:35:25；全毕 19:42:20）
环境：本机双 clone 并行；`/tmp/s638`；s637 收账 commit（5dc836667）先于窗开落地＝钉扎纪律第 14 演零竞争；legs.head=5dc836667。
同 session 等窗收账闭环（非断点轮）；两臂齐全第 37 演零空窗（rearm.s638.terminal.log 0 字节＝零真重臂）。

## 窗与 legs

- 布防（s637 收账 commit 内 19:2x）脚本生成＋19:29:25 三臂起动（waiter 直臂＋rearm 监督者＋post 分开臂，ps 三进程验证 1413636/1413632/1413634）。
- **窗码面＝开工捕获实窗全列**：窗＝d083daa37→legs.head 5dc836667＝**7 commit＝docs ×3＋cron 快照 ×1＋merge ×1＋renderer 产品码 ×2**（本流 1〔5dc836667 s637 收账＋s638 布防＝docs〕＋对端 6〔**码 ×2：1b29406c7＋0cd331fa8＝T91 nav phase marks＝apps/renderer＝非本流构建图〔wpt-runner/zero-engine 不依赖 apps/renderer〕**；merge 16d2b1d97；docs ×2：efc04ef4d wptfyi-css KC-5／17f9c688a wptfyi-html 前置账勘误；cron 快照 9a30b4394〕）。布防预载 4 commit→实窗 7＝开工捕获补录（9a30b4394＋17f9c688a＋本流收账 commit 自身入窗）＝实窗为准。
- 候窗 6m00s（19:29:25→19:35:25）；window.s638.judge＝**clean-soaked procs=0 load<3 across 3×45s checks**。
- leg1 19:35:25→19:38:31（3m06s）rc=2，**Compiling ×6＝重编产物同值形**（窗码面非 runner 构建图码＝产物 byte 同值＝重编非换代，s636 同族形）；leg2 19:38:31→19:40:34（2m03s）rc=2，Compiling ×0。
- **runner 候选五十七持位形兑现**：pre=mid=post=**`4fe4ccc6`**/129,763,000 三点同值持位（s637 换代新锚持位第 1 轮）。
- 桥测（run-legs 内，同 recipe）：test 26.87s，rcb=**101**（fail=2 挂账双键所致＝预期形）。
- solo1/solo2 19:41:45→19:42:04→19:42:20（~19s/~16s）rc=2 ×2；failset（`^Fail` 原始序）md5 `329e35ed` 双 solo IDENT ≡s611–s638＝发射形同值第 31 连。
- all.done 19:42:20；窗起至全毕 **6m55s** 零空窗（leg1 3m06s／leg2 2m03s／桥 26.87s／solo ~19s＋~16s）。

## join 判定（analyze.out）＝非平凡窗＝mpreload 谱第三键单腿摆动＝held 主预判（平凡窗持位形）落空按形记账

- `stable=8516 disagree=1 l1only=0 l2only=0`；**NEW=0 GONE=1 FLIPS=0**；分布 8304P/206F/**5T**/1U（Pass 8305→8304＝唯一位移键脱离 stable）。
- **位移键＝mpreload 谱第三键**：`html/semantics/scripting-1/the-script-element/module/modulepreload-referrerpolicy.html :: Modulepreload with origin policy should send origin-only referrer`——held 基线（s634.stable.canon）**Pass**→本窗 **leg1 Pass／leg2 Fail**（disagree=1＝GONE=1 同键；**摆动键落 leg2＝s635 ob/s636 mpreload 均落 leg1 的先行位 2 连止**；谱内摆动非码移＝窗码面 engine/桥零触点）＝**谱内摆动第 3 演＝同文件族键位轮换第三键**（s626 no-referrer→s636 unsafe-url→s638 origin-only＝三键各单腿瞬态）。
- **leg1 norm md5 `c735d37188a0fa0ea338b47d2432a26f`＝整腿复现链第 41 连 s597→s638 持位（leg1 逐字节 ≡s635–s637）；leg2 norm md5 `c9bf94800f293b0a9b62a0689f68280e`＝断一键形＝与 leg1 差仅该键＝s636 位形镜像**（s636 leg1 断/leg2 链→s638 leg1 链/leg2 断）。
- s638.stable.canon md5 `f063fa7432dca67796c66dae08bd0b10`（≠f1aef7b7＝**链断第 3 轮**：s635 80d67629／s636 eb541be0／s638 f063fa74 三轮离链且互异）；keys md5 `e3a79361eca9fbfda907cbebbb9fe845`（≠057764d3）。
- 布防对照：平凡窗持位形主预判落空＝mpreload 谱内摆动再现（副形"谱内摆动再现第 4 演"兑现——布防预判演次按单键计，本窗为谱内第 3 键首演）＝flaky 键单腿瞬态不可预报＝按形记账非布防缺陷。

## b638 判定（bridge.out）＝"不动"结果面主预判兑现＝同值持位第 12 轮

- keys=**1736** fail=**2** NEW=0 GONE=0；s638.bridge.keys md5 `bb075eee0e34d743166b19a028e9dde6`（≡b627=…=b637＝同值持位第 12 轮）。
- FAIL 双键持位＝`min_max_single_arg_collapses_to_argument_terms`（对端 v3 刀挂账续）＋`test_get_computed_style_numeric_special`（对端 FAM-F 刀挂账续＝part03.rs:1297 期望未修）＝**均归对端刀作者面，本流不代修（规则 9/10）**。
- 窗桥触点核实＝**零**（窗无引擎/桥 .rs/.js＝renderer 码＋cron 快照＋docs）＝"不动"前提与结果面双双成立（s637 非零触点持位→s638 零触点持位＝两形皆按面判读）。

## 哨位（analyze2.out，SENTINELS_DONE）

dialog band r72＝2P 双腿（flaky 键持位第 27 轮）；select-event r77＝279/279 keys 8F Fset ≡ se.577＝持位；password r77＝225/225 keys 0F＝持位；**mpreload r100＝l1 15P／l2 14P/1F＝摆动键落 l2（谱内摆动第 3 演首演键）**；waapi r159＝picker P/checkmark F 双腿；focus-dynamic r118＝121 all P；**onblur r70＝双腿 2F/8P Fset 双 IDENT ≡ob.s559＝稳位第 38 轮**；族带 r71＝band8 hits=8×2 全 Fail（聚合形持位第 55 连）；双发伪影 ×2 恒（dup ×4）；爆栈 so1/so2/sob=0/0/0；**'No such' 归零审计 0/0/0 第 33 演**（预装修复形第 10 轮续）。轮次标签全对齐（r72/r77/r77/r100/r159/r118/r70/r71＝与 s638 布防计划算术一致）。

## 窗底账（s638 窗＝d083daa37→legs.head 5dc836667＝7 commit 全列）

- 本流 1：5dc836667〔s637 收账＋s638 布防〕＝docs。对端 6＝码 ×2（1b29406c7＋0cd331fa8＝renderer T91＝非本流构建图）＋merge 16d2b1d97＋docs ×2（efc04ef4d wptfyi-css KC-5／17f9c688a wptfyi-html 前置账勘误）＋cron 快照 9a30b4394。engine/桥触点 ×0。
- 布防预载→实窗偏差如实入账：预载 4 commit（renderer ×2＋merge＋docs ×1）＝实窗 7（补 cron 快照＋对端 docs ×1＋本流收账 commit）＝码面判读不变（renderer 码非本流构建图＝join 码移形候选不触发；实窗 zero engine/桥触点）。

## 基线决策（s639）

- join：**对账基线继续 held＝s634.stable 8517**（f1aef7b7/057764d3）＝断一键回位形主预判第 4 演观察（回位形在册 2 连先例〔s626→s627＋s636→s637〕＝mpreload 谱第三键回 Pass 双腿→8517 回位＝canon f1aef7b7 链复挂第 34 连观察）；副形＝谱内摆动再现（同键或谱内第 4 键单腿瞬态＝记形）；迁移形＝双腿 Fail FLIPS=1＝键迁 Fail 域＝基线迁移决项提请用户。不采 s638.stable 8516＝谱内在册 flaky 键单腿瞬态＝焗入基线固化瞬态（s626→s627/s636→s637 两先例＝回位后链复挂）。
- 全桥：s639 桥基线＝**b639＝cp s638.bridge.keys**（keys 1736＝bb075eee 第 12 轮持位；s639"不动"主预判＝第 13 轮观察；对端若修 part03.rs:1297＝numeric_special 回 Pass keys 1737 fail=1 销账形）。
- runner：**候选五十七持位主预判续＝4fe4ccc6**/129,763,000（持位第 2 轮；leg1 Compiling ×6 重编产物同值形预期；对端若推 runner 依赖域刀＝换代/重编按面归因＝候选五十八条挂号）。
