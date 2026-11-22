# s628 收账证据——平凡窗零位移轮＝join 链复挂第 26 连（stable 8517 持位＝布防零位移主预判逐字兑现）＋b628"不动"分支兑现（keys 1736 fail=2 bb075eee 同值持位）＋runner 候选四十七兑现＝新锚 db62802c＋布防预装审计两缺（跨 session 断点续收）

日期：2026-11-22（布防 ~12:43；候窗 ~11m；窗开 12:55:02；全毕 13:01）
环境：本机双 clone 并行；`/tmp/s628`；s627 收账 commit（0f184e0bf）先于布防落地＝钉扎纪律四演零竞争；钉扎 0f184e0bf＝legs.head。
**跨 session 断点**：s627 session 布防后 12:57 撞 429 限额中断（限额重置 14:17）；两臂 detached 存活＝窗自动跑完＝s628 session（本 session）开工捕获收账（s614/s615/s618/s624→s625 断点先例系列续演）。

## 窗与 legs

- 布防 ~12:43 三臂起动（waiter 直臂＋rearm 监督者＋post 分开臂；rearm.s628.log 不存在＝零真重臂）；候窗 ~11m（12:55:01 clean-soaked procs=0 load<3 across 3×45s checks＝机器空闲形）。
- **平凡窗＝布防零位移主预判兑现**：窗码面 6 commit 含对端 style-system 码刀 2aa1c9288（KC-1 contain-intrinsic-size＝runner 依赖域直源三次）＝**join 语义隔离预期成立**（style-system contain-intrinsic-size 域与 semantics 语料隔离＝平凡形）。
- leg1 12:55:02→12:57:44（2m42s）rc=2，**Compiling ×8**（product-version/style-system/net/layout-engine/engine/page-runtime/webview/wpt-runner＝KC-1 刀重建＋runner relink 重编形＝布防预判命中；较 s627 ×7 +1＝style-system 面增量）；leg2 12:57:44→12:59:34（1m50s）rc=2，Compiling ×0。
- **runner 候选四十七条挂号兑现**：pre=`0f4dc6a1`/129,701,424（stale＝s627 新锚转旧）→ mid=post=**`db62802c`**/129,701,624（新二进制）＝"必被覆盖"形精确兑现（+200 字节）。
- 桥测（run-legs 内，同 s627 recipe）：build 45.67s（test profile；Compiling ×5＝product-version/style-system/net/layout-engine/engine＝较 s627 ×4 +style-system＝KC-1 刀增量）+ test 21.87s，rcb=**101**。
- solo1/solo2 各 ~11s 级 rc=2 ×2；failset（`^Fail` 原始序）md5 `329e35ed` 双 solo IDENT ≡s611–s628＝发射形同值第 21 连。
- all.done 13:01；窗起至全毕 **~5m58s** 零空窗（两臂齐全第 27 演）。

## join 判定＝零位移主预判兑现＝链复挂第 26 连

- `stable=8517 disagree=0 l1only=0 l2only=0`；8519 行 norm ×2＋canon ×2；集级 206 Fail/8305 Pass/5 Timeout/1 Unsupported（≡s627 全分布持位）。
- **NEW=0 GONE=0 FLIPS=0**：s628.stable.keys md5 `057764d38ee311a47c34ecfc3e1e3192`（≡s627＝同值持位）；s628.stable.canon md5 **`f1aef7b7d7eafeeef3dd0989ba6bc3d9`**（≡…＝s626=s627＝**链复挂第 26 连**）。
- 双腿 norm md5 双双 `c735d37188a0fa0ea338b47d2432a26f`＝**leg1 整腿复现链回挂第 32 连**（s597→…→s627→s628 恒）；**per-leg canon 与 s627 逐字节相等**（diff 空）＝2aa1c9288 style-system 刀对 semantics 语料零位移实证（强于 md5 同值：byte 级全等）。
- ⚠ analyze.out "NEW=8517"＝**伪影非实测**（布防预装缺 s626.stable.keys/canon 且 analyze.sh 基线指针未换＝sort/join 读空流；权威 NEW/GONE/FLIPS 以 s627↔s628 md5 同值＋逐字节 diff 双证定谳，见审计缺段）。

## b628 判定（bridge.out）＝"不动"分支主预判兑现＝同值持位第 2 轮

