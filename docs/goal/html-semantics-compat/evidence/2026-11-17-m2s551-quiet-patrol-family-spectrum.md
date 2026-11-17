# M2 s551 — **零码变守成轻覆盖轮 + 族带摆动谱新形轮**：轮首 pull 零新携入，窗内零码面刀（钉扎 HEAD=052748af4＝s550 收口 docs 笔；窗 docs 尾 9404d6d41（R5760）+ 052748af4 均零码面）——码态 ad487ea89→ad487ea89 恒，runner 七代 9a13023968c1 持位（leg1 Compiling ×6 44.68s 确定性重编、md5/size 128,075,392 恒＝零换代，leg2 Fresh 0.22s）——v3.1 浸泡门判净 14:38:42 HEAD=052748af4 钉扎起腿：双腿常态全绿（leg1 3:26 含重编/leg2 2:39，rc=2 惯例 0 爆栈）——join **stable 8513（stable canon 8305P/201F/6T/1U）disagree 0 + l1only 1 + l2only 1**（raw l1 8516＝8307P/201F/7T/1U、l2 8516＝8307P/202F/6T/1U，双腿总数首次同 8516，P 恒 8307）；vs s550 stable **NEW 3 + GONE 0 + flips 0——位移全落 customizable-select home-end 族带**：3 子测键 End/Home/PageUp F/F 同相回归 stable（回 s549 形）+ PageDown 子测键 l2only F 单腿发射 + non-detailed 整文件键 l1only T 发射（s550 l2only 的换腿形）+ detailed 整文件键 T/T 同相 stable 持位（s550 回归形第 2 轮）；**归因：双腿同 runner md5 恒（同二进制）+ 窗零码变，位移纯腿间时序摆动，非族非哨键逐键恒等＝零码变零位移清偿，「申报=实测」谱系第三十九例（轻覆盖轮）**；**谱形注记：l1 整文件键 T 发射与 3 子测键并存（仅 PageDown 单吞）——整文件键/子测键「绝对互斥」模型修正为时序竞争非绝对互斥（整文件超时先至吞未完成子测键、已发射子测键保留；s550 l2 整文件键 T 吞全部 4 子测键为竞争全吞特例）**

日期：2026-11-17（s550 收口同日续轮；判净
14:38:42 起腿）。
承前：s550 收口轮（双刀窗归因 + 族键回归
观察兑现轮）。

## 窗与 runner（零码变轮底账）

- 轮首 pull 零新携入（s550 收口 push 后远
  端无新笔）；窗 = 9404d6d41→052748af4
  均零码面 docs，码态恒 ad487ea89。
- runner 审计（刀 G 工法）：pre/mid/post
  均 9a13023968c1/128,075,392（七代持位）
  ——leg1 Compiling ×6（product-version
  链 mtime 重编，44.68s）但二进制 md5 恒
  ＝确定性重编零换代，leg2 Fresh 0.22s；
  provenance 零异常（零码变窗预期形）。
- **工法注记（waiter 自阻塞）**：轮首
  waiter 以相对路径 `./wait-and-run.sh`
  启动，cmdline 不含 `/tmp/s551/wait-and-run`
  → v3.1 check_clean 的自排除模式不命中
  ，waiter 把自己计成重进程、静窗永不判
  净（14:32 起 ~1.5 min 自锁）→ 杀弃以绝
  对路径重启后 14:38:42 判净。**waiter 启
  动必须绝对路径（自排除依赖 cmdline 字
  面匹配）**，后续轮沿用。

## 双腿 join（HEAD 052748af4 树）

- v3.1 判净 14:38:42 HEAD=052748af4 钉扎
  起腿（clean-soaked procs=0 load<3 across
  3×45s checks）。leg1 14:38:42–14:42:08
  （3:26，含确定性重编）/ leg2 14:42:08–
  14:44:47（2:39）常态，rc=2 惯例 0 爆栈
  。
