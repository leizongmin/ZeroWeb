# ihtml 残面机制普查——inserthtml 拆分插入路径缺失 + 场景选区解析惰性归因

- 日期：2026-11-13
- 数据源：`.acceptance/dps/e1-ihtml.log`（r2 post @ffdcd28af bin 788d4b10 组合态，judge vs v2 post VERDICT OK 零漂移背书）；分析脚本 `.acceptance/dps/census-ihtml-r2.py`（只读）
- 方法：119 Fail 键全量解析（swcss 态 / inserthtml 载荷形态 / 场景容器 / 断言型 / expect-got diff 签名）；swcss 双态同形键去重计缺陷。前档 [2026-11-12 族归属普查](2026-11-12-residual-family-census-r5450.md) 只到容器/命令轴，本轮进机制轴。

## 总定性

119 键**全部** `compare innerHTML`（无 qV/qI 债）；双态去重后 **89 缺陷**。按 expect/got diff 签名：

| 签名 | 键数 | 定性 |
|---|---:|---|
| **NOOP（插入内容完全未到达，got=场景原样）** | **101** | 绝对主面——inserthtml 在这些场景整体未应用 |
| SHAPE（已应用但形态/包裹错） | 9 | 孤列表项块化 delta 4 + 未闭合行内载荷吞后续 4 + CE-false 邻接 1 |
| assert_in_array（归一化数组期望） | 9 | 注释位 3 / 空 span 载荷 3 / pre-line ws 2 / pre-CE-false 1，多数亦为 NOOP 变体 |

## 机制正交两轴（89 缺陷归因）

现有实现面 = 「简单场景（bare-text/p/div 文本位）内联片段插入 + caret 恢复（F3 刀）」；残面缺口在两条正交轴上：

**轴一：载荷块级性 →「容器拆分插入路径」缺失（~63 键 / ~44 缺陷）**

载荷含块级/列表/表格/hr 标签时全场景惰性（got 原样），或孤载荷到达但缺法定包裹。Chrome 语义 = 删选区 → 按载荷块级性**拆分当前容器**（块容器拆 p/div/li/dt/dd；行内同名容器拆 a/nobr 为三段）→ 插入 → 归一：

- 块载荷全场景 NOOP 50 键/29 缺陷：`<p>abc`（未闭合，parser 补全）进 `<p>foo[bar]baz` → exp `<p>foo</p><p>abc</p><p>baz</p>`；进 `<ol>{<li>foo</li>}<li>bar</ol>` → exp 拆 `<p>abc</p><ol>…`；`<li>abc</li>` 进 `<ol><li>f[o]o</ol>` → exp `<ol><li>f</li><li>abc</li><li>o</li></ol>`（24 键 dedup 23，块载荷 list 族几乎每案独立场景）。
- `<hr>` 断块效应 4 键/2 缺陷：`<p>f[o]o` → exp `<p>f</p><hr><p>o</p>`（载荷单标签但块级语义）。
- 行内同名容器拆分 3 键/3 缺陷：`<a>abc</a>` 进 `<a>f[o]o</a>` → exp `<a>f</a><a>abc</a><a>o</a>`（nobr 同型 ×1）——与块拆分同一原语的行内层。
- 孤列表项块化 delta 4 键/4 缺陷（已应用形态错）：`<li>abc</li>` 进 bare `f[o]o` → exp `f<div>abc</div>o` / `f<p>abc</p>o`，got `f<li>abc</li>o`（到达、缺包裹）；`<dt>/<dd>` 进 bare → exp 包 `<dl>`（×2）。

**轴二：场景类 →「选区端点解析/非可编辑邻接回落」惰性（~37 键 / ~30 缺陷）**

同一载荷在特定场景类整体静默无效（含内联载荷）：

- **styled span 选区内 24 键/11 缺陷**：`foo<span style=color:#aBcDeF>[bar]</span>baz` + `ab<b>c</b>d` 载荷（swcss 双态同败）→ got 原样；`{bar}` 局部选区同败；`[bar</span>baz]` 跨出壳形 → **半应用**（foo 被删且载荷到达，但选区内 span 残留 `bar` 未删，2 键 1 缺陷）——壳内选区端点解析失败是主形。
- **raw-text 容器 9 键/7 缺陷**：xmp/script/pre 各 3，`abc` 载荷进 `<xmp>f[o]o</xmp>` → exp `<xmp>fabco</xmp>`，got 原样——raw-text 上下文 caret 定位/逐字插入缺失。
- **contenteditable=false 邻接 3 键/3 缺陷**：CE=false span 邻位 `[X]`/`[Y]` 插 `<i>Z</i>` 未应用/未按期望半删；`<pre contenteditable=false><span contenteditable>[1234]</span>` 插 `<pre>abc</pre>` 数组期望未达。
- **simple 场景特例 ~5 键**：全选 `<p>[foo]</p>` 插空载荷/`<span style=display:none>`（3 键 2 缺陷，空内容插入后占位 br/空壳未落）+ `<table><td>` 场景 1 键。

**散余**：未闭合行内载荷吞后续 4 键/2 缺陷（`<b>`/`<b>abc` 载荷 → got `foo<b>abcbaz</b>` vs exp `foo<b>abc</b>baz`——插入后 caret 应落壳外，baz 被吞进新壳；F3 caret 刀邻域残余）；assert_in_array 归一化族 9 键/9 缺陷（注释位敏感性 ×3、空 span ×3、pre-line 尾空白 ×2、pre-CE-false ×1——Chrome 录音含注释/空白逐位形）。

## 与 ip 道交叉及刀序建议

1. **拆分插入原语与 ip 刀共享**：轴一的容器拆分语义 = insertParagraph div/p 容器子族 116 键（并行轮已认领）同一底层原语的第二消费方——ip 刀落地的拆分原语可直接被 ihtml 轴一复用（呼应 2026-11-12 普查「ihtml 随 ip 道归并」预判）；孤列表项块化 delta（4 键）亦属该原语的载荷归一面。
2. **轴二场景解析惰性为独立道**：styled span 选区（24 键最大子族）+ raw-text 容器（9 键）+ CE=false 邻接——选区端点在行内壳/raw-text 上下文的解析回落缺失，机制独立于轴一，可作并行小刀；styled span 半删残余（1 缺陷）挂 deleteSelection 壳边界族（F6 线）。
3. **吞后续 4 键**挂 F3 caret 邻域复检；**assert_in_array 9 键**多为注释/空白归一化敏感性，修复优先级最低（ Chrome 录音逐位形，建议收尾批量判读）。
