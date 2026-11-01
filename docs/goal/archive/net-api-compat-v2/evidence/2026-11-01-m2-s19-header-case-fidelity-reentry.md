# M2-S19 — 重入小片：Headers 实例/XHR 异步源头名大小写保真补全（2026-11-01）

**切片**: 归档后重入——S10 记账 1/2 清偿（[入口文档](../../net-api-compat-v2.md)
「后续可推进小片」清单第二项）。语料无钉（无 WPT 钉），spec 保真切片，
**净账 0 增益 / 0 回归 / 0 MISSING**（全通道 A/B）。

## 缺口（承 S10 记账）

请求头名首见原始大小写线上保真（M2-S10 落 `_hCase`：lowername → 首见原始
name，仅请求 wire 出口消费）当时覆盖数组 / record / setRequestHeader 三形态；
两形态仍丢 case：

1. **Headers 实例作 fetch init 头源**：`_fillHeaders` 对 Headers 实例走
   `Symbol.iterator`（entries sort-and-combine，只产小写名）→ append 首见
   记录的是小写名，实例 `_hCase` 随 fill 丢失。spec 语义 fill 对 Headers
   对象迭代其 header list 本就保 pair 名
   （https://fetch.spec.whatwg.org/#dom-headers）；浏览器同型（API 面归一
   小写、网络面保用户 case）。
2. **XHR 异步路（fetchFn dict init）**：setRequestHeader 首见原始名存于
   `_zwXhrHeaderCase`，异步 `fetchFn(url, {headers: _zwXhrHeaders})` 未透传
   → 线上小写（同步路径 caseMap 参数已保）。

附同根形态：`new Request(url, {headers: hInstance})` 构造时经 fill 同丢
case，分发 `fetch(request)` 时 wire 已小写——记账 1 的 Request ctor 变体。

## 实现面（4 处旁路补齐，fill 路径零改动）

- `part01.js` `_headersToWire`：caseMap 参数缺位且 src 为含 `_hCase` 对象时
  旁路合并（仅限 `h._h` 已有键）——覆盖 `fetch(init.headers)` 与
  `fetch(input.headers)`（Request-like）两侧。
- `part01.js` fetch 主实现：`init.__zwHeaderCase` 内部成员透传 `_headersToWire`
  第二参（与 `__zwInternalHeadersWire` 同型内部通道；XHR dict 源无 `_hCase`）。
- `part03.js` XHR 异步 send：init 增 `__zwHeaderCase: self._zwXhrHeaderCase`。
- `part01.js` Request ctor：fill 后从实例源旁路补 `_zwHeaders._hCase`。

约束遵守：`_fillHeaders`/Headers ctor 内**不加任何探针**——headers-record
计数代理面（`log.length` 恰等断言）禁不住 instanceof/gOPD/属性读任一 trap
（M2-S2 教训在案）。四处旁路全部在 fill 之后或 init 直读位；计数代理语料仅
现于 `new Headers(proxy)`，fetch init/Request init.headers 无 Proxy 计数面。

## 验证

- 引擎钉 3 件（`js_dom_bridge_tests/part93.rs`，__zw_fetch 同步契约捕获
  headersWire）：Headers 实例源保 `THIS-is-A-test` / Request ctor 变体同 /
  XHR 异步 `X-MiXeD-Case` + 同名值合并；API 面 `get('this-is-a-test')` 恒绿
  （归一语义零扰动）。**逐 hunk load-bearing 实证**：stash shim 两文件 3 钉
  全红；单独回退 Request ctor hunk 该钉红。
- 全通道 A/B（post-fix vs pre-fix 同树双轮，`make testharness-net-api`）：
  12,078 行唯一差异 `eventsource/format-field-retry.any.js` 页级
  Pass→Fail——重连时序负载 flake，隔离复跑 3/3 绿（本 goal 既有同族在册
  判例：M5-S2 eventsource 1T stash 对照先在漂移）。**净账 0/0/0**。
- 引擎 lib（test-guard 包裹）：**3645 过 / 1 败**
  （`test_ua_display_li_list_item_canvas_inline_s24_flip`——渲染域在册红，
  stash 对照无本改动同败；S18 档的 11 败名单已被并行流收敛至本 1 项）。
- clippy `-D warnings` + rustfmt 干净；bench-gate 定向（zero-engine）
  **A/B/A 三轮**：带改动 1 FAIL（page/medium/first_paint_wall_ms 366.3 vs
  预算 362.35）== stash 对照 1 FAIL（364.9）同指标同幅——渲染域在册边际
  项（run-rules §10 渲染流域归，不放宽不代修）；首轮 4 FAIL 系通道运行
  窗口机器态噪声（同码跨轮 FAIL 集 0/5/10 摆动判例，M5-S8 在案）。

## 基线一致性核查（本轮主任务）

- **归档提交同机复测**：worktree @ `42c890d81`（二期归档提交）+ 当前工作区
  语料，全通道 12,078 行与当前 tip **逐 subtest 字节一致**（0 增益 /
  0 回归）——归档窗口（159 commits，含渲染流 s18x-s19x / justify / perf
  系列）零引擎漂移，通道现状与文档基线一致。
- 账面差归因：归档终态 11,040/12,077（S18 参考轮）vs 当前 11,035/12,078
  （两树同值）——差异源 **gitignored 语料工作区漂移**（websockets 377 案 vs
  归档 376 等）+ 机器态敏感腿，非代码面；v1 master 既有口径「跨轮次对比以
  同树 A/B 为准」（M5-S6 语料漂移注记）适用。8 项页级 Timeout 两树同值，
  非本窗口回归。
- 遗留待决不变：request-cache 98F 口径、语料仓滚动 tag（均挂用户）；
  abort 目录外 signal 关联体中止（S9 记账 1-2）继续随语料扩批评估。

## 记账

1. XHR 同步路 `_headersToWire(self._zwXhrHeaders, self._zwXhrHeaderCase)`
   与异步 `__zwHeaderCase` 通道并存——两路 caseMap 形态不同源（同步 scheme
   dispatch 内联 wire、异步 fetchFn dict），未强行归一（精准修改边界）。
2. Response ctor `new Headers(init.headers)` 同型 fill 丢 case 未补——响应
   头 wire 出口不消费 `_hCase`（request-only），API 面归一语义正确，非缺口。
3. 语料漂移核查中 `websockets/resources` 列目录 403 限流（匿名配额）——语料
   滚动按 S13 结论收敛 pin 单点，本片零语料改动。
