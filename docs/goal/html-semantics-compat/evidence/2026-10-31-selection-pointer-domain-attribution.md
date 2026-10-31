# selection-pointer forms 域翻绿归因勘定（2026-10-31，勘定轮——候选④销案，零源码改动）

## 入场与问句

s172 记「selection-pointer forms 域 Fail→Pass ~22 案——gEBI 幽灵面修复的意外
收益域/或 solo 负载差，域未深挖；域后续若回红再查」。本轮勘定确切归因。

标的：`html/semantics/forms/the-input-element/selection-pointer.html`（单文件
21 子测——对全部可见 input type 做 pointer 拖拽跨 `<input>`，断言 selection
anchorNode/focusNode 仍为 foo→bar 文本节点）。

## 红时代失败签名（在册证据重读）

s160 漂移轮日志（/tmp/zw-s160-drift-selection-pointer.log，21F）：

```text
assert_equals: anchorNode expected Text node "foo" but got Text node "foo"
```

期望与实际**打印逐字相同而断言失败** = 节点身份分裂（两个不同 Text 节点同内容
"foo"）——window 命名访问（`foo` 裸标识符）取到的 childNodes[0] 与 selection
锚定节点分属两个树视图副本。ghost/双视图类症状签名（s172 name-attribute
同族），非坐标/时序类失败。

## 三态 A/B 矩阵（本轮实测，test-guard + RAF 环境齐全）

| 树 | 跑法 | 结果 |
|---|---|---|
| main 3bbc7ca56（s176） | 全量 FILTER=forms（8 job 负载） | **21P/0F** |
| main − s172 gEBI 修复（8d29b5d6a 逆补丁，code-only） | solo FILTER=selection-pointer（load ~14） | **21P/0F** |
| d5ec12d94（s167 时代树——full1 记录本域红） | solo FILTER=selection-pointer（load ~14） | **21P/0F** |

逆补丁法：`git show 8d29b5d6a -- <part06.js> <coverage.rs> | git apply -R`
（s172 修复随后被并行 editing 流触碰，整文件回退不可行；code-only 逆补丁干净
适用。coverage.rs 逆补丁仅为测试面完整，两文件均验后还原）。三态均确认
cargo 重编译生效（Compiling 行在案）。

## 定谳

1. **修复收益证伪**：s172 gEBI 幽灵根修回退后本域仍全绿——该修复不是本域的
   门（其 name-attribute 靶面不受影响；两者 ghost 症状同类但门不同）。
2. **时代性代码缺陷证伪**：在记录本域红的 s167 时代树（d5ec12d94，full1 红
   的运行基点）上今日 solo 复跑全绿——红不可由该树代码+语料确定性复现。
3. **归因 = 环境漂移（机器级暂态）**：与 s160「环境漂移域 A/B 铁证与刀无关
   记账」、s171「solo 冷却回绿销账，漂移域为负载敏感暂态」两条在册结论同形
   互证。s159 时代「空闲态复跑仍 Fail」与今日同树 solo Pass 并存，指示当日
   机器态（并发 rally 负载/checkout 活动时间线，s159 已录）为触发条件；当日
   状态不可复得，绿态跨树/跨负载（s172→s176 五轮全量 + 本轮 solo ×2 + 历史
   树）稳定。

## 处置

- **候选④销案**（无施工面、零源码改动）。
- 「回红再查」方法升级（记账）：本域回红时按序跑两步对照再定刀——
  ①历史树对照（`d5ec12d94` 今日绿已钉，红复现于该树 = 环境再漂移；红不复现
  = 新代码回归）；②s172 修复回退对照（区分本域门是否已被新刀依赖）。
  两步走完仍无法归因才进入勘根施工，避免「修复收益」误判路径重演。
- 漂移域纪律沿用：并发 rally 期取证须 solo 窗口（run-rules 候选①对齐）。

## 局限注记

- 本轮 solo 跑处于 load ~14 环境（并行流活跃），非严格空闲窗——对「绿稳定」
  结论是加强（负载下仍绿），对「红时代机器态不可复得」结论是削弱（无法排除
  更高负载窗翻红）；已由三态矩阵与五轮全量绿对冲。
- 语料为 gitignored 本地数据，历史树复跑使用今日语料（selection-pointer 及
  其依赖 input-types.js/ahem.css 无变更记录，confound 风险低）。
- 中间态产物：/tmp/s176/sp_ab_red.log（s172 逆补丁）、
  /tmp/s176/sp_s167tree.log（s167 树）、/tmp/s176/sp_main_green.log（main）。

## 下一步候选（本流）

⑤多进程 renderer SetDomSnapshot entries 化（s174 挂账，低优先）；ua_display_li
与 nested-shadow 布局几何均渲染流不越界。
