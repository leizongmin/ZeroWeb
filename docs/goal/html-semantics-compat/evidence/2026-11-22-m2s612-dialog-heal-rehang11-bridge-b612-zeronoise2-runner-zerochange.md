# s612 收账证据——dialog 愈链复挂第 11 连＋b612 零噪声第 2 连＋runner 零换代（演 1 作废／演 2 重臂）

日期：2026-11-22（演 1 窗开 2026-11-21 23:51:43；演 2 全程 2026-11-22 00:03–00:17）
环境：本机双 clone 并行（对端 wptfyi-css 流）＋mykernel 外来验证负载；`/tmp/s612`；warehouse 归档 `/tmp/s612-abort1/`（演 1 隔离件 19 个）。

## 演 1（作废，补账九/十已记）

- 23:31 三臂起动（waiter＋post＋rearm v3.3）；23:51:43 净窗判净（procs=0 load<3 3×45s），legs.head 钉扎 `791ad19f1`。
- leg1 23:51:43→23:55:17（3m34s）rc=2＝完整有效腿（9026 行＝8305P/206F；rc=2 与 s611 双腿同形＝失败数退出非截断）；runner pre `8c7c016d`/129,506,936 → mid `9aa0fcc3d`/129,507,000（+64B，Compiling ×6）＝**候选三十一换代兑现证据（跨演保留）**。
- leg2 23:55:16 编译窗（Compiling ×8：layout-engine→engine→page-runtime→webview→wpt-runner 等）与本流 push rebase 树换（23:55:54 checkout 至 6e668da5a、~23:57 continue）完全重叠＝混代；桥测将自 G1a 树构建＝b612/双腿同代判据全污染 → 23:59 停链（legs.done 未写、桥测 rc 未落盘），19 件隔离 `/tmp/s612-abort1/`。
- 同窗碰头信号：对端 `bbb940734`（G1a 码刀，parent=791ad19f1）携 stale 工作树副本回写本流控制面（header/账本/补账七/八 退 29b449a15 代＋s611 evidence 文件删 −194；对端零新增内容 grep 核证），补账九 rebase 解中原样恢复（`f57f8297c` 携 +411/−136 含 evidence 重建）。

## 演 2（有效收账轮）

- 布防：00:01 预构建一致代 runner `1debf71f3`/129,510,360（50.87s 仅 runner 本体重链＝deps 指纹 fresh＝混代疑虑解除）；00:03:32 三臂重起（双绝对路径，ps 验证）。
- 窗：00:08:17 clean-soaked（procs=0 load<3 3×45s，mykernel 重载退坡后），legs.head 钉扎 `a8de9c124`；候窗 ~4m45s。
- leg1 00:08:18→00:10:59（2m41s）rc=2，Compiling ×6（冗余重建，字节同值）；leg2 00:10:59→00:13:32（2m33s）rc=2 零编译；双腿 log 9031/9021 行＝8308P/206F 同形。
- runner：pre=mid=post `1debf71f3`/129,510,360（零换代形）。
- 桥测：rcb=0，40.89s，`1738 passed; 0 failed`；s612.bridge.keys md5 `fa1ff46b37…` ≡ b611.keys 字节同；new/gone/fail keys 全空。
- solo1 00:15:33→00:16:01（28s）／solo2 00:16:01→00:16:22（21s）均 rc=2；failset（`^Fail` 形）md5 `329e35ed…` 双 solo IDENT，与 s611 solo failset 逐字节同＝发射形同值持位。
- all.done 00:16:37；窗起至全毕 ~8m20s 零空窗；两臂齐全第 11 演。

## join 判定（analyze.out）

- `stable=8517 disagree=0 l1only=0 l2only=0`；8519 行 norm ×2＋canon ×2；集级 206 Fail/8305 Pass/5 Timeout/1 Unsupported。
- **NEW=1**＝`html/semantics/interactive-elements/the-dialog-element/dialog-overlay.html :: dialog computed overlay initially 'none'`（s611 翻出的 flaky 键双腿复归 Pass＝恰键愈链）；GONE=0 FLIPS=0。
- s612.stable.canon md5 **`f1aef7b7d7eafeeef3dd0989ba6bc3d9`**（≡s610 canon＝链复挂第 11 连实质恢复）；s612.stable.keys md5 `057764d38ee311a47c34ecfc3e1e3192`（同 s610 keys＝完整回归 s597–s610 稳态）。
- 双腿 norm md5 双双 `c735d37188a0fa0ea338b47d2432a26f`（leg1 整腿复现第 16 连 s597→s612；leg2 自 s611 `47da33bd` 分叉复归＝双腿恒恢复）。

## b612 判定

- keys=1738 fail=0 NEW=0 GONE=0＝**零噪声第 2 连兑现**；补账十「G1a 归因非零噪声」预判证伪（安全向）：G1a `computed_style.rs` +11 为 impl 面，未增删 js_dom_bridge 测试键。

## 哨位（analyze2.out，SENTINELS_DONE）

dialog band r46＝2P 双腿（flaky 键复归观察兑现）；select-event r51a＝279 keys 8F Fset ≡ se.577；password r51b＝225 keys 0F；mpreload r74＝15/15P 双腿（持位第 11 轮）；waapi r133＝picker P/checkmark F 双腿；focus-dynamic r92＝121 all P；onblur r44＝8P/2F Fset 双 IDENT（稳位第 12 轮）；族带 r45＝band8 hits=8×2（聚合形持位第 29 连，页级 T 零再现）；双发伪影 ×2 恒；爆栈 so1/so2/sob=0/0/0；'No such' 归零审计 analyze.out/analyze2.out 0/0 第 7 演。

## 窗底账（s612 窗＝29b449a15→a8de9c124＝14 commit 全列）

- 码 ×2：`4440d8f94` CLIENTWIDTH-PADBOX v3（js_dom_shim/part04.js；桥/semantics 触点=0）；`bbb940734` G1a interpolate-size＋gCS 读回（engine/js_dom_bridge/computed_style.rs +11＝桥触点 ×1＋style-system ×3；桥测零新钉）。
- docs ×12：对端 7（a0ea8c1db／da12bf0f8／69966c309／4d859a238／56c1b698e／349669437／6e668da5a）＋本流 5（b804834a2／5817ffb76／791ad19f1／f57f8297c／a8de9c124）。
- 演 1 窗（作废）＝29b449a15→791ad19f1＝9 commit（补账八/九全列核实）。

## 基线换装

- join：s612.stable **8517**（f1aef7b7/057764d3）。
- 全桥：b612 keys **1738**（fa1ff46b37，≡b611 字节同）。
- runner：**1debf71f3**/129,510,360 在锚。

## 教训条目（窗底账完整性系列增补）

**legs 运行期本仓工作树钉扎**：窗开 legs 在飞期间禁止本仓 commit/push/rebase/checkout 一切树变更（leg2 混代即树换撞编译窗实录）；补账 push 一律等 all.done 之后。等窗期（legs 未起）pull/commit/push 照常。
