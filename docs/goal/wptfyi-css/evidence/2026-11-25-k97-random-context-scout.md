# kc5r97 勘轮 verdict：random() 求值上下文接入刀位定谳

- 日期：2026-11-25
- 轮型：勘轮（残面再勘选刀位认领，不改码）
- 前置：kc5r96 已收束（origin/main 5d86efbb4，Pass 6749）
- 目标：从 css-values 语料 3133 失败中选下一刀位；重点勘 random 域残余 ~65 键

## 一、上游基线（wpt.fyi aligned，master+experimental）

抓取：`GET https://wpt.fyi/api/results?product=<b>&label=master&label=experimental&aligned&path=/css/css-values/random-computed.tentative.html&max-count=1`（302 → wptd summary_v2，2026-11-25 抓取；本树 .acceptance/wptfyi-css/k97/sum-*.json.gz 留档）。

| 浏览器 | 版本 | run | random-computed | 判读 |
|---|---|---|---|---|
| chrome | 160.0.8178.0 | 90d39060 | 169/177 | 近全过 |
| firefox | 163.0a1 | e00a7b14 | 173/177 | 近全过 |
| safari | 257 preview | 6d6d9ccd | 165/177 | 近全过 |

同 run chrome 全量 random 域页级：

| 页 | chrome | 判读 |
|---|---|---|
| random-in-animations.tentative | 37/37 | 全过 ⇒ 本树 24 红全可瞄准 |
| random-in-custom-function.tentative | 63/63 | 全过 ⇒ 本树 8 红全可瞄准 |
| random-in-if.tentative | 14/14 | 全过 ⇒ 本树 7 红全可瞄准 |
| random-invalid.tentative | 39/39 | 全过 |
| random-in-descriptors.tentative | 4/4 | 全过（挂账 descriptor 4 键） |
| random-in-media.tentative | 4/4 | 全过 |
| random-in-container-query.tentative | 9/9 | 全过 |
| random-serialize.tentative | 68/71 | 3 键 chrome 红＝候选预期性红 |
| random-item-computed | 5/37 | **chrome 自身红池** |
| random-item-valid | 1/21 | **chrome 自身红池** |
| random-item-serialize | 1/15 | **chrome 自身红池** |
| random-item-nested | 0/5 | **chrome 自身红池** |

判定 A：random()（非 item）域上游三浏览器全过或近全过 ⇒ 本树该域全部红键为合法 F2P 空间，无预期性红池。
判定 B：random-item 族为 chrome 自身未实现红池 ⇒ 排除出刀位候选（预期性红）。

## 二、毒化面证伪链（勘轮两假说均否证）

假说 1（hash 算法偏差）：WPT 硬编码期望暗示 spec 钉死 PRNG 算法，本地 sha256 键串与规范形有偏。
否证：css-values-5 §9 原文（drafts.csswg.org/css-values-5，2026-10-27 ED）——"The exact random-number generation method is **UA-defined**"；§9.4——"It is intentionally unspecified how the random cache name is used to achieve this. It can literally cache a generated random value, or be used as a seed for a PRNG, etc."。本地 random.rs 注释「UA-defined 方法——spec 只约束同 name 同值、异 name 独立」与规范一致；硬编码期望非 hash 派生产物。

假说 2（min/max 交换 swap）：勘轮由「期望值全是端点（9px/0.3/100px/'3 / 9'/'3px 6px 9px'）」推理实现做了 swap 而非塌缩。
否证：random.rs:1239 `let max = if max < min { min } else { max };` 已实现 spec §9.1 塌缩（"If the maximum value is less than the minimum value, it behaves as if it's equal to the minimum value"），:957 注释亦载明钳制恒返 min。端点期望值正是塌缩的确定性产物，与实现一致。模块头 :24 注释「求值期交换」为陈旧表述，与 :1239 矛盾——登记注释修正挂账（勘轮不改码）。

结论：毒化面＝**random() 作为 math function 在各求值上下文的接入缺口群**（类型上下文不支持 random() → 值保原文/拒收 → 断言不等），非 hash、非塌缩。

## 三、本树 14 红键分解（kc5r96 b1 battery，message 全列）

| # | 键 | message | 归类 |
|---|---|---|---|
| 1 | font-size 'random(30%, 10%)' | Random values should be equal | typed 上下文（% 叶解析） |
| 2 | font-size 'random(10px * 10% / 1%, 0%)' | is a supported value … false | parse 期拒绝 |
| 3 | --percent 'random(10%, 30%)' | ≥ 10 got NaN | typed custom property 求值缺口 |
| 4 | translate 'random(fixed random(-2, -1), 10%, 100%)' | should include 'fixed' | §9.4.1 fixed 替换串化 |
| 5 | background-image image-set calc(random(fixed 0.3, 0, 10) * 1x) | Random values should be equal | typed 上下文（image-set/resolution） |
| 6 | aspect-ratio 'random(3, 1) / random(9, 6)' | Random values should be equal | typed 上下文（ratio） |
| 7 | filter drop-shadow(random(3px,1px)…) | Random values should be equal | typed 上下文（filter） |
| 8 | font-palette palette-mix(in lch, --blue random(90%, 10%), …) | Random values should be equal | typed 上下文（font-palette） |
| 9 | rotate 'random(1, 0) random(1, 0) 1 90deg' | Random values should be equal | typed 上下文（rotate） |
| 10 | corner-shape superellipse(random(3, 1)) | is a supported value … false | supported 门（corner-shape 属性面） |
| 11 | offset-path ray(random(30deg, 10deg)) | Random values should be equal | typed 上下文（offset-path） |
| 12 | custom property random() inside transform() should not have 'fixed' | Random values should be equal | §9.4.1 untyped 求值时机语义 |
| 13 | random() percentages nested should have 'fixed' | (assert) false | §9.4.1 fixed 替换串化 |
| 14 | @property initial random() not allowed | Random values should be equal | invalid 检测 |

分组：typed 上下文接入 ~8（#1/5/6/7/8/9/11＋#2 parse）、fixed 串化语义 3（#4/12/13）、typed custom property 1（#3）、supported 2（#2/10）、invalid 1（#14）。

## 四、刀位结论

认领 **k98＝random() 求值上下文接入刀**：
- 主攻：random-computed 14 键（#6/#9/#7/#11 等纯数字/角度键最浅——parser 类型上下文放行 math function 即翻；#1/#3/#5 依赖属性求值上下文；#4/#12/#13 是 §9.4.1 computed 期「未简化完 → key 换 fixed BASE」串化语义）。
- 次攻（上游全过 ⇒ 全 F2P 空间）：random-in-animations 24、random-in-custom-function 8、random-in-if 7。
- 排除：random-item 族（chrome 红池）、random-serialize 3 键（候选预期性红，刀面触碰前先逐键核 chrome 状态）。
- k98 首步：探针页逐键实证 actual 值（battery message 无 actual，需断言复刻），按缺口深度排序定刀序（parse 放行层 → 求值层 → fixed 串化层）。

## 五、挂账登记

- random.rs:24 模块头注释「求值期交换」与 :1239 塌缩实现矛盾——注释修正（可并入 k98 face）。
- random-serialize 3 键：刀面前先核 chrome per-subtest 状态。
- corner-shape 属性面（#10）可能超出 random 域（属性未注册），独立勘。
