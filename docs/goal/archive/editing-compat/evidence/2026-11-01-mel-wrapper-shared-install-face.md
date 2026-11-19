# editing-compat —「wrapper 安装面」深水轮：`_zwMEl` 逐实例安装面共享化（2026-11-01）

承接 [插入侧解析残余归因轮](2026-11-01-insert-parse-construction-attribution.md)
（seg55 定谳 = wrapper 逐实例安装面）与 [CE 环 details 全树扫成员资格快路轮](2026-11-01-details-scan-membership-fast-path.md)
（R4100 census 闩、zz-s5 中位 107ms 基准）。本轮交付：`_zwMEl(snap, parent)`
构造器（part03.js，原 ~2,350 行逐实例安装、每元素 ~100+ 闭包）的**非 tag 门控
成员**提升为模块级共享函数/描述符，zz-s5 墙钟中位 **107→68ms（−36%）**，
七语料 + 双 lib + bench + web-components + html-semantics 门禁零位移。
seg55（bulk fill 解析构建）与 seg54（t8r 物化 `_wcRebuildAsHandle`）同源
（`document.createElement` 内部同调 `_zwMEl`），单轮覆盖两侧。

## 1. 转换模式与覆盖面

两种等价形态，体内既有闭包捕获局部 `node` 的语义逐位保留：

- **共享函数**：方法体逐字搬移到模块级，前缀 `var node = this;`。构造器
  安装点换 `node.foo = _zwMElFoo` 共享引用。
- **共享描述符**：getter/setter 提取为模块级函数后组共享描述符对象传
  `Object.defineProperty`——own-property 可见面（gPN/hOP/gOPD 三查）逐位
  保留，只换函数身份。

转换经临时脚本 73 items 落地（44 引用替换 + 9 体搬移 + 4 getter 提取 +
16 显式改写），diff +2,017/−1,783。sanctioned 重写仅四类：`_r117MVal(` →
`_zwMElR117MVal(node, `（R117 内联校验函数上移带 node 出参）、移入体尾钩
补 `node` 参数、`attrs` → `this[_zwAttrsSlotKey]`、事件监听存储
`this._zwMEvLs || (this._zwMEvLs = [])` 惰性建（R167 `_zwMEvLs` 双引用
审计项，原为构造器局部数组）。

层级落点（part03.js）：共享函数层 ~18071 起（`_zwAttrsSlotKey` 键常量、
getAttribute 族、事件族、几何/containment 族、R117、mutator 族、
`_zwMCollectControls`、validity 族、classList `_zwCL*` 共享族 +
`_zwMClassListFor(node)` 工厂、dataset 共享 trap 族 + `_zwMDatasetFor(node)`）；
描述符层 ~19738-19983（reflect str/bool 族、outerHTML/innerText 族、
NamedNode 族、id/className、slot 族、`_zwIdlSetNoop`）；移入层 13 个
（replaceChild/insertBefore/appendChild/removeChild/replaceWith/focus/click/
dispatchEvent/validationMessage/willValidate/validity/attributes/`_zwMAttrNS`）；
构造器 ~20003 起只余 tag 门控安装点 + 槽位初始化。

## 2. 等价边审计（逐条定案）

- **分离调用**：共享函数脱离实例调用变 TypeError——对齐真实 DOM
  Illegal invocation 正语义（原逐实例闭包无此约束，无仓内消费面依赖旧态）。
- **描述符 getter `.call(他节点)`**：读他节点（WebIDL 正语义），与原逐实例
  getter 只能读自身相比是放宽而非收窄，消费面（part01/04/05 grep）无依赖。
- **id/className getter 改读 attrs 数组**：R307 镜像属性退役，attrs 数组
  唯一权威；setter 走 `setAttribute` 不变。
- **ownerDocument 写旗 `_odWritten167` 删**：改判 `_zwOdOwn !== undefined`
  等价（写旗唯一用途 = 区分「已落 own 槽」，槽存在性判定同义）。
- **新增非枚举槽**：`__zwAttrsArr`（元素→attrs 数组）、`__zwOwnerEl`
  （attrs 数组→元素）、`__zwOwnerNode`（classList/dataset→目标元素）。
  gPN 各 +1 键——own enumerable 不可见（`enumerable: false`），无
  exact-gPN 计数消费面（part01-08 grep 定案）。
- **`_zwIdlSet` → 共享 no-op `_zwIdlSetNoop`**：保留属性存在性
  （`_zwMReflectIdl` 以 `if (node._zwIdlSet)` 走 skip 分支），防
  removeAttribute 后 IDL setter 回写复活属性——探针实测
  removeAttribute('id')/'class' 后属性不复活、随后 IDL setter 正常反射回
  attrs、classList 写照常落 attrs ✓。
