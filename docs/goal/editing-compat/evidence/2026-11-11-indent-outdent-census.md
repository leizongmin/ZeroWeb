# indent/outdent 权威残键集普查（census v1，2026-11-11）

## 目的与处方

控制面 v1.94 ⑯ 处方：「下一簇＝indent 27F 普查（fbq 后 post 日志散佚，须现树 solo 重跑取权威残键集）」。fbq 时代 1302P/27F（2026-11-07 证据）的 log 实体散佚（`.acceptance/dps/batch-fbq-indent-post.log` 为 fetch 超时死体），且 v7 判读域（13535525b）之后 runtime 面已入站 24 文件（s399 part01.js +153／position-area／color／corpus FILTER pipe），旧数不具权威性。同批补测 outdent（同族，indent 刀将触共享路径）。

## 批程

`batch-ind-census-v1.sh`（.acceptance/dps/）：零源码变更测量批，无锚门/judge 基线（本跑产出即新基线）；域绑定＝发射 HEAD + delta 入档（ind-census-domain-delta.txt，vs 13535525b 24 文件）＋双腿间 HEAD 复核。ensure_solo 逐字节复用 v7（foreign 租户等待）。发射 11:56:34，ALL-DONE 12:00:37 为 relay 合并口径：attempt-1（owner 发射）indent 腿完成后会话拆除杀批 → writer 轮接力补走 outdent 腿（12:00:37 完走）→ owner attempt-2 重射 12:03:33 双腿绿双跑恒一复证；现存产物 ind-c1-* 为 attempt-2 生成，relay 链存 .attempt1。

- head **3a528a13f**（=origin/main，origin/main..HEAD 空）；bin md5 **570d87f14f194853d9e5db97352f1c2d**
- 腿：`make testharness-editing-run-one FILTER=<page> TIME_LIMIT=1800`（make 既有 fetch/disk-guard/test-guard/runner-release 链），ZW_SCRIPT_TIMEOUT_MS=300000
- rc=2 为 runner 残余 Fail 退出码传播（editing-gate-rc-bad-signal 先例），门＝P>0＋计数

## 结果

| 页 | P | F | T | 对账 |
|---|---|---|---|---|
| indent.html | 1302 | **27** | 0 | 与 2026-11-07 fbq 旧测 1302P/27F **逐数恒等**——runtime 24 文件入站零移数，旧分族证据链继续有效 |
| outdent.html | 2377 | **173** | 0 | 无旧权威数可比；**editing/run 现存最大残簇**（此前控制面只记 indent 27F/img 23F，outdent 残量首次入账） |

## indent 27F 族级分类（全 compare innerHTML，apply 面）

八微族（残键清单 ind-c1-indent-fails.txt，27 行全量在档）：

| 族 | 数 | 代表形 |
|---|---|---|
| ol 嵌套列表 indent | 4 | `<ol><li>foo<ol><li>bar</ol>[baz]</ol>`——第三 li 缩进与既有嵌套 ol 的交互 |
| blockquote 相邻/聚合 | 10 | `[foo]<blockquote>bar</blockquote>`、`<blockquote style="margin-right: 0"><p>foo[bar</p><p>b]az</p></blockquote>`（块内跨段 ×4＋相邻 ×2＋sandwich `<blockquote>foo</blockquote>[bar]<blockquote>baz</blockquote>` ×1＋ws 邻接 `</blockquote> <p>[bar]` 形 ×3——v1 收口核验勘误补行 11/17/18/19，枚举总和 27 对账） |
| br 边界 | 7 | `foo[bar]<br>baz`、`foobar<br>[ba]z`、`<ul><li>a<br>{<br>}</li>`＋collapsed-at-br ×3（`<p>{}<br>foo`） |
| table tbody 跨行 | 1 | `<tbody data-start=0 data-end=1>` 部分跨行（apply 面，tq 刀 qV 读面同页不同面） |
| rtl 混排 div | 1 | `<div><p>Foo[barbaz<p dir=rtl>…]` 跨两段含 rtl |
| swcss=true margin-div | 2 | `<p>[abc]</p>`、cE=false 嵌套 editable 内 `[abc]` |
| cE=false span 全选 | 2 | `{ <span contenteditable=false>A</span> ; … }`（swcss true/false 双形）——**与 fb 页 R13 同能力族**（wrap 不可编辑边界） |

#37 归因定性：27 行全为 compare innerHTML（确定性 apply 结果），无录音矛盾类表面候选；页无 variant 窗（单窗整跑），窗界伪影类不适用。got/exp 明细可得（assert_in_array 行），样例：`foo[bar]<br>baz` got `<blockquote>foobar<br></blockquote>baz` vs 期望 `<blockquote>foobar</blockquote>baz`——选区尾后 br 被包进块内，Chrome 丢弃（真差异）。

## outdent 173F 粗查（族级精分类留下一轮）

- 命令形态：plain ×49 / swcss=false ~116 / swcss=true ×8
- 语境：**list（ol/ul）115/173 主导**（嵌套列表 outdent 边界）；blockquote 64；br 7；table 2；rtl 0；cE=false 0
- 实现锚点：命令表 `part06.js:2807`（`indent: 'formatIndent'`），apply 臂 `part06.js:5343-5352`（indentNodes 忠实移植），helper 分布 part01e

## 判读与下一簇次序

1. apply 面 compare 行为确定性高，两页残键均为真差异候选（无 #37 伪影嫌疑行）；逐族 got/exp 归因后开刀。
2. 按 DC-4 簇序降序：**outdent 173F（list 语境 115 主导）升为最大簇**；indent 27F 八微族中 cE=false ×2 与 fb R13 同族（可搭 R13 能力刀）。
3. 语料/产物：ind-c1-indent.log / ind-c1-outdent.log + 各 -fails.txt / -timeouts.txt / ind-census-domain-delta.txt（.acceptance/dps/）。
