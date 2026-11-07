# M2 s327 — 残红普查 + testdriver selectorFor nth 兜底臂修复：beforeinput 16F→1F+1T 红位移（wrap 身份墙定性），forms 域零回归

日期：2026-11-07。切片号 s327（承 s326——五面收口后队列枯竭，普查再生队列）。

## 普查结论（残红全景，s326 后）

全量基线（`make reftest-upstream` 15993P/1027F + testharness 四域 FILTER 跑，日志
logs/s327-{ref,forms,tl,links,md}-r1.log）：

- **reftest 通道 html/semantics 95 红页构成**：forms 43（**38 select 族 = A2
  appearance:base-select 挡口** + range-intrinsic-size 挂账 + select 邻接 3）、
  interactive-elements 26（**全 dialog top-layer/backdrop 族**，s320 已勘根移交对端
  R5431 账本㉔）、text-level-semantics 15（**bdi-auto-dir 族 13 + br-bidi/wbr 已移交**
  ——bdi 13 页新移交记录：para_bidi.rs isolate 臂 + dir=auto first-strong，见跨流移交）、
  grouping-content 8（ol 标记 6 = s321 已勘根 + zzz 探针 2 语料卫生）、embedded 3
  （embed-fallback-image 挂账 + iframe-pdf 结构性 + srcdoc-print 已移交）。
- **testharness 通道**：forms 4735P/211F/10T（r1 基线）——211F 全为 select 族
  A1/A2/A3/C 面 + **beforeinput.tentative.html 16F（唯一非 select 新簇）**；
  text-level 34P/0F、links 139P/0F/4T（两域 testharness 面清零）；**microdata：语料
  无该目录**（wpt-data/html/semantics/ 十目录无 microdata，Mission 十子域其一缺位，
  挂账 fetch 侧核对）。
- **定谳**：非 select 非移交的可施工新簇仅 beforeinput 16F → 本切片施工对象。

## 勘根（beforeinput 16F，三探针钉形）

16F 表形 `testdriver target has no stable selector`，非语义面。三探针（gitignored
本地页，日志 logs/s327-{pr,p2,p3}-r1.log）逐层钉：

1. **selectorFor nth 兜底臂全废**：臂用 `parent.querySelectorAll(':scope > tag')`——
   本仓 `:scope` 语义 = 文档根（query.rs:134 `:root` 等价，非 qSA 上下文引用元素），
   非根 parent 恒空 → 臂永 null → 无 id 匿名控件（页用 `container.innerHTML` 造
   7 input×2 + textarea×2 无 id）整链拒绝。
2. **sync-turn 组合器视图滞后**：探针2/3 实证 innerHTML 同 turn 内 qSA 组合器查询
   （`#container input`/`#container > input`）= 0 而 children/outerHTML 可见——
   applied view 未落位；send_keys 出队跨 turn 后解析已通（R145 模型既定）。
3. **event.target 包装器身份断裂（真墙）**：send_keys 通后（r1 红位移实证：focus
   前缀 + keydown→InsertText→input 序全走通，oninput 触发、await 解析），
   `events.has(element)` false——`_proxyCache` 键 = `_elKey` 原始 sel 串（无归一），
   children 访问器物化的 wrapper 与 `_resolveInputTarget`→`_wrapSelector(sel)` 按
   dispatch sel 串造的 wrapper 键形不同 → 同节点双代理 → Map 按对象身份键控 miss。
   探针1 实证 `event.target !== container.children[0]`。

## 施工（runner 内嵌 JS selectorFor，1 处）

`tests/wpt-runner/src/testharness.rs` selectorFor nth 兜底臂：兄弟枚举从
`:scope > tag` qSA 改直查 `parent.children` 按 tag 计数（等价 nth-of-type 语义，
不依赖 `:scope` 上下文），verify `qSA(sel).length === 1` 不变。零引擎改动。

## 验证

- solo：`FILTER=beforeinput` r1 = 1F + 文件级 Timeout（16 pending 不再超 selector
  拒绝；首子测红移到 events 断言 = 身份墙实证）。
- 回归：`FILTER=forms` 全域 r1 4735P/211F/10T → r2 4738P/203F/11T，**非 beforeinput
  非探针键逐行 diff 零位移**（d2 定案；净移 +3P/−8F/+1T 全由探针页 7F+3P 与
  beforeinput −15F+1T 构成，账目精确对账；form-nameditem/checkable 族两轮同态——
  过滤 diff 显示的新红为 diff 锚点漂移伪影，grep 双跑核对排除）。
- `make testharness-uievents` 全域 A/B 对照（selectorFor 臂原生消费方）：no-WIP
  vs with-WIP 两跑 2008P/271F+T 同态，(key,state) 去重后 1998 键 diff 零位移
  （日志 logs/s327-uie-{nwip,wip}-r1.log，s327 收口轮补录）。

## 残墙与 s328 立项（挂账）

beforeinput 余 1F+1T 的墙 = **shim 包装器身份统一**：dispatch 目标 wrapper 与页面
持有 wrapper 须同 `_proxyCache` 键。候选方向（s328 勘根后定）：①`_elKey` sel 串
归一化（深、全局键面风险）；②dispatch 路径经 children 访问器同源物化（局部）；
③resolveInputTarget 反查页面可见证形。修复红利待 s328 身份墙破后评估（selector
解析已通，语义链只剩 identity 一环）。

## 下一步

1. **s328 = shim 包装器身份统一勘根/施工**（beforeinput 1F+1T 收口 + 红利外溢）。
2. 挂账：microdata 语料缺位（fetch 侧核对）；bdi-auto-dir 13 页移交对端 para_bidi。
3. （承前）A1/A2/A3/C select 族、dialog top-layer、range、print pagination 等
   跨流移交跟踪不变。
