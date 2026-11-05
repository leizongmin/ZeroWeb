# ③'-a/b/d-apply 同窗：forecolor 五 compare 行等价分组主刀与门禁终账

日期：2026-11-05　提交：2cc95aaac（③' 主刀，rebase 前 539200c21，part01d.js +187/−26）+ e29b3a8bc（numericFace 收口，part01d.js +20/−13 + qv5/ab 等探针入库常驻）　基线：a04dd1522（⑥ 后、③' 前）
窗口：控制面 ③'-a/b/d-apply 同窗——forecolor a 分裂 ×2 + b 载体换形 ×1 + d 幽灵 br ×2

## 1. 目标与语料期望形

五个 compare 行（corpus forecolor.js，行号 = data 行号）在 2017 Chrome 录制下的期望形：

| 行 | fixture | sw | 期望形 |
|---|---|---|---|
| :376 | `<span style="color: blue">foo<span style="color: brown">[bar]</span>baz</span>` | true | 双形皆可（keyword outer 可接受） |
| :382 | 同上 | false | `foo<span style="color: blue">[bar]</span>baz`（仅切分，无壳改写） |
| :435 | `<span style="color: rgb(0, 0, 255)">foo<span style="color: brown">b[ar]</span>baz</span>` | true | span 载体延伸 carry |
| :441 | 同上 | false | `<font color="#0000ff">` 载体换形（span carry → font attr） |
| :727 | `<font color=brown>fo[o</font><span style=color:brown>b]ar</span>` | true | keyword 保留 + 跨标签接续 |
| :732 | 同上 | false | 同上（tag 面） |
| :737 | `<span style=color:brown>fo[o</span><span style=color:#0000ff>b]ar</span>` | true | 纯形接续 |
| :742 | 同上 | false | 幽灵 br 不产生（text-text 交界禁 br） |

CSSOM 形前提（zz-probe-cssom 实证）：本仓 CSSOM 与 Chrome 一致保留 authored
keyword（gPV("blue")="blue"），hex 原文保留（gPV("#0000ff")="#0000ff"）；
vendored normalizeColor 对 keyword 失真（brown→rgb(0, 0, 0)），故 harness
normalizeSerializedStyle 对 keyword 不可依赖，等价判定须引擎侧自带。

## 2. 九刀实现（part01d.js）

1. `_zwVaNormalizeStyleAttrCss(raw, preserveColorKeywords)` 第二参：color/
   background-color decl 的 `_zwVaChromeColorKey(pv) === pv` 时保留原文；
   clone 路径默认 falsy → 仍走 rgb 归一。
2. `_zwVaChromeColorKey(v)`：`_zwPlainNamedColor` 命中 → 小写 keyword；
   否则 `_zwVaNormalizeCssColor`。Chrome 语义 keyword→keyword、hex→hex。
3. `_zwVaEquivalentValues` color 臂：**仅 forecolor 且非 cssFlag** 走
   chrome-keyword 比较（tag 面 keyword 敏感）；backcolor/hilitecolor 不变
   （语料 :193 keyword aqua merge → bg 面 keyword 盲，不可外溢）。
4. `_zwVaSpecifiedValue` font color attr：数字色走 normalizeCssColor，
   失败保留原文（attr "blue" ≡ #0000FF 合并，语料 :419/:425 行为）。
5. `_zwVaCanonicalizeAncestorStyleAttrs`：normKw（保留形）/normPlain 双归
   一，祖先 attr 落 normKw；normKw ≠ normPlain 的元素推入
   `_zwVaKwPreservePending`。
6. 模块级 `_zwVaKwPreservePending` 队列（紧邻 `_zwVaPeelRestore`）。
7. `_zwVaSetSelectionValue` 尾部（peel-restore 后）：pending 元素以非保留
   形自归一（parentNode 守卫），清队列——修 d-727/732 基线 rgb 形（该 rgb
   来自 canonicalize 原地改写而非 clone，keyword 保留刀曾使其破形）。
8. `_zwVaRematerializeCarries` 新 tag 面臂：span 载体 + 纯单 decl
   （style 单属性仅 color）+ 嵌套祖先 specified 色可解析 → 换形
   `<font color="#hex">`（`_zwVaShiftEndpointsForInsert` +
   movePreserving）。cssFlag 改局部变量。
