# s176（2026-10-31）：shadow 域定位选择器寻址——compound `zwsh<len>:<host>|<local>` 全链路 + slot 分配接线 + 同树 closest 根修，shadow-slots 段 LOST 20（0P/20F）→ 恢复注册 2P/18F

## 入场定刀位

s175 FIXME 承接：引擎侧 shadow 寻址。s173/s175 定谳 detached 镜像对引擎
light-dismiss/hit-test 不可见（shadow-slots 段 20 行 0P/20F，枚举通道无翻绿
价值）——重开前置 = shadow 内容以引擎可寻址形态进入既有读写/命中/布局通道。

## 设计

compound 定位选择器 `zwsh<len>:<host>|<local>`：

- **host 段** = light 树宿主的既有唯一选择器（`unique_selector_for_node`，宿主
  是 light 元素，s175 可达门通过）；
- **local 段** = shadow 根子树内唯一（阶梯 `#id` → `tag.class` → 裸 `tag` →
  位置链 `tag:nth-child(i) > …`，唯一性域收窄为 `query_selector_all(sr, cand)`，
  元素序 1-based 与 structural path 同约定）；
- `len` = host 段字节长，前缀切分使段内字符零约束（host 段含 `|`/`:` 无歧义）；
- 识别即权威：`find_by_selector` fastpath 三层返回——`None` = 非 grammar 走后续
  通道；`Some(None)` = 识别但不可解析，**不回落通用引擎**（compound 串非合法
  CSS，通用解析误匹配）；`Some(Some(id))` = 命中；
- 嵌套 shadow（宿主自身在 shadow 树内）不支持：宿主唯一选择器从 doc root 解析
  不着 → None，与深 JSON 序列化不下降嵌套 shadow 的既有边界一致。

## 根因链（勘根四层，逐层修复后 replica 3P/17F 实证）

shadow-slots 段 20 行全红的机制叠加：

1. **shim flat-parent 环**：engine `parentNode` 跳过 ShadowRoot 非元素父
   （`parent_element_node` 语义）→ shadow 节点的 parentNode 直接锚宿主；
   `_zwLdFlatParent` 把 shadow 节点摊平进宿主的 slot → 元素塌进自身 slot 成
   64-hop 环 → doc-order sort 腐败 → light-dismiss 误选 dialogA。修：compound
   （`zwsh` 前缀）节点穿通不摊平（part01.js `_zwLdFlatParent` 守卫）。
2. **parent selector 空串**：`parent_selector_for_doc` 经
   `unique_selector_for_node` 铸父选择器——封装查询门对 shadow 内容恒 None →
   空串 → shim 侧 null → flat 链首个 slot hop 截断（diag `chainB:[0,0]`）。
   修：`shadow_content_entry_selector` 回落（selector_match.rs）。
3. **slot 分配零调用点**：dom `resolve_slots` 生产端与 layout
   `get_assigned_nodes` 消费端各自就位，但 `resolve_slots` **无任何调用点** →
   分配表恒空 → layout slot 替换全走回退内容 → 被分配的 light 子树（含嵌套
   宿主链）无布局盒 → gBCR `[0,0,0,0]` → clickOn 坐标塌 body@(0,0) →
   light-dismiss 误分类。修：`materialize_declarative_shadow_roots` 逐宿主
   `resolve_slots`（shadow.rs）。
4. **命中/寻址断链**：runner `resolve_pointer_target` 页内扫描 `body, body *`
   走封装查询门，shadow 盒上的落点塌 body；engine hit-test 选择器铸造点对
   shadow 内容拿不到串。修：hit_test.rs 两铸造点 shadow 回落 + webview
   `element_from_point` 原生命中（runner headless 无
   ElementFromPointBridge）+ runner 仅对 `zwsh` 前缀引擎命中采信（light 路径
   字节级不动——s176 standing lesson：engine-canonical sels ≠ shim sel
   命名空间）。

### 回归勘根（本刀内闭环）：form-submit-dialog-shadow P2F

带 compound 实地址后 solo 域门 P2F 出现该案（s175 shipped 为 Pass）。勘根：
s173/s175 的 Pass 系**镜像惰性凑巧**——`shadowRoot.querySelector('button')`
返回镜像代理，寻址不着 → click no-op → dialog 保持 open。s176 实地址化后
click 到达真激活臂：`_flForm.closest('dialog')` →
`closest_matching_selector` 匹配集恒从 doc root 求 + `parent_node` 链穿
form → ShadowRoot → host → **跨界命中 light dialog** → method=dialog 关闭
dialog。spec：closest 沿祖先链求值，祖先不穿 shadow 边界（ShadowRoot 的
spec 父为 null）；whatwg/html#7971 即本语料 normative 见证。修：匹配域换
`shadow_scope_root`（同 `element_matches_test_selector` 先例）+ 命中返串
compound 回落。light 调用者 scope = doc root，行为逐字节不变。

## 施工（11 文件 +431/−31，rust 6 + shim 3 + 新模块 1 + 钉测 1）

1. **新模块 `js_dom_bridge/shadow_address.rs`**（~260 行）：
   `shadow_scoped_selector` / `shadow_unique_local_selector` /
   `try_shadow_scoped_fastpath`（位置链段按元素序索引直查——CSS 求值经
   `parent_element_node` 跳过 SR 非元素父，SR 直下元素位次被算进宿主域失真；
   minting 与解析两侧同计数口径对称）/ `shadow_scope_root` /
   `shadow_content_entry_selector`（嵌套 shadow 守卫）/ `ds_scoped_entry`。
2. **js_dom_bridge.rs**：mod 接线 + `find_by_selector` fastpath（识别即权威）+
   `node_entry_json` 元素条目 `s` 字段 shadow 回落。
