# module 切片 R1 前置定谳——TT script 门位置＝gate-before-transpile＋policy 输出替换源文本（判据体实证）

日期：2026-11-27 · 权威语料：`tests/wpt-runner/wpt-data/trusted-types/script-enforcement-{006,007,008.https,009.https,010}.html`（判据体逐页 cat，方向词反向页已按体定谳）· 本文档裁决 [立项评估](2026-11-27-scatter-census-and-module-slice-assessment.md) §2 遗留问题「门作用原始文本 vs 转换后文本」，并固化 R1 落码形状。

## 定谳结论（三条）

1. **门作用于原始源文本；default policy 返回值替换有效源文本**——后续一切消费面（module 体转译执行、importmap JSON 解析、inline CSP check）全部吃 policy 返回值。证据：006 module 臂 policy 按**字面标记串**（`case "module": return "window.log_message('MODULE');"`）分派入参＝收到原始文本；返回值执行后 log MODULE＝输出即模块体。gate-after-transpile 被证伪（policy 收到的会是转译文本，字面 switch 落空）。
2. **门在 prepare 早期、有效类型分派之前；policy 可 mutate 元素 type 属性且改动生效**——010/011 体（名与体反向）：policy 回调内 `removeAttribute('type')`（module 元素→classic 执行）／`setAttribute('type','module')`（classic 元素→module 执行），断言按**改后类型**走（messages 以 CLASSIC/MODULE 为准）。R1 落码须在门后**重读**元素 type 再分派执行路径。
3. **importmap 型脚本＝门→返回 JSON 即活动 import map→后续动态 import() 经该 map 解析**——006 importmap 臂返回 `{"imports": {...}}` 后 `await import(...?message=UNMAPPED)` log IMPORTMAP＝动态 import 走 policy 安装的 map；two-importmaps 臂两次 CREATE_SCRIPT＋顺序 INSTALL＝每次 importmap 脚本独立重装/合并 map。

## 12F 面分解（006-009 各 3F 同形）

面名形态＝`Untrusted {HTML|SVG}ScriptElement of {importmap|module|two importmap(s)} type uses the source text returned by the default policy (…for inline CSP check)?`：

- 006（HTMLScriptElement，dispatch 变体）×3＋007（SVGScriptElement，dispatch 变体）×3：policy 输出直接执行/装 map。
- 008.https（HTML）×3＋009.https（SVG）×3：同断言序列，差异＝inline CSP check 以 policy 返回文本为对象（页内无 .headers 侧车，CSP 仿真面在页内——落码时按 008 页复核其 check 注入点）。
- classic 同名面已过（刀㊷ 门覆盖 classic）；010/011 module 臂（各 1F）＝上述第 2 条的 type-switch 语义。

## R1 落码形状（依本定谳）

1. shim script prepare：TT enforced 页对 `type=module`/`type=importmap` 元素（含 SVG 变体）接 classic 同款 `__zwTtSinkValue('TrustedScript', rawText, sink)` 门——sink 串 `HTMLScriptElement text`/`SVGScriptElement text`（007 页 policy 直 log sink 实证）；现 shim part03 非 classic MIME 早退面（`ty.indexOf('javascript')<0 && ty.indexOf('ecmascript')<0 → return`）为接入点邻域。门后重读 type（010/011 语义）。
2. policy 输出消费三线：module 体（输出→module 转译/执行管线输入）、importmap（输出→map registry 装/合并）、inline CSP check（输出→check 对象，008/009 面）。
3. 配套依赖（不可拆）：**importmap registry＋动态 import() 解析链**（runner 挂钩域现缺 `__zw_compile_module`/`__zw_compile_inline_module` 注册，见评估文档 §2 缺口 2/3/4）——importmap 六臂（006/007/008/009 各含 1-2 臂）无此链不成。module 两臂（006/007 module face）可先试＝依赖最小集。
4. corpus 支撑面已有：`support/logMessage-module.sub.js`＋`script_messages_for`（页 Fail 非 harness-error 实证）。

## 切片影响（对评估文档 §2 建议的修订）

R1 从「module exec 过 TT 门」修订为「**prepare 期门扩展＋输出替换＋importmap/动态 import 最小链**」一体刀位（门与消费链耦合，拆开则 importmap 六臂必败）；R2/R3 并入 R1 后余量为 010/011 type-switch 面＋8 臂 importmap 时序面（two-importmaps 顺序重装）分批。工时形态不变：单独立项、电池 A/B 锚定推进。
