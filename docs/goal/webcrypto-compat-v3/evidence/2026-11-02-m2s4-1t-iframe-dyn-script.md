# M2-4 1T 销案：iframe 子文档动态 script 执行（2026-11-02）

## 范围与结论

销谱系挂账 1T（algorithm-discards-context.https.window.js，一/二/三期连续
T 案）：**13 子测试全 Pass，案级 Timeout 键退役，T 5→4**。全量分母
360 案 / 81,851 keys = P 52,781 + F 29,066 + T 4（m2s3b 锚
81,839 = P 52,768 + F 29,066 + T 5，净增 +12 keys 全 Pass）。
红线 P→非P **0 例**、F 逐位全等（29,066）。余 T 4 全部为 Hybrid-KEM
generateKey ×4（window/worker 双腿 × failures/successes，D1 域门控）。
**M2 无依赖面四切片至此全部收口**（M2-1 门禁面 / M2-2 散新案 / M2-3+3b
worker 腿 / 本切片 1T），余下域切片全部待 D1 拍板。

## 根因（M2-2 轮 triage 精化的实证收口）

12 个 async_test 全依赖 iframe 子 realm 脚本执行链：主文档
`runInChild` 建 iframe → `document.createElement("script")` 内联
preamble（`closeChildOnAccess` 给 algorithm 对象属性挂 getter）→
`child.contentDocument.body.appendChild(script)` 注入子文档 → 子文档
`crypto.subtle.*` 调用 normalize 时读 `.name` 触发 getter →
`top.closeChild(testId)` 跨 realm 回调宿主 → 移除 iframe + `t.done()`。

runner 此前断链在第一环：主文档动态脚本面（part04 s95/s97 trap +
`_zwScriptMaybeStartDynamic`）只覆盖主文档 appendChild 站；iframe 子
文档 body 是 `_makeDetachedDocument` 工厂合成体（`_zwMakeIframeDoc`
包装），SCRIPT 元素 append **静默不执行**——getter 脚本从未跑，
harness 记 `pending:13`（M1/M2-1/M2-2/M2-3/m2s3b 五锚一致案级 T：
"testharness completed without reporting registered tests
(state={loaded:true,pending:13,…})"）。

## 修复（`crates/engine/src/js_dom_shim/part05.js`，+68 行纯插入）

1. `_zwMakeIframeDoc` 工厂出口包装 `doc.body.appendChild`：R112 串行
   合并/通用挂树两分支记账语义先行（`_r206bBaseAppend.apply` 原样），
   返回前对 SCRIPT 元素子触发 `_zwRunIframeDynamicScript` 同步执行
   （同 spec prepare 的 append 完成即启动语义）。
2. `_zwRunIframeDynamicScript` 执行体：
   - **run-once 记账**复用主文档动态面双表（`_zwScriptStarted` sel 键 /
     `_zwRanScripts` handle 键——s95/s96/R387 任一执行过即 no-op，
     already started 语义同 spec prepare 步骤 1）；
   - **type/language 门**沿 s113 `_zwScriptTypeClassify`（非 classic
     不执行不置记账）；**nomodule 门**沿 s94 口径（-house 惯例：nomodule
     拦 classic，module 已被分类门拦截）；
   - 空内联体跳过；**外链 src 不执行**（FIXME 在案——既有 corpus 面
     markup 提取走 `_zwRunIframeScripts`，需求面出现时按
     `_zwPrepareSrcScript` 接线）；
   - **域绑定**沿 R206（markup 脚本合并 Function）同构：`window`/`self`
     绑 iframe win（defaultView，未物化时宿主全局近似）；`parent`/`top`
     恒绑宿主全局（跨 realm 回调 `top.closeChild` 面）；`document`/
     `location` 绑子文档。执行体 `new Function` + 错误走
     `_zwReportListenerError`（listener 错误上报既有通道）。

### 兼容性论证

- 包装作用域 = `_zwMakeIframeDoc`（iframe 子文档）**专属**；模板
  contents owner doc（part05 `_makeDetachedDocument('')` 直调）与
  `createHTMLDocument` 面不经过此函数，零触碰。
