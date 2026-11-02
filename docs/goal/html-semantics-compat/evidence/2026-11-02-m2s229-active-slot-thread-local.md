# M2 s229 — forms misc 散面聚类 → :active 簇勘根 → DOC_ACTIVE_SELECTOR 跨页串态修复（1F→P，A/B 单点位移闭合）

日期：2026-11-02。切片号 s229（承 s228）。

## 入场定刀位

s228 下一步候选首选 not-broken 2F——控制文档 s224 记录已归因 renderer 流车道
（RectBridge 快照无同步 reflow；handler 注入 `apps/renderer`），本车道不入侵，
转次选「forms misc 散面逐案勘根」。数据源复用 s228-forms-baseline.log
（forms 域 4492P/225F/10T），零重跑。

## 散面聚类（14 非门禁文件）

- **:active 簇 3F**：active-onblur 1F（button space 持键 `:active` 断言）
  + checkable-active-onblur-with-click 2F（parent label 压按）。同款
  checkbox/radio space 案与 for= label 压按案全 Pass——机制存在、边界断链。
- table/label 关联簇 3F（form.html 2F + form_owner_and_table 1F）。
- keyboard 3F（date-readonly / radio-nav-order / radio-scroll-into-view）。
- 语料缺口 1F：color.window.js——`/css/support/color-testcommon.js` 缺失
 （support 目录只有 computed/inheritance/parsing 三 testcommon），同 s227
  语料资产缺口类，挂账不进引擎。
- 散单 7F：email-set-value、input-type-change-value 2F、
  checkbox-switch 3F（tentative switch IDL）、valueasnumber、input-type-button、
  reset、inserted-or-removed。
- **同根对照**：14 文件无一使用 innerHTML 构建（静态 HTML 或
  createElement+appendChild=R100 收编）——s228 identity 断链根不命中，散面
  全部独立分型。

s229 定刀 ：active 簇（3F、两候选根、机制面清晰）。

## 勘根（探针 3 轮 + Rust 插桩 2 轮 + 基线对照 3 轮）

- **probe1（并测 A1/A2）**：button space 后 `qSA(':active')=b1`、
  `b1.matches=true`——**A1 未复现**；`lc.control=cb2` 嵌套关联 getter 完好；
  parent label 压按 pointer 序列 `disp(body,…)`——**hit 落 body 不落 label**。
  JS 侧 wrap `globalThis.__zw_set_doc_active` 零拦截（shim IIFE 闭包内同名
  引用绕过 globalThis wrap——插桩失明教训）。
- **probe2（复刻真实页 + rect/坐标插桩）**：嵌套 label `lcRect=(0,0,0,0)`
  **布局盒零尺寸**（内含 input 有正常 rect）→ origin+(2,2) 落绝对 (2,2) →
  命中 body；纯文本 for-label 盒正常 `(158,12.5,33,23.28)`、hit 正确落到
  label——但 `input:active` 仍 null（后证为跨页串态干扰，见下）。
- **Rust 插桩（终极仪器）**：`__zw_set_doc_active` 回调 + `sync_active_state`
  加 eprintln 跑真实文件——写入分布 `#b1`×1、`#checkbox/#radio` 各 1、
  `#cb1/#r1` 各 2、**`body`×2**（parent label 压按零盒命中写入）。
  button 页 gate 正确写 `#b1` 后 sync 全程见 `#b1`、断言仍败——标记链完好，
  失败在并发层。插桩时序扰动下失败形态翻转（got body ↔ got null）——
  同根两投影。
- **定谳**：`DOC_ACTIVE_SELECTOR` 为**进程级 `static Mutex<String>`**
  （js_dom_bridge.rs），按压态本质 per-document；runner rayon 多 job
  （8 线程，每线程独立 WebView 顺序多页）共享同一槽——并发页
  with-click 写入的 `"body"` 串进 button 页断言查询，在本页视图解析出 body
  假匹配；时序翻转时读到并发页清除 `""` 得 null。产品同进程多 webview
  同病，非测试假象。

## 施工

