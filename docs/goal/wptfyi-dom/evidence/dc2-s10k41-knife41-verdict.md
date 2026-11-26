# 刀㊶ 判读（s10k41）——eval(TrustedScript) TT 集成薄包装＝no-csp 页 6F 清零

日期：2026-11-26 · 域：trusted-types / eval 集成 · 树锚：本刀 patch `knife41-final.patch` md5 `232eea2526f6352c67406cadee43ad9c`

## 目标簇（池内入刀前）

- `eval-no-csp-no-tt.html` 2F＋`eval-no-csp-no-tt-default-policy.html` 2F＋`eval-csp-no-tt.html` 2F＝**6F 连片**，形全等：`eval of TrustedScript works`＋`indirect eval of TrustedScript works`——`assert_equals(eval(p.createScript('1+1')), 2)` 得对象 `"1+1"` 非值 `2`。

## 根因与修复（2 文件，码刀见 commit）

### 根因：eval 门装置条件不含「TT 支持即生效」面

- TT spec §eval()-integration 在 TT **支持即**生效（无 CSP/无 enforcement 页 `eval(TrustedScript)` 亦须 ToString 编译）。现全包装装置条件＝CSP enforced／report-only／script-src 无 unsafe-eval——三页均不满足（no-csp 两页零 meta；eval-csp-no-tt 的 script-src 含 unsafe-eval）→ eval 保原生 → native eval 非串实参不转换（PerformEval 原样返回）→ TrustedScript 对象直穿。
- Function 编译门（刀③③b）已有 TS 臂（`new Function(ts)` 面全绿为证）——缺口仅 eval。

### 修复：TT-API-已用旗＋eval 薄包装（part12＋part14）

- k29 返修二边界承袭：JS 层全局包 eval 退化直接 eval 声明落点（Range 框架五测崩）——**不可**为 no-csp 页全量装门。
- 窄条件：`_zwTtCreatePolicy` 顶置 `__zwTtApiUsed` 旗＋调 `__zwMaybeInstallEvalThinGate`——页面实际调用过 createPolicy 才装薄包装（dom 语料非 TT 页零装置零退化；TT 语料页自身无直接 eval 声明断言面，值语义经 native 尾调等价）。
- 薄包装单臂：`isScript(x)` → `String(x)` 落 native 编译（WebIDL wrapper 边界 ToString）；**其余一切实参原样透传 native**——CSP/TT enforce face 只归全包装承载（全包装已在位时薄装 no-op，其 TS 臂已覆盖同一语义）。
- 边界挂账：宿主 `__zwUninstallEvalGate` 与薄装无交互（语料无卸载后 TS eval 形态；FIXME 待证据）。

## 判读

### solo 页级（post 树）

| 页 | base | post | rc |
|---|---|---|---|
| eval-no-csp-no-tt | 8P/2F | **10P/0F**（整页全绿） | 0 |
| eval-no-csp-no-tt-default-policy | 5P/2F | **7P/0F**（整页全绿） | 0 |
| eval-csp-no-tt | 5P/2F | **7P/0F**（整页全绿） | 0 |
| eval-csp-tt-default-policy（全门回归抽检） | 24P/0F | 24P/0F | 0 |
| eval-csp-tt-default-policy-mutate | — | 6P/0F | 0 |
| eval-csp-tt-no-default-policy | — | 13P/0F | 0 |
| eval-with-non-trusted-script-object | — | 1P/0F | 0 |
| eval-with-permissive-csp | — | 9P/0F | 0 |
| eval-function-constructor | — | 65P/0F | 0 |

### 电池四腿＋TT 同树 A/B（字节绑定 `232eea25…`，setsid pid 23247）

四腿＋TT A/B 全数落定（driver `/tmp/s10k41/driver-k41.log`，全程 15:12:38→15:46:55Z＝34min）：

| 腿 | k40 基线（HEAD 树） | k41 post | 判 |
|---|---|---|---|
| san | 713P/65F | 713P/65F | **SET-IDENTICAL** |
| dp | 1314P/221F/1T | 1314P/221F/1T | **SET-IDENTICAL** |
| wc | 17653P/920F/20T | 17349P/920F/20T | F/T multiset **恒等**；P −304＝gethtml 截断抖动已知噪声页（单页摆幅 ±200~300 家族） |
| dom | 57525P/362F/12T | 57197P/362F/11T | **F 362/362 multiset 恒等**；T 漂移＝ParentNode-replaceChildren 单页两测级 T→页级 T（停摆/时序形——该两 MutationRecorder 测 solo 复跑 29P/0F/2T 同族在场，非刀位） |
| TT base | — | 2198P/61F/20T | P/T 与 k40 post 同带；F −1＝k40 发射时在场已删的 zz-knife40-probe（对账实恰，非 flake）——base 与 k40 post 真实页逐键同带 61F |
| TT post | base 2198P/61F/20T | **2204P/55F/20T** | **+6P/−6F，NEW=0**，Timeout 20 恒等——恰三目标页×2 面 |

GONE 6 键（全真实页）：eval-no-csp-no-tt ×2＋eval-no-csp-no-tt-default-policy ×2＋eval-csp-no-tt ×2。三方对账：k40 post 62F（真实 54＋旧探针 7＋k40 探针 1）→ k41 post 55F（真实 **48**＋旧探针 7，k40 探针已删）＝真实 −6。

**TT residual 池真实 54F → 48F。累计 84,240 → 84,246（距 ≥99,947 差 15,701）**。
