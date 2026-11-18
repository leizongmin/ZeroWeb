# 刀 TS：transition 简写 CSSOM 车道（写面展开落 longhand + 读面 canonical 重组）

日期：2026-11-18。认领：CLAIM-knife-ts-shorthand-cssom.md（23:0x 落盘）；
执行窗：刀 TP 落库 `cdff181b9` 后同流接续。

## 池与根因（探针地面真值）

- css-transitions/parsing/transition-shorthand.html **0P/18F**（三形全灭，
  共同因子 `cubic-bezier(0, -2, 1, 3)` 负 y 控制点）；
- css-transitions/parsing/transition-behavior.html 23F（canonical 序重排 +
  `asdf display` 类非法形未拒 + supported 串比对，全走简写车道）；
- css-transitions/transition-behavior.html 5F＝引擎离散转换行为面，非写门
  车道，不入本刀。

单页探针（zz-probe-tshort，b1-runner 即 TP 后 HEAD 态）：**连平凡合法形
`display 3s` 五 longhand 读回全空、简写读回 raw 原文**——CSSOM
setProperty/named setter 车道对 `transition` 简写 raw 直存不展开；inline
attr 车道（引擎级联下游展开）不受影响。`parse_timing_function` 负 y 嫌疑
证伪（仅 x 域门，符合 easing-1 spec）；`split_outside_parens` 括号感知
核过无缺陷。

## 面与机制（css_style_declaration.rs 单文件七钩）

- 写面展开臂 ×2（named setter / setProperty）：`ts_expand_decls` 走
  style-system `expand_shorthands` 单源（gap-rule 刀 B 同款形态）——展开
  落 5 longhand、简写名与其所控 longhand 旧声明一并让位；展开空（非法形
  /重复组件）→ 声明忽略（spec setProperty 语义）。`asdf display` 的
  "asdf"/"display" 双 property 组件在 parse_single_transition 已拒。
- 读面派生 ×2（named getter / getPropertyValue）：`ts_family_value`——
  简写名 canonical 重组（raw 存储形优先展开同形、css-wide keyword 原样）；
  longhand 名直查优先、本名落空且简写声明在时按层序展开提取。
- canonical 重组 `ts_serialize_from_longhands`（css-transitions-1
  §serializing-shorthand-values）：每层 `<property> <duration> <timing>
  <delay> <behavior>`，等于初值的组件省略，例外 delay 非零时 duration
  即使 0s 也强制输出（消歧两位 `<time>`）；列表下标取模环绕（等长名单
  即逐层直取，spec LCM 展开最小实现）。
- 空值复位镜像 ×2（R3211 空值臂）+ removeProperty 覆盖镜像（旧值重组形
  返回 + longhand 一并移除）。
- parse_style/inline attr 车道零触碰（raw 存储形态经读面派生兼容；
  cssText raw 往返不受扰——语料 cssText+transition 断言暴露≈零：
  仅 crashtest 与 @starting-style 规则解析页）。

## 钉测（23:1x）

`knife_ts_shorthand_serialize_verdicts` + `knife_ts_family_value_verdicts`
（期望串逐形锚定 WPT F 消息——canonical 六形 / 层提取五值 / 直查优先 /
非法拒 / css-wide 原样）**首执即绿**（2 passed 0 failed，4206 filtered）
＝DOA 预防闭环连续第三例。fmt CLEAN（rustfmt 收敛两笔）+ 钉测编译窗
内 peer engine WIP（animation/clip_interp）同树编译通。

## 电池（23:15 发射，账后回填）

systemd unit `zw-knife-ts-battery-2315`；knife-ts-battery.sh（knife-tp
配方逐字抄：b0=HEAD `cdff181b9` face / b1=HEAD+ts-face.patch 双建 + M2
md5 异同性证 + ride-along s1/s2——在飞他流件 parse_basic_shape/
parse_extended_border/animation/clip_interp/es_module/part115 六文件
双侧同态）。腿：css-transitions（主靶二页 + 全域 P2F 面）× b0/b1 +
css-properties-values-api（哨兵）× b0/b1。预期 +41F 方向（18+23），
P2F 零容忍。

### v1 账（01:15 回填；session 429 中断后 resume 收账）

- **账面：F2P=0 / P2F=0 / net=0，两语料四腿全零**（recount-knife-ts.py
  独立复算：css-transitions keys b0=1531/b1=1532、pva b0=b1=1371；
  b0-ret.json 与 b0.rerun.json map-identical 双源一致）。
