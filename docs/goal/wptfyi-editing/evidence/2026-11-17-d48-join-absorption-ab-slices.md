# d48 落地轮：join 壳吸收深化 A/B（A+B1+B2 落、B3 判不落）

日期：2026-11-17。基点：HEAD 35c8988ce（d46 + ilb 3312189d7）。改面仅
`crates/engine/src/js_dom_shim/part01.js`（+130/−5，含 B1 先探后删补丁）。
四修位出自 [前置诊断](2026-11-17-d48-join-shell-absorption-diagnosis.md)。

## 四修位终态

- **A（join dst 壳下降，落）**：`_zwCeJoinDstDescend(dst)`（11782 区）——
  块界 join 接收容器两级下降：①dst 为列表容器（UL/OL/DL）→ 降末行块
  （LI/DT/DD，倒序跳纯空白）；②行块唯一可见子为 p/h1-6/div 且壳内有非空
  活内容 → 降入壳。`_zwCeJoinApply` 4 调用点统一过下降。守卫：纯空白文本
  不算内容；空壳/多子/纯文本行不降（jdwsp 反证面不受影响）。
- **B1（FD 行尾 br 挡界，落）**：FD 块界臂（12797 区）行尾 br 透明消费——
  caret 尾紧邻 br 且 br 为所在块可见末子时消费一枚后按块界 join 并入次行
  块。br 后有非 br 可见内容（行中 br）保守不动。**先探后删补丁**（A/B 发现
  P2F 后加）：消费前只读探针镜像 re-resolve 确认链上确有可并后邻——末块
  行尾 br（`<p>abc[]<br></p>` 期望保持 br 占位、基线 no-op Pass）不消费走
  原臂，消除该 P2F。
- **B2（左空摘除判空，落）**：select-delete 核左无残余摘除门（12392 区）
  判空改「无可见内容」（空/纯空白文本子不计）→ 摘空文本子 → 摘壳 → 左
  向上剪枝（剪至 lca/el，g<16）。
- **B3（同块全选 splice 占位 br，判不落）**：+8F/−2P 净 +6 但污染 2 绿面，
  回退。机理与探针证据见下节。

## A/B 口径与结果

双链同守卫（test-guard 4/8G/time-limit 1500/1500/2700）：基线链
stash part01.js 后独立 `CARGO_TARGET_DIR=/tmp/d48-base-tgt` 重建（二进制
`_zwCeJoinDstDescend` 标识 0 命中，码态隔离核验）；WIP 链
/tmp/d46-tgt 二进制（终态含补丁，标识 2 命中）。探针排除口径 zz-probe。

- **input-events 全语料**（30 cases，终态二进制）：基线 1114P/2176F/4T →
  **1364P/1964F/3T**；**F2P 223 / P2F 0（净 +223）**，族分解：joining-dl
  -elements Backspace +66 / Delete +36（=A）、dil Backspace ol/ul +31/+31
  （=B2）、dil Delete ol/ul +8/+8（=B1）、joining-dl-and-another-list
  +20/+20（=A）、forwarddelete +2 / backspace +1（B1/B2 外溢）。P 差
  +250 = 翻面 223 + ?Backspace 案本轮完整跑完尾段新入账 27（基线侧该案
  case 级超时截断，T 4→3；new 38 键即尾段，非翻面不计对账）。基线→修复中过渡
  观测：B1 初版 P2F=1（`<p>abc[]<br></p>` FD innerHTML，期望保持 br、got
  `<p>abc</p>`——消费后无 join 目标即 return，br 已吃）→ 先探后删补丁后
  fd 探针 P2F=0。
- **html-semantics 哨兵**（950 cases，终态二进制）：P 恒等 5378
  （5378P/425F/32T → **5378P/426F/31T**），F2P=0/P2F=0、键差 0；1 键
  T→F 过渡在 customizable-select 超时案带内瞬态
  （select-home-end-pagedown-pageup-detailed.optional），哨兵绿。
  （补丁前中间轮 d48w 为 5378P/430F/30T，同为 P 恒等绿。）
