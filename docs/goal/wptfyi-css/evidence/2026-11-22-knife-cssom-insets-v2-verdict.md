# 刀 CSSOM-INSETS-V2 收账证据（2026-11-22）

## 刀面（r4 终版，patch md5 `a0a59f31bc46fd739ef5755053400168`，633 行）

四文件：

1. `crates/layout-engine/src/engine/abspos.rs`（census 根因①②④）
   - nested-CB pass（`resolve_abspos_against_nested_cb_inner_m`）签名增 `fixed_cb: Option<(f32, f32, f32, f32, bool)>` 穿线：fixed-CB（transform/filter/containment 祖先，`is_fixed_cb_ancestor`）成为 fixed 后代 CB；fixed 直挂子 gate `None`（视口链不触，防与 11.5/11.6 双应用）；递归点 `child_fixed_cb` 下传本盒 padding 盒 + CB direction。
   - root-CB pass 与 nested pass 的 stretch 臂 + 定位臂（left/top/right/bottom 四处 × 两 pass）+ stretch_fixed containment 臂统一接线 `resolve_abspos_calc_inset`/`resolve_abspos_inset_px`（%/calc inset 布局期按 CB 轴基解析）。
   - `#[allow(clippy::too_many_arguments)]`（fixed_cb 加参后 8/7 越 clippy 阈；canvas crate 同型 walk fn 同款先例）。
2. `crates/layout-engine/src/engine/abspos_sizing.rs`：单测 caller 适配新参数（`None` 占位）。
3. `crates/engine/src/js_dom_bridge/computed_style_insets.rs`（census 根因③）
   - `over_constrained_computed_fallback`：CSS2 §10.3.7/§10.6.4 过约束形（对侧 inset 非 Auto + 四 margin 全非 Auto + 本侧被忽略侧：bottom 恒忽略、水平 ltr 忽略 right/rtl 忽略 left）回落 computed 绝对化（% 按 CB 对应轴、Calc 同 R5465 parent_length 基）；等式成立时双臂同值零漂移，P2F-safe（Px 被上游门排除、Auto 走 rect 读回既有臂）。
   - `containing_block_padding_rect` 返回附带 CB direction（rtl 判定用）。
   - v1 doc `+ rect 表` 假 list 标记改写（clippy-1.99 `doc_lazy_continuation`）。
4. `crates/engine/src/js_dom_bridge/computed_style_cache.rs`（clippy-1.99 我流债，行为中性）
   - gCS echo 预判 `if_same_then_else`（width/height 臂与 inset 臂同体 `Some(s.clone())`）合并单臂；
   - GEN_RECT_CACHE `bind_instead_of_map`（`.and_then(|..| Some(map.clone()))` → `.map(..| map.clone())`）；
   - 该文件 3 处 fmt 债清偿（v1/used-size 流遗留）。

## 电池谱系（R 系 4 轮，判读全同值）

| 轮 | patch md5 | face | 结果 |
|---|---|---|---|
| r1 | `6fe6d6f3…` | 3 文件，③ 臂带尺寸 definite 门 | ③ 族 30 案未翻（over-constrained 形 inset 单独过约束、尺寸 auto），去门修正存档 `results-knife-cssom-insets-v2-r1-pre3fix/` |
| r2 | `6fe6d6f3…` | 3 文件（fmt 前字节） | GATE PASS 198/P2F=0（`results-*-r2-fmtdirty/`） |
| r3 | `e2420f59…` | 3 文件 rustfmt 终版 | GATE PASS 同值（`results-*-r3/`）——rustfmt 使 patch md5 变更＝判读域失效，重跑重建字节绑定 |
| r4 | `a0a59f31…` | 4 文件（+cache clippy/fmt 债修复） | **GATE PASS 终版**（`results-knife-cssom-insets-v2/`） |

配方：`knife-cssom-insets-v2-battery.sh`（chmod 400；patch 双向校验 + 树面白名单 4 文件 + 前置静默闸让位跨 clone 在飞批 + b0=HEAD 剥面/b1 重放 + M2 双二进制 md5 异硬门 + test-guard 包裹 build per-proc 6/total 14/1800s、leg per-proc 4/total 8/3000s CASE_WALL_S=60）。

日志：`knife-cssom-insets-v2-battery-093255.log`（r1）、`…-r2-101334.log`、`…-r3-102830.log`、`…-r4-104615.log`。

