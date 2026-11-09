# R5457 证据：#33 slice-2 float §9.5.1 右缘锚定（垂直父语境）——RL 容器孤立正交 float 块流起始重锚

日期：2026-11-08。驱动案：`css/css-writing-modes/float-lft-orthog-htb-in-vrl-002.xht`（R5456 后 ✓2.90% 近似带）＋同构 `float-rgt-orthog-htb-in-vrl-003`（✓4.43%）。挂账：#33 slice-2（R5456 收窄后切片一）。

## 1. 归因（chromium oracle 双态探针 + ZW LAYOUT_DUMP 定谳，800×600）

chromium（headless=new，.xht XML 解析 + 注入 getBoundingClientRect；第一版探针以 .html 复制后 `<![CDATA[` 非 CDO token 整表失效回退默认样式的教训存档）：

- test 臂：html=[572.52,0,227.48,513]（BFC 根 shrink **含 float 溢出**）；.vp=[736,8,56,497]（非 BFC 中间父 shrink **不含 float**，w=56=border 16+单列行高 40）；float:left=[572.52,16,211.48,40]——**右缘 784 = .vp content 右缘，float 向左溢出列宽**；行盒被推到 float 下（y=56）。
- ref 臂：abs=[572.52,16,211.48,40]（right:16px → 右缘 784）；span top:40 相对偏移 y=56。
- 003 float:right=[554.72,457,229.28,40]——右缘同 784。**单列容器中 float:left/right 双方向块流位（右缘）一致**（WPT 断言 as far to the right as possible 同证）。

ZW 修复前（R5456 HEAD）：002 float:left content 系 x=0（taffy 交换帧按逻辑 y 左起排，LR 语义）→ 物理 x=784 右出视口被裁；003 float:right 因 taffy 右 float 臂已贴右缘偶然正确。in-flow 子右锚已由 R4429 `anchor_vrl_root_children` / viv restack 覆盖，float 子被 R4428 viv float 子树排除显式跳过＝缺口。

## 2. 修复体

- `vertical_block_flow.rs` `anchor_orthog_float_rl_start`（pub，step 12.7 调用点 anchor_slr_root_children 之后；RL 样式页 `styles.values().any(VerticalRl)` 预扫描 fast-path，R4460/R4217 先例）＋内部 walk＋gate 纯函数 `rl_single_orthog_float_target`。
- 动作：`float.x = b.content_width − float.margin_right − float.width`（margin-box 右缘贴 content 右缘；允许负 x＝左溢出，chromium 同形；0.5px 浮点噪声阈值不动）。
- gate（窄门）：容器 wm=VerticalRl＋非 float/abs/fixed/layout-container（R4475 坐标口径：非 float 容器子坐标 content-box 相对）＋恰一个流内 float 子且无其他流内子（ortho-htb-alongside-vrl-floats 族混排容器不触，R4428 taffy 几何自洽域维持）＋float 子自身正交 htb（RL float 于 RL 父属 viv float 域）。
- 家族保护实证：ortho-htb-alongside-vrl-floats-002（body 3 block 子混排）gate 排除；RL float 于 RL 父（viv float 族）正交门排除；clearance-calculations-vrl-008 同理不触。
- 窄门单列近似 FIXME：多列容器块流位随行号向左推进，本臂恒按首列右缘锚（有驱动案再探）；VerticalLr 镜像（LR 块流起始=左缘=taffy 现状） FIXME。
- kill-switch `ZW_ORTHO_FLOAT_RL_ANCHOR=0`（default-on）。

探针验证：修复后 test 臂 float 物理 x=573 w=211 → 右缘 **784 = .vp content 右缘**（chromium 572.52..784，左缘差 0.5px 级＝宽度舍入）。

## 3. 常驻测试

