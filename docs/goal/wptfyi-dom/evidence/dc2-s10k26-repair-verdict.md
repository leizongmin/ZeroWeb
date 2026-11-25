# DC-2 切片 10 返修收账——meta 入树 TT enforcement 置脏＋R522c replaceChild 原序恢复（TT 131→127／DOM 368→364＝两页环境敏感红面 8F 全清）

**日期** 2026-11-26　**判定** PASS（刀域 8F→0F 全清＝solo/电池双证；五腿键集净 +8＝GONE 8/NEW 0；WC/DP/SAN SET-IDENTICAL；Timeout 键集逐腿恒等）

## 提交与判读绑定

| 提交 | 内容 | 判读二进制 | 电池 |
|---|---|---|---|
| 返修 `35af9fc09`〔谱系 35af9fc09＝发射锚→push 前 pull --rebase 携入 html-semantics s765 docs＝`23e9e6a86` 推送终态；part04 blob 恒等 3c4b296e02＋patch-id 恒等验证〕（part04 appendChild/insertBefore META 置脏＋R522c 原序恢复，49+/10-） | b3fab8330 两处回归修复 | `654ab97fa842fdf802d2119e69d746df` | v11 |

v11 判读时 HEAD＝发射锚 35af9fc09（发射器 EXPECT_HEAD 断言即此值）；判读二进制构建于同 part04 字节（blob 3c4b296e02），rebase 重写不触判读绑定。

发射器 `.acceptance/dps/s10k26-battery-v11.sh`（EXPECT_MD5+EXPECT_HEAD 双断言、flock 单例、五腿 test-guard 包裹、setsid detached）；18:02:27–18:34:33Z（32min，WC 5m44s/DP 13s/SAN 17s/TT 16m40s/DOM 9m12s）。

## 根因链（两处回归均出自 b3fab8330＝对端 workers OOM 修复笔）

1. **HTMLElement-generic「TT enabled」script.src 4F**：b3fab8330 的 `_zwTtEnforced` 微任务域 memo 把 enforcement 读取收窄到「解析期首查＋HTML sink 写面出口置脏」两臂；`document.head.appendChild(meta)`（CSP3 meta 动态注入，语料 HTMLElement-generic.html:82）是纯 DOM 变更非 HTML sink，不经任何既有置脏臂 → 同同步块内 memo 缓存旧值 false → plain string 赋 script.src 不抛 TypeError → 4 Fail（Trusted/String × connected/non-connected）。
2. **Range-mutations-replaceChild 4F**：b3fab8330 顺带重写 R522c（刀⑳ script-enforcement-004 Node.replaceChild 面的 handle 换文本子分支），把既有「先移除后插入＋_r522ref 预取 oldChild.nextSibling＋_r522OldR87 判别补 adjustRemove」改为「先插后移」且丢弃预取/判别——live-range 边界序偏离语料 testReplaceChild 期望模型 remove(old)→remove(new)→insert(new)（先插后移把边界推成 insert 后位次 net +1＝`Wrong end offset expected 0 but got 2`）。
3. **返修**：append/insertBefore 两臂 META 入树即调 `__zwTtCspDirty()`（nodeType===1＋typeof 守卫早出，spec CSP3 meta 要素每次入 head/document 即重新处理，https://www.w3.org/TR/CSP3/#meta-element ）；R522c 整段恢复 b3fab8330^ 原形（与 `/tmp/part04-pre-b33.js` 快照 diff 恒等实证，仅注记补回归说明），https://dom.spec.whatwg.org/#dom-node-replacechild 。

## v11 判读（锚＝k25 v10 终态 TT F 131 / WC F 1116 / DP F 222 / SAN F 50 / DOM F 368；F+T 键集 TT 196 / WC 1136 / DP 223 / SAN 51 / DOM 380）

| 腿 | Fail | GONE/NEW | Timeout 键集 |
|---|---|---|---|
| TT | **131→127** | **GONE 4＝HTMLElement-generic 全部 / NEW 0** | 65=65 键集恒 |
| WC | 1116→1116 | 0/0 | 恒 |
| DP | 222→222 | 0/0 | 恒 |
| SAN | 50→50 | 0/0 | 恒 |
| DOM | **368→364** | **GONE 4＝Range-mutations-replaceChild 四面 / NEW 0** | 恒（12=12） |

**净 +8＝累计 83,821**（距 ≥99,947 差 16,126）。v10 记账时挂账的 8 键环境敏感红面全额清偿；s10k25 判读注记「返修刀 part04 meta 脏位与 replaceChild 页无交集」的预判被返修刀实际面（R522c 同笔恢复）覆盖。

## 验证链

1. **solo 三连（release `654ab97f`）**：HTMLElement-generic.html **81P/0F**（v10 电池 4F）；Range-mutations-replaceChild.html **60P/0F**（v10 solo 56P/4F 两连恒）；script-enforcement-004.html 恒 4F（normalize/moveBefore/cloneNode/Range.cloneContents 四钉，v9=v10=solo 同四钉＝预存非 R522c 恢复所致，刀⑳原目标页其余 27P 不动）。
2. **电池 v11**：上表五腿，GONE/NEW 逐键 comm 对账（LC_ALL=C awk 键提取，v10 锚 /tmp/k25-*-v10.keys）。
3. **fmt** 零 diff；**guarded-clippy** workspace 0 warning（50s 增量）。
4. **make test workspace 腿 2F＝预存非本刀**（`r5309_text_wrap_nowrap_layout` nowrap/mis-ref 两钉：ch 单位 320px≠160px，layout/style-system 域）：单文件隔离实证——part04.js checkout 至 HEAD~1（返修前）重构建重跑＝同 2F，本刀排除。破坏窗＝k25 门禁绿（0fe7a0b2c）之后的合流窗：PR #327 perf/t112-layout-probe merge（ea1dafc63）／k129 d06dbbca6／k130 8480d82f8 均触 engine/style-system/layout（渲染流域）。make test 于 workspace 腿中止，renderer/browser/GPU/QuickJS 各腿本轮未跑（后续轮补跑确认）。

## 残留挂账

- make test 剩余腿（renderer/browser/GPU/QuickJS）本轮中止未跑；r5309 两钉红面归渲染流域（k25 门禁绿→合流窗引入），待对端修复后补跑全量门禁。
- TT 残 127F 按簇（前账：default-policy-report-only 9F／script-enforcement-008/009 各 5F／createHTMLDocument 残 5F 等）；WC adopted-callback 66F。
- appendChild/insertBefore 每次元素插入新增两次小串分配（nodeType 守卫早出非元素）；bench-gate 未跑（js_dom_shim JS 不在 bench 面），TT 腿墙钟 16m40s vs v10 16m46s 无回归信号。若未来 bench 覆盖 shim JS 变更路径，回看此处。
