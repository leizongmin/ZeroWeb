# M2 切片 166：跨批 handle 正置表锚定 + media 翻转链三修（stylesheet-media-change 断言 1 翻绿）

- **日期**：2026-10-31
- **刀位**：s165 候选①（R51c 查询视图扩展）+ 候选③前半（link 通道 recompute media wrap 死代码），靶面 `the-link-element/stylesheet-media-change-no-fetch.html`
- **结果**：断言 1 翻 P（跨批 handle gCS 恒 '' → 正确红）；断言 2 仍 F——勘根精确化到外链槽位级联序（s161 FIXME 债，跨切片）；断言 3 因断言 2 前置未达（a2 翻绿后自然复验）

## 入场勘根（断言 1）

账本（s161）记载失败点为断言 2，实测前移：断言 1（`waitForStylesheet()` 后 `getComputedStyle(p).color === "rgb(255, 0, 0)"`）已红——got `""`。根因 = **跨批 handle 边界**（s161 FIXME 在册）：

- `computed_style_by_handle` 视图 = `parse_html(基座)` + 权威 applier 重放 `[applied..count)`——`nodes` 映射只覆盖后缀 `CreateElement`；
- p/link 均 createElement 动态创建，首 turn drain 后物化进基座，重放区间为空 → `nodes.get(handle)` None → gCS 恒 `''`。

**修法定谳（复用优先）**：R100 正置表基础设施已存在——`FORWARD_HANDLE_MAP`（callbacks.rs，webview `merge_handle_selectors` 每次 apply merge 后发布、导航清空、slice33 Remove 驱逐），`__zw_get_tag_handle` 的跨 execute 回落已用「handle→selector→`find_by_selector` 视图 doc」解决同一跨批身份问题。gCS-by-handle 通道未接入是唯一缺口。施工 = miss 路径接入同款回落，零新增 plumbing。

## 断言 2 勘根（探针 8 轮，验后删）

probe a（状态面）定谳：flip 时 `_zwSsRecomputeExt` 被触发（rcCalls=1）、pending handle 读可见 `'all'`（`__zw_get_attr_handle`）、sel lw 读盲（`sel_attr_override` 仅认 selector 键，`SetAttrOnHandle` 挂队不可见）→ media 读须 handle-first。

probe b/e（链路面）：四趟 `_zwStylesheetMaybeStart` 全带 `(null,__n0,noOv=false)`；翻转前唯一一次 `__zw_set_external_css('')`（shim 初始化装载）。覆盖门按 handle 形 `'@__n0'` 查找，落表键形随取回臂身份形态（sel 形/handle 形）→ **双键形覆盖门**修复（handle 形 miss 回落 sel 形，两形皆缺才整体 return，partial overlay 语义不变）。

死代码（s161 引入、s164 记账）：`_rcText` media wrap 装填后未消费，`_rcOut` 恒发裸文本——media=none 的表文本曾无条件入级联。修复消费。

probe g/h（判层）：翻转后直接 `__zw_set_external_css('p { color: green }')` 同特异度不翻绿；`'html body p { color: green }'` 高特异度翻绿（handle + selector 双通道同绿）→ **overlay→级联通道全通**，剩余缺口纯为级联序：`collect_stylesheets_with_parsed_external`（pipeline/mod.rs）把外链槽文本 push 在文档 `<style>` 之前——同特异度时文档 `<style>p{color:red}</style>` 后收集恒胜。即 s161 挂账「外链槽位级联序 FIXME」的精确定位（文档序交错需 collector/webview/shim 三层协调，全语料 A/B 独立切片）。

## 施工（3 文件）

1. `crates/engine/src/js_dom_bridge/computed_style_cache.rs`——`computed_style_by_handle` 增加 `forward: Option<&Mutex<HashMap<String, String>>>` 参数；`nodes` miss 时经正置表锚 selector、视图 doc `find_by_selector` 回定位（与 pipeline render 重建 rescue 重锚同法；selector 位次漂移与 rescue/slice33 同暴露面）；头注 FIXME 边界段翻新为已解路径 + 残余近似记账。
2. `crates/engine/src/js_dom_bridge/callbacks.rs`——`__zw_get_computed_style_by_handle` 传 `sel_handle_map_snapshot()`（锁仅在 miss 路径获取）；注册处注释同步。
3. `crates/engine/src/js_dom_shim/part03.js`——`_zwSsRecomputeExt` 三处：media 读 handle-first（handle 可锚时 `__zw_get_attr_handle` 先行，空回落 sel lw 读）；`_rcOut` 消费 wrap 后文本（死代码修复）；覆盖门双键形。s164 FIXME 块相应条目解除。

