# M2 s485 — 他流刀入树归因轮：site-opt 流 PR #305 iframe navigator 平面对象语义刀（T72/P42，bb194436b 主刀 + ff81a8d92 工厂读取加卫降级返修）html-semantics 面零位移清偿（非族 8509 键逐键零差 + flips 0，GONE 4 全族相位伪影），P 集 8307 持位 + picker 增益键第 8 轮保持 + 五观察哨零新翻红 + 全桥绿窗 1648P/0F（T72 钉 4 测试合法入桥清单 1644→1648 纯增长零破）

日期：2026-11-15。承前：s484 下一步「守成巡查（他流入树归
因轮或轻覆盖）」。开工 `git pull --rebase` 携入 **a520bc2f9
Merge PR #305 site-opt/t72-g1-iframe-navigator**（19:27:47）：
- bb194436b 主刀 fix(engine) iframe navigator 平面对象语义
  （T72/P42）；ff81a8d92 返修工厂读取加卫降级（T72/R1 DF-1）。
- 落树面：shim part05.js +26/−1（`_zwMakeIframeWin`）+
  part119_t72_iframe_navigator_pins.rs +219（新钉文件）+
  js_dom_bridge_tests.rs +4。半径申报 iframe navigator
  平面对象域（shim 工厂），html-semantics 面预期零位移（与
  s479/s481 双刀「申报=实测」谱系同预期），join 实测为准
  ——**实测兑现预期**。

## 二进制 provenance

- leg1 起跑 make 重建（零 `Compiling` 行但 runner mtime
  19:40:04 晚于刀 19:27:47，size 126886624→126890752 +4KB
  ——「零 Compiling 行但重建」系统性现象持续在册）；leg2 零
  重建同二进制。刀后钉：`strings target/release/zero-wpt-runner
  | grep -c _zwMakeIframeWin` = 6 命中。桥跑同二进制。
- 码态核查：`git diff 688210311..HEAD -- crates/` = 277 行
  （即本刀三文件），HEAD = 7c7037536 docs（码态 a520bc2f9）。

## 双腿 join（7c7037536 树 = 码态 a520bc2f9，make testharness-html-semantics ×2）

- leg1 **8516 行（8307P/202F/6T/1U）**——族子测相位形；leg2
  **8513 行（8307P/198F/7T/1U）**——族混合相位形（optional
  整文件 T + detailed T + 子测缺，7T = 非族 5 + 族 2 整文件
  键同 T）。双腿零截断，零重建。
- join：**stable 8510（8305P/198F/6T/1U），disagree 0，
  leg-only 4/1 全部族键**：l1only = optional 4 子测键（leg1
  子测相位）；l2only = optional 整文件键（leg2 T）；detailed
  整文件键双腿同 T 入 stable。
- **vs s484 stable（8514，族 5 键在册基带）：GONE 4 + NEW 0
  + flips 0**——GONE 4 = optional 4 子测键（双腿相位差不入
  stable，s481 同构族伪影；与刀无半径交集——iframe navigator
  语义与 select-home-end 键盘导航域无交）；**非族 8509 键逐
  键零差 + detailed 族键签名恒等 T**。
- T 键集 6 = 非族 5 三轮恒等 + detailed 整文件 T；**P 集账
  本口径 8307 持位**（双腿双 8307P；stable 机械口径 8305P
  六轮同形）。双发发射伪影 2 键照旧在册（sort -u 去重吸收）。

## 观察哨复核（五组全零新翻红）

- onblur 哨组 stable 10 键全在册 8P/2F（following-label 双键
  Pass 持位，红池 parent-label 2F 照常）——本刀 shim 变更同
  在 part05.js（与 J2 刀同文件），label 域哨组零翻红为刀半
  径不扰编辑/label 域旁证。
- dialog-overlay 哨第 14 轮零翻红（双键 stable Pass）；
  modulepreload 哨第 17 轮零翻红（15 键 stable 全 Pass）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 8 轮保持）/
  checkmark Fail（双相位带形不变）。

## 全桥套件绿窗收口跑（刀后树旁证，清单合法增长）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1648P/0F rc=0**（test 阶段 24.92s；filtered 2389 同值）。
测试清单 vs s484：**1644→1648 纯增长 +4，GONE 0**——新增
4 测试全数 T72 钉（iframe_navigator_plane_object_t72 /
proto_absorb_t72 / proto_degrade_t72 / service_worker_slot_t72
），与刀落树面（js_dom_bridge_tests.rs +4 + part119 钉文件
+219）逐一对号，钉面合法增长零破。

## 结论

s485 收口（他流刀归因轮，本流零源码改动）：site-opt 流
iframe navigator 刀 html-semantics 面零位移清偿——stable 差
集仅族相位键（GONE 4 全族 + NEW 0 + flips 0），非族键逐键
零差，「申报=实测」谱系第三例（s479 d35 / s481 双刀 / 本轮
T72）；P 集 8307 持位，picker 增益键第 8 轮保持，五观察哨
零新翻红，全桥绿窗钉面 1648 零破（T72 钉合法入桥）。挂用户
门维持：GB-20261114-VIDEO + GB-20261031（簇 D）+ s137（s445
已提示，不重复通知）。hang 观察项静默哨维持（双腿零真 hang，
零截断）。

工件：/tmp/s485/{s485-leg1,s485-leg2}.log、s485-{l1,l2}.norm、
l{1,2}.canon、s485.{stable.keys,stable.canon,
leg1only.keys,leg2only.keys}、flips.txt、st485.sig、
b485.list（vs s484 b484.list +4 GONE 0 对账）、
s477-{onblur,dialogov,mpreload,waapi}.keys（哨键组复用）、
s485-bridge.log、norm.sh/canon.sh/join.sh（s478 沉淀管线复
用）。