## r4 判读（10:50:15 PREFLIGHT-QUIET → 10:59:42 GATE-DONE）

- M2 双二进制 md5 异：b0 `5bc5b423374ec2576a6ff4e43ba26076` ≠ b1 `7cce9d472cdff063f3c6e11bdbf0f3f1`（此前 r 系 b0/b1 亦异）。
- FACE-MARKER 符号打印 b0=0/b1=0（release 内联消符号，预设打印不设门；行为证据由 judge 承载）。
- 主靶 cssom：keys 3459 恒，P 2065→2263，**F2P=198 P2F=0 new=0 gone=0**。
  - getComputedStyle-insets-absolute：F2P=90（132→42F）
  - getComputedStyle-insets-fixed：F2P=108（204→96F）
  - 余四 insets 页（relative/static/sticky/sticky-container）F2P=0 零漂移
- 四哨 SET-IDENTICAL：
  - css-position：keys 0（结构性空）恒
  - cssom-view：P 565=565
  - css-sizing：P 3291=3291
  - css-values：P 990=990（b0 991 中 1 键 random-computed flake 落 judge 排除域，excluded=1，OM-17 定谳 doc_seed 采样漂移）
- GRAND：**F2P=198 P2F=0 new=0 gone=0 excluded=1 → INSETS-V2-GATE-PASS**

## 残池 138F 三根因（挂后续刀，b1 JSON 可复核）

1. **RTL 过约束锚定引擎布局臂 ~42F**（absolute 18 + fixed 24）：`'right' expected "400px" but got "0px"`——rtl CB 下引擎把锚定边算错（布局层锚边选择，非 gCS 读侧）。
2. **fixed nested 臂坐标换算 ~24F**（pct-abs 12 + calc-abs 12）：`'left' expected "150px" but got "290px"`（pct）/`expected 148 got 288`（calc）——+140 常量偏移，fixed 元素经 transformed 祖先链时 nested pass 坐标基换算缺一环。
3. **静态位族 ~72F**（abs auto-used 24：`'left' expected "15px" got "40px"`；fixed static 48：`expected "127px" got "320px"`）＝census ⑤ WM 静态位缺口 + fixed static position。

## 提交前门禁序列记录

- **fmt**：本刀面 4 文件 rustfmt 终版零 diff（canonical config：`rustfmt.toml` max_width=120/style_edition 2024，stable toolchain）。workspace 既有债 17 文件 45 hunk（HEAD 提交态漂移，非本刀引入；清单见 `results-gate-maketest-insets-v2/fmt-check-head-debt.diff`）。热文件属在飞流域（calc_size=flex-wrap、apply=style-system 等）按双 clone 协作协议不跨流代修，挂账 master.md。
- **clippy**：`zero-layout-engine` 全 crate `-D warnings` 净；本刀面 4 文件零 lint（r1 发现的 `too_many_arguments` 已按 canvas 先例 allow；v1 流债 `doc_lazy_continuation`/`if_same_then_else`/`bind_instead_of_map` 3 处已修）。`zero-engine` crate 级被 12 处外域 lint 阻断（animation.rs ×5、computed_style.rs ×4、computed_style_cache 域外 0、border_shape.rs ×1、css_style_declaration.rs ×1、js_dom_bridge.rs ×1——clippy-1.99 新 lint 扫出的他流直推历史债），挂账 master.md。
- **make test 配方腿**：fmt 门被 workspace 既有债阻断（make test 入口第一步），按配方逐腿手动执行（`gate-maketest-insets-v2.sh`，结果见 `results-gate-maketest-insets-v2/`）——预期 12 个 pre-existing layout-engine lib 失败（fontcycle 刀 8b70cb9fe ch 兜底改口径未更新旧 0.5em 期望测试：r5162_table_fixed_dist ×3、inline r3634/r3624/r3623/r3636、inline_finalization r3625 等）与 HEAD 基线差=0。

## 教训

- rustfmt 通过后 patch md5 变更即判读域失效：r2→r3（fmt 前后）与 r3→r4（扩面）两度重跑电池重建字节绑定。语义中性（rustfmt/allow/lint 修复）不减责——判读绑字节不绑语义。
- battery 脚本 pid 判活：`setsid $!` 是包装壳即退，真 pid 在 ps 父链（bash script 子进程）；误绑壳会假报 SILENT-DEATH。
