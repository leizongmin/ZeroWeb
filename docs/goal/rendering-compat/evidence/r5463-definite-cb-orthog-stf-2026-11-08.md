# R5463 证据：definite-CB 正交块 §7.1 约束扩展＋stf intrinsic 基校正——vrl-007 5.02%→4.70% 翻绿，双族 100% 全绿，corpus 净 −1F 回归 0

日期：2026-11-08。基线 HEAD：`8d9fb462d`（R5462）。驱动案：`sizing-orthog-htb-in-vrl-007`（R5462 挂账「definite-CB 域 R5463 首选」）。

## 1. 归因：R5462 definite-CB 门排除域＋stf intrinsic 基错位（frame 双计）

- **definite-CB 域塌陷**（探针三变体：FULL/SIMPLE_NOMARGIN/SIMPLE_MARGIN100 同形，临时已删）：R5462 pass gate「父 style 宽 Auto」排除 definite 域 → taffy 交换帧旧测量路径维持塌陷（正交块 w=66/cw=60 按 3 数断行、h=784 塌 ICB fill）；父 cw=400 **definite 已被 taffy 解出**（探针 dump 证），仅子臂未重解。body 水平 margin 100px / sentence 前后均不影响塌陷形态。
- **§7.1 语义**（007 meta 断言原文）：CB width definite 400 → constraint = min(available=400, ICB=800) = 400。available = CB 行内尺寸（物理水平 = 父块方向 extent；vrl 父 style width 即该轴）。
- **chromium oracle**（headless=new，.xht XML + title 注入零流内污染）：`OBX=260 OBY=8 OBW=400 OBH=126`＝border-box = constraint 400、内容 6 行 × 20 + border 6。**OBH=126 与 ZW 修后逐位同**（同字体度量 6 行）。OBX=260 与 ZW 的 sibling 列位差（ZW p1 local [86,106] vs oracle [364,384]）＝vrl 块流基位/列 restack 另账（ref 臂 table absolute 同形模拟，self-source 判定两侧一致域）。
- **stf intrinsic 基错位**（判别矩阵 border × CB 域定谳）：`block_max/min_content_width` 返回 **border-box 基**（inner + 自身 frame，R5368 float 外宽消费口径），R5462 stf 公式以 content 基消费再外加 frame_h → **frame 双计**。SHORT 域实证：border 3 maxc=202 vs border 0 maxc=196（差恰 = frame 6）；R5462 SHORT 钉锁值 336 = 196+70+70 双计（FULL 域 maxc > available 恒被钳，双计被掩盖——016 仍 0.00% 的原因）。

## 2. 修复：definite-CB constraint 分支＋调用点剥 frame（ZW_ORTHO_BLOCK_STF 域内扩展）

- gate 扩展：父 style 宽 `Auto → ICB`（原路径）；`Px → min(父 content_width, ICB)`（**definite 分支**——父 LayoutBox content_width 在 11.12 时刻已由 taffy 解出 definite 值，观测 parent_cw=400/900 证；CB900 变体 constraint=800 证 min 钳生效）；`%/关键字 → return`（FIXME）。匿名父保守跳过维持。
- stf 基校正：`minc/maxc = intrinsic_sizing(...) − frame_h`（调用点剥，函数语义不动——float/R5368 消费方按外宽贡献消费自洽）。
- action 不变：available_border = constraint − Px margin；stf = min(maxc, max(minc, available))；水平 IFC with_vertical(false) 断行；width/height/content_* 写回；父 WM 判向 x 位移。

## 3. 验证