- **selection 全语料哨兵**（738 文件，终态二进制 vs 基线链同窗口对拍）：
  **49,006P/4,298F/9T 双链恒等，F2P=0/P2F=0、T 过渡 0**——jdwsp 三族反证哨
  与全部键级面一并恒等；键差仅 `selection/stringifier_editable_element
  .tentative.html` 案级超时边界（new 1/gone 5，基线侧该案超时后截 4 NotRun，
  瞬态非翻面）。记档：基线链账面较 d47 轮账 48,942P/4,312F 漂
  +64P/−14F，系 d47→d48 窗口语料/runner 换代带内漂移（跨流共享 wpt-data
  面），双链同窗口对拍不受影响。
- 四文件定向探针（终态二进制）：fd F2P=2/P2F=0；dil F2P=78/P2F=0
  （Backspace ol/ul +31/+31 = B2 跨容器全选形；Delete ol/ul +8/+8 = B1）；
  joining-dl 两文件 F2P=142/P2F=0（dl-elements Backspace +66、Delete +36、
  dl-and-another-list 两变体 +20/+20）。补丁前中间轮 d48w 全语料中 ?Delete
  变体受 case 级超时截断（351/440 键，gone 90），共享键仅见 F2P=29、探针
  真值 +36（截断尾段吞 7 翻面；marker 翻面 0，T 恒 4）；终态轮该案截断
  换位至 ?Backspace（base 侧截断、尾段 38 键新入账 27 Pass），?Delete 侧
  完整 +36 与探针真值合。

## B3 判不落（机理 + 探针证据）

B3 = 同块全选（`[text]` 整选）splice 摘空后补占位 br（两处：backspace
12719 区 / FD 12896 区）。探针级 A/B：dil F2P +8（4 变体 × `[]`-form
`Backspace at "<ol><li>[list-item1]</li></ol>"` 族）但 **P2F=2**：
`Backspace at "<ol><li>{list-item1}</li></ol>"` 两列表标签的 innerHTML 面
（期望 `<ol><li><br></li></ol>`）在 **pair 页**（[]+{} 两案同页序贯）
翻红，隔离页干净；页侧 pre-run 断言（选区 (li,0)-(li,1)、li 单子、fresh
li 存在）全过 → 变异发生在 shim 内部执行段。切片二分（toggle B3 out →
P2F=0）定谳 B3 元凶。两变体浅修证伪：replaceChild 变体自身 exec 失败；
removeChild+appendChild 变体污染依旧。

机理归因：**innerHTML 重置 ghost 投影**（part01.js:12298 已记录的
shim/engine 分裂遗留）——`{}`-form 经 `handleNodeSelectMarker` 提升元素锚
走 select-delete 同块臂 splice 摘空，摘除后占位 br 在 ghost 投影下物化
错位；同页前一案（[]-form 文内 splice）遗留 detached li 的投影中
`[br]` 元素毒化 `{}`-form 的 `{}`-form ② splice/占位（[]-form 遗留
`[text('')]` 不毒化）。净收益 +6 但污染既有绿面 → 判不落，解域移交
master ③ generation-aware（与 d47 别名族共同解域）：投影带 generation
标记后同块臂占位 br 可重开。遗留 8 个 `[list-item1]` 全选红键待该刀。

## 遗留与下一步

- B1 残留：`Delete at "<p>abc[]<br></p><p>def</p>"`（两 p 形）F→F 未解
  （br 未消费、bi=0，与单 p 形同 DOM 局部条件行为分裂，疑 pair 序贯
  ghost 影响，未深挖）——join 后邻 p 形并入走 ②/后续刀。
- master ③ generation-aware 刀：B3 +8 与 d47 别名族共同解域。
- master ①(b) backspace/fd 主文件 243F、B4-S2 剩余、DC-3 对账照旧。