- **主靶页 b1 逐面核**：transition-shorthand.html 18F 原态、
  transition-behavior.html 23F 原态——face 在 b1 二进制（M2 md5 异、
  b1 构建重编译 zero-engine）但**零位移**。
- **根因（resume 后静态面定谳）**：v1 face 七钩全落 **V8 native 模板
  车道**（`native_style_named_setter` / `native_style_set_property_invoke`
  等——ZeroBrowser 产品通道）；runner WPT 语料 `el.style[prop]=v` 走
  **shim inline 通道**（part03.js T50/T54 臂 → `__zw_style_shorthand_
  expand` / `__zw_style_shorthand_reconstruct` / `__zw_css_supports`
  宿主回调，gap/font 刀先例同站）——两通道零交集，face 对 runner 语料
  完全 inert。CLAIM 前置探针"raw 直存"地面真值正确，修法接错车道；
  钉测首执即绿（helper 单元级）与 WPT 零位移并存＝车道错接的典型签名。
- **b0 崩溃事故**：css-transitions b0 首腿 free(): invalid pointer
  零字节 JSON（23:21）；同 bin 重射 unit `zw-knife-ts-b0-rerun-2332`
  23:32→23:39 干净收账（b0-ret.json）。v1 b0/b1 账以 rerun+b1 为准；
  崩溃未复现、疑多电池并发内存峰（unit 峰值 2.8G）瞬态。
- **基线交叉核**：knife-ts b0 vs knife-tp b0 map-diff=577——其中
  transition-property-invalid 15 项 tp=fail/ts=pass＝已落库刀 TP face
  自身账（方向一致，非本刀）；properties-value-inherit-002 等整页
  缺失＝CASE_WALL_S=60 页级截断（keys 1531 vs 2091），b0/b1 两侧
  截断集一致（GONE=0/NEW=0）不扰 A/B。
- **结论**：v1 face 不弃（ts_* 助手与钉测为 Round A 底座），Round A
  重接三站（expand_json / reconstruct_from_style / decl_supported），
  电池 v2 重射。

## Round A（01:19-01:4x，2026-11-19 窗）

面（本流独占两文件 diff，单变量 A/B）：

- `css_style_declaration.rs`：`transition_shorthand_expand_json`
  （空值→名单逐名清 / 非法形→`[["transition",""]]` 自清形＝声明忽略
  净效果 / 合法→名单 longhand canonical JSON）+ gap 站分派臂 +
  `gap_rule_shorthand_reconstruct_from_style` transition 臂
  （`style_attr_decls` 共享解析抽出）+ V8 native 七钩原样保留
  （产品通道 parity）+ 钉测扩 4（expand_json / reconstruct_from_style
  两新站 + 原两钉）**首执即绿**（4 passed 0 failed，01:2x test-guard
  包裹实跑）。
- `js_dom_bridge.rs`：`decl_supported` transition 臂（font 臂同型——
  known_properties 只列长手，简写支持性 = expand_shorthands 非空，
  matcher `shorthand_supported` 同源 oracle）。
- 探针页 zz-probe-tshort2（runner 链验证，01:3x）：expand/readback/
  invalid/length/supports 五组全 Pass、computed 臂按预期 Fail（Round B
  面）；跑后即删（存档 .acceptance/wptfyi-css/zz-probe-tshort2-saved.html）。
- 电池 v2：unit `zw-knife-ts2-battery-0140`，knife-ts2-battery.sh
  （v1 配方承继 + FACE 扩两文件 + ride-along 十九件法证）；预期
  css-transitions **+30F** 方向（shorthand 页 15 + behavior 页 set 12 +
  unrelated supports 3；computed 臂 11F 属 Round B 不在本账），哨兵
  pva + css-transitions 全域 P2F 零容忍。

### 电池 v2 账（02:05 回填）

- 双建 M2 异（b0=`5572171b` / b1=`e054e317`），S2 ride-along 十九件窗内零
  扰动；四腿 rc=1 干净收（01:41-01:58），TRAP-CLEAN 零残留。
- **css-transitions：F2P=54 / P2F=3 / NEW-F=1 / net=51**；哨兵 pva 四零净；
  v2-b0 vs v1-b0(rerun) 交叉核 map-diff=1（color-mix 截断变异，见下）。
- 分账：双靶页全翻——transition-shorthand.html **18/18**、
  transition-behavior.html **23/23**；红利 F2P=13（transition-computed.html
  8＝test_computed_value 的 supports 门+简写 set+computed 读链、
  transition-invalid.html 4＝expand 站拒收、transition-valid.html 1）。
  旧"computed 臂 11F 属 Round B"分账**作废**——computed 面已随三站入账，
  Round B（`transition_shorthand_to_css` 修形）待 v3 后重新定界或撤销。
