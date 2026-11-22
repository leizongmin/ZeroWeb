# 刀 M4-b witness 判定：P2F 红线 fail-stop＋famd1b 勘误（2026-11-22）

witness＝2468c5d8 续开 fork（famd1b 执行方血统）；电池发射权归复活 fork
（unit `zw-knife-famf...` 同窗另有 fork-d FAM-F）。发射方未出收账笔前本笔
先行独立判读，发射方 verdict 落库后以「双 witness 对账」合并（M4-a 三笔
先例，canonical 指定后并存）。

## 电池档案（发射方 unit zw-knife-m4b-hyphens-discrete）

- 09:26:02 发射 → 09:58:08 EXIT rc=0。patch md5 `96886a61fde9d6601ce8374117300c29`
  （face：animation.rs hyphens 关键字臂＋import HyphensComputedValue；
  transition_registry.rs DISCRETE_TRANSITIONABLE 17→18 追加 "hyphens"，
  双文件 +27/−2）。
- 双臂 bin：b0 `7c6d3c426127868aa197cbf0af778909`（HEAD 态，face 剥除 build）/
  b1 `a71ce60353fc4fe2c7d91d4d891cbdc3`（face 态 build）＝M2 异 md5 ✓；
  b1 与 witness probe-verify face-runner（09:24，同面同树）md5 恒等＝构建
  确定性双证。ride 1 文件（es_module.rs s419）S1/S3 零扰动。
- 首次发射 PATCH-WRONG-TARGET（gate grep 缺 src/ 分量）经发射方修脚本后重发，
  本笔不深究。
- RECONCILE 内嵌 python 于 MSGDIFF 打印处 None 下标崩溃（脚本缺陷）——
  原始 JSON 完好，本 witness 以键级 judge（knife-m4a-r2-case-judge 同法）
  补判。

## RECONCILE（键级，861 页全谱）

| 语料 | 判读 |
|---|---|
| css-values | 零位移（含 zz-famd1-intake-probe.html 零位移＝探针页骑乘无扰） |
| css-text | hyphen-no-interpolation.html P 28→38 **+10**＝主判据面（allow-discrete 两 variant＋all 车道＋Animations 面 observed "auto "→ 翻转，MSGDIFF 10/126） |
| css-animations | **P2F=16**（全部集中 Element-getAnimations.tentative.html，见红线） |
| css-transitions | 零位移 |
| ASYM | 0 页；截断真损 0 |

F2P=10 与 CLAIM 预注册恰集吻合（预注册 6 案＋all 车道翻转 bonus 4），
主判据兑现；红线独立触发。

## 红线（VERDICT: FAIL-STOP）

P2F=16：Element-getAnimations.tentative.html 16 案 b0 Pass → b1 Fail，
b1 msg 形如 `getAnimations returns … expected 1 but got 0` /
`[object CSSAnimation] but got "[object Undefined]"`＝动画注册表空读。

### face 因果定谳（三冻结 bin 单页 2×2 判别探针，同窗 10:00-10:02）

| bin | 树态 | Element-getAnimations | dynamic-changes |
|---|---|---|---|
| m4b-b0 `7c6d3c42` | HEAD（face 剥除） | 17/21 ×2 | 5/5 ×2 |
| famd1b-b0 `a7f5cb8f` | HEAD（07:2x 建） | 17/21 ×2 | 5/5 ×2 |
| m4b-b1 `a71ce603` | face（hyphens 臂＋registry append） | **1/21 ×2** | 5/5 ×2 |

- HEAD 态双 bin（构建相差 2h）同窗恒 17/21；face bin 恒 1/21；邻页
  dynamic-changes 三 bin 恒 5/5（损伤页面特异性）。
- 结论：16 P2F＝**face 真机制回归**，非 flake／非时窗。断链半区
  （animation.rs hyphens 臂 vs transition_registry.rs 追加）待
  serialize 窗开启后 isolation build 二分（famf/cssom-insets-v2 双电池
  在飞，本 fork 已让位 crates 触碰）。
- 机制候选：registry append 后离散可过渡清单消费点（animation.rs /
  transition_registry.rs）对 18 元素的遍历或长度假设失配 → 动画注册
  链路空转。修复处方候补：二分定位后修正消费点或以逐条目追加替换单点
  清单改写；M4-b2 再判据＝本页回 17/21 基线＋css-text +10 保持。

## famd1b 判定笔勘误（涉 9b0ad54f3 已入库 verdict 的 +16 口径）

- f1b0（a7f5cb8f，famd1b 时代 HEAD 态 bin）**同 bin 双窗翻转**：famd1b
  电池窗 07:4x 扫 css-animations＝1/21；本窗 10:02 探针＝17/21（md5 恒等）。
  m4b-b0 在 09:33 电池窗亦 17/21。⇒ Element-getAnimations 页存在环境敏感
  regime（famd1b b0 窗疑并发电池残余负载），**基线＝17/21**。
- 故 famd1b verdict 所记「css-animations Element-getAnimations +16（1→17）
  真增益」不成立：其 b1 的 17＝基线复现，b0 的 1＝环境污染伪影；D1-b 49F
  口径中 16F 撤销（余 33F＝calc-interpolation +22＋probe +11 口径不变，
  其中 probe 页系 zz- 排除域，入账口径以 tally 收账笔为准——D1-b 本就
  fail-stop 未收账，tally 无影响）。
- CSSTransition-effect.tentative 的 P2F=1 fail-stop 结论**不变**（该页
  solo 判据独立于本次勘误域；witness 复核时同法可再证）。
- 教训入律：动画时序域页（getAnimations/event-dispatch 系）单跑读数不作
  face 增益证据，须跨窗或同窗三 bin 隔离后才可入 F2P 口径。

## 处置

- m4b face **弃置不入账**（tally +13269 不动）；树载 face 已由本 fork
  10:0x 剥净（animation.rs/transition_registry.rs 对 HEAD 净），发射方
  无需处置。patch 存 `.acceptance/wptfyi-css/knife-m4b-hyphens-discrete-face.patch`
  作 M4-b2 基座。
- 电池产物 `results-knife-m4b-hyphens-discrete/`＋判别探针
  `results-m4b-getanim-probe/` 留证。
- M4-b2 处方待 isolation 二分笔；F2P +10 主判据面真实，修复断链半区后
  可重发（CLAIM 判据沿用）。
