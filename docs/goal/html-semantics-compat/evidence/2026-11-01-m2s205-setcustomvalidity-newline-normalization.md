# M2-S205：setCustomValidity 换行归一——setCustomValidity-normalize-newlines 1F 收口

- **日期**：2026-11-01
- **目标面**：`html/semantics/forms/setCustomValidity-normalize-newlines.html`
  1F（expected `\r\n|\r`→`\n` 四行归一，got 原样）（s204 后 misc 残面
  最小单根刀位，上轮候选序首位）
- **状态**：收口——靶面 1/1 Pass，forms 全树 keyed diff 净 F→P=5
  （1 靶键 + onblur flake 域 4 键顺向回摆）P→F=0 键漂移 0 timeout 位移 0

## 勘根

- **spec 落点**：HTML §4.10.21.3 The constraint validation API
  （#dom-cva-setcustomvalidity）setCustomValidity(error) 方法步两步——
  ①Set error to the result of **normalizing newlines** given error（Infra
  normalize-newlines：`\r\n`/`\r` → `\n`）；②存为 custom validity error
  message。归一在**存入时**，validationMessage getter 原样回读。
- **shim 双实现同缺**：全 shim `_customValidity` 写点恰两处（grep 枚举）
  ——part04.js:5996 proxy 形 `_customValidity[key] = String(msg)`（runner
  语料到达路径：解析文档 getElementById 产物）；part03.js:18393 plain-node
  形 `_zwMElSetCustomValidity`（本地树形）`this._customValidity = String(m)`
  。两处均存原串。读侧（part04:6016 / part03:18763）原样返回——归一落点
  收敛为两处存入侧，getter 不动。
- **Rust 侧无同面写点**：`crates/dom/src/document/validation.rs` 原生约束
  校验无 setCustomValidity 通道；全仓 .rs 无该面实现（grep 核实）。

## 施工

（shim 2 文件 + rust 测试 1 文件；语料零改动）

- **part04/part03 两存入点**：`String(msg)` 后接
  `.replace(/\r\n/g, '\n').replace(/\r/g, '\n')`——仓库惯用法内联链
  （part03:17510/26843 等同款），两处不为一次使用引共享 helper。同面双形
  同修（guidelines #6 双路径共享等价；s204 R57 双形同刀先例）。
- **part08.rs R2825 桥测试扩换行归一断言**：`setCustomValidity('a\r\nb\rc\n')`
  → validationMessage `a\nb\nc\n`。语料文件为 gitignore 本地数据
  （`.gitignore:84`，10-15 本地加样），「兼容性修复附常驻测试与修复同一
  提交」由已提交桥测试等价承载（proxy 形，与 WPT 语料面同路径）。

## 验证

- 拼接 shim（14 part，`js_dom_bridge.rs:5377-5413` include 序）
  node --check SYNTAX_OK（6.84MB）。
- 靶面：`make testharness-html-semantics
  FILTER=setCustomValidity-normalize-newlines` → 1/1 Pass（stash 基线同键 F）。
- forms 全树 stash A/B：**净 F→P=5 P→F=0 键漂移 0 timeout 位移 0**——
  两侧 (key,state) 对各 4711（4712 行去重 −1 同名对）、400 文件覆盖全同；
  1 靶键 + onblur flake 域 4 键顺向回摆（active-onblur button 1 +
  checkable-active-onblur 2 + with-click 1；s204 +3 后继续回升，双向摆动
  再证在册 flake 归因）。
- **runner 静默口径注记**：forms 树磁盘 822 文件（.html+.window.js）仅
  400 产结果行——静默集 = reftest 参照页（-ref.html）161 + manual/support
  + crash 测试页/外观 reftest/iframe 内嵌流等无 testharness 结果形态
  ~263；A/B 双侧同覆盖，diff 有效。后续切片不必按磁盘文件数核对覆盖。
- zero-engine lib：**3664P+1F**（1F = ua_display_li 渲染流域在册键，基线
  同败非本刀）；新增换行归一断言在 `test_constraint_validation_r2825`
  内通过（测试数不变，断言内扩）。
- fmt：本刀 Rust 插段零 diff（part08.rs 既有 fmt 偏差属 HEAD 已提交内容，
  归口历史提交只提及不修）；guarded-clippy workspace 过（19.35s 零警告）。

## 挂账

- part03 plain-node 形无独立定向测试（与 proxy 形同一表达式字面，代码检视
  + 桥测试 proxy 形承载；runner 到达路径为 proxy 形，无语料命中本地树形）。
- onblur flake 域持续采样（本轮 +4 顺向样本，s204 +3 后连续两轮回升）。
- number B 簇 editor-buffer 独立立项勘根（s203 已勘定）与 misc 残面次刀
  （output mutations 2F / form action 反射 base-url 2F / form owner
  table 2F）按序待刀；customizable select 177F 挂起待拍板（GB-20261031）。
