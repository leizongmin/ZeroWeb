# ua_display_li D4 bisect 定谳补充（ZRG 2026-10-31 巡检轮）

对 `test_ua_display_li_list_item_canvas_inline_s24_flip`（part32.rs:996，make test
js_dom_bridge 既有红，R5368 同轮亦遇）做 git bisect 归因与机制定位，补充 D4 挂账
的引入提交证据。归属与收口路径不变：渲染流 D4 待裁决（webcrypto D2 征询在案）。

## 现象与钉位

- 断言 `getComputedStyle(li).display` 期望 `'list-item'`（spec UA sheet
  `li { display: list-item }`），实测 `'block'`。slice25 翻转钉（af677d7f0 时代），
  pin shim `_zwUaDisplay` li 专支。
- 65de5f28e / 42e662bf5 / 650eab54e 三腿手工复现 FAIL；f4db50f0d（10-26 轮）
  make test 全绿。

## bisect（区间 650eab54e ← f4db50f0d，11 腿）

**first bad = ee2dc953b**（10-30，s161 动态元素 gCS handle 视图通道 + 外链
media 门三层）。完整 log 见 ZRG 轮证据
`.rally/zeroweb-regression-guard/20261031T220448Z/ua-display-red-bisect.log`
（gitignore 不入库，结论摘录于此）。

## 机制：两套 UA display 模型分歧被 s161 通道暴露

- shim `_zwUaDisplay`（part01.js:10109）：li → `'list-item'`（spec/Chrome 准确，
  slice25 翻转钉锚定）。Chrome/154 实测 gCS(li).display = `list-item`。
- Rust StyleSystem 级联：li → `block`（style-system lib.rs:102 block 集）+ marker
  按 tag 名门控（lib.rs:930-935 自注「与 spec li{display:list-item} 的语义等价性
  待裁决」——即本 D4 残余账）。
- s161 前：动态 li（createElement 产物）无 host gCS 通道 → shim 查空落 UA 表 →
  `list-item`，钉绿。s161 后：`__zw_get_computed_style_by_handle`（handle 视图
  parse + 重放 + `compute_styles_for_doc` 真实级联）返回非空即短路 UA 表 →
  Rust 级联的 `block` 直接成为 gCS 读数，钉红。
- 该红因此是 s161 handle 通道把既存 Rust/shim 模型分歧**首次暴露**到动态元素
  gCS 读数面；非 s161 新引入模型分歧本身。

## 对裁决的输入

1. 统一终态指向 Rust 级联 ua_display li → `list-item`（与 shim 表、spec、Chrome
   对齐），marker tag 名门控与 `display:list-item` 语义解耦需一并裁决。
2. WPT corpus li 面与 layout marker 联动基线位移需在裁决时同轮 A/B。
3. 临时缓解选项（若裁决前需消红）：handle 通道 display 属性 miss 时回落 shim
   UA 表（保 author style 真值优先，仅 UA 层对齐 shim 模型）。

记录人：ZRG 2026-10-31 巡检轮（2026-11-01 落档）；证据同轮
`ua-display-red-attribution.md`（.rally，不入库）。
