# 残余挂死归因修复轮（spin2）：m670 move 环进展守卫——fd 全页 T 归零 + delete solo 形态治愈、全页形态恒存（battery3 反证）

日期：2026-11-03（editing 流，控制面下一步 ⑥ 残余挂死两站点 solo 归因修复窗收口）；2026-11-04（battery3 del 腿判读修订：全页形态反证，diag3 定谳改判）；2026-11-04（battery3 收官 fd/mt 判读 + 对侧 x1/x-del-1 探针收编：mt 系统性 T 两层机制定性〔近端 300s 预算击穿 × 根因 in-face 慢化〕+ 旧代 mt 基线静默早止误账修正 + 尾段 +73 案发现）；2026-11-04（diag6 probe-v3 收编：diag4c「单案自旋」判读撤改——solo 267s 完走 + 全位点零离群，in-face T 改判跨页累积慢化 × 300s 预算耗尽〔del 1.48×/mt 1.65×〕，spin2d 2× 墙钟判别在飞）；2026-11-04（diag7 onerror 收编判读：全 8 页 err 空——异常早止证伪、45b「工作到点」定谳，spin2d del 腿 0T/+206 行模型 D 逐字节命中，del 面慢化定谳收口）；2026-11-04（spin2d 双腿判读收口 = spin2 残余挂死窗关闭：del 0T/7,842 行 + mt 0T/10,092 行，两尾页对 solo 元组级逐位恒等、非尾页对基线恒等 P→F=0，模型 D 双腿定谳，条件挂死证伪，详见 §3 末段）

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

