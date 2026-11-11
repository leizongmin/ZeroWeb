# M2 切片 402 — shadow 森林 handle 身份跨探测环刷新重建（driving 1F→P）

日期：2026-11-11（承 s401 下一步：本流域散单 link-rel shadow DOM CSSOM，先核 s168b referent）
commit：见 git log s402 提交

## referent 核验与勘根

s168b（2026-10-31）把 `link-rel-attribute.html` shadow 案修到 3P/0F 后，M3 Batch 2
（38b972751 getHTML/outerHTML 声明式模板回环）改变了序列化面——本轮核验 referent：
solo 复跑 driving 案 shadow 子测回红（**M3 序列化回环回归**，非 s168b 修复面失效）。

探针 `zz-probe-shadow-gcs`（v4，复刻真实相位：light rel 变更四步 → 静态宿主
attachShadow + importNode 模板 → 读 gCS → await 30ms 探测环刷新 → 再读）A/B 定谳：

- 修复前 `t30: sd=""` + `byHandle30=""`——刷新后 shadow 树内元素 gCS 返空串；
  `dumpHasSRM=true`——dump 已携声明式标记。
- 根因链：shim `_zwSrQualifies180` list 臂与引擎 serializer ShadowRoot 臂对命令式
  根恒发 `<template shadowrootmode>` → 探测环 dump 携标记 → refresh re-parse 经
  parser fixup 物化 shadow 树——**树不丢，丢的是 handle 身份**：序列化产物不带
  handle 标记，`__zw_get_computed_style_by_handle` 的 `nodes[handle]` 只来自重放
  窗口 `[applied..count)`（s161 契约），队列 drain 尽（applied == count）后窗口空
  → shadow 树内 handle 全落空 → gCS ''。刷新后新 attach 的 shadow 一个探测周期后
  同病（`fresh d2=''` 实证）。
- s168 全量重放臂（`prefix_has_shadow && !base_has_marker → start=0`）服务注册期
  原文形态（基座缺全部落地效果）；互补形态 = 基座含标记但 handle 身份不在窗口
  （drain 尽的 dump/engine 形态基座）——两臂现分流互斥。

## 施工（一新模块 + 一分支重构）

- `crates/engine/src/js_dom_bridge/shadow_forest.rs`（新，419 行含 9 单测）：
  - [`shadow_forest_mutations`]：共享队列前缀 `[0..applied)` 的森林切片——三遍
    O(n)：①种子（AttachShadow/AttachShadowOnHandle root_handle）+ 最后归属表
    （handle 锚 placement 覆盖写 child→parent，light 锚/RemoveHandle 记离林——
    最终归属口径：dump 已是终态真相，forest→light 挪回不重放防双份）；②成员闭包
    （记忆化下行 + 环防御）；③顺序过滤发射。light 宿主命令式 shadow 整树 prune
    （AttachShadowOnHandle 宿主终态离林 → 宿主 create 不重放 → attach 发射必撞
    unknown host handle Err 中止切片尾部——根出种子迭代至不动点，子树 mutation 全
    弃，重放零 Err，后继森林链不受截断波及；该子树视图缺与今日行为同形 lenient）。
  - [`strip_declarative_shadow_templates`]：剥除全部 `<template …shadowrootmode…>`
    span（深度配平嵌套 template，普通 template 保留；无标记串零开销透传）——剥后
    parse 不物化 shadow，森林交给切片重放重建（handle 身份随 Create*/attach 链回归）。
- `crates/engine/src/js_dom_bridge/computed_style_cache.rs` `computed_style_by_handle`：
  `prefix_has_shadow && base_has_marker` 分支 = 剥模板基座 parse + 森林切片重放
  （重注册 handle，`nodes` 持久表跨两段调用）+ 窗口 `[applied..count)` 照旧续放；
  s168 臂（else 分支）语义不变。模块头注释钉根因链与近似面。

## 过程陷阱

- **join 键字段反转重演（s401 在册坑）**：s401 `cur2.pairs.u` 是 `state\x1fkey` 形，
  本轮 pairs 脚本产 `key\x1fstate`——直接 hash-join 全量假 NEW（2.4MB 输出）。
  先对齐方向再 join；本轮 pairs.sh/gone.sh 已按 `key\x1fstate` 固化。
- **matches! 闭包内 ref 绑定**：`.all(|mm| matches!(mm, …{ ref handle, .. }))` 在
  2024 edition 下 mm 已是引用——E0027（显式 ref 与隐式借位冲突）；顶层
  `matches!(slice[0], …)` 是位表达式不受影响。闭包内去掉 `ref`。
