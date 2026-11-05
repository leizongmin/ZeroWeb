# M3 续 dataset 簇收口（2026-11-05）

## 簇规模与收敛

M1 基线 `html/dom/elements/global-attributes` dataset 六案（dataset.html /
dataset-get / dataset-set / dataset-delete / dataset-prototype /
dataset-enumeration，42 子测试）。M3 documents 收口时控制面挂账「dataset
3F」实为旧 `_datasetProxy` 面的可见部分；本轮按 spec 全语义收口：
**42 子测试 0F**（head 26P/16F → 终版 42P/0F，16 F→P）。

## 修复面（spec：HTML `#dom-dataset`「get a DOMStringMap's name-value
pairs」/ DOMStringMap getter/setter/deleter 步骤；DOM `#dom-element-setattribute`
R116/R122/R5126 既有小写化语义）

1. **named property 可见性 round-trip 门**（part05 `_datasetProxy`）——
   eligible 属性 = `data-` 前缀 + 余段无 ASCII upper alpha；可见键 =
   余段 kebab→camel。eligible 余段上 camel→kebab 恒复原，round-trip 门
   等价实现：`data--foo` 可见键是 `Foo` 不是 `-foo`；`datafoo`（setAttribute
   小写化产物）无 `data-` 前缀不入表。get/has/delete/ownKeys/gOPD 对
   不可见键统一：读/有落原型链（Reflect），删/枚举 no-op/不产出
   （dataset-delete `-foo` 组三断言 + dataset-prototype 透见面）。
2. **DOMStringMap 全局接口对象**——无 [Constructor]（`new` TypeError）、
   prototype non-writable non-configurable；**prototype.constructor
   non-enumerable**（WebIDL interface prototype object——enumerable 会经
   for-in 原型链枚举混入 dataset 计数，dataset-enumeration 3/2 计数面）。
3. **dataset ns 门**（part04）——dataset 仅 HTML/SVG/MathML 三 ns；
   `createElementNS('test','test').dataset === undefined`。ns 从
   `_nsHandles[handle].namespace` 印记取（同 R374 relList 判据；无印记 =
   隐式 HTML ns，解析产物 SVG 元素经 host ns 判定保持 dataset）。
4. **setter 校验序**（set trap）——① 名含 `-` 后跟 ASCII 小写 →
   SyntaxError DOMException（`-foo` 案）；② camel→kebab（大写前插 `-`
   转小写）；③ 结果名含 invalid attribute local name 字符（NUL/ASCII
   空白五字符/`/`/`>`）→ InvalidCharacterError（`foo␠` 案；`;foo`、
   `ெfoo` 合法名不拒）；④ 写入**委托元素自身 setAttribute**
   （`_makeProxy(sel, handle)`，见 6）。
5. **删除委托 removeAttribute**（deleteProperty 可见键臂）——直接
   `__zw_remove_attr*` 只记 host mutation，R122 客户端实例层
   （`_zwAttrInstances`）残留使 `hasAttribute`/`getAttribute` 读 stale
   旧值（dataset-delete 8F 根因，探针案定谳：delete 后 `'foo' in
   dataset` 已 false 而元素 hasAttribute 仍 true——attr-names 走
   mutation replay 无实例层，hasAttribute get trap 先查实例层）。委托
   removeAttribute 复用 R122 实例层剔除 + Attr ownerElement 解绑 +
   R46「属性不存在不产 record」MO 语义全链。不可见键 no-op 返 true。
6. **set/delete 一律经元素自身方法**——dataset set/delete 只落裸 host
   调用会绕过实例层 upsert/反射缓存同步/MO record/CE
   attributeChangedCallback 共享链（`_makeProxy` 按 `(sel, handle)` 缓存
   族（guidelines #32），identity 安全）；回调缺席回落裸 host 调用。
7. **原型 shine-through**——get/has 不可见键 Reflect 落 target 原型链
   （target 原型 = DOMStringMap.prototype → Object.prototype）：
   `dataset.toString === Object.prototype.toString`、`Object.prototype
   .expando = 42` 污染后透见 42，`"toString" in dataset` 为真。

## 关键判定

- **`data-Foo` → `dataset['foo']` 案不依赖本轮任何改动**：spec「All
  attribute names on HTML elements in HTML documents get ASCII-lowercased
  automatically」由既有 R116/R122/R5126 setAttribute 小写化承担（存侧
  `data-Foo` → `data-foo`），dataset 读侧只需按存储后小写名建键。
  上游 master 语料与 pin 逐字一致（curl 对拍）。
- 属性层（R116 小写化、R135 名校验、host mutation apply）零改动；
  dataset 全部语义收敛在 `_datasetProxy` + 全局 DOMStringMap +
  part04 ns 门。

## 遗留探针清理（corpus gitignored，本流工作残渣）

- `html/dom/elements/global-attributes/__m4dsprobe.html`（上轮 429 中断
  会话 dataset delete 诊断探针）——已删。
- `dom/__m3probe3-5.html`（M3 期 bisect 探针；M3 evidence 已清理
  probe2 同族）——已删。A/B 计数已剔除。
- `editing/other/zz-probe-*.html` 属 33 号 editing 流并行工作残渣，
  不属本流，未动。

## 验证

- 定向（FILTER=dataset 六案 42 子测试）：head 26P/16F → **42P/0F**。
- 全通道 A/B（test-guard 包裹，同日 back-to-back；head `4046c20a9`
  二进制 vs 本轮终版二进制均 make 重建）：

  | 通道 | head | 终版 | 结论 |
  |---|---|---|---|
  | testharness-html-syntax | 61808P / 562F / 7T | 61825P / 545F / 7T | **+17P/−17F**：16 落 dataset 簇 + 1 落 `__m4dsprobe` 遗留探针页（已删）；逐子测试对拍 0 Pass→Fail、0 新增 Fail |
  | testharness-dom | 56112P / 394F / 16T | 56162P / 396F / 16T | 剔除 `dom/__m3probe3-5`（WIP-only 探针页，已删）后 **0 Pass→Fail、0 新增 Fail**；`Range-mutations-insertData` head-only 13P 系 head 跑 1500s 时限 vs WIP 900s 的截断深度波动（Fail 列双侧恒 0，M3 同款） |

- 机时注记：dom 通道当日漂移超 M3 先例——head 首跑 900s 默认时限被
  guard 硬杀零产出，加长 `--time-limit 1500` 完成；WIP 跑 900s 内完成。
  墙钟跨跑不可比（guidelines #11），本轮语义结论仅取逐子测试集合差。
- documents 簇复验：FILTER=documents **229P/3F 不动**（M3 收口值）。

## 残余挂账（本簇无）

dataset 六案 0F 收口；`.window.js` 形态（dataset-binding.window.js）仍属
控制面挂账候选（形态缺口，非语义缺口）。
