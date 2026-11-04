# M2 s276 — avail 元组键 CORS 态校验（型 E different-crossorigin 收口，2 shim 文件 4 刀 + 单测 3 文件）

日期：2026-11-04。切片号 s276（承 s275 下一步候选①：型 E different-cross
origin——list of available images 元组键比对）。shim 2 文件 4 刀 + 单测
3 文件期望更新（engine 单车道，零 runner 改动）。

## 勘根

1. **红面定谳**：`Unhandled rejection "The img should not be loaded from
   the list of available images because of different 'crossorigin'"`
   ——页内第一段 eager 图（无 crossorigin）装载 image_path 进 avail；
   第二段 lazy 图（loading=lazy + crossOrigin=anonymous）同 URL below
   viewport，断言 Promise.race（timeout 2000ms 胜 = Pass；onload 先至
   = reject = Fail）。我侧 avail 读点（part03 runupdate 同步 settle 臂）
   只按 URL 键比对 → anonymous lazy 图误命中 unset 条目 → 同步装载 →
   onload 先至。
2. **spec 元组语义**：#the-list-of-available-images 键 = (URL, CORS
   settings attribute state) 元组；#cors-settings-attribute 缺失/空 =
   unset、use-credentials 字面、其余含非法值 = anonymous（invalid value
   default）。available 检查先于 lazy 决定（step 6），命中要求元组匹配。
3. **语义调和（周边 Pass 面核对）**：
   - `image-loading-lazy-crossorigin-change`（deferred img 用最新
     crossorigin）：跨源 URL 本无 avail 条目，刀后 miss → 真 fetch
     anonymous CORS 失败 → catch 臂改 src 同源 → fetch onload——链路
     等价 Pass 维持（solo 复核）。
   - `image-loading-lazy-available`/`use-list`（同态命中面）：unset 条
     目 + unset 读图同键命中维持。
   - `available-images-onload`（i2 crossOrigin=true 非法值→anonymous
     态）：刀后 miss → 真 fetch 同源 anonymous → onload + drawImage
     origin-clean（fetch settle 条目带 finalUrl 供 canvas 源判）——
     断言可达 Pass 维持。
   - `list-of-available-images-matching.https.html` 不在 runner corpus
     （SW 面，全树零行）；`does-not-coalesce` 既有全树红非本刀面。

## 施工（shim 2 文件 4 刀 + 单测 3 文件）

1. **part03 新增 `__zwAvailXoState5277` helper**：CORS 态归一（幂等——
   'unset'/''/null/undefined → unset、use-credentials 字面、其余 →
   anonymous）。**首轮缺 'unset' 字面分支**：读点对已归一条目 .xo 二次
   归一时 'unset' 落 default 'anonymous' → available/use-list/
   different-crossorigin 三页全 miss（探针 dump 实证
   `helper("unset")=anonymous`）——补字面直通修复。
2. **part03 runupdate avail 读点元组校验**：`_imXoState` 于入口读
   crossorigin attr 经 helper 归一；命中条件追加
   `helper(条目.xo) === _imXoState`。条目缺 .xo（单测 fixture/旧条目）
   经 helper null→unset 兜底，维持既有命中面。
3. **part03 fetch settle 写点 + part04 静态 settle 两写点补 `.xo`**
   （fetch 臂用 run 启动时 `_imXoAttr` 快照 = 请求的 CORS 态；part04
   settle 循环共享读 `_zwImXo`，SVG 臂/host 快照臂两写点携带）。
4. **单测 3 文件期望更新**：
   - `part88.rs` fixture 三 avail 条目补 `xo: 'unset'`；(4a) crossorigin
     missing→anonymous 状态变迁断言重写——元组不匹配 miss 后无 fetch
     宿主真请求失败派 error（`3/1`），(4b)/(5)/(6)/(6b) 计数链同步重写
     （referrerpolicy 两次变迁各触发重验各派 error → `3/3`、srcset 移除
     `0/3`/`3/3`）——**触发面观察从 load 计数改 error 计数**（变迁触发
     语义不变，触发后 avail 结果按 spec 元组门）；
   - `part88.rs` (a3)/`part89.rs` (a2)/(h1) 三处 avail 条目 JSON 期望补
     `"xo":"unset"`。

## 验证

- **solo**：different-crossorigin Pass（onload 不先至，timeout 胜；
  日志 s276-solo-dcx.log）；available/use-list/available-images-onload/
  crossorigin-change 全 Pass 零位移；coalesce 维持既有红。
- **族**（FILTER=image-loading-lazy，39 结果行）：REGRESSED=∅，FIXED =
  different-crossorigin 1 键；余 4 红零位移（multicol / below-viewport-
  dynamic / in-viewport-dynamic / multiple-times 挂账全集）（日志
  s276-family-r2.log）。
- **全树 A/B**（stash 同二进制法）：A/B 8345 全键 REGRESSED=∅，FIXED =
  different-crossorigin 主键 + checkable-active-onblur-with-click 2 键
  （**flaky 噪音归因**：forms :active 交互页与本刀无路径关联；B 态
  solo 连续两轮稳定 Pass、A2 态偶发红——8 job 时序 flakiness，非本刀
  FIXED，不计入）（日志 s276-fulltree-{a2,b2}.log）。
- **make test**（后台 setsid，r3）：70 test-result，engine **3724
  passed + 5 failed** 既有同集（R-t2qq adopted-stylesheets 5 红）；
  本刀新增 5 个 img 单测红（r1 strict 读点 fixture 缺 .xo miss）经
  fixture 补 .xo + 宽容读 + 期望更新后全绿（`img_lazy_viewport_gate_
  r5277`/`img_runtime_deferred_settle_r5275`/`img_sweep_selection_
  density_r5277`/`svg_img_settle_density_r5277`/`img_relevant_mutations_
  r5277` 五测定向复跑 ok）；clippy 并行臂（-D warnings）零 error；
  `cargo fmt --all --check` 干净（日志 s276-maketest-r3.log）。
- 探针页 zz-s276-probe（验后删）：helper 幂等缺口定位面。

## 挂账更新

- 型 E **different-crossorigin 收口销账**（1 文件 1 子测 Fail→Pass）。
- 新注记：decode() 兜底读点（part03 `decode` 面按裸 URL 查 avail）仍
  无元组校验——现 corpus decode 页无 crossorigin 交叉面零位移，元组
  细化面挂账（零 corpus face，低优先）。
- 承前不动：multicol 窗序反向面（解除面 = runner trickle/async 慢流
  时序）+ multicol 上游主干状态待补查（wpt.fyi 三轮网络超时）；
  below/in-viewport-dynamic Chrome-only 口径；multiple-times 主干双红
  销案维持；runner 全量钉内存大户二分（s270 挂账①）；root margin 净
  速分档（s271 挂账①）；dynamic import() referrerpolicy 穿参。

## 下一步候选

1. below/in-viewport-dynamic Chrome-only 口径勘根（s272 挂账延续——
   inflight 阻塞链调度 vs spec delay load 字面冲突面）。
2. multicol 上游主干状态补查（wpt.fyi API 恢复时）。
3. 型 D 余红（multiple-times 主干双红）复核销案口径。
