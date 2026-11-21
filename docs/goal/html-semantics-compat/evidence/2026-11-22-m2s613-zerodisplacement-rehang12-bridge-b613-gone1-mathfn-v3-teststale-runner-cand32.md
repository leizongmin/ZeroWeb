# s613 收账证据——完全零位移链复挂第 12 连＋b613 GONE 1（对端 v3 刀桥测期望 stale）＋runner 候选三十二兑现

日期：2026-11-22（候窗 00:26–02:39:44；窗开 02:39:44；全毕 02:46:50）
环境：本机双 clone 并行（对端 wptfyi-css 流）＋外部 contention（DC3 语法套件 1h27m／对端 CSINTERP r2b legs／ZeroSeed 验收链多轮／mykernel 构建多轮）；`/tmp/s613`。

## 布防与等窗

- 00:26 三臂起动（waiter＋post＋rearm v3.3，双绝对路径，ps 三进程验证＝两臂齐全第 12 演）；waiter 臂 1 内开窗，rearm 零触发。
- 候窗 2h13m44s＝外部 contention 门拒窗非自锁照规长等（s598 1h53m50s／s611 ~3h49m 先例续）；load 振荡 12.2→1.7→8.3→4.4→1.8；02:27 与 02:14 两次干净缺口未对齐 3×45s 连检（ZeroSeed strict-docs/synced 链恰启）。
- 等窗期内碰头第 3 次（2dfeb607e stale 删本流 01:02 补记 17 行）原样恢复（67fcf5530＋补账十二，飞书第 3 次通报）；恢复后至窗开 pull 均零碰头（对端 docs 刀 ×4＋GRID 刀＋批十均自有域）。

## 窗与 legs

- 窗：02:39:44 clean-soaked（procs=0 load<3 3×45s），legs.head 钉扎 `3e882ea74`（run-legs 起腿前自行 pull 至最新＝含三码刀）。
- leg1 02:39:44→02:42:49（3m05s）rc=2＝完整有效腿，Compiling ×9（三码刀驱动）；leg2 02:42:49→02:45:11（2m22s）rc=2 零编译。
- runner：pre `1debf71f3`/129,510,360 → mid=post `5ebba762`/129,574,608＝**候选三十二精确兑现**（窗码面 runner 依赖域直源：css-parser types.rs +35／engine js_dom_bridge/computed_style.rs +26／js_dom_shim part03+part10）。
- 桥测（run-legs 内）：`./target/test-guard --compile-first --per-proc-mem 8 --total-mem 16 --time-limit 1200 -- cargo test -p zero-engine --lib js_dom_bridge`，21.11s，rcb=**101**。
- solo1 02:46:28→02:46:39（11s）／solo2 02:46:39→02:46:50（11s）均 rc=2；failset（`^Fail` 原始序）md5 `329e35ed…` 双 solo IDENT ≡s611/s612＝发射形同值第 6 连。
- legs.done 02:46:20；all.done 02:46:50；窗起至全毕 **7m06s** 零空窗（vs s612 8m20s）。

## join 判定（analyze.out）

- `stable=8517 disagree=0 l1only=0 l2only=0`；8519 行 norm ×2＋canon ×2；集级 206 Fail/8305 Pass/5 Timeout/1 Unsupported（同 s612）。
- **NEW=0 GONE=0 FLIPS=0**＝完全零位移主预判精确兑现（三码刀入窗零位移＝对端自报零回归本流 corpus 独立核实成功）；rc1=2 rc2=2＝失败数退出非截断。
- s613.stable.canon md5 **`f1aef7b7d7eafeeef3dd0989ba6bc3d9`**（≡s610=s612＝链复挂第 12 连）；keys md5 `057764d38ee311a47c34ecfc3e1e3192` 同值持位。
- 双腿 norm md5 双双 `c735d37188a0fa0ea338b47d2432a26f`（leg1 整腿复现第 17 连 s597→s613；双腿恒恢复第 3 演）。

## b613 判定（bridge.out）＝零噪声第 3 连预判证伪（GONE 1 形）

