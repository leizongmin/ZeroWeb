# DC-2 · d22 轮：B1 切片 2——子文档键路由（子窗 test_driver 面 + 键语义 shim 直应用）

日期：2026-11-14。修复队列 B1（[d20 三桶归类](2026-11-14-dc2-residual-bucket-triage.md)
簇 1+2）切片 2。原计划 runner 侧键路由（[d21 切片 1](2026-11-14-dc2-b1-slice1-child-doc-editing-surface.md)
挂账），落地改为 **shim 侧子窗 test_driver 面**：iframe 子文档不经 webview 键
管线（选区/树均在 shim 工厂域），键语义在子窗 `window.test_driver` 直应用
（per-iframe Selection / 切片 1 execCommand 薄面同域近似）——runner 侧
selectorFor 文档限定/`__zwSelector` 跨文档形因此**无需**（主 stub 键路径对
子文档不再触达）。

## 落地面（part03 +17 行 / part05 +946 行）

1. **子窗 `test_driver` 面**（part05 `_zwMakeIframeWin` win 字面量）：按
   corpus 消费集 `send_keys`（无修饰键路径）+ `Actions`（keyDown/keyUp/click/
   send，修饰键粘滞忽略——字符字面已含大小写）+ `set_test_context`（ETU
   构造器消费，no-op）。PUA 键码映射与 runner send_keys 同形：
   /E017→backward/forward delete、E006/E007→enter、E010/E011/E012/E014
   →end/home/left/right（方向键非折叠选区先收拢、折叠态字符步进/行端）；
   其余 PUA 拒绝（FIXME(B1 切片 3)），修饰键/escape/tab 等吞掉。
2. **键入语义**（`_b1InsertText`）：文本节点 data splice（工厂文本节点无
   insertData）+ 元素容器落点建文本子；typing-around-link 边界面——光标在
   链接文本端点（offset 0 首文本 / len 末文本）时键入**不扩进链接**，文本落
   链接外侧，链接内层非链接行内链剥出后与新文本重裹置原位（corpus
   `<a><b>[]abc</b></a>` → `<b>XY<a>abc</a></b>` 形）；边界面单键性记账
   （`_b1TypedAt` 同点续键不重触发，显式重定位复位）。
3. **删除语义**（`_b1Delete`/`_b1Join`/`_b1JoinMembers`）：字符删除 + 同容器
   相邻文本合并；块级 join（delete-in-child-of-{head,html}）——backspace 并
   前一可见块、forwarddelete 并后一可见块，可见门（族标签未显式
   display:block 才可 join）、caret 落族标签内整 join no-op、body 吸收臂
   （对侧 children 经 body innerHTML 重写并入，R220 纪律——不走 childNodes
   补丁）、join 缝不可见元素移出合并块原序回插（assert_in_array 前后两形
   皆收）。
4. **insertParagraph**（`_b1Enter`）：title/style/script 容器 pre（white-space
   pre 系）插 `\n`（空容器 `\n\n`）、非 pre no-op；块容器（div）拆分——
   caret 位置切 children、右半入同标签克隆、空侧填 `<br>`（defaultParagraph
   Separator 不改宿主标签）。
5. **子文档结构面**（切片 1 遗留断链补齐）：
   - part03 headEl **属性面**（set/get/has/removeAttribute，本地属性表存取
     不入 _tree——corpus 尾步 `head.setAttribute("style",…)` /
     `head.removeAttribute("style")`）；
   - docEl **虚树 innerHTML 读面**——head 专属标签（meta/title/style/script/
     link/base/template）按 body 树顶层提升进 `<head>` 段序列化（不移动节点
     ——Range 语料 body 视图偏移不变），headEl 子与 docEl 附加子按
     `__zwB1Slot` 槽位（before-head/before-body/after）定位；
   - docEl **insertBefore 虚参照**（`insertBefore(div, childDoc.body/head)`
     ——真浏览器 body/head 是 docEl 子，本仓双树形下按槽位入 docEl
     childNodes + NotFoundError 门）；
   - **body 视图树根上行链接**——解析树顶 parentNode 链止于树根镜像
     （getRootNode 断），接回 body 视图（只加 parent 链不移动节点）；
   - `_zwMakeIframeDoc(undefined)`——门 `title !== undefined`，旧 `''` 给每个
     iframe 文档塞空壳 `<title></title>`（docEl innerHTML 双 title 根因）；
   - 工厂元素 `focus`/`blur` no-op + `isConnected` parentNode 链上行（
     `div1.isConnected ^ div2.isConnected` 存留判定）；
   - 子 execCommand delete/forwarddelete **折叠臂**经 `doc.__zwB1KeyFace` hook
     走键语义（与 sendBackspaceKey 同期望表）。

## 对拍账目（同机同命令，键归一 集-差 口径）

| 语料 | before（selfull4/inputfull2） | after | F2P | P2F | new-P | new-F |
|---|---|---|---|---|---|---|
| selection 全语料（selfull4→selfull5） | 47,326P/2,639F | 47,716P/2,273F | **367** | 1（见下） | 24（本刀探针页 zz-probe-b1s2 24/24 全绿） | 0 |
| input-events 全语料（inputfull2→inputfull3） | 211P/915F/3T | 恒等 | 0 | 0 | 0 | 0 |

