# M2 s244 — snapshot option 邻接 IDL 面补面（selected/disabled/defaultSelected/index/form；s242 三联外接口销案）

日期：2026-11-02。切片号 s244（承 s243）。根：s242 新账的快照 OPTION
邻接接口面——`_zwParseEl` 查询产物四读 undefined、selected/disabled 写落
expando（读/写源分裂同 s242 形）。

## 入场定刀位

s243 下一步候选：snapshot option 邻接 IDL 面（s242 新账，独立勘根轮）。
备选 img SVG 密度族（需独立勘根轮，未动）。

## 勘根（探针四轮 zz-s244-probe.html 验后删）

- **wrapper 世界语义盘点**（本轮只读映射，不改动）：selected setter =
  `_optSelectedDirty[key]=!!v` + `_zwAskForReset`（part05 R3908b——**不反射**
  selected 属性，option-selectedness-script-mutation「IDL selected does not
  reflect」钉面）；getter 读序 dirty → `_optSelectedClean` → attr；
  defaultSelected setter 反射 attr + 非脏时 selectedness 跟随（R5009 片 d）；
  disabled = 布尔反射；index = SELECT 祖先走查 → qSA identity 比对位次，
  无容器回落 **0**（option-index.html「no container → 0」、datalist 内亦 0
  语料锚）；form = `_zwLegendOptionForm`（OPTION 沿 parentNode ≤64 找
  SELECT → 其 `.form`，无 → null）。
- **probe1 物种矩阵**：快照四读全 undefined（s242 后仅 value/label/text 三联
  已补）；`document.createElement('option')` 实为 **wrapper 物种**（有
  `__zwHandle`——Option 构造器走 document.createElement；真 plain `_zwMEl`
  option 仅 foreign-doc `createHTMLDocument` 工厂产物，本轮未入账）。
- **probe2 内部定谳**：`_zwPath` 全程宿主派生且完整（o1 =
  `[sig:HTML||…, sig:BODY||…, id:f, id:s]` 根→父，sig 形自带 TAG）；select
  行 full 键串与本元素路径前缀**逐字节吻合**；脏 selectedness 全局表跨实例
  可读（`crossInst.sel=true sameObj=false`——每 query 新实例，R5254 键级
  寻址生效）。**阻塞点单发**：`ownerDocument.querySelector('#s')` = null
  ——DOMParser 产物无 `_zwOwnerDoc`（仅 detached-doc 工厂 part03 设），
  ownerDocument 回落**主 document**，文档内祖先解析无通道；index/form 静默
  走回落值（0/null，与真值同形不可辨——o2.idx=0 应 1 暴露）。
- **probe4 边缘矩阵**（工厂补面后）：optgroup 嵌套位次计数正确（a2.idx=1）、
  双 select 各自闭域（b1=0/b2=1）、datalist→0、无容器→0、form=null（无
  FORM 祖先）、selected false 写读 false 且 attr 不落、dirty 压过 attr 读、
  defaultSelected 读恒 attr、disabled attr 级直设可读（`setAttribute` 后
  getter=true，物化前 `_attrs` 单源）。

## 桥力学（两块）

- **doc 级查询产物挂根上下文**（`_zwParsedDoc` querySelector/querySelectorAll
  产品补 `e._zwRootHtml = this._html`——元素级 `_zwWrapQ` 同口径）。此前
  doc 级产物无根上下文：matches 走自身包裹近似、s244 祖先解析无数据源。
  ownerDocument 回落面不动（R156 WPT Element-matches 依赖主 document
  defaultView，`_zwOwnerDoc` 化会引入缺 defaultView 崩面回归——探针 odoc
  通道弃用改根 html 通道的归因）。
