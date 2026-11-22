# s626 收账证据——非平凡窗（对端 engine 刀入窗）＝join 链断 24 连现首个 disagree/GONE 形（modulepreload no-referrer 单腿单发翻＝三仪交叉捕捉）＋b626 双红（numeric_special 新挂归对端 FAM-F 刀＋min_max 续挂）＋runner 候选四十四兑现＝新锚 73fb7a44

日期：2026-11-22（布防 11:59:53；候窗 ~1m30s；窗开 12:01:23；全毕 12:07:38）
环境：本机双 clone 并行；`/tmp/s626`；布防前补记 commit（915878a48）先于布防落地＝s624 教训"收账 commit 先于布防"形首演零竞争；钉扎 915878a48＝legs.head。

## 窗与 legs

- 布防 11:59:53 三臂起动（waiter 直臂＋rearm 监督者＋post 分开臂；rearm.s626.log 不存在＝零真重臂）；候窗 ~1m30s（12:01:23 clean-soaked procs=0 load<3 across 3×45s checks＝机器空闲形）。
- **非平凡窗＝布防前补记预判全兑现**：窗码面 8 commit 含对端 engine 码刀 99b30903c（aspect-ratio 插值几何幂律＋applier 落盘＋gCS 规范形 x / 1）＝runner 依赖域直源。
- leg1 12:01:23→12:03:59（2m36s）rc=2，**Compiling ×6**（engine 刀重建＋runner relink 重编形＝布防预判逐字命中）；leg2 12:03:59→12:06:02（2m03s）rc=2，Compiling ×0。
- **runner 候选四十四挂号兑现**：pre=`a58c919e`/129,827,000（stale 盘上二进制＝旧锚）→ mid=post=**`73fb7a44`**/129,690,632（新二进制）＝"必被覆盖"形精确兑现（pre 旧锚→mid/post 新值位移指纹）。
- 桥测（run-legs 内，同 s625 recipe）：build 33.29s（test profile 重编形）+test 20.30s，rcb=**101**。
- solo1 12:07:13→12:07:24（11s）／solo2 12:07:24→12:07:35（11s）均 rc=2；failset（`^Fail` 原始序）md5 `329e35ed7bd8b6e00f921f25671c57dd` 双 solo IDENT ≡s611–s626＝发射形同值第 19 连。
- all.done 12:07:38；窗起至全毕 **6m12s** 零空窗（两臂齐全第 25 演）。

## join 判定（analyze.out）＝链断 24 连＋首个 disagree/GONE 形

- `stable=8516 disagree=1 l1only=0 l2only=0`；8519 行 norm ×2＋canon ×2；集级 206 Fail/8304 Pass/5 Timeout/1 Unsupported（−1 Pass vs s625）。
- **GONE=1 DISAGREE=1（同键）NEW=0 FLIPS=0**＝`html/semantics/scripting-1/the-script-element/module/modulepreload-referrerpolicy.html :: Modulepreload with no-referrer policy should not send referrer`：leg1 **Fail**／leg2 **Pass**（disagree→stable 除名）。
- **归因＝单腿单发不稳定形、非刀效应**：双腿同代（73fb7a44）；leg2 norm md5 双双持历史恒 `c735d37188a0fa0ea338b47d2432a26f`（整腿复现第 30 连 s597→s626＝leg2 侧链续持）＝新二进制对 semantics 语料零位移；leg1 norm `684b139e135fe963ab3bd6cb057e0d4c`＝整腿复现链 leg1 侧断于第 30 连（与恒唯一差异＝该键一行 Pass→Fail）。referrer policy 断言涉真实 fetch 行为＝网络/时序敏感面。
- **三仪交叉捕捉**：join GONE/disagree ＋ mpreload 哨位带 r88 leg1 1F ＋ 集级 8304P −1＝同一事件。
- s626.stable.canon md5 **`8020e850ad87c9dac8819a89b9a8ccfa`**（≠f1aef7b7＝**链复挂断在第 24 连**，新链首环）；keys md5 `163ad56eb9b52a14e39a38d917df75ba`（≠057764d3）。

