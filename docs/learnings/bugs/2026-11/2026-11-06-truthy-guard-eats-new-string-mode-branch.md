# 真值判断链吞掉新增字符串模式分支（q16 list-wide 死码四批）

## 问题描述

`_zwJusBlockExtend(range, chromeTight)`（`crates/engine/src/js_dom_shim/part01c.js`）原有分支结构为 `if (chromeTight) {closure 钳制} else {条件爬升}`。为 q16 转换主流程新增字符串判别档 `'list-wide'` 时，新分支写成 `else if (chromeTight === 'list-wide')` 追加在真值判断之后——字符串 `'list-wide'` 为 truthy，永远被第一个 `if (chromeTight)` 截进钳制臂，新分支成为死码。

后果具有迷惑性：toggleLists enable 档从此按 closure 钳制语义跑（与 q15 基线完全不同的 range 形状），门禁批连续四轮（fails 177→97→97→97）测的全是钳制臂行为；逐轮归因把 P2F 记在爬升门、纯壳判据等实际未上机的改动头上，修正方向一再跑偏。

## 根因分析

1. **分支插入点错误**：布尔档（`true`/falsy）与字符串判别档混在同一判断链时，真值判断必须放字符串判别之后（或收紧为 `chromeTight === true`）。
2. **验证缺位**：四轮门禁都只看批间 delta（fails 数、F2P/P2F 计数），没有先做「改动是否真的生效」的最小检验。若先 grep 二进制内 marker、或对一个目标 fixture 做单点探针（期望 list-wide 形状、实际钳制形状），一轮就能暴露分支未生效。
3. **判读锚定在自以为的改动面上**：P2F 族归因时默认「改动已按设计生效」，把回归解释成设计缺陷而非未生效。批数据与心智模型矛盾（basic 行按理不变却 P2F）才是真正线索——矛盾出现时应立即 diff 工作区核对实际改动面与分支可达性。

## 解决方案

- 分支重排：`if (chromeTight === 'list-wide') {…} else if (chromeTight) {…} else {…}`；其余调用点（justify 传 `true`、list-state query 传 `false`、其余传 undefined）逐一核对不受扰动。
- 防回归做法：新增判别档时，先 `grep -n` 该档的调用点与判断链顺序，再上机；首个门禁批前对目标 fixture 跑单点探针确认新分支路径真的被执行。
- 批内混源码（启动后改 src）按既有规则停批整跑；本例追加：**判读前先确认批二进制包含待测改动**，否则轮间 delta 全部作废。
