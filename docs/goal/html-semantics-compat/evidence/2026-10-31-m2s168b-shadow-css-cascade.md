# M2-S168b Shadow CSS 级联下发（commit 2/2）

**日期**: 2026-10-31
**刀位**: s168 commit 1 落地后续接（上轮 429 中断遗留 WIP：ACTIVE_SHADOW_CSS_ENTRIES 槽 + `compute_styles_with_shadow` 填表 + shim shadow 枚举已成形，靶面 a1–a4 反应链 a1 即卡）。本刀收口 commit 2：勘根四症结 + 靶面全绿。

## 靶面与反应链

`the-link-element/link-rel-attribute.html` shadow 案四断言：a1 初始级联绿（attachShadow + importNode 模板 clone，shadow link rel=stylesheet 外链 `.green`）；a2 同 turn `setAttribute('rel','no-stylesheet')` 翻黑；a3 `rel=stylesheet` await load 回绿；a4 同 turn `removeAttribute('rel')` 翻黑。

## 勘根定谳（四症结，全部探针/插桩钉死）

1. **light 覆盖门吞 shadow 下发**（a4 死因）：s168 commit 1 shadow 物化产物进入 `__zw_query_all` 文档全域枚举（`links=["#warm","#sl1"]` 实测），而其表文本键是 wrapper handle 形 `@__n3`——`_zwSsRecomputeExt` light 循环覆盖门（`hitK/hitS` 双 miss → 整体 `return`）把它当「未齐」拦下，**函数尾部全部下发（shadow 清空/entries/blob）不执行**：a4 清空死（div 恒绿）、p2 初始死（host2 记录永不派）。插桩链：`maybeStart rel="" isSheet=false` → removal-arm `keyInTable=true` → `recompute enter` 有、dispatch 无、无 EXC。修复：shadow 记录收集**前移到 light 枚举前**，顺带记 `_rcShadowSels` 身份集合（`__zwSelector`/`@handle` 双形），light 循环按集合跳过。
2. **query 枚举键形漂移**（症结 1 修复后 a4 仍死）：身份集合初版只记 `__zwSelector`+`@handle`，而 `__zwSelector` 对物化产物恒 null——枚举发的是 **id 派生 sel** `#sl1`（id 有）与**裸 tag 小写** `link`（id 无，诊断探针钉）。补两形 bookkeeping：`getAttribute('id')` → `'#<id>'`、`tagName.toLowerCase()` → 裸 tag。共存 light 同名 id-less link 会被一并跳过（引擎 lw 首匹配在物化序下本就可能落 shadow 产物，退化面早于本切片）——FIXME 记 shim：spec 侧 `document.querySelectorAll` 不见 shadow 树，engine 枚举排除 shadow 子树才是正解。
3. **part04 双 rel 臂 TAG 门严格 `=== 'LINK'` 阻挡 shadow link**：物化产物 plain handle 宿主 tag 表 miss → `_realTag` 回落恒 DIV → maybeStart 不触发。修复：同 media 臂容忍式 `(_ssRlTag === 'DIV' && !sel && handle)`（setAttribute + removeAttribute 两臂；helper 内 TAG 门同款容忍已有）。
4. **s168 memo 臂先行于 s154 印记门**（回归，目录首跑 5F 暴露）：memo 臂无 fired 守卫——同值 attr touch（rel/href/type/media）重入时直接落表 + **二次派 load**，`link-multiple-load-events` 四案 + `stylesheet-media-change-no-fetch` P→F。修复：memo 臂守卫加 `_zwSsFiredMap[_ssKey] !== _ssAbs`（同身份同 URL 印记在册 → 落 s154 门静默返；rel 移除臂清印记后再入仍命中 memo，语义不变）。

另修 **computed_style_cache tree_dirty 分支占位 bug**（上轮 WIP 遗留）：`shadow_scopes` 占位 `Vec::new()` 使 `finish_generation_update` 的 `slot.10.is_empty()` 谓词失真——shadow 页任何树脏变更误入增量通道退化文档表级联。修复：现场 `collect_shadow_scoped_sheets(&doc, &shadow_entries)` 同步重建（style-system `compute_styles_incremental` shadow-enclosed 退化臂 plain 无 scopes 同证危害）。

## 施工（7 文件）

