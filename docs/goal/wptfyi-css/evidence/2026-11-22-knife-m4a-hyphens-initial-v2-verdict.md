# 刀 M4-a hyphens 初始值 v2 判定笔（2026-11-22 07:1x，fork-d witness 收账·双 fork 草案合流定稿）

## 判定 TOTAL：gained=2 lost=0 → P2F=0，**入账 +2**（tally +13113→+13115）＋主判据 observed 翻转精确兑现＝初始值三源校正机制定谳

> 合流注记：两 fork 同窗独立判读，数据全同（EXIT 时刻/bins md5/F2P 案名/主判据翻转）。
> 本笔＝对端先后两稿（r2-verdict 07:16:34＋v2-verdict 07:16:58）合并定稿，校正两处
> 事实：ASYM 实为 3 行（草案记 1 行且「event-order 本轮未现」与 RECONCILE 原始输出
> 矛盾）；transitions b1 腿时长 6m55s（草案一处误抄 7m05s）。r2-verdict 过渡稿
> 经实例-B 复活补对账节后与定稿双存档（07:2x 收编 commit，对账节在其尾节）。

## 电池档案

- unit `zw-knife-m4a-hyphens-initial-r2`（06:36 首发 FAIL＝FACE-APPLY-CHECK-FAILED
  树载 v1 遗留 face，剥净后 06:42 重发；serialize 门 06:42:37 过），log
  `knife-m4a-hyphens-initial-battery-r2-0642.log`，EXIT rc=0 07:15:32。
- face v2：`knife-m4a-hyphens-initial-face-v2.patch` md5
  `4a877c80e04308f2a8ae0f48c13981c5`，5 文件（patch-only 前向 apply 协议，
  battery 内 strip→b0→apply→b1）：
  - `default_impl.rs` Default impl hyphens None→Manual——**真实 gCS 回声源主修**
  - `registry.rs` initial_value 臂 None→Manual（v1 承袭；`hyphens: initial`
    声明解析源）——css-text-4 §hyphens-property Initial: manual
  - `types.rs` 枚举 doc 初始值事实校正
  - `extended.rs` test_hyphens_initial_value 期望 None→Manual（v1 face 破坏
    的既有域内测，随刀更新）
  - `apply_coverage.rs` 钉测 test_hyphens_initial_manual（v1 承袭）
- bins：b0 `a58e53e6880e12a0fa91e2fa17102085`（59.5s）/ b1
  `21354368932121c97507ac5678727e25`（56.7s）；M2 门过。
- RIDE 4 文件（animation.rs D1-b face／abspos.rs stretch face／es_module.rs
  s419／master.md）S1/S3 md5 恒等零扰动；EXIT 后本实例复测 3 码文件仍恒等
  （master.md 漂移＝peer docs commit 收编，bin 无涉）。
- 语料腿 8 全 rc=0：values 2m52s/2m46s、css-text 2m34s/2m33s、animations
  3m12s/3m10s、transitions 6m56s/6m55s。
- 快照双存档：`snap-knife-m4a-r2-0715/`（peer）＋`snap-knife-m4a-v2-0716/`
  （本实例，EXIT 后即时），bins md5 恒等互证。
- 钉测防线：dev 测 `test_hyphens*` 5 passed（06:39 发射前，v1 DOA 已解）；
  收账前 guard 测 style-system hyphens 组＋engine companion 单测（结果见
  处置节，companion 为两 fork 合流新增的 engine 侧随笔）。

## RECONCILE 逐项

1. **主判据精确兑现（claim 预注册 A 分支）**：`css/css-text/animations/`
   hyphen-no-interpolation.html 126 案 MSGDIFF 14，两形——
   - **observed 翻转 8 案**（主判据本体）：CSSA ×4＋WAAPI ×4，hyphens
     from [initial] to [auto] t∈{0.5,0.6,1,1.5}，observed b0 `"none "` →
     b1 `"manual "`（expected `"auto "` 仍 fail＝M4-b 真实缺陷持位，claim
     预注册「非本刀判据」）；
   - **expected 回声自愈 6 案**：allow-discrete transitions 车道 expected
     b0 `"none "` → b1 `"manual "`（observed 仍 `"auto "`＝M4-b 持位同上）。
   v1 的「另有路径」悬案就此闭合：路径＝`ComputedStyle::default()`
   （default_impl.rs），registry 臂与 Default impl 双源同改后回声翻转；
   M4-c 独立侦查刀前提消解（无第三路径），M4-b 保留。
2. **F2P=2 恰集**：`css/css-text/inheritance.html` P 26→28——
   `Property hyphens has initial value manual`＋`Property hyphens inherits`
   双案 fail→pass（spec 钉值页，claim「F2P 面：spec 钉值页翻 pass 计入账」
   恰集兑现，零外溢；N=42 双侧恒等零截断）。
3. **P2F=0 红线双口径**：861 页键级复核（knife-m4a-r2-case-judge.py，
   peer 实例）＋本实例独立 case-name 键级差分，两口径一致——intersection
   内零 Pass→Fail、截断真损=0、F2P 恰集外零翻绿。
4. **ASYM 3 行挂起零折算**（草案勘误后权威数）：css-animations
   event-dispatch.tentative（b0=11/b1=13）与 event-order.tentative
   （b0=1/b1=4）＝b1 异步多发现形；css-transitions event-dispatch.tentative
   （b0=18/b1=11）＝截断形（b1 缺 7 案在 b0 全 fail 态＝截断中性，peer
   键级复核定谳）。三 corpora gained=0 lost=0＝零 pass/fail 位移；按
   CSINTERP-S2A 四眼补笔判例（异步发现数漂移族、跨 run/bin 双向）挂起
   零折算，双向不入账。

## 判读链（v1→v2 完整机制考古）

- v1（registry 单源）零位移＋主判据不命中＋钉测 DOA（06:31 判定笔＋姊妹
  abandon 笔）→ 钉测首执揭出 `ComputedStyle::default().hyphens` 独立于
  registry；v2 补 default_impl 源后 observed 翻转＋钉值页翻绿＝三源校正
  （registry initial_value 臂／Default impl／序列化臂 None→"none" 映射
  不动）闭合。
- 「P2F=13」首判勘误与「转 M4-c」分叉经键级重放勘误（仲裁 NOTES 勘误
  二笔），见 v1 判定笔双 witness 对账节。

## 处置

- face 5 文件入库（fix commit）＋engine 侧 companion 随笔 2 处（协调情报
  条款兑现，test/comment-only 零 release bin 效应）：part03.rs
  test_get_computed_style_text_break_cluster `#def` hyphens 断言 none→manual
  ＋注释、computed_style.rs hyphens_to_css doc 注释校正——guard 测随刀执行
  验证；tally +13113→+13115。
- 残面挂账：M4-b（allow-discrete 车道 observed 恒 "auto "，6 案）＝
  transition 触发门对 discrete 值的真实缺陷，claim 预注册域外，独立刀位；
  ASYM event 页 flake 域维持双向不入账。
