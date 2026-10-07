# slice42 裁决——named-access 内聚性三件套收口

基线：origin/main d1e2a3aae（slice41 合并树 bba78d605 之上的 R4990 产品变更轮；按卡「产品变更则 merge 进分支全量重测」直接基于新 tip 开分支）。
三件套原文：`.acceptance/site-optimizer/baidu-storm-20260929/diag/evidence/slice33/integ/VERDICT.md` :68-73。

## ① morph 成元素的全局 0 命中不回收 → 真实缺陷，最小修复

**判定**：spec 取值算法每读按当下 named objects 求值，空集则属性不存在（https://html.spec.whatwg.org/multipage/window-object.html#named-access-on-the-window-object）。修前 morph 产物（`_zwNAGlobalMorph` 写入的元素全局，wired 腿为 own expando）在成员跌至 0 后 stale 至快照换代（`_zwNAGlobalMorph` 的 `g !== installed && g !== undefined` 臂直接 return），自然红实证（`red-mutation/natural-red-prefix-s42.log`）。

**修复**（`crates/engine/src/js_dom_shim/part05.js`）：
- `_zwNAMorphEls` 账本（`__zwNAMorphElsStore`，跨 shim 重执行存活）：morph 写入两臂（集合→元素、undefined 恢复→元素）登记；0 命中且现值 identity 等于账本条目时回收——脚本自有改写不追删（spec 脚本 own property 遮蔽命名属性）。
- 回收写面 `__zwNADelete`（wired 腿 backing + own expando 双面）：单面 delete 会经 NPO get 复活 backing 残留的原集合。
- `_zwNAInstallCollection` 清账本（集合面接管该名）。

**判定中发现同路径缺陷（一并修复）**：集合面 0 命中回收（`g === installed` 臂）原 `delete globalThis[name]` 对 wired 腿 backing 安装值 no-op（NPO `deleteProperty` 恒 false 经链传播；part05 `__zwNADelete` 接线处申报同款），容器批删双成员形态 stale 空集合残留——同轮换用 `__zwNADelete`（quickjs 腿 own delete 语义等价）。自然红 = `named_access_batch_zero_hit_recycle_backing_s42` 修前 FAILED。

**钉**：`named_access_morph_product_zero_hit_recycle_s42`、`named_access_batch_zero_hit_recycle_backing_s42`；既有钉按新行为翻转：part34 `named_access_attr_shrink_morph_then_captured_live_s32`（`__r_stale`→`__r_recycled`）、part35 `named_access_recycled_collection_regenerates_s33`（FIXME⑥ 复活面移至 live 名验证，I-7 再生面保持）。

## ② `_proxyCache` 死条目 → 纯卫生，边界保持（钉固化 + 申报收窄）

**判定**：误解析面已被既有守卫覆盖——(a) 移除后 `__zw_handle_for_selector` 失映射，新查询回落 sel proxy 面（R100 修后语义，钉断言「新查询 identity = 新元素 ≠ 旧 captured」）；(b) JS 已持有的旧 proxy 读回落 R3029 语义保持（detached 读 nodeType 实证）。死条目保留恰是重挂 identity 稳定的前提：R52 消零清除曾致 remove 窗口期 proxy 重建分裂（R315 翻转修复，part05 `_recordHandleChild`）。内存滞留有界于单文档 context（reset_context 整体释放）；清除机制（如 remFlat 逐出）会重引 R52/R315 类 identity 分裂，成本/风险超收益。

**钉**：`named_access_dead_handle_proxy_entry_boundary_s42`（四断言：R100 identity 反查基线、detached 读回落、新查询不经死 handle、重挂 identity 翻转保持）。

## ③ live 集合末位近似（树序不保）→ 真实缺陷，最小修复

**判定**：HTMLCollection 成员按树序（https://dom.spec.whatwg.org/#concept-collection）。修前 slice32 NA 集合 childList 并入恒末位 push（`_zwHCLiveInvalidate` 反链未记账时点不能就地锚定）、attr 并入恒末位 push（`_zwNAAttrChanged`）——`window[name][i]` 直读 held 数组，序可观测。t8e 的 `_zwHCTreeOrderSync` 只覆盖 scoped 集合（:10461 门），文档级 NA 集合无暴露面申报在 NPO/slice32 之后已失效。

**修复**（`part05.js` + `part01.js` 尾叫传 inDoc）：
- childList 面：`_zwHCTreeOrderSync` 新增 NA 文档级臂——反链记账后的尾叫时点整集合 CDP 排序（`DOCUMENT_POSITION_FOLLOWING=4`，与 `_zwNACollectMatches` 安装序同款）；inDoc 门对齐成员并入 R54 口径（detached 批跳过，防 disconnected 读数扰动）。
- attr 面：`_zwNAAttrChanged` join 分支排序后 replace（成员先于 attr 变异入树，反链已记账，CDP 新鲜）。

