# s632 收账证据——非平凡窗＝select-event.html 页级双腿超时塌陷形（s626 以来首个 join 位移轮＝stable 8517→8477）＝对端重腿负载归因非代码位移定谳（runner 候选五十一持位形兑现＝pre=mid=post b0919257 同值持位＝窗码面 docs ×7 码 ×0）＋b632 不动结果面持位第 6 轮＋'No such' 归零第 27 演＋同 session 等窗收账闭环

日期：2026-11-22（脚本生成 16:00／三臂起动 16:00:45；候窗 ~1m30s；窗开 16:02:15；全毕 16:11:08）
环境：本机双 clone 并行；`/tmp/s632`；s631 收账 commit（6d7407da2，rebase 于对端 b45e662fd 之上）先于布防落地＝钉扎纪律第 8 演零竞争；legs.head=6d7407da2。
同 session 等窗收账闭环（非断点轮）；候窗 clean-soaked 零门拒段（16:00:45→16:02:15＝procs=0 load<3 across 3×45s；rearm.s632.log 空＝零真重臂＝两臂齐全第 31 演零空窗）。

## 窗与 legs

- 布防 16:00 脚本生成＋16:00:45 三臂起动（waiter 直臂＋rearm 监督者＋post 分开臂）。
- **窗码面＝候选五十一持位形主预判前提兑现**：窗＝2f5923b96→legs.head 6d7407da2＝**docs ×7 码 ×0**（本流 1〔6d7407da2 s631 收账〕＋对端 6〔b29107e7e 批十六勘误＝goal 脚本面非 runner 码／6f5c59633 wptfyi-css evidence／faf5a79db＋eac455dd5＋b45e662fd rendering R5908/R5909／2aea16bda wptfyi-html 批十六时序勘误＋跨流碰撞还原〕）。
- **窗开即遇对端重腿负载**：leg1 期 load 1min 峰 **22.51**（16:06 实测；judge 16:02:15 clean 后对端重腿随即起动）＝leg1 3m49s／leg2 3m06s（对照 s631 2m32s/1m56s 明显拖慢）＝门拒窗未触发（窗已开）照规跑完。
- leg1 16:02:15→16:06:04（3m49s）rc=2，**Compiling ×6＝重编产物同值形**（窗码面码 ×0＝产物 byte 同值＝重编非换代）；leg2 16:06:04→16:09:10（3m06s）rc=2，Compiling ×0。
- **runner 候选五十一持位形兑现**：pre=mid=post=**`b0919257`**/129,706,824 三点同值持位（s631 新锚持位）。
- 桥测（run-legs 内，同 recipe）：build 1m08s（test profile；Compiling ×3）+ test ~20s，rcb=**101**。
- solo1/solo2 16:10:46→16:11:08（~11s/~11s）rc=2 ×2；failset（`^Fail` 原始序）md5 `329e35ed` 双 solo IDENT ≡s611–s632＝发射形同值第 25 连。
- all.done 16:11:08；窗起至全毕 **8m53s** 零空窗（两臂齐全第 31 演）。

## join 判定（analyze.out）＝非平凡窗＝select-event.html 页级双腿超时塌陷形

- `stable=8477 disagree=0 l1only=23 l2only=0`；**NEW=1 GONE=41 FLIPS=0**；分布 8264P/**206F/6T**/1U（Timeout 5→6＝页级键新增；206F/1U 持位）。
- **塌陷形解剖**：`select-event.html` 页级键（`:: select-event.html`）双腿 **Timeout**（baseline 无此键＝NEW=1）＝页超时截断子测试注册。password 块 45 keys（baseline 计）→leg1 注册 27／leg2 注册 4：held（双腿齐）=4、l1only=27−4=**23**（全部 password 块）、双腿均缺=45−27=**18**、GONE=45−4=**41** ✓；select-event 全页 279 子测试键→leg1 注册 261（−18 全 password 块）／leg2 注册 238（−41 全 password 块）＝塌陷面严格限于 password 块。其余 8516−279 键零位移。
- s632.stable.canon md5 **`afe83abc`**（8477 行）；keys md5 `555f07d0`（塌陷形基线）。
- **非代码位移定谳**：窗码面码 ×0＋runner 三点同值＋双腿 norm md5 分叉（leg1 `bb7e1f9b`／leg2 `b4a1bf24`＝塌陷深度不同）＋唯一环境变量＝对端重腿负载（峰 22.51）＝**负载归因页级超时塌陷形**（页级 T 形谱内在册先例＝band 谱"页级 T"语境；非 band8 键）。
- **回休观察**：s633 预期 password 块回位 45＋页级键消失→stable **8517**＝canon `f1aef7b7` 回位＝链复挂第 30 连（s626→s627 断一键回环形同族先例）。

