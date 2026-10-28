# M2 s130 · radio 组根域枚举（同 root + form owner）——radio 组属主 3F 收口 + radio-disconnected-group-owner 5F 搭车翻绿（真翻红 0）

日期：2026-10-28 · 切片：html-semantics M2 · 前序：s129（composed/trusted 面）

## 入场定刀位

s129 收口勘误（join +3 唯一键归因修正：实为探针页三 Pass 键）后
按控制面下一步入场：radio.html 组属主语义 3F 单根勘根（orphan
tree / different form owners / different name——s125 组排他轮新入
账，组属主（tree/form/name 分组）判定缺失）。

## 勘根定谳（含排误）

1. **三案同根：detached 树组员 document qSA 不可见**。三案语料全
   在 detached container（`document.createElement('div') +
   innerHTML`，不入文档）内构组；s125 组排他枚举走
   `document.querySelectorAll`（part05 s125 块）——detached 树组员
   不在文档查询面，排他静默失效（首断言即败：`radio1 should be
   unchecked expected false got true`）。探针两轮取证
   （zw-probe-s130/s130b，验后删）：detached 容器子代是 handle 代理
   （`__zwHandle`，非 `_zwMEl` 镜像）；`parentNode` 链可达容器根
   （INPUT>DIV）；**容器根 qSA 子树枚举 identity 真**（subLen 2、
   `r3[0] === first`）；`.form` detached 活解析（FORM / null / 异
   form 异对象）——s125 form-owner 键比对（`_foKey125` 句柄串键）
   可直接复用。
2. **「not in the same tree」案在 s125 面已 Pass 系负案侥幸**：
   detached radio（remove 后）form-owner 键 null 与文档内 form 键
   不等被跳过——恰是 form 维度救场，而非树域判定成立；document 域
   枚举对无 form 组（orphan 案）仍错。勘根教训入账（见过程教训）。
3. **valueMissing 组枚举同根 + form owner 比对缺失**：part03
   `_validityState` radio 臂走 `globalThis.document.querySelectorAll`
   同名枚举且**无 form owner 比对**——orphan 案 radios[2]（detached
   form 内 checked）会被误计入无 form 组（「The second radio
   should be invalid***」断言面：radios[0] remove 后 radios[1]
   required 未勾应 missing）。
4. **残余 2F 异根（插入步 radio checkedness 同步族，s125 FIXME
   「其余置真现象」同族显性化）**：radio-disconnected-group-owner
   「Appending … disconnect form」（radio2 IDL checked=true 后
   appendChild 进 form——脏 checkedness 插入时组员变更无 uncheck
   钩子）+「Removed elements」后半（radio2 带 checked 属性
   appendChild 进 form 后 radio3 应 uncheck——checked 属性同步不跑
   排他）。本修复推进「Removed elements」案 2 断言（valueMissing
   after remove 臂已通，现卡 appendChild 进 form 的 radio3 uncheck
   臂）——根不同，让位 s131 候选。

## 施工（engine shim part03/05.js，无 Rust 变更）

1. part05 s125 组排他块：枚举根域修正——parent 链顶（≤64 跳）
   `nodeType === 9` → `document` qSA（文档面绿面不变；探针实测
   documentElement 级元素 qSA 异常返 0，文档面不经元素级）；链顶非
   document（detached）→ 链顶子树 qSA。注释附 spec 组定义链接与
   探针结论。
2. part03 `_validityState` radio 臂：同根域策略 + `_vlFoKey`
   form owner 键（`.form` 句柄串键，同 part05 `_foKey125` 形）——
   同 form owner 或均无才计组员（required/checked 组状态聚合前
   比对）。

## 验证

- 定向：`FILTER=the-input-element/radio.html` **12/12 全 Pass**
  （3F→0：orphan tree / different form owners / different name）；
  `FILTER=forms/constraints` **917P/3F 与 s129 全同额**（3 Fail =
  form-validation-validate fieldset 族，异根不动）；
  `FILTER=forms/the-input-element` **1509P/132F/3T**（s129
  1501P/140F/3T，join 翻动恰 8 行全 F→P、new/gone 0）。
- 全量 A/B × /tmp/s129-full.log 原始行口径 **6780P/1032F/103T →
  6785P/1029F/103T**（本轮两探针页参跑：+5 探针 Fail 键，−3 s129
  探针 Pass 键）；净语料口径键级 join **恰 8 Fail→Pass**
  （radio.html 3 + radio-disconnected-group-owner 5：tree root/
  disconnect tree/document fragments/shadow roots/non-HTML
  containers），**真翻红 0、new non-Pass 0、gone 0**。本轮无 guard
  内存杀。
- engine lib **3570/0**；wpt-runner **256/0**（76 ignored）；
  clippy（zero-engine --all-targets -D warnings）零告警；fmt 净；
  shim 拼接（js_dom_bridge.rs 装配序）node --check 过。
- 探针页 zw-probe-s130.html / zw-probe-s130b.html（gitignored
  wpt-data）取证后**已删**；语料零改动。常驻语料即回归面
  （radio.html 12 断言 + radio-disconnected-group-owner 8 断言在
  账）。渲染面零依赖——reftest/product-smoke 未跑。

## 挂账（本切片更新）

- **radio 组属主语义 3F / radio-disconnected-group-owner 5F 收口
  移出候选**。
- **插入步 radio checkedness 同步族 2F 显性入账**
  （radio-disconnected-group-owner 残余：「Appending … disconnect
  form」+「Removed elements」后半——appendChild 进 form/disconnect
  tree 后，checked 属性同步或脏 checkedness 均不触发新组组员
  uncheck；s125 FIXME「checked 属性增设/defaultChecked/激活行为等
  其余置真现象未走组排他」同族，s131 候选）。
- 承袭不变：button-events「type=button shouldn't trigger」与该文
  件级 Timeout、empty.html/empty-with-base.html Timeout、
  form-submission-0 186F、execution-timing 构成面、lazy-loading
  26F、meta color-scheme 21F、写流盖章断链、part02 setAttribute 绕
  notify、041 nested write、deep clone 非 HTML ns、css-module
  corpus 门（待决）、www2 可达性、module-import-referrer TLA、
  dynamic import() with 透传、classic 外链 SRI、143 write、defer
  多写完成序、接口对象基建面、FIXME 组、/tmp/zw-hb*.txt、FileList
  缺索引访问、live 树逗号列表 qSA 根缺口、渲染流面让位累积；
  **待决（须用户拍板）**：css/text 数据模块语料资源缺失、
  green-2x2.png 语料资源分发、parser form element pointer 4F 缓行
  （2026-10-23）。

## 过程教训

- **Pass 案 ≠ 该维度已实现**：not-in-the-same-tree 案 Pass 是
  form-owner 键比较的侥幸（detached null 键 ≠ 文档 form 键），树域
  维度其实缺失。勘根时对每个 Pass 案逐一归因（哪个维度恰好救场），
  避免把侥幸 Pass 当已有能力——组属主缺失因此迟至 s125 后才暴露。
- **detached 树面探针三件套**：qSA identity（返回物是否本体）、
  parentNode 链顶 nodeType（能否判 detached）、`.form` 活解析
  （form owner 键可用性）——一张探针页定三疑，根域改造前提一次取
  证齐。
