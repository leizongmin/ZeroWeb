# ⑦ exec 包裹形状：peel 非对称重设计 + br extraneous 重入断环（2026-11-05，乙续四）

## 背景（qv-fix4 轮遗留）

⑦ 残余「exec 包裹形状」族主刀：部分选中冲突祖先 peel/split 形
（`fo[o<font size=2>b]ar</font>baz`、`<span brown>fo[o</span><span blue>b]ar</span>`
等跨元素行）。fix3 版 `_zwVaPeelOne`（clone 保界设计）五语料
F2P 120 但 P2F 3（forecolor 跨元素 2 qV-after + multitest 1 旧二进制），
观察探针暴露 clone 丢失。

## 第一笔：peel 非对称重设计（part01d.js `_zwVaPeelOne` 重写）

### 桥根因（part04.js M 代理 insertBefore wire 分发实证）

wire 分支只接：`__zwSelector` 印解析元素 ref、`__zwHandle` 章戳 ref、
textEl 注册文本、plain 元素 ref（nodeType===1 D3 `_zwPosSelOf` 臂）——
**解析文本 ref（无印无章）全分支 miss 静默 no-op**（fix4 ar 丢失定位链：
`peel ins parent[...5 kids...] cloneIn=at2` 后 `peel done parent[...4 kids...]`，
insert 成功、`_zwJusMovePreservingRanges(clone, parent, 3)` 内部
`ref = newParent.childNodes[idx]` 解析到文本 ref 落空）。

### 设计（无边界重写——随动保界）

两侧机制不对称，各避开文本 ref 落点：

- **start 侧**：外段前缀 `[0..ci)` 移入 el 的属性克隆（createElement +
  attr 逐一 copy），克隆以 **ref=el** 插挂（解析元素 ref = sel 印臂，
  实证落地面）；子级后搬入（forceValue wrap 臂同款：先空挂后搬子）。
  start 端点容器 = 保留中段首子级 kids[ci]（不移动）→ 边界原值即正确。
- **end 侧**：**不克隆**——内段前缀 `[0..ci)` 裸搬出 el 到 parent
  （`_zwJusMovePreservingRanges` clearValue simple-modifiable 同款上移，
  全程桥实证面），el 原样保外段后缀 `[ci..len)` 属性。end 端点容器 =
  末搬出子级（随动 epInNode 臂保界）→ 边界原值即正确。
- 跨元素行文本切分沿用管线 pre-step boundary splitText
  （part01d :1228-1255）；中界复合位 `_zwVaEdgeComposite` 返 -1 →
  peel no-op。

### 验证

- 观察探针 zz-probe-observe（10 TREE + 3 QV，语料期望逐字）：13/13 PASS
  （P1-fs-t/f、P2-p-t/f、P3-plus1、P4-fc-t/f、P5-bc-t/f、P6-fn-t +
  qCV P1/P4/P5 三案）。
- 守卫探针 zz-probe-vawrap2 10 案：tA1/tA2（同元素双缘
  `<font size=1>foo[bar]baz</font>` 三兄弟 font 形）/tB/tC/tD/tE/tG/tH
  PASS（tF 旧期望按设计过期；tI 多属性 teal 面见残差段）。

## 第二笔：br extraneous 重入断环（part01c.js 两处）

peel 重设计后五语料 F2P 120 但 forecolor 2 案 P2F 挂起：
`<span style=color:brown>fo[o</span><span style=color:#0000ff>b]ar</span>`
qV-after 期望 `rgb(0, 0, 255)` 得空串。异常 stash 定位
（`__zw_ce_value_query` catch 临时插桩，已还原）：
**RangeError: Maximum call stack size exceeded**，栈帧恒一：

```
_zwJusNodeIndex ← _zwJusPrecedesLineBreak ← _zwJusIsExtraneousLineBreak
← _zwJusIsVisible ← _zwJusIsBlockStartPoint ←（ climbs → 重入）
```

### 环证明（part01c 源码逐行）

qV 走查 `_zwValGetAllEffectivelyContained(rng, _zwValIsFormattable)` 逐
document 序节点调 `_zwValIsFormattable` → 行内元素（含 br）走
`_zwJusFormattableVisible` :1808 `nodeType !== 3 → _zwJusIsVisible`：

1. `isVisible(br)` → br 臂 → `_zwJusIsExtraneousLineBreak(br)`
2. → `_zwJusPrecedesLineBreak(br)`：br 无子级 → 爬升
   `offset = NodeIndex(br)+1; n = parentNode`
3. → while 条件 `_zwJusIsBlockBoundaryPoint(parent, brIdx+1)`
   → `_zwJusIsBlockStartPoint` → childNodes[brIdx] 恰是该 br
   → `_zwJusIsVisible(br)` **重入 1**（无界互递归）

手建同形树不炸的定谳：brq 探针 rng 端界尾部无后继节点 →
`_zwJusNextNodeDescendants(endContainer)` 返 null → 走查 never reach br；
管线后树 sel 端界（#t:b,1）后仍有 ar 文本 → stop 非空 → br 入走查。
此前族内 qV-after 恒空串（catch 兜底返 ''）的 br 树行全为此环静默受害面
——forecolor 39F→23F、fontname 59F→6F、fontsize 78F→13F 的大面积
翻转皆因断环后 qV 走查首次走通。

