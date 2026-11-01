# M2-S206：output value/defaultValue 活读模型——mutations 2F 收口

- **日期**：2026-11-01
- **目标面**：`html/semantics/forms/the-output-element/mutations.window.js`
  2F（Descendant mutations and output.value and .defaultValue /
  output and output.form.reset()）（s205 后 misc 残面次刀候选序）
- **状态**：收口——靶面 2/2 Pass，forms 全树 keyed diff 净 F→P=2
  （恰两靶键）P→F=0 键漂移 0 timeout 位移 0

## 勘根

- **spec 落点**：HTML §4.10.12 The output element（whatwg live 版逐字
  核对，2021 rework 模型）——元素持 **default value override**（null/串，
  初始 null）；**default value** = override 非空取 override，否则后代文本；
  value getter=**后代文本活读**；value setter 步①**无条件** override :=
  default value ②String replace all；defaultValue getter=default value；
  defaultValue setter：override 为 null 时仅写文本（override 保持 null）、
  非空时仅置 override；**reset algorithm for an output element**：
  ①String replace all with default value ②default value override 置
  null（output 属 resettable form-associated element，form reset 触发）。
- **shim 旧模型三缺**（`_outputValue`/`_outputDefault` 全触点 grep 枚举）：
  ①value/defaultValue getter 只读脏值镜像/override——后代突变不可见
  （R2846 赋值即位模型，test 1「after setting textContent」断言面）；
  ②value setter override 捕获走 host 缓存直读（`__zw_get_text_handle`）
  ——JS 本地子文本盲，捕获 ''（test 1「after setting value」断言面）；
  ③form.reset() OUTPUT 并入 `c.value = c.defaultValue`——等价 value
  setter 步①，复位后 override 仍非空，后续 innerHTML 突变下 defaultValue
  冻结复位前值（test 2「after setting innerHTML」断言面，expected
  "something" got "value"）。
- **语料锚定**：mutations.window.js 两用例全为 createElement 本地树同步
  断言；output.html 在新旧两模型下断言观测等价（Pass 持平，佐证刀位
  精准）。

## 施工

（shim 2 文件 + rust 测试 1 文件；语料零改动）

- **part03.js getter 双活读臂**：value getter OUTPUT 分支（:26866）——
  镜像非空先回（承同 turn 宿主写 pending 期），否则
  `_makeProxy(sel, handle).textContent` 融合视图活读（R81 fused
  childNodes：JS 本地子 + 宿主同源）；defaultValue getter 同型（:28594，
  override ?? 活读）。注释挂 spec 链接 #dom-output-value /
  #dom-output-defaultvalue 与突变面失效理由。
- **part04.js value setter**（:14390 区）：override 捕获改融合视图读
  （`_makeProxy(sel, handle).textContent`，getter 同源）；镜像承同 turn
  读与宿主写照旧（spec 步①无条件捕获语义一致——null 时捕获即置值）。
- **part04.js innerHTML/textContent= 双失效臂**（:13300/:13651）：OUTPUT
  的 String replace all 语义面即删脏值镜像（value getter 回活读）；
  override 不触碰（value= 捕获跨 textContent= 稳定——test 1
  「after setting textContent again」断言面；textContent= 旧 R2846 覆写
  override 为冲掉捕获的错误语义，一并纠正）。
- **part04.js form.reset() OUTPUT 单列臂**（:12153-12171）：OUTPUT 撤出
  `c.value = c.defaultValue`（TEXTAREA 照旧），改 spec reset algorithm：
  `c.defaultValue` 读默认值（getter 同源）→ `c.textContent =` 写回
  （string replace all；R49 同值守卫命中即 no-op，两命中形态同 turn
  活读均收敛默认层）→ `delete _outputDefault` + `delete _outputValue`
  （spec 步② override 置 null + 镜像清，value/defaultValue 回落活读）。
  s133 注记「OUTPUT 无脏键模型」句同步改真。
- **part09.rs R5324 新桥测试**
  （test_r5324_output_value_defaultvalue_live_read_and_reset）：两腿镜像
  WPT 两用例全断言序（后代突变活读腿 + form.reset() 复位算法腿，
  createElement 本地树 proxy 形）——语料为 gitignore 本地数据，「兼容性
  修复附常驻测试与修复同一提交」由已提交桥测试等价承载（s205 先例）。

## 验证

- 拼接 shim（14 part，`js_dom_bridge.rs:5377-5413` include 序）
  node --check SYNTAX_OK（6,866,013 B）。
- 靶面：`make testharness-html-semantics FILTER=the-output-element` →
  mutations.window.js 2/2 Pass（stash 基线同两键 F）；
  output-setcustomvalidity/output.html Pass 持平。
- forms 全树 stash A/B（FILTER=forms）：**净 F→P=2（恰两靶键）P→F=0
  键漂移 0 timeout 位移 0**——两侧 (key,state) 对各 4711（4712 行去重
  −1 同名对，与 s205 同构）、覆盖全同；本轮 onblur flake 域零采样。
- **output-validity.html 1F 双侧同败**（A/B 无位移）→ pre-existing
  未入账残面归因：`:valid`/`:invalid` 伪类面——spec output barred from
  constraint validation（两伪类应均不匹配，含 setCustomValidity 后），
  引擎当前匹配 :valid。挂账待刀。
- zero-engine lib：**3665P+1F**（1F = ua_display_li 渲染流域在册键，基线
  同败非本刀；R5324 新测 +1 过，改名后定向复跑 1/1）。
- fmt 本刀零 diff；guarded-clippy workspace 过（12.64s 零警告）。

## 挂账

- output-validity.html `:valid`/`:invalid` 伪类面 1F（pre-existing，本轮
  A/B 定谳双侧同败）——新入 misc 残面候选池。
- part05.js defaultValue setter null 分支与 spec 分歧（spec：override
  null 时仅写文本、override 保持 null；引擎 R2846：置 override + 有镜像
  时不写文本）——语料未达（output.html 两模型观测等价、mutations.window
  不设 defaultValue），不扩刀记挂账。
- reset 按钮 UA 激活路径（s134 平行近似链）未核 reset 算法同面（语料
  本面只达 form.reset() 方法路径）。
- value= 宿主写 pending 期 textContent 融合视图读旧本地子（同 turn
  value 镜像断言不受累；语料无「value= 后立即读 textContent」形态）。
