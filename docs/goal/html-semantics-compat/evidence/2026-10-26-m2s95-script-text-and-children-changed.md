# M2-s95：script.text IDL + script children-changed→prepare 记账（12F 翻绿 + 克隆族解挂 4 页）

日期：2026-10-26。归属：html-semantics-compat / s86 切片序③ the-script-element 基面（承接 s94，HEAD 47bc3dffd）。

## 1. 面与基线

s94 HEAD 基线（/tmp/s94-final-scan2.txt，FILTER=scripting-1 口径 574 行 263P/270F/23T + 18 NotRun）。入场探针自 270F 聚类选 runner 可达 classic 脚本文本语义两簇：

- **script-text.html 6F**：HTMLScriptElement.text getter（child text content）/ setter（string replace all）——全 detached `createElement("script")` 纯 IDL 面（Getter 排 comment/PI/元素子；Setter 四值臂 + text node reuse 建新节点）。
- **emptyish-script-elements.html 6F**：解析期空源脚本 bail-early 不置 already-started → 后补 Text 子**须执行**；有源/空白源脚本已启动补文本 no-op（7 臂 6 红，no-op 臂白捡绿）。

排除（runner 上限）：script-charset-01 6F（serve-with-content-type.py 执行器）、script-text-modifications{,-csp} 2F（slow.py `pipe=trickle` 时序 + stylesheets-blocking 面）。

## 2. spec 依据与刀位

- **text IDL**（webReader 取回 #dom-script-text 原文）：getter = child text content（**直接**子 Text data 依树序拼接，comment/PI/元素子不计——语料断言 `.text="  TEXT  "` 而 `textContent="  TEXT  ELEMENT"`）；setter = string replace all，普通 DOMString 非 LegacyNullToEmptyString——null → `"null"`、undefined → `"undefined"`（Setter(null)/(undefined) 双臂）。
- **children changed steps**（#script-children-changed-steps + whatwg/html#3419）：未连接直接返回；post-connection 步骤对 parser-inserted 元素跳过——但空源 bail 发生在 already-started 置位前、且 prepare 前置步骤已清 parser document，故解析期空源脚本补文本**须执行**、有源脚本已启动 no-op。

### 刀①：script.text IDL（shim）

- get trap（part03）：`prop === 'text'` + SCRIPT 门——直接子 Text walk（R5269 title.text 同口径；无 lw 回落——脚本子文本即执行源，树视图权威）。
- set trap（part04）：null → `"null"` 归一后委托 textContent setter（R3907 option.text 同口径——子替换/文本子物化/MO 记账全继承；text node reuse 建新节点同 spec）。

### 刀②：children-changed → prepare（shim part04 + part06）

- **appendChild trap 头部**：SCRIPT 父记 pre-mutation child text content 快照（`_zwS95ChildText`——页面可见融合子视图直接子 Text 拼接，仅 SCRIPT 父快照其余 appends 零成本穿透）。
- **尾部触发块**：pre 空 + post 非空为执行候选 → connected 门（spec 步骤 1）→ **already-started 门** → nomodule 门（R387 同口径，不置 run-once——中止在置位前）→ classic eval（R387 内联臂同面）+ 执行后置位。
- **already-started 记账**：part06 `__zw_set_current_script` 投递时按元素 `__zwSelector` 记 `_zwScriptStarted`——空源 bail 脚本 extract 期跳过不投递（不记）、nomodule/for-event 门压制同样不投递（不记），与 spec prepare 置位时机对齐；handle 形（动态插入/克隆产物）沿 R387 `_zwRanScripts`。
- **script cloning steps**：cloneNode 传播 already-started（R3901/R3902b input/textarea 克隆态传播同型位置）——started 源的克隆（sel/handle 双形判定）记 `_zwRanScripts[克隆 handle]`，浅克隆补文本 no-op 且顺带抑制 R387 对 started 克隆的重执行。

## 3. 回归与记账返工（本轮主发现）

首轮纯 pre/post 近似（无 already-started 记账）复扫目标页全绿，但全子树 diff 出 **execution-timing 059/060/062 三行 P→F**：

- **059/060**：执行中脚本自摘子（`removeChild(firstChild)` / `innerHTML=''`）再补文本——pre 快照在自摘后捕获成空串，近似把已投递脚本误判「未启动」，注入代码误执行进 eventOrder。
- **062**：started 脚本 `cloneNode(false)` 后补文本——克隆未传播 already-started，同因误执行。

返工即刀②的**投递时记账 + 克隆传播 + 执行后置位**（pre/post 降为第二道近似）。三页回绿，且连带解锁克隆/空脚本补文本族：**061/064/065 三页 file Timeout→全绿（解挂）、036 Timeout→subtest Fail 显形、066 F→P、127/128 F→P**。

## 4. 验证

- **定向**：script-text.html 6P/0F；emptyish 7P/0F；execution-timing 059/060/062 回绿、120-126 与 s94 基线逐行一致、127/128 Pass；探针 4 臂全绿（探针页临时，验后已删）。
- **全子树终扫**（FILTER=scripting-1，test-guard 包裹，同 ZW_TESTHARNESS_RAF_FRAME_DRIVEN）：/tmp/s95-final-scan.txt **281P/256F/19T + 18 NotRun**。diff vs s94 基线（name 键）：**F/T→P 14 行 + 解挂 4 页 + 066 F→P；P→非P 0 行**。14 行归因：script-text 6 + emptyish 6 + execution-timing 127/128 2；解挂归因：061/064/065/036/066 全部克隆族（刀②克隆传播连带）。域账自此按 **282P/256F/19T** 记（raw 281P + 惯例 +1）。**新基线 /tmp/s95-final-scan.txt**。
- **单测腿**（test-guard `--per-proc-mem 4 --total-mem 8 --time-limit 1500`）：zero-engine lib **3530/3530**。
- **fmt / clippy**：纯 `.js` shim 变更（无 `.rs` 触碰），rustfmt/clippy 不涉；编译与运行验证 = zero-wpt-runner 重建 + zero-engine lib 全测。
- reftest 未跑：无 render/layout 面（脚本文本语义/执行触发），沿 s91-s94 影响面口径。

## 5. 挂账残面（本轮不动）

- **execution-timing 编号套件 78F**：036 显形的 deep clone 族（克隆传播只盖浅克隆自身——后代 script 子树经 innerHTML 重建不在克隆步骤）、122-126 type/async 改属性面、005-012 document.write 族。
- **module 尾片多簇**（dynamic-import credentials/referrer-policy、charset、import-meta、css-module 等——.py 执行器前置）；script-charset 12F（.py）；svg-script-fragment-clone 3F。
- **nomodule-set-on-async-classic-script 2F**（window.onload promise 门）；load-error-events 族（元素 onload 投递保真，s94 sync 臂同账）。
- **FIXME(spec-gap)**：children changed 其余变异入口（insertBefore/append()/textContent=/.text setter 已连接面）未挂同一触发；语料全臂走 appendChild 先覆盖本入口。
- execution-timing 顶层 / 完整接口对象基建面（承袭）；parser form element pointer 4F（用户决策 2026-10-23 缓行）。