9. `_zwVaWrap` 跨标签接续：`crossTagBlock = forecolor && !cssFlag`；两个
   reuse 臂加 `!(crossTagBlock && _zwJusTag(sibling) !== 'font')` 门；臂1
   else + 臂3 br 条件加 text-text 交界禁 br；臂3 `crossTagFont` 刻痕
   （单 text-text move、无删除、无 br）；`_zwVaForceValue` wrap 调用传
   command；新 `_zwVaCrossTagContinuation`（forceValue 尾 bare-insert 后
   单 move 接续，四门：forecolor + 非 cssFlag + 邻元素 editable +
   交界两端皆 text）。

## 3. 钉测（zz-probe-qv5.html，gitignored 本地语料）

八行 got 逐字对齐语料期望形（探针 test 名内嵌 got 串，runner Pass 行直读）：
a-376 / a-382 / b-435 / b-441 / d-727 / d-732 / d-737 / d-742 全数命中。
机制与语料 setupDiv/runCase 同款（tests.js :5283/:5637）。

## 4. 门禁终账

五语料（make testharness-editing-run-one，TIME_LIMIT=3600；q5 = 539200c21
arms 二进制，q6 = +numericFace e29b3a8bc 二进制）：

| 语料 | 基线 F（fix15a*） | q5 F | q6 F | 判读 |
|---|---|---|---|---|
| forecolor | 9 | **7** | **5** | 4 目标 compare F2P（q5）+ 裸 keyword :113/:268 F2P（q6，numericFace 收口）；余 5 = 既有读数面/嵌套壳族零新红 |
| backcolor | 0 | 0 | **0** | 零位移 |
| hilitecolor | 0 | 0 | **0** | 零位移 |
| fontsize | 5 | **6** | **6** | +1 新红 = font1-straddle 行（q5/q6 fail 集逐字节恒一），归因见 §5.3 |
| fontname | 3 | 3 | **3** | 已知 variant 切割恒一，零位移 |

*fix15a 直腿基线（14:16-14:40）早于 M3 682f0632d / ⑥ a04dd1522，仅作
直腿参照；元组权威判读 = multitest 行级 vs jccut 基线（5ad2d49cf 引擎）。

multitest 10 片行级元组（judge-q5.py，正则含 `?V-V` 窗口后缀修正；
基线 jccut-mt-part = 5ad2d49cf 引擎新鲜基线 10,077 行）：

- q5 批（arms，片 1-4 有效 + 净树重跑 5-10 全账）：**F2P=0 / P2F=0 /
  newF=1 / lost=1**——newF = forwarddelete `"<p>abc[]<br> </p> <p><br></p>"`
  尾带截断差分行（基线 ?9001-last 1,077 行截断未达、新鲜 1,092 行方达；
  该行属 part01c delete 面，part01d value-command 臂结构性不可达），
  lost = 基线 Timeout 行被完走治愈；双独立验证轮（1d32a3a5/rollover）对账同判。
- q6 批（+numericFace，全 10 片，二进制 grep -a -c numericFace=6 执行体
  证据）：片 1-9 fail 数逐值恒基线（30/24/34/24/50/49/59/47/14），
  全 10 片 **F2P=0 / P2F=0 / newF=0 / lost=1**（fresh 10,078 行 vs base
  10,077 行；lost = 基线 ?9001-last Timeout 行被完走治愈——本轮尾片
  1,078 行对基线截断 1,077 行、q5 重跑 1,092 行，均在 ⑪ 尾带截断漂移带；
  零行为翻转）。
- 门禁批基建竞态（首轮 slice-5 假挂）：trap pristine 恢复与 slice-5 runner
  启动同秒竞态，runner 读到全量语料烧满 1200s guard 上限、行缓冲未刷出
  （0 行假象；进程采样 102% CPU 全窗佐证）——净树重跑 128s/50F 与
  numericFace 二进制 q6 复跑 107s/50F 双臂证 arms/numericFace 无罪，
  记基建伪影不记行为。

enginelib（make guarded-test，净树重跑）：**3,740P / 5F**——五红名恒一
（p56/p99s/r3276/r2810/r2809，与 ⑥ 后 final-verify 记录一致）。

## 5. 三疑似新红归因

三行均 direct-leg-only（multitest.js 自有行集，三行皆不在其中）：

