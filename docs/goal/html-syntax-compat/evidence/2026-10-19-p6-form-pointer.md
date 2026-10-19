# P6 修复——fragment-parse form-pointer 边界（2026-10-19）

## 结果

`fragment-parse-form-in-template-001.tentative.html` **1F → 1P 翻绿**，全通道
**61228/61333 = 99.83%**（P5 冻结 61227 + 1）。**账目修正**：该案非 post-M4
漂移回归，而是 **M4 冻结起既有 1F**——M4 收口证据「template.content.querySelectorAll
补面（ParentNode 面缺）翻绿」记述不实：补面只消除 `'not a function'` 报错，
`inputs[0].form` 断言自补面起一直红（P5 轮「M4e 曾翻绿」系对该记述的错误推断，
本轮 bisect + 机制探针证伪）。

## 归因（bisect 结论无效 → 机制探针定案）

1. git bisect（f54ab9285 good 端 → b2fde4965）落点 376019d30 为纯 docs commit
   ——落点无效。复核发现 good 端 f54ab9285 本身折入了 part04 +29 的 QSA 补面
   代码，而其紧邻后续（376019d30）单案即红（同一 `expected null but got
   Element node <form>` 签名）→ 该案在「修复」commit 上即红，bisect 前提不成立。
2. 临时探针单测逐层定位：`inputs[0]` 为 content 视图 plain 节点，
   `__zw_parent` 解析为 EMPTY（`_ancestorChain` 断在 input），但
   `Object.getOwnPropertyDescriptor(i0, 'form')` 为 **自有 ACCESSOR**——
   part03 get trap 的 `.form` getter（R5009 片 e TEMPLATE 门所在）被自有属性
   遮蔽，门从未参与该形态。
3. 元凶：`_zwMEl` 工厂自有 `form` accessor（part03.js，2026-08-15 d6c31b30d
   引入，html5test 探针面）——祖先链 `while (root.parentNode)` 与
   `for (ancestor = node.parentNode; ...)` 上行不识别 template content 边界：
   inline content 视图把解析子的 parentNode 指到模板 proxy（视图捷径非真实
   祖先），上行逃逸进宿主文档命中宿主 `<form>`。

## 修复（R5196）

`crates/engine/src/js_dom_shim/part03.js` 自有 `form` accessor：

- 祖先链上行：命中 TEMPLATE 元素即断（R5009 片 e 门同序镜像）——宿主 form
  不越界关联；content 树内真实 form 祖先（[input, FORM, TEMPLATE]）先于
  TEMPLATE 命中，关联不受影响。
- form= 属性关联 root 上行：停在 TEMPLATE 本身——同 content 树内 id 匹配
  仍关联（spec form owner 树内查找），宿主侧 form id 不可见。
- spec 依据：template contents 是独立树，fragment 解析不设 form element
  pointer（WHATWG#12257 / chromium 490290430）。

## 常驻测试

- `test_template_content_form_owner_boundary_r5196`（js_dom_bridge_tests/part21.rs）
  三面：WPT 形态 `inputs[0].form === null` + 同树 `form="cf"` 关联保持 +
  跨树 `form="hostform"` 返 null。
- WPT 案本体（通道常驻）：单案复跑 Pass，全通道 +1P。

## 门禁

- fmt clean；guarded clippy（workspace `-D warnings`）干净；
- 全通道 61228/61333（分母/子测形态与 P5 冻结一致，escaping div.innerHTML
  1F 维持——独立机制，见 P5 残差账）。

## 账目影响

- P5 残差账「vs-M4 漂移：form-pointer 1」行**关闭**（改记：M4 冻结起既有
  1F，P6 修复）；vs-M4 漂移余 **escaping noscript 1F**（M3a/M4 证据实记
  9/9——真实 post-M4 漂移，待归因）+ script-inserted 双案（rendering-compat
  域内）。