- **P2F=3 单机制**（transition-valid.html 部分组件形）：
  `ts_serialize_from_longhands` 恒发 property 组件——`"1s"`→`"all 1s"`、
  bezier→`"all bezier"`、`"1s -3s"`→`"all 1s -3s"`，违 css-transitions-1
  canonical 初值省略规则；其余四组件省略已对（behavior 页全翻佐证）。
- NEW-F=1（animate-with-color-mix.html color-mix 三件套计 1 键）＝**预存红
  截断暴露**：v1-b0/v1-b1 双侧 fail 实证（v1-b1 四件全 Fail），v2-b0 页级
  截断（keys=3）缺席——不入本账。
- v2.1 修正（02:03-02:05）：初值省略臂（property=`all` 同省、全层初值保
  `all`——空串非合法 `<single-transition>`）+ parts/join 防前导空格（首枚
  新钉抓红 `" 1s"`）+ 钉测扩 6 断言（knife_ts **4 passed 0 failed**
  test-guard 实证）。
- 电池 v3（02:16 ts3/ts2b 双射碰撞后收敛，账 of record = ts2b 四腿
  `zw-knife-ts2b-battery-0216`，02:17:26→02:34:32 GATE-DONE 四腿 rc 干净
  TRAP-CLEAN rc=0）：**F2P=57 / P2F=0 / net=+57，VERDICT GREEN**（零容忍
  兑现）。逐页＝shorthand 18 + behavior 23 + computed 8 + invalid 5
  （+1＝`initial 1s` css-wide 门兑现，b0 raw 收实证）+ valid 3（+2＝omit-all
  兑现：`1s`/`cubic-bezier(0, -2, 1, 3)` computed 读回 canonical）。A/B
  provenance：b0=`5572171b`（HEAD face、无 expand 站分派臂）/b1=`54ab4fbb`
  （v2.1 face + transition.rs 门）——Text-file-busy 碰撞使终态文件对恰为
  全账 A/B（ts3 原 b0 文件失已法证存底 NOTES 02:20）。transition.rs 效应
  只在 b1 expand 站显形＝attributable 非 ride-along 相消。双工具终态互证：
  ts4 账口径 57/0 CROSS-CHECK GREEN（剔 probe/color-mix/cancel-003）、ts3
  raw 59＝57+探针页 2 键翻转（b0 腿跑在探针页创建后，readback 失败即 P2F
  bug instrument 证明，剔后恰 57）+GONE=1 color-mix 截断伪影；pva 哨兵
  0/0/0；xcheck v3-b1 vs v2-b1 diff=10＝6 WPT 键（css-wide 门 + omit-all
  5 形）+4 探针键全预期。三重独立互证：ts3-b1 快照初账 57＝b1v2 重射 57＝
  ts2b 终态 57。探针页 zz-probe-p2f.html 全停后已删（存档
  `.acceptance/wptfyi-css/zz-probe-p2f-saved.html`）。

### b1v2 重射账（02:1x 汇流笔——先行 face 实测，v3 归因关键输入）

另一 fork 会话（本流 01:40 电池发射者，CLAIM 追记同窗）以 02:12 树态
（= 上述 v2.1 修形 + ts_expand_decls css-wide 门 + transition.rs parser
门 + 钉测/fmt）重建 b1v2（bin md5 `397fb39d`）重射 css-transitions b1 腿
（b0 复用 v2 存档 bin，ride-along md5 零漂移复核）：

- **F2P=57 / P2F=1 / net=+56**：3 P2F 全数回绿（omit-all 生效），靶页
  valid 10/10、invalid 5/5、shorthand 18/18、behavior(parsing) 28/28 全 P；
  F2P=57 与 v3 预期数吻合。
- **P2F=1 = transitioncancel-003.html**（"Final width should be 200px
  but got 100px"——旧参数 transition A 跑完至 100px、width→200px 新参数
  transition B 未启动签名）；**solo 单页对拍定谳真回归**（b1v2 fail /
  b1(01:40) pass，无负载因素）。五腿追踪：v1-b0/v1-b1/v2-b0/v2-b1 全
  pass，唯 b1v2 fail——增量锁定 02:04-02:12 树态四处（omit-all / 门 /
  transition.rs parser 门 02:04 / 钉测 fmt）；三者静态码路均够不到该
  测试写路径（"width 0s 5000ms" 无 css-wide token）。
