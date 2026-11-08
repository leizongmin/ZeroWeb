# R5464 证据：vrl definite 容器列组 CB 右缘锚定——修复 apply_inner restack Σ 基准发散，corpus 真修复 1、runner 域暴露红 1（oracle 实锤保留）

日期：2026-11-08。基线 HEAD：`319c47d63`（R5463）。驱动案：`sizing-orthog-htb-in-vrl-007`（R5463 挂账「vrl-007 残余 4.70% 块流基位域」）。

## 1. 归因：restack Σ 基准与 definite 容器宽发散（viv 嵌套域）

- **管线序勘误**（本轮关键认知修正）：`apply_vertical_block_flow`（restack，engine.rs ~1677）在 **11.12 正交子宽重解之后**跑，restack 用**新列宽**——007 形态 Σ=440（p1 20 + ob 400 + p10 20），列组贴 Σ 右缘：sb [420,440]、ob [20,420]、sa [0,20]，**溢出 CB 400 右缘 40px**。R5463 证据档「sb local [86,106]」为旧管线序下的观测，已被本轮实测修正。
- **树结构勘误**：007 页 body **继承** html vrl（writing-mode 可继承，页面只在 html 设 vrl）——cb 的 restack 属 viv（vertical-in-vertical）嵌套域而非正交父域；修复 gate 不能按 parent_wm 判。
- **机制**：`compute_vertical_block_flow` 排列基准 = Σ 子 outer 宽（第 69 行 content_width=Σ，VerticalRl: child[i].x = content_width − Σ_{0..=i}）；auto 容器 extent=Σ apply_inner 改写自洽；R4566 `width_definite_keep` 对 definite CSS 宽容器不改写物理宽 → **基准与容器 content 宽发散**，组贴 Σ 右缘而 CB 右侧露空隙（容器宽 > Σ）或溢出右缘（Σ > 容器宽，007 形态）。
- **chromium oracle headless 探针逐位定谳**（.xht XML + title 注入零流内污染）：007 精确形态 `cb=280,8,400,477 ob=260,8,400,126 sb=660,24,20,445 sa=240,24,20,445`——**sb 右缘 680 = cb content 右缘**（280+400）、ob [260,660]、sa [240,260] **溢出向左**（sa.left 240 < cb.left 280，不钳容器左缘）。CSS Writing Modes §7.1：vrl 块流起始 = CB 行内 start 侧 = content 右缘。

## 2. 修复：apply_inner definite 容器列组 CB 右缘平移（kill-switch `ZW_VRL_COLUMN_CB_ANCHOR`）

- 落点：`apply_inner` restack（`compute_vertical_block_flow` 调用）之后、子重定位之前——整组平移 `dx = b.content_width − layout.content_width`（margin-box 基准同移，DOM 序与列距不变；溢出向左允许负 x，与 compute_vertical_block_flow 的钳 0 有意不同）。
- gate：vrl + CSS 宽 definite（非 Auto，同 R4566 width_definite_keep 判据 = Σ 基准与容器宽的发散域）+ content_width > 0.5 退化守卫；dx |·|≤0.5 no-op。env 读一次在 `apply_vertical_block_flow` 入口（热路径不逐节点读）；apply_inner 签名加 `column_cb_anchor: bool`（单调用点）。
- 首版 11.13 独立 pass（11.12 后重排）**已废弃**：restack 在 11.12 后才跑终态位，晚 pass 与 restack 双写冲突；修订版落 restack 本体。
- auto 容器域零接触（gate 排除，016 0.00% 保持）；vlr 左缘锚定与容器宽无关镜像域不动（vlr 宽重解兄弟列重叠 FIXME 另账）。

## 3. 验证

