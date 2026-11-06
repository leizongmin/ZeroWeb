# M2 s321 — ol start/reversed/type 列表标记值族勘根+施工：reversed 反序与 alpha 计数统一，6 红维持（ref 侧 s309 挡板），4 上游案转可信绿入常驻

日期：2026-11-07。切片号 s321（承 s320 下一步候选①：ol start/type/rev 列表标记值族勘根）。

## Solo 基线

`make reftest-upstream FILTER=grouping-ol`（六红页精确匹配）+ the-ol-element 全目录回看。
日志 [logs/s321-ol-initial.log](logs/s321-ol-initial.log)。六红（上游 corpus）：

- grouping-ol-rev-reftest-001 ✗1.03%
- grouping-ol-start-reftest-001 ✗2.67% / -002 ✗2.76%
- grouping-ol-type-reftest-001 ✗3.24% / -002 ✗3.85% / -003 ✗2.11%

## 勘根（复合根：ref 侧 s309 挡板 + test 侧本流两处真 bug）

### Ref 侧挡板：s309 span>p 消失面（渲染流域，六红全挡）

六红 ref 页统一用 `<span><p>N.</p></span>` 嵌套结构承载序号文本
（css/CSS2 counter 展示惯例）。s309 已勘定该结构在 ZW 渲染为空
（nested block-in-inline hoisting，root 在 layout-engine：tree.rs:4213 /
inline/mod.rs:1755 / collect_items.rs:770-844，渲染流 R109）。
**s309 账面「ZW test 侧正常」须修正**：test 侧同时有本流 bug（下述），
双侧错位部分抵消使 diff% 偏小（1%~4% 次阈值带），掩盖了六红中 test 侧
成分。修复 test 侧后六红 diff 反而微涨（如 type-002 3.85→3.93%）——
方向为逼近真值（ref 侧仍消失），非回归。

### Test 侧 bug ①：reversed 反序计数未实现（paint list_item_counter）

`list_item_counter`（paint/painter/text/text_list.rs ~:2143）旧实现恒
`counter = start or 1`、步进恒 +1，`reversed` 属性零读取。规范
（HTML rendering #lists-2）：

- 无 start：首个 li 序号 = li 计数（3 li → 3,2,1）
- 有 start：从 start 递减（start=5,3 li → 5,4,3）
- li `value` 属性仍钉住，其后自该值继续递减

### Test 侧 bug ②：alpha 计数越界回落 '?' 而非规范十进制

CSS Counter Styles 3：lower-alpha/upper-alpha 负值或越界回落 decimal
（spec fallback 语义）。旧实现三处漂移（guidelines #6 双路径漂移先科）：

- paint 臂（~:1622）：index>26 时输出 `'?'` 字面量（含句点）
- advance 臂（~:1900）：`LowerAlpha if index > 0 && index <= 26` 守卫，
  越界静默跳过
- catch-all 臂（~:2086）：LowerAlpha|UpperAlpha 落入 decimal 分支但
  上游已截留，路径不一致

`crate::text::text_list::format_counter_alpha`（s319 前后已实现
bijective base-26，27→"aa"、负值→decimal）是规范正确实现，paint/advance
两臂未复用。

## 施工（单文件 crates/engine/src/paint/painter/text/text_list.rs）

1. `list_item_counter` 反序化：读 `reversed` 属性，`step=-1`；默认起点
   无 start 时 reversed 取 li 直接子计数（is_li 过滤）、否则 1；
   `value` 钉住语义不变，循环步进改 `counter += step`。
2. alpha 统一：paint 臂删 `'?'` 回落，直接 `format_counter_alpha(index,
   upper)` + `"."`；advance 臂删 `index>0 && index<=26` 守卫；
   catch-all 臂移除 LowerAlpha|UpperAlpha（不可达残留）。三臂全部
   收敛到 format_counter_alpha 单实现（附 HTML #attr-ol-reversed /
   CSS Counter Styles 3 fallback 规范链接与 s321 FIXME 残余注记）。