**diag6 收编后的再修正（2026-11-04 03:1x）**：全位点插桩（probe-v3：canon-in/
out 追踪 + c1-c4 等 11 计数位点）击杀时快照零离群 + x-del-1 solo 267s 完走
含 637 案位后全部尾段 → 全页形态亦非自旋，in-face T = 跨页累积慢化（~1.5×）
下 300s 预算耗尽；上文「自旋站点不同」一并撤改（无自旋站点），假说 C 由
定性转定量。详见 §3 diag6 段。

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
  （trace 尾停 `delSel`）——**del = 确定性单案自旋击杀〔03:1x 撤判，见下
  diag6 段〕**（s615 排除：
  splitParent=0；m670 排除：守卫版字节恒等论证——若 m670 环自旋守卫必
  破环移位截断；spinner = delSel 内未计数环，计数线性无异常：in-face
  m670×36/c674×42/move×37 vs solo 完走基线 m670×64/c674×59/s615×9），
  精确定位待更细插桩（对侧 diag 窗）。两站点近端同为 300s 看门狗击杀、
  形态二分：del 单案自旋（同案恒定）vs mt 慢化截断（漂移）〔形态二分
  表述随撤判一并修订，见下 diag6 段〕。本流不
  solo 复现避免 CPU 争抢污染对侧近边界页。
  **diag6 probe-v3 收编 + diag4c 判读撤改（乙续自证复核，03:0x；对侧
  diag6 02:26:56-02:58:42 rc=2，probe-v3 插桩版 28.92s 重建，COORD
  四十七/四十五/四十八轮在案）**：本流独立复核 diag6 出账——非 probe
  判读行 7,635 + 1T 与 v3 基线 **diff 空**（唯一差异 = 本流提取正则排除
  的 T 行本身；8 probe 行每 variant 1 行），截断深度跨五代二进制恒定
  （第 8 页 635 案行 + 1T、created=637/pending=1 同位）。**probe-v3
  全位点击杀时快照零离群**：新增 canon-in/out 追踪 + c1-c4 计数（击杀页
  c1=440/c2=533/c3=111/c4=113，落 7 健康页正常带 312-636/267-787/8-167/
  26-326）+ ffc/restore/sameHost/move/splitParent 位点，trace 尾 = canon
  平衡对 + apply/delSel 正常节拍——11 计数位点 + 7 追踪点无一异常，
  位点插桩路线对该问题已尽。**撤判依据（预算算术，对照臂 = 同二进制
  solo 完走）**：x-del-1 solo ?7001-last **267s 完走 842 案行 0T**（含
  637 案位以后全部尾段）→ 无条件单案自旋不存在；diag4 solo 同页 325s
  （含 make 包装）完走复证。**改判机制 = in-face 跨页累积慢化 × 300s
  预算耗尽（慢化非挂死）**：del in-face 均速 637 案/300s = 2.12/s vs
  solo 842 案/267s = 3.15/s（**1.48×**）；mt in-face 939/300 = 3.13/s vs
  solo 1092/211 = 5.18/s（**1.65×**）——两站点同量级 ~1.5×，假说 C 由
  定性转**定量定谳**；del 截断恒定 = 页 8 solo 成本（~250-300s）本就贴
  300s 预算边界 + CE-false 组高成本案簇确定性落点；mt 漂移 = 成本面
  平坦、到点位随代际开销漂移。慢化载体（R350 族进程内单调增长态 vs
  守卫代增量）仍待归因（m670-revert A/B 仍为反事实判决实验）；残存
  不可证伪面 = 未插桩区条件性挂死，判别 = **spin2d 2× 墙钟探针**（本流
  03:10 在飞，zw-g28spin2d：del+mt 全文件 ZW_SCRIPT_TIMEOUT_MS=600000
  直呼 runner——慢化模型预测两腿 0T 尾段首次全量出账 del +206/mt +153
  行；条件挂死模型预测 600s 同深度 T）。旧代 mt 静默早止（pending=0
  无 T）与守卫代预算 T（pending=1）签名异通道，diag7 onerror 通道归
  对侧 probe 线。

  **diag7 onerror 收编（验证流 diag7 执行位，03:22-04:00:28 rc=2；zw-g28diag7c
  unit，TIME_LIMIT=3600 三方重载容噪）**：onerror + unhandledrejection 双钩子
  安装于 delete.html 全部语料 script 之前（globalThis.__zwDiag7Err，ES5，
  return false 不改默认行为），页尾 probe 块扩展 'DIAG7 ' payload（trace 尾 40
  + cnt + err slice(0,3)）。**判读（b37-续流 03:08 轮武装窗口判据）**：全 8 页
  probe **err 全空**——中止瞬间唯一可见通道零捕获，**「驱动早止 + 未捕获异常」
  （46 精化）证伪**；T 唯一 ?7001-last（state tests:637/pending=1，截断深度与
  diag4c/diag6 同位——细化插桩不移深度二次复证）→ **45b「300s 击杀时语料任务
  仍在执行（工作到点被终止）」实锤**。**细化位点零扰动实证**：挂死页 kill-time
  cnt 与 diag6 逐字节同一（c1=440/c2=533/c3=111/c4=113/move=37/m670=36/
  c674=42/ffc=138/restore=60/sameHost=1184/s615=8/splitParent=0），trace 尾
  [delSel, canon-in, canon-out] 平衡对——甲线增补的 front-check/cbp-check/
  merge-dispatch/delSel-ret 分段标记不改变任何计数与深度。**与 spin2d del 腿
  互证（乙续 face，del 腿账 03:10:48-03:51:02 rc=1 仅 probe fail 行）**：
  **0 Timeout、7,850 行 = 7,636 基线 + 206 尾段恢复 + 8 probe——模型 D del 腿
  预测逐字节命中**；页 8 = 843 行（842 solo 完走全额 + 1 probe），完走 cnt 命中
  solo 基线（m670=64/c674=59/s615=9，c1=870 ≈ 2× 击杀时 440 = 全页 canon 工作
  量翻倍于截断点），err 同空——**del 面慢化非挂死定谳收口**（300s 预算 ×
  ~1.48× 跨页累积慢化，600s 预算即全量出账）。mt 腿（03:51:02 起在飞）归乙续
  判读，模型 D 预测 0T + mt 尾段 +153（939→1,092）。**双实例事故（判读不采用）**：
  zw-g28spin2-diag7b（03:01:12，session 重放孤儿）与 zw-diag7（03:08:52）同
  face 双跑共写 g28spin2-diag7.log（行级交错 + 前数页 pre-hook 语料混代），
  03:22 双停，日志归档 g28spin2-diag7.log.polluted-ab 留证。**代际注记**：
  diag7c 二进制 = 同源码 35.54s 真重建（b6888ce88 + 树内 WIP +30/-1 含甲线细化
  位点；03:22 轮「无重建」表述与盘上 Finished 35.54s 不符，如实修正）；spin2d
  del 腿跑 03:09:25 旧 inode（同源码构建），两代语义恒一。**收尾**：delete.html
  已全洁净还原（先还原 diag7-pre 态、再按甲线 03:30 轮归属指令撤除 diag4 probe
  块至 corpus-backup 原貌）；multitest.html 实测亦为原貌（甲线「探针已加」注记
  与盘上不符，如实记档）；四 shim 文件 WIP（+30/-1）待 spin2d mt 腿完走后由本
  执行位 git checkout 归零（甲线 03:30 轮授权）。**残余开放**：慢化载体归因
  （m670-revert A/B 反事实判决 + R350 族进程内增长态插桩候选）；spin2d mt 腿
  判读（乙续）。

  **spin2d 双腿判读收口（乙续判读位，04:26:01 done；judge-spin2d.py 元组级
  口径：页级伪 T = title==page 单列、diag7 probe 行剥离、非尾页对电池3 基线
  g28s2b-*-rows.txt 恒等、尾页对 solo x-*-solo.log 恒等；判读权经 COORD 五十
  轮预注册归乙续，验证流 04:28 出独立提取回执 10,092/0T 逐字节同数不越位）**：
  **del 腿**（03:10:48→03:51:02，40.2 min）raw 7,850 tuple = 0 伪 T + 8 probe
  剥离 + 7,842 实案行；7 个非尾页对基线逐位恒等（P→F=0），尾页 ?7001-last
  635→**842 == solo 842 逐位恒等**（724P/118F；恢复 +207 实案 = +146P/+61F，
  基线含伪 T 记账 +206 预测逐位命中）。**mt 腿**（03:51:02→04:26:01，35.0
  min）raw 10,092 tuple = 0 伪 T + 0 probe（语料洁净态）= 10,092 实案行
  （P 9,673/F 419）；9 个非尾页对基线逐位恒等（P→F=0），尾页 ?9001-last
  939→**1,092 == solo 1,092 逐位恒等**（996P/96F；恢复 +153 = +133P/+20F，
  预测 +153 逐位命中）。**模型 D 双腿定谳**：两腿 600s 预算下 0T + 尾段首次
  全量出账且对 solo 逐位恒等——预注册判别落定，**条件挂死模型（S）证伪**；
  与 diag7c 互证闭合（err 全空 + kill-time cnt 与 diag6 逐字节同一 + 600s
  全量出账 = 四证合流）——**del/mt in-face T = 跨页累积慢化 × 300s 预算耗尽**
  （del 1.48×/mt 1.65×），非挂死。**spin2 残余挂死窗关闭**：四代插桩
  （diag4c/diag6/diag7c/spin2d）零条件挂死正证据，「未插桩区条件性挂死」无
  正证据支撑、仅余不可证伪形式面。**新账挂账**：尾段恢复新 Fail 全为已知族
  形态（del：flex/grid span delete compare 族 + CE-false nbsp 组；mt：table
  th/b ×inserttext B 族 + ws pre compare 族 + flex 跨块 inserttext gh 族），
  非回归、为后续修复轮扩大分母记账。**残余开放（唯一）**：慢化载体归因
  （m670-revert A/B 反事实判决 + R350 族进程内增长态插桩候选，甲线协调窗）。

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
5. **Timeout 行不是挂死证明，先做预算算术再下挂死结论**：T + pending=1
   与「看门狗到点时仍在正常推进」二义；对照臂必须同二进制 × 同进程形态
   （solo vs in-face），solo 完走时长与 in-face 截断深度一除即得慢化倍数
   （本轮 del 1.48×/mt 1.65×）。位点插桩全正常带 + 行集逐位恒等时，
   预算边界条件比死循环更省假设——spin2 窗「残余挂死」最终改判为
   300s 预算边界 × ~1.5× 跨页累积慢化，自旋假说（含 m670 零进展环、
   delSel 未计数环两代版本）均被 solo 完走 + 全位点快照撤判。
