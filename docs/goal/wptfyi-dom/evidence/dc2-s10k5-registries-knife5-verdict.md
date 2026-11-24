# DC-2 s10-registries 刀⑤（setHTMLUnsafe DSR 直通面 ＋ template contents 无 BC null registry 链）判读收账（2026-11-24）

k4 收账后新开轮（同 session 续）：残面序首两项——k4 判读档残面节「ShadowRoot-init-declarative
残 1F（setHTMLUnsafe unsafe 子树 face 链）＋ShadowRoot-init-customElementRegistry 残 1F
（template contents 宿主 null registry 链）」双双翻绿。

## 范围与根因（两残面探针定谳）

**残面 A**（ShadowRoot-init-declarative 残 1F「…parsed via setHTMLUnsafe should use
null registry」，探针失败形 `Cannot read properties of null (reading 'firstElementChild')`
＝`container.firstElementChild.shadowRoot` null）：setHTMLUnsafe trap 走 part13
`__zwSanSetHtml` 桥（`__zwSanSetHtml` 存在即接管，非 `_zwSetHtmlUnsafeImpl` 直调），
`_zwSanSetHtmlImpl` 在 **san=null**（safe=false 且 options 无 sanitizer——过滤恒等形态）
仍走 `serialize`+reparse 中转：mock 的 DSR 视图是**影子字段**（`__zwDeclarativeSR` 不进
light childNodes），序列化产出 `<div></div>`（template 消失形，探针 B 实证），reparse
后 DSR 整体缺席。engine 深臂（`__zw_parse_html_subtree_unsafe`）`sr`/`r` 键与视图链
（`_zwAttachDeclarativeSRView`）探针 D 实证完好——断点纯在中转有损。

**残面 B**（ShadowRoot-init-customElementRegistry 残 1F test12，探针失败形
`expected null but got object`）：三链——① `template.content.cloneNode(true)` 克隆宿主
（R145 `_tplContent.cloneNode` plain 子走 `_wcRebuildAsHandle`＝document 重建→归属主
文档）→ registry 解析链尽 fallback `_zwCeDocDefault(主文档)` → global registry；spec
template contents owner document 无 browsing context → 文档 registry null。
② attachShadow 显式 null 根克隆（clonable）：R5433 克隆臂 imperative 源只读
`_zwCeElRegSlots` WeakMap，而 explicit null 盖的是 wrapper **own 槽**
（`_srWrap._zwCERegistry = null`）→ slot undefined → 无操作 → 克隆根落 attachShadow
缺席默认（global）。③ 宿主再克隆印传播缺位。

## 修复面（五针，纯 JS shim：part13 ×1、part03 ×1、part04 ×3，零 Rust 行为面）

1. **part13 `_zwSanSetHtmlImpl` 无消毒直通臂**：san=null 时跳过 serialize+reparse
   中转，直通 `_zwSanBridge.setHtmlUnsafeImpl(sel, handle, str)`（过滤恒等——中转对
   DSR 影子字段有损；safe=true setHTML 恒 san 非 null 不进此臂，sanitizer-api 语料面
   零变化）。https://wicg.github.io/sanitizer-api/#dom-element-sethtmlunsafe
2. **part03 解析链 fallback 印判**：`_zwCeResolveElementRegistry` 链尽 fallback 前读
   `el._zwCeNoBc === true` → 返 null（无 BC 文档域印）。
   https://html.spec.whatwg.org/multipage/dom.html#template-contents-owner-document
3. **part04 R145 contents 克隆盖印**：`_tplContent.cloneNode` 两分支产物（trap 克隆
   产物 / `_wcRebuildAsHandle` 重建产物）盖 `_zwCeNoBc = true`。
4. **part04 trap 克隆印传播**：cloneNode trap 尾（R5433 臂后）源 own 印镜像到克隆
   产物（克隆不改 node document 归属，null 语义随宿主克隆）。
5. **part04 R5433 imperative own 兜底镜像**：slot undefined 时读源 wrapper own
   `_zwCERegistry`（非 undefined 即镜像——explicit null 形态），克隆根不再落缺席默认。

常驻钉测 **part151_s10k5_unsafe_dsr_nobc_pins.rs** 两针（同提交）：setHTMLUnsafe DSR
镜像子 registry null（全链：直通→视图→引擎物化→handle 重建→转登 decl 印）；contents
克隆 null 链五断言（宿主/attachShadow/SR 子/cloneHost/cloneSR/克隆 SR 子）。

## 字节绑定链

| 阶段 | md5 |
|---|---|
| 终态 runner（电池判定二进制） | `66dcecc1fbb505c27bf94d2721f79f14` |
| part13-sanitizer.js（直通臂） | 待补（commit 态） |
| part03.js（fallback 印判） | 待补（commit 态） |
| part04.js（三针） | 待补（commit 态） |
| part151 钉测 | 待补（commit 态） |
| js_dom_bridge_tests.rs | 待补（commit 态） |
| 电池发射器 | 待补（commit 态） |

前置钉测：`cargo test -p zero-engine --lib s10k`（单针协议沿 刀③/刀④）＝
test_s10k4_dsr_mirror_parent_pins / test_s10k5_unsafe_dsr_nobc_pins /
test_s10k2_dsr_contents_pins / test_s10k3_iframe_registry_pins **四代全绿 4 passed
0 failed**。拼接 29 分片 node --check SYNTAX_OK（9,972,384 字节）；fmt --all 零新
diff；clippy -p zero-engine 唯一 warning＝cssa_registry.rs:262 identical-if
（3e8cdfc12 css 流域既有，本刀域外不动，零新 warning）。

探针页 zz-k5-probe.html gitignored 刮擦面用后即删（A1/B1–B4 五断言全绿后删），
本钉测即常驻回归载体。

