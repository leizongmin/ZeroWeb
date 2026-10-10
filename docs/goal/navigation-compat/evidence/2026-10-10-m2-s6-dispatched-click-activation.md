# M2-S6 — 派发 click 激活行为 + 锚默认动作 helper 化（2026-10-10）

**通道**: `make testharness-navigation`（test-guard 包裹；FILTER 单案迭代 + 全量
per-subtest 精确 diff vs S5 终态）
**前序**: [evidence/2026-10-10-m2-s5-pagereveal-reveal-face.md](2026-10-10-m2-s5-pagereveal-reveal-face.md)

## 切片内容

| 改动 | 文件 | spec/WPT 锚点 |
|---|---|---|
| ① proxy `dispatchEvent` 派发路径接 click 激活行为——此前该路径**零默认动作**（仅 click() API 与 part03 plain-node click() 有）：notPrevented 且 type='click' 且 tag A/AREA → 锚默认动作。② click() API 分支的锚默认动作块（href own-expando-first 读 + download own 门 presence + hash/非 hash 双路）原样抽出为 `_zwAnchorActivate(sel, handle)`（part03 顶层，`_realTag`/`_makeProxy` 间），两路径共享 | `part04.js`（两处接线）+ `part03.js`（helper 定义） | spec activation behavior（click 事件默认动作经 dispatch 运行——synthetic 派发 click 同样跑激活，real browser 对 `dispatchEvent(new MouseEvent('click'))` 触发锚导航）；WPT navigate-svg-anchor-fragment「\<svg:a\> click fires navigate event」 |

## 定位要点

- svg:a 的 `dispatchEvent(new MouseEvent('click'))` 此前派发后无激活 → navigate 恒不
  fire → Timeout。S4T 曾挂账「svg:a dispatched click 激活缺失」，本轮定位到**通用**
  缺口：proxy dispatchEvent 路径对任何元素都不跑默认动作，非 svg 专属。
- svg:a 与 html:a 同分支命中：tagName 'a' 经 `_zwAsciiUpper` → 'A'，tag 检查零改动。
- transient activation 不签发（untrusted 派发）——case `userInitiated === false`
  断言面保持（激活 helper 不触 `_zwTransientActive`）。
- `preventDefault`（notPrevented false）跳过激活——case 末尾 `location.hash === ''`
  断言面（navigate 监听内 preventDefault 取消）依赖既有 hash-setter 取消语义。

## 跨域爆炸半径测量（共享面双臂）

- **dispatchEvent 接线**（全 proxy 触达）：navigation 全量 diff 恰一行（目标案）；
  dom/events + html + uievents 语料 grep 派发 click 用例——全部 div/input/label 载体
  （Event-dispatch-click 系列 checkbox/radio、Event-stopPropagation div、textInput div），
  **零 A/AREA 载体**——惰性新增对其余案零观察面。
- **click() helper 化**（逻辑原样搬移）：navigation 锚族（navigate-anchor-download
  ''/filename ×4、anchor-download-intercept ×4、same-url、fragment、userInitiated、
  same-origin-cross-document）全量轮复绿。

## 挂账核验（本轮顺带）

- **bfcache 族 6 案**：全部引用 `back-forward-cache/resources/helper.sub.js` +
  `common/dispatcher/dispatcher.js`——两资产均不在本地语料（runBfcacheTest 未定义
  即此根因），且 helper 语义需 popup + 真跨文档导航——单文档 runner 不可达，维持
  bfcache/M3 挂账（资产 + 形态双缺口）。
- **focus-reset basic/multiple-intercept 2T**：helpers.mjs 的 `waitForFocus` 需要
  `test_driver.send_keys(TAB)` 键序焦点遍历——DOM 焦点域挂账属实。

## 数字（按 evidence txt 原始 Pass 行）

| corpus 域 | S5 后 | 本轮 | Δ |
|---|---|---|---|
| 全量 | 399 P / 476 = 83.8% | **400 P / 476 = 84.0%** | +1 |
| navigation-api | 237 / 255 | 237 / 255（92.9%） | 0（本案 html/browsers 域） |

**零回归**：全量 diff 恰一行（navigate-svg-anchor-fragment Timeout→Pass）。

## 质量门禁

- `make test`：全绿（数字见本轮终态行；part03/part04.js 变更轮）。
- fmt/clippy 不适用（纯 .js shim 变更，无 .rs 改动）。
