# s10k73 刀61：OpaqueRange/AbstractRange 地基（构造器＋活调整原语）＝query 形 3F 首批翻绿＋物化形基建缺口定谳

- 日期：2026-11-28（UTC 00:07 发射）
- 基线：chain72（刀60 后 dom 失败行 225＋Timeout 10；bin 12946e4dfc4a＝9750ed392）
- 刀体：工作树三文件（part06.js 构造器/注册表/调整原语＋part03.js createValueRange 分支与 setRangeText 精细调整＋part04.js value setter 全量调整钩），bin 8743b2702810
- 电池：chain73 pid 3148743，17.6min，EXIT=1 与基线同形

## 判定（chain73）

- dom：失败行 225 → **222**（−3 恰 3 键零新增，comm 双向；Timeout 10/10 恒等）：
  - OpaqueRange-validation 2F 全翻绿（`querySelector('textarea')` 形）
  - OpaqueRange-range-updates "createValueRange works with disconnected form controls"（detached 动态树形）
- san 5F／tt 32F 逐字节恒等（comm -3 diff=0）；ref rc=0 Layout 521/521＋Text 204/204
- census：84,540 → 84,543（+3）。DC-3 校准线 ≥94,093，差 9,550。

## 结构定谳（探针四轮，跑后已删）

shim 元素三形态——**方法表可达性分叉是 OpaqueRange 巨簇 189F 的真实结构**：

| 形态 | 来源 | 方法表 | createValueRange |
|------|------|--------|------------------|
| handle 形 | createElement/innerHTML 动态树 | part03 `_makeProxy` 全表 | ✓ 可达（本刀） |
| sel query 形 | `querySelector` → `_zwQueryWrapIdentity`（`__zw_handle_for_selector` 命中 → `_wrapHandle`） | 同表 | ✓ 可达（本刀） |
| 物化形 | `body.firstElementChild` → `_childNodeList` 融合视图条目 → `_zwParseEl`（part02:5432，own 方法表 56 keys） | 第三张表 | ✗ 缺失 |

`_zwParseEl` 物化形缺口清单（probe v3 实证：style/focus/value 写读/appendChild/getBoundingClientRect 都有，**selectionStart/setRangeText/createValueRange 三点 undefined**；value= 落 expando own 槽假读——s242 OPTION 注记同款先例）：**R3019 `_ensureMutTree` 的 text-control IDL 桥**（沿 s242 转发先例铺 value/selectionStart/setRangeText/createValueRange → 本刀原语）是独立基建刀位。

## 刀面（三件）

1. **part06**（StaticRange 之后）：`AbstractRange`/`OpaqueRange` 全局构造器（原型链接：OpaqueRange.prototype → AbstractRange.prototype；startContainer/endContainer getter 恒 undefined）＋prototype 读侧（startOffset/endOffset/collapsed，读时**惰性判活**）＋`disconnect()`（幂等/隔离）＋几何 stub（空 rects/零 rect——batch 2 再做真字形几何）＋注册表 `__zwOpaqueRanges`＋创建工厂 `_zwOpaqueRangeCreate`（快照 owner document 与 INPUT type，auto-disconnect 判定基）＋`_zwOpaqueRangeAdjust`（DOM Range replaceData(offset,delLen)→insLen 边界规则：p≤offset 不动／删除区内折叠 offset／越删除区 offset+insLen+(p-offset-delLen)）＋`_zwOpaqueRangeReplaceAll`（value 全量替换＝replaceData(0,oldLen,newLen)，全部非零 offset 折叠 0——同值在 setter 短路不达）＋suppress 旗标。
2. **part03**：方法表 `createValueRange` 分支（input 非文本类恒抛 NotSupportedError；双参 required；`_zwSelUint` wrap；越界 IndexSizeError 先于 start>end collapse；UTF-16 code unit 计数）；setRangeText 内 suppress 窗口（try/finally 异常安全）＋精细 adjust 发布。
3. **part04**：INPUT value 模式与 TEXTAREA 两处 value setter 真值变更臂挂 `_zwOpaqueRangeReplaceAll`（suppress 窗口跳过）。

## 挂账（OpaqueRange 巨簇残账 186F，分三类）

1. **物化形基建刀**（≈140F：programmatic 28＋display-none 18＋auto-disconnect 17＋offset 11＋disconnect 5＋supported 6＋unsupported 1＋basic 3＋range-updates 3＋update-event-order 4＋validation 剩余 0）＝`_zwParseEl`/`_ensureMutTree` text-control 桥。
2. **几何 batch 2**（geometry-* 45F）＝值域文本逐字形几何（`_zwTextElGeometry` 原语已有，接控件 value 面与 display:none 联动）。
3. **交互/高亮**（interactive 26F＋highlight* 19F＋update-event-order 4F）＝selection 集成与 Highlight/CSS.highlights registry（独立域）。
