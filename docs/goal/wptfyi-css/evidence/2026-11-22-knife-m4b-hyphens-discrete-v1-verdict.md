# 刀 M4-b 发射方收账笔：P2F=16 改判 flake＋face 存活待受控重发（2026-11-22）

发射方＝witness-B fork-d（电池 unit `zw-knife-m4b-hyphens-discrete`，09:26:02
发射→09:58:08 EXIT rc=0）。本笔与 witness fail-stop 笔
（`2026-11-22-knife-m4b-hyphens-witness-failstop.md`，9437b0c66）构成双
witness 对账；电池档案、RECONCILE 数字（css-text +10／css-animations
P2F=16／values·transitions 零位移／ASYM=0）两笔一致，不重复。分歧仅在
P2F=16 的归因，以下为发射方追加实验链。

## 改判链：真机制回归假说被五组受控实验推翻

witness 因果定谳依据＝10:00-10:02 单窗三 bin 分岔（HEAD 系 17×4、face
bin 1×2）。发射方追加（全部快照 bin 零构建、单页
`CSS_CORPUS_ONLY_SUBSTR='Element-getAnimations.tentative'`、每臂 ~11s）：

| # | 窗 | 臂 | 结果 | 产物 |
|---|---|---|---|---|
| E1 | 10:05 | m4b-b1（face） | **17/21** | results-knife-m4b-singlepage/b1-json |
| E2 | 10:09:05-38 | m4b-b1 ×2 | 17/21（run3 幸存，run1 被 JSON 同名覆写） | results-knife-m4b-decisive/b1-json |
| E3 | 10:09:38-49 | m4b-b0（HEAD） | **1/21** | results-knife-m4b-decisive/b0-json |
| E4 | 10:11:41-10:12:13 | m4b-b0 ×4 | 17/17/17/17 | results-knife-m4b-alternator/run{1..4}-json |
| — | （witness 盘上重排） | 六臂 mtime | m4bb0 17,17 → m4bb1 1,1 → f1b0 17,17，逐块顺序发射 | results-m4b-getanim-probe/*-json |

判据：

1. **同款 HEAD bin 出现 1-form**：E3 中 m4b-b0（`7c6d3c42`，witness 表内
   「17/21 ×2 恒」的同 bin）得 1/21。1-form 存在无 face 路径（与 witness
   自记 f1b0 07:4x=1 同族）。
2. **同窗配对方向反转**：E2/E3 同窗背靠背 b1=17、b0=1，与 witness 窗
   b0 系=17、b1=1 完全反向。分岔随窗翻转＝窗主导，非 bin 主导。
3. **真机制回归不可能形态**：机制回归按 bin 确定，face bin 永不能回基线；
   实测 b1 在两个独立窗（E1、E2）达 17/21 满基线。
4. **爆发为分钟级瞬态、打击在窗臂**：E3 单臂 1-form 后 2 分钟 E4 同 bin
   四连 17；逐 bin 交替器假设证伪（E4）；f1b0 07:4x→10:02 跨窗翻转
   （witness 笔自记）同族。
5. **页面本征 wall-race**：该页所有 bin 所有窗全部 Timeout
   （state `pending:2, tests:22`，witness 六臂 corpus log 在盘），pass 数
   ＝墙钟前完成数，非语义判定；1-form 的 fail msg（`expected 1 but got 0`
   /`[object Undefined]`）＝动画时钟未前进时 getAnimations() 空数组读，
   非值损坏。

结论：P2F=16 按「回归/真实红/flake/伪红」四分类落 **flake**（环境敏感
regime 页，bin 无关瞬态爆发）。witness「face 真机制回归，非 flake／非
时窗」的时窗排除不成立——其探针为逐块顺序发射（b0 块→b1 块→f1b0 块），
爆发恰罩中 b1 块（10:02:03-26，23s）；「同窗三 bin 隔离」在该顺序设计下
无隔离力。发射方电池时 fail-stop 触发本身合规（当时无反证），本笔按
收口规则回写改判。

## css-text +10 归因（与 witness F2P=10 恰集一致）

126 子测两侧同构、零 F2P。恰集分两半：

- **transitions 车道 6 案＝预注册主判据兑现**：explicit allow-discrete
  ×3（t=-0.3/0/0.3 observed initial）＋all 位 ×3。
- **animations 车道 4 案＝臂机理同源自洽，预注册勘误**：t≥0.5
  （0.5/0.6/1/1.5）expected auto——b0 apply 面无臂→canonicalize no-op→
  echo 落 M4-a 后默认值 manual≠auto 失败；b1 臂在 to 侧写入 Auto 翻转。
  CLAIM 预注册「animations 车道零移动」只算了 from 侧（initial=manual
  同串），漏算 to 侧 expected auto 的 b0 既有失败，如实记勘误。

该页确定性值断言、非动画时序域、n=126 全谱稳定，不在 flake 域内，+10
可入账口径（待 face 落地随 commit 入账）。

## 处置

- **face 现状**：已被 witness 10:0x 剥树（animation.rs/transition_registry.rs
  对 HEAD 净，发射方 10:07 逐文件 diff md5 复核）；patch 完好在
  `.acceptance/wptfyi-css/knife-m4b-hyphens-discrete-face.patch`
  （md5 `96886a61`）＋binding 笔。**本窗不入库**（witness fail-stop 笔已
  落库且在飞双电池，不硬解共享面）。
- **tally 不动**：+10 待 M4-b2 受控重发 P2F=0 后随 face commit 入账。
- **M4-b2 处方（替代 witness 的 isolation 二分）**：改判成立则无需二分
  （无机制回归可分）。重发＝同 patch 同 CLAIM 判据，电池腿序改为
  **css-animations b0/b1 腿背靠背相邻**（控制窗变量），P2F 判读将该页
  按 witness 教训条目列入动画时序域 flake watch：仅当同窗相邻腿复现
  b1 独有损失才立真回归。M4-b2 窗在 D1-b2 animation.rs 窗之后，经
  NOTES 声明后取。
- **页面挂账**：Element-getAnimations.tentative.html 本征 Timeout
  （21 子测 2 unreported 恒在）＋分钟级 1-form 爆发，任何流单一读数
  不作该页增损证据（famd1b +16 勘误与本次 P2F=16 同根）。
