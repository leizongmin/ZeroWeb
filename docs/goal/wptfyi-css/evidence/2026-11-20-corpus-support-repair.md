# ZW3 语料修复：css/support helper 缺失批露与回基线（2026-11-20 03:2x）

## 定谳（法证链）

**现象**：本 clone（ZW3）五语料腿大批页级 wholesale-fail（单 Fail
「declared tests: 0」）——刀 TD b1 JSON（02:4x 口径）盘点：masking
58/62 页、animations 74/123、transitions 63/123、pva 37/111、typed-om
8/359；合计 ~244 页整页失分数千子测。错误签名 `ReferenceError: X is not
defined`，X ∈ {test_valid_value 29 / test_invalid_value 26 /
test_computed_value 23 / test_interpolation 15 / test_composition 7 /
test_no_interpolation 5 / assert_not_inherited / test_shorthand_value /
test_keyframes_name_* / waitForAtLeastOneFrame / test_not_animatable}。

**根因**：`wpt-data/css/support` 在本 clone 只有 10-10 精选仓时代 3 张
图片，**上游 pin 的 9 个 testcommon JS + 13 个 css/png fixture 自始未拉**
（目录 mtime 10-10 23:23 未变＝历史从未有过 JS）。11-19 22:45 五 corpus
重拉（css-animations/masking/pva/transitions/typed-om，`.goal-fetched-`
标记为证）走 CSS_FETCH_ONLY 白名单未含 `css/support`——35/41 号脚本 DIRS
与 Makefile weekly 三分片均含该目录，本地 22:45 那次过滤名单漏列。runner
script fetcher 按 `/css/support/X` 绝对路径解析到 `wpt-data/css/support/X`
（testharness.rs `wpt_data_script_fetcher`），缺文件即 script load 失败。

**波及面**：仅本 clone 本地电池腿（刀 co 23:34 / TD 00:06+02:25 的
masking/transitions/animations/pva 哨兵腿）。已落库刀账不受染：
- 主靶页不依赖 support JS（typed-om/pva registered-property/
  animation-range/shapes/fonts 页均 test() 直书）——+134/+68/+47/+44 等
  账实子测计量，成立；
- 哨兵「零差异」系两侧对称塌（b0/b1 同缺）——零回归结论成立但**盲区**：
  masking 哨兵在塌态下 62 页全 Fail，页级/子测级位移不可见。
- 残面普查（批三）的池量（mask-border ~419F 等）出自健康口径（knife-g b1，
  masking 1489F），本刀修复后 masking F=1489 **精确对上**＝普查数字有效。

**与 weekly 账无关**：weekly css-face 三分片 fetch 名单均含 `css/support`
（Makefile 646/654/657），CI/weekly 自拉语料健康基——控制面 weekly
91,520/198,899 账面不受本缺口影响。

## 修复（corpus-repair-1120.sh）

上游 pin `315976933870b34d6ea30e3f6643403edae678ba`（goals/lib.sh 同 pin）
逐文件 jsdelivr→raw 回落拉取 23 文件：css/support 22（9 testcommon JS +
13 fixture）+ resources/idlharness.js。**逐文件 md5 与 ZW2 副本对账 23/23
全等**（ZW2 = 10-30 35 号脚本正典拉取同 pin；五 corpus 树内容字节级同
pin 已验：masking/pva/transitions/typed-om 四目录整树 md5 恒等，
css-animations 恒等（排除 ZW2 本地探针 knife-waapi-probe.html））。
写 `.goal-fetched-<pin>` 标记（幂等快路径口径，后续 weekly 式全量拉不再
重列）。`resources/WebIDLParser.js` 上游 pin 404（构建期生成资产，两 clone
均缺）——6 页 css idlharness 维持双 clone 已知缺口，非本修复范围。

## 回基线（corpus-repair-verify-1120.sh，bin fad090d9＝HEAD 态）

五语料腿单侧复跑（test-guard per-proc 4/total 8/3000s + CASE_WALL_S=60），
基线=修复前刀 TD b1 JSON：

| 语料 | Pass 前→后 | Fail 前→后 | wholesale 前→后 |
|---|---|---|---|
| css-masking | 15→4866（+4851） | 62→1489 | 58→8（残 8=hit-test/hittest 真败+idl 缺口） |
| css-transitions | 164→994（+830） | 496→537 | 63→41（残=事件/promise 族真败） |
| css-animations | 41→652（+611） | 393→650 | 74→32 |
| css-properties-values-api | 908→908（+0） | 463→463 | 37→37（非缺口页——单测真败，池③根因） |
| css-typed-om | 12000→12000（+0） | 406→406 | 8→8（同上） |
| **合计** | **13128→19420（+6292）** | | |

reconcile（ab-compare vs 修复前 TD b1）：**gained=6292 lost=0 regressed=0
pages_gain=95** 全域零回归；全文 results-corpus-repair-1120/
reconcile-vs-prerepair.txt。pva/typed-om 零位移＝其 wholesale 页（37+8）
系单测真败（custom-property 「A single animation is running expected 1
but got 0」等引擎缺口）非 support JS 缺失——预修复盘点 244 wholesale 中
~119 页 `is not defined` 归本缺口，余为真面。

修复后池基线（本 clone 首次可测）：池② mask-border **8 页 419F**＝
composition 93×3（composite-add 通道缺）+ width-interp 66/558 +
source-interp 42/294 + slice-interp 31/434 + outset-interp 1/168（插值
残面小头——B1 border-image 共享臂体已收插值主面）；与批三普查 ~419F
**精确对上**＝普查口径健康基实证。池④样区（animations
getAnimations/getComputedTiming/event-dispatch/KeyframeEffect 面）~152F
待认领时按页精盘。

## 口径纪律（后续轮次）

1. 后续刀电池哨兵腿自本批起按修复后基线判读；masking 哨兵恢复页级/
   子测级灵敏度。
2. 换 clone / 重拉语料先 tally：`find` 清单 md5 + support JS 在位检查
   （`ls wpt-data/css/support/*.js | wc -l` ≥9），续 [[wptdata-corpus-per-clone-drift]]。
3. 池② mask-border ~419F / 池④ WAAPI interface 簇 ~150F 本批起在本 clone
   可直接认领（此前页不可执行）。