- 触发条件 = 对工厂 body 显式 `appendChild(SCRIPT)`；innerHTML/解析
  面不入站。全语料扫描 `createElement("script")` 动态面 8 文件
  （dom/html 两族）均为主文档站（既有 s95/s97 承接，本切片不改），
  iframe body SCRIPT append 在现库唯一消费面即本销案目标案。
- 包装以 `_r206bBaseAppend.apply(this, arguments)` 全参透传，返回值
  原样——非 SCRIPT 子的 append 行为逐位等价；SCRIPT 子追加一次同步
  `new Function` 执行（执行失败上报不回滚 append）。
- dom 通道 A/B 实证零回归（见下）。

## 验证阶梯

1. `node --check` shim 17 分片按 bridge 拼接序整体校验通过。
2. 定向：`make testharness-webcrypto FILTER=algorithm-discards` ——
   **13 子测试全 Pass**（此前同命令五锚案级 Timeout）。
3. 全量对账 vs [m2s3b 锚](2026-11-02-m2s3b-full.json.gz)（r3 =
   终版二进制，test-guard `--per-proc-mem 4 --total-mem 8
   --time-limit 3600`）：
   - **P→非P = 0**；F 逐位全等（29,066 → 29,066）；案数 360 → 360。
   - 状态迁移 14 keys 全落目标案：案级 Timeout 键 ×1 退役 +
     13 个子测试键 None→Pass（generateKey/importKey/digest/encrypt/
     decrypt/sign/verify/deriveBits/deriveKey×2/wrapKey/unwrapKey×2）。
   - 新分母 **81,851 = P 52,781 + F 29,066 + T 4**（净增 +12 全 Pass，
     T 5→4）。新锚 [m2s4](2026-11-02-m2s4-full.json.gz)。
4. 两连跑确定性：r1/r2 状态逐项全等（**0 变化**，81,851 keys 逐
   (case, subtest) 对照）；r3（终版二进制，含 R115 注释行恢复——
   与 r1/r2 二进制仅注释差）与 r1/r2 同账。
5. **dom 通道双二进制 A/B**（part05 为 engine 共享面，js-dom corpus
   是 iframe 工厂最重消费方）：`/tmp` 双二进制（HEAD 基线 stash 构建
   vs 本切片）各跑 2 次 `testharness-dom --json`（test-guard
   `--per-proc-mem 4 --total-mem 8`），2×2 采样：
   - **共享键零回归**：任一基线臂曾 Pass 的键在任一新臂非 P = **0**；
     Fail→非F = **0**。
   - 差异全落两类已知噪声：① Range-mutations dataChange/deleteData/
     replaceData 报告视界漂移（案级 page-timeout 中途收案，reported
     keys dataChange [440,468,481,476]、deleteData [485,478,487,476]、
     replaceData [487,475,489,478]——**基线二进制对自身即漂移**
     （base1↔base2 diff 47 keys ≈ base↔new diff 48 keys），R350 计时
     族 flake 判据）；② ParentNode-replaceChildren T 案相位漂移
     （new1 单发案级 Timeout 相位，其余三臂 2 子测试 T 相位——
     M2-3b A/B 已记账同形，双相位皆为既有态）。
6. `cargo test -p zero-wpt-runner`：**276 passed / 0 failed**（76 ignored；
   先 `--no-run` 预编译，test-guard `--per-proc-mem 8 --total-mem 8`
   包裹执行——M2-3b 阈值口径）。

## 谱系账

- 一期 33,109 分母红线全程保持（本切片零回归 + M1 以来四切片红线
  连续 0 例）。
- **1T 谱系挂账销案**：algorithm-discards-context 五锚连续 T →
  13/13 Pass，1T 不再存在（33,109 分母内最后一个非域门控非 P 案）。
- webcrypto 谱系余非 P 面：全部分母 Fail 29,066（新族域红面 +
  旧 `.tentative.` 路径 retired 面随收口重定基退役）+ T 4（Hybrid-KEM
  D1 域门控）——零移交边界内，无外挂挂账。