- `crates/layout-engine/tests/r5457_orthog_float_rl_anchor.rs` 3 钉：①float:left 右缘贴块流起始（content 系 `x+w == content_width`，对 html/.vp 塌宽债稳健）＋R5456 钉维持（行高 40＋262 域）②float:right 同判例（块流位与方向无关）③gate 排除钉——float+流内块子混排容器 float 维持 taffy 原始 x≈0 不右锚。
- `..._kill_switch.rs`（独立进程，R3858 教训）：`=0` 复现旧 taffy 左起排 x≈0（旧行为失败证据）。

## 4. 门禁矩阵

- fmt clean；clippy `-p zero-layout-engine --all-targets -D warnings` 绿。
- layout-engine **lib 2444P / 0F / 2I**（R5455/R5456 基线逐位同）；tests 全腿 0F（含新钉 4 测）。
- `make reftest` **725/725**（Layout 521 + Text 204）。
- corpus kill-switch 双窗（同二进制）：w1(ON) **695F** / w2(OFF) **699F**；互翻 10 案（P→F 3：tao-128/148、iprd-008；F→P 7：ibbim-003、bcdc-001、tao-127/147/153/154、ocmc-dynamic）全落 table-anonymous-objects / border-collapse / dynamic 负载噪声指纹家族，**solo×3 全 ✓✓✓**（ON 态同二进制）＝**新增稳定红 0**。净 −4F 噪声级，不进代码归因。
- 驱动案 solo：002 ✗2.90%→**✓3.26%**（同近似通过类；+0.36pp＝float 右移归位后与未修行盒列的重叠面变化——行盒 inline 轴 float 排除债未修，对错误 ref 的正确偏差，R5442 判例域）；003 **✓4.43% 持平**（右缘本就贴，重锚 8px 级微移）。
- product-smoke 全 PASS（struct 六站＋form-input-perf-gate）。
- bench-gate（`ZERO_WEB_BENCH_CRATES=zero-layout-engine`，静窗 load=7 入窗——并行 clone 持续构建期以等待轮询＋bench-report 自带 loadavg ABORT 双保险守纪）：**GATE PASS**，25 指标全预算内；`block_layout_1000_elements` 3.5499ms vs 基线 3.5431ms（**+0.02%**，预算 4.7832ms）＝新增 RL 预扫描＋env 门零可见成本（R4460 同型扫描先例量级）；`deep_nesting_50_levels` 302.3µs vs 基线 297.2µs（+1.7%，预算 401.2µs）。

## 5. 残余与挂账更新

- **勘误**：R5456 证据 §5「ref 臂 abs 蓝框 x=−83 出视口不绘制」为修复前态误作残余——R5456 测量臂落地后当前 HEAD ref 臂 abs x=695 w=89，右缘 784 已正确（绘制中）。真残余＝**宽度域** 89 vs chromium 211.48（taffy absolute 交换帧 sizing 收窄，R5455 家族域）。
- #33 slice-2 剩余：行盒 inline 轴 float 排除（推压机制在但未生效，.vp 行盒列未让位）＋html shrink-to-fit 含 float 溢出（BFC 根 extent，ZW html w=32 vs chromium 227.48）＋.vp extent（行盒列 40 贡献缺失，ZW w=16 vs chromium 56）＋taffy absolute 交换帧 sizing（ref 臂 abs 宽度）。
- 002/003 双臂同真绿（≤strict）未达：float 归位后残差主项移到行盒排除域（float 蓝盒与文字行盒重叠）＋extent 塌宽。

## 6. 取证命令存根

```bash
# 驱动案 solo（002 ✗2.90%→✓3.26%；003 持平 4.43%）
./target/release/zero-wpt-runner reftest-upstream float-lft-orthog-htb-in-vrl-002
# 布局 dump（float x=573 w=211 右缘 784；ref 臂 abs x=695 w=89 勘误取证）
LAYOUT_DUMP=1 ./target/release/zero-wpt-runner reftest-upstream float-lft-orthog-htb-in-vrl-002
# corpus 双窗
./target/release/zero-wpt-runner reftest-upstream
ZW_ORTHO_FLOAT_RL_ANCHOR=0 ./target/release/zero-wpt-runner reftest-upstream
```