## b632 判定（bridge.out）＝"不动"结果面主预判兑现＝同值持位第 6 轮

- keys=**1736** fail=**2** NEW=0 GONE=0；s632.bridge.keys md5 `bb075eee0e34d743166b19a028e9dde6`（≡b627=b628=b629=b630=b631＝同值持位第 6 轮）。
- FAIL 双键持位＝`min_max_single_arg_collapses_to_argument_terms`（对端 v3 刀挂账续）＋`test_get_computed_style_numeric_special`（对端 FAM-F 刀挂账续＝part03.rs:1297 期望未修）＝**均归对端刀作者面，本流不代修（规则 9/10）**。
- 窗桥触点核实＝**零**（docs ×7 无 .rs/.js）＝"不动"分支前提与结果面双双成立。

## 哨位（analyze2.out，SENTINELS_DONE）

dialog band r66＝2P 双腿（flaky 键持位第 21 轮）；**select-event r71＝262/239 keys 8F Fset ≡ se.577**（塌陷形下 8 个已知 F 键双腿全注册＝F 键先于页超时完成＝Fset IDENT 恒）；**password r71＝207/184 keys 0F**（−18/−41＝塌陷形映射）；**mpreload r94＝15/15P 双腿直证＝回休后持位第 5 轮兑现**；waapi r153＝picker P/checkmark F 双腿；focus-dynamic r112＝121 all P；onblur r64＝10 keys 8P/2F Fset 双 IDENT ≡ob.s559（稳位第 32 轮）；族带 r65＝band8 hits=8×2 全 Fail（聚合形持位第 49 连，页级 T 于 band8 零再现）；双发伪影 ×2 恒；爆栈 so1/so2/sob=0/0/0；**'No such' 归零审计 0/0 第 27 演**（预装修复形第 4 轮续）。轮次标签全对齐（r66/r71/r71/r94/r153/r112/r64/r65＝与 s632 布防计划算术一致）。

## 窗底账（s632 窗＝2f5923b96→legs.head 6d7407da2＝7 commit 全列＝docs ×7 码 ×0）

- 本流 1：6d7407da2〔s631 收账＋s632 布防〕。
- 对端 6：b29107e7e〔wptfyi-html 批十六勘误补档＝goal 脚本面非 runner 码〕＋6f5c59633〔wptfyi-css KC-3 evidence 补档〕＋faf5a79db〔rendering R5908 部署窗追记〕＋2aea16bda〔wptfyi-html 批十六时序勘误＋跨流碰撞还原＝对端域〕＋eac455dd5〔rendering R5908 收尾 pull 补记〕＋b45e662fd〔rendering R5909 收账〕。
- 布防补记（docs ×2＋goal 脚本面 ×1＝码 ×0）与实况偏差＝布防后落窗 4 commit（全 docs）＝持位形前提保持。

## 基线换装

- join：s632.stable **8477**（555f07d0/afe83abc＝塌陷形基线；s633 回休形主预判＝password 块回位＋页级键消失→8517/f1aef7b7 回位观察；对端重腿再现＝塌陷形重开按形记账）。
- 全桥：b632 keys **1736**（bb075eee 同值持位第 6 轮；fail=2 双挂账均归对端刀面续）。
- runner：**b0919257**/129,706,824 持位（候选五十一持位兑现入账；候选五十二挂观察＝s633 窗在飞 watch）。
