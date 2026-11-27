# s10k67 刀55：iframe R255 子文档 body 前段 head 提取（headEl 视图）——Range-insertNode/surroundContents 15,x 92F 清零＋clone/delete/extractContents 8F 同族红利

日期：2026-11-28（chain67 链窗 20:48:38Z→21:08:30Z）。刀体 commit `cdc0d1a33`（单文件 `crates/engine/src/js_dom_shim/part05.js`，+19；发射 bin `74db1f231b60`＝提交域字节，链后 md5 恒等复核）。刀号：**刀55**（十进制续编，承刀54）。

## 刀位与基线

控制面挂账：**下一主池＝dom/ranges 289F 簇归因分刀**（chain66b 域 F 构成：ranges 289＋traversal 29＋nodes 24＋events 11＋historical 4＋html 2＋window 2＋lists 1）。F 页分布勘验：`Range-insertNode.html` 46F＋`Range-surroundContents.html` 46F（最大非 tentative 簇）＋OpaqueRange tentative 族 189F＋TreeWalker 29F。

## 根因（链式探针，/tmp/s10k67-probe1..7 日志）

1. **失败形态解壳**：46F/页全部集中在单一 range `[document.head, 1, document.head, 1]`（testRanges[15]，common.js 共享；22-23 node 变体 × position/DOM 双断言）。raw log 细看：失败断言体＝「Unexpected exception thrown when setting up Range for actual insertNode()」expected null but got **IndexSizeError: The given offset is out of bounds**——setup 阶段 `restoreIframe` 后 `run()` 里 `setStart(document.head, 1)` 即抛，resulting DOM/position 只是 step 首断言的统一壳。head childNodes.length = 0（offset 1 越界）。
2. **探针臂链**（zz-head-restore.html 六臂：初始 head／cloneNode 本体／跨文档入 referenceDoc／strip＋回挂／setStart／insertNode）：首跑全 Fail——**探针假象陷阱**：探针页初放 `dom/`（fixture 在 `dom/ranges/`）→ 相对 src `Range-test-iframe.html` fetch **404 空 text/plain** → shim S184 text/plain 壳臂包 `<html><head></head><body><pre></pre></body></html>`（P1c `_zwMarkup` 直读＋P1d XHR `404|text/plain|` 定谳）→「head 空」是壳伪象非引擎真相。移探针到 `dom/ranges/` 后真形显影。
3. **真形**：P1 初始 iframe head＝**0 子**（head 元素非 null）；P1b `_r159HeadHtml` 槽 absent。markup（P1c）＝完整 fixture（doctype＋title＋link＋meta＋`<body onload=run()>`＋script），200 text/html。
4. **引擎域定位**：`_zwMakeIframeDoc`（part05:2545）两分支——R159（显式 `<html>` 对）剥 head 段存 `doc._r159HeadHtml` → T 切片（part05:3197，editing goal d34）独立解析入 headEl（`_zwMBuildBodyTree(inner,'',false)` scripting=false，无双执行——iframe 脚本提取链独立于 DOM 建树）；**R255 分支（无显式 html 对、有 `<body>` 标签）bodyInner 只取 `<body>` 后段，body 前段（title/link/meta）整段丢弃、`_r159HeadHtml` 不设** → headEl 恒空。Range-test-iframe.html 正是 R255 形。真浏览器解析合成 head＝[#text,TITLE,#text,LINK,#text,META,#text]（P1 刀后实跑 7 子恰同形）。
5. **机制面就绪佐证**：insertNode 元素容器尾段 R219 end-sync 注释已点名 15,x「endOffset expected 2 got 1」簇已修；headEl s408 三件套（appendChild/removeChild/insertBefore＋属性面）在案；`setStart` 对 headEl mock childNodes 校验正常。唯一缺口＝headEl 无内容。

## 修复（part05.js，R255 有 `<body>` 分支）

body 前段（`_r255BodyOpen.index` 之前）剥 `<!doctype>`/`<html>`/`</html>`/`<head>`/`</head>` 包装（`</?head\b[^>]*>` 合并剥——`<header>` 经 `\b` 不误伤），余段非空即存 `doc._r159HeadHtml = '<head>'+余段+'</head>'`，复用下方既有 T 切片 headEl 独立解析面，零新增机制。纯空白前段（minimal srcdoc 常形）不设槽＝零变化。FIXME 记档：body 前非 head 专属内容真浏览器会隐式闭合 head 归 body，此处并入 headEl（旧版恒丢，语料无此形）。

## 判读（chain67，20:48:38Z→21:08:30Z，pid 2864904，四腿）

单页先行：Range-insertNode **1840P/0F**、Range-surroundContents **1840P/0F**（锚各 46F）。

| 臂 | 锚（chain66b） | 实跑 | 归因 |
|---|---|---|---|
| san 全臂 | 773P/5F/0T | **773P/5F/0T** | F 集逐字节恒等＝刀零 san 影响 |
| dom 臂 | 57489P/361F/12T | **57542P/261F/12T** | **F 页集净减 5 页＝−100F、零新增 F 页**；存活 56 页 F 行多重集逐字节恒等；T 集恒等；ΔP+53 vs ΔF−100 的 −47 差＝Range-mutations dataChange/replaceData 截断噪声带（已知族，F/T 集恒等是门） |
| tt 臂 | 2290P/14F/18T | **2290P/14F/18T** | F/T 集逐字节恒等（真实池 0F 维持） |
| ref 臂 | 725/725 | **725/725**（Layout 521/521＋Text 204/204） | 恒等 |

清零五页全绿实证：insertNode 1840P＋surroundContents 1840P＋cloneContents 187P＋deleteContents 125P＋extractContents 187P（同族红利 8F＝clone 3＋delete 2＋extract 3——common.js 共享 testRanges 的 head 端点同根因）。腿 rc 留痕：san 2／dom 2／tt 2／ref 0（rc 非门信号）；ref 腿 9 秒形态与 chain66b 逐位一致（20:26:31→20:26:40 同款）＝既有 runner 常态。发射断言 `74db1f231b60`+head `cdc0d1a33` 过＋链后 md5 恒等。语料页即常驻回归。

## 残相挂账

- **insertNode 元素容器 collapsed 形 startOffset 不前移**（engine R219 只同步 end＝(parent, newOffset)；common.js myInsertNode 同构只 setEnd）——spec 步「set range's start to (parent, newOffset)」Chromium 形 start=end=(head,2)，双边自洽恒 (head,1)/(head,2)＝语料无钉面；若未来语料换 Chromium 基线绝对真值需同步 start。
- R255 无 `<body>` 标签臂（else 分支）head 段仍剥丢不设槽（同族残面，语料未触）。
- OpaqueRange tentative 族 189F＝Highlights/OpaqueRange API 全未实现（shim 0 命中），需整面特性刀位评估（geometry/highlightsFromPoint 疑涉渲染集成）。
- dom 残池刀后构成（261F）：ranges/tentative 189＋traversal 29＋nodes 24＋events 11＋historical 3＋window-extends-event-target 2＋html 2＋lists 1。

census：+100 → **84,492，距 DC-3 校准线 ≥94,093＝9,601**（原线 ≥99,947＝15,455）。
