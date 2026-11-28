# s10k76 刀63：OpaqueRange 几何 batch 2（控件盒行模型）＝44F 清翻＋mEl 真元素盒＋setSelectionRange 最小面

- 日期：2026-11-28（UTC 01:31 chain76 发射）
- 基线：chain74（刀62 后 dom 失败行 129＝Fail 119＋Timeout 10；bin 2085ad0cf971＝254d19425）
- 刀体：两文件 shim（part06 控件盒行模型＋死条目零盒门；part03 mEl gBCR text-control 臂＋setSelectionRange 最小面），bin `2e211023a070`＝提交域字节（发射锚断言＋链尾 md5 恒等）
- 电池：chain76 pid 3266862，17.7min，EXIT=1 与基线同形；chain75 中判轮（bin `f498a3ded55b`）先曝 disconnect 回归，修后重射

## 判定（chain76）

- dom：失败行 129 → **82**（Fail 71＋Timeout 11；算术闭合 129 = 82 + 47 − 1）：
  - **真翻面 44**：geometry-basic 12＋geometry-multiline 12＋geometry-mutations 12＋geometry-complexity-and-visibility 6＋display-none 2（几何宽度残案）
  - **组成迁移 1 行**：update-event-order 页 4 Fail → 1 页级 Timeout（`setSelectionRange` 解堵 setup 后测试进入 `test_driver.send_keys` 等待域——beforeinput/input 事件序＋真键入＝交互/事件簇挂账形态，页仍非通过；非回归）
  - **零新增 Fail**（comm 双向；per-page F 构成 diff 仅 6 归因页＝无隐匿回归）
  - ΔP +30 vs −ΔF +46 缺口 16＝Range-mutations 双页截断噪声带（F 构成恒等排除回归；census 只计 44 真翻面）
- san 5F／tt 32F 逐字节恒等（comm -3 diff=0）；ref rc=0
- census：84,636 → **84,680**（+44 只计真翻面）。DC-3 校准线 ≥94,093，差 9,413。

## 结构定谳（probe63＋solo 两轮＋chain75 中判）

- mEl face（物化形 text-control）`getBoundingClientRect()` 对 input/textarea/visibility:hidden 全返零盒（probe63）；`setSelectionRange` undefined（mutations 页 setup 全崩）；style width/whiteSpace/fontSize/lineHeight 读写回环 OK；`focus()` 在（R114）。
- 断言形状四页钉面：caret 盒（宽 0/高>0/入元素盒 0.5px fuzz）；选区逐行 rect ≥1、跨行 ≥2；软换行（pre-wrap＋width 160px）≥2 rect；mutation 关系断言（insert-before 右移／内插扩宽／append 左缘不动 ±0.5／delete-before 左移／overlap 收缩／'\n' 增高）；display:none 空、visibility:hidden 盒在；disconnect 后 gBCR **height == 0**。
- **solo 首轮证伪均匀步进**：mutations「append does not shift left edge」expected 34.023 got 33.092（|Δadv|=0.31 × 3 字符偏移 = 0.93 > 0.5 fuzz）——行均值随行内容改写漂移。改**字形 pens 1:1 精确位**（canvas measureText 逐字形原语 `_zwTextElGeometry`，字数==字形数时前缀字形位稳定）；聚类行（代理对 🌠A🌠 字数 4≠字形 3、阿语簇）落行宽/字数均匀近似（surrogate 归一断言 `wPair + 0.05 ≥ max(wHalf)` 均匀模型可过）。
- **chain75 中判曝死条目回归**：disconnect 页「bounding rect height is 0」——offset 归 (0,0) 的死条目落入 caret 盒臂返高 lineH≠0。修＝`_k63CaretBox` 首 `r._zwDead` 门返全零 rect（刀61 stub 语义保真）；solo disconnect＋basic 复跑 rc=0 后重射 chain76。

## 刀面（两件）

1. **part06**（替换刀61 几何 stub）：`_zwOpaqueRangeControlBox(el)` 控件盒行模型（display:none → null；textarea '\n' 硬换行分-row＋input 单行；软换行按 style.width/行 advance 断行 cap；空白行占行槽——blank-line bounding 高度断言面；行高 lineHeight 优先/首行 em 盒/1.25em 回落；行内定位 pens 1:1/均匀步进双模）；`_k63RowX`（pens 行字形原点−首字形锚，均匀行步进；末位=行右缘）；`getClientRects` 逐行 rect（collapsed/无盒 → []）；`getBoundingClientRect`＝并盒/caret 盒（硬换行位落本行行尾——multiline parity (6,6)/(7,7) 断言面）；死条目（`_zwDead`）全零 rect。
2. **part03**：`_zwMElGetBoundingClientRect` text-control 臂（input/textarea 经 `_zwOpaqueRangeControlBox` 返真元素盒——与值域几何同一模型构造保证 containment 恒成立；display:none 零盒；其余标签保持零 rect）；`setSelectionRange` 最小面（`_clampSelOffset` uint→clamp→start>end swap→`_zwSelStart/_zwSelEnd` own 槽；非文本类 InvalidStateError；FIXME：真选区态＋select/selectionchange 事件序挂交互刀）。

## 挂账（OpaqueRange 残账 49F＋散池）

1. **交互/选区 26F**（interactive-basic 16＋overlap-and-selection 10）＝`setSelectionRange` 选区态（selectionStart/End IDL＋selectionchange）＋setRangeText 缺省形/selectMode（桥面 FIXME 锚已留）。
2. **高亮 19F**（highlight 8＋highlightsFromPoint 11）＝Highlight/CSS.highlights registry（独立域）。
3. **update-event-order 4F（现页级 Timeout 形）**＝beforeinput/input 事件序＋test_driver 真键入（交互/事件簇，随 1 消化）。
4. 非 OR 散池 26F＋其他 T 10 不变。
