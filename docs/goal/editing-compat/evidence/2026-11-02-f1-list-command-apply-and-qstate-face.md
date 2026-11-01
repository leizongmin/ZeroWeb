# F1 list 命令簇：insertorderedlist/insertunorderedlist 应用引擎 + qState/qIndeterm 读面

日期：2026-11-02（editing 流，控制面下一步 ①，基于 gate13/g14b 基线对账）

## 0. 背景与范围

普查轮（2026-11-01）定案 F1 为最大单族：iloop/illist 全语料 compare F got 与输入
恒等（命令零应用）+ qState before/after 面读面未接，~600F。本轮移植 vendored
toggleLists/getSelectionListState/splitParent/fixDisallowedAncestors/outdentNode
（ol/ul 臂切片）到 part01e 新分片，part06 branch3 补两命令应用臂 + qState/
qIndeterm 读面。

## 1. 实现（三文件）

### 1.1 part01e.js（新建，348 行，挂 part01d 后）

- `_zwListSelectionState`（getSelectionListState :3949 移植）：blockExtend +
  contained 过滤（editable + 非缩进元素 + ol/ul 或其子或 li 允许子）→ everyOf
  'ol'/'ul' → hasOl/hasUl → 'ol'/'ul'/'mixed'/'mixed ol'/'mixed ul'/'none'。
- `_zwListToggle`（toggleLists :5690 移植）：mode = state==tagName ? disable :
  enable；enable 换 otherTagName 型（邻接同名列表并入 splitParent+wrap）/ 并
  主循环（p/div→li 换型、li/ol/ul 直收、inline 段 wrap 成 li、sublist 聚合套壳）；
  disable 连续兄弟聚 dsub → utag 元素走 `_zwListOutdentListNode`（outdentNode
  ol/ul 臂切片：摘 reversed/start/type → preserve-descendants 撤壳 → 子
  fixDisallowedAncestors）→ 其余 splitParent + fixDisallowedAncestors。
- `_zwListSplitParent`（:5058 忠实移植）：尾段独在臂（!firstIn && lastIn）逆序
  后移 + 尾 br 保全；首段缺位臂 cloneNode 壳承接 + id 摘除；通用臂成员前移 +
  followsLb/precedesLb br 保全 + 空壳删除。
- FIXME 挂账：normalizeSublists（嵌套归并）、recordValues/restoreValues（样式
  保全）、outdentNode simple/indentation 臂与 ancestorList 缩进臂、
  isSimpleIndentationElement 缩进合并新父臂。

### 1.2 part01c.js（一笔）

`_zwJusWrapViaHtml` 返回值：fresh np（createElement 新包）经 innerHTML 重写后
原引用已脱挂，返回 host 内 open-tag 重定位后的新包元素（复用包返回原元素）。
list enable 主循环依赖返回值继续套壳——裸 `<li>` 无 ol 壳实证。消费面仅 part01e
一处取返回值，justify 簇弃返回值调用不受影响。

### 1.3 part06.js（三笔）

1. branch3 应用臂（justify 臂后）：`insertorderedlist/insertunorderedlist &&
   rng2 && host2` → `__zw_ce_list_toggle(cmd=='insertorderedlist' ? 'ol' : 'ul')`；
   事件口径同 justify 臂（beforeinput 已由事件序派发、input 走既有尾部）。
2. `queryCommandState`：list 命令 → `__zw_ce_list_state(cLow).state`（format
   fallthrough 前）。
3. `queryCommandIndeterm`：list 命令 → `__zw_ce_list_state(liLow).indeterm`
   （值命令查询前）。state = list state 恰为本命令 tag；indeterm = mixed |
   mixed（ol|ul）。

## 2. 探针定案（zz-probe-list.html，gitignore）

六案全绿（vendored 忠实行）：`foo[]bar`+iol → `<ol><li>foobar</li></ol>` +
qStateIol=true；同形 iul → `<ul><li>foobar</li></ul>`；ol 内 caret qState before
Iol=true；ol 内 iol toggle-off → `<li>foobar</li>`（vendored 忠实：isAllowedChild
(li,div)=true，fixDisallowedAncestors 不换 div）+ qState 双 false；ol 内 iul →
`<ul><li>foobar</li></ul>` 换型；双 p 跨块选区 iol → `<ol><li>foo</li><li>bar
</li></ol>`。

