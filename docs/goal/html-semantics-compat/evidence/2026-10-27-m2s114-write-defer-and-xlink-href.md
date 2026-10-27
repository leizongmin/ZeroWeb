# s114：write 面 defer 改道 + xlink:href 回退——execution-timing 096/142 翻绿

日期：2026-10-27。目标域：`scripting-1` execution-timing write 面族。单文件轮：
shim `part06.js`（`__zwDocWriteRunFace`）。

## 入场定谳（三候选根因）

控制面候选 142/143/096 各 1F，失败断言定谳：

- **096** `document.write("<script defer src=...>")`：write 面把写入外链全走
  「取回即执行」阻塞链——got `[#1, external#1, #2]`，spec 要求 defer 外链并行
  取回、the-end 相位（interactive 后、DCL 前）执行。
- **142** SVG inline script `document.write("<script xlink:href=...>")`：write 面
  只读 `src` 属性，xlink:href 写入形外链**根本不执行**（eventOrder 缺
  `external script #1`）。
- **143** SVG 嵌套 inline script：write 注入的多余 `</script>` 需关闭外层 SVG
  script 提前执行（外层以已有文本子节点求值）——write 片段解析与主解析器
  open-element 栈耦合，深层面，**本轮不动挂账**。

write+defer 语料全树仅 096（102 命中 grep 但 defer 只在标题文字）；xlink:href
写入形仅 142——爆炸半径封闭。

## 修复（part06.js write 面）

1. **defer 改道臂**：src 槽位带 `defer` 且无 `async`、`__zwSchedRegister`
   在场（runner 环境）→ 不走阻塞链：立即 `fetch`（并行语义，挂 s110
   `_zwScriptFetchInFlight` 计数供 loadStep5275 load 等待），代码到齐入
   `__zwDocWriteDeferQueue` FIFO；排水单元全局只注册一次，投
   `__zwSchedRegister('defer', 0, fn)`——runner 生命周期 interactive 后排水
   （s112 布防点实时查 `__zwSchedHasDeferred()`，运行期注册可见）、同任务派
   DCL（spec the-end 序）。取回晚于排水（慢源）置 `DeferDrained` 立即补执行。
   门保持：nomodule / `_zwScriptTypeClassify` blocked（s113 门不变式——无
   新增无门执行路径）；throwaway（嵌套求值窗）槽位仍跳过留主链；
   `__zwSchedRegister` 缺席（非 runner）维持既有阻塞链。
2. **xlink:href 回退**：src 读空后回退读 `xlink:href`（回放物化 HTML script
   带字面前缀属性名）；阻塞链副本创建处全属性透传后补 `setAttribute('src')`
   ——既有 fetch 链零改动复用。src 在场形透传已带，仅回退形落重写。
3. **open() 记账重置**：`DeferQueue/DeferArmed/DeferDrained` 随既有 s97 记账
   同步重置（新写流独立）。

## 验证

- 定向：096、142 各 1 子测 Pass（/tmp/s114-096.log、/tmp/s114-142.log）。
- **execution-timing 全子树 A/B**（150 页 vs s113 基线归一 per-page diff）：
  唯一变化 096/142 Fail→Pass，行数恒等 138 页行，零 P→非P
  （/tmp/s114-exec-timing.log）。
- **全量 testharness 套件 A/B**（同协议，test-guard 包裹，
  /tmp/s114-full-suite.log）：6546P/1221F/111T → 6545P/1220F/112T，行数对账
  7878→7877 无蒸发页。归一 per-page diff 三处：096/142 F→P（目标）；
  focus-dynamic-type-change-on-blur 3 子测 P→T——**负载性 flake**（零
  document.write，同二进制定向重跑 80P/0T 复原，/tmp/s114-focus-retry.log）；
  modulepreload-cross-origin-referrerpolicy 第三行基线即 2F+1T、满量跑漂 3F、
  定向重跑复原 2F+1T（既有不稳子测，零 write）。**全树零 P→非P 成立**。
- Rust/依赖零变更（include_str! 重编即生效）；无新增 fmt/clippy 面。渲染面
  零依赖——reftest/product-smoke 未跑。
- scripting-1 累计：865P/140F/37T+15N → **867P/138F/37T+15N**。

## 挂账

- **143**：write 注入 `</script>` 关外层 SVG script——write 片段解析（R5001
  innerHTML 重放）与主解析器 open-element 栈耦合；外层 SVG script 需按已有
  文本子节点提前求值（SVG script 文本拼接语义）。
- defer 改道近似：多 defer 写入按取回完成序近似写入序（spec 文档序）；
  interactive 后注册的排水单元无二次布防（流后段 defer 写入不排水）。
  语料均无此形。
- part06.js 18436 行（既有超 2000 行限，本轮 +51 行未拆——拆分需独立
  切片按职责走）。
- 其余承袭 s113 挂账不变（063 Timeout、non-external-no-import、parse-error
  族、referrer-policies 等）。
