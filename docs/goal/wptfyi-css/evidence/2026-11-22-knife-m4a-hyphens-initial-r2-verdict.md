# 刀 M4-a hyphens 初始值 r2 判定笔（2026-11-22 07:1x，fork-d witness 收账）

## 判定 TOTAL：gained=2 lost=0 → P2F=0，**入账 +2（tally +13113→+13115）**；主判据精确兑现（observed "none "→"manual "）＝真实初始值源 default_impl.rs 定谳

## 电池档案

- unit `zw-knife-m4a-hyphens-initial-r2`（witness 06:42 发射；serialize 门
  06:42:37 过），log `knife-m4a-hyphens-initial-battery-r2-0642.log`，
  EXIT rc=0 07:15:32（八腿全 rc=0：values 2m52s/2m46s、text 2m34s/2m33s、
  animations 3m12s/3m10s、transitions 6m56s/6m55s）。
- face v2：`knife-m4a-hyphens-initial-face-v2.patch` md5
  `4a877c80e04308f2a8ae0f48c13981c5`，5 文件（registry.rs initial_value 臂＋
  **default_impl.rs Default impl＝真实回声源主修**＋types.rs 枚举 doc＋
  extended.rs 既有测期望校正＋apply_coverage.rs 钉测）。patch-only 前向贴附，
  reverse-check 绿。
- bins：b0 `a58e53e6880e12a0fa91e2fa17102085`（1m00s）/ b1
  `21354368932121c97507ac5678727e25`（56.7s）；M2 门过。
- RIDE 4 文件（es_module.rs＝s419＋animation.rs＝D1-b face＋abspos.rs＝stretch
  face＋master.md 他流行），S1/S3 md5 恒等零扰动（master.md 于 S3 检查后他流
  追笔，docs 域零 bin 效应，见仲裁 NOTES 06:41/06:5x 笔）。
- 快照：`.acceptance/wptfyi-css/snap-knife-m4a-r2-0715/`（JSON/bin/log 全套）。
- 钉测防线：dev 测 `test_hyphens*` 5 passed（06:39 发射前实测，v1 DOA 已解）。

## RECONCILE 逐项

1. **主判据精确兑现**：`css/css-text/animations/hyphen-no-interpolation.html`
   MSGDIFF 14/126，两形——
   - **observed 翻转 8 案**（主判据本体）：CSSA ×4＋WAAPI ×4，hyphens
     from [initial] to [auto] t∈{0.5,0.6,1,1.5}，observed b0 `"none "` →
     b1 `"manual "`（expected `"auto "` 仍 fail＝M4-b 真实缺陷持位，claim
     预注册「非本刀判据」）；
   - **expected 回声自愈 6 案**：allow-discrete transitions 车道 expected
     b0 `"none "` → b1 `"manual "`（observed 仍 `"auto "`＝M4-b 持位同上）。
   v1 的「另有路径」悬念就此闭合：路径＝default_impl.rs Default impl，
   非新机制。
2. **F2P=2 恰集**：`css/css-text/inheritance.html` P 26→28——
   `Property hyphens has initial value manual`＋`Property hyphens inherits`
   双案 fail→pass（spec 钉值页，claim「F2P 面：spec 钉值页翻 pass 计入账」
   恰集兑现，零外溢）。
3. **键级复核（knife-m4a-r2-case-judge.py）**：861 页 intersection 全案
   status diff＝**P2F=0／F2P=2**；ASYM 1 页（event-dispatch b0-fail-missing=7，
   截断中性零 pass 损）。P2F 红线绿。
4. values/animations/transitions 三处零位移（页级＋键级）。

## 落库（同笔伴生，7 文件）

- face 5 文件＋**engine 伴生 2 文件**（test/comment-only，Default 翻转后的
  必翻红面随刀校正）：`part03.rs` test_get_computed_style_text_break_cluster
  `#def` hyphens 断言 `"none"→"manual"`＋spec 注释；`computed_style.rs`
  hyphens_to_css doc 注释（diverge 记述过时校正）。painter 零风险定谳：
  连字符指示器仅 `matches!(…, Auto)` 臂，Manual≠Auto 零渲染漂移。
- 断言实证链：`#hyph-manual → "manual"` 既有通过行＝`hyphens_to_css(Manual)`
  序列化已被覆盖；Default→Manual 由 style-system dev 测 10 passed 实证
  （含钉测 test_hyphens_initial_manual）——伴生断言两段机制均有实跑覆盖。
- v1 判读双笔（本流
  `2026-11-22-knife-m4a-hyphens-initial-v1-verdict.md`＋姊妹流
  `2026-11-22-knife-m4a-hyphens-abandon-verdict.md`）存档，其「P2F=13/
  转 M4-c」分叉经键级重放勘误（仲裁 NOTES 勘误二笔），M4-c 侦查前提随
  主判据兑现消解。
- 残面挂账：M4-b（allow-discrete 车道 observed 恒 "auto "，6 案）＝
  transition 触发门对 discrete 值的真实缺陷，claim 预注册域外，独立刀位；
  ASYM event 页 flake 域维持双向不入账。

## 双 witness 对账（07:2x 补节）

EXIT 后两实例同窗独立收账（仲裁 NOTES r2 收账声明笔），判读事实层完全
合流：主判据命中（MSGDIFF 14/126，observed 翻转 8＋expected 回声 6——
案数拆分与实例-B 亲读 0642 日志逐行一致）、键级 P2F=0/F2P=2、TOTAL
gained=2 lost=0、入库＋tally +2。补充互证：实例-B 独立快照
`snap-knife-m4a-r2-0716/`（含 ride 追踪件）与本笔 `snap-knife-m4a-r2-0715/`
b0/b1 runner md5 恒等（a58e53e6…/21354368…＝发射记录）——双独立快照
同源 pristine，判读输入无分叉。