- 常驻钉 4＋kill-switch 锚 1：`r5463_definite_cb_orthog_stf.rs`（007 精确形态 400×126/cw=394/ch=120＝oracle OBW/OBH 逐位同；CB900 constraint 钳 ICB 800×66；SHORT STF 收缩 202×26 < CB；vlr 镜像 400×126 x=sentence 列 20 后不动）＋`..._kill_switch.rs`（=0 复现 definite-CB 旧塌陷 **66×784**＝旧行为失败证据）。
- R5462 钉随基校正勘误：SHORT 336→**266**（196+70 正确值）；definite-CB 分支改断言新语义 266（原「不触 90」失效＝本轮预期行为变化）；文档头加 R5463 注记。
- 驱动案：vrl-007 **5.02% → 4.70% ✓ 翻绿**（阈值内；残余 4.70% = vrl 块流基位/sibling 列位域另账）；**orthog-htb-in-vrl/vlr 双族 100.0% 全绿**（007..012 definite-CB 批次全 ✓：008 2.03%、009 1.02%、010 4.36%、011 1.61%、012 0.63%）。
- 内测 reftest **725/725**；fmt＋clippy（-p zero-layout-engine --all-targets -D warnings）清零；layout-engine 全腿 **lib 2444P ＋ 33 测试二进制 0F**（含 R5462 钉 5+1勘误、R5463 钉 4+1；临时 probe 提交前已删）。
- corpus kill-switch 双窗（同二进制 16594）：w1(ON) **691F** / w2(OFF) **692F**——comm 互翻：OFF 独红 4（vrl-007＋016 双案 kill-switch 回塌＝R5462/R5463 域语义）＋ON 独红 3（insert-block-003＋tao-147/152）；movers solo×3 定谳：vrl-007 OFF solo **✗5.02% 复红**（env 穿透＋因果实锤）、intrinsic-percent-replaced-dynamic-005 双窗 solo 全绿＝纯噪声、ON 独红 3 案 solo 全 ✓0.00%＝全量窗噪声指纹家族（insert-block-003 新入观察名单）＝**净 −1F、净回归 0**。
- product-smoke PASS（welcome struct 六断言＋morning 320 结构门＋form-input-perf-gate）；bench-gate 定向 zero-layout-engine 静窗（1min load 2.39 入窗）**GATE PASS 25 指标（NEW=0）**，morning total 53.53ms（预算 2000ms）。

## 4. 残差与挂账（未了结债）

- **vrl-007 残余 4.70%（及 008-012 亚 5% 带）**：断行/约束域已修，余差 = vrl 块流基位/sibling 列 restack 域（探针证 p1 列位 local [86,106] vs 块流起始应在 CB 右缘；oracle OBX=260 同证）。有驱动案集中时开案。
- **R5461 abs AR 臂同病观测**（abspos_sizing.rs:171/219）：`block_max_content_width` 返回含 frame 值被当 content 基 + frame_h 消费——同款双计形态；现行 AR 案面绿（maxc 被 cb_w 钳或 frame=0 页面），无稳定红，FIXME 有驱动案再收。
- **% /关键字 CB**（constraint 源 %/fit-content 等）域 FIXME；显式宽子内容断行塌陷域（R5462 挂账）维持；inline 元素子域维持；% margin resolve FIXME 维持。
- **维持**：c5525-fltmrgn-000 ✗3.92%；R5447 collapse 显式臂 ½border 同口径核查；㉗-㉘/R5432 销账面/deep intrinsic 48 变体矩阵/**待用户决策清单**①-⑤（无新增）；对端 js_dom_bridge 红腿观察（九案）；deep_nesting 基线漂移观察；R4213 行组参照系再对齐未镜像（FIXME）；inline_finalization 超 2000 行拆分挂账；revert-layer-014 构建/环境时代漂移观察；正交 inline-block htb 叶旧 vertical 测量路径 FIXME；R5457 窄门 FIXME（多列容器块流位随行号左移/VerticalLr 镜像）；R1099 vertical advance 开门待视觉消费驱动案；definite-height 混合内容容器（探针 H 域）；块子混排 viv 容器 extent（R109 片段域另账）＋inline 级元素子混排（R4469 原子注入口径）；html root sizing 链低优先。

## 5. 取证命令存根

```bash
# 驱动案 solo（ON/OFF 态）
make reftest-upstream FILTER=sizing-orthog-htb-in-vrl-007
ZW_ORTHO_BLOCK_STF=0 make reftest-upstream FILTER=sizing-orthog-htb-in-vrl-007
# chromium oracle（.xht XML + title 注入，零流内污染）
google-chrome --headless=new --dump-dom file:///tmp/r5463-oracle/probe.xht | grep -oE '<title>[^<]*</title>'
# corpus 双窗（剥 ✗ % 后 comm 互翻）
make reftest-upstream 2>&1 | grep '^  ✗' | sed 's/ ([0-9.]*%)$//' | sort > /tmp/w_on.txt
ZW_ORTHO_BLOCK_STF=0 make reftest-upstream 2>&1 | grep '^  ✗' | sed 's/ ([0-9.]*%)$//' | sort > /tmp/w_off.txt
```