**钉**：`named_access_live_collection_tree_order_childlist_s42`、`named_access_live_collection_tree_order_attr_join_s42`。

## 变异 RED

`red-mutation/`：三修复面四变异点（M-A/M-B/M-C1/M-C2，纯 delta + 修复态行号），变异态 21P/6F——6 红全数命中三面覆盖测试；逐字节还原 sha256 自证（`79a654cc…` 前后一致）。

## 门禁（终锚 402dc9e85）

执行中途 origin/main 四次前移且含产品变更，均按卡 merge 进分支全量重测：
- 第一轮 merge 28784ba23（R4991 + navigation-compat M2）→ 中间锚 ca55467d4：make test 20,189P/0F/198I/70 腿、reftest 704/704、WPT 43P/1F/2T。
- 第二轮 merge d3fffc60e（t8f hydration-insertbefore + R4993）→ 中间锚 aae73e4c9：make test 20,194P/0F/198I/70 腿（t8f +5）、reftest 704/704、WPT 43P/1F/2T。
- 第三轮 merge 381fbb761（M2-S3 session history，part02 shim +66 行；R4994 调查轮树恒等）→ 中间锚 595425f09：数字与二轮持平全绿。
- 第四轮 merge 9d689ecdc（R4995 layout column AR + perf docs）→ 终锚 402dc9e85，下表实测。

| 门禁 | 终锚实测 | 基线 | 归因 |
|---|---|---|---|
| fmt | 零 diff | — | — |
| clippy v8（workspace -D warnings） | EXIT=0 | — | `gates/clippy-v8-s42.log` |
| clippy quickjs（-D warnings） | EXIT=0 | — | `gates/clippy-quickjs-s42.log` |
| make test | 20,194P/0F/198I/70 腿，EXIT=0 | 20,182P/0F/198I/70 腿（bba78d605） | +12 = upstream 7（R4990 +1、R4991 +1、t8f +5，均 v8 腿；R4993/M2-S3/R4995 零新增单测）+ slice42 5（part42；engine 不在 QUICKJS_TEST_CRATES，quickjs 测试腿不变） |
| reftest | 704/704，EXIT=0 | 704/704（slice41 同锚实测） | runner `reftest` 子命令 = css21 固定清单（reftest_data::css21_reftest_cases），与 wpt-data 上游导入（reftest-upstream 层）无关，零漂移 |
| WPT named-access targeted | 43P/1F/2T | 43P/1F/2T | Fail/Timeout 身份与基线一致（cross-origin-named-access.sub / named-objects / window-named-properties），零 P 回退 |

make test 抖动处置（slice41 先例：scoped 复跑留独立 log + 全量重跑出干净 log）——终锚上三连全量各命中一个不同时序型测试（SW pending/ready、renderer onload、wpt-runner harness Timeout），各自 scoped 复跑 3/3、5/5、6/6 全绿；根因 = 并发兄弟会话门禁负载尖峰（ZeroWeb-2 checkout integration tests + renderer 进程群）。先例同款：s38/s39/s41 均曾 rerun；`maketest-s42-flake-{run1,run2,run3}.log` 与 `*-scoped-rerun.log` 留档。

R4990/R4991 的 5 条上游 reftest 导入（imported-tests.txt +5）属 `reftest-upstream` 口径，不在本门禁 704 案清单内。

## 申报收窄文本（替代 VERDICT :68-73 对应行）

1. ~~morph 成元素的全局 0 命中不回收——slice32 边界钉保持~~ →【slice42 收口】morph 产物经 `_zwNAMorphEls` 账本 identity 判据 0 命中回收（`__zwNADelete` 双面写；脚本自有改写不追删）；同路径集合面 wired 腿 delete no-op 一并修。残余：morph 产物 >1 命中不升格集合（跟随面不升格口径保持）。
2. ~~`_proxyCache` 死条目不清~~ →【slice42 判定：纯卫生，边界保持】误解析面有 R100/R315 守卫，清除反破 identity 稳定；钉 `named_access_dead_handle_proxy_entry_boundary_s42` 固化现状语义。
3. ~~live 末位并入近似（insertBefore 中插树序不保）~~ →【slice42 收口】childList/attr 两面树序落位（尾叫 CDP 排序 + attr join 排序）；钉两面。

## 残余面（本轮新证，未修）

- slice32 集合面 parsed 子树移除的成员维护缺口：`_zwHCCollectSubtree` 对 sel 父仅回落 pending 桶（R51c），parsed 后代不入 remFlat——容器删除后 slice32 集合 parsed 成员 stale 至换代（自然红实证：原 parsed-box 形态 `__r_gone=false` 且集合长度不降）。slice40 补偿扫仅覆盖动态账本 `_zwNADynEls` 面，本面同族缺口归后续轮次。
- slice27 静态单命中面（安装期元素全局）不 live、stale 至换代——申报保持（:10849 勘误后口径）。
