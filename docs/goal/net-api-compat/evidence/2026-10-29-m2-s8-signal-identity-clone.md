# M2-S8 — Request signal 身份族（构造器克隆 + init 覆盖三态）（2026-10-29）

**切片**: goal v2.10 小片清单——「signal 身份族 4F」（承 M2-S6 记账 2）。
**二期 M 收口后小片第 4 轮**。

## 4F 归因分解

| 腿 | 根因 |
|---|---|
| Signal state is cloned | Request 构造器直接引用附着（`signal === input.signal`），无克隆语义 |
| Signal on request object created from request object | 同上（克隆后 `clone.signal !== request.signal` 断言必败） |
| Signal on request object should also have abort reason | 克隆缺 abort reason 传递 |
| Signal removed by setting to null | `fetch(request, { signal: null })` 未按「显式非 AbortSignal → 移除」处理 |

注意：**「Clone aborts with original controller」腿基线即 Pass**（旧引用附着
语义下 request.signal 与 clone.signal 同对象，双 listener 同队自然有序）——
本片要求它在升级为独立对象克隆语义后**保持** Pass，是本片唯一的顺序暗礁
（见「过程轮」）。

## 实现面

### 1. 构造器克隆原语 `_zwAbortSignalFollow(source)`（`part02.js`）

新 signal 对象 + 依赖登记（`source._zwDependents`），非 listener：

- 源已 aborted → 立即同 reason abort（克隆不丢已中止状态，sync）；
- 源在 → 登记依赖，级联由 `_zw_abort_signal` 统一完成（dom spec
  「signal abort」步骤 3-6 的 dependent signals 语义）；
- 源 null（init 显式 null / 无源缺省）→ 裸新 signal。
  https://dom.spec.whatwg.org/#abort-signal-abort
  https://fetch.spec.whatwg.org/#dom-request

### 2. `_zw_abort_signal` 依赖级联（`part02.js`）

步骤 3-4：依赖 BFS 展平收集并置 aborted+同 reason（**不派发事件**）；步骤
5-6：源与依赖的 abort steps（listeners + onabort）按源先依赖后逐个派发。
事件顺序钉 `['original-aborted', 'clone-aborted']`；深层依赖（克隆的克隆）
按发现顺序续后。无 `_zwDependents` 的 signal（既有全部路径：controller /
timeout / AbortSignal.abort / fetch 入口 abort）零变化。

### 3. Request 构造器三态 + accessor（`part01.js`）

- `signal` 在 init：AbortSignal → follow 克隆；非 AbortSignal（含 null）→
  裸新 signal（fetch spec：「if init["signal"] exists」分支的成员检查语义）；
- init 无 `signal` 且 input 为 Request-like → follow input.signal（载体回落）；
- accessor：getter 返回克隆对象（[SameObject]），setter 忽略赋值恒置新
  AbortSignal（spec Request signal 无 setter 语义的近似——引擎无 [LegacyUnforgeable]
  抛 TypeError 面，语料无钉）。

### 4. fetch signal 三态（`part01.js` `_zwFetchMainImpl` + h2 流式检查点）

fetch spec「signal member」三分支同步到两处取值点：

- `init.signal instanceof AbortSignal` → 用之；
- init 有 `signal` 但非 AbortSignal → **移除**（不回落载体——
  `fetch(request, { signal: null })` 腿的钉子）；
- init 无 `signal` → 回落 input.signal（Request 载体继承）。

## 过程轮（未落提交的中间态）

r1 用 listener 挂源实现级联（follow 时注册在源 listeners 队尾，先于测试体
注册的 original-aborted listener）→ `_zw_abort_signal` 按注册序派发 →
clone-aborted 抢在 original-aborted 之前：abort/general 38P/23F（4 目标腿
绿 + clone-order 腿翻红）。**dom spec 步骤 5-6 顺序语义**（依赖 abort steps
在源 abort steps 之后）改为 dependent 模型（`_zwDependents` 登记 + abort 时
BFS 收集按序派发）后 r2 清偿。中间态未提交，最终 A/B 相对基线 +4/0。

## 验证

- probe（`testharness-net-api abort/general`）：general.any.js
  35P/26F → **39P/22F**——本片 4 腿 + clone-order 腿（保持 Pass）全绿；
  余 F 全为 body-abort 族（记账 1，非本片面）。
- 全通道 A/B（`netapi-m2s8-r1.json` vs **直接前轮**
  `netapi-m2s7-r1.json`，逐 subtest）：**+4 增益 / 0 回归 / MISSING 0**
  （10893 → 10897 pass）；per-file Δ 仅 abort/general（其余全通道逐文件
  零变化，integrity.sub / request-structure 零扰动）；stderr **0 panic**，
  构建 0 warning（`m2s8-build-r2.log`）。

## 记账（未清偿）

1. **body-abort 族 ~30F**（response.\* rejects with abort reason 6F +
   Stream errors once aborted 3F + connection-closed/body 调用组合面）：
   体消费中 signal abort → body stream error(AbortError)——引擎 body-abort
   机制（M2-S6 记账 1/3 同面），abort/general 余量主体。
2. Request signal setter 的 [LegacyUnforgeable] 抛 TypeError 面语料无钉，
   未入门；随 request 构造器语料扩批再评估。
