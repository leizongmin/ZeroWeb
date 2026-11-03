# 残余挂死归因修复轮（spin2）：m670 move 环进展守卫——fd 全页 T 归零 + delete solo 形态治愈、全页形态恒存（battery3 反证）

日期：2026-11-03（editing 流，控制面下一步 ⑥ 残余挂死两站点 solo 归因修复窗收口）；2026-11-04（battery3 del 腿判读修订：全页形态反证，diag3 定谳改判）；2026-11-04（battery3 收官 fd/mt 判读 + 对侧 x1/x-del-1 探针收编：mt 系统性 T 两层机制定性〔近端 300s 预算击穿 × 根因 in-face 慢化〕+ 旧代 mt 基线静默早止误账修正 + 尾段 +73 案发现）

## 0. 范围与站点盘点修正

控制面 ⑥ 立项时的两站点表述经本轮证据修正为三点位：

1. **multitest ?9001-last**：v3 replay（g28fix-replay-mt.log）已无 Timeout、
   尾案（multitest.js 末案 `<div>abc</div><div><b>{def</b></div>`）完整出账
   ——**6a3421c88 的 :606 守卫已治愈**，控制面「挂死跨 v1/v2/fix 三代恒存」
   表述过时（v2 battery 中 mt ?9001-last 本就完整出账 1004 行，此前 tests:599
   截断归因属误记，实为 fd 腿）。
2. **forwarddelete.html?6001-7000**：v1/v2 battery 双代 Timeout 恒存（截断于
   forwarddelete.js:2529 `foo[]<div contenteditable=false>bar</div>baz`），
   **控制面漏账站点**；本轮实锤同为 :606 守卫（s615 splitParent 环）治愈——
   fix-land 复验漏跑 fd 腿，治愈状态拖两代未记账。
3. **delete.html?7001-last**：真残余。v1/v2/v3 三代 Timeout 恒存（截断于
   delete.js:3183 CE-false 组首案 `<p>{}<span contenteditable=false>ab</span>
   </p>`「Do not delete ancestor blocks which still has non-editable content」，
   last completed = :3177），本轮 m670 守卫修复——**battery3 全页形态反证：
   守卫仅治愈 solo（variant 裁剪）形态，全页 8-variant 挂死逐位恒等恒存、
   未解（见 §3 battery3 行与 §1.3 机制修正）**。

## 1. 归因链

### 1.1 fd 定谳（diag2，g28spin2-diag2-fd.log）

TEMP-DIAG 页尾探针（globalThis.__zwSpinDiag 快照出账）实锤：

```
SPINDIAG {"tag":"s615-break","iters":0,"extra":{
  "child0":"t:\"bar\"^div[1]ce=false^div[3]ce=^div[2]^body[11]^html[2]^#document",
  "startBlock":"div[3]ce=^div[2]^body[11]^html[2]^#document"}}
COUNTS {"cwA":9,"cwB":7,"eqFirst":2,"cwD":2,"m670":1,"c674":1}
```

fd 挂死 = s615 splitParent 环对 CE-false 父（`div[ce=false]`，
part01e.js:47-48 早退）零进展；:606 守卫首迭代即 break（iters:0）→ 页完成。
COUNTS 中 m670/c674 各仅 1 次 = 守卫破环后合并路径正常走完。探针行为只读记
录（不 throw 不改 DOM），守卫本身是 6a3421c88 已提交代码 → 结论对无插桩二进
制成立（diag3 守卫版纯构建复证，见 §3）。

### 1.2 delete 归因（diag1 反证 + diag3 solo 正结果 + battery3 全页反证）

- **diag1（throw-bail 插桩版）负结果**：环体 tick 4096 次/30s 墙钟双门
  throw 破环 → 页 1500s test-guard 击杀永不完页（zw-g28spin2-diag1.log，
  Error 124）。机制：throw 于 move 环中途 unwind，合并半途撕裂 DOM 不变量
  （endBlock 半搬空 + startBlock 混入序列化拷贝），后续宿主调用对该损坏树
  进入**不可中断自旋**（V8 terminate_execution 打不到 Rust 内部段）——与
  v1/v2/v3 无插桩版「JS 级自旋 300s 可中断出 Timeout 行」形态对照，证
  throw 式诊断 bail 在 merge/多节点操作路径不安全。
- **diag3（m670 守卫版，无插桩）solo 正结果**：delete.html?7001-last +
  forwarddelete.html?6001-7000 两页（runner FILTER 子串匹配连带，见 §4.2）
  0 Timeout 全出账（zw-g28spin2-diag3-del.log）——**仅证 solo（variant
  裁剪）形态治愈**。初版「m670 守卫治愈定谳」表述经 battery3 del 腿
  全页判读修订（全页形态逐位恒等复现挂死，见 §3）；diag3 的 +206 行
  恢复出账为裁剪效应下的真治愈，但对真实挂死形态（全页序）无效。

