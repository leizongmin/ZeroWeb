# M2 s147 · 刀 B「runner 同键多行」根收口——testharness 注册 API 子文档 realm 隔离（同路径自导航递归注册泄漏）

日期：2026-10-29 · 切片：html-semantics M2 · 前序：s146（iframe.src pending 导航门）

> 过程注记：本切片勘根与施工主体在上一 session 完成（429 限流中断于
> 全量验证后），本轮从工作树现场（part01.js 未提交改动 + /tmp 验证
> 产物 zw-s147-*）接续：复核 diff、重建 join、补齐定向复现与全部
> 门禁，证据口径以本轮可复现命令为准。

## 入场定刀位

s146 下一步候选①：刀 B runner 多跑机制勘根（多跑/首遍挂/键集报法
漂移三面收口）。在册失败形态：

- form-submit-action 全量 50 行 = 10 案 × 5 注册（1 组 Pass + 4 组
  Fail）；s141 记录「runner 执行 3 遍 30 行=10 案×3，遍 1/2 全挂
  gEBI click not function、遍 3 才 8P/2F」同型。
- src-then-lazy-load / link-multiple-load-events 键集随负载漂移
  （s144/s145/s146 三轮在册文件级 Timeout 报法漂移）。
- details 族 lib flake ≈40% 翻转率（s145 强化观察）。

## 勘根（探针七轮二分 + runner 架构核对定谳）

1. **「多跑」不是 runner 循环重跑**：`run_html_semantics_cases`
   （testharness_html_semantics.rs:144）每文件**只执行一次**
   `run_testharness_html`，产出 `Vec<HarnessSubtestResult>` 按 harness
   注册逐条打印——同键多行 = **同名多次注册**，s141 的「runner 执行
   3 遍」旧解读修正为「同 sandbox 重复注册 ×N」。
2. **注册泄漏通道 = 同路径自导航递归**：form-submit-action 的
   「missing form action」案（formAction/submitterFormAction/
   formOwnerAction 全 null，corpus:35-40）提交 URL = 文档自身地址
   → target iframe 装载**父页副本**；嵌套副本的脚本体（R206
   `_zwRunIframeScripts` 合并 Function）裸标识符 `test()/
   async_test()` 解析既有近似落 globalThis（宿主 harness 所在）→
   宿主 harness tests 数组重复注册 ×N → 嵌套副本继续 populateForm/
   提交 → 递归（s146 实测 4 层 Fail 注册 + 1 层清洁 Pass）。
   上游语义：iframe 文档脚本解析 test() 走**子窗自身全局**（独立
   realm）——无 harness 即 ReferenceError，永不可达父窗 harness
   （https://html.spec.whatwg.org/multipage/nav-history-apis.html#the-window-object）。
3. **首遍挂 / 键集漂移同根**：嵌套副本的 loadPromise/导航链挂死 →
   harness 永不 done → 文件级 Timeout；挂起时序下注册子集上报 →
   键集随负载漂移（src-then-lazy-load、link-multiple media 键同型
   报法漂移的在册观察）。探针链：probe1-4（修复前/部分变体）恒
   4×Fail 注册，probe5-6（影子落位）1×Pass 收敛。

## 施工（part01.js 1 处，+36/−2）

`_zwRunIframeScripts` 页面段求值（R206 combined Function）**前置
影子声明**：

- **激活门**：嵌套文档路径（`pageUrl` 去 query/fragment）== 顶层
  case 路径（`globalThis.location.href`）——仅同路径自导航形态；
  异路径 helper iframe（runTestsFromIframe 模式，
  dynamic-imports-credentials 族）的嵌套 test() 落宿主 harness 是
  既有承重契约（结果经宿主回调上报），保持不动。
- **影子**：18 个 harness 注册 API 名（test/async_test/promise_test/
  generate_tests/setup/promise_setup/done/on_event/step_timeout/
  add_start/test_state/result/completion_callback/
  promise_rejects_js/dom/quotaexceedederror/exactly）各生成一个
  Function 作用域声明，**声明式委派**：调用时 `window[name]` 取值
  （harness 段先于页面段求值，prologue 直捕会拿到 pre-harness
  undefined）；子窗无该名 → `ReferenceError` 中止调用——per-part
  try/catch（R206 既有隔离）接住，副本页面段首个注册调用即止，
  populateForm/再提交级联源头掐断。
- **安全边界**：影子声明仅本 Function 作用域；exportSuffix206 扫描
  parts 原文不含影子名——无 win 回写、无自委派环；页面段同名
  function/var 声明照常覆写影子（上游局部声明语义）；门未命中
  （异路径/相对 pageUrl）时拼入串为空串，corpus 面逐字节不变。