1. forecolor :113 裸 `blue` tag 面（期望 `foo<font color="#0000ff">[bar]</font>baz`）
2. forecolor :268 裸 `cornsilk` tag 面（期望 `foo<font color="#fff8dc">[bar]</font>baz`）
3. fontsize :438 `<font size=4>foo<font size=1>b[a]r</font>baz</font>` straddle

### 5.1 门禁批基建竞态（与归因无关但须记实）

首轮门禁批死于批杀拆卸竞态：20:25:56 trap 恢复 pristine 全量语料与
slice-5 runner 启动同秒发生（并行 numericFace 轮 20:25:57 并发 make 撞窗），
runner 读到 ~10× 工作量全量语料烧满 1200s guard 上限、行缓冲未刷出
（0 行日志假象；1d32a3a5 轮 ps 采样 102% CPU 全窗佐证）。
净树重跑 slice-5 = 128s/1000 行/50F = 基线恒一——arms 无罪、批健康、
"numericFace 死循环"假设不成立（草稿须自跑门禁另证）。

### 5.2 裸 keyword 两行（:113/:268）——根因已钉死

根因归我臂：chrome-keyword 比较刀越界到 effective/attr 面。机制：
`_zwVaForceValue` 建 fresh `<font>` 载体后，仅当 effective 值检查**不**等值
才补写 style 声明——改刀前 attr `#0000ff` vs 新值 `blue` 数值等值 → 跳过
style → attr-only 形（:114/:269 期望）；改刀后 chrome-keyway 使
"blue" ≠ "rgb(0, 0, 255)" → 补写冗余 `style="color: …"` → 形破行红。
语料要求 keyword 敏感仅属指定形（CSSOM 保留 authored 形），有效值/attr
面恒数值等值。

处置：numericFace 收口已落地 e29b3a8bc（part01d.js +20/−13，
`_zwVaEquivalentValues`/`_zwVaLooselyEquivalent` 增第四参，全部 9 处
effective 面调用传 numericFace=true 走 `_zwVaNormalizeColorKey` 数值键，
specified 面 6 处保持 keyword 敏感——逐 hunk 审定调用点分面无一遗漏；
zz-probe-qv5/zz-probe-ab 探针随 commit 入库常驻）。q6 批实证：forecolor
7F→5F（:113/:268 F2P），qv5 探针 q114/q269 got 形变 attr-only
（`foo<font color="#0000ff">bar</font>baz` / cornsilk #fff8dc），与语料
期望形逐字一致；fontsize/fontname fail 集对 q5 逐字节恒一（numericFace
不触非 color 面）。

### 5.3 fontsize straddle（:438）——③' arms 回归实锤，已修（q7 终批确认）

zz-probe-ab A/B 实测（worktree a04dd1522 pre-arms vs 主树 arms+numericFace，
test-guard 包裹二进制）：

- **pre-arms got** = `<font size="4">foo<font size="1">b</font>a<font
  size="1">r</font>baz</font>` —— 与语料期望形逐字一致（arms 前绿）。
- **post-arms got** = `<font size="4">foo<span style="font-size:x-small">b</span>a<span
  style="font-size:x-small">r</span>baz</font>` —— 内层 font 壳被换形 CSS
  span（arms 后红）。fc-blue/cornsilk 两案 pre/post 均 attr-only 期望形
  （numericFace 交叉印证）；fd 截断行两臂同族形（继承性印证）。

根因（非 forecolor 门泄漏，而是早退丢失）：③'-b 主刀把
`_zwVaRematerializeCarries` 函数首 `if (!_zwVaCssFlag()) return;` 改为
cssFlag 局部变量 + 新 forecolor span 臂后，**tag 模式 font 载体
（fontsize/fontname/forecolor carry）落穿 `if (cssFlag && span) continue;`
直入下方 ⑨ css flag-form 重物化臂**——tag 模式被换形
`<span style="font-size:…">` CSS 形。逐臂走查预判 no-op 与表象矛盾的
缺口正在此处（走查只看了新增臂的门，没看被抽走的早退对旧臂的覆盖）。

修复（一行）：`if (cssFlag && span) continue;` 后补 `if (!cssFlag)
continue;`——镜像改前整函数早退语义；forecolor tag span 载体由上方
③'-b 臂消费后 `continue` 不及此，③' 五目标行不受影响。q7 终批
（run-q7-fsfix.sh：qv5/ab 探针 + 五语料 + mt 10 片 + enginelib）确认。

