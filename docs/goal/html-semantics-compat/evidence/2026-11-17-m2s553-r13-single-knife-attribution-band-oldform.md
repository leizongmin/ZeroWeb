# M2 s553 — **R13 单刀窗归因轮 + 族带旧形回归轮**：窗内码面刀 ×1 = R13 cE=false wrap 刀（b65398b99 fix(editing)，part01g.js +76/−18 wrap 命令整面纳入（fb 362 no-op 转 wrap＋indent 三连形双键＋容器形分叉）+ part134_r13_cefalse_wrap_pins.rs 新钉测 104 行 + js_dom_bridge_tests.rs +1 注册行；s552 收口时在飞候选兑现落窗）——码态 d582d2ee9→e4fb2c42e（伴两 docs-only 非触发器），runner 八代 5d81df5207/128,083,488→九代 96f2c0a8df/128,087,248 换代（+3,760，leg1 Compiling ×6 重编，R13 经 zero-engine 依赖链入 runner 链接，provenance 零异常）——v3.1 浸泡门判净 15:51:11 HEAD=e4fb2c42e 钉扎起腿：双腿常态全绿（leg1 3:31 含重编/leg2 2:43 Fresh 同二进制，rc=2 惯例 0 爆栈）——join **stable 8511（8305P/199F/6T/1U）disagree 0 + l1only 0 + l2only 2**（raw l1 8513 行＝8511 键、l2 8515 行＝8513 键，双腿各含双发伪影 2 行重复发射）；vs s552 stable **NEW 11 + GONE 0 + flips 1，位移两组全落带内**：①password 11 键 NEW＝s552 l1 单腿文件级超时吞形自然回归兑现（select-event 两腿各 45 password 键全 Pass、文件完整跑完，无吞形再现）②home-end 族带旧形回归＝non-det 整文件键 flip F→T（回 s549–s551 Timeout 带形）＋Home/End 2 子测键 l2only F 部分发射（assert_true: expected true got false；PageDown/PageUp 未发射）；detailed 变体双腿仍 90s page-script 0-declared 超时 F/F（与 s552 stable 恒＝非新位移、该形持位第 2 轮续观察）；**归因：族带摆动 s530 起在册、90s 0-declared 模式刀前在册（s549 leg2 先例）、R13 语义面（cE=false wrap 命令族）与 base-select 键盘 testdriver 页无路径关联——判摆动谱旧形回归非 R13 码面，s552 新极形（双整文件键 F/F 0-declared 位移形）未复现→按既定判据走常态记账、升级路径（单页探针+对端 pre-knife runner 反事实 A/B 前置申请）未触发**；非带非哨键零码面位移＝R13 零位移清偿，「申报=实测」谱系第四十例（单刀窗）

日期：2026-11-17（s552 收口同日续轮；判净
15:51:11 起腿。rate-limit 429 断点续轮——
上轮 15:26 完成起腿准备、15:57 双腿跑毕，
19:13 限额重置后本会话续做 join 分析收口
，腿工件 provenance 完整无缺）。
承前：s552 收口轮（双刀窗归因 + 族带新极形
轮）——本轮兑现其首要观察项裁定。

## R13 刀底账（s552 排程兑现）

- s552 收口时「对端刀信号：editing 流 R13
  cE=false wrap 刀（刀序 2）在飞候选」——
  轮首实查已落树：b65398b99（part01g.js +
  part134 pins + bridge 注册行），窗内唯一
  码面刀（6b247acd1 渲染 docs + e4fb2c42e
  s552 收口 docs 均 docs-only 非触发器）
  ——单刀窗归因轮。
- runner 审计（刀 G 工法）：pre
  5d81df5207（八代）→ post 96f2c0a8df
  （九代，128,083,488→128,087,248，
  +3,760）——leg1 Compiling ×6（
  zero-engine 及下游链），leg2 Fresh；
  part01g.js 经 js_dom_shim 入链＝换代
  归因 R13，provenance 零异常。

## 双腿 join（HEAD e4fb2c42e 树，含 R13 刀）

- v3.1 判净 15:51:11 HEAD=e4fb2c42e 钉扎
  起腿。leg1 15:51:11–15:54:42（3:31 含
  重编）/ leg2 15:54:42–15:57:25（2:43，
  Fresh 同二进制），rc=2 惯例 0 爆栈；
  leg2 无劣化形（vs s552 3:14 回落常态带）。
