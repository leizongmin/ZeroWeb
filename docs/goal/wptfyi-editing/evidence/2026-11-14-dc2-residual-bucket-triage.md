# DC-2 · d20 轮：残余失败三桶归类（triage，零引擎改动）

日期：2026-11-14。口径：d19 终态二进制 ledger（[d19 证据](2026-11-14-dc2-focus-noop-selection-modify.md)
selfull3 / inputfull，test-guard 包裹同机跑）。本批纯分析归类，无引擎改动；
产出修复队列供后续批次（d20+）按杠杆序开修。

## 总量与分布

- selection 全语料残余 **2,646F / 258 文件**（总 47,320P）：editing/other
  2,403F（196 文件）绝对主导；selection/sidecar 桶 shadow-dom 62F、
  contenteditable 33F、caret 16F、moveup/movedown 24F、bidi 12F、散页
  ~90F。CI 11-12 口径「selection bidi 0/92」已被 d19 modify 收敛到 12F
  （仅 modify-by-character RTL 视觉轴近似面）；shadow-dom 62F 主桶为
  tentative getComposedRanges 族（getComposedRanges 11+mutations 12+slot
  3+direction 7+isCollapsed 4+collapse-extend 4+flattened-tree 12 等）。
- input-events 全语料残余 **910F / 23 文件**：get-target-ranges 族
  871F（joining-dl 220 + deleting-in-list-items 158 + backspace 142 +
  forwarddelete 131 + joining-dl-and-another-list 76 + non-collapsed 35 +
  typing 12）+ exec-command 109F。

## editing/other 2,403F 桶归类（message 族聚类）

| # | 簇 | F | 形态 | 定性 |
|---|---|---|---|---|
| 1 | **子文档 execCommand 缺失**（rejection） | 333 / 33 文件 | `iframeElement.contentDocument.execCommand is not a function`（258）+ `utils.document.execCommand is not a function`（74）+ child doc 面散布 1 | **桥层缺口**：execCommand/queryCommand* 全族只挂主 document（part06 R3254 系），part05 子文档工厂无该面。delete/insertparagraph-in-child-of-{head,html}（designMode on/off × backspace/forwarddelete 全矩阵）+ typing-around-link DesignMode 变体的 setup/驱动链直接断 |
| 2 | **DesignMode 变体 getRootNode 断链**（rejection） | 146 / 8 变体 | `Cannot read properties of undefined (reading 'getRootNode')` | 同在 typing-around-link DesignMode：EditorTestUtils sendKey 的 light-DOM 上行 `e.getRootNode({composed:false})` 打在 undefined editingHost（child body 视图/视图链缺口），与簇 1 同域双断点 |
| 3 | **text-control execCommand 期望矩阵** | ~474 / 3 文件 | `In <input type=X>/\<textarea> in contenteditable, execCommand(...)`：多数 `expected false but got true`（selectAll 等少数反向 `expected true but got false`） | **行为门缺失**：选区落文本控件内时 execCommand 支持性/生效矩阵（tentative 语料按 Chrome 实录），本实现无此门恒 true；矩阵含反向行，修复须逐命令对照语料期望 |
| 4 | join-pre-and-other-block × method/block | ~138 | 块连接 DOM 断言 | P2/M3 应用语义域 |
| 5 | typing-around-link 非断言残余 | ~138 | assert_in_array 实值不在期望数组（如 `<b><a><i>XYabc</i></a></b>` 载体嵌套序） | DesignMode 断链解除后的真实行为面（载体吸收/嵌套序） |
| 6 | editing-around-select-element | ~100 | execCommand delete/insertText 与 `<select>` 共存期望 | 编辑须跳过/保形 select 元素 |
| 7 | cloning-attributes-at-splitting-element | 17 | `Cannot read properties of null (reading 'getAttribute')` | split 元素克隆路径 null 崩溃（bug，可修） |
| 8 | insertparagraph-in-child-of-{head,html} designMode 变体 | ~30 | `TypeError: iframeElement.c...`（同簇 1 域） | 随簇 1/2 解锁 |
| 9 | legacy-edit-command / editing-div-outside-body 等 | ~60 | execCommand 返回值/结果面散布 | 散簇，随主链收敛 |

## 修复队列（杠杆序）

1. **B1 · 子文档编辑面**：child document 挂 execCommand/queryCommand*
   （document 参数化薄面——corpus 面向 delete/forwarddelete/insertparagraph/
   inserttext 与 designMode 编辑宿主，选区取 childDoc.getSelection()）+
   child body/元素视图 getRootNode 面补齐。直释 **479F**（簇 1+2）并解锁
   DesignMode/delete-in-child 族的真断言面（簇 5/8 的后续收益另计）。
   位置：part05 子文档工厂 + part06 execCommand 主体参数化点。
2. **B2 · input-events getTargetRanges**：beforeinput
   `getTargetRanges()` 返回将受影响 DOM ranges（tentative 语料逐 inputType
   录音）。直释面 **871F**。需按 inputType 实现前瞻 range 计算（与 P2
   apply 引擎的选区映射复用面大）。
3. **B3 · text-control execCommand 门**：选区在文本控件内的支持性/
   生效矩阵（含反向行）。直释面 **~474F**，修前须逐命令核对语料期望表。
4. **B4 · shadow-dom getComposedRanges 族**：tentative composed-range
   映射（slot/flattened-tree 语义），**62F**。
5. **B5 · 小簇**：cloning-attributes null 崩溃（17F）、editing-around-
   select 保形（100F）、join-pre（138F）等按批穿插。

## 与既有账目的关系

- bold notbold 1F bisect（master.md 既挂）与 mt 慢页补跑保持挂账；本机
  本轮处于并行流 workspace 编译链接风暴（load 261），重型 bisect 构建/
  长跑顺延至下轮。
- d19 后 selection 残余 2,646F 中 editing/other 2,403F 的簇 1+2（479F）
  与 input-events 871F 合计 **~1,350F 属已知断链/缺口面**，修复队列落地
  后三目录预估缺口（~4.1k）将进一步收窄。
