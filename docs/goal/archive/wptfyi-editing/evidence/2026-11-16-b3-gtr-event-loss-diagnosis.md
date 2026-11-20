# B3 诊断轮：gtr beforeinput 908F 分解 + join 首触发事件丢失根因链（未落树）

日期：2026-11-16。范围：纯诊断 + 修复实验（shim 改动已回滚，零净码变）。

## 908F 真构成重分类（全语料 473P/2885F 态，探针 + 消息分类实证）

| 簇 | F | 形态 | 落点 |
|---|---|---|---|
| list-items 删除 | 592 | `<ul><li>list-item1{</li><li>list]-item2</li></ul>` 元素锚 range 选区 | `__zw_ce_delete_impl` 12026 sc 非 text silent return；**no-op 无树变**（O2 探针实证） |
| dl join | 152 | `<dl>…{</dl><dl>…]</dl>` 元素锚 | 同上（O3） |
| non-collapsed selection | 76 | 跨块 range 选区 | 同簇 + ④ 臂 defer |
| backspace/forwarddelete 主文件 | 90 | 混合：宿主起点 caret / src 非末子保守不并 / run 中段（br/img 前邻）/ 纯 div-p join | 多点（见刀清单） |
| got=2 双派 | 342 | 多案上下文 | **本轮修复实验直接暴露同机制嫌疑（下）** |

**事件丢失集群（探针 P15g 形纯 div-p-text join）在 908F 中占比极小**；此前 round-3 工作日志把它当主簇是误判。

## join 首触发事件丢失：根因链（探针 b3bi→b3k→b3l→b3m→b3n 全链）

现象：join 形（`<div><p>abc</p>[]def</div>`）**同页首触发** bi=[] input=0 但树变完成（abcdef）；二触起 bi=1 input=1。与 promise_test 序号无关（b3m M2 第 2 测 join 首触发仍丢），与 join 形首触发强相关。

定位数据（zz-probe-b3l/b3n）：

```
L1/N1 首触发: hook=1 disp=0 docCap=1 bi=[] input=0 after=abcdef
N1 补测:      querySelector(sel) 与 gEditor same=false sameNode=false（首跑未合流）
N2 二触发:    same=true sameNode=true（合流）→ bi=1 input=1
```

机制：首触发必走 `__zw_ce_caret_range` 世界同步回灌（10212 `!el.contains(锚)` → `el.innerHTML=_htmlSync` 幂等重建）+ 回植；此后 impl 世界内 querySelector/锚链上溯产物全落 query 家族代理，外部 listener 挂点（页面家族 gEditor 代理）在 impl 内不可达——派发沿 query 家族链冒泡（document capture 收 1 个 beforeinput）但宿主 listener 表零触发，且壳落位段跨家族引用抛异常丢 input（docCap=1 而 input=0）。二触起 contains 命中零开销跳过、家族合流 → 全通。

## 修复实验（4 刀递进，isolated 全兑现、语料面净 -2 → 回滚）

1. impl 内 re-query el：无效（家族身份由对象创建路径决定，不随树态）。
2. 派发目标换锚链上溯宿主（`_zwEvHost`）：isolated N1 bi=1 兑现；语料 P 473→471（-2），non-collapsed 双案 got=2。
3. `__zwCeOrigEvHost` global（caret_range 回植前捕获页面家族宿主代理供 impl 派发）：isolated N1 **bi=1 input=1 全兑现**（壳段抛异常也消失）；语料 div-p 案 **got 0→2**。
4. 条件化（`el.isSameNode(origHost)` 同节点保 el）：双派未消（多案态 isSameNode=false——代理换代后同节点不同对象）。

**语料面双派机制**（未定谳，下轮续）：每案 initializeTest 写 innerHTML → query 家族视图滞后 → **每案都回灌**（非仅首案）→ 未合流态恒存；修复打通派发后多案上下文暴露双收（div-p 案 0→2）。与既有 got=2 342F 集群（isolated 单收、corpus 双收）**机制同源嫌疑高**——打通正确性修复会把更多 got=0 案变成 got=2，**先修双派再落事件丢失刀**。

## d40 补测：B3P 注入语料探针（双派分辨）

`zz-probe-b3p.html`（backspace.tentative 完整副本 + `__zw_ce_delete` hook 计数 +
每案事件账 + 末尾汇总 dump，基线二进制跑）：

