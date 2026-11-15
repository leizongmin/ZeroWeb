# `target` expando 串化刀——插值 harness 全域解锁探针定谳

2026-11-15。driving: DC-3 masking 再分解（见
[dc3-snapshot-refresh §B′](2026-11-15-dc3-snapshot-refresh.md)）——22 页
setup 即崩的单一根因，探针升级为全域定谳。

## 1. 探针法（探针页留本地不入库——wpt-data 整目录 gitignore，fetch
会冲掉，源码在此内联可随时重建；zz- 前缀约定同 css-color caprobe 先例）

- `wpt-data/css/css-masking/animations/zz-mprobe-composition.html`：
  换装 `zz-interpolation-testcommon-probe.js`（P() 日志点）+ try/catch
  包首个 `test_composition`，诊断经必失败断言 message 回传 corpus JSON。
- `wpt-data/css/css-masking/animations/zz-mprobe-expando.html`：11 项
  expando 回读矩阵（原始类型/普通对象/元素引用/自引用/属性名 `target`/
  style.setProperty/gCS/数组/函数/template.content）。
- 重建要点（两页同构）：`<script src="/resources/testharness.js">` +
  testharnessreport + probe harness（mprobe-expando 不需要 harness）+
  body 内单个 `test(function(){...}, '名')`，结尾
  `assert_true(false, 'ZWEXPANDO|ZWPROBE ' + 诊断串)` 必失败回传。
- runner：`./target/test-guard --per-proc-mem 4 --total-mem 8
  --time-limit 300 -- ./target/release/zero-wpt-runner
  testharness-css-masking zz-mprobe`（既有二进制，零重建；二进制
  mtime 07:47 疑含 g50 WIP 片，根因面与其无交集，g50 落地后宜复跑）。

## 2. 定谳：expando 属性名 `target` 串化（对象值经 setAttribute 路径）

矩阵结果（zz-mprobe-expando 全文回传）：

| 用例 | 回读 | 判定 |
|---|---|---|
| `a.n1=42 / a.s1='xy' / a.b1=true` | number/string/boolean | ✓ |
| `a.o1={k:1}` → `a.o1.k` | number:1 | ✓ 普通对象正常 |
| `a.e1=b`（元素，任意名）→ `.tagName` / `typeof` | 'DIV' / 'object' | ✓ |
| `a.self=a` → `.tagName` | 'DIV' | ✓ 自引用正常 |
| **`a.target=b` → `typeof a.target`** | **'string'** | **✗ 唯 `target` 名串化** |
| style.setProperty / gCS 对象 / 数组 / 函数 | 全部正常 | ✓ |

crash 链：`createTargetContainer` 尾部 `targetContainer.target = target`
（interpolation-testcommon.js:277）→ 回读得 String → :373/:385/:387
（插值路径）与 :451/:453/:459（composition 路径）
`xxxTargetContainer.target.style` → String`.style`=undefined →
`setProperty` of undefined。真实浏览器 `el.target` 为普通 expando，
不串化。根因面：engine shim 元素属性 set 陷阱把**属性名集合**（`target`
为已知 HTML 属性名）的对象值路由 setAttribute → `String()` 化；
站点在 `crates/engine/src/js_dom_shim/`（include_str! 嵌入自
js_dom_bridge.rs:6219+，24 片中属性 set 陷阱待刀轮精确定位）。

## 3. 爆炸半径：337 页 / 56,365 上游子测（剩余缺口 ~47%）

wpt-data css 全域引用 `interpolation-testcommon.js` 的页面 **337 页**
（chrome summary 在册 335，上游子测合计 **56,365**；旧
interpolation-test.js 引用页为 0，无混杂）。两组路径（插值/composition）
都在 :277 set 后读回 → **全部 setup 即崩**（declared tests: 0）。
分布（页数）：css-transforms 32 / css-backgrounds 30 / motion 22 /
css-gaps 21 / css-masking 21 / css-values 18 / css-fonts 16 /
css-grid 16 / css-sizing 16 / css-text 12 / filter-effects 12 /
css-ui 11（余长尾）。对验收线余 ~119,496 占比 ~47%。

**口径诚实**：56,365 是这些页的上游子测分母——刀修的是「0 结果崩溃」
→「真实结果通道」；每页子测转真实 pass/fail 后的通过率未测（插值断言
依赖 engine 插值语义支持），转化率须 A/B 实测，此处不预记通过数。

## 4. 刀规格（下轮执行）

1. shim 属性 set 陷阱：属性名集合命中（`target` 等）时对象值改走
   expando 存储，仅原始值串化走 setAttribute；get 对称。
2. shim 为 include_str! 嵌入 → 改动必重建二进制；重建前核 git status
   干净（g50 WIP 落地后），重建后 provenance（mtime>commit）留档；
   在跑电池窗口冻结重建（shim include_str! 陈旧二进制毁 A/B 前科）。
3. 回归验证：zz-mprobe-expando 的 `name-target` 应转 object；
   masking 22 页 setup 应产出真实子测（不再 declared tests: 0）；
   随后按 §3 分布逐组 A/B（B0=刀前 results 目录）。
4. 附带发现（非本刀）：`Object.prototype.toString.call(div.style)`=
   `[object ]`（缺 Symbol.toStringTag 品牌面）；`t.content` 返回
   object 正常。