## 3. 探针轮三 bug 修正

1. **tag 常量大小写**：`otherTagName = utag === 'OL' ? 'UL' : 'OL'` 大写常量传
   `_zwJusIsHtmlElement`（经 `_zwJusTag` 小写比较）恒 miss——ol→ul 换型恒等
   实证。tag 常量全改小写。
2. **WrapViaHtml 脱挂返回**：fresh np 场景返回脱挂引用，enable 主循环套壳断链
   （裸 `<li>` 实证）→ 返回重定位后元素（1.2）。
3. **mixed 判序反写**：`hasUl ? 'mixed ol' : 'mixed ul'` 双反 + 缺 'none' 尾，
   对照 vendored :4013-4033 修正（hasOl→'mixed ol'、hasUl→'mixed ul'、否则
   'none'）。裸 li 案 qIndIul 误 true 实证。

## 4. 对账（gate15，systemd user unit 串行批）

gate15（干净二进制，批 00:46-01:50，systemd user unit 串行 + test-guard）：

| 语料 | 基线（g13/g14b） | gate15 | Δ |
|---|---|---|---|
| insertorderedlist | 1,438P/323F | **1,619P/142F** | F→P 181、P→F 0 |
| insertunorderedlist | 1,721P/344F | **1,910P/155F** | F→P 189、P→F 0 |
| multitest | 9,486P/606F | **9,538P/554F** | F→P 52、P→F 0 |
| inserttext | 2,777P/232F | 2,777P/232F | 恒等 |
| insertparagraph | 6,720P/475F | 6,720P/475F | 恒等 |
| forecolor | 2,279P/48F | 2,279P/48F | 恒等（五值对照） |
| wss-ilb / wss-ip | 552P/0F / 1,104P/0F | 恒等 | typing ttags 消费面零位移 |
| join 三案 | 3,456/2,607/1,455P/0F | 恒等 | 零位移 |
| chd / ipihost / void×2 | 420/880/72+48P/0F | 恒等 | 零位移 |
| run-ilb | 1,641P/50F | 1,641P/50F | 恒等 |
| indent | （首轮） | 1,182P/147F | 新基线（list×indent 交互面） |
| engine lib | rc=0 | **rc=0** | |

合计 F→P **422**、P→F **0**。multitest F→P 52 逐案归 list qState-after 面（fam 普查
qState 24+24 + 零星）。

### 4.1 残余 F 定谳（iloop 142F 形态抽样）

- **dd/dt 子树入列形**：`<dl><dt>foo<dd>[bar]<dt>` got `<li><dd>bar</dd></li>`
  vs expected `<dd><ol><li>bar</li></ol></dd>`——dd/dt 作为子列表容器保留壳，
  li 不裹 dd（vendored items 聚合 dd/dt 特殊臂未移植）。
- **缩进合并形**：blockquote 祖先时 expected li 并入外层 ol（isSimpleIndentation
  Element 缩进合并新父臂 FIXME ④），got 产 blockquote 壳。
- **聚合边界形**：`foo<ul><li>[bar</ul>baz]` 部分跨列表选区双 li 合并（sublist
  聚合尾边界）。
- **qV/qState 零散**（~16+49）：stylewithcss 形 qState 期望 true got 空等读面
  零散（iloop 49 + illist 同族）。

compare 主面（1,438→1,619 的 181 翻绿对应）已收口，残余为嵌套/缩进交互面
（FIXME 挂账）与读面零散。

## 5. 残余与挂账

- F2 insertParagraph 前段包裹（ipara 465F + multitest D2 21F + inserttext `\n`
  族，~490F）→ 下一步主刀。
- multitest D1 delete 撤壳（~100F）、D4 typing 尾 br 边界（~24F）。
- list FIXME：normalizeSublists / recordValues / restoreValues / outdent 缩进臂
  / isSimpleIndentationElement（嵌套列表与缩进交互面）。
- qIndeterm mixed 判定已按 vendored 修正，gate15 前后对账（iloop 29/illist 24
  零散面）。