### 修法（印记 + boundary 候选跳过，两处）

- `_zwJusIsExtraneousLineBreak`：in-flight 印记 `__zwExProbeInFlight`
  （重入返 false）；finally 清印记。
- `_zwJusIsBlockStartPoint`：in-flight br 不作 boundary 候选
  （视为不存在）→ precedes 爬升后回看更前节点取真实语义。
  逐字 trace（`o<br>bar` 形）：StartPoint(blue, 2) 跳 br → 假 →
  precedes 继续 `childNodes[2] = bar` 可见 → precedes=false →
  extraneous=false → br 可见 ✓（语义不变、递归有界）。
  `_zwJusIsBlockEndPoint` 无需同改（br 非块节点，其 br 臂恒 false 不断环）。

### 验证

- zz-probe-xel：qCV=`rgb(0, 0, 255)` ✓（两 swc 模式；异常 stash 还原后
  qerr=undefined 断无异常）；环探测 parent 链 #t→SPAN→DIV→DIV→BODY→
  HTML→#document 线性有限（探针 child-contains-ancestor 提示为
  祖先容器自然含子孙的启发误报，非环）。
- zz-probe-observe 13/13 保持。

## 语料判决（judge-fix5，对 fix3 基线行级元组差，分母逐位恒等）

| 语料 | 行数 | P | F（基线→现） | F→P | P→F | newF |
|---|---|---|---|---|---|---|
| fontsize | 2,422 | 2,409 | 78→13 | 65 | 0 | 0 |
| fontname | 2,225 | 2,219 | 59→6 | 53 | 0 | 0 |
| forecolor | 2,327 | 2,304 | 39→23 | 16 | 0 | 0 |
| backcolor | 1,026 | 1,020 | 12→6 | 6 | 0 | 0 |
| hilitecolor | 1,218 | 1,207 | 25→11 | 14 | 0 | 0 |
| multitest | 9,811→9,827 | 9,404→9,417 | 407→410 | 0 | 0 | 3（尾窗截断带新暴露行） |
| **五语料合计** | | | 213→59 | **154** | **0** | **0** |

- 五语料残差 **213F→59F**（-154）。forecolor 原两案 P2F（qV-after
  `<span brown>fo[o</span><span blue>b]ar</span>` 双 swc）已翻转回 P。
- 大面翻转机制：断环前任何含 br 树的 qV-after 走查均炸栈被 catch 吞成
  ''——154 案 F 中主体为该静默受害面（qV-after/indeterm-after 族）。
- fontsize 残余 13F 含多属性 teal 族
  `<span style="font-size:32px; background-color:rgb(0, 128, 128)">[a]bc</span>`
  ×2（语料要求非冲突属性保留于内段——end 侧裸搬不保背景色，
  prop-strip clone 设计挂后账）。

## 门禁

- make guarded-test 终核（test-guard 包裹，fix6-guarded-test2.log）：engine lib **3,727P/5F**——
  对上轮 3,724P/5F 的 +3P 系并行流补面测试，5F = s249 CSSOM 规则文本面已知红名单逐名恒一
  零新增（p56_adopted_snapshot_restore_hook_keeps_page_value /
  test_p99s_rule_style_preserves_custom_property_case / test_stylesheet_add_rule_remove_rule_r3276 /
  test_stylesheets_rule_style_r2810 / test_stylesheets_write_r2809）；fmt 门禁前置通过。
  make test 因该已知红 rc=101 中止 workspace 段（既有先例口径）。

## 第三笔：fix6 值查询修剪 + big/small UA 臂（part01c.js，语料中性）

- `_zwValIsFormattable` 类型阈值前置：非 text/img/br 直接 false，避免
  每个非文本节点走 FormattableVisible→isVisible 子树下沉（multitest
  30 分钟 0 行排查期的性能修剪；带 OPTIMIZATION 注释）。
- `_zwPlainFontSizePx` big/small UA 臂：父链爬取基准 px、legacy ±1 档、
  clamp 1..7（fontsize relative size 语料期望面）。
- `_zwValEffectiveValue` fontsize 16px 兜底：宿主读 ''/16px 时改读
  `__zwPlainComputed`，非空非 16px 才返回。

判决（judge-fix6 对 fix5 逐位比对）：五语料 fail 集合逐位恒等
（fontsize 2422 / fontname 2225 / forecolor 2327 / backcolor 1026 /
hilitecolor 1218 行），对 base 仍为 154 F→P / 0 P→F / 0 newF——
fix6 三笔在值语料面中性，保留。

## multitest 慢化排查定谳：三层伪象叠加（非 shim 回归）

现象链：fix4 全页 ~20 分钟 9,811 行；fix5/fix6 树 30 分钟 0 行。
逐层排除后三层定谳：

