# 残余挂死归因修复轮（spin2）：m670 move 环进展守卫——delete/fd 两腿 T 归零零翻转

日期：2026-11-03（editing 流，控制面下一步 ⑥ 残余挂死两站点 solo 归因修复窗收口）

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
   last completed = :3177），**本轮 m670 守卫修复**。

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

### 1.2 delete 归因（diag1 反证 + diag3 定谳）

- **diag1（throw-bail 插桩版）负结果**：环体 tick 4096 次/30s 墙钟双门
  throw 破环 → 页 1500s test-guard 击杀永不完页（zw-g28spin2-diag1.log，
  Error 124）。机制：throw 于 move 环中途 unwind，合并半途撕裂 DOM 不变量
  （endBlock 半搬空 + startBlock 混入序列化拷贝），后续宿主调用对该损坏树
  进入**不可中断自旋**（V8 terminate_execution 打不到 Rust 内部段）——与
  v1/v2/v3 无插桩版「JS 级自旋 300s 可中断出 Timeout 行」形态对照，证
  throw 式诊断 bail 在 merge/多节点操作路径不安全。
- **diag3（m670 守卫版，无插桩）正结果**：delete.html?7001-last +
  forwarddelete.html?6001-7000 两页（runner FILTER 子串匹配连带，见 §4.2）
  0 Timeout 全出账（zw-g28spin2-diag3-del.log）→ m670 守卫治愈定谳。

### 1.3 机制（delete.js:3183 形）

`<p>{}<span contenteditable=false>ab</span></p>` delete backward → 算法入
sibling-merge 臂（part01f.js else 臂），`while (endBlock.hasChildNodes())
_zwJusMovePreservingRanges(endBlock.firstChild, startBlock, -1)`（:670-672）
对 plain 子树（CE-false span 内层）走 shim 桥 appendChild「序列化拷贝、不摘
原父」语义（part01c.js _zwJusMovePreservingRanges 注记）→ child 数不降 →
零进展 JS 自旋。可中断性形态与 v1/v2/v3 300s Timeout 恒存吻合；:606 守卫只
覆盖 s615 splitParent 环不覆盖本环，故三代恒存。

## 2. 修复

part01f.js m670 环进展守卫（本轮唯一产品码改动）：

- 合法迭代必令 endBlock.childNodes.length 单降 1；长度不降（move 早退/吞
  异常/拷贝不摘）或 >256 迭代即 break 回落——守卫只在恒不进展态触发，与
  :606 守卫同哲学（不进展不自旋）。回落保留混合壳，尾段
  _zwDelRestoreValues 按原对象恢复安全。
- c674 空壳爬升环（:674-678）复核不加分担守卫：removeChild 无条件单进展必
  终止（COORD 廿九轮注记的候选在本轮证据下排除），守最小实现。

## 3. 验证（diag3 solo 双页 vs 基线切片）

| 切片 | 基线 | 守卫版 diag3 | 共享行状态 | 增量 |
|---|---|---|---|---|
| delete ?7001-last | v3 replay 切片：636 行/1T | 842 行/0T | 共享行逐位恒等，P→F=0 | +206 行恢复出账 |
| fd ?6001-7000 | v2 battery 切片：599 行/1T | 1000 案行 + 1 探针行/0T | 共享行逐位恒等，P→F=0 | +401 案行恢复出账（另有 1 行 spin-diag 探针出账系页内 TEMP-DIAG 产物，非案行） |

- 增量行 = 挂死截断压抑的尾部案恢复出账（基线 Timeout 伪行随治愈消失）；行 = runner 断言行（Pass/Fail/Timeout/Unsupported）。delete.html 在 diag3 时已撤 TEMP-DIAG 探针（842 行全案行），fd 页探针后随 corpus 还原撤除。
- engine lib（守卫版复跑，g28spin2-enginelib.log）：**3,720P/5F**，5 红名单与守卫前 g28fix-enginelib.log 逐名恒一（全 s249 CSSOM 重序列化已知回落，不回退）。
- 全分母 battery（收口复验，两代）：
  **battery1（zw-g28spin2-battery，22:05 失败）**：delete 腿 `FILTER=delete.html`
  子串连带 del+fd 16 页，3000s 墙钟击杀（22:05:50 rc=2）——runner 行缓冲在进
  程退出时才 flush，守卫击杀 = 全部行集丢失；fd 腿启动 100s 后随 battery2 换轨
  Terminated，mt 腿未跑。教训：双文件一腿预算必须按页数放大 TIME_LIMIT。
  **battery2（zw-g28spin2-battery2，22:07:17 起，判读中）**：leg-delfd
  `FILTER=delete.html` TIME_LIMIT=4500（连带双文件在预算内）+ leg-mt
  `FILTER=multitest` TIME_LIMIT=3000 两腿。判读 vs g28-v3-del（7,636 行/1T）/
  g28-v2-fd（7,090 行/1T）/g28-v3-mt（10,019 行/0T）基线，判读脚本
  .acceptance/ipara-port/judge-spin2-battery.py；落账复核口径：P→F 归零 +
  新增行全落挂死截断恢复面（delete ?7001-last / fd ?6001-7000）+ mt 逐位恒等。

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
