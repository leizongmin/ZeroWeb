# R77 — M1 L2 live 视图第二刀：handle identity 反查方案验证 + 剩余 -8 诊断（负结果归档）

**日期**: 2026-08-16
**里程碑**: M1 L2（R43 诊断后的第二次实现尝试）
**结论**: **不 land**——case.html 130→122（-8）。但相对 R43（-10 且根因不明）前进两步：handle identity 反查方案**已验证可行**，probe 模式全绿；剩余 -8 组成已精确定位。

## 本次实现（已回退，零残留）

1. `with_query_doc_live(html, mutations, f)`——QUERY_DOC_LIVE_CACHE 键 `(html, mutations.len())` 全量重放（R43 形态复用）+ fail-soft 回落快照。
2. `LIVE_SELECTOR_TO_HANDLE` thread_local——apply_dom_mutations 返回的 handle→selector map 倒置；新回调 `__zw_selector_handle(sel)` 查询。
3. shim `_wrapSelector` 先查反查——live 命中的 handle-created 元素返回**原 handle proxy**（`_proxyCache['@'+handle]` 自动同对象 identity）。
4. 升级 `__zw_query_match`/`__zw_query_all` 两回调。

## 验证进展（相对 R43）

| 实验 | R43 | R77 |
|------|-----|-----|
| live 视图即时可见性（append→querySelector 命中） | ✓ | ✓ |
| handle identity（live 命中返回原 `__n20` proxy） | ✗（返 sel-proxy + 裸 string） | **✓（探针实证 obj:__n1,__n2,__n3）** |
| append→query→finally remove 语义 | 未测 | **✓（probe3 三 subtest 全过：mid=1/after=0/carry 无泄漏）** |
| case.js 确切子集（is_html + createElementNS 3 ns + try/finally） | 未测 | **✓（probe4 count=3 全 handle）** |
| case.html 全量 | -10 | **-8**（120→122 vs 基线 130） |

## 剩余 -8 诊断（未解，下刀点）

case.html 全量下 `getElementsByTagName abc` 仍 got 9 vs expected 3——**多 subtest 累积语义**：probe3/probe4 单 subtest 模式全过，但 case.html 的 setup() + 数十 subtest 序贯（setAttribute/getAttribute/createElement 族在前）组合下，前序 subtest 的容器在 host live 视图中未被移除。probe 无法复现的最小差异待找：
- setup() 的 outer_product 数据循环外是否还有元素创建
- `attributes[0].localName` 断言（setAttribute 族）失败是否改变了后续 subtest 的 pending 队列状态（test_set_attribute 的 div 不 append 不 remove——但 `setAttribute Abc` 断言失败 expected "abc" got "Abc" 说明 **live 视图下 attr 读回路径也变了**——`node.attributes[0].localName` 对 handle 元素走 shim registry，live 不应影响；疑 `_wrapSelector` 反查波及 attributes 查询路径的 sel 解析）

## 对 M1 L2 完整方案的增量设计输入

5. **handle identity 反查链路（本刀验证）**：apply 返回 map 倒置 + `__zw_selector_handle` 回调 + `_wrapSelector` 前置查询——三件套是 live 查询与 JS 侧 handle proxy 身份统一的可行形态，M1 完整方案直接采用。
6. **probe 模式与 case.html 全量的差异**在 setup/多 subtest 组合——下刀须带 per-subtest pending 队列快照 diff（对拍两个 subtest 间 mutations.len 与 live 查询结果）。
7. `_wrapSelector` 反查是**全局前置**——attributes/innerText 等其它经 sel 包装的路径也被波及（setAttribute Abc 的新失败形态疑此）——完整方案应把反查限定在 query 返回的 selector 集合内（wire 层标记而非全局回调）。

## 状态

代码全部回退（工作树 = R76 `2610061f` 基线，case.html 130 Pass 复核），探针文件清理。诊断归档供 M1 L2 第三刀使用。
