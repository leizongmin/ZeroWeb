# 刀 FAM-E composition 电池判定：gained=119 / lost=2＝预注册红线 fail-stop（2026-11-22）

电池＝unit `zw-knife-fame-battery`，12:36:42 发射 → GATE-DONE/EXIT rc=0
13:04:25（发射 session 死于 429 后腿独立跑完，产物齐）。face＝`animation.rs`
8 hunk（patch md5 `bcc546be3e91c6a5a632b40fd3dd1efa`，tree md5 `9ea2063f…`
markers=13）＝E1 可加 add 求和臂＋E2 calc-mix 混合 compose（新符号
`parse_calc_mix_pair`/`calc_mix_pair_str`，S1 门 b1≥2/b0=0 过）＋applier 六臂。
HEAD=8f03a3c02；bin b0 `fc5121b2` / b1 `edec42c8`（results 目录 provenance.txt
落盘）。

PROVENANCE-DIRTY-TREE：peer 在途稿（insets/cache/abspos v1 期残稿四文件＋
es_module.rs）骑两臂（发射时已记 ride，s1/s2/s3 md5 核窗内零扰动）。判读三
语料（css-sizing composition 页/css-values/css-transitions）与该四文件触面
（cssom insets echo/abspos stretch）零交集；FAM-F 收账面 125P b0 持位
tripwire 过＝组合态噪声未入判读。

## RECONCILE（键级，6 composition 页逐案＋全谱页级）

| 语料 | F2P | P2F | 判读 |
|---|---:|---:|---|
| css-sizing | 118 | 0 | composition 6 页 P 125→243（census 三簇 E1/E2/E3 主战区） |
| css-values | 1 | **2** | interpolate-size-width-composition +1；calc-size-height-interpolation −2 |
| css-transitions | 0 | 0 | **ASYM event-dispatch.tentative b0=30→b1=17**＝b1 页级 Timeout |

TOTAL gained=119 lost=2 → 预注册红线（lost>0）＝**FAIL-STOP 不入账**，
tally +13617 不动；face 未入库，树已剥净（animation.rs 对 HEAD 净）。

## 红面键级归因

- `calc-size-height-interpolation.html` ×2（`from [calc-size(auto, size * 2)]
  to [50%]` 同形两案 Pass→Fail）：b0 经 HEAD calc-size 端点路径插值成功，b1
  新 calc-mix/compose 臂劫持同域端点（与 bordershape v4「新路径劫持旧委托」
  同家族）。同形系统翻转，非随机。
- `event-dispatch.tentative.html`：b1 页级 Timeout（state=loaded pending=14
  phase=4，b0 30 案全产物）＝dispatch 臂吞/阻 transition 事件致页 await 不
  归——非崩溃、非产物截断（页级 Timeout 条目＋缺失 13 案名单齐）。

## v2 处方（后续迭代预注册方向）

1. calc-mix 臂类型门：仅 %/calc 混合格准入，calc-size()/keyword 端点委托
   HEAD 原路径（calc-size 插值页零回归 tripwire 预注册）。
2. dispatch 臂：无 composite add 的 transition 车道早退保 HEAD 事件语义
   （event-dispatch 页持位 tripwire）。
3. E1 可加求和增益面（+118）与红面不相交可分流：求和臂单独出刀先行。

电池产物 `results-knife-fame-composition/`（b0/b1-json＋bin＋provenance）留证。
