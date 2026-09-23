# Web API 第二批 — 运行时控制面板（master.md）

**入口文档**: [../web-api-batch2.md](../web-api-batch2.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-24（M4：DC 逐项判定满足 + 平台差异挂账定稿，goal 收口）

---

## 当前状态

**专项定位**：M12 余面收编——Clipboard API + Fullscreen API 语义落地，WPT 两 corpus
为验收标尺。DnD 排除挂账（宿主拖拽输入管线深依赖）。
**状态：已收口（2026-09-24）**——终态 clipboard-apis 83.6% / fullscreen 88.0%，DC-1~4 全满足。

**与兄弟 goal 的边界**：
- security-hardening — 权限语义供数关系（本 goal 最小权限查询面，其 DC-4 落地时对齐）
- rendering-compat — `:fullscreen` 样式面跨域记账，`crates/style-system` 不碰
- cdp-protocol / devtools — 无协议面耦合
- android-browser / desktop-browser — 全屏窗口语义触 host-runtime 平台面时 git log 核对

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | clipboard-apis / fullscreen 两 corpus fetch 脚本 + 导入 + 基线 | ✅ M1（2026-09-23） |
| P2 | navigator.clipboard 四方法 + ClipboardItem/Clipboard 面 + ClipboardEvent + 内存后端 | 🔄 83.6%（M3-s1 复跑 61/73 零丢失）；余簇 = custom formats ×6（tentative）、svg 净化 ×1、内容规范化 ×1、DOMParser remove ×1、basics 停滞 ×1、infra ×2 |
| P3 | 最小权限查询面（security-hardening DC-4 对齐点） | ✅ 最小面落地（M2-s3）：状态注册表 + query 活状态 + denied 门 + runner set_permission；M3-s1 增 PermissionStatus 真类 + fullscreen allowWithoutGesture；完整语义层仍归 security-hardening DC-4 |
| P4 | Fullscreen 事件/状态面 + viewport 联动 | ✅ 域内收敛（M3-s1 + M3-s2，2026-09-24）：61.7%→**88.0%**（132/150，全绿案 37/55，三轮零丢失）——异步状态机 + 激活门/消费 + PermissionStatus + options 校验 + removal 联动已落；viewport 联动以 corpus 三案绿验证（timing ×2 + screen-size），真窗口 OS 级全屏挂账 M4；余簇全部跨域记账（:fullscreen 伪类/rendering、SVGElement instanceof/js-dom、iframe 管道、shadowRoot 面、window.event 持久性/infra、栈模型 remove-last） |
| P5 | 平台剪贴板后端（host-runtime 能力评估）或差异记账 | ⏳ M4 挂账定稿 |

## 已完成切片

- **M1（2026-09-23）**：`fetch-clipboard-apis-subset.sh` + `fetch-fullscreen-subset.sh`
  （pin 315976933870）+ runner 双子命令（`testharness-clipboard-apis` /
  `testharness-fullscreen`，observers 式内容级 skip）+ Makefile 四目标（test-guard 包裹）
  + 基线 evidence ×2（clipboard 21.1% / fullscreen 63.0%，账本 88 行）。
- **M1 基建修复（WAB2-M1）**：engine `script_gen.rs` strict-eval 全局发布扫描补
  `async function`/`function*`/`async function*` 形态（clipboard-apis helper 全 async
  形态 → 24 案 ReferenceError 的根因；修复前口径 13.0% → 修复后 21.1%）。单测 +
  R147/R198/R201 邻接回归全绿。
- **立项基线事实修正**：引擎已有部分 fullscreen 语义（viewport 桥遗产）——fullscreen
  基线起点 63.0% 显著高于立项预期（立项记「无 requestFullscreen 面」）。
- **M2 切片 1（2026-09-23）**：ClipboardItem/Clipboard 富 MIME 面——part02.js clipboard
  块重写（ClipboardItem 全局类 + types/presentationStyle/getType 代际失效 + Clipboard
  接口 instanceof + write 输入校验簇 + read/write 存取）。clipboard-apis 21.1%→62.9%
  （44/70，全绿案 5→15）。evidence/2026-09-23-m2-s1-clipboard-apis.{md,json}。
- **M2 切片 2（2026-09-23）**：execCommand('copy'/'cut') defaultPrevented 桥（part06 真
  DataTransfer + part02 `__zwClipboardStoreWrite` 钩子）+ runner 静态资源 MIME 映射
  （png/svg/json/css 等，`Response.blob()` type 断言链）+ `CORPUS_CASE_TIMEOUT` 30s 时限
  预算（basics ~19 激活周期 × ~0.8s/周期超 10s 伪超时；30s 下 basics 9→11 真子测绿，
  停滞点 ~10-12 周期非纯时限——runner 探针循环长程行为记档）。clipboard-apis
  62.9%→70.8%（51/72，全绿案 15→19）。evidence/2026-09-23-m2-s2-clipboard-apis.{md,json}。
- **M2 切片 3 / P3 最小权限面（2026-09-24）**：权限 denied 拒绝语义——clipboard IIFE
  权限注册表 + `__zwSetPermission`/`__zwPermissionState` 内部钩子（约定同
  `__zwClipboardStoreWrite`），read/readText 门 clipboard-read、write/writeText 门
  clipboard-write，denied → NotAllowedError（'prompt' 不拦截 = WebKit 风格，保零回归）；
  navigator.permissions.query 活状态消费；runner `set_permission` 白名单 + TESTDRIVER_STUB
  页面侧实现（`unsupported_testdriver_command_is_explicit` 单测改 set_context）；+
  read(options) unsanitized 字典校验（null/非序列 TypeError、多格式/非 text/html
  NotAllowedError）+ write() image/* 魔数校验（malformed → DataError，无解码器按类型魔数
  甄别）+ DataTransfer types 'Files'（file 项非空追加）。clipboard-apis 70.8%→83.3%
  （60/72，全绿案 19→25，通过集零丢失 +9 精确命中计划面）；fullscreen 3 案解锁
  （Pass 92 持平，分母 146→149 = 61.7%，解锁效应非回归）。单测
  `test_clipboard_permission_denied_and_validation_wab2m2s3`。
  evidence/2026-09-24-m2-s3-clipboard-apis.{md,json} +
  evidence/2026-09-24-m2-s3-fullscreen.{md,json}。
- **M3 切片 1（2026-09-24）**：fullscreen 异步状态机——spec「run the fullscreen steps」
  渲染机会执行（setTimeout 1ms 近似，晚于用例 0ms 定时器）替代 R2938 同步模型：
  requestFullscreen WebIDL options 校验（非字典/枚举/screen getter）+ ready check
  （connected + `_nsHandles` 命名空间：HTML/SVG svg/MathML math）+ 激活门（fullscreen 权限
  granted 豁免，瞬态激活**同步消费**）+ 同元素 no-op + step 内重校验；exitFullscreen 异步
  step 化 + 非全屏 TypeError 拒绝 + 双 exit 单事件；事件派发 `_fireFsElementEvent`
  （target=全屏元素、bubbles/composed、`new Event` 真原型修 instanceof 断言根因、detached
  回落 document、reject 先入列微任务派事件）。瞬态激活面：`__zwUserActivate` 钩子 +
  navigator.userActivation（runner click/send_keys/Actions.send 命令签发即授予）；runner
  `bless` 白名单 + stub（授予激活 + 执行回调返其 promise）；PermissionStatus 真类 +
  query 升级（fullscreen allowWithoutGesture false→TypeError）。fullscreen 61.7%→**86.0%**
  （129/150，全绿案 5→34，通过集零丢失 +37）；clipboard 复跑 61/73 零丢失。
  单测 test_fullscreen_api_r2938（重写新语义）+ test_fullscreen_activation_and_steps_wab2m3s1。
  evidence/2026-09-24-m3-s1-fullscreen.{md,json} +
  evidence/2026-09-24-m3-s1-clipboard-apis.json。
- **M3 切片 2 + 域内收口（2026-09-24）**：节点移除 → 全屏退出联动（part06
  `_fsOnNodeRemoved` + part04 remove() 钩子）——移除子树含全屏元素（自身/祖先链身份
  比对，移除前视图）→ fullscreenElement **同步**置 null + 异步 change **target=document**
  （forceDoc 强制 document 派发 + 显式设 ev.target——`_dispatchToListeners` 第四参是
  currentTarget 不写 target；handle 元素 gBCR 探针 stale rect 绕开）。remove-single/
  remove-parent/remove-first 转绿（remove-child 无涉保持）。fullscreen 86.0%→**88.0%**
  （132/150，全绿案 34→37，pass set 零丢失 +3）。**M3 域内收口判定**：viewport 联动以
  corpus 三案绿验证（exit-timing/timing/screen-size），真窗口 OS 级全屏挂账 M4；余簇
  全部甄别跨域归属（:fullscreen 伪类→rendering-compat、SVGElement instanceof/shadowRoot
  getElementById→js-dom、iframe 管道/cross-origin、window.event 持久性→infra、栈模型
  remove-last→挂账）。单测 test_fullscreen_removal_exit_wab2m3s2。
  evidence/2026-09-24-m3-s2-fullscreen.{md,json}。

## 下一步计划

**Goal 已收口（2026-09-24 M4）**——DC-1~DC-4 逐项判定全部满足（见
evidence/2026-09-24-m4-dc-verdict.md），无下一步计划。重入/后续挂账：

- 平台剪贴板桥（arboard 已在 browser 壳层 ↔ navigator.clipboard 内存 store 互通）→
  webview/host-runtime 后续 goal，重入条件：桌面剪贴板互通需求点名
- 真窗口 OS 级全屏桥（engine grant 路径 → winit set_fullscreen + viewport 回灌）→
  host-runtime 流域，重入条件：视频/演示真全屏需求点名
- 可选余簇（不阻 DC）：M2 custom formats ×6（tentative）、remove-last 栈模型

**跨域记账（终态归属，回流不越界）**：
- `:fullscreen` 伪类 / display:contents UA 面 / `::backdrop` → rendering-compat 流域
- SVGElement instanceof / shadowRoot.getElementById → js-dom 流域
- iframe 依赖面（cross-origin / navigate-iframe / allowfullscreen / detached-iframe）→
  重入 = iframe 文档管道
- runner infra：window.event 派发后持久性、`/common/` 绝对路径 fetch、basics 探针
  ~11 周期停滞、copy-event isTrusted、remove-last 栈模型所需事件语义

**待用户决策清单**：
- （暂无——重入条件均为需求点名型，无当前阻塞）

## 验证基线

- 基线 evidence：`evidence/2026-09-23-m1-clipboard-apis-baseline.{md,json}`
  （33 案 / 15-71 = 21.1%）+ `evidence/2026-09-23-m1-fullscreen-baseline.{md,json}`
  （55 案 / 92-146 = 63.0%）；账本 `imported-testharness.txt` +88 行（WAB2-M1-baseline）
- 最新 evidence：`evidence/2026-09-24-m4-dc-verdict.md`（DC 逐项判定 + 平台挂账定稿）+
  `evidence/2026-09-24-m3-s2-fullscreen.{md,json}`（55 案 / 132-150 = 88.0%，全绿案 37）；
  此前 `evidence/2026-09-24-m3-s1-*`（86.0% + clipboard 83.6%）+ `evidence/2026-09-24-m2-s3-*`
- 质量门禁（M4 终验，2026-09-24）：`cargo build --workspace` 通过 + `make test` 68 段
  全绿（test-guard 包裹）+ `cargo clippy --workspace --all-targets -- -D warnings` 全过 +
  `cargo fmt --all -- --check` 干净 + `make reftest` failed 0（零回归）+ `make bench-gate`
  （定向 zero-engine）GATE PASS 26 指标全预算内；engine 面 js_dom_bridge R2964/WAB2-M2/
  WAB2-M2-s2/WAB2-M2-s3/R2938（重写）/WAB2-M3-s1/WAB2-M3-s2/R2948 单测绿

**碰撞管理**：碰 engine shim 面前与 security-hardening / cdp-protocol `git log` 互核。
