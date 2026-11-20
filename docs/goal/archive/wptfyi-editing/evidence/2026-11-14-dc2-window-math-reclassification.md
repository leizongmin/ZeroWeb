# DC-2 · d17 批：六文件残余 18F 窗口数学重分类（切片割裂伪失败 13F 定谳）

日期：2026-11-14。前置：d16 批（[strike REMOVE 方向规则 + u/s css-true 裸 span 吸收](2026-11-14-dc2-strike-direction-m1-absorb.md)）。
基线：face 1 六文件 18F（bold 8 / italic 3 / underline 3 / strike 4 / sub 0 / sup 0；
d16 记 17F 系减法笔误，见账目节勘误），哨兵 34 名集 + 2 页级 Timeout，
钉测 7/7。本批零引擎改动，face 计数不变。

## 结论先行

六文件 18F 全谱重分类完成（真值 = Chrome 录音 + harness 窗口数学）：
**伪失败 13F**（run 页 meta variant 切片割裂，harness 固有，任何浏览器同红，
不进修复队列）+ **真失败 5F**（strike S2e qS-after 1F + bold b-in-i 块边界
innerHTML 2F + italic i-in-b 块边界 innerHTML 2F）。d16 的"读数互为镜像
证实真实分叉"论据瓦解（:336 的镜像读数实为切片伪失败读 pre-exec DOM 的
巧合对仗），d13 对 underline 的分类大部分恢复成立，d13 遗留标签
"u-qS 条目 134 真失败"证伪（实为切片割裂 flag 泄漏断言）。

## 机制（窗口数学，d13 同款强化版）

run 页 `<meta name=variant>` 三/四窗切片（subset-tests.js：`currentSubTest`
逐 subsetTest 调用递增，窗口外**不创建 test**）+ tests.js runConformanceTest
结构（tests.js:5556）的组合效应：

1. 每条目子测数 = `ncmd + 2 + 6×查询键数`（execCommand 闭包 + 非编辑区
   检查 + innerHTML + 每查询键 6 断言）。
2. **execCommand 调用在 subsetTest 闭包内**（tests.js:5610-5615）——窗口外
   条目的命令**不执行**；before/after 读数在函数体内无条件执行
   （tests.js:5588-5599/5620-5629）。
3. 窗口起点割裂条目（窗口内首个子测偏移 > exec 偏移）的窗口内断言跑在
   **未执行命令的 DOM/flag** 上：qS-after 读数 = pre-exec 读数 = 录音
   stateB；qS(stylewithcss) before 读数 = 窗口起点 flag 泄漏链断供值。
   Chrome 在 wpt.fyi 同红（录音自证：同一 DOM+选区上 Chrome 的
   stateBefore 读数即 got，见下"录音自证"）。

各窗边界行号（0-idx 子测区间，`?N-last` = 序号 ≥N）：

| 文件 | 窗口 | 割裂条目 | 子测区间 | exec 偏移 | 窗口内首偏移 |
|---|---|---|---|---|---|
| strike | ?1001-2000 | 条目 65（row 64）`<strike>foo[bar]baz</strike>` sw=false | 989-1004 | 0-1 | 11 |
| strike | ?2001-last | 条目 133（row 132）`foo<s>[bar]</s>baz` sw=true | 1986-2001 | 0-1 | 14 |
| strike | ?2001-last | 条目 134（row 133）`foo<s>[bar]</s>baz` sw=false | 2002-2017 | 完整跑 | —（泄漏断言） |
| underline | ?1001-2000 | 条目 65（row 64）`foo<strike>[bar]</strike>baz` sw=false | 989-1004 | 0-1 | 11 |
| underline | ?2001-last | 条目 133（row 132）`foo<u>[bar]</u>baz` sw=true | 1986-2001 | 0-1 | 14 |
| underline | ?2001-last | 条目 134（row 133）`foo<u>[bar]</u>baz` sw=false | 2002-2017 | 完整跑 | —（泄漏断言） |
| bold | ?1001-2000 | 条目 75（row 74）`foo[<span bold>bar</span>]baz` | 995-1010 | 0-1 | 5 |
| bold | ?2001-3000 | 条目 139（row 138）`<span 900>foo[barbaz</span>}` | 1998-2006 | 0 | 2 |
| bold | ?3001-last | 条目 204（row 203）`<span 100>fo[o</span><span 200>b]ar</span>` | 2996-3011 | 0-1 | 4 |
| italic | ?2001-last | 条目 131（row 130）`<span italic>fo[o</span><span oblique>b]ar</span>` | 1989-2004 | 0-1 | 11 |

bold 共 4 窗（总子测 3048），italic 3 窗（2073），strike/underline 3 窗（2147）。

## 逐条定谳（12F 伪失败）

每条的 got = pre-exec 读数 = 录音自身 stateBefore（"录音自证"：伪失败断言
的期望 after 与录音 before 必不等——命令本会改变该状态——而窗口让
after 读数落在 before 状态上，Chrome 亦然）：

1. **strike ?1001 条目 65 qS(strike) after**（期望 false）：exec 跳过 →
   读 `<strike>foo[bar]baz</strike>` 原样，bar 在壳内 → got true = 录音
   stateB true。Chrome 读同一 DOM+选区 = true（录音 stateB 即证）。
2. **strike ?2001 条目 133 qS after**（期望 false）：同上（`foo<s>[bar]</s>baz`
   原样，bar 在壳内）。