3. 单测 4 个：reversed 默认递减（3,2,1）/ start 递减（5,4,3）/ value
   钉住后向下延续（10,4,3）/ format_counter_alpha 越界回落数列
   (a..z,aa,ab,AA,aaa,0,-3)。

## 验证

- 单测 4/4 通过（test-guard 包裹，`cargo test -p zero-engine --lib`）。
- 像素探针（product-smoke）：reversed 组 test 侧与 chrome 序号一致，
  grouping-ol-rev-reftest-001 test 侧从 '?' 序号变为 3,2,1。
- solo 前后（reftest-upstream reversed 族）：reversed-1a/1b/1c/2 四案
  0.00% **真通过-可信**（DC-14），reversed-1d（嵌套 div li）/1e
  （display:none li）维持**近似通过**（次阈值绿：ZW 计数器走 DOM 直接
  子遍历，未按 CSS counter scoping 排除非 li 兄弟/隐藏项——不在本切片
  范围，挂账渲染流 counter scoping 面）。
- 六红页 test 侧改善但 ref 侧 s309 挡板仍在，全案维持红（diff% 微涨，
  逼近真值方向）；**六红清除条件 = 渲染流修 s309 hoisting**（已账）。
- 4 上游案 `make import-wpt` 入常驻断言集账本（imported-tests.txt，
  与本修复同提交）：reversed-1a/1b/1c（vs reversed-1-ref）、reversed-2
  （vs reversed-2-ref）。机制注记：四案语料本就在册（fetch-wpt-data
  全树拉取，reftest-upstream 文件走查通道逐轮全量运行），导入=
  账本资产化钉形而非新增运行面；导入脚本对已存在 corpus 文件跳过
  （mtime 保持 2026-10-15 原值实证），字节未离 WPT_DATA_REF pin。
  DC-14 分类核对（提交前终跑）：reversed-1d/1e 维持近似通过不入账本。

### 全量 A/B（vs s318 基线 15994P/1026F）

新二进制 reftest-upstream 全量 17020 案 **15992P/1028F**（净 −2P/+2F）。
行级 join（LC_ALL=C，/tmp/s318-fails.txt vs /tmp/s321-fails.txt）：

- REGRESSED 4：table-anonymous-objects-130/148/153 + insert-inline-in-
  blocks-n-inlines-middle-001
- FIXED 2：insert-block-in-blocks-n-inlines-end-001 + table-anonymous-
  objects-146

按 s292 协议同二进制 solo 复跑定谳：130 solo 1/1 PASS；148/153/
insert-inline-middle **solo ×3 全 PASS**。同二进制跨跑状态翻转 = 时序
抖动，REGRESSED 与 FIXED 两侧互为镜像（table-anon 抖动族在册先科
R5418；insert-* 动态插入时序敏感）。**零确定性回归，净差为双侧抖动
噪声。**

## 门禁

- cargo fmt --all -- --check 干净。
- cargo clippy --workspace --all-targets -D warnings（guarded-clippy）
  干净（首跑暴露测试模块缺 `use crate::text::text_list::
  format_counter_alpha` 导入，已补）。
- make reftest 常驻内联集 725/725 全绿（该集为内联 Rust 表，
  与导入账本无交集；4 导入案的常驻运行面=reftest-upstream 全量
  通道，见上文 A/B）。
- make test 全量：见控制文档状态行（本切片提交前完成）。

## 下一步候选

1. svg-img-with-external-stylesheet / srcdoc-print 两散红勘根
   （embedded 4 中剩余 2）。
2. customizable-select ~30（forms tentative 高级面）评估立项价值。
3. （挂账承前）embed fallback-image（R57）、print pagination 7 键、
   range-intrinsic-size；counter scoping（reversed-1d/1e 次阈值绿根）
   待渲染流 counter 面立项时移交。
