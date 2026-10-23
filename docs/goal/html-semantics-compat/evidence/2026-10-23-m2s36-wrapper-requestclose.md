# M2 切片 36 — 挂账：impl-doc wrapper requestClose（2026-10-23）

## 范围

s22 起挂账（evidence/2026-10-22-m2s22-r4200.md）：dialog-requestclose-3
`dialog.requestClose is not a function`——`implementation.createHTMLDocument`
合成 doc 的 dialog 元素方法面缺 requestClose（R4200 实现只在 host proxy
prop 特判层）。

## 探针钉定

- dialog-requestclose 面 5 files 78P/1F，唯一红正是挂账项
  （inactive document 案）；main doc 的 detached-from-tree dialog
  requestClose 已可达（走 proxy 特判）。
- detached doc 的 dialog 元素 `_zwMEl` 产物四方法全缺
  （requestClose/close/show/showModal typeof 全 undefined）——R125 原型
  链已接 HTMLDialogElement.prototype 但 prototype 是空占位。

## spec 钉定

「request to close the dialog element」步骤
（interactive-elements.html）：①无 open → return；②**not connected 或
node document 非 fully active → return**（合成 doc 无 browsing context
恒非 fully active——dialog-requestclose-3 断言面）；③close watcher 请求
（cancel 可取消）。

## 落地（R5238，part03 原型成员挂载）

`HTMLDialogElement.prototype.requestClose`（R4100 details.name 同款
plain 世界供给惯例）：spec 步骤照抄——open 缺 return；ownerDocument ≠
主文档（非 fully active）return；watcher 机制只在 trap 世界，条件判毕
恒 no-op，open 保持。trap 世界 proxy get trap 走 part04 R4200 完整语义
版、先于原型命中，双世界不重叠可达。enumerable: false 同项目惯例
（for-in 面零变化）。

范围纪律：close/show/showModal 的 detached 缺失**不扩修**（挂账只记
requestClose，三方法无语料红面），记已知边界。

## 集群验证

- dialog-requestclose 面 5 files：**78P/1F → 79P/0F**（EXIT 1→0），
  REG 0 / GAIN 1——挂账项翻绿。
- dialog 集群 82 files 411 subtest：350P/60F/1T → **351P/59F/1T**，
  同 key 对 s35 账本 diff **REG 0 / GAIN 1**
  （数据 [2026-10-23-m2s36-wrapper-requestclose.json](2026-10-23-m2s36-wrapper-requestclose.json)）。
- forms 域 401 files **4627 subtest 3899P/707F/20T 对 s34 基线计数逐
  位一致**（REG 0 / GAIN 0；s34 双跑同口径）。
- 探针临时文件（zz-probe-s36-detached-dialog.html 6 案 3P/3F）验证后
  已删——回归由常驻语料 dialog-requestclose-3 承担；探针 3F 即
  close/show/showModal detached 缺失实证。

## 门禁

zero-engine lib **3429P/0F**；clippy `-D warnings`、fmt clean；零新增
依赖。runner release 二进制重建（shim include_str!）。R5237 撞渲染流
已占用，改 R5238。