6. **异常早止候选必须用页面侧捕获通道裁决**：「驱动早止 + 未捕获异常」的
   裁判面在页面，不在 runner——V8 看门狗 terminate_execution 在 Rust 层拦截
   （try_catch.has_terminated → cancel_terminate_execution），不产生 JS
   error 事件，故 onerror/unhandledrejection 双钩子（安装于全部语料 script
   之前）是异常逃逸的唯一观测面；且武装窗口约束（v8_runtime.rs Arm/Disarm
   seq 协议）保证 Timeout 行存在 ⇒ 击杀时必有被武装且持续执行的 JS 窗口，
   「早止 + 纯空转到看门狗」按字面不成立。判据（b37-续流校准）：err 非空 →
   异常早止实锤；err 空 + T 在 → 45b「工作到点被终止」。diag7 全 8 页 err 空
   + T 唯一同位 + kill-time cnt 与 diag6 逐字节同一 + 600s 全量出账四证合流，
   45b 定谳——插桩轮先装捕获通道，再谈机制归因。
## 5. spin2d 探针执行与判读存档附记（2026-11-04；判读定谳见 §3 末段，不重复）

本节只记 §3 末段判读账未覆盖的执行面事实：

- **墙钟**：zw-g28spin2d 03:10:48→04:26:01（75.2 min）；del 腿 40.2 min、
  mt 腿 35.0 min（600s 脚本预算直呼 runner，绕过 make 配方内联 300000；
  test-guard 7200s 墙钟包裹）。
- **负载环境**：跑间三方 CPU 共载（diag7c + selection m2-c3 + 本探针，
  16 核各占 ~1 核），按 battery 先例容噪；runner 无逐页时戳出账，慢化
  倍数维持 §1.4 diag6 段 solo/in-face 均速算术口径。
- **判读装置存档**：`.acceptance/ipara-port/judge-spin2d.py`（乙续判读位
  武装，03:32 COORD 登记；基线解析核对 del 7,636 tuple/1 伪 T/tail 635、
  mt 9,940/1/939、solo del 842 / mt 1,092 全数对上）+ 原始日志
  `g28spin2d-2xbudget.log`。验证流 04:28 独立提取回执与本位判读逐字节
  同数（COORD 五十轮预注册判读权归乙续）。
- **diag7c**（03:22-04:00:28 rc=2）判读归验证流（err 全空 → 45b/D 合流，
  见其 86cefcd37 落账），与本探针互证不互代。