- 对 v3 的推论：v3 b1（02:16 树 = b1v2 树 − ts_expand_decls 门）若门为
  真无操作（钉测摘门树 4 passed 佐证），transitioncancel-003 大概率仍
  挂——v3 预期 P2F=0 需按实测修正；回归源二分下一步 = 摘 omit-all 或
  摘 transition.rs parser 门 solo 对拍。
- **推论被终态实测推翻（02:35 收口改判）**：v3/ts2b 终态双跑（同 face
  内容重跑）P2F=0 全域、cancel-003 双腿稳定 pass——b1v2 solo P2F=1 未复
  现，改判**负载敏感形 flake**（5000ms 时序页，刀 TP 已档同签名；与审计
  fork 02:25 判读一致，solo 对拍单例不足定谳真回归，两全腿跑满为终态
  证据）。上述「二分下一步」无对象，撤案。

### v4 face-verify（02:47 收口笔——提交面==验证面红线兑现）

提交链落库（`917378f05` ANIMCOMP 十件 + `8adbf44fc` TS 四件 + `78294481d`/
`1b8fc7c85` docs）后，docs/commit fork 以当前树（=提交面 + ATP 孤儿 WIP
恒定背景）重建 b1（md5 `a88280116`，含 02:11:30 parts.join 收口态与 doc
注——终账电池 b1 `54ab4fbb` 为 ts3 face 02:06:05 patch，两者差 parts.join
收口与文档注，css-wide 前置门已不在树）单侧腿 face-verify
（results-knife-ts-v4-faceverify，b0 对照沿用 v2 存档 JSON）：

- **css-transitions F2P=57 / P2F=0 / net=+57**，逐页 shorthand 18 +
  behavior 23 + computed 8 + invalid 5 + valid 3 与终账电池逐键一致；
  cancel-003 pass（第 5 腿独立复现＝b1v2 solo fail 负载敏感 flake 终证）；
  哨兵 pva 0/0/0。
- parts.join 收口编辑自此入验证面；刀 TS 全链（v1 车道错接 → v2 P2F=3 →
  v2.1 修形 → v3 终账 GREEN → v4 face-verify GREEN）闭合。

## Round B（02:48 发射，03:44 探针定谳收笔）

电池 zw-knife-ts-rb-battery-0248（另一 fork）：b0=v4 存档 bin
（md5 `a882801163f`）/ b1=整树构建（"树即 face"）。账面 F2P=0/P2F=0
net=0 形 RED，但独立 fork 法证（NOTES-knife-ts-rb-forensics.txt 03:16）
定谳 **null 账根因＝fix 未进被测二进制**：computed_style.rs
duration-force 修形编辑 mtime 02:50:34 晚于 b1 构建收官（02:48:38）
112 秒——"编辑撞构建窗"同形态再现。GONE=560＝腿窗他流编辑腿并发负载
页级截断，重射需静窗。

### 三值探针定谳（03:41，zz-rb-probe/2/3 三发，靶 a882801163f）

- probe1/2（detached 元素）computed 读全空串——detach 短路 vs 引擎缺口
  未定谳，corpus 零键依赖不入账，作废；probe3（appendChild 挂树）为准：
- **写道 V1 Pass**：`transition: "0s 1s"` 展开 attr 落 delay+duration；
- **store A-P2/A-P5/A-P6 Pass**：computed delay=1s / duration=0s /
  property=all——传播链前两层齐全；
- **A-P3（RB 靶面）got "1s"≠"0s 1s"**：简写道重组器**收到 delay** 但
  省略 0s duration——duration-force 修形对症，预期 F2P=1 成立；
- **A-P4 新缺口 got "all"**：长手道 `style.transitionDelay="1s"` 重组
  输入传播断——独立缺口，RB 修形不覆盖，Round C 候选刀项。
- 探针页三发即删（存档 .acceptance/wptfyi-css/*-saved.html）。

### 重射前置门（03:42 实测）

RIDE-21 现测 vs 02:47 s1 快照 ≥4 文件 md5 漂移（css-parser ATP 流
推进中，03:37 九文件批量编辑在飞）——整树 b1 ≠ v4bin+单修形，多变量
污染。**RB 重射等 ATP 流提交落库后执行**（届时整树 b1 需重新核背景
恒定性）。教训入账：整树 b1 的单变量前提必须每次发射前以内容 md5
对照快照实测，mtime 分代法证在共享树 rebase 后失效（autostash pop
批量改写 mtime）。
