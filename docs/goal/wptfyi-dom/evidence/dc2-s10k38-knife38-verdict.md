# dc2-s10k38 刀㊳ 判读（2026-11-26）

**提交** `6ac09dd16`（rebase 于 df2db2f4f docs-only 之上，携入零码面＝判读域不受扰）；刀面三文件纯 js（part02 +24/−1、part03 +30/−7、part04 +24/−0，`--numstat`）。engine 纯 js 刀免 clippy（刀㉟/刀㊲ 先例）。

## 刀面与根因

1. **part03 `_zwMakeSvgHref` 写臂**（svg-script-set-href 7F → 0）：
   - href.baseVal 串赋值原值透传，不预 String 化——预化把 TT 实例变 plain 串被 setAttributeNS 门拒＝赋值静默不持久。
   - 门抛 TypeError 传播出 setter（旧 catch 全吞＝串赋值静默不写，且违规事件照常经 `_zwTtViolation` 的 setTimeout 任务入队、落进后续测试收集窗＝non-attached setAttribute 页「2 violations」假象，实为泄漏非独立 bug）。
2. **part03 SVGAnimatedString `baseVal` 反射写 setter**（className.baseVal 4F）：旧 getter-only FIXME 账；写走元素 setAttribute('class') 全管线（class 非属性 sink，part14 表无条目）；animVal 维持只读（spec SVGAnimatedString.animVal readonly）。
3. **part02 `_zwParseEl` innerText 快照 getter**（block-string 3F＋DOMParser-parseFromString 1F）：解析元素缺 innerText＝`doc.body.innerText` undefined；spec 未渲染元素 innerText 与 textContent 同值，解析文档永不渲染恒走回落。
4. **part02 parseFromString WebIDL DOMString 强转**（DOMParser-parseFromString 1F）：`null→''`（刀③b② 保守承）改 `String(str)`＝null→"null"；domparsing grep 无 null→'' 依赖面，A/B 无漂移佐证。
5. **part04 R127 sel-sel replaceChild 插入段接线**（Node-replaceChild 1F＋Node-childNodes-cache-2 1F 红利）：
   - 根因三段交互（探针 k38j preMerge `arm=incr res=[]` 实证）：①adopt record 把 newChild 记 removed（同父 replace 的 spec 首段）；②replace record 的 un-remove 把 newChild 摘出 removedSet 并触发 T84 `_oc.ri` 水位回退（removed 数组缩长）；下一次 overlay 增量臂按回退水位把 removed[0]=oldChild 重滤；而 added 里的 newChild 因静态页面 wrapper 无 `__zwHandle` 亦无 `_zwSelPendingParent`，并入臂 `if (!nd || !link) continue` 恒跳过——净果视图 []（`a.firstChild===null`，R127 分支从未做插入段 wire）。
   - 修复：补 R182 先例（insertAdjacentElement sel 分支）反链 `_zwSelPendingParent = { parentSel, nextSibling, prevSibling }`；位次锚取 old 摘除前上下文（`_r127Prev/_r127Next`），next 锚与 newChild 重合（同父相邻形 [b,c]）时回落 newChild 自身摘除前 next（`_r127NewNext`，[b,c,d] 形锚 d 保序）。仅 sel 父接线，handle 父维持旧行为。

## 验证

- **solo 四页全绿**：trusted-types-svg-script-set-href 18P/0F RC=0；block-string-assignment-to-DOMParser-parseFromString 5P/0F RC=0；DOMParser-parseFromString innerText 双面绿；dom/nodes/Node-replaceChild 29P/0F（基线 28P/1F）。
- **电池四腿零回归**（字节绑定 `git diff | md5sum` = `40a197119f1f1c6d5245470069cd2bf4` 断言，REAPPLY-MD5-OK 闭合）：san 65 恒／dp 222 恒／wc 940 恒（多重集恒等零漂移）；dom base 376→374＝GONE 2 NEW 0（`Node-replaceChild` mutation events＋`Node-childNodes-cache-2` caching bug 同根红利）。
- **TT 同树 A/B**（checkout→基线→re-apply→md5 断言）：Fail 87→74（真实页 79→67＝**−12**；探针页 8→7＝zz-knife3b2-probe p16 随 edit ③/④ 翻面，合成页不计累计）；Timeout 20=20 恒；多重集 comm GONE 13 NEW 0。
- **存量面 A/B 排除**：ParentNode-replaceChildren 2 Timeout（FILTER 加宽才暴露）pre/post 恒等＝存量非本刀回归；DOMParser-parseFromString-regression 1F pre/post 恒等＝存量。
- **归因纪律**：GONE 13 行中真实语料 12 行计累计；`zz-knife3b2-probe.html` 1 行单列不计。

## 账目

- 累计 84,213 + **14**（TT 12＋dom 2）＝**84,227**（距 ≥99,947 差 **15,720**）。
- TT residual 池 87→**74**（真实 67＋探针 7）＋Timeout 20 页行不变；svg-script-set-href 7／block-string 3／DOMParser-parseFromString 2 键出池。
- script-enforcement-008/009（各 3F）本刀诊断：语料需 module 执行／importmap registry／动态 import()——page-script 管线三项全缺（script-sandbox 的 import_map.rs/es_module.rs 服务独立沙箱非 DOM 页），**非残刀，须新子系统切片**，证据 script-enforcement-008 语料结构（default policy 按 classic/module/importmap 类型变换脚本源＋CSP meta 哈希断言）。

## 下一步输入

残池最大簇：script-enforcement-004 4F／policy-without-return-value 4F／script-enforcement-006/007 各 3F／block-text-node-insertion-into-script-element 3F；module/importmap 6F 需切片级立项。电池工件 `/tmp/s10k38/`（k38-*.log 四腿＋tt-base/tt-post＋判读 lines）。