- 常驻钉 4＋kill-switch 锚 1：`r5464_vrl_column_anchor.rs`（definite 组贴 CB 右缘 sb {380,−40}/ob [−20,380]；SHORT 无溢出组右锚 {158,380}/ob [178,380]；vlr 镜像零接触 {0,420}；auto cb 零接触 {0,222}）＋`..._kill_switch.rs`（=0 复现 Σ 锚位 ob [20,420]/列 {0,420}）。文件内 9 处单元测试 apply_inner 调用补 `true` 第 6 参（生产默认）。
- 驱动案：vrl-007 **4.70% → 4.61%** ✓（列锚正确化绿带内改善；残余主体 = body 在 vrl 容器下 auto 物理宽应 fit-content（oracle 400）而 ZW fill 600 → cb 绝对位差 ~184px，另账 FIXME）；**orthog-htb-in-vrl 17/17 + vlr 镜像族 20/20 全绿**。
- 内测 reftest **725/725**；fmt＋clippy（-p zero-layout-engine --all-targets -D warnings）清零；layout-engine 全腿 **lib 2444P ＋ 全部测试二进制 0F**。
- corpus kill-switch 双窗（同二进制 16594）：w1(ON) **689F** / w2(OFF) **696F**——comm 互翻＋solo×3 定谳：**真修复 1**（flexbox-writing-mode-009：OFF solo ✗1.50% 稳定红 → ON solo ✓0.86%）；**runner 域暴露红 1**（multicol-under-vertical-rl-scroll：OFF ✓0.00% → ON ✗1.12% solo×3 稳定——oracle 实锤 B A-local [106,186] 右缘 = content 右缘（scrollbar 13px 后）、D [−394,106] 溢出向左＝修复语义逐位正确；OFF 绿系 ref 臂 `right:0` 与 Σ 锚位重合＋runner 不执行 `scrollLeft=800` 脚本的双臂巧合自洽，案面红暴露 runner script/scroll 能力缺口，修复保留）；ON 独红另 1 案 insert-block-001 solo×3 全绿 = 噪声（观察名单成员）；OFF 独红另 8 案（tao-129/145/148/152/154＋cv-099＋iprd-005/008）双窗 solo 全绿 = 全量窗噪声指纹家族。
- product-smoke PASS（welcome struct 六断言＋morning 结构门＋form-input-perf-gate）；bench-gate 定向 zero-layout-engine 静窗（1min load 2.28 入窗）**GATE PASS 25 指标（NEW=0）**，morning layout 28.38ms（预算 67.28）。

## 4. 残差与挂账（未了结债）

- **vrl-007 残余 4.61%**：断行/约束/列锚三域已修，余差主体 = **body 在 vrl 容器下 auto 物理宽（块轴）应 fit-content（oracle 400）而 ZW fill 600** → cb 绝对位差 ~184px（ref 臂 table viewport 绝对定位，test 臂随 cb 位）；html/body sizing 链旧账（在册 FIXME「html root sizing 链低优先」）的 concrete 形态，有驱动案集中时开案。
- **multicol-under-vertical-rl-scroll**（新挂账）：runner 不执行 ref 臂 scrollLeft 脚本 → 双臂一致域巧合绿不可持续；修复后该案红为 runner script/scroll 能力缺口（oracle 证 ZW 几何已对齐 chromium）。归 wpt-runner 能力域，非 layout 缺陷。
- **vlr definite 容器宽重解兄弟列重叠**（新观测 FIXME）：vlr 组左起累进、11.12 宽重解不位移 x → ob 增宽吞 sa 列位（vlr 钉 {0,420} 记录现行态）；vlr 族现行 20/20 绿（diff 带内），有驱动案再收。
- **维持**：R5461 abs AR 臂 frame 双计 FIXME；%/关键字 CB、显式宽正交子、inline 元素子、% margin resolve 各 FIXME；c5525-fltmrgn-000 ✗3.92%；R5447 collapse ½border；㉗-㉘/R5432 销账面/deep intrinsic 48 变体矩阵/**待用户决策清单**①-⑤（无新增）；对端 js_dom_bridge 红腿观察；deep_nesting 基线漂移观察；R4213/R1099/inline_finalization 2000 行/revert-layer-014/正交 inline-block htb 叶/R5457 窄门/definite-height 混合内容容器/块子混排 viv extent＋inline 级混排等既有账。

## 5. 取证命令存根

```bash
# 驱动案 solo（ON/OFF 态）
make reftest-upstream FILTER=sizing-orthog-htb-in-vrl-007
ZW_VRL_COLUMN_CB_ANCHOR=0 make reftest-upstream FILTER=sizing-orthog-htb-in-vrl-007
# chromium oracle（.xht XML + title 注入，零流内污染）
google-chrome --headless=new --dump-dom file:///tmp/r5464-oracle/probe7.xht | grep -oE '<title>[^<]*</title>'
# corpus 双窗（剥 ✗ % 后 comm 互翻；-13 = OFF 独红修复候选，-23 = ON 独红回归候选）
make reftest-upstream 2>&1 | grep '^  ✗' | sed 's/ ([0-9.]*%)$//' | sort > /tmp/w_on.txt
ZW_VRL_COLUMN_CB_ANCHOR=0 make reftest-upstream 2>&1 | grep '^  ✗' | sed 's/ ([0-9.]*%)$//' | sort > /tmp/w_off.txt
comm -13 /tmp/w_on.txt /tmp/w_off.txt   # OFF 独红
comm -23 /tmp/w_on.txt /tmp/w_off.txt   # ON 独红
```
