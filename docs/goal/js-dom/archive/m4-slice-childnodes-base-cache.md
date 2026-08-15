# M4 切片 R55 — childNodes 基底缓存 + 兄弟对缓存（同 turn 重复读消 host 往返 + identity 稳定）

**日期**: 2026-08-15
**轮次**: R55（上一 session 开工后 rollover，本 session 完成验证 + land）
**目标**: docs/goal/js-dom.md（M4 WPT dom 基线建立与扩展；R52 遗留「per-subtest testFn 成本」的延续）

## 背景

R52 已把 setup 侧 per-op 成本修到恒定（`__zw_child_nodes` 等六函数接 QUERY_DOC_CACHE），但
Range-mutations 族每 subtest 的 testFn 仍有**每读一次 host 往返 + 全子重包装**的成本与
**identity 不稳定**问题：

- `el.childNodes` 每次读：`__zw_child_nodes` 回调 → JSON.parse → **每个子节点重新
  `_wrapNodeEntry`**（新 proxy 对象）。
- WPT dom/common.js 的 `indexOf(node)`：

  ```js
  while (node != node.parentNode.childNodes[i]) i++;
  ```

  每 i 一次 `childNodes` 读 → 每次读重建全部子包装 → `node != childNodes[i]` 的 identity
  比较在某些路径下永远不等（靠 host 侧 wrapper 缓存兜住大部分，但 pending overlay 分支
  仍重建）。Range-mutations testFn 每 subtest 数十次此类读，是 insertBefore/dataChange
  >420s 的 per-op 主源之一（R52 诊断）。

## 关键洞察：缓存安全性来自 dom_html Arc 的回合不可变性

host 侧 `register_dom_callbacks(&mut sandbox, …, dom_html, …)` 时把 `Arc<Mutex<String>>`
快照固化；mutation flush 写 `WebView.cached_html`，**下一回合重注册才换 Arc**。因此：

- 本回合内基底（`__zw_child_nodes(sel)` 的返回）**恒定**——按 sel 缓存安全。
- pending 差异（本回合 append/remove）由 `_zwOverlayPendingChildNodes` **每读现算**叠加在
  基底上，语义不变。
- 跨回合：重注册 = dom_html 换代 → host 侧注入失效脚本全量清缓存（callbacks.rs 注册开头，
  幂等 no-op——shim 未装的首注册静默）。

## 实现

| 文件 | 改动 |
|------|------|
| part05.js | `_zwChildBaseCache`（Map，sel → 基底包装数组；512 sel 软上限全清）；`_childNodeList` 无 handle 分支读/写缓存；`_zwOverlayPendingChildNodes` no-pending 快路径返 `out.slice()`（调用方原地写不得污染缓存本体）；`globalThis._zwChildBaseInvalidateAll` 暴露。`_zwSiblingBaseCache`（sel → {p,n} 兄弟对包装）+ `globalThis._zwSiblingBaseInvalidateAll`（挂 hoisting 可达位置，part04 运行期引用） |
| part04.js | `previousSibling`/`nextSibling` get trap 改读 `_zwSiblingBaseCache`（miss 时 `__zw_sibling_nodes` + 双包装一次入缓存；512 软上限）——旧路径每次读双 host 往返（sibling + parent）+ 重包装 |
| callbacks.rs | `register_dom_callbacks` 开头 execute 失效脚本（注册即 dom_html 换代；`typeof` 守卫幂等） |
| part18.rs | 单测 +2：`r55_childnodes_base_cache_identity_and_freshness`（缓存命中 identity / 缓存内文本可编辑 / pending overlay 可见）、`r55_reregister_invalidates_base_cache`（重注册换代后读到新基底） |

## 附带 identity 收益

缓存数组里的节点对象稳定：`p.childNodes[0] === p.childNodes[0]`（旧行为不等）。
`indexOf` identity 循环、MutationObserver addedNodes 与 childNodes 交叉比较等 WPT 模式
直接受益。

## 验证（同树 stash A/B，release runner）

| 文件 | R54 基线 | R55 | pass/fail |
|------|---------|-----|-----------|
| appendChild | 1.37s | 0.50s | 34/36 逐项一致 |
| appendData | 3.40s | 2.05s | 160/224 一致 |
| insertData | 6.53s | 6.52s | 134/248 一致 |
| removeChild | 0.31s | 0.35s | 2/18 一致 |
| replaceChild | 0.50s | 0.48s | 16/44 一致 |
| splitText | 0.70s | 0.40s | 56/60 一致 |
| deleteData | 38.56s | 32.55s | 184/380 一致 |

四子目录零回归：nodes 3096P/4389F/6TO（与 R54 **per-case diff 逐字节一致**）、
collections 48P/0F、traversal 953P、events 189P。
engine v8 2145 / quickjs 1415 全绿，clippy 双矩阵干净，fmt 无 diff。

## 上一 session 的诊断遗留（probes 已删）

上一 session 为定位 insertBefore >420s 写了 11 个 panic!-style perf probe（沙箱内复刻
doTests 逐组计时）。本 session 判定：这些 probe 不可提交（panic! 诊断用），已删；hang 的
归因已由 R51c/R52/R53 完成（per-subtest testFn 成本 + live collection 失效循环），
R55 缓存是该归因下的第一刀，insertBefore/dataChange/replaceData 三文件的彻底解决在
M1 L2（polyfill 桥改读 live Document 消 re-parse + per-node 桥）。

## 下一步

M1 L2 正题开刀（master.md 下轮候选 (a)）：childNodes/parentNode 读缓存于 JS 侧 registry
（写时维护）消 host 往返的最小切片，kill-switch 内、A/B 门就绪。
