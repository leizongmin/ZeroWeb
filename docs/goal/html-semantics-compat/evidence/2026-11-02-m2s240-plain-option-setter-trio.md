# M2 s240 — plain option value/label/text 写三联补面（s239 FIXME 账，勘根推翻「缺省回落 text 写」假设）

日期：2026-11-02。切片号 s240（承 s239）。根：s239 挂账「option 三联 setter 面
（getter-only 安装）」——勘根两层产出：①语料**有**钉面（s239 账面「语料无钉面」
修正）；②s239 账面「value setter spec 缺省回落 text 写」假设被钉面推翻——
value/label setter = **纯内容属性反射**（attr 缺省仍写属性，无回落）。

## 入场定刀位

s239 下一步候选：option value/label/text setter 面（本切 FIXME 账延伸，s239
getter-only 安装的写侧）。

## 勘根（规范获取受限 → 语料钉面定谳；仪器：zz-s240-probe.html 探针，验后删）

- **规范面**：html.spec.whatwg.org 本环境不可达（WebFetch 域校验拒绝 +
  webReader MCP 抽取错误，同 s238 HTTPS 超时先例）→ 语义锚点改用本地语料：
  - `html/dom/reflection.js` customGetter 分支（option.value/label 在
    elements-forms.js 标 `{type:"string", customGetter:true}`）：注释
    「**reflected only on setting, not getting**」——domTests 清空但
    idlTests/idlDomExpected 保留 → **IDL set → getAttribute 断言照跑**，且在
    attr 缺省的 option 上跑 → setter 钉形 = 纯属性写（含
    'option.value IDL set undefined' → getAttribute "undefined"，R5009 注记
    既有钉面互证）。**s239 账面「value 缺省回落 text 写」推翻。**
  - `the-option-element/option-text-setter.html`：text setter 钉形 = replace
    all 单 Text 子（旧 Text 子 identity 更替、isConnected false、`.text=''`
    后 firstChild null）。
  - 身份路径三臂与钉形全部吻合：value=part04 R5009 族（14624 无条件属性写
    verbatim String）、label=part05 R3069-fix（显式列表写同名内容属性
    verbatim）、text=part04 R3907（委托 textContent）。part04 label 读侧
    注记「OPTION 专用分支缺省回落 text」经查只指 **get** 侧——set 侧无回落
    臂，全路径无矛盾。
- **探针形状勘定（五形状）**——「plain option」非单一形状，写面触达面各
  异：
  - **A/C（template content / live innerHTML 直写）**：产物为 handle-backed
    wrapper（`__zwHandle="__n{n}"` 本地发号，own 无 accessor，读经 trap）。
    set value/label 属性已写（getAttribute/`sel.value` 链读新值）但 IDL
    getter 读旧快照（"v1" vs attr "x2"）——**读/写源分裂**（写落属性层、
    IDL get 读 pending 快照层），身份路径既有缺陷，本切不动挂账；set text
    经 R3907 委托 textContent 全程新鲜（三子测形全对）。
  - **B/E（DOMParser 文档）**：`_zwMBuildBodyTree` 世界，`createElement`
    产物无 `_zwMEl` accessor（desc 无、getter undefined、IDL set 落 expando
    attr 不动）——s239 家族向 DOMParser 工厂的延伸面，本切不动挂账。
  - **D（detached doc createElement）**：真 `_zwMEl` 产物（s239 accessor 在
    own、getter 面 s239 已通）——**本切安装面**。before：三写全静默 no-op
    （getter-only accessor 非严格赋值吞掉——value 'x2'→读仍 "v1"、label
    'L2'→"l1"、text 'T2'→kids 不变）。

## 桥力学

- setter 三臂同 getter 的材化自适应口径：`__zwSelector/__zwHandle` 在场 →
  委托 `_makeProxy(sel, handle)` 对应 prop 写（路由进 trap 既有链——R5009/
  R3069/R3907 与 MO/CEReactions 记账全继承，零语义漂移、随身份链未来演进
  自适应）；双 null → plain 本地写：value/label `node.setAttribute(name,
  String(v))`（verbatim DOMString——null→"null"/undefined→"undefined"，与
  R5009 逐字钉形同口径）；text `node.textContent = v`（R310 replace-all 单
  Text 子、空串无子；null 经 LegacyNullToEmptyString 归 ''——与 R3907 原样
  委托一致，两态行为连续）。
