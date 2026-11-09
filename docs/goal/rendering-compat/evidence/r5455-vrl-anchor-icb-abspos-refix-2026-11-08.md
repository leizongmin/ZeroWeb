# R5455 证据：vertical-rl 根锚定 × 视口-CB abspos 双计回退（background-position-vrl-018/020/022）

日期：2026-11-08。驱动案：`css/css-writing-modes/background-position-vrl-{018,020,022}.xht`（三案仅 `background-position: left top/center/bottom` 之差，共享同一 abs 绿条覆盖层）。

## 1. 归因链

### 1.1 chromium oracle 定谳正确语义（JS 几何探针）

018 test 页 chromium headless 注入探针：`html=[427,0,373,513] body=[435,8,357,497] htmlCS=[800,513] bpos=0% 0%`。
根盒 shrink-to-fit 373（= body margin-box 8+357+8）右贴 ICB；红 tile 列（repeat-y, 100 宽）落根盒左缘 427..527，恰被 `right:273px + width:100px + height:100%` 的 abs 绿条（ICB-CB：无 positioned 祖先 → x = 800−273−100 = 427）完全遮盖 → 判据 "no red"。

### 1.2 ZW 病理几何（LAYOUT_DUMP，修复前）

```
html   x=427.0 w=373.0   ← 根盒 shrink + 右锚已对（anchor_vrl_root_box）
body   x=435.0 w=357.0   ← 静态链已对
div    x=854.0 w=100.0   ← abs 绿条：854 = 正确 427 + 根锚 delta 427（双计），出视口
```

taffy 0.7 以**静态父**作 CB 解析 abs 子坐标（% inset 由 pass 11.5 预解析；Px inset 依赖水平模式「静态父链原点 = ICB 原点」巧合落位）。`anchor_vrl_root_box` 根盒右移 delta（= icb_width − root.width = 800−373 = 427）后，paint walk 沿静态链累加把 delta 计入 abs 子终位。缺失的绿条 = 100/800 × 全高 = **12.50%** 全部差值。

隔离探针（/tmp/zw_abspos_probe 六形，修复前）：水平 abs ✓ / 垂直静态 ✓ / 垂直 abs（right、left、显式高、含文本）全白 = 域确认为 vertical-rl × abspos。

### 1.3 修复体

`vertical_block_flow.rs`：
- `anchor_vrl_root_box` 返回实际施加的右移量（未过锚定门 = 0.0）；
- 新 pass `refix_icb_abspos_x_after_vrl_anchor`（float 修正后调用，链原点终态）：对静态链无 positioned 祖先的 abs/fixed 盒 `b.x -= anchor_dx`。**不触 positioned-CB 域**（其 CB 随 restack 同帧移动，回退会过度修正；FIXME 有驱动案再探）。不解析 inset、不建快照，最小语义面。`ZW_VIV_SIZING` 同族 kill-switch（anchor 返回 0.0 时 pass 不进）。

修复后 dump：`div x=427.0` ✓（html/body/img 逐位不变）。三案 solo `✓0.00%`。

## 2. 常驻测试

- 上游 corpus 三案即常驻 reftest（0.00% 真绿；test 臂关键量与 chromium JS 探针**逐位同值**，oracle 定谳非自源自洽绿）。
- `vertical_block_flow.rs` tests 新钉 `test_refix_icb_abspos_x_after_vrl_anchor`：ICB-CB abs 回退 −427 / fixed 同域 / positioned-CB 域不回退 / abs 静态后代自身坐标不变，四断言。

## 3. corpus A/B 与伪绿解除披露

- 本轮双窗（同二进制）：w1 698F / w2 695F（基线 R5454 696F）。w1↔w2 互翻五案（insert-block-in-blocks-n-inlines-middle-003、cascade-highlight-005、canvas-intrinsic-dynamic、table-anonymous-objects-146/154）= 负载噪声指纹，不进代码归因。
- **稳定新红仅 float-lft/rgt-orthog-htb-in-vrl-002/003 两案（✓2.31% → ✗6.91%）＝伪绿解除**。基线二进制复核（git stash 往返同 dump）：
  - test 臂（修复前后逐位同）：orthog-htb float 盒 `h=NaN` 不绘制（真债＝既有挂账 #33 orthogonal float 域，R4428 float 排除致 viv 整页关闭，float 高 NaN 源待归因）；
  - ref 臂（修复前）：abs 条 x=1472 出视口（同一双计缺陷）不绘制；修复后 x=744 落位正确（与 span 同位）。
  - 基线「✓@2.31%」= 双臂各自残缺互偿（同 estimate-fonts-vs-window-truth 伪绿家族）；修复使 ref 臂正确后暴露 test 臂真债 → 诚实红。002/003 现成 #33 驱动案（真红 6.9% 可复现）。

## 4. 门禁矩阵

- fmt clean；clippy `-p zero-layout-engine -p zero-wpt-runner --all-targets -D warnings` 绿。
- layout-engine lib 全腿（compile-first：裸腿总内存 17.1G > 16G 守卫击杀 → 按 R5454 纪律分腿）**2444P / 0F / 2I**。
- integration **793P + 1 环境伪红**（debug 侧 zero-browser 缺二进制 panic → `cargo build -p zero-browser -p zero-renderer` 补建后该案 PASS）。
- `make reftest` **725/725**（Layout 521 + Text 204）。
- corpus：见 §3。真态口径 = **REGRESSED = 伪绿解除 2 案（#33 暴露，非 refix 几何错误）**；FIXED = 018/020/022 三案。
- js_dom_bridge 6 红基线二进制同型（stash 往返 `cargo test -p zero-engine js_dom_bridge` 同 6 失败）= 对端九案红腿家族维持，非因果。
- product-smoke 全 PASS（welcome oracle + struct 六站）。
- bench-gate 静窗（load 1min 0.51 + 全机进程零判定）**GATE PASS 25 指标，全部预算内**（NEW=88 = mb/zero-webview 域首次记录，无基线对照项）。

## 5. 取证命令存根

```bash
# 布局 dump（修复前 854 / 修复后 427）
LAYOUT_DUMP=1 ./target/release/zero-wpt-runner product-smoke \
  tests/wpt-runner/wpt-data/css/css-writing-modes/background-position-vrl-018.xht --out /tmp/x.png
# 三案 solo
./target/release/zero-wpt-runner reftest-upstream background-position-vrl-018
```