### 1.3 机制（delete.js:3183 形）

`<p>{}<span contenteditable=false>ab</span></p>` delete backward → 算法入
sibling-merge 臂（part01f.js else 臂），`while (endBlock.hasChildNodes())
_zwJusMovePreservingRanges(endBlock.firstChild, startBlock, -1)`（:670-672）
对 plain 子树（CE-false span 内层）走 shim 桥 appendChild「序列化拷贝、不摘
原父」语义（part01c.js _zwJusMovePreservingRanges 注记）→ child 数不降 →
零进展 JS 自旋。可中断性形态与 v1/v2/v3 300s Timeout 恒存吻合；:606 守卫只
覆盖 s615 splitParent 环不覆盖本环，故三代恒存。

**battery3 全页反证后的机制修正（2026-11-04）**：m670 守卫把本环迭代硬界为
≤256（长度不降即 break，或计数门 break），守卫版二进制（strings 实锤含
m670Guard）全页形态仍逐位恒等复现挂死 → **全页形态的自旋不在 m670 环体内**
——要么守卫 break 后回落壳在下游路径（_zwDelRestoreValues/后继 merge 步）
另入环，要么污染态下同案在更早路径自旋（根本未达 m670 环）。触发案与截断
点同 solo 形态同案同位，但自旋站点不同；solo/全页分叉指向跨 variant 状态
污染（假说 C 实证，见 §3 battery3 行与 §4.4）。

## 2. 修复

part01f.js m670 环进展守卫（本轮唯一产品码改动）：

- 合法迭代必令 endBlock.childNodes.length 单降 1；长度不降（move 早退/吞
  异常/拷贝不摘）或 >256 迭代即 break 回落——守卫只在恒不进展态触发，与
  :606 守卫同哲学（不进展不自旋）。回落保留混合壳，尾段
  _zwDelRestoreValues 按原对象恢复安全。
- c674 空壳爬升环（:674-678）复核不加分担守卫：removeChild 无条件单进展必
  终止（COORD 廿九轮注记的候选在本轮证据下排除），守最小实现。

## 3. 验证（diag3 solo 双页 vs 基线切片）

| 切片 | 基线 | 守卫版复验 | 共享行状态 | 增量 |
|---|---|---|---|---|
| delete ?7001-last solo（diag3） | v3 replay 切片：636 行/1T | 842 行/0T | 共享行逐位恒等，P→F=0 | +206 行恢复出账 |
| fd ?6001-7000 solo（diag3） | v2 battery 切片：599 行/1T | 1000 案行 + 1 探针行/0T | 共享行逐位恒等，P→F=0 | +401 案行恢复出账（另有 1 行 spin-diag 探针出账系页内 TEMP-DIAG 产物，非案行） |
| delete 全页 8 variant（battery3 del 腿，2026-11-04） | v3 全页基线：7,636 行/8 页/1T | 7,636 行/8 页/1T | 全行集逐位恒等（P→F=0、F→P=0、新增 0、消失 0），?7001-last T 同案同位复现 | **0——m670 守卫对全页形态零效果** |
| fd 全页 8 variant（battery3 fd 腿，2026-11-04） | v2 全页基线：7,090 行/8 页/1T | 7,491 行/8 页/0T | ?6001-7000 +402 案行恢复、其余 7 variant 逐位恒、P→F=0 | **:606 治愈守卫版保持**；battery2 −8 尾截不复现（flake） |
| mt ?9001-last 全页（battery3 mt 腿，2026-11-04） | v3 基线：1,019 行/0T | 939 案行 + 1T | 三向共享 939 行前缀逐位恒等，P→F=0 | **新 T 复现（2/2）→ 系统性**；近端 = 300s 预算击穿（in-face 慢化），根因 = 跨 variant 污染（假说 C）待前缀二分归因 |
| mt ?9001-last solo 1200s（x1 探针，2026-11-04） | v3 基线：1,019 行/0T | 1,092 行/0T/211s | 前 1,019 行与基线逐位同序恒等 P→F=0，尾段 +73 案全新 | **solo 干净完走 + 旧基线静默早止误账实锤**（页全集 ≥1,092 案） |
| delete ?7001-last solo 1200s（x-del-1 探针，2026-11-04） | v3 solo 切片：636 行/1T | 842 行/0T | 共享行逐位恒等，P→F=0 | diag3 solo 治愈逐位同幅复证（in-face 挂死恒存不变） |