- raw canon：l1 8516（8307P/201F/7T/1U）、
  l2 8516（8307P/202F/6T/1U）——双腿总
  数首次同 8516；P 恒 8307 双腿恒等、U 恒
  1；T 7/6 差 1（l1 整文件键 T 发射）、F
  201/202 差 1（l2 PageDown 子测键发射）
  。
- join：**stable 8513 + disagree 0 +
  l1only 1 + l2only 1**（stable canon
  8305P/201F/6T/1U）。
- **vs s550 stable（8510）：NEW 3 + GONE 0
  + flips 0——位移全落族带，非族非哨键零
  位移逐键恒等**（零新 Pass 形零新 Fail
  形，NEW 3 均 F/F 回归非增益形）：
  - NEW 3 = 族 3 子测键 End/Home/PageUp
    F/F 同相回归 stable（回 s549 stable
    形；第 4 子测键 PageDown 本轮 l2only
    F 单腿发射、stable 外）。
  - l1only 1 = 族 non-detailed 整文件键 T
    发射（l1 形；l2 吞）——s550 l2only
    的换腿形。
  - l2only 1 = 族 PageDown 子测键 F（l2
    形；l1 吞）。
  - detailed 整文件键 T/T 同相 stable 持
    位（s550 回归形第 2 轮）。
  - **谱形注记：本轮 l1 同时发射整文件键
    T 与 3 子测键（仅吞 PageDown）——s550
    l2 整文件键 T 吞全部 4 子测键的「绝对
    互斥」模型修正为时序竞争非绝对互斥
    **（整文件超时先至则吞未完成子测键，
    已发射子测键保留入 raw）。
  - **归因判据：双腿同 runner 9a13023968c1
    （md5 恒同二进制）+ 窗零码变（
    ad487ea89 恒），腿间差异必为时序态非
    码面态**；族带摆动自 s530 起在册，本
    轮 3 子测键回归形在摆动谱内——**判族
    带时序摆动谱内记账，非码面位移**。
- P 集 stable **8305P 恒等持位**（第 4 轮
  ，s548/s549/s550 同值）；F 198→201（
  +3＝3 子测键 F 回归入账）；T 6 恒（
  detailed 持位；l1 整文件键 T 单腿不入
  stable）；U 恒 1（
  control-active-with-multiple-clicks 整
  文件键恒）。
- 双发发射伪影 2 键照旧在册（wrapping
  exact len + 149.html for=' WINdow）。

## 观察哨复核（五哨持位 + mpreload 常规计数第 13 轮）

- onblur 哨组 10 键双腿 join 零 flip
  （8P/2F parent-label 红池照常，**F 集与
  s550 逐键 IDENT（diff 零行实证）**）——
  flaky 键双腿双 Pass：零新例第 40 轮（
  s512 时钟顺延）。
- dialog-overlay 哨第 80 轮：双键双腿双
  P 常规持位。
- **modulepreload 哨：15 键双腿全 P/P
  ——常规连轮计数第 13 轮零翻红**（
  s550 排程兑现）。
- waapi 哨 2 键：picker-icon 双腿双 P
  （s478 刀增益第 74 轮保持）/ checkmark
  F/F 恒 F 带形不变。
- focus-dynamic 121 键双腿全 Pass：
  收窗后第 33 轮零再现。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8
--total-mem 16 --time-limit 1200）`cargo test
-p zero-engine --lib js_dom_bridge`（仓根执
行，guard rc=0）：**1690P/0F**（test 阶段
38.68s）。清单 vs s550（b550.k 1690）：
**1690 set 恒等，NEW 0 + GONE 0（
s551.bridge.new/gone 零行实证）**——零码
变窗清单不变合理，绿窗零破，基线
b550.k→b551.k（set 同）。

## 对端观察

- 对端刀信号持续：css 文法刀后续（
  shapepos 残面队列余项：位置串化规范刀
  ~498F/round 列表完整建模/inline inherit
  /path() 命令级插值/calc 代数插值）+
  wptfyi-css 刀 K（关键帧关键字面，与本流
  animation 面相关——在飞监视）+ wptfyi-css
  残面（mask-border 家族支持门定谳、B3
  残面 348F 余项、jsanim flip Steps(Start)）
  + wptfyi-editing d52 后续 + editing 流
  B4-S2 残面刀 + rendering dialog 16 红块
  协同面，join 实测为准。

