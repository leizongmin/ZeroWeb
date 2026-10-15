# M2 切片 1 — the-input-element IDL/类型面（2026-10-15）

## 范围

HTML §4.10.5.4（dom-input-value 四值模式）、§4.10.5.5（value 净化算法）、
§4.10.5.22（type 变更步骤）、§4.10.5.4 克隆步骤——`HTMLInputElement.value`
IDL 的 get/set 语义面。对应 spec 链接逐条落在 `part01.js` R3901 助手块注释。

## 运行方式

- 通道：`./target/test-guard --per-proc-mem 4 --total-mem 8 -- ./target/release/zero-wpt-runner
  testharness-html-semantics --json`（与 M1 基线同 guard 参数，墙钟 9000s；forms
  域先行单独跑确认后补全量九域，两者数字一致——forms 面 2320P 复现）。
- 原始账本：[2026-10-15-m2-input-value-modes.json](2026-10-15-m2-input-value-modes.json)
  （九域全量 1213 案逐案逐 subtest）；回归判定按 M1 基线
  [2026-10-15-m1-baseline.json](2026-10-15-m1-baseline.json) 逐 subtest 精确 diff。

## 总账

| 面 | 基线 Pass | 本轮 Pass | Δ | 基线 Fail | 本轮 Fail |
|---|---|---|---|---|---|
| the-input-element（1597 subtest） | 528 | 951 | **+423** | 1059 | 641 |
| forms 域（4109 subtest） | 1897 | 2320 | **+423** | 2178 | 1760 |
| **九域全量（6609 subtest）** | **2715** | **3248** | **+533** | 3474 | 3152 |

**回归 0**（基线 Pass → 本轮非 Pass 逐 subtest diff = 0，九域全量）。81 个文件
有提升，主簇：type-change-state +254、type-change-state-weekmonth +68、
valueMode +26、script-for-event +18、number +15。本轮多执行 240 subtest（如
the-input-element/maxlength.html 基线文件级未出数，本轮 5 subtest = 4P/1F）。
Timeout 178→193、NotRun 1→15：落在基线已败文件（embedded-content 数字名
文件、scripting-1 module load-error-events 族），与本轮 bench-gate 首测并发
测量窗重叠，回归中性（不影响 0 回归判定）。

## 实现面（engine shim R3901）

- `part01.js`：`_zwInputValueMode`（四值模式映射）、`_zwNormalizeInputType`
  （ASCII-lowercase + 关键字表白名单）、`_zwSanitizeInputValue`（texty strip
  \r\n、url/email trim、temporal/number 失效→''、range 钳制、color→'#000000'）、
  `_zwApplyInputTypeChange`（类型迁移四步：default 族 attr 承接、file 清值、
  净化、非可选→可选选区复位 {0,0,'none'}）。
- `part03.js`：value getter 头部 INPUT 模式分派——filename 恒 ''（无文件选择
  面）、default 族读内容属性（缺省 ''/'on'），type 读走 latest-wins。
- `part04.js`：value setter INPUT 分支——filename 非空串抛 InvalidStateError、
  `[LegacyNullToEmptyString]`、default/default-on 写内容属性、value 模式
  净化+dirty 缓存**不写内容属性**（修幻影 attr）；setAttribute/cloneNode/
  `input.type=`（part05）三路接类型迁移；clone 复制 dirty 缓存。
- 用户编辑通道绕行净化（spec：净化只服务 IDL set/attr 变更/创建/类型迁移；
  键入中间态 '1e' 是合法 badInput）：`__zw_text_input`/`__zw_text_delete`/
  execCommand insertText（part06）+ `script_gen.rs` 三处生成脚本，经
  `globalThis.__zwSkipValueSanitize` 旗标。

## 排查记录（中途回归 13 → 0）

首轮全域跑回归 13（4 文件）：根因单一——value setter 切出 INPUT 分支时误删
else 首行 `_inputValues[key] = String(value)`，该行是 **TEXTAREA 的缓存写**
（textarea 无专属缓存面，getter 靠它读回）。恢复该行后 13 回归清零：
reset-form（output ×2，Precondition 17 卡点）、setRangeText textarea ×6、
validity-textarea-defaultValue ×3、validity-valueMissing select/textarea ×2。

## 已知 FIXME（代码内挂账，不阻塞收片）

- datetime-local 净化宽松校验、week W53 未按年收紧、time 时区后缀接受。
- 类型迁移 dirty 旗标与 `removeAttribute('type')` 迁移缺口。
- 本域剩余失败簇（下一切片候选）：showPicker ~130F、stepUp/stepDown 43F、
  files null-for-non-file 23F、untrusted-key-event 42F、focus-dynamic-type-change 73F。
