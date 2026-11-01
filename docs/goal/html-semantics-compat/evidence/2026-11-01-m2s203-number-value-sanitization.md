# M2-S203：number 值消毒文法收口——number.html + 约束测试 setter 面收口

- **日期**：2026-11-01
- **目标面**：`html/semantics/forms/the-input-element/number.html` 3F +
  `number-constraint-validation.html` setter 子测 1F（s181 分型「值清洗算法族
  27F」number 残面的 setter 文法簇，本刀 A 簇）
- **状态**：收口——4F→P 全数落账（forms 全树 keyed diff 净 F→P=4，P→F 仅
  onblur flake 域摆动样本，双向摆动有基线侧同败证据）

## 勘根

入场：s202 收口后 forms 现势 F 集重聚类，number 消毒残面簇 8F 定两簇：

- **A 簇（本刀，单一根）**——setter 消毒文法：`_zwSanitizeInputValue`
  number case（part01.js:2359）正则 `/^[+-]?(?:\d+(?:\.\d*)?|\.\d+)
  (?:[eE][+-]?\d+)?$/` 三处宽于 spec「valid floating-point number」文法
  （HTML §2.4.4.2 + number state 消毒 §4.10.5.1.10）：①`[+-]?` 容前导
  `+`（spec 只容 `-`，`+1`→''）；②`\d+(?:\.\d*)?` 容 `.` 后零位（`1.`→''）；
  ③无有限值检查（`2e308`→Number=∞→''）。面定谳：number.html
  "value = +1"/"value ending with '.'"/"value >= Number.MAX_VALUE" 3F +
  约束测试 4（setter `'1.'` → value '' 且 badInput false）1F。
- **B 簇（挂账）**——编辑路径语义：约束测试 1/3（send_keys 键入 `1.e`→''
  +badInput true、`1.`→"1"+badInput false）+ defaultValue-clobbering
  number arm + scientific-key-events——需要 editor-buffer 模型（键入原文
  保留/前缀解析 + badInput validity 接线），非消毒函数单点。

宽松正则同形共 4 处：part01.js:2359（number，本刀）、part01.js:2380
（range s189 钉态，非法 → default，语料无更严文法驱动面，不动）、
part04.js:243 / part03.js:2814（属性前缀解析算法，异算法，现势正确）。

## 施工

（shim 1 文件，净 +9/−1；rust/语料零改动）

- **part01.js** number case：`/^-?(?:\d+(?:\.\d+)?|\.\d+)(?:[eE][+-]?\d+)?$/
  .test(v) && isFinite(Number(v))`——三缺陷同刀收口；`.1`/`-0`/`1e2`/
  `9007199254740993` 等合法面 verbatim 保留（语料钉态）。range 同形正则
  按 s189 钉态原样。spec 链接两处（文法定义 + number state 消毒）。

## 验证

- 拼接 shim（14 part 按 js_dom_bridge.rs 序）node --check SYNTAX_OK；
  消毒函数 36 案单测（13 keep + 23 drop）全对。
- 靶面 stash A/B：number 域 125 键 **F→P=4 P→F=0 键漂移 0**
  （maxlength-number Timeout 双侧同态）；残 F = B 簇 4 键 + 既有
  valueasnumber-time/number-disabled（基线同败，非本刀）。
- forms 全树 A/B（WIP×2 + 基线×2）：F→P=4 双样本稳定（恰 4 靶键）；
  P→F 仅 onblur flake 域（w1 4 键、w2 2 键不稳定子集）——**基线侧同败
  实证**：b2（stash 基线）checkable-active-onblur spacebar 键 F（b1 P）、
  交替靶测基线样本 active-onblur Buttons spacebar 键 F 而 WIP 样本 P——
  双向摆动跨两侧二进制，载荷敏感在册 flake 域，非本刀回归。同载荷配对
  （w2 vs b2）净 F→P=4 P→F=1（onblur 摆动）键漂移 0。
- interactive 域 number 邻接面收口核验：the-summary-element 20 键
  bit-exact（该域唯一 type=number 语料，click 语义不触消毒路径）。
- zero-engine 全量 3664P+1F（1F = ua_display_li 渲染流域在册键，s201
  起 stash 基线同败，非本刀）；fmt --check 过；guarded-clippy 过。

## 挂账

- **B 簇（编辑路径 number 语义）**：约束测试 1/3（键入原文保留 + 前缀
  解析 + badInput validity 接线）、defaultValue-clobbering number arm
  （键入 `123e` → validity.valid false）、scientific-key-events
  "Insert - at start"——需 editor-buffer 模型，立刀位已勘定。
- range 状态同形宽松正则（part01.js:2380）：非法 → default 面 s189 钉态
  正确，更严文法无语料驱动，不动。
- onblur flake 域持续采样（本轮 +5 摆动样本：双侧二进制均有 F 记录）。
- 值清洗算法族其余残面（month/datetime-local/time/date 部分案 +
  valueAsNumber 小数秒等）按 s181 分型表继续单根分刀。
