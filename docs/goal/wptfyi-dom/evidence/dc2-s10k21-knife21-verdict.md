# DC-2 切片 10 刀㉑收账——Trusted 接口原型补自有 constructor（HTMLElement-generic 残 20F 清零）

**日期** 2026-11-25　**判定** PASS（TT 226→206＝净 +20 face，GONE 20/NEW 0；WC/DP/SAN/DOM Fail/Timeout 集逐键 SET-IDENTICAL）

## 提交与判读绑定

| 提交 | 内容 | 判读二进制 | 电池 |
|---|---|---|---|
| 刀㉑ `c994e43bd`（谱系：发射前 69656222d → pull --rebase 携入渲染流 3 笔 docs 后重写 `faa1092f2`〔电池发射锚〕→ 收账前注释字形订正 amend（刀㡸→刀㉑，shim 串注释字节变化＝行为零变化）`c994e43bd`；发射锚二进制 `75b423f7` 以 `faa1092f2` 构建，发射器 EXPECT_MD5+EXPECT_HEAD 双断言在锚上通过） | `_zwTtMakeInterface` 原型补 WebIDL 强制自有 `constructor`（writable/non-enum/configurable，值＝接口对象）＋part167 钉测六向 | `75b423f79f55cc58386d9e193a1ed8e9` | v5 |

发射器 `.acceptance/dps/s10k21-battery-v5.sh`（EXPECT_MD5+EXPECT_HEAD 双断言、flock 单例、五腿 test-guard 包裹、setsid detached）；墙钟形与 v4 逐腿一致（WC 7m40s/DP 13s/SAN 17s/TT 16m41s/DOM 10m49s）。

## 根因链（探针→归因→单行刀）

1. **探针定谳**（临时 probe，后转正为钉测）：`pp.createHTML('<b>x</b>').constructor.name === 'Object'` 而非 'TrustedHTML'——`_zwTtMakeInterface` 的 `proto = {}` 缺自有 `constructor`，实例 `x.constructor` 落 Object.prototype 链读出 Object；`toString()` 本身正确（返原始串，`===` 恒等）。
2. **失败面反转**：v4 残 20F 全打 `= Object`（Trusted 变体的名字），`= String`（plain 串变体）全在 Pass 列——plain 串 enforced 抛 TypeError 引擎早已正确（001 静态 meta 页 `innerText` 抛面全绿佐证）；Trusted 变体因 `constructor.name='Object' != getPropertyType(...)` 被测试要求抛，而 sink 品牌集正确**接受**匹配型 Trusted 值不抛→误判 fail。script.src 全绿恰证 gate 类型感知（TrustedScript 值入 TrustedScriptURL sink 被拒）。
3. **刀**：WebIDL [interface prototype object](https://webidl.spec.whatwg.org/#interface-prototype-object) 强制原型自有 `constructor`——单处补位，品牌 WeakSet/instanceof/proto 链零接触。

## v5 判读（锚＝k20b v4 终态 TT F 226 / WC F 1116 / DP F 222 / SAN F 50 / DOM F 364）

| 腿 | Fail | GONE/NEW | Timeout 键集 |
|---|---|---|---|
| TT | **226→206（Pass 1856→1876，+20）** | **GONE 20 全 HTMLElement-generic / NEW 0** | 65=65 恒 |
| WC | 1116→1116 | 0/0 | 恒 |
| DP | 222→222 | 0/0 | 恒 |
| SAN | 50→50 | 0/0 | 恒 |
| DOM | 364→364 | 0/0 | 恒 |

GONE 20＝innerHTML/srcdoc/text/innerText/textContent × Trusted 变体 × connected/non-connected × 两 phase（TT enabled＋meta 移除后）。

## 常驻钉测

part167 两测六向：V1 四类 constructor 恒等（TrustedHTML/TrustedScript/TrustedScriptURL/工厂/政策）＋V2 匹配型 Trusted 值五 sink 接受不抛＋V3 plain 串 enforced 五 sink 抛 TypeError（保信）＋V4 script.src 类型感知拒 TrustedScript 值（保信）＋N1 无 meta 负面页（constructor 恒等与 enforcement 无关、plain 不抛）。门禁批内 2/2 绿；s10k 全家 23/23 绿（`cargo test -p zero-engine --lib s10k`，test-guard --compile-first 包裹）。

## 质量门禁

fmt 零 diff；clippy `-p zero-engine --lib --tests -D warnings` 0 warning；提交 lei-pre-commit-guard PASS（3 文件 148 行，安全发现 0）；release 重建 `cargo build --release -p zero-wpt-runner`（test-guard 包裹 1m00s）。

## 残面挂账（次刀输入）

TT 残 206F：secondary-document 30／from-literal 21／block-string-assignment Document-write 10／default-policy-report-only 9／createHTMLDocument 8／script-enforcement-008/009 各 5 等；HTMLElement-generic 全清（48/48 面两刀收齐）。

## 累计

DC-2 累计 83,721→**83,741**（刀㉑净 +20，五腿电池口径）；距 ≥99,947 差 16,206。
