# M2 切片 390（s390）——form 元素指针关联施工：解析期指针侧表 + 桥查询通道 + shim 同树门（4F→4P：form-control-infrastructure 域 4F→1F、the-input-element 域 5F+2T→4F+1T）

日期：2026-11-10（承 s389 下一步候选：the-input-element 域余红分诊——s387 sweep 观测 5F 四案逐案勘根流内可施工性）
域：html/semantics/forms/the-input-element（分诊起点）→ form-control-infrastructure（施工外溢主受益域）
性质：**施工轮——四案分诊（3 挂账 + 1 施工）+ 解析期指针关联三面施工 + 同树门回归修复 + 全树验收对账**

## 分诊（the-input-element 域余红 5F 四案，solo 实测 /tmp/s390/*.log）

| 案 | 红形 | 勘根 | 判决 |
|---|---|---|---|
| reset.html「form element pointer」1F | `reset button resets controls associated with their form using the form element pointer` | reset 表单归集走 `form_control_selectors`/`form_owner_node`，两路只认 form 属性与祖先嵌套，无解析期指针位 | **施工**（本切片 driving 面） |
| date-readonly-keyboard.html 1F | `Readonly date input should not change value via keyboard interaction` | runner 键盘交互仿真不触达 date input 内部 segment（trusted key 事件仿真上限，同 date input 键盘族史） | 挂账（runner 上限面，不施工） |
| radio-focused-and-scrolled-into-view.html 1F（文件级） | `Focusing offscreen radio via keyboard should scroll it into view` | keyboard focus 触发滚动入视（spec focus 步 scroll-into-view 语义）缺位，跨 layout/runner 两端 | 挂账（scroll-into-view-on-focus 立项面，单点刀不可达） |
| checkable-active-onblur-with-click.html 4F（7 子测 3P） | 直接按压路径 Pass；**label 按压**（parent/following ×checkbox/radio）4 子测 Fail | :active 清除在直接按压路径已通；label 按压激活转发路径的 active 态机缺位 | 挂账（label 按压激活面；sweep 观测 2F 为截断族，solo 全形 4F） |

## 勘根（HTML §13.2.4.4 form element pointer + §13.2.6.1 step 21）

form-associated 元素与指针 form 的关联在「create an element」step 21 由解析器裁决，约束集：元素 form-associated 且非 form-associated custom element、form element pointer 非 null、非 template contents 解析、fragment context 为 null、listed 元素无 `form` 属性例外、**intendedParent 与指针 form 同树**、插入目标重定向映射查。html5ever（0.38）已实现全约束检查并在通过时回调 `TreeSink::associate_with_form(&elem, &form, nodes)`——ZW 的 DomBuilder 此前未实现该回调，指针关联面整体缺位，坏标记位（表格 foster-parented `<form>`：form 是 table 兄弟非控件祖先；`<form>` 被隐式闭合后指针残留）全部落空。spec 期望：指针位覆盖坏标记位，常规嵌套两路同判；`form` 属性跨树关联（owner 算法首查项）shim 层已有（R4005）。

## 施工（9 文件三面）

| 面 | 文件 | 内容 |
|---|---|---|
| 解析期指针侧表 | `crates/dom/src/parser.rs` + `document/mod.rs` + `snapshot.rs` | `DomBuilderInner.form_owner_map` 侧表：`associate_with_form` TreeSink 回调落表（html5ever 已做全部 step 21 约束检查，ZW 只收对）；`from_builder_parts` 增参搬移（template_contents_map 同通道，NodeData 尺寸敏感理由同——不加 ElementData 字段防微基准劣化 R 记录在案）；`Document::parse_form_owner` 查询；`pristine_snapshot` 携带克隆（S1 快照 = fresh parse 契约不变式）。仅坏标记位有条目，常规文档空表。 |
| 域内归集回落 | `crates/dom/src/document/form_state.rs` | `form_owner()` 指针位优先、未命中回落最近 form 祖先——validation.rs R3284 复用面（radio required 组）同受益。 |
| 桥查询通道 | `crates/engine/src/js_dom_bridge.rs` + `callbacks.rs` + `form_activation.rs` | `form_owner_node` 次查指针侧表（form 属性 → 指针 → 祖先三级；fetch 数据归集/隐式提交 `enclosing_form_selector`/激活事务等既有消费面同受益）；沙箱注册 `__zw_parse_form_owner` 回调（控件选择器 → owner 唯一选择器，`parse_form_owner_selector_doc` doc 版供 live/快照双视图）。 |
| shim 指针兜底 + 同树门（s390b） | `crates/engine/src/js_dom_shim/part03.js` + `part05.js` | part03：`.form` getter 本地 form 属性/祖先两路落空后调 `__zw_parse_form_owner` 兜底（handle-only proxy 动态创建产物无解析期条目查表恒空不误关联）；`_zwLiveTreeRoot` helper（proxy 沿 parentNode 上行到根，`===` 根比较——wrap/getElementById 同 `_proxyCache`（R331/R4005）同元素恒同 proxy，根引用比较即树身份）。part05：`_formControls` host 列表入 elements/named items 前过同树门（owner 根取不到不设门保守沿用旧行为）。 |

### 同树门（s390b）——table_2 中途回归勘根与修复

