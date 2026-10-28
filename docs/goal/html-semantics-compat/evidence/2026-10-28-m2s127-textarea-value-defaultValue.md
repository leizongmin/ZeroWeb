# M2 s127 · TEXTAREA value/defaultValue getter 面收口（child text content + API 值归一 + setter null 折叠——value-defaultValue-textContent 10F→0F，真翻红 0）

日期：2026-10-28 · 切片：html-semantics M2 · 前序：s126（img relevant mutations 三根因收口）

## 入场定刀位

全量基线（/tmp/s126-full.log）翻拣 forms/the-textarea-element 域：
value-defaultValue-textContent.html 整页 12 子测试 10F，语料全为
detached createElement textarea（handle 形）——value/defaultValue 读面
单根，入场。

## 勘根定谳（10 失败断言逐条映射）

1. **value getter 非脏路径 handle 形钉死**（主根因，textContent/
   nodeValue/data/子节点增删 6 断言 + defaultValue 3 断言）：旧路径
   handle 形落 stable-key 惰性初始化读 value **属性**——textarea 无该
   属性 → `_inputValues[key]` 缓存 `''` 钉死，live 树一切变更不可见。
2. **API 值换行归一缺失**（treat CRLF differently / normalizes CRLF
   even spread over multiple text nodes + 搭车 wrapping "Newlines
   should be normalized to LF"）：HTML §4.10.11 API value =
   raw value 归一 `\r\n|\r → \n`，旧 getter 原样返回。
3. **child text content ≠ textContent**（"reflect child text content,
   not textContent"）：child text content 仅直接 Text 子数据拼接；
   `_zwTaLiveText` 的元素 `textContent` 并入臂把后代元素文本也并入。
   R81 textContent getter 融合视图不受影响（各自语义）。
4. **value setter null 折叠缺失**（"tests for the value setter"）：
   value IDL 带 `[LegacyNullToEmptyString]`，`null → ''`，旧 setter
   存 `"null"` 字符串。
5. **defaultValue getter R3049 冻结捕获 stale**： defaultValue 应
   live child text content（defaultValue= 初值语义由 setter 替换子
   节点保证，见 s125 前模型）；冻结捕获在子节点变更后失效。

## 施工（engine shim part01/03/04/05.js + part01.rs 注释，无 Rust 行为变更）

1. part03 `_zwTaLiveText`：删元素文本并入臂——child text content
   仅直接 Text 子（spec 链接注释同步）。
2. part03 value getter 补 TEXTAREA 臂（`_rvTag3` 已有 tag 判别处）：
   脏（`_inputValuesSet[key] === true`）→ dirty 缓存读侧归一；非脏 →
   sel 形 `__zw_get_text(sel)` / handle 形 `_zwTaLiveText(handle)`
   后同款 `\r\n|\r → \n` 归一（setter 存原串，归一统一在读侧）。
3. part03 defaultValue getter：R3049 冻结捕获 → live（sel/handle
   双形态同 value getter 非脏路径）。
4. part04 value setter：`_s127TaSet = (_r3906IsTa && value == null)
   ? '' : String(value)`（[LegacyNullToEmptyString] 折叠，仅
   TEXTAREA）；`_inputValues`/`_inputValuesSet` 缓存写保留（2026-10-15
   learning：写入点当契约看）；三处写点改用折叠后值；
   `_textareaDefault` sel 捕获行删。
5. part05 defaultValue setter：删 `_textareaDefault` 捕获行（R57
   live 视图同步逻辑保留）。
6. part01：`_textareaDefault` 声明 4 行删 + `__zw_reset_form_state`
   链移除；part04 reset 注释 R3049 → s127 live 语义。
7. part01.rs `test_textarea_default_value_r3049`：仅注释更新为 s127
   live 语义说明，**断言不变**——①defaultValue= 初值 ②value= 不改
   defaultValue ③顺序无关 ④reset 还原，在新模型下全部兼容。

## 验证

- shim 拼接（js_dom_bridge.rs 装配序）node --check 过。
- 定向 `FILTER=forms/the-textarea-element`：**33P/8F**（修复前基线
  该域 19F）。value-defaultValue-textContent **10F→0F**（12/12 Pass）。
- 全量 A/B × /tmp/s126-full.log：原始行口径
  **6738P/1073F/103T → 6749P/1060F/103T**；键级 join 翻动 = 11 条
  Fail→Pass（恰为本修复 10 + wrapping 搭车 1）+ 2 条 GONE
  （zw-probe-s126.html——s126 探针页验后已删的 corpus 态差，非代码）；
  **真翻红 0、Fail→Timeout 0、新增 0**；既有 ×2 重跑键（parse-a-srcset
  / wrapping exact-len / 149.html）两轮一致。口径注：s126 账本记录
  6737P/1071F/103T = 同文件原始行扣探针 2F、既有 ×2P 记 1 行；本轮起
  记原始行口径并以 join 为门禁。
- 首跑 test-guard 单进程内存超限杀一次（4.31GB>4GB；ZeroWeb-3 并行
  测试流在场）复跑过——并行摆动（s126 同款注记）。
- engine lib **3570/0**（+11 单元 bin 全过）；wpt-runner **256/0**
  （76 ignored）；clippy（engine+wpt-runner --all-targets -D warnings）
  零告警；fmt 净。
- 语料零改动、本轮无探针页（勘根经日志断言消息 + shim 读码闭合）；
  runner 侧无代码改动——常驻语料 value-defaultValue-textContent.html
  12 断言即回归面（与修复同提交）。渲染面零依赖——reftest/
  product-smoke 未跑。

## 挂账（本切片更新）

- **textarea value/defaultValue 面收口移出候选**。
- the-textarea-element 域残余 8F 各单点（change-to-empty-value /
  maxlength / minlength / placeholder-lineheight / textLength / type /
  wrapping LF+col-count ×2）——未勘根，非本轮面，同域收割候选。
- 承袭不变：form-submission-0 186F、execution-timing 构成面、
  lazy-loading 26F、meta color-scheme 21F、写流执行身份盖章断链
  FIXME、part02 setAttribute 绕 notify 臂、041 nested write、radio
  组排他其余置真现象、deep clone 非 HTML ns 面、naturalWidth SVG
  viewBox 22F、current-pixel-density 11F（渲染流面让位）、css-module
  corpus 门（待决）、www2 可达性、module-import-referrer TLA、
  dynamic import() with 透传、classic 外链 SRI FIXME、143 write、
  defer 多写完成序、接口对象基建面、FIXME 组、/tmp/zw-hb*.txt、
  FileList 缺索引访问、live 树 document 级逗号列表 qSA 根缺口；
  **待决（须用户拍板）**：css/text 数据模块语料资源缺失、
  green-2x2.png 语料资源分发、parser form element pointer 4F 缓行
  （2026-10-23）。

## 下一步候选（s128）

挂账序复核（form-submission-0 186F POST 传输契约勘根大基建评估 /
execution-timing 构成面单根）；或全量基线翻拣下一单根面（域翻拣刀法
同款）；或 the-textarea-element 域残余 8F 同域收割逐点勘根。
