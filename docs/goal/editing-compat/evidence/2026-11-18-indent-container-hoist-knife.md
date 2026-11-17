# indent 容器整体升级刀（R17 B 族 4 键＋E 族 div 键）与残 9F 收敛（2026-11-18）

## 刀面（r16-indent.log 14F 中本刀 5 键）

- B 族 4 键（styled wrapper 再包裹方向）：`margin-right: 0" dir="ltr` ×2（L514/L524）＋webkit-indent-blockquote ×2（L719/L729）——got 新 bq 嵌既有壳容器 C 内，exp 新壳包 C 外（`<blockquote><blockquote style=…>` 双壳形）。
- E 族 div 键（L89）：`<div><p>foo<p>bar[dir=rtl]` 形——同根因容器形，exp `<blockquote><div>…</div></blockquote>`。
- 非刀面 9 键（C4/D3/E-表 L234/F1）本轮零扰动，failset SET 恒一实证。

## 根因链（TEMP 探针取证，判读后删除零残留）

1. spec 算法链：indent → block-extend range → do-while 求新边界（[execCommand spec](https://w3c.github.io/editing/execCommand.html#the-indent-command)）。容器 C 全部子代为可见块时，(C,0) 恰满足 block end point 定义（offset−1 子为可见块）、(C,len) 恰满足 block start point 定义（offset 子为可见块）→ do-while 两端都不动，newRange=(C,0)-(C,len) 滞留 C 内侧。
2. 默认档（非列表域）blockExtend 尾爬升为条件式（part01c.js `_zwJusBlockExtend` else-branch）：`startNode !== endNode` 且非祖先关系才爬——两侧滞留同一容器时守卫全部封死，逃逸失败。
3. node list `_zwJusGetContainedNodes(newRange)` 收 C 的全部子代 → wrap 以子代为界 → 新 bq 嵌 C 内（got 形）。
4. 探针实证：styled bq 形（L514）blockExtend 返回 (C,0)-(C,len)；div 形（L89）同形；列表域 li 臂两侧端点恒落 (ol,i)/(ol,i+1)（既有行为面，不入刀）。

## Chrome 录形判读（语料权威）

- 全覆盖形（B4＋L89）：C 的全部子代入选 → 录形新壳包 C 外。对应 spec 注记 "puts them at the outermost possible location"。
- 部分覆盖形：styled bq 内只选 p1（p2 残留）→ 录形仍嵌内（indent.html 现绿键 entry 1＝判别键）。两形分界＝全覆盖。
- spec 原文该面行为由 blockExtend 无条件尾爬升产生，但无条件爬升会越过宿主把宿主祖先链收进含域（宿主 div 入列 → 新壳外包宿主），与录音不符——Chrome 实为宿主天花板＋全覆盖判据，故收口在 node list 构建面而非 blockExtend 爬升面。
- 排除域：ol/ul/li（列表域 C 族）与 table/tbody/thead/tfoot/tr/td/th（表格逐 td 录形）不入升级。

## 刀体（part01g.js `_zwCeIndentApply`）

nodeList 构建后（li-vis 检查前）升级环：对每个成员取 `parentNode` 作容器 C，门＝可编辑元素 ∧ 非宿主（hostChk 自 `nodeList[0]` 上爬）∧ 非 list/table 家族 ∧ 有子代；fullCov 判据＝C 的 childNodes 全部在 nodeList（子代全在列＝段必然连续）；命中则以 C 替换其子代连续段（`splice` 至首子代位，`up` 回退重扫）。ksig 特征串 `fullCov`（release 二进制 3 处）。

## 验证

- part138 钉测 4 形（`test_indent_container_hoist_pins`）：styled-whole（L514 形 exp 双 bq 嵌外＋无嵌内形反断言）、div-whole（L89）、webkit-whole（L719）、part-cover 反断言（p1 单选保持嵌内）。1 passed。
- 全 lib **4164P/0F**（+1＝本钉测参与跑；`--skip test_get_computed_style_font_shorthand_r2761` 外域红留痕）。
- clippy/fmt：本流文件（part01g/part138/js_dom_bridge_tests.rs）零漂；`clippy -D warnings` 全仓 1 error 在 animation.rs:2044＝外域红（见下）。
- indent 语料 smoke（bin 65358d52）：**1320/9/0**，failset 差量恰 −5＝B4+E1，零新增红（comm 对 r16-indent.log 实证，r17-indent-newred.txt 空、resolved=5）。

## 门禁批 zw-r17 ALL-DONE（2026-11-18T03:35:26+0800）

八腿全绿 @084b6bf7d bin 65358d5204d2bd14f247920d223d9c36 全程恒一（wall ~62min 含 L0 test-guard build＋load_wait）：

- indent 1320/9/0（24s）翻绿恰刀面 5 键 B4+E1＋零新增红（comm 实证）＋resolved 细验 N1 webkit-indent-blockquote=2／N2 margin-right: 0=2／N3 div 形=1。
- fb 5043/3（946s）＋fs 2419/3（60s）＋img 962/0（12s）＋event 176/4（2s）＋ie 1476/1852/3（148s）fail/timeout 集全部 SET-IDENTICAL 对 r16 fresh 锚恒一。
- lib 4164P/0F pin-in 7/7（新增 container_hoist；--skip 外域 font 红留痕）。
- mt 10090/2/0 X37 双键恒一零暴露（multiplier2 终审，1833s）＝nodeList 构建面共享域（indent/list/ip）零扰动双哨兑现。

## 残 9F（r17-indent.log，四族）

| 族 | 键数 | 形 |
|----|------|----|
| C list 嵌套域 | 4 | `<ol><li>foo<ol>…[baz]</ol>` 等 |
| D p 成员邻接合并缺失 | 3 | `<bq><p>foo</bq> <p>[bar]` 等 |
| E-表 整表单枚 | 1 | `<table><tbody data-start=0 data-end=1>…` |
| F li 内尾 br | 1 | `<ul><li>a<br>{<br>}</li>…` |

## 外域红归因（非本流，挂账）

- `test_get_computed_style_font_shorthand_r2761`（lib）＋clippy animation.rs:2044（f64→f64 cast）：均 2d3e71d19（fonts 流）引入；本流改动 stash 后 clean HEAD 复红实证，`git log` 归因同源。
