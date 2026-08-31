# R210 Evidence — surroundContents 步骤 1/2 校验序（InvalidNodeTypeError / InvalidStateError）

**日期**: 2026-08-24
**切片**: M4——R209 后 surround 剩余最大簇 `assert_throws_dom: A INVALID_STATE_ERR must be thrown`（115F，20,x 跨容器族）
**改动面**: `part06.js`（surroundContents 校验序）+ `part21.rs`（回归单测）

## 一、根因（探针实证）

- 20,x 族（range `[paras[0].firstChild,0, paras[1].firstChild,0]`，cac=DIV 正确）：
  host surroundContents 对「非 Text 节点部分包含于 range」不抛 InvalidStateError
  （spec `dom-range-surroundcontents` 步骤 2）——探针 `HOST DID NOT THROW`，
  sim 侧 common.js mySurroundContents 首步即返 "INVALID_STATE_ERR" →
  assert_throws_dom "did not throw"。
- 24,8 等（newParent=document/doctype）在补步骤 2 后反转暴露**校验序**问题：
  步骤 1（newParent 是 Document/DocumentType → InvalidNodeTypeError）必须先于
  步骤 2（部分包含检查）——R209 的步骤 1 实现位于函数中段，新加的步骤 2 在其前
  使 24,x 的期望异常类型错配（10 P2F）。

## 二、修复

| # | 内容 |
|---|------|
| ① | surroundContents 步骤 2：cac 子树 DFS 检查非 Text 节点部分包含（`partial(n)` = 是 start 容器祖先 XOR 是 end 容器祖先——common.js isPartiallyContained 同款），命中抛 InvalidStateError；guard 2048 防失控 |
| ② | 步骤 1 块（R209 的 newParent nodeType 9/10 → InvalidNodeTypeError）移到函数头部——spec 校验序 1→2 还原 |

评估后**回退**的两件（记录为 R211 靶点）：
- head/docEl/body 三站点的 contains/cDP/iEN/cloneNode 方法面：surround 单文件
  823→814（净 -9）——这些站点的方法缺失由其他路径消费，「补方法面」改变行走路径
  产生副作用，需要单独评估（非纯增量）。
- CDATASection cloneNode（nt=4 分支，经源 doc createCDATASection 重建）：单独
  正确，但 sim 侧克隆修好而 host 缺 CDATA 跨容器 extract → 6,x positionTests
  树分歧 -34（两侧对称性教训同 R209）。与 host extract 同切片再 land。

spec 依据：https://dom.spec.whatwg.org/#dom-range-surroundcontents

## 三、验证链

- **单文件**：surroundContents **733P/1107F → 823P/1017F（+90P，P2F=0 / F2P=90 纯增）**
- **insertNode**：628P 保持（零扰动）
- **全量（polyfill）**：R209 基线 51096P/3937F/22T → **51188P/3847F/20T（净 +92P/-90F）**
- **全量（native 对照）**：**51187P/3847F/21T**——flips 仅 1（既存 flaky
  insertBefore-iframe-crash Pass→Timeout，历史轮次同形态）；passive-by-default
  only-in 差异为用例 subtest 名 wording 漂移（既存）
- **engine 单测**：2349 全绿（新增 `test_surround_invalid_state_and_step_order_r210`
  ——跨容器 InvalidStateError / Document newParent InvalidNodeTypeError /
  整选区不误伤三断言组）
- **fmt / clippy**：零 diff / 零警告
- **make test**：（见 master.md R210 行记录）

## 四、commit

`284522ce7`
