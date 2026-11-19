# 2026-11-06 qState 选区恢复耦合刀 sr1——出口收钳同位孪生解析，iol/iul :38 形各 1 行翻绿

## 债与判读

P10 残债「选区恢复耦合」亚族：action 后选区停列表外致 qState-after 读 false
（期望 true）。本刀先以临时插桩（`__zwDbgListSel` 门控，随判读撤除）取
`_zwListToggle` 五时点 trace（E0 入口 / E1 blockExtend 后 / E2 toConvert 后 /
E3-E4 出口收钳前后），诊断页 zz-probe-qstate4.html（wpt-data 本地语料，
gitignored）三形同跑：

1. **:38 形** `[foo<ul><li>bar]</ul>baz`（real 债行）塌缩链定位——
   E2 时选区已塌到 `text"bar]"@3..@3`（retag 助手 `_zwListRetagPreservingSelection`
   主 range 壳外端点无 path 快照，引擎 remove 钩子按纯删除收缩未获补偿）→
   Chrome-delta 收钳（retag 臂 828-852）不触发（塌缩后两端点皆壳内）→
   enable 主循环 `_zwJusWrapViaHtml` innerHTML 整体重写：旧壳节点 detached
   （rfInDoc=false 实证）、端点文本身份灭失，ViaHtml 映射失败落 fallback
   `(host, childNodes.length)`＝DIV@2 → **出口收钳三分支全 miss**（容器 host
   非壳；retaggedFinal 已 detached）→ 读面 false。
2. **:43 形** `foo<ul><li>[bar</ul>baz]`（吸收形，回归守卫）：端点重锚落壳边界
   （OL@2）→ 壳分支收钳生效（LI@0..LI@1）→ 读 true 恒绿。**retag 臂
   Chrome-delta 收钳是 :43 的承重件，不可移除**（推翻本席前轮「移除 828-852」
   设计——插桩实证该钳对 :38 根本不触发，移除只伤 :43）。
3. **:393 形** `<ul><li>foo<ul><li>[bar]<li>baz</ul><li>quz</ul>`（对照）：exec
   实跑窗口内全管线选区不脱壳，读 true、html 与语料 expected 逐字合——语料
   :393 行 fail 落 ?1001-last 而 exec 行在 ?1-1000，**切片跨窗伪影坐实**（⑪
   doctrine 签名 A），自 P10 残债 7 行中除名：real 可修仅 :38 形 iol/iul 镜像
   2 行。

## 刀（part01e.js `_zwListToggle` 出口收钳）

- **shellHome 树位锚**：toConvert 收尾、wrap 重写臂灭失身份前，记换壳列表
  `(parentNode, nodeIndex, nodeName)`。
- **同位重建孪生解析 `shellSuccessor`**：出口收钳第三分支（原为
  `retaggedFinal` in-doc 才可用）扩展——rf detached 时按 shellHome 解析同父
  邻近同名新壳为收钳目标。位次带漂移容差：重写臂吸收壳前内容会平移重建
  孪生位次（探针实证记录 OL@1 → 重建后 OL@0），自记录位次向两侧邻近扫描
  同名首中者；父级失配（非 in-doc）放弃维持 marker/path 恢复结果。
- 收钳动作不变：钳入壳最深首/尾端（li 边界），读面端点最近列表祖先＝本
  命令型 → true。

## 门禁（sr1 批）

- probe4 终跑三形全绿：:38 形 `LI@0..LI@1` qSA=true；:393 形 qSA=true；
  :43 形 qSA=true（路径未变）。
- 基线零漂移复核：iol fresh base 1720P/41F、iul 2027P/38F，与 P10 账面逐数
  恒一（iul 基线先行于源码编辑起跑，二进制前像完整性保住）。
- 腿：iol 1721P/40F（F 41→40）、iul 2028P/37F（F 38→37）。
- judge（judge-sr1.txt 行级元组，vs 本席 fresh base）：**F2P=2 / P2F=0 /
  newF=0，VERDICT OK**。翻绿逐名＝:38 形两镜像：iol
  `[foo<ul><li>bar]</ul>baz` qState-after、iul `[foo<ol><li>bar]</ol>baz`
  qState-after，均 ?1-1000 窗。
- enginelib `make guarded-test`：**3,758P/5F**，红集与 P10 账面五行恒一
  （p56_adopted / p99s_rule_style / r3276 / r2810 / r2809，stylesheet 族已知
  红，非 editing 面）。

## 残债更新

结构 qState 残债 7 行 → 4 行（:393 形伪影除名 + :38 形 ×2 翻绿）：

1. :478 wholesale 同型嵌套 ×4（iol/iul 镜像各 2）：期望 false 的命令史态
   （'none' range）纯读不可达，维持 P10 定谳不追。
2. 独立债不动：iul dps-div straddle 行（qState+qInd after 双行，②）、
   empty-body write-face ×3（③）。
