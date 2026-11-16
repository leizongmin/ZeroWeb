# path 门臂撤除刀（二十一轮后续，b1v10 干净链权威验证）

日期：2026-11-16。性质：twin 在飞 WIP 认领 + 单变量 corpus 验证轮。

> 留痕：本文件为双流对称收敛后的合并终态稿（对端 14:49:17 首写、本流
> 14:49 写出同账异名稿、互删后按磁盘终态由本流按对端全文重建），内容以
> 对端三口径账为准。

## 1. 刀内容（提交 = animation.rs，+54/−9）

前轮 twin 会话 14:18 留在工作树的 WIP，按「前轮可提交工作树认领件」协议认领：

1. **path 门臂回滚**：`clip_pair_echo_preserve` 删除
   `matches!((C::Path(_), C::Path(_)))` 臂——path()/path() 段数/命令失配对
   回归离散翻转语义（Chrome 期望即端点形），doc bullet 记 v5/v6 定谳链。
2. **Steps 位置钉** `test_timing_function_steps_position`（11 断言）：逐形态
   钉 css-easing-1 §step-easing-functions 语义（Start=ceil 上包络、
   Both=两端钳+ceil、None=内缩 round、缺省 End=floor），算术逐条复核
   （Steps(2,None)@0.5=floor(1.0)/2=0.5 等）。
3. gate 测试 path 断言同步翻转（「段数失配对离散翻转取 to」）。

## 2. 验证门禁（提交前）

- `cargo fmt -p zero-engine --check` rc=0。
- 默认特性 clippy（`-p zero-engine --all-targets -D warnings`）：全图唯一
  错 = `crates/script-sandbox/src/es_module.rs`（peer s419 WIP
  `too_many_arguments`，+221 行在飞认领面）——本刀 animation.rs 零命中。
  `--no-default-features` 形态 44 错为既有非绿基线（该组合非项目门禁），
  不作绕行账。全 workspace clippy 待 peer s419 落地后随下轮补跑。
- engine lib（test-guard `--compile-first`）：**4,074P/0F**（+1 = 新钉）。

## 3. b1v10 干净链（twin v6 链采认，停己采彼）

本流准备自发射 b1v10 链时发现 twin 已于 14:39:56 发射 v6 链（CHAIN-START
在本回合开始前 1 分钟），按「在跑链禁发重型入口」停己采彼：

- provenance：CHAIN-START 记 HEAD=808aa31a7（含上轮收口 docs）、脏树仅
  animation.rs WIP（字节 = 本刀提交内容）+ es_module peer WIP；
  **bin md5 = 3ab4488a**（留档 bin-trans-b1v10/，与 14:2x 干净重建同值 =
  构建确定性互证）。
- engine lib 4074P、七套件 corpus BATCH rc=0、CHAIN-DONE 14:42:56。

## 4. 对账（三口径）

| 口径 | gained | lost | regressed | pages_gain |
|---|---|---|---|---|
| vs b1v4（权威 B0） | **741** | 0 | 2 | 29 |
| vs b1v9（+222 预览批） | **0** | 0 | 0 | 0 |
| vs b1v8（隔离双刀合计） | **222** | 0 | 2 | 29 |

**vs b1v9 全零分歧 = 本刀单变量测量**：两二进制 lib 差仅 path 门臂与注释
（钉为 `#[cfg(test)]` 不入 release；es_module 同字节），corpus 全子测逐位
恒等 ⇒ **门臂撤除 corpus 零可见效应**。

+222 分解（b1v8→b1v10，Steps 修复 + 门臂撤除双刀合计）：187 非 path 增益
（Steps at(1) 编码族）+ **37 path 对增益**（shape-outside-path +21、
clip-shape +7、shape-shape +3、offset-path-008 +5、offset-path-001 +1）+
2 REG（= 上轮已定谳伪红恒定：inherit→shape at(1) 上游缺右括号对）。

## 5. v6 中间账 ±8 定谳修正

上轮间接账（b1v5→b1v6 −8）系**混杂归因**：v5→v6 捆绑 keyword 半径 Fix 1、
calc 结构化检测、门臂数变更，−8 无法单归门臂。本轮单变量测量（门臂撤除
零 delta）取代之：门臂对 corpus **零效应**——b1v8 中 Path/Path 门的 echo
与旧 Steps bug 的 from 侧回落对该族产出同形 FAIL，门的在场/缺席均不可见；
37 path 对增益的真根因是 Steps 修复解锁的 at(1) 翻转（eased 0→1），门臂
撤除仅是语义对齐（Chrome 离散端点形）+ 代码简化。已提交码内 doc bullet
记的 v6 叙事按当时证据成立，本节为后续单变量测量，不回改码注释。

## 6. 编译时序复核（b1v9 纯度认证维持）

上轮证据「b1v9 = 纯 58b3721ad 码态」复核成立：engine release 编译窗
≈14:14:10-40（链 14:13:56 起跑、runner md5 ~14:15:3x），早于本流 14:14:57
的 path 回滚编辑落地 → 447334b9 未扫入编辑；447334b9 ≠ 3ab4488a 的 md5
差异 = 门臂在场 vs 缺席（非混态）。上轮「混态作废」的处置（要求干净重跑）
仍正确——b1v10 即该干净重跑，数值互证 +741/0/2。

## 7. 账目

path 门臂撤除刀：**corpus 单变量零效应，+741/0/2 权威账维持**（b1v10 =
干净链新权威二进制，bin-trans-b1v10 md5 3ab4488a）。语义对齐 + 简化收益
保留，语义风险（DOWNS）零实测。residual面队列不变：③path() 命令级兼容
插值（37 增益为翻转语义，非逐命令插值）、inline-style 全局关键字解析疑点
（上轮挂账）。