- keys=**1736** fail=**2** NEW=0 GONE=0；s628.bridge.keys md5 `bb075eee0e34d743166b19a028e9dde6`（≡b627＝同值持位）。
- FAIL 双键持位＝`computed_calc_serialize_tests::min_max_single_arg_collapses_to_argument_terms`（对端 v3 刀挂账续）＋`tests::test_get_computed_style_numeric_special`（对端 FAM-F 刀挂账续＝part03.rs:1297 期望未修）＝**均归对端刀作者面，本流不代修（规则 9/10）**。
- **窗桥触点逐文件核实**：`git log 63e0cd0f1..0f184e0bf -- crates/engine/src/js_dom_bridge/`＝**零触碰**＝布防"不动"分支主预判逐字兑现（2aa1c9288 直触 `crates/style-system/src/property/apply_advanced.rs` +14/−8＝桥域外，已核实）；contain-intrinsic-size 未涉桥测断言面＝条件双路径 resolved＝不动形。

## 哨位（analyze2.out，SENTINELS_DONE）

dialog band r62＝2P 双腿（flaky 键持位第 17 轮）；select-event r67＝279 keys 8F Fset ≡ se.577；password r67＝225 keys 0F；focus-dynamic r108＝121 all P；onblur r60＝**Fset 双 IDENT ≡ob.s559**（10 键 8P/2F 总面传递性；稳位第 28 轮）；族带 r61＝band8 hits=8×2 全 Fail（聚合形持位第 45 连，页级 T 零再现）；双发伪影 ×2 恒；爆栈 so1/so2/sob=0/0/0。轮次标签全对齐（r62/r67/r67/r90/r149/r108/r60/r61＝与 s628 布防计划算术一致）。

**三哨位空转＝预装缺档（本轮特记）**：s477-mpreload.keys／s477-waapi.keys／s477-onblur.keys 三参考档未入预装集→mpreload r90／waapi r149／onblur r60 的 P/F 显示面空转（'No such' ×7）。**传递性兜证**：per-leg canon 与 s627 逐字节相等 ⇒ mpreload r90＝15/15P 双腿（回休后持位第 1 轮兑现＝间接证）、waapi r149＝picker P/checkmark F 双腿、onblur r60＝10 键 8P/2F 双腿必然成立；onblur Fset 判等（用 ob.s559.keys）本轮直接验证 IDENT 双腿。

## 审计缺段（布防预装两缺＝s629 清单修复项）

1. **analyze.sh 基线指针未换**：骨架 s626.stable.keys/s626.stable.canon 未换为 s627.*（亦未随基线换装改名入预装集）→ NEW/GONE/FLIPS 行伪影（NEW=8517）。零位移定谳不受影响（md5 同值＋byte-diff 双证）；bridge-extract.sh 的 b626→b627 换针正确（同批换装不同命）。
2. **三参考 keys 缺档**（如上）＝'No such' 归零审计第 23 演非归零（3 档缺＋1 指针缺＝预装清单审计两缺）。修复：s629 预装清单补全三档＋analyze.sh comm 基线直接写 s628.stable.keys/canon。

## 窗底账（s628 窗＝63e0cd0f1→legs.head 0f184e0bf＝6 commit 全列）

- **码 ×1**：2aa1c9288〔对端 style-system 刀＝KC-1 contain-intrinsic-size 分轴 none 混形受理＝apply_advanced.rs〕。
- docs ×5＝本流 1〔0f184e0bf s627 收账〕＋对端 4〔49fb9f302 wptfyi-html 批十五收账／8f03a3c02 wptfyi-css KC-1 收账／cde99ad26 wptfyi-html 批十五补档＝goal 脚本面非 runner 码／68876b5b8 R5906 部署窗追记〕。data ×0。
- 布防补记"已见 4 commit＋收账落窗"差 1（49fb9f302）＝补记兜底条款按设计兜住。

## 基线换装

- join：s628.stable **8517**（f1aef7b7/057764d3＝链复挂第 26 连基线；零位移持位观察续）。
- 全桥：b628 keys **1736**（bb075eee 同值持位第 2 轮；fail=2 双挂账均归对端刀面续）。
- runner：**db62802c**/129,701,624 新锚（候选四十五 0f4dc6a1 时代终结；候选四十七兑现入账）。
