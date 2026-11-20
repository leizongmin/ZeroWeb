# DC-2 · u/s decoration 载体作者臂（2026-11-12）

P2 挂账簇 4（8 P2F）清账 + inline 载体双形解挂：gCS 作者样式表规则臂
（state 读）+ forceValue u/s 载体 span 识别（apply 写面）。

## 修复面

- **state 读**（`part01c _zwFmtDecorationOf`）：单元素 text-decoration
  own 值走查插作者臂——`_zwPlainAuthorDecl`（复用 P2 ③'-c 作者快照
  匹配机具，同款有界简单选择器域），读序 host → inline → **作者表** →
  UA 表（级联序作者 > UA、inline > 作者；非继承不爬升不变）。
  host gCS 对 `<link>` 作者样式表盲、reset.css
  `.line-through/.underline/.underline-and-line-through` 类载体 span
  原先 state 读 false 误应用外裹——本臂闭合。
- **apply 写面**（`part01d _zwVaForceValue`）：u/s 命令对全含选区的
  decoration 载体 span（inline decl 或作者规则面）新增落点门
  （vendored 录形注释原文 "Should update text-decoration declaration
  in the <span> which is a container of the range"）：
  - css flag true：原位并 style——新值与 inline 既有 line token 合并按
    canonical 位序（[underline, overline, line-through]）输出；作者
    token 不入 inline 面（class 继续级联供给）。
  - css flag false：inline decl 面 → 跳过 ⑦ 下潜、落尾臂外裹
    （`<u><span style=…>…</span></u>` 录形）；作者面（类壳）→ 下潜
    内容侧（`<span class=…><u>…</u></span>` 录形）。
  - 裸 span（无 decoration 面）不入本门，跨 span 裸壳组录形零漂移；
    ⑦ 下潜臂以 `!usSpan || !tdUsFace` 门保持原路径。

## 方法

同树前后二进制对跑（test-guard/systemd scope 包裹，
`ZW_SCRIPT_TIMEOUT_MS=300000`），断言级（全子测试名）前后 diff：

- base = `dc2us-base-runner`（HEAD 284a7bbb4，rebase 前）
- after = `dc2us-after2-runner`（rebase 后当前树 rebuild；含 css-stream
  4 commit f06979d02 等——css-values 数值维度与 editing shim 零写面
  重叠，非 u/s 文件逐案计数恒等佐证归因干净）
- 面 1 脚本 `dc2us-face-run.sh`（六 format 文件 + 8 探针，同 P1/P2
  口径）、面 2 `dc2us-mt-run.sh`（multitest + misc 哨兵）。

## 数字

面 1（六文件 + 探针）：usbase 11,722/279 → usafter2 **11,748/253**
（+26P / −26F）：

| 文件 | base P/F | after P/F | ΔP |
|---|---|---|---:|
| underline | 2101/46 | 2115/32 | +14 |
| strikethrough | 2055/92 | 2067/80 | +12 |
| bold/italic/sub/sup | 恒等 | 恒等 | 0 |
| 探针 8（tstyle 1F、mtmini 1F、p2diag 5F 既有诊断） | 恒等 | 恒等 | 0 |

- 断言级：**F2P=26 / P2F=0 / newF=0**。26 全为载体簇：类载体
  `class="line-through"/"underline"/"underline-and-line-through"`
  （作者臂，innerHTML + qS-before 双断言）+ inline 载体
  `style="text-decoration:…"`（css-true 原位并 + css-false 外裹双形，
  含 blink/blue dotted 长手丢弃形）。
- 原 P2 簇 4 的 8 P2F 名单全部落在 26 F2P 内（P2 fix1 与 usbase 六
  文件逐案 P/F 恒等 → 其失败集为本轮 base 失败集子集，类载体失败
  全部翻转）。

面 2（multitest + misc 哨兵）：usbase 10,166/24/0T → usafter2
**10,310/24/1T**（断言级逐名 diff：**P2F=0 / newF=0 / 丢失=0**）：

- misc 401/22 恒等、multitest F=2 恒等名——共享断言集合前后逐名一致。
- multitest +144 P 与 1 T 全部落在 `?9001-last` 慢页（multitest-ab，
  f95fdd4d9 perf 修复前的树）：after2 本跑多推进 144 断言后该脚本撞
  300s 门记 Timeout；base 同页更早门杀、后半程断言未入账（P2 已记
  「multitest +380（F 恒 2）」同款门杀非确定性）。144 新 Pass 逐名核对
  零 u/s 载体相关（bold/fontName/delete/italic 序列），归因门杀进度差
  而非本刀行为漂移。

## 残余与下一步

- strike 80F / underline 32F 长尾 = P2 既有清单：多值 decoration 偏序
  删除（`foo<s>[bar]</s>baz` 族，clearValue FIXME 挂账）、跨段落
  merge、table 选形、styled-child wrap。
- 批入账前 pull --rebase（28+ 并行流入：P39 epoch 章/tag stamp 快道/
  CE walk tagHint 等 engine-shim perf + css g13 批）后 rebuild
  复验 u/s 两文件：underline 2115/32、strikethrough 2067/80 恒等
  ——+26 存活，归因干净。
- 粗体/斜体值面作者臂（bold/italic 的 class 载体）未接——语料当前零
  `<style>` 消费面，挂后续。