## 6. 判读与挂账

**③'-a/b/d-apply 同窗收口（含 numericFace 收口 + fs :438 回归修复）**：

- 五 compare 目标行全数落账：a-382/b-441/d-737/d-742 于 q5 F2P（539200c21
  三刀），裸 keyword :113/:268 于 q6 F2P（numericFace e29b3a8bc）；a-376/
  b-435/d-727/d-732 既有通过面保持。forecolor 语料 9F→5F（余 5 = 既有
  读数面/嵌套壳族：table qV、brown straddle qI/qV、嵌套壳 compare、
  leak-cut 伪影——零新红）。
- **红线事件与闭环**：fontsize :438 straddle 系 ③'-b 抽走
  `_zwVaRematerializeCarries` tag 模式早退的回归（§5.3 A/B 实锤，
  pre-arms 绿→arms 红 P2F 定性）——一行修复补回停臂语义，q7 终批全量
  门禁确认（qv5 十钉 + ab 四案 + 五语料 + mt 10 片 + enginelib，
  行级元组对 jccut 基线 P2F/newF=0 为放行门）。
- multitest 全分母（q5/q6 双批）行级元组零位移（§4）；enginelib
  3,740P/5F 五红名恒一。multitest ?9001-last 尾带 1,077-1,092 行漂移
  带维持 ⑨/⑪ 伪影口径（31 fail 骨架 flake 族记账不变）。
- 门禁批基建教训两条入账：①共享账本行首锚定 marker 会被 NOTE 正文
  误配（watcher 锚 `^`）；②worktree 复刻 v8 构建需 RUSTY_V8_ARCHIVE
  为**文件**（build.rs copy_archive 直接 open 该路径，目录→IsADirectory
  panic；主树 gn_out/obj/librusty_v8.a 可直接 cp）。
- 挂账不变：③' 存活读数面（forecolor 余 5 之行 1/3/4/5 族）、
  ④ fontsize qI-after ×2、J-B table 16F、J-C 跨壳 br ~44F、
  part01c.js 2,275 行超限拆分、F1/F2 大项。

## 7. q7 终批终账（执行轮补记，22:23:53 ALL-DONE）

- **执行体证据**：bin-mtime 1793886995 > src-mtime 1793886901 +
  `grep -a -c 'fs 收口' target/release/zero-wpt-runner` = 1（修复标记在执行体内）。
- **探针**：qv5 十钉 rows=10 fails=0；ab 四案 4/4 Pass——fs-font1-straddle
  got=`<font size="4">foo<font size="1">b</font>a<font size="1">r</font>baz</font>`
  与语料期望形逐字一致（§5.3 修复实证）。
- **五语料**：fontsize 2,422 行/**5F**（:438 straddle 双变体 Pass，fail 集 =
  q6 −straddle 行其余逐字节恒一，6F→5F）；forecolor 2,327 行/**5F**（q6 逐字节
  恒一，目标行保持）；backcolor 1,026 行/**0F**、hilitecolor 1,218 行/**0F**、
  fontname 2,225 行/**3F**（恒一，已知 variant 切割伪象）。
- **multitest 10 片行级元组**（judge vs jccut 基线 10,077 行）：fresh 10,092 行
  （?9001-last 完整出账 1,092 行 solo 恒等、零截断），**F2P=0 / P2F=0 / newF=1 /
  lost=1**——与 q5 批签名完全一致：newF = forwarddelete
  `"<p>abc[]<br> </p> <p><br></p>"` 尾带截断差分行（part01c delete 面，part01d
  value-command 臂结构性不可达，q5 已定谳入档）；lost = 基线 ?9001-last Timeout
  行被完走治愈（改善向）。片 1-9 fail 数 30/24/34/24/50/49/59/47/14 对基线逐值
  恒一；片 1-6000 逐键零位移复核通过。
- **enginelib**（make guarded-test，116s）：**3,740P / 5F**——五红名
  p56/p99s/r2810/r3276/r2809 逐名恒一零新红，clippy 0 error。
- **判读**：③'-a/b/d-apply 同窗 + numericFace 收口 + fs tag-mode 停臂修复
  **全账放行**。落地链 = 2cc95aaac（③' 九刀）+ e29b3a8bc（numericFace
  收口）+ fs 停臂（本提交，q7 门禁批验证的修复笔）。