## 验证

- `cargo check -p zero-engine` 过；`cargo fmt` 本刀区域过（fmt 对 `js_dom_bridge.rs`/`computed_style.rs` 上游区域的扫动已回退，归口不动）；`make guarded-clippy` workspace 全绿。
- engine lib `js_dom_bridge` 单测 1359P/11F——失败名单与 HEAD A/B 基线同集（fetch 网络族 9 + `ua_display_li` + `style_sheet_from_handle_style_element_r113` 环境面），零新增失败。
- 定向 `FILTER=stylesheet-media-change`：断言 1 翻 P，失败点从断言 1 前移断言 2（`expected "rgb(0, 128, 0)" but got "rgb(255, 0, 0)"`）。
- `FILTER=the-link-element` 目录：112P/2F——`stylesheet-media-change-no-fetch`（断言 2 级联序债）+ `link-rel-attribute` shadow 案（s164 基线已红，s166 候选②靶面），双 F 均非本刀回归。
- 全量案级 join（full1 `/tmp/zw-s166-full1.log` 对 `/tmp/s165-full-run.log` 上游 s165 基线 7172P/776F/88T；(key,state) 对 `sort -u`，行首 `^(Pass|Fail|Timeout) html/` 过滤——基线 log 为 make+tee 包装混入语料盘点/诊断行，未过滤首跑 308 LOST 全为噪声假位移）：
  - **F2P 3 / P2F 0**：`conditionally-block-rendering-on-link-media-attr` ×2 + `style-element-media-match-block-script` ×1——基线为**断言级 Fail**（`script should be blocked until the stylesheet is loaded` got false；`Both style sheets … registered … after 2 seconds` Fail），本轮 Pass 行逐字对上。归因 = 覆盖门双键形修复的**饿死解除**：旧门单键形 miss → 整体 `return` → 该页全部 overlay 下发饿死（含 s164 style 文本通道的 @import 取回下发）——media-match 的 block-script 与 link 双表注册断言同源翻正。基线 Fail 非截断形态（Fail 键非 Timeout 键）是确定性失败主证；基线跑负载状况未知（另一会话树）不作归因主证；单样本 + 触发页形未二分——三案保绿转 s167 定向复验作门修复金丝。
  - **LOST 90 全截断镜像，无 Pass 翻红**：select-event 75 + focus-dynamic 8 Pass 子测试键（两 log 文件级 Timeout 键**均在**——同文件多跑截断深度差）+ name-attribute 2 Pass→文件 Timeout 镜像 + dialog-closedby 文件 Timeout 键换 2 Fail 子测试键（文件推进，在册环境漂移域非翻红）+ style_events 文件 Timeout 键换 error-event Pass（s164 搭车面）；selection/script-type-and-language 文件 Timeout 键新增 = 同文件多跑第二态（子测试键两轮全同）。本轮机器三跑并发（本全量 + dom 流全量 + runner rebuild）为最重负载窗口。
  - raw 终账 **7091P/775F/89T**（基线 7172P/776F/88T；ΔP −81 = 截断镜像净差）。靶面 stylesheet-media-change 单 Fail 键保持——单 async_test 断言顺序执行，断言 1 内部推进不体现为键位移（a2 失败整测 Fail）；shadow 案 Fail 保持（候选②靶面基线红）。

## FIXME 记账

- **解除**：跨批 handle 引用边界（正置表锚定回落落地）；link 通道 recompute media wrap 死代码（消费落地）。
- **精确化**：外链槽位级联序（s161 FIXME → 勘根证据链齐：collector push 序 + probe g/h 高特异度对照 + 双通道翻绿实证）——升格 s167 首选候选。
- **新记**：pending 键形盲区——`SetAttrOnHandle` 挂队对一切 sel-form lw 读不可见（本次 media 读 handle-first 落地；同类 sel-form 读点是否还有同形盲区未普查，踩坑语料再补）。

## 下一步 s167 候选

1. 外链槽位级联序（collector push 序 → 文档序交错，本刀证据链齐，需全语料 A/B 独立切片）；
2. shadow DOM 级联面（`link-rel-attribute` shadow 案）；
3. `color_scheme_decl` 继承序列化面勘根（s165 挂账）。
