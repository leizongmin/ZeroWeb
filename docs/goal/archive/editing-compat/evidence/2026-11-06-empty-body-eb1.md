# 2026-11-06 eb1 空 context 建列表臂——empty-body write-face 收口（iol 41→36 / iul 38→33）+ iul dps-div straddle 伪影定谳

## ① iul dps-div straddle 行：签名 A 伪影定谳（零源码修改）

残债行：`<ul><li>foo<ul><li>b[ar</ul><li>b]az</ul>` 的 qInd+qState after 双行
（?1001-2000，基线红）。⑪ doctrine 三重门判定：

1. **exec 出窗**：该 case 的 16 断言块里 exec 子测试全局编号 #990-991 < 1001，
   落 ?1-1000 窗；?1001-2000 窗只重做 setup 不执行 exec → after 读数=未修改树，
   对 Chrome post-action 录音必红（签名 A straddle）。
2. **行数拼回**：全案 16 断言 = ?1-1000 窗 11 行 + ?1001-2000 窗 5 行，重装无缺。
3. **fs-win 变体**：单窗 `?985-1010`（exec 与 after 读同窗）复跑，qInd+qState
   after 双行全 PASS（/tmp/zw-p21/iulwin-985-1010.log）。窗内邻案（?985-1010
   覆盖外推 case）的 compare 行红为窗口切位伪影（?1001-2000 基线行 Pass 在位
   复核），非引擎债。

定谳：伪影除名，iol/iul 语料首次纳入 ⑪ 伪影清单（此前只审计值面五语料）。

## ② eb1 刀：`_zwListToggle` enable 空 context 建列表臂（part01e.js +40）

empty-body 语料形（Chrome 录制口径，语料 accepted 终树）两臂：

- **①空白 div 形** `<div>{}</div>` / `<div>{}<br></div>`：主循环换型臂对空 div
  也整壳替换（探针实测 `<ul><li>…</li></ul>` 丢 div 壳）。改臂：nodeList 恰 1
  且为空 div / 仅 br 占位 div 时，清空 div（消费 br）、div 内新建 `utag` 壳+空
  li，caret 入 li，div 壳保留。
- **②裸 host 形** `{}`（host 空）：nodeList 空，主循环 no-op（探针实测纯文本落
  host）。Chrome 在 caret 位新建列表壳。锚点两轮修正：
  - 初版锚 `newRange.startContainer` + `_zwJusIsEditingHost || _zwJusIsEditable`
    守卫——**fail**：空 host 下 blockExtend('list-wide') 爬出 host 落祖先容器
    （插桩实证 sc=祖先 DIV、ce=null，双谓词全 false 不落臂）。
  - 二版改锚入口 `range`（blockExtend 前）caret 端点但守卫仍含 isEditable——
    **11 P2F 回归**（iol 全 compare+qState）：嵌套形 caret 落 li 内文本，父元素
    li 可编辑 → 误落臂在 li 里植空列表，且 `nodeList=[]` 灭掉换型主循环。
  - **终版**：入口 range.startContainer 须 nodeType===1 且 **`_zwJusIsEditingHost`
    本尊**（host 外一律不落臂）；空白 p 形仍走换型臂（语料已绿不动）。
- caret 经活动 range 直写（disable 出口同款，setStart/setEnd 即选区生效）。

### 探针（wpt-data 本地 zz-probe-eb1.html，gitignored，9/9 OK）

7 空形（bare/div/divbr/p/pbr × iul + bare/div × iol，语料同款双 exec 列表命令 +
inserttext 'abc'）终树逐字合 accepted（`<ul><li>abc</li></ul>` /
`<div><ul><li>abc</li></ul></div>` 等）+ 2 嵌套守卫形（`<ul><li>[foo]<ul>…` 换型
单 exec，arm 不得落臂）终树 `<ol><li>foo<ul><li>bar</li></ul>baz</li></ol>` 恒
绿。探针自带 setupDiv 简版标记解析（'[' ']' 倒序删除 + 同节点 offset 补偿）。

### 门禁（eb1 批）

- 腿：iol **1720P/41F → 1725P/36F**、iul **2027P/38F → 2032P/33F**。
- judge（judge-sr1.py POST_PREFIX=eb1，vs 2026-11-05 fresh base，含 sr1 已翻行）：
  **F2P=10 / P2F=0 / newF=0，VERDICT OK**。逐 corpus 5+5 = sr1 :38 形 qState-after
  ×1 + eb1 write-face ×4（bare `{}` compare + bare `{}` qState-after + `<div>{}`
  compare + `<div>{}<br>` compare，iol/iul 镜像）。
- enginelib `make guarded-test`：**3,759P/5F**，红集五行恒一（p56_adopted /
  p99s_rule_style / r3276 / r2810 / r2809，stylesheet 族已知红）。
- 勘误注记：初版守卫批次 judge 曾写 regress-eb1-*.txt（11 行）；终版 OK 判读不
  覆盖该文件（judge 仅在 pf/nf>0 时写）——/tmp 遗留文件已清，后席勿误读。

## 残债更新（② 空 context 族收口后）

empty 族 write-face 全绿；**读面残债独立挂账**：

1. **value-face**：qV before/after 录 `"false"`/`"true"` 字符串的 13 案（iol
   `{}`+`<div>{}` 残余 + iul `{}`+`<div>{}<br>` 残余 + 3 非空 span-style 案）——
   判别子未知（非空壳已证非条件、swCSS=true 录 "" 已证非 swCSS 位）。
2. iul `<div>{}<br></div>` qState-after：读面红（compare 已绿），与 value-face
   同族挂账。
3. iul div-br qSa/qVa 行：case 块跨 #2000 不可达窗（straddle 邻族，记档不追）。
4. ③ :478 wholesale 同型嵌套 ×4：命令史态（'none' range）纯读不可达，P10 定谳
   不追不变。
