# M2 s481 — 他流双刀入树归因轮：css 流 composite-add 刀 e42482bfd（shim add/accumulate 端点预合成+apply 十臂插值三臂）+ 编辑流 J2 跨 body 边界块并块刀 688210311（part05.js _b1BlockTags+_b1JoinMembers J2 臂）html-semantics 面双零位移清偿（stable flips 0 非族逐键零差，P 集 8307 持位，picker 刀增益键第 4 轮保持；五观察哨全零新翻红）+ 全桥绿窗

日期：2026-11-15。承前：s480 下一步「守成巡查（他流双刀入树则
归因轮）」。开工 `git pull --rebase` 远端零新提交（HEAD =
94cf8a6e6 即 s480 收口 docs，push 时已携入两刀）；`git diff
94cf8a6e6..HEAD -- crates/` 空确认码态 = 688210311 含：

- **e42482bfd feat(engine) composite-add 刀**（css 流，18:16）：
  shim part03.js +138/−12（`_zwComposeAddKeyframes`/`_zwHasAdd`
  等族——add/accumulate 动画注册前端点与 underlying 预合成按
  replace 登记）+ engine animation.rs +134（apply_single_property
  font-size/weight/size-adjust/letter-spacing/word-spacing/text-
  indent/min-max-sizing 十臂 + 插值三臂）；刀自证 corpus
  +272/0/0。半径申报 engine 动画面。
- **688210311 fix(editing) J2 刀**（编辑流，18:19）：part05.js
  +79/−2 零 Rust（`_b1BlockTags` 36 标签块面表 +
  `_b1JoinMembers` body 吸收臂 J2 分支跨 body 边界块并块）；
  刀自证 delete-in-child-of-html 42P/4F 净 +8。半径申报
  editing delete 域。

两刀 html-semantics 面均预期零位移（s479 d35 刀「申报=实测」
同形预期），join 实测为准——**实测兑现预期**。

## 二进制 provenance

- leg1 起跑 make 重建（零 `Compiling` 行但 runner mtime
  18:32:25 晚于两刀 18:16/18:19，与各轮 leg1 系统性重建现象
  同形）；leg2 零重建同二进制。刀后钉：`strings
  target/release/zero-wpt-runner | grep -c` ——
  `_zwComposeAddKeyframes` = 2 命中、`_b1BlockTags` = 3 命中
  （+ s479 `_zwExecCmdStylingCmds` = 5 持位）。solo/桥跑同二
  进制。
- 并跑窗注：leg1 窗内 ZeroSeed 他流 acceptance 作业并跑（host
  load 峰 10.89，起跑时 4.0 低于 s480 leg1 窗 6.5）——本轮双腿
  均零截断相位（6T 未膨胀），非 s480 型二页截断，无需 solo。

## 双腿 join（94cf8a6e6 树 = 码态 688210311，make testharness-html-semantics ×2）

- leg1 **8514 行（8307P/199F/6T/1U，distinct 8511）**；leg2
  **8516 行（8307P/199F/6T/1U，distinct 8514）**——双腿 P/F
  计数同形，仅族键分布差。
- join：**stable 8509，disagree 1，leg-only 1/1，全部限族**：
  - disagree = detailed 页整文件键（leg1 F vs leg2 T）；
  - l1only = optional 页整文件键（leg1 整文件相位）；l2only =
    End/Home/PageDown/PageUp 4 子测键（leg2 静窗子测相位，
    s478 形）。
- **vs s480 stable：GONE 4 + NEW 0 + flips 0，非族键逐键零
  差**——GONE 4 全数族键（detailed 整文件 + End/Home/PageUp
  三子测，本轮双腿相位差不入 stable，族相位谱 s464 起在册；
  PageDown 子测键两侧 stable 均不在）。
- **管线伪差勘误（本轮新录）**：join.sh 内 `sort -u` 会把双
  发同形行去重成单签名，stable.canon 须从 l1u（去重源）经
  awk NR==FNR 生成；本RunLoop 首算误从未去重 l1.canon 生成，
  "exact len"/"for='  WINdow" 两键现 " Pass Pass" 伪 flips——
  该两键双腿 raw 均双发同形 Pass（s479/s480 各腿同形 2 行，
  长驻 runner 发射伪影非本树新形），去重源重算后 flips 0。
- T 键集 5 = 五非族整文件 T 页与 s480 恒等（style-error-01 /
  audio-loading-lazy-source-deferred / sizes-auto /
  update-media / video-loading-lazy-source-poster-deferred）；
  detailed 族整文件键本轮 disagree 不入 stable（s480 时 T）。
- **P 集账本口径 8307 持位**（双腿双 8307P）；stable 机械口
  径 8305P（与 s479/s480 同形）。picker-icon 刀增益键 stable
  Pass 第 4 轮保持；checkmark 姊妹键 stable F 双相位带形不
  变。

## 观察哨复核（五组全零新翻红）

- onblur 哨组：stable 10 键全在册 = 8P + 2F（following-label
  双键 Pass 回位持位第 3 轮，红池 parent-label 双键照常 F）。
- dialog-overlay 观察哨第 10 轮零翻红（双键 stable Pass）；
  modulepreload 观察哨第 13 轮零翻红（15 键 stable 全 Pass）。
- waapi 哨 2 键（s477 建组）：picker-icon Pass / checkmark F
  在册。

## 全桥套件绿窗收口跑（双刀后树旁证）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1644P/0F rc=0**（测试清单 vs s480 set 恒等 1644——两刀一
改动画面 + 一改动 editing delete 门，桥钉面零破零新增，为刀
面无回归旁证）。

## 结论

s481 收口（他流双刀归因轮，本流零源码改动）：composite-add
刀 + J2 刀 html-semantics 面双双零位移清偿——stable 差集仅族
相位键（GONE 4 全族 + NEW 0 + flips 0），非族键逐键零差，与
s479「申报=实测」形态归为同类（对照 s478 WAAPI 刀破预期首现
增益）。P 集 8307 持位，picker 增益键第 4 轮保持，五观察哨零
新翻红，全桥绿窗钉面零破。挂用户门维持：GB-20261114-VIDEO +
GB-20261031（簇 D）+ s137（s445 已提示，不重复通知）。hang
观察项静默哨维持（双腿零真 hang，ZeroSeed 并跑窗零截断）。

工件：/tmp/s481/{s481-leg1,s481-leg2}.log、s481-{l1,l2}.norm、
l{1,2}.canon、s481.{stable.keys,stable.canon,st481.canon.u,
disagree.keys,leg1only.keys,leg2only.keys}、flips-stable.txt、
t481.keys/t480.keys、s477-{onblur,dialogov,mpreload,waapi}.keys
（s478 哨键组复用）、s481-bridge.log、
norm.sh/canon.sh/join.sh（s478 沉淀管线复用）。