1. **行只在进程末统一打印**（main.rs `run_observers_cmd` 收集全部
   cases 后统一 println，testharness.rs:1626-1636）：全页 10 variant
   顺序跑 ~20 分钟 stdout 零字节属正常设计，此前被误读为挂死。
   variant 窗语义：subsetTest 的 currentSubTest 按子测试计数
   （~16.5 行/条目），?1-1000 = 前 ~60 条目全断言 + 全 628 条目
   非门控内联段（setup/前后查询/双 documentElement clone）；
   CASE_TIMEOUT_LONG=60s 只在 run_page_scripts 返回后 tick，拦不到
   同步内联段。
2. **并发争抢混叠历史测量**：bisect 期 10.8s/条目测量与
   `cargo build -p zero-browser` prebuild 并发（browser-prebuild.log
   02:35 vs bisect 02:26-02:35）；「30 分钟 0 行」的 run6 leg 亦为
   多 leg 并发期产物。
3. **外部 SIGTERM 误杀**（dmesg 铁证）：本轮独占全页跑 20 分钟静默期
   03:07:17/20 两次 `gdb -batch -p 1331056 -ex bt 20` ptrace 附着，
   03:07:52 runner 收 SIGTERM（"make: Terminated"）——非本会话操作，
   疑似并行 rally 实例僵尸清理误伤（零输出页被当卡死）。

健康实证（同树）：忠实复刻 probe 全 628 条目 pass 99.8s 无病态条目
（唯一 671ms @ i=566，styleWithCSS+delete+insertText 形）；真实页单
variant 裁剪 probe ?1-1000 全窗口 2.5 分钟出账 970P/30F（base 同窗
30F，数量一致），与 fix4 ~2 min/页节奏一致。

另证（非回归面）：qV 前向走查 stop 边界
（`_zwJusNextNodeDescendants(endContainer)`）不越宿主后继，#log
前置于 #test-container，走查不扫累积日志；runConformanceTest 每条目
双 documentElement.cloneNode 的 O(N²) 为 fix4 基线既有成本。

## multitest 出账通道定谳（乙续四续，撞车修订）：-one 多 variant 无吞案面

`testharness-editing-run-one`（FILTER 单文件，10 variant 单进程）两轮
独立实测（03:13 rc=2 / 03:21 rc=2，后者 `ZW_CORPUS_CASE_TIMEOUT_SECS=600`
仍同形）观察到的「每跑恰好一个 variant 完整出账 ~1000 行（full3 幸存
?1001-2000、mt7 幸存 ?5001-6000）」——**撞车修订：非 runner 吞案面，
是两实例并行竞跑伪象**。时间 forensic：03:09-03:30 本流（乙续四甲线）
正跑逐 variant 裁剪循环（每 ~2 分钟把 multitest.html 裁成单 meta 再跑），
上述两跑的枚举时点恰落在甲线 ?1001-2000 / ?5001-6000 单 meta 裁剪窗内
——runner 启动时读到的就是单 variant 语料，跑完即 exit rc=2，行集完整。
「吞案面」假设撤改（FIXME 撤销）；正面证据：甲线 full2（02:47-03:07）
单进程顺序推进 20 分钟（终因外部 SIGTERM 截断，见前节）+ 单 variant
probe ?1-1000 完整 1,000 行出账。

官方全量通道 = `run-editing-testharness-corpus.sh` 每案独立进程分片
（单进程全量 RSS 线性累积 + 同步段超时拦不住的既有注释）；本轮 fix6
multitest 判决实走**逐 variant 裁剪独立 `-one` 进程**（10 跑合并
9,000 行 + 尾窗 900s 预算补跑 827 行），与 base 行级元组对账
（judge-fix6）：**F 407→410，F2P 0 / P2F 0 / newF 3**——3 newF 全为
?9001-last 窗 base 截断点（813 行 + T）之后 base 从未执行、fix6 页完成
（827 行）才首跑的 forwarddelete 跨块 `<p><span>foo[]</span></p>
<p><span>bar</span></p>` 形 compare innerHTML 行，按 gate28 截断点漂移
flake 族先例记非行为回归（真实翻转面：0 P→F、0 F→P）。

复跑纪律（修订定稿）：① 全页 multitest 测量必须独占（无 cargo/make
并发，含**跨实例**——同 clone 双实例各自裁剪语料会互改枚举输入）；
② 长静默期先查进程树存活与 CPU 时间增长，勿 ptrace/SIGTERM；
③ multitest 出账一律逐 variant 独立进程（裁剪跑法或官方分片脚本）；
④ 语料裁剪后必须恢复 pristine（trap 兜底 + 人工核 10 meta）。

## 残差与后续

① end 侧 prop-strip clone（多属性 teal 面保留非冲突属性）；
② wrap br 插入面（管线树 `o<br>bar` 对语料 `obar`——compare innerHTML
残余族）；③ multitest 对账已收（407F 同位恒等零翻转 + 尾窗 3 行截断带
新暴露 fd 跨块 compare innerHTML 残余，归 ⑦ 骨架族后续）；④ S5 qV 空串族
~25 行、in_array 15 行挂 ⑨ 不变。
