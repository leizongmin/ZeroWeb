# M2 切片 391（s391）——form owner reset-on-move 读时门：解析期 parent 选择器桥 + shim 指针关联 liveness 门（fci 域 1F→0F 收官）

日期：2026-11-11（承 s390 下一步候选：form_owner_and_table_3 勘根——fci 域残留 1F，parser-inserted flag 插入/移除重置面）
域：html/semantics/forms/form-control-infrastructure
性质：**施工轮——spec/浏览器双探针勘根 + 读时 reset 模拟三面施工 + 全树验收对账**

## 勘根（HTML §4.10.18.3 Association of controls and forms #form-owner + Chrome 本地实测）

**driving 面**：`form_owner_and_table_3`（fci 域残留 1F）——「parser inserted flag is not reset by insertions with the owner form, but reset by by removal from the owner form」，assert 3 `expected null but got Element node <form></form>`（s390 指针侧表静态返回解析期 owner，不感知脚本移动）。

**spec verbatim**（multipage/form-control-infrastructure.html，2026-10-16 版）：

- 「A form-associated element has an associated **parser inserted flag**.」——test 名里的 flag 是现行 spec 真实概念。
- reset 触发面：「The form owner is also reset by the **HTML element insertion steps, HTML element removing steps, and HTML element moving steps**」+ listed 元素 form 属性 set/changed/removed +（有 form 属性时）树内 id 变更/带 id 元素插入移除。
- 「To reset the form owner of a form-associated element element」：① unset parser inserted flag；② **早退保留** iff owner 非 null 且（非 listed 或无 form 属性）且 owner 是变更后最近 form 祖先；③ 置 null；④ listed+form 属性+connected → id 解析关联；⑤ 否则祖先 form → 最近祖先关联。
- 关键推论：**foster 非祖先指针关联不满足早退条件**——控件自身任何 insertion/removing/moving 后关联永久失效（直到重入 form 子树走祖先臂）；owner 容器整体移动不触发控件 reset（insertion/removing steps 只作用于被移动元素本身，appendChild 单元素非 DocumentFragment 不级联后代；innerHTML 逐子插入才逐节点 reset——spec 自身示例注记）。

**解析形状实测**（Chrome 本地 puppeteer 探针，`<table><form><tr><td><input></table>`）：

- form 是 **table 子元素**（`form.parentNode === TABLE`，非 body 前兄弟）——"in table" 模式 form 起始标签规则直接插入当前节点，**不启用 foster parenting**（foster enable 由逐 token 规则设定，非模式级；该规则的插入不经 "any other start tag" 臂）。td 有浏览器合成 tbody 包裹。
- owner 语义实测：assert 2 移动 owner 容器（table 入 div#2）→ 保留 ✓；assert 3 移动控件 → null ✓；移回 form 子树 → 祖先重关联 ✓；**控件 `remove()` → null**（removing steps 重置——「detached 控件保留 owner」是过时认知，现行 spec 三步全重置）。

**方案定夺**：Option A **读时 parent-liveness 门**（live parent 选择器章 vs 解析期 parent 选择器桥查比对）。Option B tombstone（`_zwMarkRemoved` 钩点记账）否决——R334 appendChild 同父 move 分支「detach+append 不抽 DomMutation::Remove、不走 `_zwMarkRemoved`」（part04.js 判例注释），且 move/remove 触点面广（insertBefore/insertAdjacentElement/replaceWith/innerHTML 祖先替换…），钩点覆盖不全会漏 table_3 精确面；读时门把重置语义归约为「控件未动 = 指针有效，动过 = live 重算」纯函数，零变异钩点依赖。

## 施工（4 文件三面）

| 面 | 文件 | 内容 |
|---|---|---|
| 桥查询 | `crates/engine/src/js_dom_bridge/form_activation.rs` | `parse_form_parent_selector_doc`：控件解析树 parent 唯一选择器（无父返空串），`parse_form_owner_selector_doc` 邻位同风格。 |
| 沙箱回调 | `crates/engine/src/js_dom_bridge/callbacks.rs` | `__zw_parse_form_parent` 注册（`with_query_view_doc` 同通道）。 |
| shim 门 helper | `crates/engine/src/js_dom_shim/part03.js` | `_zwParseParentUnmoved(proxy, sel)`：live parent `__zwSelector` 章与桥返回值比对，不一致即失效；detached（parent null/动态容器无章）同判（removing steps 重置）；桥不可用/异常不设门保守沿用（s390b 风格）。`_zwLiveFormAncestor(p)`：live 祖先链 form 探测（TEMPLATE 止，R5196 同面），供 part05 条目判别。 |
| 指针臂三处加门 | `part03.js`（`_zwFormGet` 兜底臂 + get-trap 指针臂）、`part05.js`（`_formControls` host 列表） | getter/trap：臂结构已保证仅指针关联到达，门直加。host 列表：门只作用**无 form 属性且 live 祖先链无 form** 的条目——attr 关联移动后经重解析保留、祖先嵌套移动无关，两者免门防误伤；移出控件退出 elements/named items（table_3 收集侧镜像）。s390b 同树门保持叠加。 |

