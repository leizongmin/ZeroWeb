# s113：script type/language essence 16 集合——动态面统一门 + 解析面收窄

日期：2026-10-27。目标域：`scripting-1` script-type-and-language-js（s112 解锁后
227F 真实缺口）。三文件轮：engine `extract.rs` + shim `part03.js`/`part04.js`。

## 入场定谳（探针分组）

s112 解锁页 456 子测 = 229P/227F。227 失败全为「shouldn't run」判据：type 臂 53
（bare text[] 16 / legacy 1.6-1.9 / `\0` 沾染 / type=" "）+ language 臂 173
（前后置空白 / legacy 1.6-1.9 / `\0` / xyz 拼接），加解析插入面 1（type="javascript"
被旧 `ends_with("javascript")` 误判 JS 跑了）。四动态执行臂门况盘点：
`_zwScriptMaybeStartDynamic` 旧 `indexOf('javascript')` 子串 lax 门、R387 三执行臂
/ s95 children-changed / `_zwPrepareSrcScriptGate` **全无 type 门**。

## spec 模型（/tmp/spec-scripting.html 提取，2026-10-27 live 版）

- **type string 推导**：type 字面空串 / 无 type 且 language 字面空串（或双缺）→
  `"text/javascript"`；type 在场取原值；否则 `"text/" + language` **原值**（不剥白）。
- **分类**：type string essence match（HTML §scripting languages legacy 全表 16 项，
  ASCII-ci **等值**——无参数语法，`text/javascript;charset=UTF-8` 不中，
  with-params 语料判据）→ classic；或 type 属性在场时**第二替代**：属性值剥边缘
  ASCII ws 后 essence match（language 推导无此替代——`" javascript"` 推导串含空格
  不中，语料 language 前后置空白不跑）；module（ASCII-ci）→ module；余 data block
  不执行。

## 修复

1. **extract.rs `script_type_is_javascript` 收窄**：16 集合等值表
   （`JS_MIME_ESSENCE_TYPES`）；字面空串与剥后空串分轨（`type=" "` → data block，
   语料 testTypeShouldNotRun(" ")）；module 既有剥白口径不动（s93 钉保持）。
   解析投递面与 runner inline 面两调用方自动同表。
2. **shim `_zwScriptTypeClassify`**（part03，globalThis 导出）：推导 + essence 判定
   完整实现；blocked 门接入四臂——`_zwScriptMaybeStartDynamic`（换旧 lax 门，module
   行保留）、R387 三执行臂（`_r387typ` 并入三条件）、s95 children-changed、
   `_zwPrepareSrcScriptGate`（nomodule 门后）。blocked 不执行不置 run-once
   （摘属性重走门，nomodule 同口径）；module 型沿用各臂既有让位。
3. **中间态一乌龙**：首版分类器把 essence 判定统一做边缘剥白——language 推导串
   `"text/javascript "` 剥后中集合误跑（family 余 60F 全为 language 空白形）。
   对照 spec 文本修正为「type string 精确等值 + type 属性剥白第二替代」双轨后
   family 463P/0F。

## 验证

- engine lib **3564/0**（含新钉 `script_type_js_mime_essence_set_s113`）；webview
  **814/0**；runner bin **238/0**。
- family 定向：script-type-and-language 全家 **463P/0F**（主页面 456 子测全绿，
  with-params/whitespace/empty/xhtml 面保持）。
- **全量 testharness 套件 A/B**（stash 前/后二进制同协议各一遍，test-guard 包裹）：
  前基线 6319P/1448F/111T+22N（/tmp/s113-full-base.log 存档）→ 后
  6546P/1221F/111T+22N（/tmp/s113-full-suite.log）——7900 子测行数恒等，归一
  per-page diff **唯一变化** script-type-and-language-js 227F+229P→456P，
  **全树零 P→非P**。scripting-1 全子树 638P/367F → 865P/140F。
- fmt + clippy（zero-engine / zero-wpt-runner --all-targets -D warnings）过。
  渲染面零依赖——reftest/product-smoke 未跑。

## 挂账

- 解析面 language 属性未建模（Rust extract 无语料需求；spec 推导式与动态面同，
  遇语料再补）。
- shim module 判定仍剥白口径（spec 为 type string 精确等值——`" module "` spec 是
  data block，引擎现行 module/挪白；无语料判据，不动）。
- script-type-and-language-js-svg.svg / -xhtml.xhtml 两变体不在 runner 枚举面
  （html-only），SVG language 豁免面（testLanguageShouldNotRunUnlessSVG）未验证。
- 其余承袭：096 write(defer) / 142 xlink:href / 143 eventOrder（write 面族）、
  063 Timeout、non-external-no-import（module async）、parse-error 族等原挂账不变。
