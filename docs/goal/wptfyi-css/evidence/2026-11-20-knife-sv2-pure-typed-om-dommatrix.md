# 刀 SV2 纯账收账：typed-om/DOMMatrix face +24（2026-11-20）

## 结论

SV 刀（typed-om/DOMMatrix face，patch 代 03:29）纯口径实账 **净 +24，真实回归 0**，全部落在 css-typed-om 4 页。与 peer 已收 WA 刀 r4 账 +73 精确闭合合并电池 gained=97（73+24）。face 落库 = peer 提交 `a9ce144d8`（并发窗合流：本 clone 暂存同内容被其提交消费，md5 面等价）；控制面收账笔 = `ae421a632`（其 committed evidence 含 battery#1/#2 双跑叙事，本文补纯账分账法证与 contention 剔账）。

## 电池身份

- 脚本 `.acceptance/wptfyi-css/knife-sv2-battery.sh`（flock `/tmp/knife-wa-battery.lock` 串行），运行日志 `knife-sv2-battery-2nd.log`，GATE-DONE 05:31:33，SCRIPT-RC=0。
- 面域：12 腿 6 套件（css-animations / css-transitions / css-fonts / css-properties-values-api / css-typed-om / css-shapes），b0/b1 双臂各 6 腿。
- patch：`knife-sv2-face.patch`（03:29 代），strip/replay 由电池脚本 trap 回贴，收账核 S1/S3 双 strip-check 零扰动。

## 四臂纯度法证（md5 表）

| 臂 | runner md5 | (WA face, SV face) |
|---|---|---|
| r4.b0（WA 电池对照臂） | `495c7f2b` | (OFF, ON) |
| r4.b1（WA 电池处理臂） | `fee99384` | (ON, ON) |
| sv2.b0（SV 电池对照臂） | `8a3cbd78` | (ON, OFF) |
| sv2.b1（SV 电池处理臂） | `fee99384` | (ON, ON) |

- `r4.b1 ≡ sv2.b1`（md5 `fee99384` 恒等）＝跨电池构建确定性互证，两电池各臂 bake 窗互不污染。
- ride drift 法证：04:03:55 rebase 与 04:28:04 part10 编辑均落在两电池 bake 窗之外；sv2 脚本 b1 先建（树含全 face）、b0 后剥 SV face，时序与上表一致。
- SV 纯口径 delta ＝ `b1 − b0` ＝ (ON,ON) − (ON,OFF)，WA face 两臂恒在（已由 ea98cb8ae 落库）。

## reconcile 逐页明细（b0 8a3cbd78 → b1 fee99384）

```
css/css-typed-om/stylevalue-normalization/normalize-tokens.tentative.html: P 6->16 (+10)
css/css-typed-om/stylevalue-subclasses/cssTransformComponent-2d-flattening.html: P 0->4 (+4)
css/css-typed-om/stylevalue-subclasses/cssTransformComponent-toMatrix.html: P 0->8 (+8)
css/css-typed-om/stylevalue-subclasses/cssTransformValue-toMatrix.html: P 0->2 (+2)
css/css-animations/animationevent-interface.html: P 18->19 (+1)   [交互项，不重复认领]
css/css-animations/event-dispatch.tentative.html: P 0->2 (+2)     [交互项，不重复认领]
css/css-animations/Element-getAnimations.tentative.html: P 16->1 (-15) REGRESSED=15  [contention 伪影，剔账]
== TOTAL: gained=27 lost=15 regressed=15 pages_gain=6 pages_loss=1
```

### SV 纯账 +24 明细

| 页 | ΔP | face 贡献面 |
|---|---|---|
| normalize-tokens.tentative | +10 | var() 词元归一化 + declared var 宽化 + fallback CSSUnparsedValue |
| cssTransformComponent-2d-flattening | +4 | DOMMatrixReadOnly is2D 语义与组件合成 |
| cssTransformComponent-toMatrix | +8 | DOMMatrix 不可变变换族 + 组件 toMatrix |
| cssTransformValue-toMatrix | +2 | CSSTransformValue.toMatrix 合成（消费 part05 translateSelf/scaleSelf） |

### 剔账法证

1. **−15/-15 Element-getAnimations＝CPU contention 伪影，不入账**：同一二进制 `fee99384` 在 r4 腿窗同页跑出 16P（r4 reconcile 该页无 delta 行），sv2 腿窗仅 1P——同一二进制不同腿窗结果差异＝争抢伪影，非 face 回归。
2. **animationevent +1 / event-dispatch +2＝交互项，已含在 WA r4 账 73 内，不重复认领**：两页均为 css-animations 域，需 WA face 物化＋rides 才现（r4 gained=73 已含），SV 刀不二次入账。

## 交叉闭合

合并电池（`results-knife-sv2-combined-0421`，peer 收账 evidence `2026-11-20-knife-sv2-typed-om-matrix.md`）gained=97：
**97 ＝ r4 纯 WA +73 ＋ sv2-b3 纯 SV +24**，精确闭合。合并账 b0 与 HEAD 基线同 md5＝(OFF,OFF)，其 delta 天然是双 face 合并账，按该 evidence 声明不直接入刀账，分账以 r4 与本电池为准。

## 相对单位 −2：真缺陷已根修，三臂复证

`cssTransformComponent-toMatrix-relative-units.html` 的 −2 **不是伪影，是真缺陷**：peer battery#1（b0 `2ea2adae`/b1 `84dd2e5f`）实测该页 −2 并定性真缺陷，随以 CSSTranslate/CSSPerspective toMatrix 相对长度/百分比 TypeError 守门（绝对长度 allowlist）根修；battery#2（`8a3cbd78`/`fee99384`，即本电池）零差全账。本电池三臂独立复证根修后状态：r4.b1、sv2.b0、sv2.b1 该页均 **P=2/2**，且 r4/sv2 双电池 reconcile 该页均无 delta 行——修复跨电池、跨臂向稳定，无遗留工作。

## 常驻回归

真实回归 0。唯一 REG 行（Element-getAnimations −15）已证 contention 伪影（剔账证据同上）；r4 侧 REG 1（finished-animations）属 WAAPI 残面，归 peer WA2 刀，与本刀无关。

## 工件指针

- 电池日志：`.acceptance/wptfyi-css/knife-sv2-battery-2nd.log`
- WA 电池日志：`.acceptance/wptfyi-css/knife-wa-battery-r4.log`（gained=73 lost=0 regressed=1 pages_gain=17）
- 合并账快照：`.acceptance/wptfyi-css/results-knife-sv2-combined-0421-backup/`（peer 腿覆盖前 12 JSON 保全）
- fork 仲裁：`.acceptance/wptfyi-css/NOTES-knife-waapi-fork-roles.txt`（04:31/04:35/05:13 笔）
- face patch：`.acceptance/wptfyi-css/knife-sv2-face.patch`（03:29 代，诊断留档不入库）
