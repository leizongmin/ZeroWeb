# M2-S198：HTMLFieldSetElement type IDL 常量——'fieldset' getter 臂

**日期**：2026-11-01
**入场**：s197 在册次席候选 HTMLFieldSetElement `type` 1F（the-fieldset-element/
HTMLFieldSetElement.html ::「The type attribute must return 'fieldset'」——
`fs_outer.type` 断言恒 'fieldset'）。customizable select 177F 挂起待拍板
（GB-20261031 征询在途，用户决策门禁不自行扩面）。
**结果**：**1F 翻绿，the-fieldset-element 目录 22P/0F 全绿**（s193-s197 切片
累积收口后本目录首个零失败终态）。

## 勘根

- `fs_outer.type`（静态 fieldset，无 type 内容属性）走 part04 get trap
  `prop === 'type'` 分支（R3189 枚举反射点）：OUTPUT/TEXTAREA 有 IDL 常量
  臂（'output'/'textarea'，R3189/s128），FIELDSET 无 → 落 R3037 generic
  字符串反射读 `type` 内容属性 → 恒 ''。spec（HTML §4.10.4
  #dom-fieldset-type）：`readonly attribute DOMString type`，getter 步骤
  恒返 "fieldset"（IDL 常量，与内容属性无关）。
- 解析节点面（M-El plain object）无对应臂（part03 无 FIELDSET type
  accessor）——OUTPUT/TEXTAREA 同面共享此限（既有缺口非本刀引入，语料
  无该面断言，挂账不扩刀）。

## 施工（1 文件 shim，净 +5）

- `part04.js` get trap `type` 分支：`_realTag(sel, handle) === 'FIELDSET'`
  → return 'fieldset'，OUTPUT/TEXTAREA 同型臂位（枚举反射点内、
  `_reflectedTypeEnum` 与 generic 反射之前）。静态面 getter。

## 验证

- shim 全序拼接 `node --check` OK。
- 靶面：the-fieldset-element 目录 **22P/0F/0T 全绿**（修复前 21P/1F，
  唯一 F 即本刀靶键；disabled-003 s197 双案保持 P）。
- **stash 隔离基线 vs WIP 对照（s195-s197 同法）**：
  - forms 全树：基线 **4447P/249F/12T** → WIP **4452P/246F/11T**。
    keyed diff（(key,state) 对 sort -u 去重）**F→P 恰 1 键 = 靶键**
    （HTMLFieldSetElement type）；**P2F 1 键** = button/active-onblur
    spacebar（`:active` onblur flake 域在册面，s196/s197 双向摆动样本
    +1）；focus-dynamic-type-change-on-blur 文件级 Timeout 键 ↔ 5 子测
    Pass 键漂移（同 flake 域 onblur 族负载敏感，wip 侧全 Pass 纯有利）；
    customizable-select optional 文件行形漂移（file 键/子测键两种报法
    均 Fail——挂起待拍板域报法漂移非状态位移）。
  - interactive-elements：基线与 WIP 均 **439P/44F/0T bit-exact**
    （零键位移）。
  - lib：zero-engine **3642P/1F**（在册 s166 失败同键）+ zero-dom
    **880P/0F**，stash 基线逐位一致。
- fmt 过（Rust 面零改动）+ guarded-clippy workspace `-D warnings` 过。
- 过程注记：首轮 interactive+lib 链误序（stash 未弹即排 WIP 段）启动即
  杀弃重跑，垃圾部分日志已删；并行流（rendering reftest-upstream +
  ZeroSeed 安全门）负载在途——flake 域位移与此一致。

## 残留/挂账

- M-El 解析节点面 FIELDSET/OUTPUT/TEXTAREA type IDL 常量无臂（语料无
  断言面，OUTPUT/TEXTAREA 共享既有限）。
- `type` readonly setter 面未建模（generic 反射 setter 可写 type 内容
  属性——getter 恒常量后写不可见；语料无形态）。
- `:active` onblur flake 域（s196 在册，本轮 +2 键样本：P2F 1 + 键形
  漂移 1 文件）。
- customizable select 177F 挂起待拍板（GB-20261031 征询在途）。

## 下一步候选

customizable select 177F 挂起待拍板（GB-20261031）→ s196 挂账序（父
label 零矩形 layout 缺口 2F、inert-svg-hittest 第 2 子测、`:not(:active)`
否定面、Space 激活行为 checkedness 翻转）。

## 日志

`s198-fieldset-dir.log` / `s198-forms-stash-baseline.log` /
`s198-forms-wip.log` / `s198-interactive-stash-baseline.log` /
`s198-interactive-wip.log` / `s198-lib-stash-baseline.log` /
`s198-lib-wip.log` / `s198-lib-dom-stash-baseline.log` /
`s198-lib-dom-wip.log` / `s198-clippy.log`（同目录）。