## 结论

s551 收口（零码变守成轻覆盖轮 + 族带摆动
谱新形轮）：窗零码变（ad487ea89 恒，
runner 七代持位、确定性重编零换代）；浸
泡门双腿常态全绿零爆栈；join stable 8513
（8305P/201F/6T/1U）disagree 0 + l1only 1
+ l2only 1；vs s550 NEW 3 + GONE 0 + flips
0 位移全落 customizable-select home-end
族带——3 子测键 F/F 同相回归 stable + PageDown
l2only 单腿 + 整文件键 l1only 换腿形 +
detailed T/T 持位；**双腿同 runner md5 恒
判据定谳：位移纯腿间时序摆动非码面**，
非族非哨键逐键恒等＝零码变零位移清偿谱
系第三十九例；**互斥模型修正注记：整文
件键/子测键时序竞争非绝对互斥**；P 集
8305P 恒等第 4 轮；flaky 零新例第 40 轮，
onblur F 集逐键 IDENT，dialogov 第 80 轮
常规持位，mpreload 哨常规计数第 13 轮零
翻红，waapi picker 第 74 轮，fd121 第 33
轮；全桥 1690 set 恒等绿窗零破（b550.k→
b551.k）。工法注记：waiter 启动必须绝对
路径（自排除依赖 cmdline 匹配）。挂用户
门维持：GB-20261114-VIDEO + GB-20261031
（簇 D）+ s137（s445 已提示，不重复通知
）。

下一步：守成巡查（**对账基线换 s551
stable 8513——族带出位键回归观察继续**
：PageDown 子测键回 stable、整文件键吞形
或换腿、detailed 持位均落族带判摆动谱常
态记账（时序竞争模型下子测键部分吞为新
谱内形）；全桥清单基线 b551.k 1690 持位
；flaky 键零新例时钟顺延第 41 轮；**mpreload
哨常规连轮计数第 14 轮**；dialog 哨常规
持位（第 81 轮）；他流入树归因轮或轻覆
盖——对端刀信号持续同上，起腿前 runner
provenance 异常即查码面 diff（刀 G 工法
）；起腿一律 v3.1 浸泡门（**waiter 必须
绝对路径启动**）；慢腿/劣化腿按 s544 甄
别规程、fatal abort 形按 s545 一票作废处
置，join 实测为准）+ 候门决项等待用户拍
板（video P1+P2 立项 / 簇 D A2 立项 / s137
）；rendering 流 dialog 16 红块协同面（归
rendering 流记账）。门开随批落预案不变：
簇 D 开 → customizable-select-styles.css
×47 随批落。

工件：/tmp/s551/{s551-leg1,s551-leg2}.log、
s551-leg{1,2}.rc、leg{1,2}.s551.{start,end}
、legs.done、legs.head（052748af4）、
runner.{pre,mid,post}.{size,md5}（
9a13023968c1 三点恒＝零换代）、
window.s551.judge（14:38:42 clean-soaked
）、waiter.s551.log、run-legs.sh/
wait-and-run.sh（v3.1，绝对路径重启注记
）+ s551-{l1,l2}.norm、l551-{1,2}.canon、
s551.{stable.keys,stable.canon,disagree.keys
,leg1only.keys,leg2only.keys}、s551.new.keys
（3 族键）、s551.gone.keys（零行）、
s550.k1/s551.k1b（NEW/GONE comm 底稿）+
s550.stable.{keys,canon}（基线复用）+
b550.k（复用）+ b551.k（1690）+
s551-bridge.log、s551-bridge.rc（guard
rc=0）+ s477-{onblur,dialogov,mpreload,waapi}
.keys + fd121.keys + {s550,s551}.onblurF.keys
（F 集逐键 IDENT diff 实证）+ norm.sh/
canon.sh/join.sh（s488 修复版管线，export
LC_ALL=C 前缀调用）。
