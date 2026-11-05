# M3 Batch 3 — gethtml.html 长尾归因收口：90s 看门狗墙钟收割 × 队列扫描二次累积

**日期**: 2026-11-06
**前置**: Batch 2 楔死定性（×1/×4 行数平坦 + pending:1，见
[batch2 证据](2026-11-06-m3-batch2-unsafe-apis.md)）；本轮重建探针拿到逐测试计时，
定性从「特定子测试楔死」修正为「**墙钟收割 × 每测试成本线性增长**」。

## 结论

1. **不存在固定楔死子测试**。zz-wedge2（候选象限 ShadowRoot 域全 12 步单实例 +
   2,500 复实例）2,318 行全 Pass 后死于同一墙钟；ABC 三相（纯 mutation / DSD 构建 /
   纯读）3,600 行全 Pass 页面完整。
2. **冻结点 = 90s V8 看门狗**（`script_timeout_ms` 缺省 90,000，R201
   `terminate_execution` 防死循环看门狗；+ 页加载/回收 ≈ 110s 墙钟）。乘数只放大
   harness/case 预算面，看门狗未随乘——×4 页预算 240s 下仍 ~90s 脚本窗处截断，
   行数与 ×1 同域（knee×1 = 2,168 行 vs knee×4 = 2,104 行，~3% 机器噪声差）。
3. **每测试成本随页内累计 N 线性增长**（knee 曲线 0ms@N=1 → ~100ms@N=2,100，
   斜率 ~0.037ms/test²，总量二次）。各文件冻结点不同（divx 1,915 / gethtml
   2,036-2,245 / trunc 2,121 / wedge2 2,318）= 各内容/binary 的斜率常数不同，
   90s 窗内到达的 N 不同。
4. 病根归类（I12 后精确化）= **ShadowRoot 域 declarative setHTMLUnsafe 组合写
   路径的每测试 O(N) 成本**（attachShadow×setHTML 交互项；attachShadow 单独、
   元素域 setHTML 单独均平坦），gethtml.html 的 ShadowRoot×declarative 象限
   （约全量一半）逐测试推进页级 N。R350 Range-mutations 同病先例：「非固定
   per-op 成本，放宽超时无解」。html-semantics 流 t8c 增量索引改造
   （callbacks.rs，本诊断进行时已在编）覆盖的是另一族队列扫描读面（tag/attr/
   text/innerHTML reads，对 html-semantics 页同病）；runner 侧本轮落地看门狗
   随乘数放大。

## 证据链（探针法证，研删未入库）

- **探针参数序 bug**：harness 签名 `test(func, name)` func 在前；首版探针按
  `(name, func)` 传参 → 全部 `func.apply is not a function`（3,072 行全 Fail 但
  页完整——反向证明当时无成本问题）。修正后计时才可用。
- **knee×1**（div×8 = 3,072 测试，名字嵌 `at+<ms>` 注册间隔）：2,168 行 Pass +
  页级 Timeout；斜率平坦爬升无硬停（0/8/20/25/39/46/59/63/79/88/100/111ms @
  1/193/385/577/769/961/1153/1345/1537/1729/1921/2113）。
- **knee×4**（ZW_TIMEOUT_MULTIPLIER=4，页预算 240s）：**2,104 行**——预算 ×4
  行数不增（墙钟收割模型下两轮应等 N；二次成本模型下应 ~2× N）→ 墙钟模型唯一自洽。
- **wedge2**（divx 末行反推的候选象限，ShadowRoot 域 declarative/closed/df=true/
  clonable=true/s=undefined/empty=false）：单实例 12 步全 Pass（功能正确），2,500
  复实例 2,318 行全 Pass 后页级 Timeout——无固定楔死，功能面无新 Fail。
- **ABC 判相**（A 纯 mutation append/remove / B DSD 构建面 / C 纯读面，各 1,200）：
  3,600 行全 Pass 页面完整——单相不触发收割；二次总量需要重读面叠加。
- **ABC2 细分**（D 完整象限 1,200 / E build+getter 1,200 / F 资格面 1,200）：
  页死于 F 相 N=2,641（90s 收割）；D 0→33ms、E 31→64ms（跨相累积=页 N 驱动）——
  **增长载于 build+getter 面**（setHTMLUnsafe/attachShadow 重挂/getter），非纯读面。
- **IJ 终判**（I = 仅 setHTMLUnsafe / J = +attachShadow 重挂+4 getter，各 1,200）：
  2,400 行全 Pass 页面完整（总耗时 ≈70s < 90s 窗）。I 相斜率 0→24ms（~0.020
  ms/test²），J 相 23→53ms（边际 ~0.024 ms/test²，与 I 等速=继承累积）——**增长
  从 I 相（写路径构建面）就开始，attachShadow 重挂与 getter 面边际平坦非驱动**。
- **I12 终分**（I1 = 仅 attachShadow 命令式 / I3 = 仅 setHTMLUnsafe 纯 div 无
  attachShadow / I2 = 两者，各 900，×1）：**I1 全程 0ms 平坦；I3 近平坦（≤1ms
  平台，边际 ~0.006→0）；仅 I2 组合增长 1→21ms 且边际超线性（0.022→0.044
  ms/rep）**。单面均非驱动、组合才增长 ⇒ 增长载于「命令式 attachShadow 的
  ShadowRoot 句柄上 setHTMLUnsafe 声明式内容」组合写路径（= oracle 的
  ShadowRoot-domain × declarative 象限，约占 gethtml 全量一半），元素域
  declarative 写（I3）平坦。交互项归属 engine 写路径，非队列扫描读面。
