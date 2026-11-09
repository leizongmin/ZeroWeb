# ⑩ 轮 evidence：styleWithCSS 编辑上下文门——qS-before bootstrap 污染清根（读数面族 A 刀）

日期：2026-11-05。⑨ 轮残差②「读数面族」的 A 族主刀轮。单刀：`crates/engine/src/js_dom_shim/part06.js` execCommand stylewithcss 分支加编辑上下文门控。判决：五语料 + justifycenter 对 fix9/b0 基线 **F2P 9 / P2F 0 / newF 0 / lost 0**（multitest 守卫见 §5）；enginelib 终核 **3,727P/5F** 已知红逐名恒一（s249 CSSOM 族 4 + p56）零回归。三腿门禁全绿收口。

## 1. 根因链（A 族 qS stylewithcss-before quirk）

37 个 editing/run 语料壳页带页级 bootstrap（fontsize.html :33-39 原文）：

```js
try { document.execCommand("styleWithCSS", false, "true") } catch(e) {}
```

该调用发生在页面加载时——**无选区、无 editing host**。Chrome 录音环境此调用未生效（Chromium 命令 enablement 门：无选区不可编辑上下文 → IsEnabled 失败不应用），证据是 fontsize 语料录音反推：case 1（`foo[]bar`，无 styleWithCSS setup）之后 case 2 的 `queryCommandState("stylewithcss") before` 录音值 = **false**——若 bootstrap 生效应为 true。

本实现此前无条件应用（part06.js 原码：`globalThis.__zwStyleWithCSS = …`）→ bootstrap 置 mode=true → case 1 的 fontsize 命令入口把滞后快照 `__zwStyleWithCSSQuery` 同步为 true（justifycenter 250/250 实测的滞后一拍模型）→ case 2+ 的 before 读数污染。四语料各 1F 同 fixture（`<p>[foo</p> <p>bar]</p>` qS before，expected false got true）：fontsize/fontname/forecolor/hilitecolor。

## 2. 修复（单刀）

part06.js stylewithcss 分支：模式应用前加编辑上下文门——沿用 `_zwQueryCommandState` 编辑命令 enabled 同款判定（`_zwExecCmdFindEditingHost(rng)` 沿 startContainer 祖先链找首个 contenteditable=true host + 选区 endContainer 落 host 内）：

- 门内（语料 case 内 setup：选区已落在 editing host）→ 应用值，返 true；
- 门外（页级 bootstrap：无选区无 host）→ no-op，返 false（Chromium IsEnabled 失败 Execute 返 false 同型）。

滞后一拍快照模型（execCommand 入口 `__zwStyleWithCSSQuery ← !!__zwStyleWithCSS`，:4375）不动。消费面审计（09:5x 核验方勘误修正）：mode 消费面有两类——查询面 `queryCommandState` 滞后快照一处（`return !!__zwStyleWithCSSQuery`）+ **apply 面 `_zwVaCssFlag()` 三处**（part01d :755 forceValue carrySrc raw-attr 保留、:879 wrap 形 font-vs-span 决策、:1165 peel 门）——初版审计 grep 误用 `_zwCssFlag` 函数名漏检，multitest 15 行 compare innerHTML F2P（全为 value 命令 + inserttext 的 forceValue wrap/carry 案例）实证 bootstrap 污染同时扭曲 DOM 输出形，门控后归 Chrome 录音形；other 通道 103 案零 styleWithCSS 引用；集成/桥接测试面（part06.rs :1690 事件计数、:2059 supported/enabled）不经门路径。

## 3. 探针证据（zz-probe-qsq.html，入库为 A 族回归见证）

新增 `tests/wpt-runner/wpt-data/editing/run/zz-probe-qsq.html`（真 testharness `test()` 面，S0-S4 九断言）：复刻 fontsize.html bootstrap + tests.js case1→case2→case3 序列，逐步断言 Mode/快照/三查询。

- 修复前（r0-probe-prefix.log）：红 3 / 绿 6——S0（bootstrap 后 mode 污染 true）、S1（case1 入口快照污染 true）、S2（case2 before qS=true 应 false）；
- 修复后（r3-probe-postfix2.log）：**9/9 全绿 rc=0**——S0 falsy + 新文档 qS=false、S1 快照 false、S2 三查询合录音、S3 门内 setup 返 true + 快照 true + after true、S4 case3 before stale true（滞后模型保真）。

探针工程两笔教训（首版零行）：① 自定义 `row()` div 输出不经 testharness 计数——runner 只认 `^(Pass|Fail|Timeout)` 行；② `editing_case_skipped` 对源码不含 "testharness" 字符串的页静默跳过（testharness_editing_corpus.rs :55）——页须真引 testharness.js。

## 4. 判决（judge-fix10.py，行级元组口径）

| 语料 | base（fix9/b0） | post（fix10） | F2P | P2F | newF |
|---|---|---|---|---|---|
| fontsize | 2414P/8F | 2415P/7F | 1 | 0 | 0 |
| fontname | 2221P/4F | 2222P/3F | 1 | 0 | 0 |
| forecolor | 2309P/18F | 2310P/17F | 1 | 0 | 0 |
| backcolor | 1026P/0F | 1026P/0F | 0 | 0 | 0 |
| hilitecolor | 1213P/5F | 1214P/4F | 1 | 0 | 0 |
| justifycenter | 6073P/130F | 6078P/125F | 5 | 0 | 0 |
| **合计** | | | **9** | **0** | **0** |