`DOC_ACTIVE_SELECTOR` static Mutex → `thread_local! { RefCell<String> }`，
与周边查询基础设施（`LIVE_QUERY_DOC`/`VIEW_DOC_CACHE`/`MUT_APPLIED_CURSOR`）
同一归属模型（callbacks.rs 既有注记：`zero_dom::Document` 非 Send，
thread_local 即正确归属）。同步更新声明处设计注释（static 串态实证 +
归属判据）。改动面：js_dom_bridge.rs（声明+sync）+ callbacks.rs（写回调），
共 ±20 行。

## 验证

- **定向守卫测试**（新增 `js_dom_bridge_tests/part95.rs`，随修复同提交）：
  他线程写入不串入本线程 sync + 同线程写读回路保持，
  `./target/test-guard … cargo test -p zero-engine s229_doc_active_slot_thread_local`
  1 passed。
- **基线判谳**（防回归误判）：stash 修复跑基线二进制单文件 3 连跑——
  following-label ×2 基线也 3/3 Fail（s228 全套件基线的 Pass 是负载时序
  侥幸样本），修复无回归引入。
- **域 A/B**（`make testharness-html-semantics` 全树，forms 域键级
  (key,state) 对 join）：4492P/225F/10T → **4493P/224F/10T**；lost/gained
  各恰 1 行 = `active-onblur :: Buttons should clear :active…` Fail→Pass，
  **零其他状态翻转**。
- clippy `-p zero-engine --all-targets -D warnings` 干净；fmt 干净。
- 探针验后删（zz-s229-probe.html 两版）；Rust 插桩 eprintln 已撤净
  （grep s229-trace 零命中）。

## 承袭挂账（更新）

- **parent-label 压按 2F（新根①：布局零盒）**：`<label>text <input></label>`
  嵌套 label 布局盒 (0,0,0,0)（内含 inline-block input 有正常盒）→ hit-test
  落 body → `__zwSetActiveFromSel` 从 body 找不到 LABEL → 标 body 而非
  control。修复方向：布局面 inline 容器含替换类子元素的盒碎片化；
  **render-foundation/layout-engine 流车道**，本车道不入侵。
- **following-label 2F flake（新根②：页内异步清写跨测试窗）**：Actions 命令
  异步队列下前一测试的 cleanup pointerUp/Tab keyup 清写可落入下一测试
  down→assert 窗（trace2：16 写 vs 期望 18、 tid 全 main 单线程）。
  全套件负载下侥幸 Pass、单文件稳定 Fail；testdriver 命令排空时序属
  runner/testharness 面，挂账待钉。
- **color.window.js 1F**：语料缺 `/css/support/color-testcommon.js`，同 s227
  补资产流程（上游 sparse clone + update-wpt-data.sh 挂账）。
- **散面余项**：table/label 簇 3F、keyboard 3F、checkbox-switch 3F、
  email-set-value 等 7 单案——全部独立根，未动。
- 其余承袭不动：innerHTML sel 容器 identity 架构（s228 三层计划）；
  lazy-loading ~24F；img naturalWidth SVG viewBox 22F；用户门禁三项；
  error.html 破图 alt 7F；sizes-auto Timeout 68；svg-img-density 7F；
  part06 `_zwFieldsetDisableFocusReset`（s215）；part03 `_zwMElCheckValidity`
  FORM 短路（s210）；onblur flake 族（s209~）；渲染门 FIXME（s218）；
  flat tree slot FIXME（s218）；modern API img 树变更余项（s221）；
  TreeMutated IMG 移除臂余项（s221）；mimesniff 裸空白 value（s222）；
  image-alternate-no-store（s225）。

## 方法论教训

- 「断言得到从未在本页出现的元素」先疑跨页/跨线程串态：单文件复跑 PASS
  而全套件 Fail，串态嫌疑大于引擎语义 bug。
- shim 侧 wrap `globalThis.__zw_*` 会被 shim IIFE 闭包内同名引用绕过——
  JS 插桩静默失明；全量可见性直接在 Rust 回调加 eprintln（带
  `thread::current().name()` 验线程归属）。
- 基线 Pass 单样本不足为凭：落修复前对疑翻转面跑基线二进制 3 连跑判谳，
  防把负载侥幸当回归。
