# R211 Evidence — extractContents 的 CharacterData 区间分支

**日期**: 2026-08-24
**切片**: M4——R211(a) CDATA 跨容器 extract 计划的**安全子集** land（extractContents 分支独立 +34；CDATA cloneNode + surround CharData 路径因成对依赖回退记 R212）
**改动面**: `part06.js`（extractContents CharData 区间分支）+ `part21.rs`（回归单测）

## 一、根因

host `extractContents` 对 start/end 容器为 Text/CDATA 的 range 走
`_coveredChildren`（非元素容器返 null）→ 整体 defer 返空 fragment——spec
`dom-range-extract-contents` 的三段算法（first/last partially contained
CharacterData 子切片 + contained children 本体移动）完全缺失。

## 二、实现（extractContents 前置分支）

适用形态：start/end 容器均 CharData（3/4/7/8）且同父。产出：
- 同节点：frag = [中段切片克隆]，原节点 deleteData 掉中段
- 跨节点：frag = [start 尾切片克隆, contained 子**本体**, end 头切片克隆]，
  原树 deleteData 两端 + contained 子被 appendChild 移动
- range 收缩到 (parent, start 容器原索引)

## 三、评估回退两件（R212 靶点，两侧对称性教训第三例）

1. **CDATA cloneNode（nt=4 分支）**：单独正确（探针实证 clone=nt4 data 正确），
   但 sim 侧克隆修好而 host surroundContents 的 CharData 路径未完成 →
   6,x positionTests 树分歧 -34。与 surround 路径成对 land。
2. **surroundContents 的 CharData 区间路径**（extract→insertNode→appendChild(frag)
   + selectNode 语义）：首版两缺陷（docfrag newParent 误入 leaf-throw 检查 /
   frag 末子丢失）净 -48——回退，需单独定位 docfrag nodeType 11 不在叶子集
   （3/4/7/8）但首版路径误伤的原因与 frag 完整性。

## 四、验证链

- **单文件**：Range-extractContents **61P→81P（+20）**；Range-cloneContents
  127P→137P（+10，extract 分支连带）；surroundContents 823P / deleteContents
  45P / mutations-deleteData 456P 零变化
- **全量（polyfill）**：R210 基线 51188P/3847F/20T → **51208P/3828F/21T（净 +20P）**
  ——逐 subtest 转移：F2P 24 全在 Range-extractContents，P2F 4 同文件（形态重分布）
- **全量（native 对照）**：**51208P/3828F/21T 逐计数一致**——A/B flips 仅 2
  （既存 flaky：insertBefore-iframe-crash / ParentNode-querySelector-All-content）
- **engine 单测**：2350 全绿（新增
  `test_extract_contents_chardata_interval_r211`——同节点中段 + 跨节点三段 +
  range 收缩三断言组）
- **fmt / clippy**：零 diff / 零警告
- **make test**：（见 master.md R211 行）

## 五、commit

914390537