- raw canon：l1 8513 行（8307P/199F/6T/
  1U）≠ l2 8515 行（8307P/201F/6T/1U）
  ——唯一键口径 l1 8511 / l2 8513（双腿
  各双发伪影 2 行重复发射）；U 恒 1。
- join：**stable 8511 + disagree 0 +
  l1only 0 + l2only 2**（stable canon
  8305P/199F/6T/1U）。
- **vs s552 stable（8500）：NEW 11 + GONE 0
  + flips 1**，位移两组：
  - **①password 11 键 NEW 回归**＝s552 l1
    文件级超时吞形自然回归（s552 首要观
    察项之二兑现）：select-event.html 两
    腿 45 password 键全 Pass、整文件无吞
    形——纯时序恢复，非码面。
  - **②族带旧形回归**：non-det 整文件键
    flip **F→T**（s552 的 F 形回 s549–
    s551 在册 Timeout 带形，双腿 T/T 入
    stable）＋Home/End 2 子测键 l2only F
    部分发射（assert_true expected true
    got false；PageDown/PageUp 两腿零发
    射）——时序竞争谱内形。
  - detailed 变体：双腿仍 90s 0-declared
    F/F＝与 s552 stable 逐键恒，**该形持
    位第 2 轮**（非新位移；若 s554 再现
    NEW/GONE 位移形→升级单页探针+对端
    pre-knife runner 反事实 A/B 前置申请
    判据不变）。
  - **归因判据**：族带摆动 s530 起在册；
    90s 0-declared 模式刀前在册（s549
    leg2 同文件同消息）；R13 语义面
    （cE=false 选区 wrap：fb/indent/
    contenteditable=false 命令族）与
    base-select 键盘 testdriver 页无编辑
    命令路径关联（页面零 execCommand 调
    用）；leg2 无劣化、password 吞形自愈
    佐证本轮机器时序回常态带——**判族带
    时序谱旧形回归、非 R13 码面；s552 新
    极形位移（双整文件键 F/F NEW/flip 入
    账形）未复现→按 s552 既定判据常态记
    账、升级路径未触发**。
- stable counts：P 8294→**8305**（回
  s548–s551 四轮恒等位，+11 全＝password
  时序回归）；F 200→**199**（home-end
  non-det 出 F 带翻 T）；T 5→**6**（
  non-det 回 T 带＋惰载 5 键持位：
  style-error-01/audio-loading-lazy-
  source-deferred/sizes-auto/update-media
  /video-loading-lazy-source-poster-
  deferred 恒）；U 恒 1。
- 双发发射伪影 2 键照旧在册（双腿同键重
  复发射）：wrapping-transformation
  .window.js :: exact len +
  execution-timing/149.html :: for='
  WINdow。

## 观察哨复核（五哨持位 + mpreload 常规计数第 15 轮）

- onblur 哨组 10 键双腿 join 零 flip（8P
  /2F parent-label 红池照常，**F 集与
  s552 逐键 IDENT（diff 零行实证）**）
  ——flaky 键双腿双 Pass：零新例第 42 轮
  （s512 时钟顺延）。
- dialog-overlay 哨第 82 轮：双键双腿双 P
  常规持位。
- **modulepreload 哨：15 键双腿全 P/P
  ——常规连轮计数第 15 轮零翻红**（s552
  排程兑现）。
- waapi 哨 2 键：picker-icon 双腿双 P（
  s478 刀增益第 76 轮保持）/ checkmark
  F/F 恒 F 带形不变。
- focus-dynamic 121 键双腿全 Pass：收窗
  后第 35 轮零再现。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8
--total-mem 16 --time-limit 1200）`cargo test
-p zero-engine --lib js_dom_bridge`（仓根执
行，guard rc=0）：**1693P/0F**（test 阶段
19.96s）。清单 vs s552（b552.k 1692）：
**1692→1693，NEW 1 + GONE 0**——NEW 1＝
test_r13_cefalse_wrap_pins（part134）
**每刀 1 聚合钉测 fn 模式持位**（第 3 刀连
续：part132/part133/part134 同型）＝R13 桥
面钉全绿。基线 b552.k→b553.k（1693）。

## 对端观察

- 对端编辑流 R13 刀落窗兑现（刀序 2），
  hs 面「申报=实测」零位移清偿第四十例。