- 增量行 = 挂死截断压抑的尾部案恢复出账（基线 Timeout 伪行随治愈消失）；行 = runner 断言行（Pass/Fail/Timeout/Unsupported）。delete.html 在 diag3 时已撤 TEMP-DIAG 探针（842 行全案行），fd 页探针后随 corpus 还原撤除。
- engine lib（守卫版复跑，g28spin2-enginelib.log）：**3,720P/5F**，5 红名单与守卫前 g28fix-enginelib.log 逐名恒一（全 s249 CSSOM 重序列化已知回落，不回退）。
- 全分母 battery（收口复验，两代）：
  **battery1（zw-g28spin2-battery，22:05 失败）**：delete 腿 `FILTER=delete.html`
  子串连带 del+fd 16 页，3000s 墙钟击杀（22:05:50 rc=2）——runner 行缓冲在进
  程退出时才 flush，守卫击杀 = 全部行集丢失；fd 腿启动 100s 后随 battery2 换轨
  Terminated，mt 腿未跑。教训：双文件一腿预算必须按页数放大 TIME_LIMIT。
  **battery2（zw-g28spin2-battery2，22:07:17→23:42:25 收官，23:48 判读）**：
  leg-delfd `FILTER=delete.html` TIME_LIMIT=4500 + leg-mt `FILTER=multitest`
  TIME_LIMIT=3000 两腿。**leg-delfd 溯源定谳守卫前二进制**（COORD 卅六轮三点
  位：日志 cargo `Finished in 0.19s` 复用 battery1 代 ~21:15 构建；守卫源码
  22:26:29 才落树晚于构建起点；?7001-last 截断点逐字节同一 v3 基线尾案行）——
  电池与守卫提交竞速，本腿重定性为守卫前全量基线复现而非守卫验证。判读
  （judge-spin2-battery2.py）：**delete 面** 7,636 行/8 页与 v3 基线逐位恒等
  （含 ?7001-last T，恒等 True）；**fd 面** 7,090→7,483 行（+393 恢复出账）、
  ?6001-7000 T 消失、P→F=0（「共享行恒等 False」系该脚本行级状态集判据对恢复
  页误伤——T 行消失+新 Pass 行必破状态集恒等，非回归；后继判读已改恢复面白
  名单判据）；**leg-mt（23:12 重建守卫版二进制，strings 实锤含 m670Guard）**
  9,990 行 vs v3 基线 10,019——**mt?9001-last 新出 Timeout**：990 行 vs 基线
  1,019（截断 -29 断言行 +1T 行），截断点 = `<div>abc</div><div>{def</div>}`
  compare innerHTML 之后，挂死案 = `<div style=display:flex><span>{abc</span>
  <span>def</span>}</div>` delete（v3 该案完整出账 5P+1 已知 F）；对照
  v1（守卫前 1,029 行无 T）/v2（1,004 行无 T）/v3（:606-only 1,019 行无 T）
  三代该页完整出账，守卫版构建唯一 T → **潜在 m670 守卫致挂信号待定谳**；
  机制疑点：break 守卫只出不进，新挂死需假设 break 后混合壳引发下游环（或该
  案另有与守卫共变的触发面）；battery2 mt 腿 23:11-23:42 机器低载，300s 看门
  狗击发非负载解释所能覆盖。**battery3（zw-g28spin2b，23:46 起）**：守卫
  版 del→fd→mt 全量三腿 `FILTER=editing/run/<f>.html`（带目录前缀修子串连带）
  TIME_LIMIT=10800。**del 腿 00:16:55 收官判读 = Branch Y 反证**（judge 脚本
  行级会计 + `strings` 复核二进制含 m670Guard）：8 页 7,636 行/1T 与 v3 守卫
  前基线全行集逐位恒等（P→F=0、F→P=0、新增 0、消失 0），?7001-last T 同案
  同位复现——**m670 守卫对全页形态零效果，diag3「治愈定谳」改判为裁剪效应**
  （solo 形态 +206 行治愈为真，全页形态挂死另有自旋站点，见 §1.3 机制修正；
  跨 variant 状态污染假说 C 实证）。本腿 23:46-00:16 独占机器无并发电池，
  复现干净。**fd 腿（00:16:55→00:46:54 rc=2）判读**：7,491 行 = 7,090 + 402
  ——?6001-7000 恢复面 598→1,000 案行（T 伪行消失，P→F=0，幅度恰同 diag3
  solo），其余 7 variant 逐位恒基线（含 ?7001-last = 491 全额）——**:606 对
  fd 全页形态治愈在守卫版保持**；battery2 的 ?7001-last −8 尾截（483 行，
  ws 族尾 8 案静默未报、无 T 伪行）**不复现 → 记 battery2 一次性 flake**
  （行号核验：基线 491 行中丢失的 8 行全落页尾 idx 483-491 ws 族案）。
  **mt 腿（00:46:54→01:19:02 rc=2）判读 = T 复现，样本 #2 定谳系统性**：
  9,940 行 vs 基线 10,019，?9001-last 939 案行 + 1T（对旧基线账 80 案未报），
  P→F=0；battery2/spin2b 两代守卫二进制三向共享 **939 行前缀逐位恒等**，
  截断点漂移（spin2b 止于案 idx 939 fontSize/foreColor/insertText 族、
  battery2 报至 idx ~988 `{def</div>}` 族）。**探针收编判读（对侧
  x1-mt-solo + x-del-1-solo，守卫二进制 × ZW_SCRIPT_TIMEOUT_MS=1200000
  solo 腿，01:23:27-01:27:54，COORD 卅八轮立项、卅九轮乙续让位）**：
  **mt ?9001-last solo 211s 完走 1,092 行 0T**——前 1,019 行与 v3 基线
  逐位同序恒等（P→F=0、vanish=0），尾段 **+73 案全新**（inserttext Y/d/B、
  ws 族、table th/b 族，含 +9 Fail）——**旧代基线 1,019 行「尾案完整出账」
  系静默早止误账**（页真实全集 ≥1,092 案；旧代枚举确定性止于 1,019、
  pending=0 无 T 静默完成，分界点 = g/h 族后首新族 inserttext Y，stop
  机制待归因）。del ?7001-last solo 842 行 0T（= diag3 逐位同幅复证）。
  **T 行 state JSON 实锤惰性建测**（corpus JS 随跑随建）：del 636/mt 940/
  battery2-mt 990 均 pending=1——截断点 = V8 看门狗 300s 到点时的建测/
  执行进度位，漂移 = 抖动，**预算击穿为近端机制**。**根因方向 = 全文件
  语境慢化（跨 variant 污染，假说 C 统一 del/mt 两站点）**：同守卫二进制
  solo 211s 完走 vs in-face >300s 截断（≈1.7× 慢化）；旧代 in-face 1,019
  案 <300s 完走 vs 守卫代 939/990 截断 → 守卫代 in-face 慢化更重（候选
  机制 = m670 break 混合壳跨页累积 + s26x/R54xx 共变面，待归因）。
  **diag4c 插桩收编（对侧，01:47:29-02:22:11 rc=2，全 8 variant × DIAG4
  探针）**：del in-face 机制补全——第 8 页 ?7001-last **probe 行出账**
  （probe 为独立 script tag，看门狗击杀语料脚本后照跑）+ T 行 state
  （created=637/pending=1，与历代同位）= 语料脚本死于案体执行中段
  （trace 尾停 `delSel`）——**del = 确定性单案自旋击杀**（s615 排除：
  splitParent=0；m670 排除：守卫版字节恒等论证——若 m670 环自旋守卫必
  破环移位截断；spinner = delSel 内未计数环，计数线性无异常：in-face
  m670×36/c674×42/move×37 vs solo 完走基线 m670×64/c674×59/s615×9），
  精确定位待更细插桩（对侧 diag 窗）。两站点近端同为 300s 看门狗击杀、
  形态二分：del 单案自旋（同案恒定）vs mt 慢化截断（漂移）。本流不
  solo 复现避免 CPU 争抢污染对侧近边界页。

