# 布局引擎 WPT 兼容性专项（Layout Compat）

**状态**: Active（启动门控——2026-11-27 批准建档立项，暂缓拉起新的并发执行流；待当前 `wptfyi-css` / `wptfyi-dom` / `wptfyi-html` 主力冲刺空出同机算力后再启动）\
**立项**: 2026-11-27，经用户批量拍板批准独立立项（触发依据：2026-11-04 既定规则「布局耦合池 > 10,000 再独立立项」已满足，GB-20261105 征询在案）。\
**父目标**: [ZeroWeb](zero-web.md) · 拆分上游: [CSS testharness 二期归档](archive/css-testharness-compat-v2.md) & [渲染兼容性](rendering-compat.md)\
**控制面**: [master.md](layout-compat/master.md)

---

## 拆分动机与基线事实

1. **触发条件成立（2026-11-05 实测，2026-11-27 拍板建档）**：
   - 在 [css-testharness-compat-v2](archive/css-testharness-compat-v2.md) M2 批次 4 实测中，布局耦合失败池达到 **22,566** 项子测试（B3 13 目录 19,286 + 一期 CSS Grid 探测挂账 3,280），扣除纯解析份额后仍 **> 15,000**，显著超过既定 10,000 独立立项门槛。
   - [rendering-compat](rendering-compat.md) 待决策清单中 `box-level bidi` 关联的 Grid 失败簇（3,280 案）亦计划归口至本目标统一承接。
2. **启动节流决策**：
   - 本目标先行完成建档、池归口与边界固化，**暂不拉起新的并发 Rally 流**，避免加剧同机 CPU、内存与静窗争抢；等当前 `wptfyi-css` / `wptfyi-dom` / `wptfyi-html` 主力冲刺释放算力后解除启动门控进入 DC-1。

---

## Mission

系统性收敛 `crates/layout-engine`（及必要联动层）在 WPT CSS 布局 testharness 与对应布局几何断言中的兼容性缺口，覆盖 CSS Grid、Tables、Sizing、Flexbox、Positioning、Writing Modes 与布局级查询接口（`getBoundingClientRect` / `offset*` / `client*` 等），并守住布局热路径性能预算与已有 reftest 基线零回退。

---

## Support Envelope

### 范围内
- `crates/layout-engine` 布局算法与盒模型几何计算（Grid、Tables、Sizing、Flexbox、Block/Inline 布局耦合断言）。
- 承接 [css-testharness-compat-v2](archive/css-testharness-compat-v2.md) B3 布局耦合池（19,286 项）及一期 Grid 探测挂账（3,280 项）。
- 与布局直接绑定的几何查询桥接与回归测试（`tests/wpt-runner`、`tests/integration`）。

### 明确排除（或需另行授权）
- **纯 CSS 解析与 CSSOM 序列化**：归 `wptfyi-css`；本目标不重复实现解析器或越面修改 `crates/css-parser` 活跃冲刺区。
- **既有挂账深结构 RFC**：`rendering-compat` 待决清单中已明确暂缓的深结构改造（box-level bidi 整体 IFC 重构、UA `<p>` 逻辑 margin + 竖排折叠、font-size-adjust 单 owner、table-internal-in-inline、multicol-2、3D 离屏隔离等）仍按各自用户门管理，不在本目标默认授权内。
- **跳过性能门禁**：所有 `layout-engine` 热路径改动必须守住 `LayoutBox` 体积钉测与基准门禁，不得以牺牲布局性能换通过率。

---

## 里程碑与 Done Criteria

- **DC-1 · 基线重建与失败聚类（启动后首轮）**：在最新 `main` 上受守卫复跑 B3 + Grid 布局耦合池，产出同 revision 基线清单，按根因聚类（Grid track sizing / Table 分配 / Intrinsic sizing / 几何读口 / 依赖深结构排除项）并制定切片优先级。
- **DC-2 · 按簇切片收敛**：按 ROI 逐簇落地修复，每片附常驻 WPT 导入或等价回归测试，且邻域 reftest 与已有布局测试集零回归（net ≥ 0）。
- **DC-3 · 性能与渲染守成**：`crates/layout-engine` 基准与 `make reftest` 无新回归，满足 [质量门禁](../engineering/quality-gates.md)。
- **DC-4 · 全量对账与归档**：布局耦合池净通过率达到 DC-1 落定的可收敛上限目标，剩余依赖外部深结构项逐条具名挂账，`master.md` 自洽并完成归档。

---

## 兄弟 Goal 边界与碰撞管理（run-rules §9）

| 兄弟 Goal | 共享面 | 边界与协作约定 |
|---|---|---|
| `rendering-compat` | `crates/layout-engine`、`make reftest` 基线 | 渲染流保留 reftest 守成与既有渲染修复主权；本目标启动后改 `layout-engine` 前须核对渲染流活跃写面，落地必须过 reftest A/B 零回归。 |
| `wptfyi-css` | `crates/style-system`、`crates/css-parser`、`tests/wpt-runner` | `wptfyi-css` 负责解析、计算样式与 CSSOM；本目标只消费已计算样式，跨入 `style-system` 或 `wpt-runner` 共享面按 [run-rules](../rally/run-rules.md) §9 记账协调。 |
| `zero-web`（ZRG） | `layout_bench` / `LayoutBox` 内存与性能预算 | 遵守 `layout_box_size_pin` 等既有性能钉测；触碰热路径须做局部 A/B 验证。 |

---

## Final Output Protocol

- 每轮仅更新 [layout-compat/master.md](layout-compat/master.md) 及 `docs/goal/layout-compat/evidence/`，不重写本入口文档（仅在目标范围或启动门控解除时更新）。
- 启动门控解除前不改动产品代码。
- 最终单行严格输出 `DONE`、`CONTINUE: <下一步>` 或 `BLOCK: <原因>`。

---

## Document Control

- **版本**: v1.0（2026-11-27 建档立项，启动门控态）
- **决策来源**: 2026-11-27 用户批量拍板记录（见 [zero-web/master.md](zero-web/master.md)）
