# R204 Evidence — compareBoundaryPoints 四件：WebIDL how + Range 常量 + 方向位 + detached createRange 递归（M4）

**日期**: 2026-08-24
**切片**: M4 大切片——compareBoundaryPoints 簇 9313F → 601F（文件 1P → 8713P）；**全量 35453P/19085F → 47049P/7489F（净 +11596P，零新增失败文件，4 文件转绿：Range-cloneRange / Range-collapse / Range-commonAncestorContainer / Range-mutations-appendData）**
**改动面**: `part06.js`（how WebIDL 转换前置 + Range 常量 + 方向位修正）+ `part03.js`（detached doc createRange own 方法）+ `part21.rs`（单测）

## 一、四件修复

| # | 修复 | 根因 |
|---|------|------|
| ① | **how WebIDL unsigned short 转换前置**（参数序——先于 sourceRange 类型检查）：ToNumber → NaN/±0/±∞ → +0；否则 sign·floor(abs) mod 2^16 负回绕；转换值非 0-3 → **NotSupportedError** | 旧 `how | 0` 截断 + IndexSizeError：NaN\|0=0 误合法、-1 不回绕 65535、参数序颠倒 |
| ② | **Range how 常量**（START_TO_START=0 / START_TO_END=1 / END_TO_END=2 / END_TO_START=3，非可写） | 常量整体缺失——WPT 用例以四常量判 how 合法性，undefined 比对使**所有** how（含合法 0-3）被期望抛错（4728F 簇） |
| ③ | **跨容器方向位反转修正**：cDP 位以接收者为参照——& 4（FOLLOWING）= source 在 this 后 → **-1**；& 2（PRECEDING）= source 在 this 前 → **+1** | 旧两支写反（expected ±1 got ∓1 各 1091/1092F） |
| ④ | **detached doc 的 createRange own 方法**（初始边界 (doc, 0)） | `_makeDetachedDocument` 的 doc 对象无 own createRange（body 对象有同名但非 doc 的）——查找落到 Document.prototype R179 转发器，转发器 `this[name].apply` 再解析到**自身** → 无限递归 Maximum call stack。"Creating context/argument range threw" 4041F 整簇根因（common.js `rangeFromEndpoints` 经 `ownerDocument(node).createRange()`） |

## 二、诊断路径（探针链）

how 形态探针（17 形态 throw/no-throw 对照 WPT 转换表全对）→ 排除 how 本身 → 失败
subtest 名「context range 11 [foreignPara1.firstChild...]」→ 范围建立探针
（`foreignDoc.createRange()` 单步）→ Maximum call stack → stash-then-call 探针
（Illegal invocation + `own:false, proto-has:true`）→ 锁定转发器自递归。

## 三、A/B 与全量

- 全量 polyfill **47049P/7489F/21T** / native **47050P**（fail 集逐行相同）
- vs R203：**零新增失败文件**，净 +11596P
- zero-engine 2344 单测全绿（含新 `test_range_cbp_how_constants_direction_r204`）；
  fmt/clippy 干净；make test 除 XOpenDisplayFailed 环境项全绿

## 四、commit

`441e97a24`