- keys=**1737**（b612=1738）fail=1 NEW=0 GONE=1；s613.bridge.keys md5 `4e20b5df86bb5d08902d235c2e01e6ce`（≠b612 `fa1ff46b37…`）。
- GONE/FAIL 同键＝`js_dom_bridge::computed_style::computed_calc_serialize_tests::min_max_single_arg_collapses_to_argument_terms`；断言 computed_style.rs:6013 `left: "min(1.5px, 10%)" right: ""`＝测试期望旧多参含 % fail-closed 空串行为。
- **归因定谳＝65efb172d MATHFN-LENPERC v3**：①′ 臂（computed_style.rs +26）蓄意把多参 min/max 含 % 从 fail-closed `''` 改为形保留（css-values-4 §10.8，旧空串即其刀注明的 gCS 空串根因；corpus serialize 页 +17 GATE 兑现），但本桥测单测期望未同步＝**测试 stale（刀作者面待修），非运行时回归**（本流 corpus join 全零位移佐证）。
- 触点核实：65efb172d computed_style.rs +26（主嫌定谳）；0096ff43d GRID js_dom_bridge.rs +23（supports 前置臂，calc serialize 域外＝排除向）；6dae130a2 OM-17 仅 js_dom_shim part03/part10 JS 层＝零桥触点排除。
- 处置＝规则 9/10：对端面记录归因＋飞书通报，本流不代修；b614 条件双路径预判见 header 下一步。

## 哨位（analyze2.out，SENTINELS_DONE）

dialog band r47＝2P 双腿（flaky 键持位观察兑现，GONE 形未现）；select-event r52＝279 keys 8F Fset ≡ se.577；password r52＝225 keys 0F；mpreload r75＝15/15P 双腿（持位第 12 轮）；waapi r134＝picker P/checkmark F 双腿；focus-dynamic r93＝121 all P；onblur r45＝8P/2F Fset 双 IDENT ≡ob.s559（稳位第 13 轮）；族带 r46＝band8 hits=8×2（聚合形持位第 30 连，页级 T 零再现）；双发伪影 ×2 恒；爆栈 so1/so2/sob=0/0/0；'No such' 归零审计 analyze.out/analyze2.out 0/0 第 8 演。脚本轮次标签滞后一位（analyze2.sh 内静态标签 r46 实为 47）＝记账按计划算术，s614 重臂时订正。

## 窗底账（s613 窗＝a8de9c124→3e882ea74＝35 commit 全列）

- 码 ×7：`228abd8b9` 批九 infra 通道（Makefile＋testharness_html_infra.rs ＋main.rs）；`a62d221df` 批十一准备（testharness.rs＋infra +319）；`f76250c81` 批九还原（还原 ee2ad7855 stale 误删）；`6dae130a2` OM-17（js_dom_shim part03/part10）；`65efb172d` MATHFN v3（css-parser types.rs +35＋js_dom_bridge/computed_style.rs +26＝桥触点 ×1）；`0096ff43d` GRID（js_dom_bridge.rs +23＝桥触点 ×1）；`0a070adee` 批十（pipe=sub {{GET}} 注入）。
- ci ×1：`4befb90f3`（ci-shards.yml ＋DC-3 周跑 docs）。
- docs ×27：对端 22（ee2ad7855／7e71f6b2e／380991026／854752f2d／992042954／01f4ef961／f3987fadf／2dfeb607e／5db919f29／43a310de2／3441cb7bd／871215a29／594fc9d99／2e73d4b01／10b839291／a555c3094／4e035b6f5／3eb35c7dd／123ef613b／39f56fcba／75c20a67c／3e882ea74）＋本流 5（3f239a910 s612 收账／8fc73d81d 补账十一／b8c622fbd 订正／dc4ec4b42 窗码面补记／67fcf5530 补账十二）。
- 窗内批九 trio（228abd8b9 添加→ee2ad7855 stale 误删→f76250c81 还原）净效果＝批九交付面在树。

## 基线换装

- join：s613.stable **8517**（f1aef7b7/057764d3，同 s610=s612）。
- 全桥：b613 keys **1737**（4e20b5df；GONE 1 挂账＝对端面待修）。
- runner：**5ebba762**/129,574,608 在锚（候选三十二兑现换代）。

## 教训条目

**码刀 GATE 覆盖面缺口（对端面）**：MATHFN-LENPERC v3 蓄意行为变更只跑 corpus GATE 未跑 js_dom_bridge 单测，桥测期望 stale 上 main（rcb=101 实录）；行为变更入库应同步受影响单测期望或至少跑受影响 crate 的 lib test。本流巡查桥测（条件触发型）充当了该缺口的检测网。
