# R5164 域核记录 — auto 高 inline-block 父高 strut 估计 + table 行位 claim 门（2026-10-17）

驱动案：`css/css-tables/whitespace-001.html`（1.05%）——csswg#8358 /
css-tables §consecutive-boxes：table 外壳内两个 50% inline-block 之间的
空白必须占位（B 落第二行）。附带翻绿
`css/css-flexbox/flex-direction-row-reverse.html`（3.68%）。

## 1. 域核

单测探针（Ahem 环境无关，断言全部用关系值）：

- **ref 形（display:block 外壳）**：两个 auto 高 50% inline-block 之间空白 +
  注释 → B 应落第二行、容器高 = 2×行高 + 边框。HEAD 实测容器高被砍到单行。
- **test 形（display:table 外壳）**：同形下 B 被 step 8 表格 pass 打回 (0,0) 叠 A。
- **显式高变体（h:20）**：HEAD 同样红（B (0,0)）——预存缺陷，非本轮引入。

根因两条，相互独立：

1. **auto 高塌行**：父容器 remeasure 时点子 LayoutBox 高未落（0）→
   `collect_items` 的 `w>0 && h>0` 门把原子盒降级为零宽 TextRun → 换行
   机会丢失、`lines=1` → shrink 臂把已正确的父高砍掉真实第二行。
2. **claim 链打回**：remeasure 的 position-reuse claim 对 Table/InlineTable
   过早生效——step 8 `adjust_table_layout_with_fonts` 随后重排表内子盒覆盖
   行位，而 step 10 `adjust_inline_block_positions`（跳过名单只含
   Flex/Grid/TableCell、不含 Table）因 claim 集跳过不再纠正。

## 2. 实现（inline_finalization.rs，双 kill-switch）

### 臂 1：高度旁路 IFC（`ZW_IB_AUTO_H_ATOMIC=0` 关）

**双 IFC 设计**：主 IFC 保持降级口径（行形状/片段/字符级断行语义与 HEAD
逐字节同源），strut 估计只喂一个同配置旁路 IFC（`h_ctx`），取
`total_height()` 作容器高。

- **strut 不喂主 IFC 的实证**：原子盒「真实化」后 break_lines 的原子前无
  条件断行会替换文本路径的字符级禁断（css-text-3 §line-break-details），
  line-breaking-atomic 族 MVS+原子无断点 22 案 0.0000→2.0833% 全带回归
  （v2 语料实测）。
- **条目资格**：`h<=0 && w>0` + 子自身有 inline 内容（空 spacer 不估）+
  非 `contain:size`（内容高 0 是语义）+ 前邻可折叠空白或容器首个 inline
  内容（跳过注释回溯兄弟；行首无先行 run 的原子永不吃原子断行门）。
- **容器级门**：兄弟间存在可折叠空白文本节点才启用旁路。首子资格域中
  「相邻原子 + 后随无断点文本」形（flexbox-items-as-stacking-contexts-002
  ref 侧 `><div` 相邻、line-breaking-atomic-005/010/012/015/027）会在原子
  真实化后把后随文本挤到 ZW 无断点词换行第二行（spec 应溢出不换行——
  ZW 潜在缺陷，另行挂账），v3 语料实测 flex-stacking-002 新红
  0.025→2.7046%；容器门后全消。
- strut 值 = `resolve_font_metrics_with_provider` 的行高，`max(0.5)` 下限。

### 臂 2：Table/InlineTable 不 claim（`ZW_TABLE_POS_CLAIM=0` 关）

`position_reuse` 赋值加 `!matches!(style.display, Table | InlineTable)` 门——
表容器行位交回 step 10 既有路径。显式高 inline-block 子的 table 容器
（HEAD 预存红）由此修复；auto 高容器主 IFC 降级口径下 refresh 宽校验
天然 bail、本就不 claim，本门只覆盖显式高域。

## 3. 语料迭代（全量 16,590，对比 R5163 基线 fail 1,119 / 可信 9,804）

| 版本 | fail | 新红 | 漂移 | 处置 |
|------|------|------|------|------|
| v1 strut 喂主 IFC | 1,119 | highlight-z-index-001、text-transform-fullwidth-001 | — | 弃（claim 路径覆写 taffy 烤入的相对 inset） |
| v2 双 IFC（无容器门） | 1,117 | 0 | 22 案 MVS 族 2.0833% | 弃（原子前断行替换字符级禁断） |
| v3 +条目资格 | 1,118 | flex-stacking-002 2.7046% | 13 案 | 弃（首子资格 + 相邻原子形） |
| **v4 +容器级门** | **1,117** | **0** | **2 案 0.0004** | **采纳** |

v4 终态：**+2 翻绿**（whitespace-001 1.05→PASS、flex-direction-row-reverse
3.68→PASS）、**0 新红**、可信 9,804→**9,809（+5）**。残漂 2 案
（inline-box-border/padding-line-break 0.0000→0.0004，均保持 PASS）：
ref 侧 inline-block（padding-left:50px）被旁路真实化后取规范正确的
原子前断行（3 行与 test 侧一致），残差为行位微移 0.04%，接受并挂账。

## 4. 验证

- 单测 +5（r5164_inline_block_auto_height_tests）：block 形两行高保持 /
  匿名表 cell 空白占位 / table 显式高行位 / shrink 臂不砍两行 / 无空白
  对照单行。
- **OFF 反证**：`ZW_IB_AUTO_H_ATOMIC=0` → whitespace-001 精确复红
  **1.05%**、flex-row-reverse **3.68%**（基线值）；
  `ZW_TABLE_POS_CLAIM=0` → unit 显式高变体精确复红（B (0,0)）。
- 全 corpus 16,590：fail 1,119 → **1,117**（+2 翻绿 0 新红）；真通过-可信
  9,804 → **9,809**。

## 5. 门禁记录

fmt 清；clippy workspace --all-targets -D warnings 0 告警；layout-engine lib
全测 **1963** 绿（1958+5）；make reftest **712/712**（Layout 508 + Text 204，
+1 案来自并行流）；make test **71 套全绿（21,074P/0F）**；product-smoke
6 页 struct PASS（welcome **17.07%** 与 R5162/R5163 同值）。bench-gate 定向
zero-layout-engine：ON/OFF 双跑结果见 goal 文档本轮条目（NEW=0 为门）。

## 6. 限制/挂账

- ZW 潜在缺陷（本轮不修）：break_lines 对原子后无断点词做词换行（spec 应
  溢出不换行）——line-breaking-atomic 族残余红的真根因，旁路容器门只是
  隔离；修复属断行域独立轮次。
- 显式高 inline-block 在 flex/grid/table-cell 内的既有 skip 名单未动。
- table-anonymous-objects-062 保持基线 1.0096% 不变——不同根因（匿名表
  推断 §17.2.1），留给 R5165。
