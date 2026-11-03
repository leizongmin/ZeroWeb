# M2 s263 — data-zw-harness 注入序勘定轮：三子面全量量化零观察面，销案

日期：2026-11-03。切片号 s263（承 s262 下一步候选 #1「data-zw-harness
基建脚本注入序」= s255 D 刀收窄余项）。本切为**勘定轮**（勘根即结论，
零施工零代码刀）：对 runner 基建注入形（`prepare_harness_html` 就地替换
`/resources/testharness.js` 标签为 harness block，reporter 随包内联、
`testharnessreport.js` 标签移除）与上游双标签装载序的可观察差做全树
量化，三子面全部落零 → **销案**。data-zw-harness 保真族至此收口：
s255 D 刀（knife 剖形）→ s261 投递标记拆分（`data-zw-inlined`）→
s263 注入序量化。

## 勘根（三子面）

1. **① reporter 位置面（两标签间页面内容）**：上游序 = 页面可含
   testharness.js 与 testharnessreport.js 两标签且中间夹页面脚本
   （脚本可依赖装载时序）；现行把 reporter 并入 harness block 同点
   注入。全树 python os.walk 扫描 wpt-data html 树（2026-11-03 实测）：
   同时引用两标签的页面 **2759 页，两标签间夹内容 0 页** → 合并注入
   与双标签序在 corpus 上不可观察。
2. **② `has_harness_ref` 宽松 contains 判假阳性面**：runner 以
   substring 判页面是否 harness 页；reftest 树 20 页在注释里提及
   testharness 标签名（注释性引用，非真实装载）——查这些页是否进入
   runner testharness 枚举：0 页（且 s261-ab.log 全量基线 0 个 css/
   键——css 树本就不进 testharness 枚举）；另有 1 页空格形
   `src = `（带空格的属性写法）被 substring 命中——该页实测正常起跑，
   宽松命中未引入错装载。假阳性面 = 零观察面。
3. **③ 标签属性丢弃面**：harness block 就地替换会丢原 testharness.js
   标签上的属性；全树扫描命中 **1 页**，属性为
   `type="text/javascript"`（JS 默认值，丢弃无观察差）→ 无害。

## 结论与验证口径

- 三子面量化证据齐备，注入序差在 corpus 上**零可观察面** → 不施工、
  销案。s255 D 刀余项随本切清空。
- 验证口径：全树扫描为本切实测（python os.walk 单进程，2026-11-03）；
  单页起跑核验走 runner 常规通道。零代码变化 → 无 A/B 义务。

## 挂账更新

- data-zw-harness 保真族销案收口（s255 D 刀 → s261 拆分 → s263 量化）。
- 承前不动（与本切无涉）：microtask checkpoint 时序族（s255）；
  credentials / shared-dependency / 并发慢 import dedup（s256）；多张
  map merge + 动态插入（s262 架构级缓议）；customizable-select 177F
  （GB-20261031 挂起）；renderer SetDomSnapshot entries 化（s174）；
  engine lib 5 红（R-t2qq，归该流自查）。

## 下一步候选

1. charset 语料资产补拉（s264——css-module/text-module charset 家族
   5 页 file-Timeout 勘根为资产缺席，端点 s115/s249/s250 已落地）。
2. scripting-1 module 面勘根分型（s248 遗留候选）。
3. s255 候选清单其余。
