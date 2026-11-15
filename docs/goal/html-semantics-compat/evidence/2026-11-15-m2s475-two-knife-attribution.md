# M2 s475 — 双刀归因轮：编辑流 04369d39a ip 容器链垫刀 + d1625c26a d34 headEl 持真 title 落树，html-semantics 面零位移清偿，P 集 8306 回位兑现 + 全桥绿窗 1644P/0F（+1 pad 钉测）

日期：2026-11-15。承前：s474 下一步「守成巡查（他流入树归因轮
或轻覆盖）」。本轮开工 `git pull --rebase` 远端两刀落树：s474
腿树（f61c4a582）之后 04369d39a（fix(editing) ip 容器链垫刀，
`_zwJusIsExtraneousLineBreak` 块级父独子臂，part121 钉测五形）+
d1625c26a（fix(engine) d34 子文档 headEl 持真 title，part03 +22
/part05 +45，delete-in-child-of-html 净 +10）——判双刀归因轮
（s466 先例流程）。半径全在编辑/iframe 子文档面，html-semantics
面预期零位移，join 实测为准。

## 双腿 join（d1625c26a 双刀组合态树，make testharness-html-semantics ×2）

- leg1 **8516 行（8306P/204F/5T/1U）**，leg2 **8513 行（8306P/
  201F/5T/1U）**；distinct 各 8514/8511（双腿各含 2 键同行重复
  Pass 噪声行：wrapping-transformation exact len + 149.html
  `for=' WINdow`，双腿同形，非分歧）；**leg-only 5 全在
  select-home-end 族**（leg1-only 4 子测 Home/End/PageDown/
  PageUp + leg2-only 族整文件键），**disagree = 0**。
- **stable 8512（8306P/200F/5T/1U，norm 配方口径，distinct
  8510）**。
- **vs s474 stable 恰 5 GONE（族 4 子测 F + 族 detailed 整文件
  T）+ 2 NEW（onblur following-label checkbox P 哨域键回位 +
  族 detailed 整文件 F）**；per-key 1 翻转 = detailed T→F（族
  键相位回摆，s473 同形），余共享键零翻转，非族非哨域键逐键零
  差。族形态 = detailed F 双腿一致 + 4 子测 leg2 缺席（族相位谱
  内振荡，单轮观察判定维持）。
- **P 集账本口径 8305 → 8306 回位兑现**（s474 哨域键出 stable
  的单轮破位按预期回位）。
- 刀邻面抽查：document-metadata/the-title-element 243 键双腿恒
  等全 Pass，headEl/title 半径邻接面无位移。

## 观察哨复核（三轮全零翻红）

- **onblur 升级观察哨第 1 轮**（s474 立）：following-label 双键
  （Checkboxes + Radios）**双腿 Pass 回入 stable**，无再现翻红
  ——继续盯防（2 轮内再现或转双腿一致 F 才升级真回归排查）；
  红池 parent-label 双键照常 F（onblur 哨组 stable 10P + 红池
  2F 在册）。
- dialog-overlay 观察哨第 5 轮复核无再翻红（哨键双腿 Pass 在
  stable）；modulepreload 观察哨第 7 轮无再翻红（15 键双腿全
  Pass，s450 起累计 50/51 腿）。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1644P/0F rc=0**（test 阶段 18.75s；基线 s474 1643P，**+1 =
04369d39a 携入钉测 `test_ip_c4_container_pad_pins`**，d1625c26a
零新钉；filtered 2389 同值）。

## 结论

s475 收口（双刀归因轮）：编辑流两刀（04369d39a + d1625c26a）
落树 **html-semantics 面零位移清偿**，P 集 8306 回位兑现，基带
形态 **8306P/200~204F/5T/1U**，族相位谱内振荡，三观察哨零翻红。
挂用户门维持：GB-20261114-VIDEO + GB-20261031（簇 D）+ s137
（s445 已提示，不重复通知）。hang 观察项静默哨维持（本轮双腿
零 hang）。

工件：/tmp/s475/{s475-leg1,s475-leg2}.log、s475-{l1,l2}.norm、
l{1,2}.ks、stable.norm、s475-stable.norm、st47{4,5}.u、
gone/new.lines、s475-bridge.log、s475-bridge.rc、
bridge47{4,5}.names。
