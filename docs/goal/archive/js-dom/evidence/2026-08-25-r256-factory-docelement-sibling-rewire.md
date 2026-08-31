# R256 Evidence — factory docEl mutation 兄弟 getter 重接线 + title ns 对齐（17,x 簇 12F 全解，+13P）

**日期**: 2026-08-25
**切片**: M4——R255 后续（17,x 12F 簇收口）
**改动面**: `part03.js`（factory docEl insertBefore/appendChild/removeChild 补
`_r130WireSiblings` + `_r223SetParent`；`_r130TitleEl` 补 ns/prefix 字段）+
`part04.js`（sibling trap 补 plainParent 解析）+ `part23.rs`（+1 回归单测）
**commit**: 4ed3dc9c6（rebase 后 hash，原始 b9fd356d0）

## 一、诊断链（harness 内嵌探针三轮）

1. **engine 级 twin-env 复刻（单测沙箱）**：独立 iframe 建 foreignDoc +
   paras[0]，sim（common.js mySurroundContents 全序）vs host
   `surroundContents` 双侧树 dump——**顶层形态完全一致**（[P{HEAD}, BODY]），
   R247 结论复证：分歧只在 harness 全环境。
2. **childNodes 递归 dump（harness 内嵌，经 assert 强制失败把树放进 Fail 消息）**：
   A/E 双树逐节点比较 IDENTICAL（nodeType/nodeName/nodeValue/attributes/
   childNodes 全等）——但 `isEqualNode` 仍返 false。
3. **nextNode 遍历 dump（getter 链）**：**首差暴露**——A 侧 walk
   `HTML → P → HEAD → TITLE → text → #comment`（**BODY 子树整段被跳过**），
   E 侧正常经 BODY。再扩展 ns/prefix/localName 字段比较，定位第二差：
   `/1/0/0/0`（HTML→P→HEAD→TITLE）**NS 分歧**——A 侧 TITLE-clone 为 XHTML，
   E 侧 TITLE 原件为 undefined（归一成 ''）。

## 二、根因（三层）

1. **factory docEl mutation 不重接线兄弟 getter**（主根因）：
   `_makeDetachedDocument` 的 docEl insertBefore/appendChild 只改 childNodes
   数组 + 裸赋 `c.parentNode = docEl`。surround 把 paras[0]（wrapper 域
   proxy）插到 docEl.childNodes 的 BODY 前后，oracle `nextNode` 走
   firstChild/nextSibling/parentNode **getter 链**——P.nextSibling 仍是旧值
   （null），BODY 子树整段从遍历里消失（childNodes 数组视图与 getter 视图
   分裂）。
2. **裸赋 parentNode 对 proxy 是静默 no-op**：wrapper 域子经 set trap，
   权威父链在 `_zwNodeParent[handle]` 注册表（`_r223SetParent` 写入
   plainParent 槽）；且 part04 sibling trap 的父解析只认
   parentSel/parentHandle，**不认 plainParent**——注册表写了也读不到。
3. **factory title 缺 namespaceURI**：`_r130TitleEl` 字面量无 ns 字段，
   isEqualNode 的 ns 归一比较（`ns == null ? '' : ns`）判空；而
   `_zwDeepCloneEl` 克隆副本落 `_zwMEl` 缺省 XHTML ns——sim（移动原件）与
   host（克隆循环）树比较失败，即使 walk 已对齐。

## 三、修复

- docEl `appendChild`/`insertBefore`/`removeChild` 后调
  `_r130WireSiblings(docEl.childNodes)`（既有 helper，position 感知 getter）
- 裸赋改 `_r223SetParent(c, docEl)`（proxy 域经注册表，plain 域经
  defineProperty 遮蔽）
- part04 sibling trap 父解析链补 `plainParent` 分支
- `_r130TitleEl` 补 `namespaceURI: XHTML, prefix: null`（与 headEl/docEl
  同款，spec createHTMLDocument 步骤 4 的 HTML 解析语义）

## 四、验证（vs R255 基线）

| 项 | R255 | R256 | Δ |
|---|---|---|---|
| Range-surroundContents | 1810P/30F | 1822P/18F | **+12**（17,x 全簇） |
| ranges 上游（Range-\*/StaticRange） | 38680P | 38693P | set-diff **+13 F2P / 0 P2F** |
| dom/nodes | 57F | 57F | 逐条一致 |
| dom/events | 579P/7F | 579P/7F | 持平 |
| dom/collections / traversal | 49P / 1602P/2F | 49P / 1602P/2F | 持平 |
| engine 单测 | 2395 | 2396 | +1 回归单测全绿 |
| fmt / clippy（-D warnings） | — | 干净 | — |

F2P 清单：17,0/4/6/9/11/13 各 2（surround DOM+position）+
Range-extractContents 17（foreignDoc docEl fragment）= 13。

**StaticRange-constructor 17P2F 假回归**：全量 sweep 日志中该文件未被
执行（0 行输出）而单独跑 17/17 Pass——harness 执行枚举伪影非代码回归，
以单独跑 + 中间轮全量日志双重核实。

## 五、R257 靶点

- 16,x startOffset 11F（`[document.body,4,document.body,5]`——R255 已定位
  iframe contentDocument 异步 fetch-rebuild 时序，深项）
- 18,0/19,6 4F（self-surround / detachedPara1 形态）
- 28,0 1F / 30,4/30,11 2F 残余
- customElements 多 registry / :scope query-root 深项
