# M2-S172 getElementById 幽灵命中根修——pending 索引出树候选剔除 + overrides 臂 R125 门链（s171 挂账收口）

**日期**: 2026-10-31
**刀位**: s171 新挂账「name-attribute 跨群互斥误关归因+根修」。s171 已钉死分叉态与勘根链（webview 隔离不复现、c1f8e9b63 排除、五连 F 稳定），本轮探针逐层剥开至根因并修。

## 勘根定谳（探针五层剥开，runner 内插桩）

runner 复制语料探针（hook globalThis 互斥入口 + PROBE throw dump）逐层定谳：

1. **互斥扫描无罪**：target 案 innerHTML 注入仅一次 `__zwDetailsScan4100(obj|true)` 调用、零 Exclusivity 调用；全树 childNodes DFS 无 ghost 节点——**e2 的 open 属性在树上**（DFS 读 open=1），「跨群误关」语义面反转。
2. **读侧分裂**：同一时点 `getElementById('e2')` proxy 读 `open=0/hasAttribute=0/getAttribute=null`，而树实节点 `hasAttribute('open')=1`——`same=1`（同对象）排除双 proxy，gEBI 返回的 proxy 与树节点 `sameObj=0`。
3. **身份铁证**：gEBI 结果 `__zwHandle='__n0'`（极早句柄）= **113 行前案 createElement 版 e2**（`e2.id='e2'; e2.name='a'; e2.open=true; insertBefore`），innerHTML= 替换销毁后其 proxy 残留为幽灵；三元素本尊（innerHTML 产物）`sel=undefined hnd=undefined`（plain 包装）。
4. **路径钉死**（gEBI 各 return 点旗标插桩）：`pending-poolN id=e2 n=2` ×3——pending-ID 索引**多候选池化臂**（R4006），pePool=[旧e2, 新e2]，`_r4006TreeCmp` 对出树候选不可比返 0 → 插入序保旧 → 幽灵胜出。
5. **根因双层**：①innerHTML= 替换销毁旧子树**不走 removeChild mutation 流**（视图直换，无 pending-removed 登记）；②池化过滤的 in-doc 门 `_zwMutationInDoc(null, hnd)` 走宿主快照（同 turn stale 仍列旧节点），且 parentNode getter 的 handle 反链悬指旧挂载点（父链验证也不可靠）——出树候选三面全漏。伴生缺陷：`_zwIdOverridesEntries` 臂裸返 `_proxyCache` 条目**零门**（同幽灵面）。

## 施工（shim part06.js 单文件）

- 新增 `_r4006LiveRooted(nd)`：**正向位次验证**——parentNode 上行每跳先验「子在实际父 childNodes 里」（反链悬指时位次必 -1），达 document 即 rooted；链断且无 `_zwSelPendingParent` 槽（wire 未 drain 解析产物的挂载凭证）→ 出树。
- pending 索引三臂补先验：fast path handle/sel 分支、pool 过滤 handle/sel 分支、R334 resel 臂（`_r4006LiveRooted(resel7)` 不过则置 null 落后续链）。
- `_zwIdOverridesEntries` 臂补 R125 门链逐字同口径（`_r125Overridden` 反向 + pending-removed + 祖先移除 + `_zwMutationInDoc` in-doc）——出树条目跳过继续找。
- webview `coverage.rs`：探针形态矩阵转常驻钉案 `test_get_element_by_id_stale_replacement_s171`（跨 name 群独立性 + 同 id 替换语义 + gEBI 真节点返回）。
- spec 口径：getElementById 只搜文档树（shadow/未挂载孤儿不可见），出树剔除与 spec 同向收紧不放松。

## 验证

- **靶案**：name-attribute.html 整案 **18 案全绿**（s167 起五连 F 的 `handling of name attribute changes` 翻绿；修复前 solo 定向 17P/1F）。
- details 目录 **42P/1F**（唯一 F = closed-details-layout-apis 布局域既有）。
- interactive-elements 全域 **439P/44F/0T**（修复前 438P/45F/0T）——F 集合逐键 diff：new-F=0、fixed-F=1（靶案），**零其他位移**。
- webview lib **824P/1F**（+1 新钉案绿；唯一 F = image_decoder 既有环境案；service_worker 一案全量跑间歇红单跑绿，flaky 与本刀无交集——worker 生命周期 vs DOM shim）。
- engine lib **3612P/11F**——stash 二分（part06.js+coverage.rs stash → HEAD 定向 js_dom_bridge::tests）**同名 11 F**，SAME 定谳零回归。
- 全域 **7250P/753F/84T**（8087 案）join s167 full1（(key,state) 变迁矩阵）：Fail→Pass 36（靶案 + closedby simple ×3 + selection-pointer forms 域 ~22——gEBI 幽灵面修复的意外收益域/或 solo 负载差，域未深挖）、Pass→Fail **2（stash 对照 HEAD 同态 Fail，零归因本刀**——script-element-css-src/json-src 竞态域漂移）、新报出 126（s167 截断域 select-event 等）、Timeout 键互换 3（已知形态）。
- fmt / guarded-clippy workspace 全绿；shim 13-part 装配 `node --check` SYNTAX_OK。

## FIXME 销账 / 挂账

- s171 挂账「name-attribute 跨群互斥误关」**resolved**——语义面反转（非互斥 bug，是 gEBI 幽灵 proxy），根修落 `_r4006LiveRooted` 三臂 + overrides 门链。
- 新观察（未挂账为 FIXME）：selection-pointer forms 域 Fail→Pass 22 案的确切归因（修复收益 vs 负载差）未逐案勘定，域后续若回红再查。
