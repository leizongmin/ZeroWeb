# M2 s278 — multiple-times 销案口径复核（签名级对齐）+ multicol 上游定性（零代码分诊轮）

日期：2026-11-04。切片号 s278（承 s277 下一步候选①：型 D 余红复核销案
口径）。零代码分诊轮（s84 先例），勘根→上游取证→spec 核实→口径落账。

## 勘根

1. **solo 现签名**（s277 三刀后复核跑，日志 s278-solo-multiple-times.log）：
   `assert_unreached: The loading=lazy below-viewport image should lazily
   load its second image, and not load it eagerly when the 'src' attribute
   is changed`——与 s272 记录同语义，s277 刀组（once-lazy 登记/在飞存活/
   noBlock 派发面）未改 restart 面：视口门 `!_imHadPrev` 项使已装载图换
   src 重启落穿 defer 直接装载。
2. **上游纵深 6 时点**（wpt.fyi /api/runs + /api/search，取证通道本轮打
   通——直连四轮超时，monocloud 代理 10801 后全通；recipe 见
   docs/learnings/platform/2026-11/2026-11-04-wpt-fyi-upstream-evidence-recipe.md）：
   2024-11 → 2026-11 每 3 个月采样，chrome 与 firefox 全部 0/1，
   **12/12 连续双红 18 个月**——长期分歧，非回归非抖动。
3. **现势 message**（GCS per-test 直键，revision 52041a16d3）：
   chrome 159.0.8140.0 / firefox 161.0a1 均 `assert_unreached`（换 src
   eager 装载）——**与本地失败逐字同签名**。
4. **红面漂移实证**：chrome 132（2024-11 同月 firefox 134 已是 restart
   面）红面曾是窗序断言（`assert_true: The loading=lazy below-viewport
   image should not block the window load event`）——上游红面随 build
   演进，「上游同红」类销案须按现势签名字面钉死，不能只记 0/1 计数。
5. **spec 定谳**（HTML standard 2026-11-03 snapshot 全文取证）：src 变更
   = relevant mutation（§4.8.4.3.2 Reacting to DOM mutations）→ update
   the image data 请求构造段 will-lazy-load 门：「If the will lazy load
   element steps given the img return true: Set the img's lazy load
   resumption steps to the rest of this algorithm starting with the step
   labeled fetch the image. Start intersection-observing ... Return.」
   will-lazy-load 条件仅 scripting enabled + loading attribute in Lazy
   state 两项——**无已装载豁免、无 restart 豁免**。spec 字面 = 测试期望
   （出视口 ever-lazy 图换 src 须 defer 至 intersection）；双引擎 shipped
   行为 = restart 绕门。**spec-vs-engines 长期分歧面**。

## 结论（销案口径升级）

- multiple-times **销案维持且升级签名级对齐**：本地红 = 上游 chrome/firefox
  现势红同 message（逐字），非我侧缺口；我侧行为与两引擎 shipped 行为同
  面，符合 Chromium-parity 兼容目标（该测试作者 Dom Farolino 为 Chromium
  lazy-loading 作者，测试期望是 Chromium 未兑现的 spec 意图）。
- **reopen 面**（若上游翻绿——引擎翻绿重开先例 s41→s53 dialog ESC）：
  刀形 = 视口门已装载臂纳入 `_zwLazyEver5277`（ever-lazy 且仍 loading=lazy
  且出视口 → defer），基建 s277 已落，单刀面低风险。
- 取证通道沉淀 learning（代理 + from/to 成对 + search 裸路径 q + GCS
  per-test 直键含 subtests[].message）。

## 顺路销账：multicol 上游主干补查

- s274/s277 挂账「multicol 上游主干状态待补查（wpt.fyi 三轮网络超时）」
  本轮销账：`image-loading-lazy-multicol.html` 上游 **chrome 1/1 +
  firefox 1/1 双绿**（revision 52041a16d3，同 multiple-times 取证轮）。
- 挂账性质改写：**非上游分歧面，我侧 runner 时序建模缺口**（页内 slow.py
  慢 async script 拖住 window load → in-viewport lazy 图 load 须先于
  window load；runner 同步快回无此窗口，settle 同步装载与延后装载结构性
  二选一——s274 裁定保 image-loading-lazy）。解除面不变 = runner
  trickle/async 慢流时序（与 s273 挂账② runner 建模限制并轨）。上游双绿
  证实断言语义两引擎可实现——runner 建模落地后可施工，无上游冲突风险。

## 挂账更新

- multiple-times 主干双红销案口径：**签名级钉死**（维持销案）。
- multicol 上游主干补查：**销账**（双绿定性，转 runner 建模缺口面）。
- 承前不动：multicol 窗序反向面（解除面 = runner trickle/async 慢流）；
  below/in-viewport-dynamic Chrome-only 口径；runner 全量钉内存大户二分
  （s270 挂账①）；root margin 净速分档（s271 挂账①）；dynamic import()
  referrerpolicy 穿参；decode() 兜底元组细化（s276 注记）。

## 下一步候选

1. runner trickle/async 慢流时序勘根（multicol 窗序反向面 + s273 挂账②
   族解除面——runner 时序建模面）。
2. runner 全量钉内存大户二分（s270 挂账①）。
3. root margin 净速分档（s271 挂账①）。