```
hookCalls=72 cases=86 bi0=47 bi1=24 bi2plus=15
firstBi2 = abc<br> []def / div>abc<p>[]def<br>ghi / div><b>abc</b><p><b>[]def</b>
         / <div><p>abc</p>[]def</div> / <div><p>abc   </p>   []def</div>
```

结论四条：

1. **impl 无双调实锤**：hookCalls=72 ≤ cases=86（每案至多一次 shim 调用、14 案
   连 wrapper 都未进 = probe 判 '' 走 TextActionState）——webview 重试假设排除，
   双派在派发/listener 层。
2. **双派案簇 = join/br 空白形**，与 got=0 集群同形——同簇案在不同环境分别呈现
   got=0（基线+未注入）/ got=2（修复二进制或注入后），**不是两个独立 bug**。
3. **注入敏感性（方法学约束）**：hook 包装 `orig.call(this, sel)` 跨包装调用使
   家族判定漂移——基线二进制下注入即出 bi=2（未注入基线同案 got=0）。**计数类
   探针数据有效、行为类探针失真**：后续探针设计必须分离计数与行为。
4. **listener 表机制定位**：listener 按 `_listenerStore['@'+handle]` 键存储
   （part01.js:58），sel↔handle 身份桥 = `_zwQueryWrapIdentity`
   （part05.js:2027，handle 桥 + evict 回收臂）——双派/零派机制必在 handle 键
   层（换代重绑轨迹）。下轮探针：跨案 dump `_listenerStore` 键序列 + corpus
   gEditor 的 handle 轨迹，分辨「同 handle 双 listener」vs「双 handle 双表」。

## d40 补测 II：B3Q 换代轨迹 + 双派根因链定谳

`zz-probe-b3q.html`（addEventListener 挂载计数 + 逐案 gEditor 引用身份轨迹，
基线二进制）：`addCalls=0 gEdChanges=4 cases=86 bi2plus=15`。

**双派根因链**（harness initializeTest 逐行核对）：

1. harness `initializeTest` 34 行每案 `gEditor !== document.querySelector(
   "div[contenteditable]")` 身份检查——**上游 Chrome 真实 DOM [SameObject] 恒等
   恒 false**，从不走 remove/add 分支，单 listener 恒挂。
2. ZeroWeb querySelector **代理身份不稳定**（86 案间 gEditor 换 5 个对象，B3Q
   gEdChanges=4）→ 检查误判换代 → 走 remove/add。
3. **remove 不中**：`onBeforeinput` 是 initializeTest **内层函数（每案新闭包）**
   ——`removeEventListener(旧gEditor, fnN)` 传的是新闭包，旧 gEditor 上挂的是
   fnN-1 → 移除失败，**旧代理 listener 遗留**。
4. 派发把新旧两个代理的 listener 都触发（同底层节点双表联动）→ fnN-1 + fnN
   各 push 一次 → **bi=2**；4 次换代 × 遗留窗口 ≈ 15 案双派量级吻合。
5. gBeforeinput/gInput 每案 initializeTest 双重重置（43-44/52-53）——累计
   假设排除；addEventListener 原型 hook 拦不到 shim proxy 方法（addCalls=0
   无效数据，方法学注记）。

**修复刀（下轮）**：querySelector 同 sel **代理身份稳定性**——`div[contenteditable]`
恒同 sel 应恒同 proxy（`_proxyCache['s:'+sel]` 模型在 query 家族成立、页面家族
路径身份断链——先定位页面域 querySelector 走的缓存路径再修）。改动面 =
querySelector 缓存语义，风险高：全语料 + html-semantics 哨兵同二进制对拍 +
selection 全语料对拍必跑。**此刀落成 = 双派集群（got=2 342F 嫌疑）+ join 首触发
事件丢失集群同时解锁**——listener 表单挂后派发单收，事件丢失修复实验（d39
`__zwCeOrigEvHost` 刀）可重新落树。

**矛盾点（下轮入口）**：`_makeProxy`（part03.js:31440）键 = `_elKey(sel, handle)`，
querySelector 产物无 handle → 键 `'s:'+sel` 恒定 → 31442 cache 命中恒返同 proxy，
且 R4005 WeakRef 复活（corpus `gEditor` 变量强持 proxy → deref 恒活）——**模型上
身份应恒定，实测 gEdChanges=4**。候选解释：①某换代点 `delete _proxyCache['s:…']`
（R52 purge 面）②part04 qS（R322）某返回分支绕过 `_wrapSelector`/cache（handle
wrap / plain wrapper 直返）③页面域与 execute 域分 cache（两家族各自 `_proxyCache`？——
`_proxyCache` 是 part01 闭包单例，除非桥两侧各装一份 shim）。下轮先审计 part04
querySelector 全部返回点 + 搜 sel 键 delete 点。

