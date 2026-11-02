# M2 s239 — plain option value/label/text 读死路补面（探针 undefined/''→spec 值）

日期：2026-11-02。切片号 s239（承 s238）。根：s236 挂账「plain option `t.value`
读值死路」的实态比账面更深——`_zwMEl` 解析工厂从未装 OPTION IDL accessor（直读
三联 undefined），`_zwOptionValue` 无 plain 臂（select.value 链读 ''）。

## 入场定刀位

s238 下一步候选三选一：plain value 死路补面（s236 显式挂账最小刀，选中）；
modal-dialog-selection（pointer 拖拽深车道，继续挂起）；checkable-active-onblur
漂移族观察——s238 全树已同形状复现（following×2+parent×2 Fail、spacebar 变体
Pass，跨二进制稳定）→ **记基线态销账**（当前纪元该 4 键基线红，后续位移再勘）。

## 勘根（仪器：zz-s239-probe.html 探针，验后删）

- **直读面（比账面更深）**：`s.innerHTML='<option value="v1" label="l1">First
  </option><option selected>Second</option>'` 后 `o1.value/o1.label/o1.text/
  o2.value/o2.text` 全 **undefined**（s236 账面「val:''」实为 s.value 链读数）。
  根 = `_zwMEl`（part03 解析工厂，innerHTML 直写/R304/DOMParser/template
  content 共用产物）只按 tag 装 href（a/area）/contentDocument（iframe）等少数
  accessor——OPTION IDL 三联从未安装：plain 节点 own 无属性 +
  HTMLOptionElement.prototype 无 accessor → undefined。
- **select 链面**：`s.value` = ''（应 "Second"）。`_zwSelectSelectedView` 经 s236
  已能返回 plain option wrapper，但 `_zwOptionValue(el)` 无 plain 臂——
  `_zwOptAttrNoNs(null,…)`→null、`_zwOptChildText(null,null)`→''。

## 桥力学

- **读分流双臂**：sel/handle 在场走 R3907 既有链（零改动），双 null 走本地
  getAttribute（存在即采纳含空串，无 ns 口径同 R122/getAttribute）+
  `_zwOptChildTextPlain`（childNodes 本地走查——`_zwOptCollectWalk` 既有 plain
  处理：nodeType 3/4 data、script 按 namespaceURI 排除、64 层帽、strip-collapse）。
- **材化自适应**：face B accessor 装 own accessor（`_zwMEl` tag 门）但读经共享
  helper（`_zwOptionValue`/身份臂提取 sel/handle 分流）——材化盖章
  （`__zwSelector`/`__zwHandle` 在场）后自动落身份臂，同一对象跨物化态读数连续。
- **setter 面挂账**：getter-only accessor（iframe contentDocument 同形）；spec
  value/label/text 均读写字面，语料无钉面，FIXME(s239) 记账待与 s228 车道交叉
  评估。

## 施工（一处文件两块，零 rust）

1. `_zwOptionValue` plain 臂 + `_zwOptChildTextPlain` helper（part03）。
2. `_zwMEl` OPTION 三联 accessor 安装（tag 小写归一判定；value→`_zwOptionValue`、
   label→身份臂 `_zwOptAttrNoNs(_zwOptKey(…))/plain 本地 getAttribute、
   text→身份臂 `_zwOptChildText`/plain 本地；注释挂 HTML §4.10.9 规范链 +
   FIXME setter 面）。

## 验证

- node --check 全 shim concat 干净。
- 探针 before/after：六面 undefined/''→spec 值全吻合（o1.value="v1"、
  o1.label="l1"、o1.text="First"、o2.value="Second"（child text 回落主面）、
  o2.text="Second"、s.value="Second"）。
- solo the-option-element **108P/0F/0T**（R3907 身份钉面零回归）；solo
  inserted-or-removed **5P/0F**（s236 钉面）；solo the-select-element/
  select-value **5P/0F**（select value 链身份钉面）。
- 全树 A/B（evidence/logs/s239-ab.log vs s238-ab.log，(key,state) 对 sort -u
  join）：**零状态翻转**——P→F 0、F→P 0、Timeout 集差 0（首跑含探针文件
  1F 噪声，删探针后重跑为准）。键集差唯一 = select-home-end-pagedown-
  pageup-detailed.optional.html 塌缩↔展开形翻转（s238 记账的并行负载形：s238
  跑 1 行文件级 Fail 键 → s239 跑 4 子测 Fail 键，净 +3 Fail 行精确闭合
  506→509，双侧同红零翻转）。行级 7706P/509F/81T（P/T 与 s238 持平）——
  s239 读面无语料钉面，零位移即预期形态，靶面验证由探针 before/after 承担。

## 残余挂账（本切不动，FIXME/注释已注记）

- option value/label/text **setter** 面（getter-only 安装；value setter spec
  语义缺省回落 text 写，语料无钉面）。
- select.value setter 对 plain option 的按值匹配写链（s236 同族账延伸）。
- 表单提交入口读 raw 非净化形（s236 已记同族账）。

## 下一步候选

option value/label/text setter 面（本切 FIXME 账，getter-only 安装的写侧
延伸，语料无钉面）；modal-dialog-selection（pointer 拖拽面存量，s236/s238 双
侧定谳基线）；customizable-select 特性面 177F（挂起待用户拍板）；img SVG
密度族（naturalWidth-naturalHeight 22F + current-pixel-density 13F，s218-225
密度车道的剩余面，需独立勘根轮）。