**回归形**：Rust 三面落地后 `form_owner_and_table_2.html` 从 Pass 翻 F（4 断言单 test() 全红）。案面：脚本在解析中途 `t.removeChild(f)` 移除 form2 后，指针兜底仍把 input3.form 关联到已移除的 form2；且 `f['image1']` 经 `_formControls` host 列表泄漏（assert 4 潜伏面，assert 1 修复即暴露——先于触发补门）。

**根因链**：①spec step 21 同树约束（intendedParent 与指针 form 同树才关联——脚本移除后 form2 detached，同树不成立，浏览器判 null）；②ZW 两视图架构：engine 查询视图重解析原始 html，解析期脚本效应（中途移除）只存在于 shim JS 视图——指针侧表与桥通道均无法感知，**门必须放 shim 侧用 live proxy 事实判定**（解析期/桥侧门方案否决）；③host 查询索引对同 turn id 变更/移除 stale（R2302 判例），proxy `parentNode` 链是同 turn 变更立即可见的 live 通道（ARIA token resolver 同理由）。

**修复**：part03 `.form` 指针兜底加同树门——owner wrap 与控件 proxy 的 `_zwLiveTreeRoot` 根引用不同树即不关联；part05 `_formControls` host 列表同门。同树（form= 跨树关联、detached 双离树 R4005 案）行为不变。

**已知近似（代码注释在案）**：live 根门与浏览器仅在异态分叉——指针关联**之后**移除 form（浏览器保留 owner，ZW 置 null）与移除后重挂回；全树 sweep join 零回归实证无语料面触达。

## 验证

- **driving**：`make testharness-html-semantics FILTER=form-control-infrastructure` **120P/1F**（patch 前同 filter 4F：form_owner_and_table（table_1，切片起点主诉）+ form.html label-in-table-for/with-control 两子测 + table_3 在册残留）；`FILTER=reset.html`（the-input-element）**6P/0F**（patch 前 1F，F→P 两臂齐）；`FILTER=the-form-element` 107P/1F 不变（form-nameditem.html 在册键同基线）；`FILTER=resetting-a-form` 14P/0F（基线 0F）。
- **table_2 回归修复**：门后 fci 复跑 120P/1F——table_2 四断言全绿（input1.form===null / input2.form===form1 / input3.form===null / `f['image1']===undefined`）。
- **全树验收 join**：无 FILTER 全语料 sweep **8202P/261F/37T（8500 唯一 (key,state)）** vs s387 基线 8181P/265F/38T（8484 键，键集 3488 文件两侧全等）。方向矩阵恰 **4 键全 F→P** 且全部落在施工域：form-control-infrastructure 3 键（table_1 + label-in-table-for.form + label-in-table-with-control.form）+ the-input-element reset 1 键。**零 P→F 回归、零 Timeout 翻转**。
- **键集差归因**：+17/−1 全在 focus-dynamic-type-change-on-blur.html——基线 sweep 文件级 Timeout 键（在档 8-job 并行时序 flake，s387 已录 solo 全 80P/0T 判例）本轮落好态：文件 T 键 −1、+17 子测键全 Pass（80 子测全绿）。归档 flake 好态观测，非本轮位移，不入红账。
- **fmt/clippy**：`cargo fmt --all -- --check` 干净（associate_with_form 一处换行修为 rustfmt 落盘）；`make guarded-clippy` workspace `-D warnings` 全绿。
- **单元测试**：test-guard 包裹 `cargo test -p zero-dom -p zero-engine` **3828P/7F**；stash A/B 定谳非本流——本流改动 stash 后基线复跑**完全同形 7F**（img_runtime_deferred_settle_r5275 / svg_img_settle_density_r5277 / fetch_filtered_response_type_matrix / iframe_service_worker_messages_dispatch / response_body_readable_stream_r2967 / t8k_websocket ×2），即 s387 已录 img settle/fetch/SW/websocket 时序族组合态在册红（engine 属流，2026-11-07 checkout A/B 在档）。zero-dom 全绿。
- **产品 smoke**：`make product-smoke` PASS + `make product-smoke-legacy` 42/42 fixtures struct PASS（37-form-controls 口径维持——指针侧表常规文档空表，不波及 legacy 表单结构面）。

## 口径更新

- **form-control-infrastructure** 域 corpus 红 **4F→1F**：table_1 + form.html 两 label-in-table 子测销案；残留 **table_3 1F**（`parser inserted flag is not reset by insertions with the owner form, but reset by removal from the owner form`——assert expected null but got `<form>`；parser-inserted flag 插入/移除重置面，与指针关联侧表正交，勘根候选）。
- **the-input-element** 域 corpus 红 **5F+2T→4F+1T**：reset 指针面销案（−1F）；focus-dynamic-type-change-on-blur 文件级 T 键 −1T 为在档 flake 好态观测（+17 子测键全 Pass，不入红账）；date-readonly-keyboard 1F / radio-focused-scrolled-into-view 1F / checkable-active-onblur 4F（solo 全形；sweep 2F 为截断族）三案分诊挂账。
- 全树键级基线更新为 **8500 唯一 (key,state)**（s387 8484 键基线的键级继承：+17/−1 均已归因于在档 flake 好态）。

## 下一步

- form_owner_and_table_3 勘根（fci 域残留 1F，parser-inserted flag 插入/移除重置面——与 step 21 关联正交，error 形已定位到 flag 重置语义缺位）。
- A2 簇 + 簇 D 立项征询合并呈报维持待用户决策。