## 4. 方法论教训

1. **throw 式诊断 bail 在 merge/多节点路径不安全**：中途 unwind 撕裂 DOM
   不变量，下游宿主调用可进入 V8 看门狗不可达的自旋。诊断出账用页尾探针读
   globalThis 快照（diag2 形态）或非 throw 守卫记录（s615 形态）。
2. **runner FILTER 为子串匹配**：`FILTER='delete.html'` 连带命中
   `forwarddelete.html`（testharness_editing_corpus.rs 对
   `{subdir}/{file_name}` 做 contains）——diag3 实际双页出账、diag1 23 分钟
   击杀账含 fd 页因素；variant solo 场景 FILTER 需含目录前缀或探针页承载。
3. **fix-land 复验必须覆盖全部受影响命令面**：fd 与 delete 走同一
   `__zw_ce_delete_apply`（part06.js:4600-4602），v3 replay 只跑 mt/ip/del/
   other 漏 fd，致 fd 治愈状态两代未记账、控制面站点盘点失真。
4. **挂死修复的治愈验证必须覆盖真实挂死形态（全页序），solo 裁剪页不作
   定谳**：spin2 diag3 solo 双页 0T「定谳」被 battery3 全页 8-variant 逐位
   恒等复现反证——同一触发案在 solo/全页形态分叉（守卫 ≤256 迭代界证明
   全页自旋不在守卫覆盖的环体内），跨 variant 状态污染（假说 C）成立。
   FILTER 裁剪页只适合作机制探针（diag1-3 形态）；归因修复轮的治愈结论
   必须以电池全量腿为准（至少挂死变体 + 其全部前序 variant 同进程装载）。
