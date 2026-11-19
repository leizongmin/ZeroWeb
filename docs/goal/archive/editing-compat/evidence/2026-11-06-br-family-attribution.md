# 2026-11-06 br compare 族归因铺底（got 六形捕获，三机制定状）

iol/iul ?1-1000 br 边界 compare ×6（三形 × iol/iul 镜像）为 dps 轮后残差
首族。dps 轮收口后即以 dump 式探针 zz-probe-br（?1-1000 窗界单跑，期望树
逐字取自 data/insertorderedlist.js :13/:18/:23 与 insertunorderedlist.js
镜像行）捕获全部六形 got 树，归因定状如下；主刀实现为下一轮工作。

## got 六形（exp → got，探针日志原文）

| 形 | setup | exp | got | 选区 |
|---|---|---|---|---|
| A | `foo<br>[bar]` | `foo<ol><li>[bar]</li></ol>` | `foo<ol><br><li>[bar]</li></ol>` | 存活（#text@0..#text@3） |
| B | `f[oo<br>b]ar<br>baz` | `<ol><li>f[oo</li><li>b]ar</li></ol>baz` | `<ol><li>f[oo</li></ol>b]ar<br>baz<ol><li><br></li></ol>` | **灭失**（#text@1..#text@1 塌缩） |
| C | `<p>[foo]<br>bar</p>` | `<ol><li>[foo]</li></ol><p>bar</p>` | `<p><ol><li>[foo]</li></ol>bar</p>` | 存活（#text@0..#text@3） |

iul 与 iol 全同构（ol→ul）。原始日志 /tmp/zw-dps/br-probe.log（got 正文在
"page script threw" 行，runner 消息预算 3.3KB 内）。

## 三机制（统一根因：br 块边界未参与 list enable 分块）

- **A 吸收臂越界**：选区前邻 br 是前块（foo）的块终止符。invisible 吸收
  臂把 br 收进新壳作直接子节点（`<ol><br><li>`）；期望该行界 br 随提取
  **移除**（语料 :15 exp `foo<ol><li>` 前无 br——非壳外保留，初稿此点
  记反，勘误）。
- **B 选区内 br 未作 li 分裂点**：oo|b 之间 br 应被消费为 li1/li2 边界。
  got 中该 br 反被独立 wrap 成幽灵壳 `<ol><li><br></li></ol>`（且落位错
  到 baz 之后），真实次块 "b]ar" 落出 members 裸留 host；尾 br（bar|baz
  之间）应随 bar 块提取被结构边界（壳缘）替代消费，got 原位保留；选区端
  点未随结构重写补偿，塌缩灭失。
- **C 宿主块未按 br 分裂**：p 内 br 应触发 p 分裂——foo 块提取为 ol（p
  的兄弟），余块 = 新 p 装 bar（br 消费为分裂点）。got ol 嵌入 p 内
  （`<p><ol><li>`），br 吞失，bar 留原 p。

语料期望树 = Blink `insert_list_command.cc` 块分割口径（br 终止块；被提
取块的终止 br 由结构边界——li 界/壳缘/块分裂——替代消费），与 dps 轮
§① 同一契约口径（语料 pin 快照即契约）。

## 修复方向（主刀设计稿）

统一契约（三形 exp 合读）：**提取边界邻接 br 一律消费移除**——A 前邻
行界 br 移除（:15）、B 选区内 br 消费为 li 界 + 尾随 br 移除（exp
`</ol>baz` 直连）、C 拆分点 br 移除（exp `</ol><p>bar` 直连）；唯非提取
边界的空行载体 br 不动。落法：enable 前行界预遍按 br 切行——提取行成员
收临时块、边界邻接 br 移除、kept 行供主循环换型；C 形另需宿主 p 拆分臂
（壳出 p）。选区端点随 `_zwJusMovePreservingRanges` 身份存活（B 灭失面
随之闭合）。主刀实现由并行轮按 RUNSTATE SEAT-CLAIM（2026-11-06 13:40）
认领在途（part01e `_zwListEnableLineScope` 预遍臂），本席让位核验。

## 门禁预期与回归隔离

- 目标：iol 14→8 / iul 18→12（F2P=6+6，P2F=0 红线守），探针六形 exp
  逐字达标。
- 隔离注意：br 分块改动共享 enable 主循环——dps 归并出口、q12 出口收钳、
  qv13 壳标三分读面需回归（zz-probe-dps 11/11 + zz-probe-qv13 26/26 复
  跑），iol/iul 全腿 judge 三段对账（dps compare 族 ×12 不得回退）。
