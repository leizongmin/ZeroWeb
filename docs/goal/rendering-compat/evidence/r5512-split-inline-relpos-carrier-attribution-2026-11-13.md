# R5512 轮次证据：block-in-inline-relpos-002 载体晶格归因——split inline relative inset 载体修复落改

日期：2026-11-13
轮次：R5512（corpus FAIL 盘点择案轻量）
择案：block-in-inline-relpos-002（✗2.90%，684F 在册零提及，规范引用窄）
结论：根因＝split inline relative inset 载体晶格缺陷（tree.rs 无条件清零片段 inset 打破
converter R5054/R5225「taffy 单次施加」契约）；修复＝Inline 片段承 inset + Block 段
wrapper 承 inset + shell 清零（chromium oracle 双探针锚定）。002 ✗2.90% → ✗2.70%，
残差为 IFC 标记位嵌套（+20 统一偏移），归口 R4482 编织切片。

## 一、缺陷签名

- 002（✗2.90%）：`.relpos` div（display:inline; position:relative; left:40px）内含
  文本 B + 两个 block div + 双 float + 空 div + 尾文本 B。REF 侧所有随 left:40px
  位移的脸（首/尾 B、floatL、floatR、block border-left）整体 +40；ZW 侧位移整体
  缺失。001（✓0.58%）同族无块子形态，不受累。

## 二、归因链

1. **R109 split inline 载体晶格**（本轮全量摸清）：split inline（display:inline 有
   block 子）产 (a) shell 盒＝inline 自身 taffy 节点（SPLIT REL flags，node_id＝
   inline dom id，fragment_node_ids＝None）；(b) Inline 匿名块片段（fragment_registry
   在册，node_id＝inline dom id）；(c) Block 段（自有 dom id，直接 taffy 子）。
2. **位移双源冲突**：taffy 原生把 relative inset 施加于 shell（converter 对
   display:inline+relative 保留 inset——R5054/R5225 契约）；容器 IFC 行标记位在容器
   有自有文本时覆写 shell x（行流定位）。有 IFC 重写形态下 shell taffy inset 被标记
   位冲掉，tree.rs 4d39e8482 又无条件清零片段 inset → 位移整体丢失（002 形）；无
   重写形态 shell 载体正确（R5225 split 锚旧绿）。
3. **chromium oracle 双探针定谳**（`/tmp/r5512/`，headless 截图 + PIL bbox，方法见
   memory zeroweb-local-chromium-oracle-probe）：
   - R1044 形（span 纯块子、自身 relative）：green 面 bbox (0,0)——父 +100 与块子
     自身 −100 **均施加**（net 0，both-apply）；
   - R5225 形（span 有文本 + 块子）：span 行面 bbox (0,0,23,16) 且 contained block
     div bbox (0,18,49,27)——片段与 block 段**同受 −100**。
   - 即 css-position both-apply 语义：两层 inset 独立叠加；载体不得是 shell
    （shell 是 ZW 内部簿记盒，chromium 无对应物，bbox＝片段并集）。

## 三、修复（tree.rs 三处 + 锚测试）

1. `rel_parent_inset` 捕获（segments 循环前，仅 Relative + **`is_inline_r109`**；
   sticky 的 taffy inset 是滚动宿主层消费语义，不保留）。
   **门控教训**：segments 循环为 split-INLINE（`split_parents`）与 block-mixed
   容器（`block_mixed_parents`）共用；无门控版本对 block-mixed 宿主也触发——
   floats-placement-vertical-004-ref2（容器 relative left:100 + H 文本 + block 子）
   片段/wrapper 双承容器自身 inset（H/blue 全 200 vs 100，✗4.17%）、
   anonymous-boxes-001（relative 无 inset 值仍产 Some(AUTO)）空 div 被多套一层
   anon 节点改 % 解析链（✗2.08%）。两案 stash A/B 确证 HEAD 全绿、改后红、门控
   后需复验归绿。block-mixed 宿主自身 inset 由 taffy 施加于容器盒一次，子绝不
   继承。
2. Inline 片段臂：例外保留 inset（替代无条件清零；sticky 仍清零）——文本/float 脸
   位移经片段兑现。
