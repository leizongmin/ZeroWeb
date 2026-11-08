# F6b sub/sup×fontsize carry 刀前分析（2026-11-08，③ F6b 开刀轮）

## 活体普查（f1b-mt-post.log，F1'-B 后二进制，main 域）

F6b 簇 14 键（knife11 普查口径：sub/sup+delete/fwddelete 4 + sub×fontsize 互序 4 + delete 族 sub/fontsize 嵌套 6）。**X37 两键（subscript>formatblock qS/qV）已定谳窗界伪影除名，不在本刀范围**。live 实测期望/现状逐键：

### C 族：sub×fontsize 同案互序（4 键，?8001-9000）

| 序 | 期望（语料） | 引擎 got |
|---|---|---|
| sub→fontsize(2)→type | `foo<sub>a[]</sub>bar`（**无 font**） | `foo<font size="2"><sub>a</sub></font>bar` |
| fontsize(2)→sub→type | 同上 | 同上 |
| sub→fontsize(3)→type | `foo<font size="3"><sub>a[]</sub></font>bar` | `foo<sub>a</sub>bar`（font 丢） |
| fontsize(3)→sub→type | 同上 | 同上 |

### A 族：sub/sup+delete/fwddelete+type（4 键，?1001-2000 / ?2001-3000）

| 序 | 期望 | 引擎 got |
|---|---|---|
| sub→delete→type | `fo<sub><font size="3">a[]</font></sub>bar` | `fo<sub>a</sub>bar`（font 丢） |
| sub→fwddelete→type | `foo<sub><font size="3">a[]</font></sub>ar` | 同构丢 font |
| sup→delete→type | `fo<sup><font size="3">a[]</font></sup>bar` | 同构丢 font |
| sup→fwddelete→type | 同构 | 同构 |

### N 族：delete+type 在 sub/fontsize 嵌套树内（6 键，?8001-9000 / ?9001-last）

| setup | 期望 | 引擎 got |
|---|---|---|
| `foo<font size=2><sub>[bar]</sub></font>baz` | `foo<font size="1"><sub>a[]</sub></font>baz`（**size 2→1**） | `foo<font size="2"><sub>a</sub></font>baz` |
| `foo<sub><font size=3>[bar]</font></sub>baz` | `foo<font size="3"><sub>a[]</sub></font>baz`（**换序 font 外**） | `foo<sub>a</sub>baz`（font 丢） |
| `foo<font size=3><sub>[bar]</sub></font>baz` | `foo<sub>a[]</sub>baz`（**font 丢**） | `foo<font size="3"><sub>a</sub></font>baz` |
| 尾选区三形（[bar</sub></font>baz]） | 同上三形对应尾版 | 同构 |

## 机制考古

1. **语料=手编 spec 期望，非 Chrome 录音**。multitest.js 案例块带编辑意图注释（`// <font> should be outer-most element`，line 2065/2179/2185）。multitest 页面加载即 `styleWithCSS=true`（工具页头注释披露录音生成上下文的顺序状态携带），per-command queryCommand 期望值内嵌。
2. **spec 参考实现 oracle 实测**（include/implementation.js `myExecCommand`，headless Chrome 驱动，2026-11-08）：A 族 3/3 与语料逐字节吻合（deletion typing style 机制实锤：delete 后 typing style 显式携带 fontSize 16px，typing 组合 `<sub><font size=3>a</font></sub>`）。**C 族/N1/N2 与参考实现有分歧**：参考实现 C 族一律 `<sub><font size=N>`（font 内）、N1/N2 也是 font 内；语料要求 font 外（editorial 规则）+ C1/C2 的 size-2 丢弃。语料是绑定目标。
3. **size 等价跳过规则**（从 N1 size 2→1 反推）：spec fontSize px↔legacy 桶映射 13px→size 1、16px→size 3；`<sub>` 内 effective = smaller(16px)≈13.3px → 与指定 13px 同桶 → 等价跳过 wrap（C1/C2 无 font 的机理）；16px ≠ 13.3px → wrap（C3/C4）。N1：删除位 effective 13px（font size=2 上下文）→ typing style 记 13px → 重发 size 1。
4. **A vs N 嵌套序矛盾**（语料自身）：A 族（裸文本+空壳删除）font 内；N 族（嵌套树内删除）font 外。判别特征=删除位是否在既有壳内。引擎需按上下文分派两种序。
5. 引擎现状 systematic 反转：N3（应丢）引擎发、N1（应换 size=1+换序）引擎保 size=2、C 族两序都错——删位 typing style 记录源与组合序双缺陷。

## 悬案与下一步

- N4-N6（尾选区）oracle SETUP-FAIL：probe 标记解析器未支持 `[bar` 无闭合 ] 形；尾版期望=闭版对应尾版（语料 line 2288+），实现时逐块核对语料原文即可。
- shim 侧待读：sub/sup 臂在 part06.js + part01f.js；fontsize 臂在 part01c/part01d/part06/part01f。F1'-B 刚动过同域组合机制（D4 拆壳臂/前向并壳/swcss span 吸收），新刀须避开回归其收口面（f1b 门禁锚 part01=a82df7cf/part01d=5ab2368e/part06=5e78d9a4）。
- 设计方向：删除侧 typing style 记录源改为删除前 effective 值 + spec px→legacy 桶重映射；组合序按「删除位壳内/壳外」上下文分派（font 外 editorial 序 vs font 内 typing-style 序）；指定值 vs typing style 双通道序差（C3/C4 font 外 vs A 族 font 内）。
- 验证：probe 十四案（含尾区三形）+ mt 全分母 judge（F2P 窗口以 14 键为预期）+ iol/iul/enginelib/battery 残余批，沿用 F1'-B 门禁批骨架（batch-f1p-b-typing-wrap-gate.sh 修正 WL_PAT v2 后的定长 grep -F 白名单门）。
