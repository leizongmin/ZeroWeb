# kc5r85 verdict——CSSOM 声明块 column-wrap 简写复位物化单刀（F2P=2 精确／P2F=0）

日期：2026-11-24。face＝**part06.js 单文件 α 刀**（r85 前证轮 b267f0d71 姊妹流
落档＋实施轮接续流认领，NOTES-kc5r85.md §零 仲裁）：

- `_zwColumnsResetWrapValue` helper（:2456 前）：columns 值→wrap 复位读值——
  合法形返 'auto'、css-wide 关键字小写透传（inherit/initial/unset/revert/
  revert-layer）、无效/函数形返 null；形门与 style-system `expand_columns`
  同构轻口（FIXME: 完整文法单源在 Rust 侧，JS 轻门覆盖语料形）。
- `_makeRuleStyle.readProp` column-wrap 位置感知扫描臂：声明块源序跟踪显式
  wrap 值与 `columns` 复位点，末位胜者（css-cascade 块内序语义；css-multicol-2
  §4.5 columns 展开含 reset-only column-wrap）。仅 'column-wrap' 名变臂，
  域外字节恒等；cssText/item()/length 不动（声明原文保留简写形，cssom-1）。
- 指纹 'kc5r85' ×2 行（include_str! 嵌入 bin strings）。

bins：b0＝r85-b0-runner **593e7f16**（HEAD b267f0d71 基线＝码面 e134036a5 含
r84 face，strings 'kc5r85'=0 双向门）、b1＝r85-b1-runner **fb6d5550**
（Compiling zero-engine=1＋bin 指纹 2＋md5 异）。stash 舞步 face md5
ed572805 恒等恢复；电池发射行与收账时树面恒等。

## 根因（单臂，前证修正）

`sheet.rules[i].style` 走 part06.js:3043 `_makeRuleStyle`（wire 通道 StyleRule
物化；insertRule 同构 :2652）——其 readProp/findIdx 为名匹配（末次不识字）不
识简写覆盖：multicol1 简写-only 块 getter 恒 ''、multicol2 前置显式 wrap 不被
后位 columns 复位。**前证轮 β 静态靶点（expand_columns 不发 wrap 复位）被实施
轮探针直证驳倒**：gCS-only 临时页三键全 Pass（probe-results/gcs-probe-b1.json）
——静态级联已按 expand_columns 尾槽复位（shorthand/mod.rs:2677 推槽＋lib.rs:2976
级联消费＋:901 派发），r84 探针 b1 multicol2 'nowrap' 系首抛 CSSOM 断言、gCS
断言从未达。姊妹 NOTES α 面归因（part03.js `_styleProxy`/`_zwMStyleProxy`）亦
勘误：该两面板为 element inline 面，本页不读。

## 探针门（发射前，probe-results/ 实证）

- gCS 前证探针（tmp 页跑后核删）：gcs-multicol1/2/3 三键全 Pass＝CASCADE-OK。
- 真页双臂（probe-r85.sh）：b0 形 multicol1 F＋multicol2 F＋multicol3 P（新
  基线守卫键先证）→ b1 三 test 全 P＝**PASS**。

## 电池与 judge（六腿全 RC=0，17:32:03–17:56:02；systemd transient
MemoryMax=24G/RuntimeMaxSec=7200，flock /tmp/zw-kc5r85-battery.lock，JSON_DIR
唯一写者）

- **multicol** total=1613 双侧恒等 P→P=1406 **F2P=2 精确命中**（±2 窗）、
  P2F=0、OTHER 页 0＝G1 全对；键级双算独立直算与 judge 逐字吻合。F2P＝
  multicol1+multicol2 恰预注册清单；multicol3 P→P 守卫键持位。
- **values 哨兵** n=9882 **SET-IDENTICAL 9882/9882**（random-computed 双稳
  采样键本代未翻面——双稳第六代观测）。
- **transitions 哨兵** n=1506 diffs=6：truncation-asym 5（event-dispatch 1＋
  color-mix 4＝r83/r84 族页集复现）归档；properties-value-inherit-003
  background-position-x '/ values' Pass→Fail 1＝四分类归因
  **boundary-sampling flake**（同 b1 bin 页过滤复跑 3×全 Pass probe-results/
  p2f-recheck/＋刀面域隔离＋同页 12 同机制姊妹键持位＋timing 消息形
  `got disallowed value "33px"`）。
- **双 judge 如实记**：judge-r85.out＝预注册门 as-run **VERDICT FAIL**
  （hard-P2F=1＝预测账精度缺口，未预注册该族条款）；姊妹流电池出账后追加
  `race_family` 归档条款（引 r82 judge/NOTES §61 族预注册＋§117 P→F 方向
  同 bin 复跑先例，本 session 独立核引用成立）→ judge-r85-amended.out＝
  **VERDICT PASS**（hard-P2F=0／race-family=1／truncation-asym=5 归档）。
  净判读 **PASS-with-attribution**（r81c dice-attribution 先例形）；as-run
  FAIL 不回填（NOTES §八 碰头记账）。
- G4 at 0.5 边界 F2P 键=0。

## 残面（r86 候选）

1. inline 面 `_zwMStyleProxy`（part03.js）同族简写覆盖读缺陷——语料现无
   读者，带上须另轮预注册。
2. 缺语料页 column-wrap-interpolation.html 补页（'wrap'/'nowrap' 端点组，
   守卫③扩面另轮预注册）。
3. neutral at==endTime 精确边界（r83 移交，独立非本刀域）。
4. transitions timing 边界敏感族窗界容差条款预注册化（properties-value-*
   前缀族已由姊妹流 judge 修订入 r85 档，跨轮沿用须正式预注册）。

链证据：.acceptance/wptfyi-css/kc5r85/（NOTES 全链＋bins＋results-b0/b1＋
judge-r85.out＋judge-r85-amended.out＋probe-results/gcs-probe-b1.json＋
p2f-recheck/ 六快照）。
