# R333 — children live 集合归属收窄（tab/renderer R2929/R2930 复活）（2026-08-28）

## 问题

`make test` 暴露 tab_js_worker 与 renderer js_worker 各 2 个失败（R2929/R2930）：

- `tab_js_worker_range_mutation_ops_r2929`：`cloneContents 不改源 → #cc 仍 3 子` 得 4——
  `#ic.insertNode(<b>)` 的 b 出现在 `#cc.children` 集合。
- `tab_js_worker_range_surround_contents_r2930` + renderer 镜像：`surroundContents 后 #sc
  仅 1 子（wrap）` 得 3——#w 内的克隆 span 出现在 `#sc.children`。

回归潜伏 15 轮未被发现：R318 后各轮 A/B 清单未含 browser/renderer crate。

## Bisect 归因

`git bisect`（good `04bde0bbd` R150 → bad `5a4470118` R325，~8 步；中间 R320 测试提交被
误标 bad 后复跑澄清）定位首个 bad commit = **R318 `bcb6196b8`**（children collection
reads fused view with live maintenance）。复核：R317 clean、R318 broken（3 轮稳定）。

## 根因与修复（part04.js + part05.js）

R318 的 children 集合 liveSpec `matches` 只判 `nodeType === 1`，无容器归属判定：

1. **其他容器插入误并**（R2929 形态）：live invalidate 门 `_r54InDoc || _r120Scoped` 的
   in-doc 半边放行文档任意处的 in-doc 插入——`#ic.insertNode(b)` 的 b 并进了
   `#cc.children`。
2. **子孙节点误并**（R2930 形态）：祖先链归属使 #w 内的克隆 span（`#w.parentNode = #sc`）
   通过 matches。
3. **构建期并入**同样只按全局 pending 表 + in-doc 门扫描。

修复三件：

- part04 children 集合**不再携带 liveSpec**——同 turn 可见性由融合视图重建保证
  （R318 原机制），R2929/R2930 的挂载面由 apply+resnapshot 后的 re-read 覆盖。
- part05 `_zwMakeCollection` 构建期并入：scoped 集合改从**作用域桶**取候选
  （`_zwPendBucket(scopeSel/scopeHandle)`），文档级集合保持全局表 + R54 in-doc 门。
- part05 live invalidate 并入门收窄：`_r333Gate = scoped ? _r120Scoped : _r54InDoc`——
  scoped 集合只在 mutation 容器即作用域容器时并入。

## 验证

- tab_js_worker R2929+R2930 / renderer js_worker R2929+R2930 全绿（双 crate 复核）
- Element-children 2P、ParentNode-children 1P（R318 的驱动面零回归）
- single-activation 132P（R318 曾破坏面，commit message 记档的回退域）
- getElementsByTagName 68P / HTMLCollection 40P（R120 门的既有消费面）
- MutationObserver 全族 119P/4F/4T 持平；Node-normalize 4P
- engine v8 2472（+1 回归测试 `test_children_collection_scope_attribution_r333`）/
  quickjs 1467 绿；fmt/clippy 双矩阵干净
- 全量 dom sweep（R332-only 构建）：54153P/51F/24T vs R330 基线 54151P/52F/23T——
  fail 集既存维持（Element-getElementsByClassName 1F = R319 备档），Timeout +1 为
  childList 文件级既存 11 pending 族的并发慢表现

## 教训

1. **A/B 清单必须固定含 browser/renderer crate**——改共享 shim（part04/05）时只看
   engine + WPT 会让跨 crate 回归潜伏十几轮。R333 后 make test 全量回归常态化。
2. bisect 起点选「最近已知 green」并在可疑点复跑多次（R320 纯测试提交被误判 bad，
   3 轮复跑澄清后继续）。
3. live collection 的归属判定要与集合语义一致：children = **直接子**（parentNode ===
   容器），getElementsByTagName = 子树 + 标签名——单一 nodeType 判定两者都不对。