- **tag 门控成员保留逐实例闭包**：a/area href、iframe
  contentDocument/contentWindow、input value、template content、媒体族
  （play/pause/load/canPlayType/paused/currentTime/duration/src）、dialog、
  img decode——门控条件依赖构造时局部 snap/tag 判定，共享化无收益。
- **Proxy trap 首参是 target 非 this**：dataset 共享 trap
  `_zwDatasetOwnKeys(_t)` 显式读 `_t.__zwOwnerNode`（this 到不了 target）。

## 3. 验证

- **语法门**：parts 01,01b,02,03,04,05,06,07,08 顺序拼接 `node --check` PASS。
- **体逐字比对**：13 个移入体 + 7 个提取体与 pre-transform 快照逐行
  content-only 比对全 OK（仅 sanctioned 重写差异）。
- **安装点目检**：构造器内剩余闭包逐个核对，全部 tag 门控。
- **行为探针 A/B**（zz-probe-s5-insert，净树墙钟）：
  - zz-s5（fresh div innerHTML= 2000 span，解析构建面 seg55）：中位
    107→**68ms（−36%）**，转换后两轮三样本 91/68/58 与 91/69/60；
  - zz-reset（容器连续重设 innerHTML=，移除+插入复合 S8 形）：~85→**30ms**；
  - zz-ap（appendChild×250，mutation 面）：44ms **恒等**（零回归）；
  - 与归因轮估算 ~20µs→~7µs/元素一致。
- **等价边行为探针**（zz-probe-mel-shared，4 案全绿）：id/class
  removeAttribute 不复活 + IDL setter 反射回 attrs、classList 写落 attrs、
  共享槽不进 gPN own enumerable（`__zw*` 零键）、共享事件族 add/dispatch
  逐位；title 反射读写通道内恒可达。
- **own 面通道不变量定谳**：runner 通道（testharness-editing-run-*）页面
  元素为 handle-proxy——`Object.getOwnPropertyNames(span)` 仅
  `_zwPaGen45/_zwPaId45` 章、gOPD/hOP own 面 undefined/false。pre/post
  transform 双二进制同探针**同读数**（换入 pre-transform 快照重编译复跑
  实证），该通道 own 面观感跨转换恒等；plain world own 面
  （`__zwAttrsArr` 等槽）由静态审计 §2 定案（非枚举、无 exact-gPN 消费面）。

## 4. 门禁（十面，systemd 单串行批，净树）

| 面 | 本轮 | 基线 | 判定 |
|---|---|---|---|
| join-different-white-space-style | 7,518P/0F/0T | 同 | 恒等 |
| insertparagraph-in-inline-editing-host | 880P/0F | 同 | 恒等 |
| insertparagraph-with-white-space-style | 1,104P/0F | 同 | 恒等 |
| insertlinebreak-with-white-space-style | 552P/0F | 同 | 恒等 |
| insertparagraph-in-editing-host-cannot-have-div | 420P/0F | 同 | 恒等 |
| insert-paragraph-in-void-element | 72P/0F | 同 | 恒等 |
| insert-text-in-void-element | 48P/0F | 同 | 恒等 |
| engine lib | 3,642P/1F | 3,642P/1F | s24 已知红唯一，红名 identical |
| webview lib | 834P/0F | 834P/0F | 恒等 |
| bench-gate（zero-engine 定向） | PASS | PASS | 恒等 |
| web-components | 4,022P/1,146F/5T | 同 | 计数级恒等 |
| html-semantics（语料续拉后） | 6,869P/695F/163T | 5,170P/558F（上轮，语料缺面） | 语料增长，见下 |

运行注记：`make test` workspace 腿 cargo fail-fast 止于 engine lib 已知红，
webview 腿（及 renderer/browser/quickjs 腿）未达——webview lib 单独
test-guard 补跑对账 834P/0F；后续轮读 `make test` 日志时按此口径。

**html-semantics ⑥ 销账**：`the-details-element` 语料 21 案续拉补入，
**42P/1F**——唯一 Fail = `closed-details-layout-apis.tentative`
（content-visibility 布局 API，engine 语义面之外），R4100 details 语义
WPT 面无红灯。全量 Fail 分布与上轮同簇（img 解码/customizable-select/
script execution-timing/textarea-wrapping）+ 新补语料面
（the-dialog-element 43F 等），无 shim 回归形态；语料增长
（interactive-elements 222 案等续拉入库）故总数与上轮不可直接恒等比对。

## 5. 残余地图（不变项）

- **appendChild trap glue**（seg20 ~85µs/append，guideline #35 整体替换）
  与 **HCLiveInvalidate**（seg24，R55/R304 重审）——各自独立整轮，本轮不动。
- 移动面 S6/S7 ~10µs/调用残余记档（次要，懒投影无 census 覆盖）。
- 构造器内 tag 门控族（~15 成员×媒体族）不再压缩——逐实例闭包为门控
  语义所需，非本面残余。
