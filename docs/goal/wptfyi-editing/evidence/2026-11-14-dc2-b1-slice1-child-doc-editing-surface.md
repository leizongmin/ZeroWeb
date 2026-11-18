# DC-2 · d21 轮：B1 切片 1——子文档编辑面（shim 三修复，跨层断链解除）

日期：2026-11-14。修复队列 B1（[d20 三桶归类](2026-11-14-dc2-residual-bucket-triage.md)
簇 1+2，479F 断链域）切片 1：仅 shim 侧（part03/part05），runner 键路由留
切片 2。before/after 同机同命令（test-guard 包裹）。

## 三修复

1. **detached doc parentNode 链修复**（part03 `_makeDetachedDocument`）：
   `headEl.parentNode = docEl` / `docEl.parentNode = doc` 两处 plain 赋值
   在 R130 setPrototypeOf（docEl→HTMLHtmlElement.prototype 等）之后执行，
   而 R3019 在 Element.prototype 挂的 parentNode 是 getter-only 访问器
   （own 缺省返 null）——非严格模式赋值静默失效，own 属性永不建立 →
   body→docEl→doc 上行链在 docEl 断。探针实证（zz-probe-b1chain）：
   getRootNode(child body) 返 docEl 非 doc；EditorTestUtils.sendKey 的
   light-DOM 上行 `e.getRootNode({composed:false}).host` 得 undefined →
   下一轮 `e.getRootNode` TypeError——DesignMode 变体 146F 触发器。改 own
   defineProperty（R223 健壮写同款，11251 先例）。spec
   https://dom.spec.whatwg.org/#dom-node-parentnode。
2. **iframe 子文档 execCommand 薄面**（part05 `_zwMakeIframeDoc`，紧邻
   R3254-E2 doc.getSelection）：corpus 六个用例文件的实际调用集 =
   defaultParagraphSeparator（per-doc 闭包状态，p/div 合法值，非法 false）
   + createLink（非折叠 extractContents→\<a href\> 包裹→选区重锚 link
   内容；折叠 false）+ delete/forwarddelete（非折叠 deleteContents+折叠；
   折叠臂 no-op true，块合并属键路径 FIXME(B1 切片 2)）+ stylewithcss/
   usecss（accept+store）。未覆盖命令返 false。333F setup TypeError
   （`iframeElement.contentDocument.execCommand is not a function`）触发器
   解除。
3. **子窗 Selection.collapse 空选区臂**（part05 per-iframe Selection）：
   旧实现 rangeCount 0 抛 InvalidStateError；spec selection-api
   #dom-selection-collapse 空选区建新 range 后置边界（collapseToStart/
   End 才有 InvalidStateError 门）。corpus setup 在 fresh 子选区直调
   `childDoc.getSelection().collapse(div.firstChild, 0)`——setup 链第三层
   断点解除。

## 分层解除实证（delete-in-child-of-head 族 68 断言，before→after）

- before 全文件唯一失败形：`TypeError: iframeElement.contentDocument
  .execCommand is not a function`（setup 第一层）。
- after 全文件唯一失败形：`TypeError: Cannot read properties of
  undefined (reading 'send_keys')`——setup 三层全通过（designMode ✓ d19、
  execCommand ✓ 本刀、collapse ✓ 本刀），断在子窗 test_driver 面缺失
  （**切片 2 边界**：runner send_keys 键路由按目标文档解析）。

## 副效应（bisect 定谳：修复 1 所致，净正）

input-events get-target-ranges-forwarddelete 页 8 行由
`apply mutations: unknown parent handle __n334` 硬错（promise 未完成，
断言不发射）转入真断言阶段：其中
`Delete at "<p><img>[]<br></p>" - comparing innerHTML` **转 Pass**
（input-events 唯一新过行），其余 7 行 Fail 但推进到 beforeinput 计数/
innerHTML 比对阶段。三修复二分（撤销 collapse / 再撤销 execCommand 面）
两步均保持 145 条发射 → 定位修复 1。机制：docEl.parentNode 接通后
selector/handle 解析上行链完整，悬垂 hard error 消除。同页两文件断言数
137→145、149→150（前轮账该页未发射完成即页级快照——非语料漂移，语料
文件 mtime 11-06 未变，pre 二进制复跑 137 恒定）。

## 对拍账目（同机同命令，键归一 集-差 口径）

| 语料 | before | after | F2P | P2F | new-P | new-F |
|---|---|---|---|---|---|---|
| selection 全语料（selfull3→selfull4） | 47,320P/2,646F | 47,326P/2,647F | 0 | 0 | 6（本刀探针页 zz-probe-b1chain 6/6） | 2（nested-video 媒体页 Timeout 键漂移 + 并行流新探针页 zz-probe-b1min） |
| input-events 全语料（inputfull→inputfull2） | 210P/910F | 211P/918F | 0（新键 +1P 见上） | 0 | 1 | 8（crash→断言阶段推进） |

- face 六文件（editing-run）：bold 3039P/9F、italic 2070P/3F、
  strike 2147P/4F、underline 2144P/3F、subscript 1241P/0F、
  superscript 1273P/0F——与 d19 终账逐文件恒等（19F 基线口径不变）。
- 哨兵 15 探针全绿同 d19 账（d6diag 22/25 与 d19 sentinels.json 逐名
  恒等——master.md 旧记「24/25」系笔误，两轮实账均 22/25 同 3 名失败）。
- 引擎 lib 钉测 ×2（终源复跑）：3944 passed / 7 failed，失败清单与
  d19 已知红集恒等（img settle ×2 / fetch filtered / iframe SW /
  readable stream / websocket ×2），零新增。

## 本刀范围与挂账

- 切片 1 到位：479F 断链域的 setup 三层（execCommand/getRootNode 链/
  collapse）全解除，676 断言全部越过 setup 进真断言/驱动阶段；F2P 释放
  待切片 2（runner send_keys 子文档路由——childWindow.test_driver 面 +
  selectorFor 文档限定 + `__zw_ce_*` 族按目标文档参数化 + designMode=on
  全文档宿主语义）。
- 账目口径注：F2P=0 系键恒等口径；新键 +1P（crash 解锁转 Pass）未计入
  旧键 F2P，证据面如实分列。