- **F2P=367 分文件**：typing-around-link-collapsed 121（DesignMode 变体
  1P→37P/36P/26P/26P）、typing-around-link-non-collapsed 80（DM 变体
  1P→31P/21P/16P/16P）、delete-in-child-of-head 64（68F→4F）、
  insertparagraph-in-child-of-head 64（72F→8F）、
  insertparagraph-in-child-of-html 24（72F→24F）、delete-in-child-of-html 14
  （46F→32F）。目标七文件合计 before 38P/724F → after 405P/357F。
- **P2F=1**：selection/deleteFromDocument.html `Range 18: [document.head, 1,
  document.head, 1]`——`_zwMakeIframeDoc(undefined)` 后 headEl 视图无 title，
  Range setStart(head,1)（offset=childNodes.length）越界 IndexSizeError。旧
  空壳 title（length 1）恰使该 offset 合法。孤例（同文件其余 59 断言恒等、
  HTMLDetails 变体 30 断言恒等）。**真修属 headEl 视图持真解析 title**（解
  析期 head 段提升 + detHtml head 段 + R164 查询 adapter identity 贯通——
  R220 矿区，另开切片），本刀挂账不回退（净 +366 断言）。
- **stringifier_editable_element 12→8 断言**：并发跑（face+钉测编译+
  inputfull3 同机）CPU 争用致页级 Timeout 键漂移；隔离复跑 12 断言与
  before 逐名恒等（5F/2P/1T/4 NotRun）——负载伪账非回归，selfull5 终账按
  隔离复核口径记。
- **face 六文件**（editing-run）：bold 3039P/9F、italic 2070P/3F、
  strikethrough 2147P/4F、underline 2144P/3F、subscript 1241P/0F、
  superscript 1273P/0F——与 d21 终账逐名恒等（19F 基线口径不变；strike 过滤
  勘误：b1 轮 `FILTER=strike` 连带 zz-probe-d15/16strike 探针页 4 断言，
  本刀按 `strikethrough.html` 精确过滤对拍 2147/2147 恒等）。
- **哨兵**：selection 通道 zz-probe 14 文件（含新 b1s2 24/24）逐文件与
  before 恒等零位移；editing-run 通道 zz-probe 54 文件对 d19 终账
  （zw-d19-sentinels.json）逐名 A/B——除 f6bwin 两变体**发射深度漂移**外
  断言级恒等：`?8001-9000` 847→1、`?9001-last` 240→252，重叠区前 240 断言
  状态 0 错配；HEAD 二进制（d21 已提交态）bisect 同病灶（639/1——两二进制
  各有一变体 ~1 断言即殁）＝共享超时预算下两窗切片进度深度非门面（d15 起
  master 已记「页级超时非门面」），非 d22 行为变更。b1 轮哨兵全扫记录损坏
  （b1-sentinels.json 11×`[]`，d21 实未跑 54 全扫——本轮补回）。
- **引擎 lib 钉测**（test-guard + 独立 CARGO_TARGET_DIR=/tmp/d19tgt）：
  3944 passed / 7 failed，失败清单与 d21 已知红集恒等（img/svg settle ×2 +
  fetch filtered + iframe SW + readable stream + websocket ×2），零新增。

## 探针（zz-probe-b1s2，24 断言全绿）

headEl 属性面 / 解析 title 元素面 / createElement('style') focus 面 / 子窗
focus 族 / docEl innerHTML 虚树形 / docEl insertBefore body 参照 / isConnected
面 / typing DM 三形（srcdoc script、setup+createLink、insertText 语义）/
handler 键面 12 断言（insert+backspace 内部、backspace/forwarddelete 块 join、
meta 搬运、visible 门 no-op、enter no-op/pre `\n`/div 拆分、doc 级结构、body
吸收、Actions 修饰路径）。开发期 7 个 diag dump 断言（querySelector 组合器 /
ETU ctor / CE typing / DM replace-link 逐步 / 手工手术）随探针收敛**移除**
——结论入下节挂账。

## 挂账（切片 3 边界）

- **typing-around-link CE（ContentEditable）变体未动**（主窗路径，
  collapsed 124F + non-collapsed 79F + after-joining-paragraphs 48F）：diag
  定位两断点——①子窗 `w.EditorTestUtils is not a constructor`（iframe 内
  script 资源加载/注入面）；②主窗 CE 宿主 send_keys 键入后宿主内容清空
  （`<div contenteditable>` dump 变空——主窗键路径宿主解析问题）。DM
  createLink 重锚 `(a,1)` 落点键入仍走链接内替换（`<a>X</a>`→`<a>XY</a>`，
  边界面未触发——`_b1NearestLink` 对元素偏移落点的 `_b1LinkEdgeAt` 空
  链接保守臂放行），corpus `ContentEditable&child=b` 变体残余 29F+16F 主域。
- **execCommand insertParagraph per-doc 分隔符消费仍缺**（FIXME(B1 切片 3)
  已标——corpus 子文档 insertparagraph 走 sendKey 键路径已承载，execCommand
  直调臂未被语料消费）。
- **delete-in-child-of-html 32F / insertparagraph-in-child-of-html 24F**：
  doc 级 join/enter 的 html 宿主形残余（headEl 序列容器 join 已通，html 根
  直下宿主与多附加子形待精化）。
- **headEl 持真 title**（deleteFromDocument P2F=1 根修）：解析期 head 段提
  升入 headEl.childNodes + detHtml head 段 + R164 查询 adapter 对 headEl 节
  点直出（identity 统一方向）——R220 矿区（sim insertNode -158P 先例），
  需全语料对拍护航，独立切片。
