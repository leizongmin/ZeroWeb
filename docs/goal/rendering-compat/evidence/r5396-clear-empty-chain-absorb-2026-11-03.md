# R5396：正 clearance × 自塌缩空块塌穿组差量落位 + 祖先高度传播（2026-11-03）

案面：`css/CSS2/floats-clear/margin-collapse-125.xht`（✗2.80%，R5395 挂账②同域备选案）。

> 承接注记：修复代码与测试成形于上轮 session 429 中断时（工作树遗留未验证改动，
> 归因取证由该轮完成，kill-switch 关断态取证记录在新测试文件头注），本轮完成
> 复验、corpus A/B、全门禁与落账。

## 归因（上轮 session：chromium oracle 探针 + 直构造探针）

- 案面结构：非 BFC 容器含 float（64×64）+ clear 空块（mt=mb=2em）+ 塌穿延续
  兄弟（mt=1em、border-top=2em），外层 td/comparator 表格形态。
- 机制：clearance 打在自塌缩（空）块上时，塌穿组 margin（自身 mt∪mb ∪ 后继 mt）
  按 CSS 2.1 §8.3.1 作为单一边缘被 clear 空隙吸收——元素 border 边落 clear_bottom，
  塌穿延续兄弟**不再重放整条链下值**，只落 clear_bottom + max(0, 链下值 − 链上值)。
  chromium oracle 实证：125 形态 mt=mb=32 → 链上=链下 → 差量 0 → 后继贴
  clear_bottom；r1316 探针族形态 mt40/mb80 → 差量 40（链下 80 − 链上 40）。
  旧实现后继整链重放 collapse(clear.mb, 后继.mt)，整段被双计。
- 第二层（祖先传播）：容器 auto 高随差量落位收缩（float 排除 §10.5.1 + 差量），
  但 td 按 taffy 期子 extent 计高——float 排除收缩不经 `clearance_extent_reduction`
  传播则 td 高 134 残留 2em 泄漏段露红底（案面像素差的第二层根因）。

## 修复（两文件三处）

1. **塌穿组差量落位**（float_positioning.rs `adjust_float_positions_with_context`）：
   - 新旗标 `clear_empty_chain_absorbed_above: Option<f32>`——clearance 生效且
     `is_empty_block(child)` 时捕获假设位折叠链上值；
   - ClearValue::None 塌穿延续兄弟消费点落
     `flow_bottom + max(0, collapsed_margin − above)`（差量为负钳 0：组不回溯）；
   - 非空块（组终止）与后继自持 clear（链重启，clearance 打断邻接）复位；
   - 尾链记账（last_flow_mb/flow_bottom）不动——R1317 containment 依赖原语义
     （012 谱系高度合成）；
   - kill-switch `CLEAR_EMPTY_CHAIN_ABSORB`（default-on，
     `ZW_CLEAR_EMPTY_CHAIN_ABSORB=0` 回退旧整链重放，OnceLock 单读）。
2. **祖先传播对称臂**（postprocess.rs `exclude_floats_from_non_bfc_auto_height`）：
   float 排除收缩时若 `clearance_active` 且 kill-switch 开，收缩量记入既有
   `clearance_extent_reduction`（R5109 字段，消费方
   `shift_siblings_after_clearance_containment` 统一传播）——与
   adjust_float_positions 收缩臂（float_positioning.rs:4032）同门同语义。

## 常驻测试

`engine/tests/r5396_clear_empty_chain_absorb_tests.rs` 4 针：

- `test_mc125_empty_clear_absorbs_chain_diff_zero`：125 主形态差量 0——clear 贴
  float 底边、塌穿延续兄弟贴 clear_bottom（旧语义落 96 的 load-bearing 位）、
  容器 auto 高 96（旧 128）。
- `test_mc125_td_propagates_container_shrink`：表格形态祖先传播——td 高 102
  （容器 96 + td 边框 6；不传播则 134 = taffy 期 extent 残留）。
- `test_positive_clearance_delta_offset_following_sibling`：正差量形态（r1316 族
  clear mt40/mb80、后继 mb140）——后继落 clear_bottom + 40（旧语义 +80），
  区分新旧语义的 load-bearing 断言 + DOM 顺序不回退（R1316 既有口径）。
- `test_absorb_flag_resets_on_self_clearing_sibling`：自持 clear 兄弟链重启——
  吸收旗标复位缺失不产生错误差量。

## 验证

- 新针 4/4 绿。
- 案面：125 ✓1/1 **0.00%（真通过-可信）**；kill-switch 关断
  （`ZW_CLEAR_EMPTY_CHAIN_ABSORB=0`）精确回 ✗2.80%（13440/480000 px，修复前
  基线值），关断态 table-anon 146/148 隔离仍 ✓0.00%（无关断态旁路效应，本轮
  亲自复核 + 上轮头注取证一致）。
- corpus 全量 A/B（双侧重跑二进制，基线 = 当前 HEAD 含 site-opt #274 合并）：
  基线 733F → 修复 731F 账面。FIXED 候选 {125 2.80% + table-anon-127 3.18% +
  table-anon-148 3.08%}、REGRESSED 候选 {table-anon-129 3.18%}——四案隔离复跑
  **3×3 全 ✓0.00% 洗清 = table-anon 并发抖动族**（R5385/R5389/R5395 在册同族）；
  真态 **731F，FIXED={margin-collapse-125}，REGRESSED=∅**。
- 本地集 **725/725 全绿**；margin-collapse-134（R5395 修）两侧保持 ✓。
- 门禁：fmt ✓ / guarded-clippy 全工作区 ✓ / make test 793P/1F——
  image_batch_e2e 败因为环境前置（zero-browser 二进制缺席），补建
  `-p zero-browser -p zero-renderer` 后隔离复跑 1P/0F（R5385/R5387 同款，
  非行为红）/ product-smoke **PASS**（welcome 18.14% 与 R5395 持平、struct-check
  全绿）/ bench-gate 定向 zero-layout-engine **GATE PASS 25 指标（NEW=0）**
  （首跑 loadavg 16 对端流负载自拒，静窗 2.9 复跑，R5392 同型流程）。

## 挂账 / 下一步

- R5395 挂账②（margin-collapse-125）本轮出账；R5395 挂账①（R5242 kill-switch
  关断态既有回归）维持；R5394 挂账（floats-clear-multicol 族 7 案特性级缺口、
  text-group-align 族口径差、float 全局 tree-order 深评、image_batch_e2e 对端
  账——本轮实证该案为环境前置形态，二进制补建后隔离绿，对端账口径待对端流
  确认）；R5392/R5391/R5390/R5389 挂账全维持；对端账（mutation_arm/
  default_actions/paste_and_go）维持。
- **下一步 R5397**：①731F 真态账本续扫择轻——floats-clear 族非 multicol 余红：
  negative-clearance-after-bottom-margin（2.55%，同域 clearance 案首选）、
  clear-applies-to-014（7.36% 族内最大单案）、floats-031（3.89%）、floats-145
  （2.06%）；margin-collapse-104（4.17%）同族候选（避开对端面）②backdrop-root
  深评（维持）③color-6（维持）。
