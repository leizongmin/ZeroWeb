# 残面族归属普查（R5450 gate v4 腿日志，DC-4 簇序重普后续）

- 日期：2026-11-12
- 数据源：`unwrap-r5450-{exec1,exec2,exec3,ip,ihtml,ilb,indent,outdent,iul,iol,mt}.log`（gate v4 各腿，@dbfc027c9 重锚后组合态）
- 方法：Fail 键逐条解析，按上下文（编辑器载体）/ 命令序列 / 场景容器三轴聚类；同场景 styleWithcss 双态键计一次缺陷（计数仍按键）。

## 总残序（R5450 各腿 Fail 键合计 ~880）

exec1 **474** > ip **206** > ihtml **119** > ilb 33 > indent 27 > outdent 9 > iul 4 > iol 2 > mt 2 > exec2 1 > exec3 1。exec1 独占 54%。

## exec1 = exec-command-with-text-editor.tentative（474F，editing/other）

单一 tentative 文件族，text-control 编辑器面（非 contenteditable 写面）：

- 页分布：`?type=password` 154 / `?type=text` 154 / `?type=textarea` 166（含 textarea-in-contenteditable 85 形）。
- 命令分布：delete/forwarddelete/inserttext 各 48、cut 30、inserthtml 24、paste/undo/redo/styleWithCSS/defaultParagraphSeparator 各 18、copy/insertparagraph 各 12。
- 断言分布（top）：`execCommand() should return false` 164、`The command should be enabled` 70、`input.target should be …` 58、`The command should be supported` 48、`<input>.value should be "a[]c"` 24、`input.inputType should be insertText` 18。
- 域归属判读：需实现/门禁 text-control 内 execCommand 语义（返回值门禁 + beforeinput/input 事件 inputType），实现域在 user_actions/shim 面而非 engine 写面；spec 为 tentative（未定稿）。**认领前需域决策**：值密度高（474 键 54% 残面）但为单文件 tentative 面。
- exec2/exec3 各 1F（recursive-exec-command-calls / exec-command-never-throw-exceptions tentative），无簇。

## ip = insertparagraph.html（206F，editing/run）

宽散写面债，非单簇：

- 命令序列：dps-div 102 / dps-p 65 / swcss×dps 变体 24 / **dps queryCommandValue before/after 断言 8**（独立小子族）/ 其他 7。
- 容器直方图（场景引串首标签）：div 69 / p 47 / span 27 / ul 11 / table 8 / plaintext 7 / h3 7 / h1 6 / b 6 / blockquote 5 / a 4 / UNPARSED 8。
- 判读：div+p 容器 116 键（56%）为最大子族——insertParagraph 在普通块容器内的分拆/继承语义；table 跨界（`fo[o<table><tr><td>b]ar` 形）8 键独立小族。

## ihtml = inserthtml.html（119F，editing/run）

宽散：容器 span 25 / plaintext 24 / p 24 / ol 14 / div 12 / ul 6 / li 4 / font 4；inserthtml 实参形态 101 键；swcss 拆分 40/79。无单一主导族——inserthtml 序列化/样式携带面债。

## 下一刀选项（按簇序，域归属先决）

1. **exec1 道（474）**：text-control execCommand tentative 面——值最大且单文件内聚，但需先定域决策（tentative spec 投入意愿 + user_actions/shim 实现域）。
2. **ip 道（206）**：editing/run 写面正域，div/p 容器子族 116 起刀；与 apply 面嵌套簇刀（在飞）不同命令域，无碰撞。
3. **ihtml 道（119）**：宽散，宜随 ip 刀顺带普查归并，不宜单独开刀。
