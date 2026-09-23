# M3 切片 1 — Fullscreen 异步状态机 + 激活面 + PermissionStatus（fullscreen 61.7%→86.0%）

**日期**: 2026-09-24
**套件**: `make testharness-fullscreen`（WPT 315976933870b34d6ea30e3f6643403edae678ba）
**原始数据**: [2026-09-24-m3-s1-fullscreen.json](2026-09-24-m3-s1-fullscreen.json)
**前值**: [2026-09-24-m2-s3-fullscreen.md](2026-09-24-m2-s3-fullscreen.md)（92/149 = 61.7%）

## 结果

| 指标 | M2-s3 | M3-s1 | Δ |
|---|---|---|---|
| subtests Pass | 92/149 = 61.7% | **129/150 = 86.0%** | +24.3pp（+37 Pass，分母 +1 = bless 解锁案真子测化） |
| 全绿用例 | 5/55 | **34/55** | +29 |
| 通过集丢失 | — | **0**（与 M2-s3 pass set diff 零丢失，+37 纯增益） | |

新增全绿 29 案（节选）：document-exit-fullscreen(-twice)、document-fullscreen-element、
document-onfullscreenchange/-error、element-ready-check ×2、after-error、and-move、
and-remove、consume-user-activation、dialog、namespaces ×6、non-top、not-allowed、
same-element、svg-text、top、twice、two-elements、without-user-activation ×2、
permission.tentative ×3、promises-reject、promises-resolve、options ×3、crashtests ×2。

## 落地面（三处）

1. **fullscreen 异步状态机**（engine part04 + part06）——spec「run the fullscreen steps」
   渲染机会执行语义：
   - `requestFullscreen(options)`：① FullscreenOptions WebIDL 校验（非字典参数 TypeError、
     navigationUI 枚举 {hide,show,auto}、screen 成员 getter 触发）；② 调用时校验
     fullscreenEnabled + ready check（connected + 命名空间——`_nsHandles` 印记：HTML ns /
     SVG 'svg' / MathML 'math' 放行，null/空/未知 ns 拒绝）；③ 激活门（'fullscreen' 权限
     granted 豁免；否则需瞬态激活并**同步消费**）；④ 同元素 no-op（不派事件）；⑤ enter
     step（setTimeout 1ms 渲染机会近似——晚于用例 0ms 定时器，after-error 案移除时序）
     重校验 connected → 设状态 + 派 fullscreenchange + resolve。调用后 fullscreenElement
     同步保持 null（document-fullscreen-element/twice 案时序断言）。
   - `document.exitFullscreen()`：异步 step 化——调用后同步保持，step 内清状态 + 派 change
     + resolve；**双 exit 均 resolve 仅单事件**（twice 案）；非全屏 TypeError 拒绝
     （promises-reject 案，旧 resolve 与上游冲突修正）。
   - 事件派发 `_fireFsElementEvent`：**target = 全屏元素**（enter 新元素 / exit-error 原
     元素，document-exit-fullscreen/not-allowed 案 `event.target === div` 断言）+ bubbles/
     composed + **Event 真原型**（`new Event`——旧 `_makeEvent` 裸对象致 instanceof 断言败，
     onfullscreenchange 案根因）；元素 detached（and-remove/ready-check 案）→ 回落
     document 派发。error 路径 **reject 先入列、事件微任务随后**（timing 案「promise
     executed before fullscreenerror handler」次序断言）。
2. **瞬态激活面**（part06 状态 + part02 navigator.userActivation + runner stub）——
   User Activation API：`__zwUserActivate` 钩子注入（runner testdriver click/send_keys/
   Actions.send/bless **命令签发即授予**近似）；`navigator.userActivation.isActive/
   hasBeenActive`（无 5s 窗口时钟，headless 诚实范围记档）；requestFullscreen 消费
   （consume-user-activation 案 isActive 翻转断言）。
3. **PermissionStatus + runner bless**（part02 + testharness.rs）——`globalThis.
   PermissionStatus` 真类（instanceof 断言 + name/state/onchange），permissions.query 升级
   返回真实例 + fullscreen 描述符 `allowWithoutGesture` 成员读取（false → TypeError，
   permission.tentative 三案全绿）；runner `bless` 白名单 + stub（授予激活 + 执行回调并返
   其 promise——same-element 案 Promise.all 直接 await bless 结果），6 个 bless-Unsupported
   案解锁。

## 已甄别未解（记档，M3-s2 / 跨域）

- **model/removal 簇 ×6**（remove-first/last/±sibling/parent/single + move-fullscreen-element）：
  移除全屏元素 → fullscreenElement **同步**置 null + change(target=document)——需 mutation
  hook 联动，M3-s2 主对象。
- **Timeout ×5**：cross-origin.sub / navigate-iframe.sub（iframe 管道，跨域记账）；
  shadowroot-fullscreen-element（shadowRoot.fullscreenElement 面，M3-s2 候选）；
  move-to-inactive-document（mutation + inactive document，M3-s2 候选）；frameset-crash
  （crashtest 挂起甄别保持）。namespaces 案本轮一度 Timeout（探针调度抖动，复跑 6/6 全绿）。
- **跨域记账**：svg-rect/svg-svg（SVGElement instanceof 面，js-dom）；element-request-
  fullscreen 基本案（`window.event` 派发后持久性——我们按 spec 派发后还原，上游依赖
  Chrome 持久到下一派发，infra 记账）；display-contents（全屏 UA display:block 覆盖 +
  `::backdrop` computed-style → rendering-compat 流域）；reordering（`/common/top-layer.js`
  fetch → runner infra）；screen-size tab-capture（getDisplayMedia 缺）；nested-shadow-dom
  （shadowRoot.getElementById → js-dom）；nested（testdriver selector 面）；allowfullscreen
  iframe（跨域记账）。

## 附带影响（clipboard corpus 回归核对）

- clipboard-apis 复跑 61/73 = 83.6%（M2-s3 60/72 = 83.3%）：通过集零丢失 +1，全绿案
  25 持平。evidence [2026-09-24-m3-s1-clipboard-apis.json](2026-09-24-m3-s1-clipboard-apis.json)。
- 单测：`test_fullscreen_api_r2938`（重写至新语义：激活门拒绝次序、同步保持、事件
  target/原型/枚举）+ `test_fullscreen_activation_and_steps_wab2m3s1`（激活消费/权限豁免/
  ready check/双 exit/options 成员/PermissionStatus）。clipboard ×4 + pointer lock + runner
  testharness 35 全绿。
