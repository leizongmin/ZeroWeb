# R5511 轮次证据：run-in 表内后代丢失归因——table-internal-in-inline IFC 语义域定性深结构转待用户决策⑨

日期：2026-11-13
轮次：R5510 下一步①（corpus FAIL 盘点续择轻量）
择案：run-in-contains-inline-002/004/005（✗1.41%/1.54%/1.41%，684F 在册最小家族）
结论：**无窄修可落**——根因是 table-internal display（cell/row）在 inline 上下文的
匿名 inline-table 语义整体缺失（CSS2.1 §17.2.1），跨四层；转待用户决策清单⑨等点名。

## 一、缺陷签名

- 002（cell）：`Run-in <span><span:table-cell>header</span></span>` 并入后 test 侧
  丢 "header"（ref 渲染 "Run-in headerStart"，test 渲染 "Run-inStart"）。
- 004（cell+float）：丢 "Run-"（`<span:cell><span:float>Run-</span></span>in header`）。
- 005（row）：丢 "header"（table-row 同样丢）。
- 003 **PASS 是假象**：其 run-in 含第二份裸 `<span>header</span>` 补位，cell 丢文本被
  重复文本掩盖（#33 self-source 掩盖活例）。

## 二、归因链

1. **PNG forensics**（REFTEST_DUMP，用后留档 target/reftest-dump）：002 test/ref 对照
   ——test 丢 bold "header"，行折点随之前移（第一行多容纳一词）。
2. **探针矩阵**（临时 zzzprobe 页 ×10，用后撤净）：run-in+cell 最小形（inline style、
   serif 16px/1、短文本）**✓0.09%**；bold/border/longtext/deffont 单 delta 全 ✓；
   **{bold+border+long} 三 delta 组合 ✗1.41%（两字体均复现）**；全 inline-style 复制
   原 002 形 ✗1.41% 同值。逐对组合（bold+def、long+def）与去除组合（noborder、
   nolong、nodef=serif）中恰缺任一 delta 的均 ✓。
3. **临时单测盒树 dump**（r5511_tmp_diag，用后撤净）：002 形 merged run-in 盒
   （target 首子、disp=RunIn、bl=false）w=54.4 h=18、**子树空**；原 run-in div 空
   （h=0）。四变体 merged 宽全同 → 盒宽非判别指标，丢失在 IFC/paint 层。
4. **ZW_PROBE_4328 全 run 流**（text.rs 临时插桩一行，用后撤净）：
   - target PathB（rip=Some）：`Run-in||Start|of|block...` ——**cell 的 run 为空**；
   - source 盒（rip=None）：`Run-in|header|Start...` 完整（同 DOM 子树另一 collect
     路径产出完整文本）。
   - 丢文本的 collect 臂定位：`build_flatten_run_for_element`
     （inline/collect_items.rs）——`has_flow_block_child` 排除表
     `{None|Inline|InlineBlock|InlineFlex|InlineGrid|InlineTable}` **无 table-internal
     族** → 宿主 span 判「含在流块级子」→ `String::new()` 空扁平 run
     （BLOCK_CHILD_NO_FLATTEN，t-trending 块子文本防泄漏臂——对真块子正确，对
     table-internal 子误伤：§17.2.1 inline 上下文应生成**匿名 inline-table**
     （行内级），R109 拆分也不把 cell 当块级（tree.rs run-in 判定
     `is_block_level_display` 只列 Block/FlowRoot/ListItem/Flex/Grid/Table——同文件
     已有正确分类先例）。`has_block_level_child`（R4312 walk 门）同形排除表 → walk
     路径同被挡，双门齐指向 flatten-empty。
5. **一般 inline 路径同受累**（超出 run-in 域的关键发现）：探针
   `A <span>B <span:table-cell>C</span> D</span>` vs 同形 display:inline 参照——
   **test 渲染 "A B"（丢 "C D"）却判 ✓0.06%**：min_mismatch_ratio 阈值吞掉
   ~130px 丢字差（<0.06%×480000=288px 允许）——**self-source reftest 判绿不可作
   本域验收**（#33 掩盖 + 阈值盲区叠加）；paint 层多盒补救
   （PROBE_4328：容器盒 collect="A|"、外层 span 盒="B|C|D"、另一盒="A|B|C|D"，
   painted_inline_nodes 去重与盒位交互产出 per-shape mask）解释了探针矩阵的
   「单 delta 过/组合败」形态。

## 三、深结构定性（为何无窄修）

正确修复＝匿名 inline-table 生成语义（CSS2.1 §17.2.1），涉及四层：
①collect_items 两门排除表（walk/flatten 选择与 flatten-empty）；②walk 臂对
table-internal 子的展开（现直接丢）；③`mark_anonymous_table_roots` 把孤立
table-internal 一律块化（`is_block_level=true`）与 IFC 层 inline 级语义矛盾；
④paint PathB 存储信号镜像与多盒去重补救面（R4312/R4310 信号族同步）。

**脆弱消费方**：tao-085-088（table-anonymous-objects 行内 cell/row 生成域）——
R5510 刚以门控收口保住其绿面（arm② 兄弟位移为其承担者），本域任何 collect 门
改动都会改其 run 流形状 → 必须全 corpus A/B + chromium oracle 逐案验收
（self-source 判绿无效，见 §二.5）。体量＝特性级 multi-session，超出轻量修复
护栏 → 待用户决策清单⑨。

## 四、本轮动作与卫生

- 常驻测试：无（归因轮，无代码变更落地；working tree 净，text.rs 插桩与临时
  探针/单测全部撤净）。
- 探针矩阵 10 页 + 临时单测 + PROBE_4328f 插桩：用后全部撤净（#31：探针页为本轮
  自建临时案，删除安全）。
- corpus 基线复用 /tmp/r5509/new-fails.txt（684F，上轮已验代码零差异）。
