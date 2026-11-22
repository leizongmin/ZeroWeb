# s625 收账证据——完全零位移链复挂第 24 连（docs/网站面窗平凡形＝窗底账 2 commit 全 docs）＋b625 keys 1737 持位（"不动"分支）＋runner a58c919e 三点恒持位＋钉扎纪律 gate 化后首演零违规＋跨 session 限流断点无损续链

日期：2026-11-22（布防 11:20:59；候窗 ~7m15s；窗开 11:28:24；全毕 11:33:11）
环境：本机双 clone 并行；`/tmp/s625`；上 session 布防后 429 限流断点（s624 收账 staged 未提交即断），本 session 按 s605 无损模式续链：staged 收账原样提交（a99cadee8）＋在飞轮 /tmp/s625 结果照收＝断点不阻链再证。

## 窗与 legs

- 布防 11:20:59 三臂起动（waiter 直臂＋rearm 监督者＋post 分开臂；rearm.s625.log 不存在＝零真重臂；waiter.s625.log 0 字节静默形）；候窗 ~7m15s（11:21:09 起臂→11:28:24 clean-soaked procs=0 load<3 across 3×45s checks），工作树钉扎 `0bdc97ab1`（本流 s623 收账提交）＝legs.head。
- **钉扎纪律 gate 化后首演零违规**：窗开→all.done 全链本流 git 操作零落地（s624 教训条目〔窗态检查独立 gate＋窗开挂起 commit/pull/push〕生效首证）；leg1/leg2 **Compiling ×0 双零**＝HEAD 未动干净形（release build mtime 沿 s624 leg2 重编后全新形）。
- leg1 11:28:25→11:30:10（1m45s）rc=2＝完整有效腿；leg2 11:30:10→11:32:10（2m）rc=2＝失败数退出非截断。
- 结果级零扰旁证：join 双腿 norm md5 恒 c735d371＋canon f1aef7b7＋bridge keys md5 4e20b5df 全链持位。
- runner：pre=mid=post **`a58c919e`**/129,827,000 三点恒＝**a58c919e 持位观察兑现**（候选三十八在锚持位；候选四十三挂号未兑现＝窗码面 runner 依赖域直源 ×0）。
- 桥测（run-legs 内，同 s624 recipe）：`./target/test-guard --compile-first --per-proc-mem 8 --total-mem 16 --time-limit 1200 -- cargo test -p zero-engine --lib js_dom_bridge`，build 0.29s（test profile 全新零编译＝s624 桥测刚建形）+test 22.60s，rcb=**101**。
- solo1 11:32:50→11:33:00（10s）／solo2 11:33:00→11:33:11（11s）均 rc=2；failset（`^Fail` 原始序）md5 `329e35ed7bd8b6e00f921f25671c57dd` 双 solo IDENT ≡s611–s625＝发射形同值第 18 连。
- all.done 11:33:11；窗起至全毕 **4m47s** 零空窗（两臂齐全第 24 演）。

## join 判定（analyze.out）＝零位移

- `stable=8517 disagree=0 l1only=0 l2only=0`；8519 行 norm ×2＋canon ×2；集级 206 Fail/8305 Pass/5 Timeout/1 Unsupported（同 s624）。
- **NEW=0 GONE=0 FLIPS=0**＝完全零位移主预判精确兑现；窗平凡形（窗底账 2 commit 全 docs＝码 ×0）。
- s625.stable.canon md5 **`f1aef7b7d7eafeeef3dd0989ba6bc3d9`**（≡s610=s612=…=s624＝链复挂第 24 连）；keys md5 `057764d38ee311a47c34ecfc3e1e3192` 同值持位。
- 双腿 norm md5 双双 `c735d37188a0fa0ea338b47d2432a26f`（leg1 整腿复现第 29 连 s597→s625；双腿恒第 15 演）。

## b625 判定（bridge.out）＝条件双路径"不动"分支兑现

- keys=**1737** fail=1 NEW=0 GONE=0；s625.bridge.keys md5 `4e20b5df86bb5d08902d235c2e01e6ce`（≡b614=…=b624＝同值持位）。
- FAIL 同键持位＝`js_dom_bridge::computed_style::computed_calc_serialize_tests::min_max_single_arg_collapses_to_argument_terms`＝对端未修 v3 刀桥测期望（挂账续）；桥红待办仍归对端 v3 刀作者面（规则 9/10，本流不代修）。
- 窗桥触点核实：窗底账 2 commit 全 docs（goal 控制面文件）＝js_dom_bridge 触点 ×0＝"不动"分支主预判兑现。

## 哨位（analyze2.out，SENTINELS_DONE）

dialog band r59＝2P 双腿（flaky 键持位第 14 轮，GONE 形未现）；select-event r64＝279 keys 8F Fset ≡ se.577；password r64＝225 keys 0F；mpreload r87＝15/15P 双腿（持位第 24 轮）；waapi r146＝picker P/checkmark F 双腿；focus-dynamic r105＝121 all P；onblur r57＝8P/2F Fset 双 IDENT ≡ob.s559（稳位第 25 轮）；族带 r58＝band8 hits=8×2（聚合形持位第 42 连，页级 T 零再现）；双发伪影 ×2 恒；爆栈 so1/so2/sob=0/0/0；'No such' 归零审计 analyze.out/analyze2.out 0/0 第 20 演。轮次标签全对齐（dialog r59／select-event r64／password r64／mpreload r87／waapi r146／focus-dynamic r105／onblur r57／band r58＝与 s625 布防计划算术一致）。

## 窗底账（s625 窗＝8b22b0aa4→legs.head 0bdc97ab1＝2 commit 全列）

- 码 ×0（CODEFACE 过滤零命中）；data ×0；docs ×2＝本流 1〔0bdc97ab1 s623 收账＋s624 布防〕＋对端 1〔f5ae04d39 R5903 收账携 96代 a58c919e 持位三证二连第七连＝对端引本流 s622 独立互证〕＝s624 布防"归 s625 窗面"预判精确兑现。
- 本流 s624 收账 commit（a99cadee8，续 session 补账）与对端 94c51cef1（wptfyi-css FAM-F）落窗后＝归 s626 窗面。

## 跨 session 限流断点实录（s605 模式再证）

上 session 于 s624 收账 staged 后 429 限流终断（收账 commit 未落）；其间 s625 三臂已布防并自动跑毕全链（布防 11:20:59→all.done 11:33:11，零人工干预）。本 session 开工 pull --rebase 捕获对端 94c51cef1 后：staged s624 收账原样提交（a99cadee8）→ /tmp/s625 全套产物收账 → s626 布防。断点不阻链、结果零丢失＝s605 无损补记模式跨 session 形再证（与 s614/s615/s618 同族，新增"布防先于断点、在飞轮自足跑毕"子形）。

## 基线换装

- join：s625.stable **8517**（f1aef7b7/057764d3，同 s610=s612=…=s624）。
- 全桥：b625 keys **1737**（4e20b5df 同值持位；GONE 1 挂账续＝对端面待修）。
- runner：**a58c919e**/129,827,000 在锚（候选三十八持位）。