## 电池判读（锚＝已推 刀④ [dc2-s10k4-wc-post-knife4.log](dc2-s10k4-wc-post-knife4.log)；窗 2026-11-24 07:33:19Z–07:47:09Z solo 零并发 cargo）

**判读超预期**：needle 1 直通臂受益面远超两残面——凡经 `setHTMLUnsafe` 建声明式
fixture 的语料页全族受益（此前 serialize+reparse 有损中转丢弃 DSR 影子字段破坏
fixture，Fail 长期沉淀 2,775 面）。逐 (status,page) tally join（/tmp/k4_tally.py 同型）
零 unknown-drift，逐键归因如下。

| 腿 | 刀⑤ 终态 | 对已推 刀④ | 判读 |
|---|---|---|---|
| testharness-web-components | P 20,571 / F 1,184 / T 20 | P +2,436 / F −2,775 / T +1 | 两目标 registries 页精确兑现（ShadowRoot-init-declarative 1F→0F +1P、ShadowRoot-init-customElementRegistry 1F→0F +1P，双页绿）＋DSR fixture 族五页大解锁 F→P 全翻 2,775 面（reference-target property-reflection-idl-setters **2,685→0**F / 225→2,910P、property-reflection 16→1F / 2,895→2,910P——子测试总数恒等 2,910/2,911 纯翻面、declarative-shadow-dom-attachment 68→0F / 586→654P、basic 5→2F / 17→20P、shadow-root-clonable 2→0F / 4→6P）＝gethtml −339P +1T＝**页预算截断非回归**（`page timeout completed with 1 test(s) unreported`、state tests:6570；fixture 真实 DSR 物化后页变重触预算，k4 该页 6,914 全跑完，超时点前用例行为一致） |
| testharness-domparsing | P 1,264 / F 271 / T 1 | 全恒等 | SET-IDENTICAL（0 differing keys）——直通臂对 domparsing 面零扰动 |
| testharness-sanitizer-api | P 415 / F 341 / T 1 | P +1 / F −1 | html5lib-basics「Testcase #5, .innerHTML matches .setHTMLUnsafe」4F→3F——直通臂使 setHTMLUnsafe 与 innerHTML 同写链、DOM 恒等，机制自洽真实增益 |
| testharness-dom | P 57,869 / F 496 / T 11 | P −47 / F 恒等 | Range-mutations-dataChange 1,204→1,157＝已知截断噪声族（F 构成恒等而 P 漂移＝截断落位 ±45P 带），非回归 |

判读口径：行为面真实增益＝2,775 F 翻面＋SAN 1＝**2,776 面**；raw 口径 P 净
+2,436+1−47＝+2,390（双截断形 gethtml −339、Range −47 不计增损，仅留痕）。

## 字节绑定链

| 阶段 | md5 |
|---|---|
| 终态 runner（电池判定二进制） | `66dcecc1fbb505c27bf94d2721f79f14` |
| part13-sanitizer.js（直通臂） | `d0a53a36a4604b7e1fc3b56be1d65346` |
| part03.js（fallback 印判） | `551e9d4cc70003f1839b45a9b7a9b4c5` |
| part04.js（三针） | `6214d9a84f7ced45f9172354a98f3421` |
| part151 钉测 | `cbd256e6ec63c92b6d272359a2ba9f8e` |
| js_dom_bridge_tests.rs | `11135131b168baac519f02d8ee8995ff` |
| 电池发射器（.acceptance/dps 本地） | `e7d12b0b1f26b838268b698a3fc60ab1` |

工作树 diff 对账：HEAD→工作树恰 4 文件 +55/−3（part03 +7、part04 +39/−3、
part13 +9、tests.rs +3）＝五针本体零漂移；runner 由该树 release 构建。

## 证据日志（同提交 add -f）

| 日志 | md5 |
|---|---|
| [dc2-s10k5-wc-post-knife5.log](dc2-s10k5-wc-post-knife5.log) | `62ffba0bf7bf2be509639b2fe32083a4` |
| [dc2-s10k5-domparsing-post-knife5.log](dc2-s10k5-domparsing-post-knife5.log) | `492dc92343e774df4272e0e6a488e627` |
| [dc2-s10k5-sanitizer-post-knife5.log](dc2-s10k5-sanitizer-post-knife5.log) | `3e2fc2eb4651a8801a4535088f4345f7` |
| [dc2-s10k5-dom-post-knife5.log](dc2-s10k5-dom-post-knife5.log) | `366818cb1628b66f655faa4cb2c91365` |

电池窗：WC 5m10s / DP 12s / SAN 17s / DOM 8m11s（07:33:19Z–07:47:09Z），四腿 rc=1
仅留痕（F 存量），腿门＝tally join 零漂移。

## 残面（下刀挂账，序沿 master.md 下一步不变）

- 本刀新暴露残面：declarative-shadow-dom-basic 残 2F（Fragment parser basic test、
  template containing a template containing declarative shadow root）、
  property-reflection 残 1F（.labels 引用 label 元素）、html5lib-basics 残 3F
  （.setHTMLUnsafe 用例族，4→3）、gethtml 页预算截断观察位（页变重，跑批注意）。
- define.html 22F（`define()` 参数校验矩阵，`_ceDefine` 头部 arity+callable 顺序
  修正、prototype Get+IsObject、四 lifecycle callable+rethrow 已预研）、
  define-customized-builtins 1F（bgsound）、element-mutation 15F、R90 trap scoped
  原型臂 instanceof、parse-time attr 区分、importNode/upgrade 页、IAH
  handle-only——沿旧序。
- iframe 合成 doc 无 legacy `customElements` 读面（刀③ 钉测勘误实证）。