- **Q1 队列加速判别**（C0 = I2 / C5 = I2+10 plain mutations / C25 = I2+50 plain
  mutations 各 600，×1）：C0 4→11ms（边际 ~0.025）；C25 **250 测试内即达
  229ms/rep**、页收割点缩至 1,451 行（C0 同窗 ~2,2xx）——队列加速 ~11× 使增长
  加速远超线性（与「每调用 O(队列) 扫 × 增量重建复合」一致），**残面 O(N) 与
  队列长度同源坐实**（C5/C25 斜率含 GC/构建噪声，方向无歧义）。
- **代码级机制定位**（只读侦查，与 Q1 互证）：`__zw_child_nodes` R5206 回落——
  sel 在 raw/live 缺席时走 `with_query_view_doc`（callbacks.rs:89）；ShadowRoot
  句柄的 sel 在选择器空间恒缺席 → setHTMLUnsafe shim 首步 `_childNodeList`
  removed 快照读每测试落视图路径。视图缓存（callbacks.rs:170 起）链增量资格检查
  `mut_guard[..key.count].iter().any(SetInnerHtml)` **无 SetInnerHtml 时短路不了
  = 每次视图调用 O(全队列前缀) 线性扫** → 每测试 O(N)、总量二次。t8c WIP
  （+390 行已核对）不覆盖此链扫。

## 修复后复测确认（预测→重建→观测闭环）

链尾在 rebuild 后（05:56 build6：zero-engine 重编=t8c WIP 在树 + zero-wpt-runner
重编=本项看门狗随乘在树）补跑 ×4：

- **gethtml.html ×4**（script 窗 360s）：**3,417 行**（3,413P/3F/页级 Timeout）
  vs ×1 同 binary ~2,0xx——行数随预算延伸，且 F=3 恰为 referenceTarget 序列化
  真实 Fail（成本伪影排除）。
- **knee2 div×8 ×4**：**3,072 行全 Pass 页面完整**（div×8 全量=3,072）——×1
  时同探针 2,168 行截断，随乘后全量真实上报。
- **knee3 div×16 ×4**：3,616 行 + 页级 Timeout（全量 6,144；截点反推斜率
  ~0.055 ms/test²，内容更重）。
- **二次模型定量自洽**：360s 窗 × gethtml 斜率 ~0.062 → 截点 ≈3,4xx ✓；× knee2
  斜率 0.037 → 3,072 内完成 ✓；× knee3 斜率 0.055 → 截点 ≈3,6xx ✓。二次累积
  仍在（knee3/gethtml 未跑完全量）——延伸只能来自窗放大，不能来自二次消失，
  两方向互斥验证。
- 残面归属：knee×1（含 t8c WIP 的 binary）仍截 ~2,1xx——t8c 索引未消除 build
  面（写路径）增长，与 IJ 判定一致（增长在 I 相写路径，不在队列扫描读面）。
  I12 终分见下补记。

## 修复面与协同

- **t8c（html-semantics 流，callbacks.rs 增量索引）**：队列扫描读面（tag/attr/
  text/innerHTML latest-wins）O(len) 逆序扫 → 增量哈希索引；其注释自带同病实测
  （「200 append 触发 44 万次查询、831ms/1098ms」）。与本轮 knee 曲线同病归一。
  本轮遵守 §9 碰撞纪律未触碰 callbacks.rs/js_dom_bridge.rs，待其落地后重建复测。
- **本轮落地（tests/wpt-runner/src/testharness.rs）**：`script_timeout_scaled_ms()`
  = `script_timeout_ms() × timeout_multiplier()`（wptrunner `--timeout-multiplier`
  语义：script timeout 与 case/page 预算同源放大；缺省 ×1.0 零行为变化）。
  R350 先例明确放宽超时不治二次病根——本项只是 mega-case 页预算语义补全，
  病根修复归队列面（t8c + 残面评估）。
- **残面修复方向（Q1 + 代码级定位后收口）**：病根 = ShadowRoot 句柄读每测试
  落 `with_query_view_doc` 视图路径 × 视图缓存链增量资格检查 O(全队列) 线性扫。
  修复候选：①链资格检查 O(1) 化——SetInnerHtml 出现次数单调计数器（入队时维护，
  t8c 增量索引同款手法），链扫消失；②ShadowRoot 句柄读走 live-aware 直读
  （句柄→NodeId 解析已有 HashMap，绕过选择器空间缺席回落）。两者均在
  callbacks.rs = t8c 编辑面，**须与其协调落地**（本轮按 §9 只定位不改）。t8c
  落地后重建 knee/gethtml 复测；队列折叠（apply 全队列 → dom_html 换代 → 清
  队列，R348 重绑同款语义）保留为兜底。

## 修复后预期

- 看门狗随乘已落地：×4 下 gethtml.html 已达 3,417/4,5xx 行且仅 3F（真实
  referenceTarget 序列化 Fail，簇 C 修复中）；剩余缺口 = 二次累积。
- ShadowRoot 域 declarative 写路径残面修复后（t8c 落地重建复测验证），×1 页
  预算内应能全量真实上报 ~4,5xx 子测试（现状 ×1 仅 ~2,0xx 行）。