**已知近似**（代码注释在案）：move-away-then-back-to-same-parent（spec 恒 null，ZW 指针复活）——语料未触达，全树 sweep join 把关。

**架构面观察**（预存，非本轮引入）：脚本 `createElement` 的 form 无 engine 静态条目，`form=` 属性指向它的控件 getter 关联可用但 elements 收集不含（host 列表 engine 静态喂）——probe 记录，挂账不施工。

## 验证

- **双探针**：Chrome 本地 puppeteer 5 faces（/tmp/s391/chrome-probe.js：解析形状 + 断言三步 + move-back + removing-steps）；ZW probe 页 zw391_probe.html 4 faces **全 Pass**（table_3 面+elements 镜像 / move-back / removing-steps / form-attr 移动存活 getter 面；首版 attr 探针误把 input id 当 form 目标——spec 判 null 正确，探针修正）。验证后 probe 文件已删（wpt-data gitignore 本地数据，防未来 sweep 键漂移）。
- **定向域**：form-control-infrastructure **125P/0F/0T**（含 probe 4 测；域红 1F→0F——table_3 F→P）；the-input-element 1853P/4F/1T 与 s390 基线同形（4F+1T 全在册：checkable-active-onblur 截断族 2F、date-readonly-keyboard、radio-focused-scrolled-into-view、maxlength-number 文件级 T——s387+s390 双代在档）；the-form-element 107P/1F（form-nameditem 在册键）；resetting-a-form 14P/0F。
- **全树验收 join**（/tmp/s391/{b3,n3}.*，\x1f tab-safe (key,state)）：无 FILTER 全语料 **8207P/260F/37T（8504 唯一 (key,state)）** vs s390 基线 8202P/261F/37T（8500 键）。键集差 −1/+5 全归因：−1 = table_3 Fail 键（F→P），+1 = 同键 Pass + 4 = zw391_probe 新增（已删，键随语料消失）。方向矩阵**恰 1 键 F→P，零 P→F、零 Timeout 翻转、零多态键位移**；stable-P 8202 / stable-F 260 / stable-T 37 全保持。
- **fmt**：本流 4 文件干净；workspace `cargo fmt --check` 仅 `crates/script-sandbox/src/v8_runtime.rs` 两处漂移（91ab28c56 并行会话 source-phase imports 提交引入，非本流，记录不清理）。
- **clippy**：`make guarded-clippy` workspace `-D warnings` 全绿（exit 0）。
- **单元测试**：test-guard 包裹 `cargo test -p zero-dom -p zero-engine` **3832P/7F**——7F 与 s390 stash-A/B 定谳的组合态在册红完全同形（img_runtime_deferred_settle_r5275 / svg_img_settle_density_r5277 / fetch_filtered_response_type_matrix / iframe_service_worker_messages_dispatch / response_body_readable_stream_r2967 / t8k_websocket ×2；engine 属流），zero-dom 全绿（P 数 3828→3832 为并行会话 91ab28c56 等提交带来的用例增量，非本流）。
- **产品 smoke**：`make product-smoke` PASS；`make product-smoke-legacy` **39/39 fixtures struct PASS**（fixture 集 42→39 为并行会话演进，0 失败为准）。

## 口径更新

- **form-control-infrastructure** 域 corpus 红 **1F→0F**：s390 残留 table_3 销案，**域收官**。
- 全树键级基线更新为 **8504 唯一 (key,state)**（8207P/260F/37T；s390 8500 键基线的键级继承：−1/+5 均已归因——table_3 F→P 与 probe 新增，probe 文件已删后续 sweep 不再出现）。
- 全树在册红维持 260F+37T 构成不变（无新增红面）。

## 下一步

- fci 域收官后，forms 树域余红回到 the-input-element 挂账三案（date-readonly-keyboard / radio-focused-scrolled-into-view / checkable-active-onblur——均 runner 上限或跨子系统立项面）+ the-select-element 等域余红分诊。
- A2 簇 + 簇 D 立项征询合并呈报维持待用户决策。