3. Block 段臂：外套匿名块 wrapper 承 inset（块子自身 inset 不动）。直接注入曾覆写
   块子自身 inset（R1044 top:−100% 被冲 → lib 红），wrapper 避之——chromium
   both-apply 语义的盒树直译。
4. shell 清零门（split_parents ∈ ∧ 有 Inline 片段 ∧ Relative）：防双载体双重施加。
   纯块子形态（无片段，R1044）不触发——shell 载体保持，green 面 (0,0) 不变。
5. 锚测试 `r5225_rel_inline_split_offset_preserved` 分桶断言：shell＝100（仅外层，
   inset 不落壳）＋ Inline 片段＝0（−100 单次施加）——比旧单桶断言强（同时钉住
   不双施加与不丢施加）。
6. R1044 单测前提 chromium-wrong（测试期望 own-only，chromium both-apply net 0）——
   断言的是盒树相对值（green.y≈−100），两种语义下均真，本轮不动、只记账。

## 四、验收

- lib：2492P/0F（含更新后 R5225 split 锚；R1044 及 OOF 臂不变绿）。
- solo：001 ✓0.58% 稳；002 ✗2.90% → ✗2.70%。PNG 取证
  （target/reftest-dump/*relpos-002*）：B/floatL/floatR/block-border 全部统一 +20
  残差（IFC 标记位嵌套＝block 段与首 B 应落容器块流原点），位移轴单次施加全对齐，
  aqua/fuchsia/A 脸与 ref 逐像素一致。
- 残差归口：R4482「混排容器 anonymous block 几何合成」切片（block 段落容器块流 +
  §9.2.1.1 编织），9 案 harvest 潜力，本轮只落位移轴。
- make reftest 725：725/725 100%（无 inline 关系面不变绿；门控后复跑同值）。
- corpus 全量 A/B（16594 vs /tmp/r5509/new-fails.txt 684F 基线）：首跑（无门控版）
  捕获 2 真回归——anonymous-boxes-001 ✗2.08%＋floats-placement-vertical-004-ref2
  ✗4.17%，stash A/B 确证（HEAD 双 ✓0.00%）→ `is_inline_r109` 门控修复；复跑
  （corpus3）685F vs 684F，diffs＝tao-127/153＋gradient-currentcolor-radial-repaint
  翻绿，三案 solo ✓0.00%＝在册噪声族（tao 互翻族/gradient-repaint 噪声族，R5509/
  R5510 同签名），**真态 FIXED=∅ REGRESSED=∅**。仓外对端 clone 的 workspace make
  test 构建相位与批跑窗重叠（load 220-280 host 侧）为 flake 窗口背景。
- clippy：workspace --all-targets -D warnings（guarded-clippy）净（含 lib test 目标；
  首轮 3 处——2×clone_on_copy＋1×type_complexity——已修）。
- make test：全绿至 zero-engine 止 3911P；7F＝js_dom_bridge 对端在册族**恰集复现**
  （img/svg settle 2＋fetch/websocket/iframe-sw 5＝R5506 空载复核 10F 组成、R5504
  「7F＝对端 js_dom_bridge 在册族 base 复现归因」先例同集；本轮改动面
  tree.rs layout 零涉及，websocket/fetch 家族另值 IO 风暴窗 host b=295/wa=90）；
  renderer/browser 腿未跑属对端门禁债（R5504 先例口径）。首轮 make test 编译期
  rustc 被杀（零诊断级联 fail）＝host IO 风暴环境噪声，重跑即净。
- product-smoke：6/6 struct PASS（welcome 800/375/320＋morning 320＋form-input-perf
  PASS）＋welcome chromium oracle **17.23%**（R5501 基线同值，welcome 面无变化）。

## 五、本轮动作与卫生

- 临时诊断单测 r5512_tmp_relpos_diag（盒树 dump + no-left 变体，mod.rs 注册）用后
  撤净；chromium oracle 探针页 /tmp/r5512/{r1044,r5225}.html 在仓外。
- 第一修复尝试（Block 段直接注入）两 lib 红教训：注入覆写块子自身 inset（R1044），
  双载体双施加（R5225 锚）——wrapper + shell 清零定型后全绿。