## d41 补测：B3R handle/sel 轨迹——矛盾点破案

`zz-probe-b3r.html`（逐案记 `gEditor.__zwHandle`/`__zwSelector` 签名，基线二进制）：

```
changes=5 transitions:
  ERR|ERR → null|div
  null|div → null|html:nth-child(1) > body:nth-child(2) > div:nth-child(1)
  null|html…>div:nth-child(1) → null|div
  null|div → null|html…>div:nth-child(1)
  null|html…>div:nth-child(1) → null|div
```

**破案**：`__zwHandle` 恒 null（sel 形恒定）、**`__zwSelector` 在短形 `div` 与
完整路径形 `html:nth-child(1) > body:nth-child(2) > div:nth-child(1)` 之间摇摆**
——同元素两个 sel 方言（短形 = 简化快照视图命中、路径形 = host 静态元素方言，
part05:14604 注释自证路径形是 shim sel 方言）→ **两个 cache 键**（`s:div` vs
`s:html…`）→ **两个 proxy** → harness 身份检查误判换代。cache/purge/绕行三候选
全排除，**根因 = 双视图 sel 命名分歧**。

**修复方向（下轮，规范化刀）**：`__zw_query_match` hit 的 sel 命名双视图归一——
返回点（`_zwQueryWrapIdentity` 前）或 hit 生成点把同元素两方言归一到单一规范形。
**改动面警示**：sel 键牵动 listener 表键、写 journal 键、R54xx/pend 桶键等全部
查询消费面——**必须全语料 + html-semantics + selection 三哨兵同二进制对拍**；
若归一成本超限，备选最小刀 = harness 兼容面（qS 返回点对**同底层元素**的已存
proxy 反查——`__zw_contains`/元素指纹比对，O(查一次) 成本换身份连续）。

**桥侧定位（d41）**：`__zw_query_match` = `with_query_view_doc(&html, &m, |doc,
served_stale| query_match_selector_doc_cached(doc, ctx, &sel))`
（callbacks.rs:866-873）——hit sel 命名来自**查询视图树节点**：live doc（进程内
常驻）/ 查询快照 re-parse 双视图（`find_by_selector` 同款双源，callbacks.rs:815-827
注释「miss 回落查询快照 re-parse」）——两视图节点 sel 命名分歧即根因所在层。
下轮步骤：①读 `query_match_selector_doc_cached`/视图树构建，确认两视图节点
sel 字段生成（快照 re-parse 为何产短形 `div`、live 为何产路径形或反之）
②选归一点（桥侧 hit 输出规范形 = 改 Rust；shim 侧 `__zw_query_canonical_sel`
新桥回调 = 加回调 + `_zwQueryWrapIdentity` 归一）③三哨兵验证链。

## 回滚确认（d40-II 追记）

shim 本轮零改动（B3P/B3Q 均纯 HTML 探针），回滚段指 d39 修复实验；d40 两轮
探针数据完整落本档。

shim `git checkout` 后重建基线二进制：N1 bi=[] input=0（基线丢失态）、N2 bi=1——回滚干净，工作树零净码变。探针文件（zz-probe-b3bi/b3k/b3l/b3m/b3n/b3o，gitignored）留本地备查。

## 下轮刀清单（按收益排序）

1. **双派根因**（got=2 342F + 本轮暴露）：多案上下文 listener/代理叠影定位——b3o O1 isolated bi=1、corpus got=2，加 `__zwCeSyncCount` 回灌计数探针分辨「listener 叠挂」vs「双代理双表」。
2. **元素锚 range 选区删除语义**（≈820F 主簇）：12026/12155 silent return 的 select-delete 全语义（P15a/b/c 形），含 li 全选删除后列表结构维护。
3. **空 range beforeinput 静默臂**（主文件宿主起点/终点 caret，corpus 首案形）：12034/12187 改派 beforeinput（空 range 快照）不派 input——语义已定谳（`checkGetTargetRangesOfInputOnDoNothing`），小刀独立可落。
4. src 非末子保守不并（12040）/ run 中段（12038）按 corpus 案例逐形放刀。
