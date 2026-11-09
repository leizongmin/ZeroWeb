# M3 热点归因——thrash 10079x 与 mut.replace.pool 358.8x 调用链

日期：2026-11-04　|　基线：HEAD `9bab87ae3`（缺陷1/缺陷2 修复后）　|　机器：i5-13500H/16 核 linux-x64（与官方采集同机）

## 结论

两个数量级异常的调用链均已定位到代码路径与实测数字：

1. **`geom.write_read.thrash`（zw ~12.4ms/op vs ch ~1.4µs）＝pending 读的全量有效视图重建**。
   每次样式写使 `RECT_LAYOUT_CACHE` 键（含队列长度 mlen）失效，随后每次 gBCR 读触发
   worker 侧整文档重建（parse cached_html + cascade + 全量 `LayoutEngine::compute` +
   HitTest 填表 + 快照克隆），~13ms/次，**成本与队列长度无关**（300 op 队列 2→600 条全程平坦）。
   同键第二次读起 O(1) 命中（0-30µs）。修复方向是增量布局/脏范围失效——架构级，
   按目标契约（性能修复仍走已有对应规范回归）**推迟到专用性能工作流**；
   缺陷1 的语义面（gBCR 反映 pending 写）已修复并有常驻回归。

2. **`mut.replace.pool`（zw ~3.3-6.1ms/op vs ch ~12.9µs）＝shim 逐节点结构开销**。
   裸池 3.4ms/op ≈ 属性池 3.6ms/op（**属性 overlay 仅 +6%，不是驱动**）；成本 ∝ 子节点数
   （5 子 0.5ms / 50 子 3.4ms ≈ 34-100µs/节点操作）；离屏 createElement+class+dataset
   基线即 ~100µs/节点，挂到文档再 +~600µs（mutation 记录 + 队列追加 + 结构簿记）。
   与几何重建无关，属 shim 每节点 JS 对象/记录成本，优化同样推迟到性能工作流；
   缺陷2 的语义面（attr 读经历史收件箱回落）已修复并有常驻回归。

附带发现（未解决，已如实记录）：probe8 的 A2 形在同一二进制下出现过 2 次 ~12-30µs/op 的
便宜读（见「未解决项」）。

## 方法

探针序列 probe8 → 8b（因子 bisect）→ 8c（独立 tab cliff 扫描）→ 8d（暖态武装因果隔离）→
8e（replace.pool 机制构成）→ 8f（evaluate 墙钟 + 读值终判别）。原则：

- 每臂独立新 tab（`ctx.newPage()` + goto + 300ms settle），臂间互不污染；
- 逐 op `performance.now()` 计时 + Node 侧 evaluate 墙钟双口径；
- Chrome 配对同探针（非隔离页计时量化在 100µs，全亚分辨率/无 cliff）；
- 单次异常按 ≥3 次规则处理（guideline #35），不重复计数。

## 热点 1：thrash 机制链（全部实测，ZeroWeb，style-geometry 600 cell）

**代码路径**（`crates/engine/src/rect_bridge.rs` `make_effective_dom_rect_handler`，缺陷1 修复引入）：
pending 读（`mlen > applied cursor`）→ 查 `RECT_LAYOUT_CACHE`（键 `(html, drain_gen, mlen, vw, vh)`）→
每次写追加 mutation 使 `mlen` 变化 → 键失效 → 全量重建：
`with_cached_document_styles_lazy`（parse + cascade；代际缓存键含 mlen → 同样 miss）+
`LayoutEngine::new(vw,vh).compute(doc, styles)`（整文档布局）+
`HitTestCache::from_document` + `fill_layout_rect_snapshot` + HashMap 克隆。

