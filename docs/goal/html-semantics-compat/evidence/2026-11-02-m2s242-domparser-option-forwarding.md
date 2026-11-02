# M2 s242 — DOMParser 快照 option accessor 三联补面（s239 家族延伸，R3019 桥转发）

日期：2026-11-02。切片号 s242（承 s241）。根：s240 挂账「DOMParser 工厂
option accessor 面」——`_zwMBuildBodyTree` 世界 option 无 IDL accessor（探针
在案：getter undefined + set 落 expando attr 不动）。

## 入场定刀位

s241 下一步候选：DOMParser 工厂 option accessor 面（s239 家族延伸，独立勘
根轮，覆盖该工厂 accessor 面）。

## 勘根（码读 + 探针物种勘定，zz-s242-probe.html 验后删）

- **物种勘正**（s240 账面「`_zwMBuildBodyTree` 世界」修正）：DOMParser 文档
  产物 ≠ `_zwMEl` 世界——`_zwParsedDoc`（part02 R2790 查询工厂）文档无
  `createElement`，全部元素面（querySelector/querySelectorAll/getElementById/
  getElementsByTagName/body/子元素查询 `_zwWrapQ`）产 **`_zwParseEl` 实例**
  （只读快照对象：own 数据属性 id/className/textContent/outerHTML/innerHTML/
  _attrs/_zwPath + 原型方法面，无 `__zwHandle`/`__zwSelector`/`__zwIsMEl`）。
  探针实证 species=parseEl。
- **before 面**（16 断言）：三读全 undefined（value/label/text 无 accessor）、
  textContent="First"（own 数据属性）、attr 读通（"v1"/"l1"）、三写全落
  expando（value.after 读回 "x2" 而 getAttribute 仍 "v1"——读/写源分裂同
  s241 形；textContent/innerHTML 写后不变——写面全死）、无属性 option 文本
  回落死（o2.value=undefined）、子查询同物种死（o3.value=undefined）。
- **工厂面盘点**：`_zwMEl` OPTION 产品（detached doc createElement/
  createHTMLDocument → `_makeDetachedDocument`）已带 s239/s240 三联 accessor
  （part03 OPTION tag 门，s240 形状 D 钉面）；snapshot 物种是唯一三联缺面。
  `adoptNode` 已有 `_zwParseEl` 物化通道（t8q `_wcRebuildAsHandle`）。
- **桥先例**：R3019 lazy 可变子树桥——childNodes/children/firstChild/
  lastChild getter 与 insertBefore/appendChild/removeChild/setAttribute/
  removeAttribute/hasChildNodes 全部**首次访问 `_ensureMutTree()`**（outerHTML
  经 `__zw_parse_html_child_nodes` 重建 `_zwMEl` 子树，attrs 经快照 `_attrs`
  播种，惰性一次缓存 `this._mtree`，物化同时 rewire getAttribute/hasAttribute/
  attributes/innerHTML/outerHTML/textContent 为 live 树视图）。IDL 三联缺的
  只是同款转发入口。
- **规范面**：html.spec.whatwg.org 本环境不可达（同 s238/s240/s241 先例），
  语义锚点 = s239/s240 已钉的三联读写语义（HTML §4.10.9：value 读 attr 存在
  即采纳/缺省回落子文本 strip-collapse、label 读 label attr/缺省回落 text、
  text 读子文本 strip-collapse；写三联 = 纯内容属性反射 verbatim /
  replace-all 单 Text 子）——转发设计零语义复制，`_zwMEl` 产品承载全部语义。

## 桥力学

`_zwParseEl` 构造器 OPTION tag 门（`this.localName === 'option'`，与原型
`rel` getter 同款小写直比）装三对转发 accessor：get →
`this._ensureMutTree().<prop>`、set → `this._ensureMutTree().<prop> = v`
（configurable/enumerable true 同 s239）。首次 IDL 访问触物化：解析 attrs
播种 → value/label 读 attr 层；outerHTML 重建子树 → text 读走查回落后者
写 textContent replace-all。写后 `_ensureMutTree` 已把 getAttribute/
innerHTML/textContent/attributes rewire 为 live 视图 → 同对象全脸新鲜（spec
一致性免费获得，无同步代码）。

## 施工（一处一块，零 rust）

part02 `_zwParseEl` 构造器尾 OPTION 门三转发 accessor + s242 注记（物种勘
正 + before 探针实证 + R3019 口径锚）。

## 验证

- node --check 全 shim concat（bridge include 序 18 片）干净；
  `make zero-wpt-runner-release` 重建 30.4s。
- 探针 before/after（16 断言逐面）：after value="v1"/label="l1"/text=
  "First"（三读通）、value 写后 attr.value.after="x2"（分裂闭合）、
  textContent.after="T2"/innerHTML.after="T2"（replace-all 穿透 rewire 面）、
  o2.value="Second"（无属性文本回落）、o3.value="v1"（子查询同物种）——
  全吻合；own 面新增 value/label/text accessor 三键。
- 语料盘点：html/semantics 全域 DOMParser 消费面 6 文件，无一触 option/
  select（tBodies.xml/toggleEvent/document-without-browsing-context 等）——
  靶面无语料行，验证由探针承担（s239/s240/s241 同形）。
- solo the-option-element **108P/0F/0T**（R3907/s239/s240/s241 钉面零回归）。
- 全树 A/B（evidence/logs/s242-ab.log vs s241-ab.log）：见下节。

## 全树 A/B（evidence/logs/s242-ab.log vs s241-ab.log）

- (key,state) 对 sort -u join：8293/8293 对，LOST 0 / GAINED 0 / 状态翻转 0
  ——连续第二个完美 A/B。行级 7708P/507F/81T 双侧逐数同。
- 靶面无语料行（html/semantics 全域 DOMParser 消费 6 文件无一触 option/
  select，码读盘点在账）——零位移与机理一致；改动臂 = snapshot 物种
  OPTION 门构造器 accessor，其余 tag 与 `_zwMEl`/wrapper 物种零通道。

## 残余挂账（本切不动）

- **snapshot option 邻接 IDL 面**：selected/disabled/index/form 等三联外
  接口（selected 需 select 上下文 selectedness 机器、index 需列表位次——
  snapshot 物种无 select 父上下文，独立勘根轮评估）。
- s241 存量：同类直写臂未同步清单（meter 六属性/progress.max/FRAMESET
  cols/rows 散点直写）；`_zwOptionValue` 键派生不一致（裸 handle vs @handle）。
- s240 存量：plain detached select.value undefined（s186 FIXME）；option.text
  null→'' vs DOMString "null" 字面差异（语料无钉面两态连续）。

## 下一步候选

modal-dialog-selection（pointer 拖拽面存量）；customizable-select 177F（挂
起待用户拍板）；img SVG 密度族（naturalWidth 22F + current-pixel-density
13F，需独立勘根轮）；snapshot option 邻接 IDL 面（本切新账）。