- **clippy doc_lazy_continuation**：doc 注释列表项续行缩进浅一格即 warn（1.99 新
  lint 面），对齐续行。

## 验证（串行管线，/tmp/s402/ 全量日志）

- 单元：shadow_forest 9 单测全过（切片三形态/挪回/挪入/applied 截断/无 shadow/
  light 宿主 prune/剥模板三面）；engine lib **3874P/7F**（7F 在册逐名同形：img
  settle ×2/fetch filtered/SW iframe/response body R2967/websocket ×2 时序族）；
  webview lib **849P/2F**（image_decoder proxy_mode + 1 在册，s401 口径同形）。
- 探针 A/B（二进制 stash 双跑）：pre `t30/byHandle30=""` → post 恒
  `rgb(0, 128, 0)`；`fresh d2=""` 维持（light 宿主命令式 shadow 视图缺，钉账
  lenient 不劣化；切片级行为由 `s402_forest_slice_light_host_imperative_pruned`
  钉死）。勘根毕删（probe hygiene，探针恒红不入 sweep 账）。
- driving solo：`link-rel-attribute` **2P/1F → 3P/0F**（solo + sweep1 + sweep2 三跑
  全绿）。
- the-link-element 域 **114P + 探针 0**（s168b 时代在册 1F
  stylesheet-media-change-no-fetch 现绿——区间内 intervening 修复，非本刀面）；
  document-metadata 域 212P + 在册红（render-blocking ×2/non-parser-created-doc/
  style-error-01 与 tentative ×5 T 族——全树 join stable 集成员，与基线同态）。
- 全树 sweep 双跑 join（基线 = stash 二进制当轮全树跑，8475 pairs =
  8227P/218F/32T raw）：
  - **sweep2 join：stable=8474 + STATE-DIFF ×1 = driving F→P 恰中 + new 0 + gone 0**
    （最干净归因形态）。
  - sweep1 join 差异键 = fdtcb 文件级 T 截断 4 子测（s387/s401 在册截断 flake，
    sweep2 复 80P 定谳）+ home-end detailed.optional T→F（在册 flake 家族振荡，
    真态 F 单列不变）。
  - 基线对 s401 cur（8474 pairs）漂移 = probe 新增 1F（本轮探针，已删）+ home-end
    F→T 振荡——corpus 真红零位移。
- fmt/clippy：`cargo fmt -p zero-engine` 干净；`cargo clippy -p zero-engine
  --all-features` clean（修 doc 缩进后零 warning）。
- 产品 smoke：渲染面零变化（gCS by-handle 视图路径为 shim 查询面，s378 同规不适用）。

## 口径

- 全树 sweep：base 8475 pairs（8227P/218F/32T raw 含探针 1F）→ sweep2 8475 pairs
  （8228P/217F/32T raw，探针 1F 已删下轮出账）。
- **本流域在册红 216F+31T → 215F+31T**（−1 driving 修复；探针行单列已删；home-end
  双态振荡在册 flake 单列不变）。

## 残差与边界（钉死入账）

- light 宿主命令式 shadow（createElement 宿主 attachShadow 后 append 进 light）：
  刷新后子树视图缺（今日行为同形，lenient 不劣化）——恢复需宿主 handle→dump 节点
  重锚（forward map 通道），语料面暂无钉案，FIXME 单列。
- sel 子 move-in 重放 `child_selector` 在剥模板基座查不到（终态在 shadow 内被剥除
  带走）→ lenient miss 视图缺该 sel 子——其无 handle，gCS by-handle 面不受影响。
- 剥模板开标签判按 `shadowrootmode` 子串（属性值含该词的病态串误剥）+ `<template`
  字面出现在注释/属性值内扰动深度配平——与 shim `_zwStripEchoHtml180` FIXME 同族，
  两发射源产物形态固定，语料面无钉例。
- 探测环 refresh re-parse 本身成本未动；本轮只修 handle 身份保真。

## 下一步

- 在册红 215F+31T。下一候选：object-handler 嵌套文档脚本通道（R206 object 移植）
  或本流域散单续勘；簇 D（customizable select 特性族）仍待用户决策。
- 挂账池承 s401（forward map 空返勘根/渲染流 5 面/form-submission-target 6T/
  sizes-auto 半边/A2+D 簇立项征询）。