四语料 F2P 各恰 A 族行（同 fixture qS before 翻 Pass）。justifycenter 追加 5 面 qS 读数翻转（?1001-2000×2、?3001-4000、?6001-last×2，含 before/after 两向）——bootstrap 污染的实际受害面大于 ⑨ 轮控制面记账的 ×4 口径；滞后面（case 间 stale 链）零位移，250/250 建模口径不受门影响。五语料残差 **35F→31F**；justifycenter 130F→125F。

## 5. multitest 守卫腿

10 切片协议（run-fix10-mt.sh，每片 TIME_LIMIT=1200，flock 单实例锁；ALL-DONE 09:41:37）合并 10,092 行对 base-multitest.log（9,812 行共享分母）行级元组。精读口径（judge-fix10-mt.py + 共享 key 切分）：

- **共享前缀（9,811 key，99.99% 分母）：F2P 15 / P2F 0**——15 行全部 `foo[]bar` compare innerHTML（fontname/fontsize/forecolor 无 setup case，?5001-6000 ×8、?6001-7000 ×6、?7001-8000 ×1，恰对应片级 fails 57→49/65→59/48→47 的 -15）：bootstrap 污染不限于 qS 读数——无 setup case 被 boot 成 CSS 模式跑 tag 形期望的渲染链，门清污染后翻绿。A 族实际受害面大于六语料 qS 记账口径。
- **post-only 281 key 全落 ？9001-last 截断带**（base 尾片 813 行+文件级 Timeout 截断，post 干净完走 1,092 行）：含 31 fail + 250 pass；31 fail 与 fix9 轮尾带 fail 数同族同数（forwarddelete+insertText 色壳 caret 族首执行行，⑨ 轮已判档骨架 flake 族，非行为回归）。lost 1 = base 尾片 Timeout 行（post 无）。
- 片级：6/10 片逐位恒一，3 片 F2P 集中，0 片恶化；multitest 残差 423F（⑨ 轮 438F→423F）。

工程事故留痕（处置即验证）：① full-file 单发 0 行失败已归档（.fullfile-failed-090414，3.5KB make Error 1，fix6 时代 full/full2 同形状）；② 并行活跃轮 stash 本轮草稿 + 无门重建二进制（09:04）致首轮切片（09:04:56 跑批）测无门树——执行体证据纪律（grep swEnabled 二进制字符串 =0）揭穿后全批作废重跑；③ 双实例同写语料互踩（09:14 + 09:19 两实例同秒双 runner）全批作废——run-fix10-mt.sh 加 flock 单实例锁后 09:21:25 批干净完走。教训沉淀 docs/learnings/platform/2026-11/2026-11-05-multitest-corpus-full-file-mode-fails-use-slices.md。

## 6. fix9a 拒绝墓碑（上轮 429 中断在飞项判决补记）

上轮兄弟实例 429 死前落地的 fix9a（part01d `_zwVaCanonicalizeAncestorStyleAttrs` 调用点收窄 + split pass 前移，未提交工作树遗留）经双证否定 git checkout 丢弃：

1. **语料权威**：backcolor.js 录音期望（L193/L233/L238/L243/L253）全为 **rgb 归一整壳形祖先**（非 fix9a 假设的 keyword 保留形）；L263 探针孪生期望嵌套 rgb 归一双壳形。
2. **实证批**：fix9a 树五语料复跑（fix9a-status.txt）fontsize 8F、fontname 4F、forecolor 20F、**backcolor 0F→2F**、hilitecolor 7F——五语料合计 41F 对 fix9 的 35F 净 +6F，backcolor 回归与语料期望矛盾双重否定。

树回 fix9（HEAD 10f15f5be）。target/release/zero-wpt-runner 曾被兄弟重建含 fix9a 代码——按执行体证据纪律（guideline #28）本轮门禁前全部重建（make 入口自动 rustc+cargo build，fix10 批首腿 fontsize secs=89 含重建）。

## 7. 工件

- 判决：/tmp/qv-fix4/fix10-{fontsize,fontname,forecolor,backcolor,hilitecolor,justifycenter,multitest,enginelib}.log；基线 fix9-*.log / b0-justifycenter.log（stash 出修复前二进制直跑）/ base-multitest.log
- multitest 切片：fix10-mt-part-{1-1000…9001-last}.log ×10（合并入 fix10-multitest.log）；作废批归档 fix10-multitest.log.fullfile-failed-090414（full-file 0 行）、fix10-multitest.log.gateless-invalid-091917（无门二进制批）
- 判读器：/tmp/qv-fix4/judge-fix10.py（六语料元组）、judge-fix10-mt.py（multitest）+ 共享 key 切分
- 探针：r0-probe-prefix.log（红基线）/ r3-probe-postfix2.log（绿收官）/ r2-probe-postfix.log（S0 undefined≠false 中间形，引导 S0 断言改 falsy 语义——引擎惰性 undefined + 读点 `!!` 归一是既有风格，不为探针加初始化点）
- 状态：/tmp/qv-fix4/fix10-status.txt（含 GATELESS-RUN-DISCARDED / DOUBLE-RUN-DISCARDED 事故行）
