# 布局引擎 WPT 兼容性控制面（Layout Compat）

**状态**: Active（启动门控——2026-11-27 建档立项，暂缓拉起并发执行流）\
**卡点**: 启动门控生效中——等待当前 `wptfyi-css` / `wptfyi-dom` / `wptfyi-html` 主力冲刺释放同机算力后再解除门控。\
**下一步**: 门控解除后进入 DC-1，在最新 `main` 上对布局耦合池（B3 19,286 + Grid 3,280 = 22,566 项）执行受守卫基线复跑与失败聚类，并按实测落定 DC-4 数值目标。

[执行契约](../layout-compat.md) · [上游来源（css-testharness-compat-v2）](../archive/css-testharness-compat-v2.md) · [总控面](../zero-web/master.md)

---

## 里程碑进度

| 里程碑 | 状态 | 说明与证据 |
|---|---|---|
| 建档立项 | ✅ 已完成（2026-11-27） | 经用户拍板建档立项并设置算力启动门控；触发依据为 2026-11-05 实测布局耦合池 22,566 > 10,000（见 [GB-20261105](../zero-web/master.md)） |
| DC-1 基线重建与聚类 | 待执行（门控等待） | 启动后首轮在最新 `main` 重跑 B3 13 目录 + 一期 Grid 挂账池，区分纯布局缺口与深结构依赖 |
| DC-2 按簇收敛 | 待执行 | 等 DC-1 聚类排序后逐簇切片 |
| DC-3 性能与渲染守成 | 待执行 | `layout_bench`、`LayoutBox` 体积钉测及 `make reftest` 零回退 |
| DC-4 全量对账与归档 | 待执行 | 达线后归档至 `docs/goal/archive/` |

---

## 归口池摘要（立项快照，待 DC-1 复测更新）

- **B3 布局耦合池**：13 个 CSS 目录，2026-11-05 测算 19,286 项（其中首测通过 2,665 / 11,088 = 24.0%，`css-tables` 40.3%）。
- **一期 CSS Grid 探测挂账**：3,280 项（含 `rendering-compat` 归口的 Grid 失败簇）。
- **合计规模**：22,566 项（扣解析份额后 > 15,000）。

---

## 碰撞记录

- 暂无活跃写面碰撞（当前处于启动门控，尚未修改任何代码）。
- 启动后修改 `crates/layout-engine`、`crates/style-system` 或 `tests/wpt-runner` 前，须先核对 `rendering-compat` 与 `wptfyi-css` 的在途写面并按 `docs/rally/run-rules.md` §9 记录协调。
