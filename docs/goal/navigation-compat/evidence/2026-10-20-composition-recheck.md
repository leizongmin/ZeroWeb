# 组成态复核 — 语料清空事故恢复 + scroll-restoration-order 收口（2026-10-20）

## 事故链与自愈修复

开工例行跑测时发现导航语料面大面积脚本错（35+ 处 `is not defined`，make exit 1）。
根因链：wpt-data 整目录重建（`make fetch-wpt-data` 在 reftest-manifest.json 缺失时
`rm -rf` 重 clone）后，goal fetch 脚本续拉撞 GitHub Contents API 未认证 403 限流，
嵌套 resources/ 目录全缺；直连网络同时退化，改走代理 + `gh auth token` 认证列目录续拉。

续拉过程暴露两个自愈盲区，已入 goal 脚本（commit `dc126dc4e`）：

1. lib.sh 扩展名白名单不含 `.mjs` → focus-reset / ordering-and-transition 两个
   resources 目录的 helpers.mjs 拉不到（S4E/S4G 当年为手工 raw 补拉，未入脚本，
   整目录重建即失）。
2. wpt-data/common/ 子集不含 get-host-info.sub.js → S4R/S9 跨源两案
   （intercept-cross-origin / navigate-anchor-cross-origin）`get_host_info` 未定义。

另定向 `FORCE=1` 补拉 replace-before-load/resources 等两个目录（快路径洞：含 .html
的目录跳过 listing，深层 support 文件永不补）。

## 组成态定谳

三份账本状态行集合（`^(Pass|Fail|Timeout|NotRun) `，空白归一 + 排序）语义 diff：

| 账本 | P | F | T | NR | 状态行 |
|------|---|---|---|----|--------|
| [2026-10-11 账本](2026-10-11-composition-recheck-r5050-r5051.txt) | 408 | 35 | 26 | 14 | 483 |
| [run2（语料恢复后、修复前）](2026-10-20-composition-recheck-run2.txt) | 406 | 38 | 26 | 14 | 484 |
| [final（修复后）](2026-10-20-composition-recheck-final.txt) | 409 | 35 | 26 | 14 | 484 |

- run2 对 10 月账本差异恰 3 行：+Fail scroll-restoration-order（新案）、
  get-host-info 两案 Pass→Fail（helper 缺失，补拉后复绿）。
- final 对 10 月账本差异恰 **+1 行新 Pass**（scroll-restoration-order），其余逐行恒等
  ——语料重建达到 10 月组成等价 + 1 新案，零漂移。
- 10 月账本含正文行，此前会话笔记「483 状态行 / 14NR=0」有误，以本表为准
  （483 为含正文文件行数；NR 实为 14，行格式 `NotRun ` 无空格）。

## scroll-restoration-order 定性与修复

**定性**：`scroll-restoration-order.html`（history-traversal 域）在 10 月语料中从未
抓取（全部账本 grep 零出现），本轮续拉首次入面——**新增可执行面的真缺口，非回归**。
案义：hash-only traverse 的保存滚动位恢复须发生在 popstate 派发**之后**、hashchange
派发**之前**（spec [apply the history step](https://html.spec.whatwg.org/multipage/browsing-the-web.html#apply-the-history)
restore scroll position；案内 assert meta 同述）。

**缺口定位**：plain back/forward/go → S3 队列 `_hist_applyTraversal` →
`_navCommitTraversal` 的恢复被 `!hashChange` 门跳过（该路径只服务非 hash traverse）；
`_hist_dispatchPopState` 只做同步锚滚 + `_defer` 派双事件，无保存位恢复。临时探针页
轨迹实证：三段 back 全程滚动位恒 [555,555]，恢复从未发生。

**修复**（part02.js，+17/−3）：

1. `_scrollToAnchorForHash` 返回布尔（是否命中指示元素；三个 return 面全布尔化）。
2. `_hist_dispatchPopState` 的 `_defer` 块内、popstate 派发后 hashchange 派发前：
   `hashChanged && !skipScroll && !_zwTravAnchorHit` → `_histRestoreScroll(cur)`
   （内部自有保存位 has 门 + manual 门复判）。

门设计：`!skipScroll` 排除 intercepted traverse（其恢复走 `_navCommitTraversal` 的
`_zwRestore`/`e.scroll()` 面，防双恢复）；无指示元素门与同步锚滚段同谓词——锚命中
绿面（S4Z manual 门 / S4R 无锚不滚顶）零触碰。

**验证**：

- 隔离：scroll-restoration-order **Pass**（修复中间版 + 最终源码两轮）。
- 全量：`make testharness-navigation` 409P/35F/26T/14NR，final 对 10 月账本零漂移
  （见上表）；三段 back 恢复 [100,200]/[50,100]/[0,0] 全过。
- per-entry 保存位数据面成立性：pushState 时 `_histApplyNav` 经 `_histSaveCurrentScroll`
  存离位滚动（初始 entry 在首个 pushState 时存 [0,0]），案义全序列自洽。
- `make test`：59 套件结果行唯一失败 `paste_and_go_blurs_and_clears_preedit` =
  **环境面非变更面**——本 session shell 缺 `DISPLAY` 致剪贴板前置断言
  （`write_text` 前置，X11/Xvfb 成立的既知约束）失败；`DISPLAY=:20`（本机剪贴板
  可写的 X 显示；`:0` 接受连接但剪贴板操作挂起）solo 复跑恒过（exit 0）。clippy
  臂零 error。与本轮 traverse 变更零交集（前置 panic 先于任何页面逻辑）。
- `make reftest`：712 比较 **0 不一致**（Layout 508/508 + Text 204/204）——traverse
  分支滚动恢复对渲染面零回归，S8/S9 同款滚动面 reftest 门禁维持。

## exit 1 定性（非本轮回归）

make exit 1 由 the-iframe-element sandbox 两案页面脚本硬抛错触发
（`open()` 返 null → `parent.addEventListener` 抛 `Cannot read properties of null`）。
`open()→null` 慢性面自 2026-08-13（`2b855a1d2`）即存在，案源与 pin 逐字节一致——
10 月基线同样抛错，仅当时账本格式不记错误明细而不可见；10 月 md 的残余抽核已将该
双案按 popup/iframe/beforeunload 三重载体挂账 M3。判读看状态行集合，exit code 不作
组成态门禁（encoding-compat 2026-10-19 行同款备注）。

## 结论与遗留

- 语料自愈链完整（整目录重建可全量重建至组成等价）；goal 资产脚本为唯一重建入口。
- M2 收口态 408/476（85.7%）→ **409/476（85.9%）**（沿用 M2 分母口径；新案计入
  分母则 409/477 = 85.7%）。
- 遗留不变：M3 iframe 深结构切片用户门控（维持挂起）；sandbox 双案 exit 1 慢性面
  随 M3 窗口域处置；replace-before-load 38F、bfcache 族、focus-reset 2T、autofocus
  14NR 等残余记账维持。