3. **selector_match.rs**：`parent_selector_for_doc` shadow 内容回落（根因 2）+
   `closest_matching_selector` 同树根修（回归勘根）。
4. **dom shadow.rs**：`materialize_declarative_shadow_roots` 逐宿主
   `resolve_slots`（根因 3）。
5. **hit_test.rs**：HitTestNodeMeta / ElementHit 两铸造点 shadow 回落
   （drag-walk 铸造点不属本刀不动）。
6. **webview.rs**：`element_from_point` pub 方法（现点现建缓存，布局树为准）。
7. **runner testharness.rs**：`resolve_pointer_target` engine compound 命中
   采信门（仅 `zwsh` 前缀）。
8. **shim part01.js**：`_zwLdFlatParent` zwsh 穿通守卫（根因 1）+
   light-dismiss shadow 扩展（`_zwLdCollectTag`/`_zwLdShadowByTag`/
   `_zwLdDialogList`（light qSA + shadow collect + doc-order 排序）/
   `_zwLdResolveTarget`（compound 同串比对）/`_zwLdFlatChain`（root-first hop
   序，shadow-hop=0 约定）/`_zwLdDocOrderCompare`（prefix=ancestor-first）/
   `_zwLdFlatContains`/`__zw_dialog_light_dismiss_check`/`__zwDialogLdPick`/
   `__zw_dialog_light_dismiss_up`）。
9. **shim part04.js / part05.js**：children getter ensure-content 钩、`.open`
   IDL、`_zwDeclarativeShadowEnsureContent` push `_wrapSelector(String(entry.sel))`、
   `_zwDeclarativeShadowHydrate`、`_zwDeclarativeShadowEnsureMirrors`——
   枚举通道以实地址形态重开（s175 撤下的 detached 镜像通道的正解回归）。
10. **dom_bridge_tests.rs**：钉测 2 案（compound 寻址 `test_shadow_scoped_selector_addressing_s176`
    + local 阶梯 `test_shadow_local_selector_ladder_s176`）。

## 验证

- engine lib **3619P/10F**（基线 3617 + 2 钉测；10F 基线同键 fetch 族 9 +
  ua_display_li）——closest 修后复跑同键同态。
- dom lib **880P/0F**；webview lib **830P/0F**（两 crate 代码自该跑后零改动；
  closest 修后以 engine lib + interactive 域复跑承载回归面）。
- interactive-elements **439P/44F/483** 对 s172 基线（439P/44F/483）
  (key,state) join **恰 4 位移**：F2P=2（form-submit-dialog-shadow /
  dialog-focus-previous-outside——均 s173 双靶承接，非 s176 新增益）；
  P2F=2（closedby-complex shadow-slots 段 click 两行——s173/s175 在册「基线
  4P 为摊平 DOM 凑巧 Pass」中 2 行假 Pass 清除、真实 Fail 显形，非回归）。
  **对 s175 shipped 终态（463 键）：common 463 键逐键同态，零真 P2F；
  GAINED=20（2P/18F）**——shadow-slots 段从 LOST 20（hooks-on 探针态
  0P/20F）恢复注册至 2P/18F。
- forms 家族全树 **4298P/402F**（FILTER=forms 全树；s175 evidence 的
  96P/12F 系 goal-script 25 号脚本子集 108 键口径），对子集 108 键 join
  **零位移**；closest 修后复跑 4699 键逐键同态。
- fmt / guarded-clippy 零告警；shim 装配序拼接（part01,01b,02-07,08a,08,
  08c,08b,08d）node --check SYNTAX_OK（113576 行）；探针验后删（zz-s176-*
  4 文件，gitignored wpt-data 区）。

## 挂账

- **渲染流（不越界）**：nested-shadow 布局几何三面——dialogB 零尺寸盒
  (8,100,0,0)、嵌套 shadow fixed 定位、popoverB slot 流位 (39,218) vs
  top:250（/tmp/s176/fix7.log 痕迹）。shadow-slots 段 18F 的主体归因面，
  布局修复后预期大幅翻绿；细分勘定留渲染流入场。
- **FIXME 新记**：命令式 attachShadow 与宿主 childlist 后续变异
  （appendChild/removeChild/slot 属性改名）不触发 `resolve_slots` 重解析——
  动态场景分配过期，待变异应用臂统一挂接（shadow.rs 注释在册）。
- shadow_scope_root 不覆盖嵌套 shadow（宿主在 shadow 树内）——与
  `shadow_scoped_selector` 同边界，深序列化先例同款。

## 教训

1. **端到端接线缺口**：`resolve_slots`（生产端）与 `get_assigned_nodes`
   （消费端）各自就位而全库零调用点——slot 面的验收标准必须是 gBCR 非零 /
   命中可达的端到端证据，不是「两端都有代码」。
2. **封装门的次生传染**：`unique_selector_for_node` None →
   `parent_selector_for_doc` 空串 → shim 层当 null 截链——身份函数的回落
   链条要逐层审，单点门禁的 None 会在下游以空串/null 双形态显形。
3. **凑巧 Pass 的脆弱性**：s173/s175 的 Pass 依赖镜像惰性（寻址不着 →
   no-op）；实地址化是进度，但每揭穿一个凑巧 Pass 必须补上其背后的真语义
   （同树 closest），否则「进度」以 P2F 形式回账。s173 首刀 18 键 P2F、
   本刀 1 案 P2F，同一模式第三、四次出现——**镜像/凑巧层 Pass 不得计入
   基线承诺**。

## 残余

shadow-slots 段 18F 布局几何归因主体（渲染流挂账）之外，其余 interactive
域 Fail 全部 s172 基线既有在册（44−18=26F 与 s175 终态 26F 逐键同态）。