- 对端刀信号持续：css 文法刀后续（
  shapepos 残面队列余项）+ wptfyi-css 刀
  K（关键帧关键字面，与本流 animation 面
  相关，在飞监视）+ wptfyi-css 残面（
  mask-border 家族支持门定谳、B3 残面
  348F 余项、jsanim flip Steps(Start)）+
  editing 流 B4-S2 残面刀 + rendering
  dialog 16 红块协同面，join 实测为准。

## 结论

s553 收口（R13 单刀窗归因轮 + 族带旧形回
归轮）：窗含一刀（R13 cE=false wrap，
part01g + part134 pins），runner 九代换代
（+3,760）provenance 零异常；浸泡门双腿常
态全绿零爆栈；join stable 8511（8305P/
199F/6T/1U）disagree 0 + l1only 0 +
l2only 2；vs s552 NEW 11 + GONE 0 + flips
1——**password 11 键时序吞形自然回归兑现
（首要观察项之二）；族带新极形未复现、
non-det 整文件键 F→T 回旧 Timeout 带形＋
2 子测键 l2only 部分发射＝摆动谱旧形常态
记账，detailed F/F 0-declared 持位第 2 轮
续观察；R13 语义面与 hs 键无路径关联、非
带非哨键零位移＝零位移清偿第四十例（单刀
窗）**；P 集 8305 恢复四轮恒等位；flaky
零新例第 42 轮，五哨持位；全桥 1693P/0F
NEW 1（test_r13_cefalse_wrap_pins，b552.k
→b553.k）。挂用户门维持：GB-20261114-VIDEO
+ GB-20261031（簇 D）+ s137（不重复通知）。

下一步：守成巡查（**对账基线换 s553
stable 8511——detailed F/F 0-declared 形
持位续观察为首要观察项**：s554 若再现
NEW/GONE 位移形→升级单页探针（runner 单
文件过滤）+对端 pre-knife runner 反事实
A/B 前置申请，再定归因；持位恒→续常态记
账；home-end non-det T 带形回摆观察＋
PageDown/PageUp 子测键发射谱观察；
password 11 键 stable 持位确认（第 2 轮）
；全桥清单基线 b553.k 1693 持位；flaky 键
零新例时钟顺延第 43 轮；**mpreload 哨常规
连轮计数第 16 轮**；dialog 哨常规持位（第
83 轮）；他流入树归因轮或轻覆盖——对端
刀信号持续同上（css 文法刀后续+wptfyi-css
刀 K+残面+rendering dialog 协同面），起腿
前 runner provenance 异常即查码面 diff（
刀 G 工法）；起腿一律 v3.1 浸泡门（waiter
绝对路径）；慢腿/劣化腿按 s544 甄别规程、
fatal abort 形按 s545 一票作废处置，join
实测为准）+ 候门决项等待用户拍板（video
P1+P2 立项 / 簇 D A2 立项 / s137）；
rendering 流 dialog 16 红块协同面（归
rendering 流记账）。门开随批落预案不变：
簇 D 开 → customizable-select-styles.css
×47 随批落。

工件：/tmp/s553/{s553-leg1,s553-leg2}.log、
s553-leg{1,2}.rc、leg{1,2}.s553.{start,end}
、legs.done、legs.head（e4fb2c42e）、
head.pre（e4fb2c42e 轮首恒）、
runner.{pre,mid,post}.{size,md5}（
5d81df5207→96f2c0a8df 九代在册）、
window.s553.judge（15:51:11 clean-soaked
）、waiter.s553.log + s553-{l1,l2}.norm、
l553-{1,2}.canon、s553.{stable.keys,
stable.canon,disagree.keys,leg1only.keys
,leg2only.keys}、s553.new.keys（11 全
password）、s553.gone.keys（0）、
s553.flips（1 族键）、s553.s1/s2（签名面
，双发伪影重复行甄别用）+ s552.stable.
{keys,canon}（基线复用）+ b552.k（复用）
+ b553.k（1693）+ s553.bridge.{new,gone}
（1/0）+ s553-bridge.log、s553-bridge.rc
（guard rc=0）+ s477-{onblur,dialogov,
mpreload,waapi}.keys + fd121.keys +
s553.onblurF.{l1,l2}（l1==l2==s552 IDENT
diff 实证）+ norm.sh/canon.sh/join.sh
（s488 修复版管线，export LC_ALL=C 前缀调
用）。
