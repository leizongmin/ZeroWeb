# d64 · DC-3 全量对账终判：三目录 146,227 ≥ 146,184 达线

**日期** 2026-11-20 · **目标** [wptfyi-editing](../master.md) · **基线** d62 判决（141,068）+ d63 回收刀（投影 ≈146,321）

## 判决输入

- **Coverage run** [44668230656](https://github.com/leizongmin/ZeroWeb/actions/runs/44668230656)：`weekly.yml run_wptfyi_coverage=true` dispatch（2026-11-20T03:30:49Z），head `380cdbd1a`（含 d63 刀 169fdf61f + css 流 optical 413730881 / GBX 053cfecf0 组合态），2h9m 成功；聚合快照 [coverage-2026-11-20](../../../../compat/wptfyi/coverage-2026-11-20.md) 生成于 05:39:36Z 并由 CI 回写 main。
- **上游漂移（另报）**：wpt.fyi aligned stable 基准由 `71b58bb191`（11-18 runs）前进到 `4b26db8978`（11-19 runs，chrome 158.0.8127.36 / firefox 160.0.1 / safari 27.2 同 rev）。分母小幅漂移：editing 文件分母 888→889、html 10,968→10,970（子测试分母 +15）、css 子测试分母 198,952→199,018、sanitizer-api 50→51。同 revision 逐键对账已由 d62/d63 两轮完成，本判决分子口径为同 revision A/B + 新 rev 快照复核双层。

## 三目录判决（判据 ≥146,184；净增 ≥130,000）

| 目录 | 11-19 快照 | 11-20 快照 | Δ |
|---|---:|---:|---:|
| editing | 104,648 | 109,787 | +5,139 |
| selection | 34,041 | 34,041 | 0 |
| input-events | 2,379 | 2,399 | +20 |
| **三目录合计** | **141,068** | **146,227** | **+5,159** |

- **146,227 ≥ 146,184（余量 +43）**；对 2026-11-07 基线 16,184 **净增 +130,043 ≥ 130,000**。目标完成条件两项全过。
- 对 d63 A/B 投影（≈146,321）差 −94，归快照分片截断噪声带（d62 同带先例）：selection 套件回收落 editing topdir（jws 五文件），+5,139 对 A/B 直算 +5,231 差 −92；input-events +20 对投影 +27 差 −7（d61 +10、d63 ie corpus +17 均在窗口内）。
- selection 目录 0 变化与 d63 P2F=5 披露（insertparagraph-in-non-splittable-element 五形）在噪声带内相抵（≤5 键，分母漂移带），不改变判决；该 P2F 已披露挂 master ③ 查询视图代际分桶域。

## 邻域核验（无可归因回退）

| 目录 | 11-19 | 11-20 | Δ | 归因 |
|---|---:|---:|---:|---|
| dom | 57,080 | 57,651 | +571 | 并行流增益，非回退 |
| uievents | 195 | 195 | 0 | 恒等 |
| css | 99,976 | 102,223 | +2,247 | css 流 optical/GBX 刀预期增益 |
| html | 73,080 | 73,076 | −4 | 上游漂移带（分母同时 +15 子测试/±文件；d63 A/B dom/uievents/selce 哨兵零回归、byte 恒等，本流刀面无 html 触达；html 域归 html-semantics 流账） |
| third_party | 51,920 | 51,920 | 0 | 恒等 |

全量报告：ZeroWeb 通过子测试 1,870,514 → **1,878,510**（+7,996），通过率 82.22% → **82.57%**。邻域对本流刀面（css_wire fresh 通道 / callbacks 桥 / part01 重映射域）无可归因回退。

## 收口门禁（组合态 HEAD 97f6c8f4c 补验）

- 定向 enginelib 腿（test-guard `--compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test -p zero-engine --lib`）：**4,235 passed / 0 failed，RC=0**（test-guard 包裹）。
- `cargo clippy -p zero-engine --all-features --all-targets`：**RC=0**。
- `cargo fmt --all -- --check`：**clean**——首轮 RC=1 落 `css_wire.rs:460` 单测断言排版，归因 css 流 FONT-1 刀 `74f08526e` 引入（非本流刀面），收口轮按机械 rustfmt 修正（4 行重排零语义变化），复检 RC=0。

## 归档动作（模式 A）

`git mv docs/goal/wptfyi-editing.md docs/goal/archive/wptfyi-editing.md` + `git mv docs/goal/wptfyi-editing/ docs/goal/archive/wptfyi-editing/`（整树 entry+master+evidence）；相对链接按新深度整体重写（master `../../`→`../../../`、evidence `../../../`→`../../../../`、root 级 tests/ 五层）并全量核验解析存在；外部引用同步：[ROADMAP](../../../../ROADMAP.md) 赛道行、[五目标计划](../../../../compat/wptfyi-90-plan.md) 表行 2、[archive 索引](../../../../goal/archive/README.md) 登记（含 editing-compat 行互指改 `./`）、[learning 路径](../../../../learnings/bugs/2026-11/2026-11-13-forcevalue-normal-is-valid-add-face-param.md) 修正；`scripts/rally-goal-04-wptfyi-editing.sh` 启动器随归档删除（01/02/33 先例）。

## 判定

DC-1 / DC-2 / DC-3 全部有据完成；共同计划预算（净增 ≥130,000）兑现；最新周跑有效产物落库；邻域无可归因回退。**wptfyi-editing goal 判 DONE，按模式 A 归档。**
