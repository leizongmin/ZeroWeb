# s10k74 刀62：物化形 text-control 桥（_zwMEl value-range 面）＝OpaqueRange 残簇 93F 清翻＋replaceData 边界订正

- 日期：2026-11-28（UTC 00:39 发射）
- 基线：chain73（刀61 后 dom 失败行 222＋Timeout 10；bin 8743b2702810＝6906e14bf）
- 刀体：工作树三文件（part03.js `_zwMEl` input/textarea 臂桥＋part06.js eager 子树断连原语＋读时父视图成员资格补判＋replaceData 边界订正＋part04.js R5340 臂移除钩），bin 2085ad0cf971
- 电池：chain74 pid 3189349，17.5min，EXIT=1 与基线同形

## 判定（chain74）

- dom：失败行 222 → **129**（Fail 119＋Timeout 10/10 恒等；−93 全为 Fail 行零新增，comm 双向；Δ 算术闭合 222=129+93）：
  - **全翻面**：programmatic-updates 28（含 whole-old/shorter-longer 边界案——随 replaceData 订正清零）、auto-disconnect 17、display-none 16（残 2＝几何宽度断言）、offset 11、supported-elements 6、disconnect 5、range-updates 3（全部）、basic 3、unsupported-elements 1
  - **意外翻面**：geometry-basic 2＋geometry-multiline 1（不断言实际矩形尺寸的行为面子测随桥翻绿）
  - **保持挂账**：geometry-* 42＋display-none 残 2＝44、highlight+highlightsFromPoint 19、interactive-* 26、update-event-order 4
- 基线构成对账：222＝OpaqueRange 186＋非 OR 散池 26F＋Timeout 10；act 129＝OR 残 93＋散池 26＋T 10。
- san 5F／tt 32F 逐字节恒等（comm -3 diff=0）；ref rc=0
- census：84,543 → **84,636**（+93）。DC-3 校准线 ≥94,093，差 9,457。

## 结构定谳（探针 probe1–6，跑后已删）

物化形取素链（18 页语料全同形：`body.innerHTML=…; body.firstElementChild`）：

- 融合视图条目是 `_zwMEl` 工厂产物（`__zwIsMEl` 印、96 own keys、**无 sel/handle 身份**）——part03 方法表 createValueRange/setRangeText 分支不可达；`querySelector('input')` 同 turn 返回**同一对象**（融合视图喂查询面）。
- parent 链 `INPUT(M) > BODY > HTML > #document` 爬达 nodeType 9 → 刀61 `_k61OwnerDoc` 判活直接可用；`el.remove()` 后 parentNode null。
- 四点 IDL 缺口实证：createValueRange/setRangeText undefined、textarea `value=` 落 expando own 槽、input `.type=` 落 expando own 槽（type 迁移对 getAttribute 不可见——auto-disconnect type 迁移面与 unsupported-elements 类型门都要求 attr 可见）。
- 断连通道成员资格信号探针：条目身份在 attr/style 写后稳定（K）；`childNodes.indexOf` 是正确信号而 `body.contains` 走 stale 爬链不可用（L）——`innerHTML=''` 清空后 indexOf=false、contains=true。
- 懒判活无「曾移除」记忆：remove→未读→reinsert→读 误活（probe W）——断连永久性须在移除点 eager 落定。

## 刀面（三件）

1. **part03 `_zwMEl`**（input/textarea 臂，沿刀61 原语铺本地桥）：key 惰性铸造（`_zwValueRangeKey` 单调序号）；INPUT `type` IDL 反射（getter 缺省 'text'／setter 反射写 attr）；INPUT value setter 包 s186 accessor 挂 `_zwOpaqueRangeReplaceAll`（同值短路）；TEXTAREA value accessor（getter=textContent／setter=textContent replace-all＋ReplaceAll 钩）；`setRangeText`（1 参 required／uint 转换先于 start>end IndexSizeError／suppress 窗口 try/finally＋精细 adjust／clamp；FIXME 挂：选区缺省形与 selectMode 重定）；`createValueRange`（非文本类 input 恒抛 NotSupportedError／双参 required／越界 IndexSizeError 先于 collapse／text 族={text,search,url,tel,password}）。`_zwMElRemoveChild` 挂 eager 子树断连（pure mEl 树父面）。
2. **part06**：`_zwOpaqueRangeDisconnectSubtree`（registry 子树域恒死——断连永久性在移除点落定）；`_k61Elaborate` 读时父视图成员资格补判（doc 爬链判活后：父 childNodes 不含本元素 → 死——覆盖 `innerHTML=''` 整体替换通道）；**replaceData 边界订正**：`p < offset+delLen` → `p ≤ offset+delLen`（spec dom-characterdata-replacedata boundary adjustment——p==offset+count 落删除区末端**折叠**；whole-old [1,3] 端 3==delLen 案实证刀61 初版 `<` 误落平移臂得 16）。
3. **part04 R5340 臂**（sel 父＋pending mEl 子移除路径——`el.remove()` 实际通路）：同款 eager 子树断连钩。

## 挂账（OpaqueRange 残账 93F，按域三分）

1. **几何 batch 2**（44F：geometry-basic 12＋multiline 12＋mutations 12＋complexity 6＋display-none 残 2）＝值域文本逐字形几何（`_zwTextElGeometry` 原语接控件 value 面＋display:none 联动）。
2. **交互/选区**（26F：interactive-basic 16＋overlap-and-selection 10）＝`setSelectionRange` 选区态＋setRangeText 缺省形/selectMode（桥面已留 FIXME 锚）。
3. **高亮/事件**（23F：highlight 8＋highlightsFromPoint 11＋update-event-order 4）＝Highlight/CSS.highlights registry＋value 写的 beforeinput/input 事件序（独立域）。
