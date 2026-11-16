# Steps 位置语义共享臂修复验证刀（二十一轮收口补测，b1v9）

日期：2026-11-16。性质：**已提交未实测码的验证轮**（零码改）。

## 1. 背景：吸收事故与补测必要性

58b3721ad（二十一轮 twin 提交）含共享 `apply_timing_function` Steps 臂修复
（`TimingFunctionValue::Steps(n, pos)` 吃 StepPosition，css-easing-1
§step-easing-functions：start/jump-start 越点 ceil 上包络、end/jump-end floor、
jump-both 两端钳+ceil、jump-none 内缩 round；此前恒 floor 弃 position，
steps(1,start)@t=0.5 误落 0 → at(1) 编码族 eased 误 0 → got=from）。
该修复由本流 14:06:39 写入工作树后被 twin 14:07+ 的提交无意识吸收，而 b1v8
权威批二进制构建于 14:01:50（**不含**该修复）——即修复已入 main 但无 corpus
实测。本轮 b1v9 补测收口。

## 2. 链与 provenance

`zw-r21-chain-v5.sh`（systemd 单元发射，census=1 unit + 单 CHAIN-START）：

1. engine lib 定向腿（test-guard `--compile-first --per-proc-mem 4
   --total-mem 8`）：**4,073P/0F**（含 Steps 修复与既有钉编译绿）。
2. release runner 重建：md5 `447334b967714f5a7546062e16284fa8`。
3. 七套件 corpus（css-masking motion css-shapes filter-effects css-gaps
   css-multicol css-typed-om）：BATCH-DONE rc=0 ×7，CHAIN-DONE 14:17:33。

provenance：CHAIN-START 14:13:56 记录 git log=58b3721ad、`git status
--porcelain` 仅 `M crates/script-sandbox/src/es_module.rs`（peer s419 WIP，
两侧同载零贡献先例）——**b1v9 = 纯 58b3721ad 码态**（工作树 animation.rs
path 门撤除 WIP mtime 14:18:00 晚于 CHAIN-DONE，未进二进制）。

## 3. 对账（B0 权威基线双口径）

`/tmp/zw-trans-compare.py` 逐子测对账，页集合 assert 相等：

| 口径 | gained | lost | regressed | pages_gain |
|---|---|---|---|---|
| vs b1v4（二十轮权威 B0，码态基线） | **741** | 0 | 2 | 29 |
| vs b1v8（隔离 Steps 修复增量） | **222** | 0 | 2 | 29 |

+222 分布 29 页（masking shape +15/control-points +13/xywh +6/discrete +4、
shapes path +21/shapeout +10/shape-shape +12、filter-effects 八页 +2~+13、
motion 十一页 +1~+10）——at(1) 编码族从 got=from echo 失败转 flip/eased=1
通过，覆盖全部七套件（非仅 path 页），与二十轮 v2 at(1) REG 定谳的残面②
（jsanim/WAAPI flip Steps(Start) 共享位）完全对型。

## 4. DOWNS 定谳：2 REG 为改进暴露型伪红

两 REG 同形：`Web Animations: property <clip-path>/<offset-path> from
[inherit] to [shape(from 15% 15px, move to 20% -10px)] at (1)`，各 1 例，
恰在 2 页。逐层实锤：

1. **上游 WPT 数据笔误**：`{at: 1, expect: 'shape(from 15% 15px, move to
   20% -10px'}` **缺右括号**——`clip-path-interpolation-shape.html:64` 与
   `offset-path-interpolation-008.html:54`，全仓 grep 恰此 2 处。
2. **expected 污染机制**（interpolation-testcommon.js:385-387）：expected
   经独立 expected target 的 `setProperty(expectedStr)` 往返产出；畸形串
   被拒收 → `getComputedStyle` 回落该 fresh target 的样式面——
   `createTargetContainer` 克隆 `#target-template`，`.target` 类规则
   （页面 stylesheet line 29）命中 → expectedValue =
   `shape ( from 5px 5px , line to 10px 10 % )`。
3. **b1v8 伪通过双巧合**：旧 floor 臂 eased(0.5)=0 → got=from 侧回落 =
   同一 `.target` 类规则值 ≡ 污染 expected → **Pass（假）**。
4. **b1v9 正确值暴露**：修复后 eased(0.5)=ceil=1 → got=to 端点（与测试名
   声明期望 `should be [shape(from 15% 15px, move to 20% -10px)]` 逐字
   吻合，仅规范化空格差）≠ 污染 expected → **Fail（真值暴露假期望）**。

同页 CSS Transitions 腿同对全部 at() 两代恒 Fail（transcript 10977-10999
逐字节恒等），非本轮增量。引擎值语义判定：at(1) 输出 to 端点是唯一符合
测试声明的值；合规引擎（Chrome 同型：actual=to 值 ≠ 污染 expected）同样
Fail 该子测——**上游数据 bug 恒红，非引擎回归**。不回滚。

## 5. 新挂账

1. **inline style CSS-wide 关键字解析疑点**（本轮实锤的新真面）：WAAPI
   from=[inherit] 端点实际 target 的 from 侧输出 = `.target` 类规则值而非
   `.parent` 规则继承值（b1v8/b1v9 at(-0.3)/at(0) transcript 同证）——
   inline `clip-path: inherit` 似未被继承链解析（回落类规则）。修通后
   inherit→shape 对 5 子测中 4 个（at -0.3/0/0.5/1.5）可真插值通过
   （声明期望即 `.parent` 值→to 的 calc 化插值，clip_interp 同型对可达；
   at(1) 因数据笔误恒红）。归队列：inline-style 全局关键字解析刀。
2. **上游 2 页 at(1) 缺右括号**：引擎侧不可修，恒红记录在案（本文件 §4）。
3. **工作树 animation.rs path 门撤除 WIP**（mtime 14:18:00，twin 流在飞
   认领面）：v6 定谳 path 门臂净损 8 的撤除刀 + Steps 位置单测钉 +
   离散翻转期望修正，未提交未验证——本轮未触碰，待其自证 A/B 后入账。

## 6. 账目

共享 Steps 臂修复（58b3721ad 内）实测收口：**+222/0/2（2 伪红定谳不回滚）**，
residual面②（jsanim flip Steps(Start) 共享位）关闭。二十一轮 +519 权威账
（b1v8）与本轮 +222 叠加后对 b1v4 基线累计 +741/0/2。伪红分类按
「转抄前独立核机制」条款以 transcript + 测试数据 + harness 源三重实锤定谳。