## 验证（本轮复现）

- 定向 form-submit-action（`make testharness-html-semantics
  FILTER=forms/form-submission-0/form-submit-action.html`）：
  **10/10 全 Pass 单清洁执行**（修复前 s146 全量 50 行 = 40F+10P
  5 注册；s141 记录遍 1/2 全挂 gEBI click not function——旧版因
  目标行为失败证据 = /tmp/zw-s146-full.log 该文件 50 行块）。
- 定向 src-then-lazy-load：**8/8 全 Pass**（s146 全量 7P + 文件级
  Timeout 行；本轮全量该文件 8 键 Pass，文件 Timeout 未复现）。
- 定向 link-multiple-load-events：media 键 **Fail 确定性复现**
  （s146 该键被文件挂起吞没——键集报法漂移解除面，非代码位移）；
  文件级 Timeout 保持（独立挂账面，该文件无表单不自导航，与本轮
  级联根无关）。
- **全量 join**（对 s146 /tmp/zw-s146-full.log，(key,state) 对
  sort -u 去重）：s147 **7071P/812F/97T** = 7980 结果行（s146
  8019 行，Δ = −40 form-submit-action 级联 Fail 行 +1
  link-multiple media 行）；键对 **7986 → 7977**，位移 13 条全归因
  **零 P→F**——lost 11 = form-submit-action 10 个 Fail 对（键级真
  翻红：同键 Pass 对两轮均在，Fail 态为级联注册假象态）+ src-then-
  lazy 文件 Timeout 对（T→P）；gained 2 = src-then-lazy 第 8 键
  `srcset = /images/not-found 2 time(s)` Pass + link-multiple media
  键 Fail（s146 挂起未上报键）。
- 门禁：clippy workspace --all-targets -D warnings 零告警；shim
  13-part 规范序拼接 node --check 过（109701 行 = s146 109667 +
  34 净增）；engine lib **3592/10F**（总数 3602 守恒；9 名单与
  s146 逐字节相同——8 已知环境面 + R3045 归口 net-api；details
  在册间歇本轮采样红，s145 ≈40% 翻转率观察同型）；runner bin
  **258/1F**（trend_csv_check 行 196 归口不变）；本轮零 .rs 改动
  （fmt 无新面）；release runner 二进制（14:35 构建，早于 14:44
  全量日志）`grep -a` 核验 `_zwRealmShadow` 在执行体内；二进制构建
  后的注释措辞整理不改行为（eval 源内注释惰性）。验证集对齐
  s142/s145/s146 同面先例（定向 + 全量 join + lib + runner bin，
  无 reftest 义务——行为面非渲染）。

## 挂账（本切片更新）

- **刀 B「同键多行」面收口**（本轮主刀）：多跑/首遍挂/键集漂移三
  面同根（同路径自导航递归注册泄漏），单点收口。
- **残余理论面 2 条**（低优先，踩坑语料再补）：①with(window) 包合
  分支（页脚本含 on(message|popstate) 赋值命中 R206 包合）下影子
  在 with 对象环境内层，若子窗 win 暴露同名 harness 属性则 with 查
  找先于 Function 作用域影子——既有全量绿面未踩，未验证；②影子名
  单为注册 API 子集（非注册类 harness 全局不在列），上游 realm 全
  隔离语义更宽，按语料踩坑扩面。
- **刀 B 余面拆账**：details 族 lib flake ≈40% 翻转率（非级联根，
  s145 归因实验定谳非确定性调度）保持独立勘根挂账；link-multiple-
  load-events / csp-blocked 文件级 Timeout 族保持独立挂账（非自
  导航形态，本轮 join 证实与级联根无关）。
- 承袭不变：C2 input-type-change url/email 2F；engine lib 9F 环境
  面；runner trend_csv_check 行 196（归口 encoding 流）；「真实再
  物化」观察项；无锚链碰撞理论面盘点（刀 D 残余）；R102 live_aware
  回调簇 pending 失明理论缺口；待决三项（css/text 语料资源、
  green-2x2.png、parser form pointer 缓行 2026-10-23）。

## 下一步（s148 候选）

①全量基线翻拣下一单根面（63F 目录基线面之外的文件级 Timeout 族
——link-multiple / csp-blocked 报法漂移同面，刀 B 余面拆账入列）；
②无锚链碰撞理论面影响面盘点（刀 D 残余，无语料面，盘点性质）；
③encoding 流 M1 进度对账 form 面 46 文件能力缺口（不施工，仅对账
边界）；④form-submission-0 余下基线 Fail 面单根翻拣（urlencoded2
6F / text-plain 6F / multipart 6F / submission-checks 3F /
input-type-change 2F）。
