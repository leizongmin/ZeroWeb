# R358 — children 集合 scoped liveSpec 重引入 + `__zw_reset_pending_state` 快照换代清桶（16→15）

**日期**: 2026-08-29
**切片**: M4 轻量修复（R357 前置项落地 + ParentNode-children 转绿）
**改动面**: `part05.js`（+换代失效钩子）+ `part04.js`（children 两分支 liveSpec）+
`tab_js_worker.rs`/`js_worker.rs`（SetDomSnapshot 挂钩）+ `part24.rs`（+1 单测）

## 1. 背景

R357 给 children 集合重引入 scoped liveSpec 时被 tab R2930 surroundContents 哨兵拦截
（快照换代后新读 `#sc.children` 把 pending 桶 stale 条目并进集合 3≠1）——根因 = **JS 侧
pending 记账没有快照换代失效语义**：`SetDomSnapshot` 替换 host 快照（同 URL 替换尤甚，
`url_changed=false` 时什么都不清），而 `_zwPendingByParent` 桶/live 集合注册表/child
基缓存/id 覆盖表全是旧快照 + 旧 mutation 批的衍生物，对新快照是 stale 源。

## 2. 改动

1. **part05 新钩子 `globalThis.__zw_reset_pending_state`**：清 `_zwLiveCollections`/
   `_zwPendingAdded`/`_zwPendingRemoved`（+惰性 Set 置 null）/`_zwPendingByParent`/
   `_zwPendingAddedById`/`_zwIdOverrides` + 复用既有 `_zwChildBaseInvalidateAll`/
   `_zwSiblingBaseInvalidateAll`。真导航不经此（`reset_context` 全新 shim）。
2. **两个 worker 的 `SetDomSnapshot` 挂钩**（tab_js_worker + renderer js_worker）：每次
   快照替换都调（不限于 url_changed）——新快照 = 新 host 真相，旧批衍生物按定义 stale。
3. **children liveSpec 重引入**（R357 评估回退件的落地）：sel 分支 `{matches: nodeType
   1, scopeSel}` + handle 分支 `{matches, scopeHandle}`——held 集合随 append/remove 反映
   （WPT ParentNode-children live 断言面）；R333 门（mutation 容器 === 作用域容器）继续
   防跨容器并入。

## 3. 验证（landing 门）

| 门 | 结果 |
|----|------|
| 全量 dom sweep（polyfill，333 文件） | **55480P/17F/18T——真实 Fail 集合 16→15（ParentNode-children 退出），零新增零回归**（Pass -2/Timeout +3 为已知 Timeout 轮转族） |
| 目标件 | ParentNode-children 0P/1F→**1P/0F** |
| R2930/R2929 哨兵 | tab `tab_js_worker_range_*` 2/2 绿、renderer js_worker 152P 绿——R357 拦截形态（surround 后快照换代）在本轮 shim 级单测 + worker 级测试双验证 |
| R318 哨兵 | single-activation **132P/0F** 保持 |
| 其余文件级门 | QSA 1975 / matches 669 / appendData 384 / MO-attributes 42 / getElementsByTagName 19 / getElementsByClassName 3 / Element-children 2 全持平 |
| browser 全套件 | 410P + 唯一 XOpenDisplayFailed 环境项（clean HEAD 同败预存）；integration 781P |
| engine 单测 | v8 2484（+1 `test_children_live_collection_and_snapshot_reset_r358`：surround 形态换代清桶 + held 集合 live + 跨容器不误并 + getElementsByClassName live 保持四断言）/ quickjs 1471 全绿 |
| clippy / fmt | engine/browser/renderer 三 crate 双矩阵 `-D warnings` 零警告 / 无 diff |

## 4. 教训

- **被哨兵拦截的修复 = 前置缺失的信号**：R357 回退时记录的「桶代际清理前置」本轮落地后
  同一 liveSpec 代码零改动语义即安全——回退记录的根因分析直接变成下一片的施工图。
- **快照换代语义跨进程边界**：shim 的 JS 侧记账生命周期必须与 host 快照 Arc 替换点成对
  维护（R344 LIVE_QUERY_DOC 刷新、R348 队列重绑、本轮桶清空——同族第三例），worker 层
  SetDomSnapshot 是该族的新锚点。

## 5. 后续

- d3d-r3（element/fragment 本树化重启）——前置均满足；但 R171 两轮 0 subtest 改善 +
  R353 查询单价 0.014ms 的证据下，其收益面仅剩性能假设，领取前建议先做一次性 blast-radius
  探针复核（同 R356 对 d3d-r2 的处理）。
- 已知 Fail 集合余 15：realm/adoption 族 6、MO document/cross-realm 3、sel 锚点/pseudo/
  redirect/replace-with 深结构 4、Range data 族 Timeout 2。