3. **strike ?2001 条目 134 qS(swc) before**（期望 true）：全跑条目，但其
   before 读数依赖前条目（条目 133 sw=true）的 stylewithcss exec 泄漏；
   该 exec 被窗口跳过 → flag 恒 false（页首 bootstrap silly-hack 在
   Chrome/我方均为门控 no-op，d13 已定谳）→ got false。Chrome 同红。
4. **underline ?1001 条目 65 qS(underline) after**（期望 true）：exec 跳过
   → bar 在 `<strike>` 内无 underline → got false = 录音 stateB false。
5. **underline ?2001 条目 133 qS after**（期望 false）：`foo<u>[bar]</u>baz`
   原样 → got true = stateB true。
6. **underline ?2001 条目 134 qS(swc) before**（期望 true）：同 3。
7-12. **bold 6F**：条目 75（?1001）= swcB before + bold qS after；条目 139
   （?2001-3000）= compare innerHTML（exec 跳过 → 读输入原样 DOM，span
   未拆分）+ bold qS after；条目 204（?3001-last）= swcB before + bold
   qS after（混合 100/200 权重 ADD，期望 after true，pre-exec 读 false =
   录音 stateB false）。d13 条目号逐一对上。
13-... wait — **italic ?2001 条目 131 qS(italic) after**（期望 false）：
   `fo[o</span><span oblique>b]ar</span>` 原样，o 在 italic 壳内 → got
   true = stateB true。italic 伪失败共 1F。

合计伪失败 = strike 3 + underline 3 + bold 6（条目 75/139/204 各 2）+
italic 1 = **13F**；真失败 = strike S2e 1 + bold b-in-i 2 + italic i-in-b 2
= **5F**；13 + 5 = 18F。

**d16 账目勘误**：d16 批与 master.md 记"face 20F→17F"，实为 **20F→18F**
（d16 F2P=2：strike 5→4、underline 4→3，分解式 8+3+3+4+0+0 = 18）。本批
两次独立 face 实跑一致（strike+underline 7、bold 8、italic 3，sub/sup 0），
18F 为实测值。

## S2e（strike 条目 124，row 123）唯一 strike 真失败的证据边界

row 123 子测 1849-1857（序号 1850-1858）完整落于 ?1001-2000（exec 跑），
非割裂条目，qS-after 翻红为真实分叉：`foo<s>ba[r</s>b]az` REMOVE →
`foo<s>ba</s>rbaz`（innerHTML 绿），Chrome qS-after 录音 **true**，我方
spec 读序（含域 formattable 全命中）读 false。Chrome 的 after 选区不可
观测——gentest 录音期望串**无选中标记**（对照 row 128 `foo<s>ba</s>[r]baz`
标记齐全、row 0 塌缩 `[]` 有标记；标记缺型 = gentest 无法在该选区形态
回插标记），Chrome 塌缩位置与读向均无从核读。邻域选区形态不外推：
row 126/128 同为 REMOVE 抽段但 Chrome 保留 `[b]`/`[r]` 区间选区，row
122（ADD 面同界形）亦无标记——"界形→塌缩"规则从语料归纳不出单值映射，
按猜测修将不可证伪。处置：挂账 Chrome-quirk 家族（exec 后选区位移映射 +
塌缩 qS 语义分层），保留 d16 归档，不进修复队列，等下游真实选区录音类
证据（如 multiscript 录音或上游 gentest 选区序列化修复）再评估。

## d16 论据修正记录

- d16"读面家族归属"以 `:336 我 true/Chrome false` 与 S2e `我 false/Chrome
  true` 互为镜像为同族证据——本批证明 :336（row 64）是割裂伪失败（got
  true = pre-exec 读数），镜像系巧合对仗，非同族机制。
- d16"解除 d13 待复核标签"中恢复的"strike 4F 全 qS 读面族"收窄为 S2e 1F。
- master.md 真失败账"u-qS 条目 134"由真改伪（条目 134 = row 133，其失败
  断言 swcB-before 是泄漏链断供伪失败；d13 原标注承自 fail 名串含
  "u-qS" 误读，实为 swc 读数）。

## 账目

- face 1 六文件：18F 不变（bold 8 / italic 3 / underline 3 / strike 4 /
  sub 0 / sup 0），本批零引擎改动。伪失败 13F 出修复队列，真失败 5F
  （S2e 1 + bold b-in-i 2 + italic i-in-b 2）。
- 哨兵电池/钉测：不适用（零引擎改动，未重跑）。
- 诊断探针 zz-probe-d17qs/d17seq/d17face（上轮 429 中断前遗留，未入库）：
  d17seq 全谱 replay 证实除 row 123 qSa 外六文件邻域 qS 读数全对齐；
  d17qs 证实 `foo<s>[bar]</s>baz` REMOVE 孤立跑 qS=false = 录音期望
  （窗口内翻红来自 exec 跳过非读面）；诊断完成已清理（d13 探针先例）。
- wpt.fyi 交叉核对尝试：per-test API 重定向到分片 summary（GCS 对象仅
  3 条 /dom/ 测试，shard 索引形态），search API 对该 run 返 0 条——上游
  逐子测结果不可达；改用录音自证（伪失败断言的期望-after 与录音-stateBefore
  必不等 + 窗口数学确定性），证明力等同 d13"wpt.fyi 同红"。
