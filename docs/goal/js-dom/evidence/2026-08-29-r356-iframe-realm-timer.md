# R356 — d3d-r2 前置切片：iframe realm 的 timer 面（`_zwMakeIframeWin` 定时器转发）

**日期**: 2026-08-29
**切片**: RFC v0.3 §6.2 路线 A 第二片（d3d-r2「iframe 树源统一」）的前置子片
**性质**: 补面修复（iframe realm 缺失的 window 能力面；零查询路径改动）

## 1. 背景与本轮范围重裁定

本轮按 master.md「R355 下一步」领取 **d3d-r2「iframe 树源统一」**（part05 iframe 工厂与
`_makeDetachedDocument` 的 bodyHtml 空态收口——src-iframe 树/查询单一来源）。

开工前以 WPT 同构探针（`testharness-dom` 注入 12 个临时 probe 文件，跑完即删）对 iframe
工厂域做blast-radius 普查，**结论推翻了 d3d-r2 的立项前提**：

| 探针面 | 结果 |
|---|---|
| iframe doc `body.innerHTML` seed → compound 查询（`div#probe1`） | **hit**（`_tree` 已含 seed 内容） |
| factory append → 同 turn compound 查询（`section#z2`） | **hit + identity**（产物 === append 返回对象） |
| setupRangeTests（insertBefore 建树）→ `qSA('div#test > p')` | **6 hits**（组合器正确） |
| traverse vs QSA identity（`*` 全量逐位 `===`） | **mismatch=-1**（全等） |
| `doc.querySelector('body')` / 结构元素 | hit（R292 归一可达） |
| getElementsByTagName（元素上下文） | **`td.getElementsByTagName is not a function`**（独立缺口，非树源） |
| iframe `win.setTimeout` / `clearTimeout` / `setInterval` / `clearInterval` | **全部 undefined** |

R169 记录的「runner 探针 iframe srcdoc 全形态 0 命中 + bodyHtml:0」在今天（R296/R306/R307/
R308/R310 等 iframe 域消费面修复落地后）**已不复现**：iframe doc 的查询/树源经
`doc.body.innerHTML = bodyInner`（part05 `_zwMakeIframeDoc`）落入 detached-doc 工厂的
`bodyHtml` 闭包，与 `_makeDetachedDocument` **同源**——两个工厂共享同一查询管线，树源
分裂的主体已在历史切片中收口。

**范围重裁定**：d3d-r2 的原目标（bodyHtml 空态收口）已无独立可达缺口；探针暴露的真实
缺口收窄为 **iframe realm 的 window 能力面**。本轮 land 首个子片：**timer 面转发**。

## 2. 改动（`part05.js` `_zwMakeIframeWin` + `part24.rs` 单测）

iframe contentWindow 补四个定时器方法，转发主 window 的记录式 stub：

- `win.setTimeout(fn, delay)` → `globalThis.setTimeout(fn, delay)`
- `win.clearTimeout(handle)` / `win.setInterval` / `win.clearInterval` 同构

转发目标 = part01 的记录式 stub（host `__zw_setTimeout` 记录 id/at → runner probe 循环
按真实时间 fire；无 host → `_defer` 微任务 fallback）。回调在主 realm 执行（子文档脚本
经形参 `window` 访问本 realm 面，与 R206 脚本通道的形参遮蔽模型一致）。

**动机**：testharness.js 自身的 load 处理（`setTimeout(() => { all_loaded = true; ... }, 0)`）
与 `step_timeout`（`typeof global_scope.setTimeout === "undefined" ? fake_set_timeout :
setTimeout`）都经 iframe window 解析；旧 win 无此名 → "not a function" 被子文档脚本
try/catch 吞 → 定时回调永不执行（静默）。common.js 的 `setTimeoutToWindow` 等真实消费
面同源。

## 3. 验证（d3d-r2 前置子片门）

| 门 | 结果 |
|----|------|
| 全量 dom sweep（polyfill，`--json`，333 文件） | **55478P/19F/18T——真实 Fail 集合 17=17 已知集合恒等零回归**；Pass -2 / Timeout +2 为已知 Timeout 轮转族（ParentNode-querySelector-All-content / query-target-in-load-event 等，R355 同款注记） |
| 文件级门 | QSA 1975P / Element-matches 669P / appendData 384P / MO-attributes 42P。**Element-matches 669 vs R355 记录 675 的 -6 经 clean-HEAD A/B 复核为环境漂移非本片回归**（stash 后 R355 二进制单跑同样 669P；该计数依赖 async iframe content 页加载时序） |
| native 路径 spot check（`ZW_NATIVE_DOM=1` Element-matches） | 669P/0F 与 polyfill 一致 |
| runner 探针（临时 fixture，已删） | `win.setTimeout/clearTimeout/setInterval/clearInterval` 全 function；`win.setTimeout` 回调 100ms 后真实 fire（`timerFired=yes`） |
| engine 单测 | v8 2481（+1 `test_iframe_realm_timer_surface_r356`）/ quickjs 1471 全绿 |
| webview quickjs | 611P（`navigator_controller` flaky 单次复现，复跑绿——R342 已记档预存） |
| integration | 781P（Vue/lit/WC e2e 含其中）全绿 |
| make test | 唯一失败 = `window_surface_present_smoke` XOpenDisplayFailed（X11 环境项；clean HEAD 复跑同样失败 = 预存，R355 同款记录） |
| clippy | v8 全 `-D warnings` 零警告；quickjs 矩阵零警告 |
| fmt | 无 diff |

## 4. d3d-r2 状态更新

- **原立项前提（bodyHtml 空态收口）**：经普查确认主体已由 R296–R310 历史切片收口，
  无独立可达缺口——**d3d-r2 的「树源统一」目标视为已达成**（探针矩阵全绿）。
- **本轮子片（timer 面）**：已 land（本片）。
- **探针暴露的残余缺口**（不属树源，逐项记档）：
  1. 元素上下文 `getElementsByTagName`（iframe 工厂元素缺方法——R181 只接了 querySelector 族）；
  2. `td.querySelector('p#a')`（工厂元素 own QSA 的 compound 形态返 0——own 简单匹配器
     不支持 compound，与 part05 注释的「复杂选择器返空 headless 近似」一致）。
- **d3d-r3 重启条件**：两项前置（d3d-r1 键统一 + d3d-r2 树源）现均满足，下轮可领取。

## 5. 下一步

d3d-r3（element/fragment 本树化重启）——RFC v0.3 §6.2 路线 A 第三片；或先收
iframe 工厂元素的 `getElementsByTagName`/compound QSA 补面（低风险高 ROI 的 R356 续片）。
