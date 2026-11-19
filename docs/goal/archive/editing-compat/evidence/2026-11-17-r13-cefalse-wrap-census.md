# R13 cE=false wrap 刀前普查（归域轮，2026-11-17）

## 归域结论

R13 刀域＝**全选区含 cE=false 行内内容的 wrap 命令能力**，共 3 键：

| 键 | 语料 | 命令 | 现状 |
|---|---|---|---|
| fb ×1 | formatblock.html?5001-last 条目 362 `{ <span cE=false>A</span> ; <span cE=false>B</span> ; <span cE=false>C</span> }` | `[["formatblock","p"]]` compare | 整命令 no-op（树不变），ret [true] 行 Pass |
| indent ×2 | indent.html 同形 `{ ... }`，swcss true/false 双形 | `[["indent",""]]` compare | **部分误应用**——仅首个 ` ; ` ws 文本子被包进裸 `<blockquote>`，三个 cE=false span 滞留原位 |

Chrome 期望（assert_in_array accepted arrays）＝**整个选区内容（三 span＋ws 文本）包进单个块容器**：fb→`<p>`（3 ws 变体形）；indent swcss=false→`<blockquote>`（plain/webkit-style 4 形）；indent swcss=true→`<div style="margin-left:40px">` ∨ `<blockquote style="margin:0 0 0 40px; border:none; padding:0px">`（**不含 plain blockquote**）。

## 根因链

1. **fb**：part01g `__zw_ce_fb_apply` 可编辑链守卫（:158-167，注记 :148-157 有意）——起点叶子到宿主链遇不可编辑元素「拦（零树变更）但返 true」，注记明言「语料条目 362 真意 wrap 不可编辑节点、本引擎不具备该 wrap 能力只拦不报 false」。R13＝补能力：守卫拦截面转 Chrome delta wrap 臂。
2. **indent**：part01g `__zw_ce_indent_apply` node list 过滤器 `isEditable && allowed-child(div/ol)`（:710-714，vendored implementation.js :7092-7098 忠实移植）——cE=false 元素非 editable 被滤出 → node list 只剩 ws 文本子（互不邻接，被 span 隔断）→ sublist 收敛为孤立文本子 → `_zwFbqIndentNodes` 逐子包壳（实跑 got 恰「首个 ` ; ` 独包」实证）。Chrome 录音含 cE=false 元素整体单容器 wrap＝Chrome 对 vendored spec 该步的 delta（语料 accepted 为 Chrome 录音口径）。
3. 两命令同根＝**wrap 目标收集把 fully-contained 非可编辑元素排除**；Chrome 实际行为整体纳入。

## 影响面约束（刀设计输入，本轮数据）

- indent 现树 fresh 腿（evtr-census-indent.log，head baf41cb14 域）＝**1304P/25F/0T**（census v1 27F→25F，两键被 intervening 刀翻绿）；cE=false 三连形双键仍在册。25F 其余族：rtl ×1、span-br collapsed ×1、tbody data-start ×1、br ×3、blockquote ×9、ol 嵌套 ×5、ul-br ×1、swcss=true margin-div ×2、cE=false 嵌套 editable ×1——均非本刀域。
- **容器形态分叉**（设计 fork，下轮定谳）：wrap-new 臂今日恒产 plain `<blockquote>`（part01g :670）；swcss=false accepted 含 plain ✓，swcss=true accepted **不含 plain**——swcss=true 键翻绿需 margin-div ∨ webkit-style 块quote 形。同面既有 FIXME（:651-653「margin-div 臂未移植」）与独立 census 族「swcss=true margin-div ×2」（`<p>[abc]</p>` got plain blockquote）——该族非本刀域（2 键），但臂形态决策须对账其 accepted arrays，防 P2F 于现役 plain-blockquote 绿行。下轮开工先 grep indent.html 语料 accepted arrays 全量对账再定臂形。
- guard 退役面：fb 可编辑链守卫注记唯一拦截面即条目 362（守卫注记自述）——转 wrap 臂后其余 fb 行为零触碰（fb 现 4F 中 R13 唯一真差异）。
- indent filter 扩展面：fully-contained 非可编辑元素无条件纳入是否触碰其他行，下轮以 indent.html 语料全量 grep cE=false 行 + mt 语料 indent 键行对账（gate 批 fb/indent/mt/singles fresh 腿兜底）。

## 工法记录

fb R13 got/exp 与 accepted arrays：fbc3-fb.log :5001-last 窗（assert_in_array 3 形）；indent 双键 got/exp：evtr-census-indent.log（fresh 腿，1304P/25F，SW 门＝P>0 计数门，rc=2 为残余 Fail 传播 editing-gate-rc-bad-signal 先例）。