| 测量 | 结果 |
|---|---|
| 基准 v3（12 样本 × 50 op，正/反序） | zw 12409/12789µs/op，样本平坦（队列 1250→1850 无增长效应） |
| probe8c e3：同元素 toggle+读 300 op，逐 op | 13.2→14.0ms/op **全程平坦**（队列 2→600 条，无斜率 → 固定成本非线性 replay） |
| probe8c e4：异元素写+读 300 op | 13.4-13.9ms/op 平坦 |
| probe8b 因子 bisect（t_mirror / 去setup / 去gap / 队列标定 K=100,300 / 大队列单脏元素 / 前序子场景残留） | 13.6-25.5ms/op **全部昂贵**——setup 清理、timer 间隙、脏元素数、队列长度、前序子场景均非驱动因子 |
| probe8f：evaluate 墙钟 | 60 op 856-874ms ≈ 逐 op 计时合计 → 成本确在读内，非延迟转移 |
| 同键批读（写 1 条后 50 次纯读） | a3 batch 28µs/op；e5 间隙后 0µs；基准 geom.read.same 20.7µs/op（其一次性重建被不计时的 warmup 吸收） |
| 冷首读（新状态首次 pending 读） | 14.6-21.2ms（e5 pre / 8f a3 cold 16.7ms，读值 33px 正确） |
| drained 快路径（队列空） | 23.3/59.3µs/op（probe8 两轮 a1） |
| 量级自洽 | 13ms ≈ 2×(parse 0.38 + style 1.12 + layout 4.35 = 5.85ms，engine-direct 管线分相)——worker 侧 replay 含快照再物化与簿记 |
| Chrome 配对 | e1/e3/e4 全部 200-300µs（非隔离页 100µs 量化，真实值 ≈ 基准 1.4µs）——**无 cliff，机制纯 ZeroWeb 侧** |

**读值语义验证**：probe8f a2 空写回落 CSS 默认 24px、a3 冷读返 33px——pending 路径语义正确
（缺陷1 修复在工作）。

## 热点 2：mut.replace.pool 机制链（全部实测，ZeroWeb，mutation.html）

`replaceChildren(50)` 每 op = 50 移除 + 50 插入（100 节点操作），无 gBCR 读。

| 测量 | 结果 |
|---|---|
| 裸池（无 className/dataset）240 op | **3.4ms/op** |
| 属性池（基准同型）240 op | 3.6ms/op → 属性 overlay ≈ +0.2ms（**+6%，非驱动**） |
| 池规模标定 5 子 | 0.5ms/op → 成本 ∝ 子节点数（34-100µs/节点操作） |
| 离屏 createElement+class+dataset | ~100µs/节点（Chrome 亚 µs）——shim 每节点基线 |
| 挂到文档 appendChild | +~600µs/新节点（mutation 记录 + 队列追加 + 结构簿记） |
| 队列成长剖面 240 op（首 20 vs 末 20） | 8.3 → 10.4ms/op——温和亚线性（队列 150→24000 条） |
| 冷首 op（3 个 fresh 文档） | 4.5-5.0ms——同 regime |
| Chrome 配对 | 全部 0（亚分辨率）；基准 12.9µs/op |

## 未解决项

**probe8 A2 便宜态（~12.7-29.7µs/op）不可复现**。probe8 脚本两轮均为便宜读；同一二进制下
15+ 个其他臂（含两个与 probe8 序逐步一致的复刻：probe8d Z、probe8f seq1）全部 13-21ms/op。
已排除：setup 清理、timer 间隙、队列长度、脏元素数、前序子场景、页内 evaluate 序数、
机器负载（load 5.7 下仍昂贵）。快照便宜读路径（同键命中 / drained）真实存在且可测，
但 probe8 观测的「写后读仍便宜」无法归因到已识别路径；领先假设是某种
no-op/失效句柄快速回落。**不作归因依据**，作为性能工作流的首要疑点：
若找到可稳定武装的快路径，即 thrash 的修复方向。

计时分辨率注记：本轮页内循环可分辨 12-30µs 差值（v1 meta 曾测 ~100µs 粒度——分辨率依
上下文不一）；本轮所有结论基于 600-874ms 级样本总量，分辨率误差 <0.02%，不影响比值。

## 环境与产物

- ZeroWeb：`target/release/zero-browser --headless --remote-debugging-port <p>`（HEAD 9bab87ae3 构建）；
  Chrome：google-chrome headless（版本随系统）。
- 探针脚本：`probes/probe8*.mjs`（6 个，自包含可复跑，SL-008 路径推导）。
- 原始输出：`raw/probe8{,b,c,c-lowload,d,e,f}-{zw,ch}.json`（Chrome 非隔离页计时量化说明见上）。
- 基准对照数据：v3 报告（[缺陷2 evidence](../2026-11-04-m3-defect2-dataset-timer-loss/reports/report.json)，
  git_sha 90572cc78 采集）。

## 对里程碑的含义

M3「首差异剖析→最小修复→配对性能与行为回归」收口：两个语义缺陷（DC-1/DC-2）已修复并配对
常驻回归；两个性能热点已归因到代码路径与数字，最小修复均为架构级工作，按目标契约转入
专用性能工作流（非本 goal 范围），残余在册。后续 M4 接线有价值指标（thrash 每 op 重建成本、
replace 每节点成本可作为门禁敏感指标候选）并做 DC-5 审计。