## b626 判定（bridge.out）＝双红（双挂账均归对端刀面）

- keys=**1736** fail=**2** NEW=0 GONE=1；s626.bridge.keys md5 `bb075eee0e34d743166b19a028e9dde6`（≠b625 4e20b5df＝4e20b5df 同值持位链断在 b614–b625 十二连）。
- **新红＝`js_dom_bridge::tests::test_get_computed_style_numeric_special` Pass→Fail**：panic at `crates/engine/src/js_dom_bridge_tests/part03.rs:1297`（left `"1.77778 / 1"`／right `"1.77778"`）＝对端 99b30903c `aspect_ratio_str` 序列化形改（float → `x / 1` 规范形）未同步 engine 侧桥测期望＝**归对端 FAM-F 刀作者面，本流不代修（规则 9/10）**。
- 旧红持位＝`computed_calc_serialize_tests::min_max_single_arg_collapses_to_argument_terms`（对端 v3 刀期望 stale，挂账续）。
- **窗桥触点逐文件核实（"不动"前提复核）**：99b30903c 直触 `crates/engine/src/js_dom_bridge/computed_style.rs`（+14/−7＝aspect_ratio_str 域限定，无 #[test] 增删＝keys 面稳定机制验证）＋`animation.rs`（+138＝非桥域）＝非"不动"路径确认、keys/fail 位移逐键归因完成。

## 哨位（analyze2.out，SENTINELS_DONE）

dialog band r60＝2P 双腿（flaky 键持位第 15 轮）；select-event r65＝279 keys 8F Fset ≡ se.577；password r65＝225 keys 0F；**mpreload r88＝持位断（leg1 14/15〔no-referrer 键单发 Fail〕／leg2 15/15P）＝与 join 同事件交叉捕捉**；waapi r147＝picker P/checkmark F 双腿；focus-dynamic r106＝121 all P；onblur r58＝8P/2F Fset 双 IDENT ≡ob.s559（稳位第 26 轮）；族带 r59＝band8 hits=8×2（聚合形持位第 43 连，页级 T 零再现）；双发伪影 ×2 恒；爆栈 so1/so2/sob=0/0/0；'No such' 归零审计 0/0 第 21 演。轮次标签全对齐（r60/r65/r65/r88/r147/r106/r58/r59＝与 s626 布防计划算术一致）。

## 窗底账（s626 窗＝0bdc97ab1→legs.head 915878a48＝8 commit 全列）

- **码 ×1**：99b30903c〔对端 engine 刀＝animation.rs＋js_dom_bridge/computed_style.rs〕。
- docs ×7＝本流 3〔a99cadee8 s624 收账／ab8ae1165 s625 收账／915878a48 s626 布防前补记〕＋对端 4〔94c51cef1 wptfyi-css FAM-F 收账／2d7374d3c R5904／817fb2faa wpt.fyi weekly cron／84c6f8f47 learnings〕。
- data ×0。布防前补记"全列 6 commit"与落窗实况差 2（84c6f8f47＋915878a48 本补记自身）＝补记"落窗由 s626 开工捕获核实"条款按设计兜住。
- 窗后对端 2983cdf47〔wptfyi-css CSSOM-INSETS-V2 收账 docs ×7 files〕落窗外＝归 s627 窗面。

## 基线换装

- join：s626.stable **8516**（8020e850/163ad56e＝新链首环；8517 复位条件＝modulepreload no-referrer 键双腿 Pass 回位）。
- 全桥：b626 keys **1736**（bb075eee；fail=2 双挂账＝numeric_special 新挂归对端 FAM-F 刀＋min_max 续挂；销账条件＝对端修 part03.rs:1297 期望后 keys 1737 fail=1 形）。
- runner：**73fb7a44**/129,690,632 新锚（候选三十八 a58c919e 时代终结；候选四十四兑现入账）。