- **祖先解析 = `\x1f` 边界严格前缀**：经宿主 `__zw_parse_html_query` 取
  option/select/form 全行（行含宿主派生 `path` + id/tag/cls/outer），行全
  路径键串（`path + '\x1f' + 键`，键派生与 `_zwEvKey` 同构）为本元素全路径
  键串的边界前缀即祖先——id/sig 键形态无关，免 tag 反查；位次 = select 前
  缀域内 option 行文档序计数（optgroup 行同前缀即计）；form = 最深 SELECT
  祖先前缀确定后其上方最深 FORM 前缀行重建实例返回。同 sig 孪生 option 键
  串同串取首个（宿主路径架构固有限度，语料无钉）。

## 施工（一处文件两块，零 rust）

- part02 `_zwParsedDoc` 查询工厂：根上下文补面（s244 注记）。
- part02 `_zwParseEl` 构造器 OPTION 门：selected/disabled/defaultSelected/
  index/form 五 accessor（s244 注记块——语义镜像 wrapper 世界机器 + 快照
  两层形读序；selected 脏态挂 `globalThis._zwParsedOptSel`（R5254 身份键表
  同族，表定义置 `_zwParsedToggleH` 旁）；**不走物化**（hasAttribute 物化前
  读 `_attrs`、物化后经 R3019 rewire 读 live 树，两态单源）；ask-for-reset
  互斥清兄弟步未携带——快照树无 selectedness 全景遍历通道且语料无钉面 FIXME）。

## 验证

- node --check 全 shim concat（bridge include 序 18 片）干净；
  `make zero-wpt-runner-release` 重建 32.6s。
- 探针 probe2/probe4（验后删）：o2.idx=1、o1.form=id:f、跨实例脏态三证
  （sel=true attr=false sameObj=false）+ 边缘矩阵八点全吻合。
- solo the-option-element **108P/0F/0T**（standing 门零回归；option-index/
  option-selected/option-form 语料面全 wrapper 物种，快照补面零位移）。
- 全树 A/B（evidence/logs/s244-ab.log vs s243-ab.log）：见下节。

## 全树 A/B（evidence/logs/s244-ab.log vs s243-ab.log）

- 行级 7709P/506F/81T → 7709P/506F/81T **零位移**；(key,state) 对（expected
  尾截断 + LC_ALL=C）7832/7832 join 全闭合，LOST 0 / GAINED 0——**第四个
  连续完美 A/B**（承 s241/s242/s243）。
- 工具注记：本轮 comm 分析首报 GAINED 1 经查为沙盒 shell 内联管道截断伪影
  （`awk|grep|head` 交互式管非确定截流，落盘材料化后复算闭合）——管道分析
  一律材料化到文件再 join（「落盘 tee 全量」教训族延伸到分析管道）。

## 残余挂账（本切不动）

- 快照写跨实例可见性边界（既有架构面，非本轮引入）：`_ensureMutTree` 物化
  写（disabled/defaultSelected attr 写）落**实例物化树**，fresh 查询重解析
  源 html 不可见（probe4 dfltSel.cross=false attr=false；selected 脏态经全局
  表跨实例可见——两通道可见性差异已注记）；s242 三联写同边界。
- 快照 SELECT options 集合面 undefined（probe1 复认，s242 账外邻接面）。
- plain `_zwMEl` option（foreign-doc 工厂产物）邻接面未勘（probe1 复认零
  语料钉面）。
- wrapper 动态建 select 的 index 位次（probe1 mElInSelect2=0 应 1——语料
  option-index 全静态 markup 面，动态臂挂账）。
- s241 存量：同类直写臂同步清单 meter/progress.max/FRAMESET、
  `_zwOptionValue` 键派生不一致；s240 存量：plain detached select.value
  undefined、option.text null→'' 字面差。

## 下一步候选

img SVG 密度族（naturalWidth 22F + current-pixel-density 13F，独立勘根轮）；
customizable-select 177F（挂起待用户拍板）；UA modal dialog 居中 sheet 缺位
（渲染流车道，s243 地面真值已移交）。
