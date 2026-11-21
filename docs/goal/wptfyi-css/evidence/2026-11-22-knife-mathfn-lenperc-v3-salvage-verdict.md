# MATHFN-LENPERC v3 salvage 判定证据（2026-11-22 00:4x，监视/四眼流合并入账）

## 判定

**net +16 入账候选（页级 gained=17 / lost=1；lost=1＝known flake 剔除）**。
P2F=0 红线安全。与 v3 预注册（增益 12 全保＋单参 35 例回绿＋P2F=0）一致，
处方（站①′ `args.len() >= 2` 收窄门）兑现。

## 运行链（r6 残缺 → salvage 补臂）

| 臂 | 来源 | 时间 | FM | md5 | 腿 |
|---|---|---|---|---|---|
| b1 | r6b unit 00:31:22 发射 | 00:33:30-00:37:26 | 1 | 8ec195d4b5e04b7223a34e2dbce6880b | values rc=1 / sizing rc=1 |
| b0 | salvage unit 00:38 补建补跑 | 00:39:13-00:43:10 | 0 | 9eb16803c7d7cbfcf72311d2a3f1a538 | values rc=1 / sizing rc=1 |

- r6b b0 臂缺臂根因：results bin/ 目录在 build 窗内消失（r6 残缺笔 00:38，
  NOTES-knife-mathfn-converge-0022.md）——cp b0-runner 失败 → 双腿 rc=127
  空 JSON → 原 RECONCILE JSONDecodeError。判定 r6 残缺不可判（先手流抢救
  b1 JSON+bin 至 results-knife-mathfn-lenperc-r6-partial-kept/）。
- salvage（knife-mathfn-lenperc-r6-salvage-b0.sh）：树净核 → 序列化门 →
  test-guard build → M2 反向核（b0 md5 ≠ b1 md5 ✓，b0 未沾 face 毒）→
  b0 双腿补跑 → case 键级 RECONCILE。
- ride 态：S2a face（五文件，半成品带 198 案 allow-discrete 门序回归，
  S2 claim note 00:40 四眼笔）＋es_module.rs s419 ride 双臂同载相消，
  A/B 判读零沾染。

## RECONCILE case 键级差分（b0-salvage vs r6b-b1）

- values minmax-length-percent-serialize.html：P 28→36（**+8**）——
  `min/max(<len>,<%>)` 多参 computed 形保留恰集（v2 页级 gross 17 的
  serialize 页份额）。
- values minmax-percentage-serialize.html：P 14→23（**+9**）——
  `min/max(%,%,…)` 含 -0%/混序/三参形保留。
- values random-computed.tentative.html：P 120→119（**−1**）——
  `random(--identifier, a, b)` doc_seed 掷币 flake，按刀H/KA2 判例剔除。
- computed 页（minmax-length-percent-computed / minmax-percentage-computed）
  **零位移**——v2 单参和形误拦 −30 全数回绿，处方门生效。
- sizing 全零。**TOTAL gained=17 lost=1 → 净 +16**。

## tally 与落库

- tally 本笔不动（维持 +12968）；**v3 face（patch d6f60920，types.rs +
  engine computed_style.rs）落库同笔 +16 → +12984**，由先手流（patch
  认领流）执行；本笔为判定+证据合并笔，先手流收账笔落地时以本行为基
  合并，勿双计。
- 产物：results-knife-mathfn-lenperc/{b0,b1}/css-{values,sizing}.json、
  bin/{b0,b1}-runner 双 md5、salvage 日志 knife-mathfn-lenperc-salvage-b0-0038.log。
