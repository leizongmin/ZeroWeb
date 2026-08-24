# R213 Evidence — extract 收缩偏移修正 + deleteContents CharData 删侧分支

**日期**: 2026-08-24
**切片**: M4——R213(a) 6,x positionTests 残余定位收口 + (b) deleteContents CharData 区间分支
**改动面**: `part06.js`（extract 收缩偏移 + delete CharData 分支）+ `part21.rs`（回归单测 + 两个旧断言按 spec 纠正）

## 一、6,x positionTests 残余根因（忠实复刻 testharness 探针）

R212 双 iframe 手工模拟与 host 一致但真实 testharness 分歧——本轮以 common.js
真源 + mySurroundContents 真函数（注入 R213 别名到 common.js 副本）忠实复刻
6,0 全流程，探针实证：**offsets A=0,1 E=1,2**——host extract 收缩到 (p5, si)，
sim 按 spec「Set new offset to one plus the index of reference node」收缩到
(p5, si+1)——后续 insertNode 落位前/后一位。修正 host 收缩偏移后：
offsets A=1,2 E=1,2 + scEqual=true + root isEqual=true（全对齐）。

## 二、deleteContents CharData 删侧分支（spec 三段的删侧）

start/end 容器均 CharData 且同父：
- 同节点：中段 deleteData + collapse
- 跨节点：start 尾段 deleteData + contained 子**逆序** removeChild + end 头段
  deleteData + collapse 到 (parent, si+1)（与 extract 收缩偏移一致）

## 三、验证链

- **单文件**：surroundContents 817P→**829P（+12，P2F=0）**；deleteContents
  45P→**58P（+13）**；extractContents 85P→**86P**；insertNode 628P /
  mutations 族不变（零扰动）
- **全量（polyfill）**：R212 基线 51216P/3820F/21T → **51243P/3795F/21T
  （净 +27P，P2F=0 纯增）**——F2P 26（delete 13 + surround 12 + extract 1）
- **全量（native 对照）**：**51244P/3795F/20T**——flips 仅 1 既存 flaky
  （insertBefore-iframe-crash Timeout↔Pass）
- **engine 单测**：2352 全绿（新增
  `test_extract_collapse_offset_and_delete_chardata_r213`——collapse 偏移 +
  delete 跨节点三段 + delete 同节点中段三断言组；R211/R212 两个旧断言按
  spec 纠正到 si+1 偏移与新插入位）
- **fmt / clippy**：零 diff / 零警告
- **make test**：（见 master.md R213 行）

## 四、方法论沉淀（忠实复刻探针）

当「手工模拟 vs host」一致但真实 harness 分歧时，注入**真源函数**（common.js +
用例本地函数改名注入）到 harness 副本跑完整流程——差异点直接暴露（本轮
offsets 一行）。比抽象推理快一个数量级。

## 五、commit

（落盘时待填）