- s239 FIXME 注记块重写（「语料无钉面」表述失实，按钉面事实改）。

## 施工（一处文件一块，零 rust）

part03 `_zwMEl` OPTION tag 门内三 accessor 各补 `set`（身份分流 +
plain 本地写），s240 注记 + HTML §4.10.9 setter 语义链。

## 验证

- node --check 全 shim concat（canonical 序）干净；browser-build 重建过。
- 探针 before/after（形状 D 九面）：value/label 写+读回环（v1→x2/l1→L2）、
  text replace-all 单 Text 子（first.data="T2"）、attr 缺省 value set 属性
  落（"y"、读 "y"，无 text 回落）、verbatim 转换（null→"null"、
  undefined→"undefined"）、空串 text 无子（kids 1→0）——全吻合；形状
  A/B/C/E 输出与 before 逐字节同（未触面零副作用）。
- solo the-option-element **108P/0F/0T**（R3907/s239 钉面零回归；首跑含
  探针文件 1F 噪声，删探针后为准）。
- 全树 A/B（evidence/logs/s240-ab.log vs s239-ab.log）：见下节。

## 全树 A/B（evidence/logs/s240-ab.log vs s239-ab.log）

- (key,state) 对 sort -u join：8293/8293 对，LOST 2 = GAINED 2，**位移唯一 =
  checkable-active-onblur-with-click「following label」checkbox/radio ×2
  F→P**——s238 记账的机器态时间漂移族本轮回摆（favorable 向）；改动臂严格
  `_zwMEl` OPTION accessor 门，checkbox/radio/label 路径零通道，定谳非回归。
- P→F 0、F→P 2（漂移族）、Timeout 集差 0。行级 7706P/509F/81T → 7708P/507F/
  81T（ΔP=+2、ΔF=−2 精确闭合 ±2）。
- 靶面无语料行（plain-setter 形状语料无钉面，s239 同形）——靶面验证由探针
  before/after 承担；写面活体消费者（模板/DOMParser 站点形）待 s239 家族
  延伸切片补 accessor 后才有语料触达面。

## 合并基线复核（收口轮）

push 前 `git pull --rebase` 并入并发流提交 7a713912a（webcrypto XOF：crypto.rs/
crypto_asym.rs + part01b/part02a，不触 part03 与 option/select 面）后重建
release runner 复跑 solo：the-option-element **108P/0F/0T** 与原验证逐行同态，
全 shim concat node --check 干净（part03 本地改动与上游 part01b/part02a 拼接
无冲突）。

## 残余挂账（本切不动，探针实证在案）

- **wrapper 读/写源分裂**：handle-backed pending 形（template content/
  live innerHTML）set value/label 落属性层但 IDL getter 读 pending 快照层
  （o.value='x2' 后读 "v1"、getAttribute 读 "x2"、sel.value 链读 "x2"）——
  s239/s236 读家族延伸候选。
- **DOMParser 文档面**：`_zwMBuildBodyTree` 世界 option 无 IDL accessor
  （getter undefined + set 落 expando attr 不动）——s239 家族延伸候选
  （独立勘根轮，覆盖该工厂 accessor 面）。
- **plain detached select.value undefined**：detached doc createElement
  select 的 value getter 未装（s186 FIXME「select/textarea/output 值模式
  族按语料需要再补」+ s236 select.value setter plain 匹配写链同族）。
- option.text = null → '' （经 textContent LegacyNullToEmptyString）vs spec
  DOMString null→"null" 的字面差异：语料无钉面、身份链 R3907 同形状（两态
  连续），挂账待钉面。

## 下一步候选

wrapper 读/写源分裂补面（s240 新账——写侧已通读侧 stale，语料无钉面探针
在案）；DOMParser 工厂 option accessor 面（s239 家族延伸）；modal-dialog-
selection（pointer 拖拽面存量）；customizable-select 177F（挂起待用户拍
板）；img SVG 密度族（naturalWidth 22F + current-pixel-density 13F，需独
立勘根轮）。
