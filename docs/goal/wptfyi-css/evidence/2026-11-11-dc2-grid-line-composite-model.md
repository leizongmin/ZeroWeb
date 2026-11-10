# DC-2 grid G2a 批：grid-line 组合形模型贯穿（integer/ident 与 span 组合）

日期：2026-11-11　批次：DC-2 grid 批后续（G2a computed 面）　基线：grid-batch/json/css-grid.json（锚页 grid-layout-properties 94P/46F）

## 1. 勘根（锚页 46F 重新分桶，修正 G1 批的猜测）

锚页失败 message 抓取（grid-batch/json/css-grid.json）显示 G1 批残余 46F 实为两族：

| 族 | 失败 | 真因 |
|---|---|---|
| grid-line 组合形（12F） | `<integer> <ident>`（'1 a'）、`span <custom-ident>`（'span a'）、`span <integer> <custom-ident>`（'span 2 a'）×4 longhand + grid-column/row pair + grid-area | `parse_grid_line` 只支持单组分（auto/整数/`span N`/裸 ident）——'1 a' 因 starts_with 数字被拒、'span 2 a' 因 Span(u16) 模型装不下 ident 被 apply 拒，样式字段落 Auto，gCS 返 'auto'。round-trip 断言失败 |
| grid-template-* used 值（24F） | template columns/rows/shorthand 全形态（含 initial/none/reset） | 期望值是布局解算轨道尺寸（"150px"/"800px"/"[a] 150px [b] 100px [c]"）——Chrome gCS 对 grid-template-* 返 used 值，**需 gCS→layout 管道**。非本批范围，挂账 G2b（§5） |

G1 批证据把 46F 猜为「发射臂缺失/cascade 缺失/span 序列化」三桶——实测定谳：发射臂与 cascade 臂早已存在（R2759/R2766/R2734，computed_style.rs:749-766、apply.rs:1155+），缺口只在解析模型深度。

## 2. 通道定谳

失败形态（got 'auto'/'none'）只能由宿主 cascade 序列化默认值产生（serialize_computed_property 初值臂）——锚页元素走宿主 gCS 通道，改动全落 style-system/parse + engine/serialize + layout-engine 落位，不涉 shim。

## 3. 修复内容（提交 c535b64b6）

1. `style-system/property/types.rs`：`GridLineValue` 结构变体——`Line { line: i16, name: Option<String> }`、`Span { count: Option<u16>, name: Option<String> }`（`&&` 组合装得下，Auto/Name 不变）。
2. `style-system/property/parse.rs`：`parse_grid_line` 全文法重写（css-grid-2 §7.1 `<grid-line>`）——`[<integer> && <custom-ident>?]` 任意序、`[ span && <integer>? && <custom-ident>? ]`、裸 `span`；行号非 0、span 整数正 u16 域；ident 校验对齐 grid_cssom::grid_line_custom_ident（拒 auto/span 关键字、数值字面量、数字开头、含 `/`）。**token 边界纠正**：`span2` 无空格是合法 custom-ident 非 span 组合——旧实现的裸前缀匹配系 guidelines #2 缺陷，4 处钉死旧行为的测试随批改判。
3. `engine/js_dom_bridge/computed_style.rs`：`grid_line_to_css` canonical 序列化（整数在前、span 前缀序，对齐 grid_cssom::grid_line_canonical）；pair/area 最小化规则与 Name 守卫不变。
4. `layout-engine/converter/grid.rs`：落位映射——span 带名线 `NamedSpan(name, n)`、裸 span `Span(1)`、行号带名线按整数域落位（命名线布局消费 FIXME 挂账：需模板 line-names 表）。

## 4. 测量（v13 记录轮，results-v13/）

css-grid corpus 全量（zwv13-run，release build 40.89s + corpus 跑动）：635 页 cases=635 pass=1415 fail=6122 timeout=76。

逐页对比 results-v13/css-grid.json vs grid-batch/json/css-grid.json：

- **共同 635 页：净 +26、零回归**（gained=26 lost=0）。
- 锚页 grid-layout-properties.html：94P/46F → **115P/25F（+21）**——翻绿明细恰为 3 组合形 × 7 属性：`<integer> <ident>`、`span <custom-ident>`、`span <integer> <custom-ident>` × grid-row/column-start/end（4 longhand）+ grid-column/grid-row（pair）+ grid-area，21 条全绿。勘根时把翻绿面估为 grid-*-start/end 12F，实际 pair/area shorthand 的组合形断言走同一 `parse_grid_line(_shorthand)` 路径，连带 +9。
- parsing/grid-area-computed.html：8P → 13P（+5）——`'9 -Z_'`、`'1 -πA'`、`'5 π_'`（Line 组合形，unicode ident 通过）、`'span 2 i'`、`'span i'`（Span 组合形）。
- 锚页剩余 25F 全部为 grid-template-* used 值族：columns 10 + rows 11 + shorthand 4，grid-line 残留清零，与 G2b 挂账面吻合。
- 页集合差：基线分片含 19 页 subgrid 页，v13 corpus 清单不含（runner manifest 差异，非回归；subgrid 页不涉 grid-line 组合形）。

## 5. 挂账

- **G2b：grid-template-* used 值（锚页 24F + 分片同型）**——需 gCS→layout 管道：taffy 已有 `detailed_layout_info()`（taffy_tree.rs:884，feature `detailed_layout_info`）→ `DetailedGridInfo{rows, columns}`（DetailedGridTracksInfo.sizes 即 used 轨道尺寸）；接线清单：layout-engine 保留 per-node grid 明细 → gCS 通道可达（computed_style_property 现无 layout 树）→ 序列化交错 stored decl 的 line-names（"[a] 150px [b]"）。架构面改动，独立批。
- 命名线布局消费：converter 落位忽略 name（FIXME css-grid-2 §8），需模板 line-names 解析面。
- content 图像 light-dark 1F、image-set/cross-fade 嵌套 1F（归 image 链账，68 批已记）。

## 6. 协作记录

同树四实例并行（68 image 链 8c166641d、70 fill/stroke+currentColor 链、e2 Typed-OM、17 DC-3 只读预备）。本批 computed_style.rs 为共享面（我 grid hunk + 70 fill/stroke hunk 未暂存并存），提交用 `git apply --cached` hunk 级拆分只带 grid_line_to_css 段；part04.rs 新增 G2a round-trip 测试随 68 同文件提交入库（8c166641d，+123 行=其 image e2e + 本批 33 行）。测量号段仲裁：v11=70（color 10,808→10,840 净+32 逐页零回归）、v12=68 image 落库终账、v13=本批 css-grid 分片。
