# R212 Evidence — surroundContents CharData 路径闭合（三个 spec 缺陷成对 land）

**日期**: 2026-08-24
**切片**: M4——R212(a) R211 回退的成对切片（CDATA cloneNode + surround CharData 路径）落地；过程中定位并修复三个真实 spec 缺陷
**改动面**: `part05.js`（工厂 appendChild fragment 展平）+ `part06.js`（步骤 1 补 nt=11 + 步骤 3 newParent 清子 + CharData 路径）+ `part03.js`（CDATA cloneNode nt=4）+ `part21.rs`（回归单测）

## 一、R211 首版 -48 的根因链（本轮三件定位）

1. **步骤 1 漏检 nodeType 11**：spec `dom-range-surroundcontents` 步骤 1 的
   Document/DocumentType/**DocumentFragment** 三类型——R209 实现只查 9/10，
   docfrag newParent 走到 CharData 路径实际变更树（模拟侧 InvalidNodeTypeError
   树不变）→ ,20 族 48F。
2. **工厂元素 appendChild 无 fragment 展平**：spec `dom-node-append-child`——
   「append fragment's children and then clear it」；旧版把 fragment 本体塞进
   childNodes，frag 子全丢（6,x 探针 P kids=2 vs 预期 3 的直接原因）。
3. **步骤 3「While newParent has children, remove its first child」缺失**：
   wrapped 元素残留 setup 期原文本（6,x 探针 P 内多一个 nt3("Ä̈b̈c̈d…")）。

三件修复后探针实证：`p5-after = nt1(P: nt4("34")nt4("5678")nt3("9012")) nt4("12")
nt3("")`、range=(p5,0,1)——与 common.js mySurroundContents 手工模拟**逐字节一致**
（双 iframe 对照探针 isEqualNode=true + walk-identical）。

## 二、实现清单

| # | 位置 | 内容 |
|---|------|------|
| ① | part06 步骤 1 | `nodeType === 11` 补入 InvalidNodeTypeError 检查 |
| ② | part05 工厂 appendChild | fragment 展平（子逐个 append 后清空 frag） |
| ③ | part06 CharData 路径 | extract → 步骤 3 清 newParent 子 → insertNode → appendChild(frag) → selectNode 语义（range 落 (parent, idx..idx+1)） |
| ④ | part03 cloneNode | CDATA nt=4 分支（经源 doc createCDATASection 重建）成对 land |

## 三、验证链

- **单文件**：surroundContents 823P→**817P**（F2P 33 / P2F 39——,20 族全转绿 +
  6,x DOM 转绿；剩 6,x positionTests 的 isEqualNode 深比较，见 R213）；
  cloneContents 127→**137** 连带；extractContents 81→**85**
- **全量（polyfill）**：R211 基线 51208P/3828F/21T → **51216P/3820F/21T（净 +8P）**
  ——F2P 47（surround 33 + clone 10 + extract 4）/ P2F 39（全 6,x positionTests）
- **全量（native 对照）**：**51216P/3820F/21T 逐计数一致**——flips 仅 2 既存 flaky
  （insertBefore-iframe-crash / MutationObserver-nested-crash）
- **engine 单测**：2351 全绿（新增 `test_surround_chardata_path_r212`——
  docfrag InvalidNodeTypeError + CDATA cloneNode 保形 + surround 树形态/range
  select 三断言组）
- **fmt / clippy**：零 diff / 零警告
- **make test**：（见 master.md R212 行）

## 四、R213 靶点：6,x positionTests 残余

双 iframe 对照探针（手工模拟 vs host）树逐字节一致且 isEqualNode=true——但真实
testharness 流程中 sim 结果与探针手工模拟存在差异（疑 common.js myExtractContents
的 newOffset 计算或 setStart/setEnd 顺序导致预期树插入位不同）。定位方法：在
testharness 环境插桩 dump 预期树。

## 五、commit

（落盘时待填）
