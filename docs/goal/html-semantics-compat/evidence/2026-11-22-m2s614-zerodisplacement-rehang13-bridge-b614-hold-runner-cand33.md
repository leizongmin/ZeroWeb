# s614 收账证据——完全零位移链复挂第 13 连＋b614 keys 1737 持位（对端未修）＋runner 候选三十三兑现＋限流断点零损续跑

日期：2026-11-22（候窗 02:59–03:27:35；窗开 03:27:35；全毕 03:35:00）
环境：本机双 clone 并行（对端 wptfyi-css 流）；`/tmp/s614`。

## 限流断点与自动续跑（本轮特记，s605 无损补记模式）

- 前一 session 02:58 完成本流 s613 收账提交（f96dd17ee）＋s614 布防（02:59 预装载全套、03:00 三臂起动），期间 pull ×2（03:00→3fba1d492 对端 docs；03:20→b1fc0f8ea weekly cron data 面），03:27:31 API 限流断点（5h 配额尽）。
- 断点 4s 后（03:27:35）臂内 waiter 自行开窗起腿，session 死亡不影响后台三臂：legs→桥→哨位→solo→all.done 03:35:00 全链自动跑毕＝两臂齐全设计无人值守韧性首证；收账由续 session 本轮补记。
- 窗开 03:27:35 clean-soaked（procs=0 load<3 3×45s），工作树已在 `b1fc0f8ea`（含三码刀＋cron data 面）＝legs.head 钉扎。

## 窗与 legs

- leg1 03:27:35→03:30:43（3m08s）rc=2＝完整有效腿，Compiling ×9（三码刀驱动）；leg2 03:30:43→03:32:52（2m09s）rc=2 零编译。
- runner：pre `5ebba762`/129,574,608（候选三十二在锚）→ mid=post `18f1d1f5`/129,583,048（+8,440B）＝**候选三十三条件挂号转确定形**（窗码面 runner 依赖域直源：engine js_dom_shim part10-typed-om.js +129（OM-18）／layout-engine calc_size.rs +67（calc-size-flex-008 回绿）／css-parser＋style-system 零行为 clippy 清偿）。
- 桥测（run-legs 内，同 s613 recipe）：`./target/test-guard --compile-first --per-proc-mem 8 --total-mem 16 --time-limit 1200 -- cargo test -p zero-engine --lib js_dom_bridge`，24.81s，rcb=**101**。
- solo1 03:34:24→03:34:44（20s）／solo2 03:34:44→03:35:00（16s）均 rc=2；failset（`^Fail` 原始序）md5 `329e35ed7bd8b6e00f921f25671c57dd` 双 solo IDENT ≡s611–s613＝发射形同值第 7 连。
- legs.done 03:34；all.done 03:35:00；窗起至全毕 **7m25s** 零空窗（两臂齐全第 13 演，rearm 零触发）。

## join 判定（analyze.out）

- `stable=8517 disagree=0 l1only=0 l2only=0`；8519 行 norm ×2＋canon ×2；集级 206 Fail/8305 Pass/5 Timeout/1 Unsupported（同 s613）。
- **NEW=0 GONE=0 FLIPS=0**＝完全零位移主预判精确兑现（三码刀入窗零位移，含 engine OM-18 shim 层与 layout calc-size 修复＝对端自报零回归本流 corpus 独立核实成功）；rc1=2 rc2=2＝失败数退出非截断。
- s614.stable.canon md5 **`f1aef7b7d7eafeeef3dd0989ba6bc3d9`**（≡s610=s612=s613＝链复挂第 13 连）；keys md5 `057764d38ee311a47c34ecfc3e1e3192` 同值持位。
- 双腿 norm md5 双双 `c735d37188a0fa0ea338b47d2432a26f`（leg1 整腿复现第 18 连 s597→s614；双腿恒恢复第 4 演）。

## b614 判定（bridge.out）＝条件双路径"不动"分支兑现

- keys=**1737** fail=1 NEW=0 GONE=0；s614.bridge.keys md5 `4e20b5df86bb5d08902d235c2e01e6ce`（≡b613＝同值持位）。
- FAIL 同键持位＝`js_dom_bridge::computed_style::computed_calc_serialize_tests::min_max_single_arg_collapses_to_argument_terms`＝对端未修 v3 刀桥测期望（挂账续，飞书已通报）；桥红待办仍归对端 v3 刀作者面（规则 9/10，本流不代修）。
- 窗内 OM-18 触点核实：b99fafd33 仅 js_dom_shim part10-typed-om.js JS 层（+129）＝零 rust 桥触点（同 OM-17 形），桥测零新增噪声兑现。

## 哨位（analyze2.out，SENTINELS_DONE）

dialog band r48＝2P 双腿（flaky 键持位第 3 轮，GONE 形未现）；select-event r53＝279 keys 8F Fset ≡ se.577；password r53＝225 keys 0F；mpreload r76＝15/15P 双腿（持位第 13 轮）；waapi r135＝picker P/checkmark F 双腿；focus-dynamic r94＝121 all P；onblur r46＝8P/2F Fset 双 IDENT ≡ob.s559（稳位第 14 轮）；族带 r47＝band8 hits=8×2（聚合形持位第 31 连，页级 T 零再现）；双发伪影 ×2 恒；爆栈 so1/so2/sob=0/0/0；'No such' 归零审计 analyze.out/analyze2.out 0/0 第 9 演。轮次标签订正已落地（dialog r48／onblur r46／band r47 等与计划算术一致，s613 滞后一位遗留清除）。

## 窗底账（s614 窗＝3e882ea74→b1fc0f8ea＝8 commit 全列）

- 码 ×3：`b99fafd33` OM-18（engine js_dom_shim part10-typed-om.js +129＝typed-om reify 路由/规整族 19 靶，JS shim 层零桥触点）；`ee61d2ebb` calc-size 列向 wrap flex 容器主尺寸槽冻结单行（layout-engine calc_size.rs +67，calc-size-flex-008 回绿）；`54a2c9380` calc-size 面 clippy 债 8 处清偿（css-parser／layout-engine／style-system 四文件，零行为）。
- data ×1：`b1fc0f8ea` weekly cron wpt.fyi 对账快照（docs/compat/wptfyi/ coverage json+md，非语料非码）。
- docs ×4：对端 3（`ae775314f` TEXT-INTERP 域拆共存发射／`247d1b0ff` rendering-compat R5890 收账／`3fba1d492` CSINTERP-S2A r3 判定）＋本流 1（`f96dd17ee` s613 收账＋s614 布防）。

## 基线换装

- join：s614.stable **8517**（f1aef7b7/057764d3，同 s610=s612=s613）。
- 全桥：b614 keys **1737**（4e20b5df 同值持位；GONE 1 挂账续＝对端面待修）。
- runner：**18f1d1f5**/129,583,048 在锚（候选三十三兑现换代）。