- `js_dom_shim/part03.js`：`_zwSsRecomputeExt` shadow 收集块前移 light 枚举前（注释全量迁带 s168b 补记）；`_rcShadowSels` 身份集合（sel/@handle/#id/裸 tag 四形）+ light 循环 skip；memo 臂 fired 守卫。
- `js_dom_shim/part04.js`：setAttribute/removeAttribute rel 臂 handle-only 容忍。
- `js_dom_shim/part05.js`：`_zwMutationInDoc` shadow root 爬升臂（`_shadowHandleMeta[h]` → hostSel/hostHandle 跳宿主续爬——连接门 spec shadow-including tree 语义）。
- `js_dom_bridge/callbacks.rs`：`__zw_set_shadow_css_entries` 回调（wire `\u{1}` 分记录、`\u{0}` 分 host/sel/text → `set_active_shadow_css_entries`）+ cs_cache 键扩元 shadow_entries 分量。
- `js_dom_bridge/computed_style_cache.rs`：ACTIVE_SHADOW_CSS_ENTRIES 槽 + GENERATION_CACHE 11 元组（slot.9=shadow_entries slot.10=scopes）/HANDLE_VIEW_CACHE/cs_cache 键扩元（两谓词逐字同步）+ `finish_generation_update` shadow 分支（`slot.10` 非空 → `compute_styles_with_shadow`）+ tree_dirty 分支 scopes 现场重建。
- `js_dom_bridge/computed_style.rs`：`compute_styles_for_doc` 汇点——shadow_entries 非空走 shadow 路径，否则原路径（纯 light 页零开销）。
- `pipeline/mod.rs`：`collect_shadow_scoped_sheets(doc, entries)`——host 键分组（发现序）、`@handle` 形宿主跳过（FIXME：引擎侧代际文档锚定不可得）、`find_by_selector` miss 回落 `query_selector_including_shadow` → `doc.shadow_root(host_id)` → `ShadowScopeStylesheets`。

## 验证

- check / fmt / guarded-clippy workspace 全绿；shim 13-part `node --check` 过。
- engine lib **3611P/11F** = s167/commit1 基线同集（fetch 族 9 + ua_display_li + style_sheet_from_handle_r113），零新增。
- 靶面 `FILTER=link-rel-attribute` **3P/0F**（a1–a4 全绿，shadow 案翻绿）。
- 诊断探针 `zz-s168b-shadow-cascade-probe.html`（上轮遗留，p1–p6 链：查询/属性读/初始级联/light 不泄漏/rel 移除/rel 回归）**F→P**（gitignore 本地数据，不随提交）。
- 目录 `the-link-element` **114P/1F**（剔探针 113P/1F；唯一 F = `stylesheet-media-change-no-fetch`，见下节——**既有非本刀**）。对 HEAD 基线（非探针 112P/2F）净 **+1P −1F，P2F 0**。
- 目录 `document-metadata` **214P/2F**（双 F 既有：media-change + `the-style-element/non-parser-created-doc` 后者 stash 二分 HEAD 复证）。对 HEAD 基线（213P/3F）净 **+1P −1F**。
- memo fired 守卫回归面：`link-multiple-load-events` 4 案修复后保持 Pass（首跑 P→F 已被守卫翻回）。

## 过程注记：既有 bug `stylesheet-media-change-no-fetch`（非本刀回归，挂账）

目录首跑该案 Fail 后 bisect 三轮（WIP shim+HEAD rust / HEAD shim+WIP rust / 全 HEAD）**全 Fail**——worktree `88ca4f5db`（commit 1）复证**已然**，commit-1 账本「113P/1F 余 1F 同为 link-rel-attribute 级联面」未覆盖此案（当时 good.css/资源文件 10-29/30 才落，账面与今日语料不同代）。症状：动态 `createElement('link')`（id-less handle-only）+ media=none 取回完成后，**文档 `<style>` 规则的计算样式整体丢失**（gCS(p).color = `''`，非错值）；静态 link 变体 Pass，88ca4f5db 与 HEAD 同态。归因 engine 侧（动态 link 原生通道 × overlay 下发交互），本刀未勘根——**下一步候选①**。

## FIXME 新记

- engine `__zw_query_all` 枚举排除 shadow 子树（spec：`document.querySelectorAll` 不见 shadow 树——dom.spec.whatwg.org queryselectorall；shim 侧键形补全是退化面兜底）。
- `@handle` 形宿主 collector 跳过（承 commit 1 账本）：动态宿主（createElement 后 attachShadow）代际文档锚定不可得。

## commit 1 账本 FIXME 状态

- 嵌套 shadow 级联：未动（本刀 shadow 枚举 `_shadowRoots` 顶层遍历，嵌套面不入）。
- `outerHTML` 声明式模板 / `getHTML()` 分型：未动。
