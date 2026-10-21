# M2-S3d：DC-3（COMP-03）验收——命中当前几何 + 暂停/取消/恢复一致性（真实窗口）

日期：2026-10-22 ｜ 里程碑：M2-S3d ｜ 判据：DC-3

## 判定

**成立**。DC-3 四场景在真实窗口（生产合成帧采集 + env 门控真实输入注入）双向成立：flag ON 五回执全 pass——(a) 位移中/冻结几何上的注入点击经 S3c 逆映射命中目标元素（hit-a/hit-b 绿）；(b) 暂停像素冻结于 46px（理论 45px=30px/s×1.5s），恢复续走 Δ=0；(c) animationcancel 恰一次、elapsed=3.589s（含忙窗顺延，±0.5s 容差内）、取消盒 bbox 回落基线正方形；(d) 忙窗（250/200ms）跨越取消/恢复时点后事件与像素均为新状态，无旧代回放。flag OFF 对照：87 帧收敛到 4 个 distinct 哈希、零越基线帧、零动画事件、hit-a 由第三击占位失配染红（判别签名）。

## 环境

| 项 | 值 |
|---|---|
| 平台 | Linux (WSL2/WSLg Weston)，窗口内容面 600×1092，scale=1 |
| fixture | tests/fixtures/compositor-animation/comp03-hit-test-pause-cancel.html（时间线：pause@1.5s / cancel@3.5s / resume@4.5s / 判定@5.5s；忙窗 3.4s+250ms、4.4s+200ms） |
| 注入 | `ZW_INJECT_CLICKS="436,311@3000;170,476@4500;548,459@5500"`（窗口物理坐标；doc = win − 91px chrome，见「坐标换算」节） |
| 采集 | frame_dump 512 帧（~8.2s 采集窗，覆盖全部点击与回执） |
| 对照 | flag ON（=1）/ OFF（=0）各 14s，各 3 次注入点击 |
| 分析 | scripts/compositor-animation-dc3-analyze.py（bbox 轨迹 + 冻结段结构化识别 + 回执五格色分类） |

## 实测

### ON（analysis-on.txt / browser-keylines-on.txt）

| 场景 | 指标 | 值 | 判定 |
|---|---|---|---|
| (a) 命中当前几何 | hit-a（位移中 hit-box，doc 落点 x=436 在基线右缘 424 外） | pass @5515ms | ✅ 逆映射命中 |
| (a) 视觉分叉 | hit-box 中心轨迹 | 224→430px，17 帧越基线（>428） | ✅ 采样位移实锤 |
| (b) 冻结几何命中 | hit-b（paused 冻结位移 45px 盒 x 69..189，落点 x=170 基线 144 外） | pass @6792ms | ✅ |
| (b) 像素冻结 | 条带反解位移 | 冻结 46px（理论 45） | ✅ |
| (c) 取消恰一次 | renderer 事件行 | 恰 1 行 `animationcancel selector=#cancel-box elapsed=3.589s` | ✅（忙窗顺延后仍±0.5s 内） |
| (c) 回落基线 | cancel-box bbox | 旋转期方块→基线正方形 500..596×320..415 | ✅ |
| (c) 基线中心可点 | hit-c（doc 548,368 中心） | pass @7790ms | ✅ |
| (d) 恢复续走 | phase-resume 格 + 条带 | pass @7957ms；冻结 46px → 恢复首采 46px，Δ=0 | ✅ 无跳相无重启 |
| (d) 忙窗跨越 | 取消@3.4s+250ms、恢复@4.4s+200ms 忙窗后 | 事件/像素/回执均新状态 | ✅ 无旧代回放 |
| 帧活性 | distinct hash | 504/512 | ✅ 持续动画 |

### OFF 对照（analysis-off.txt / browser-keylines-off.txt）

| 指标 | 值 | 判定 |
|---|---|---|
| 帧数 / distinct | 87 / 4（静态页收敛） | ✅ 无合成采样 |
| hit-box 位移 | 恒 224（基线），越基线帧 0 | ✅ |
| 动画事件 | 0 行 | ✅ 无导出无 cancel |
| hit-a | fail（红）@7726ms——唯一到达页面的第三击（cancel-box 基线中心）占 seq-0 格、id 失配 | ✅ 判别签名 |
| hit-b/hit-c/ev-cancel | 白（点击 1/2 落空不派发——见缺口） | ✅ |
| phase-resume | fail（红）——gCS transform 主线程冻结值非 translateX 形 → NaN → 判不合格 | ✅ 预期（无查询覆盖） |

## 坐标换算（本轮排查结论，后续真实输入 fixture 通用）

1. **窗口 → 文档坐标扣 chrome 高 ~91px**（scale=1）：`TAB_BAR_TOP_INSET 6 + TAB_BAR_HEIGHT 34 + ADDRESS_BAR_HEIGHT 50 = 90`（layout.rs TOOLBAR_HEIGHT），实测 doc = win − 91。初版按 ~40px 假设设计落点，三次点击在 doc 空间偏高 ~50px 全部落空——由 ui_driver 对照实验 + renderer 侧 `recv … x=99 y=169` 反推实测标定。
2. **renderer 对空白区点击静默不派发**（预存在缺口，非本切片引入）：browser 端 `dispatch_page_mousedown` 命中 None → `DispatchDomEvent.selector=None`；renderer `dispatch_dom_at` 对 None selector 走 `hit_test_element(x,y)`，仍 None 时直接返回默认结果，无任何事件到达页面 DOM。真实浏览器语义应派发至 html/body。记 master 缺口表；COMP-03 落点全部瞄准元素规避。

## 局限

- 点击 3（取消盒基线中心）旋转绕中心、任意相位均命中，不能区分「取消前/后」——定位为取消回落基线的回归守卫，取消本身由 animationcancel 恰一次 + bbox 回落基线两路证据承载。
- flag 关 phase-resume 红格部分来自 gCS transform 形态差异（matrix/none vs translateX），非纯暂停语义断言——OFF 侧以「条带恒级联值 + 帧收敛」为主要判据。
- OS 级遮挡/最小化自动化仍不可行（无 xdotool/wmctrl），「切后台」子句维持 RFC §3 记录的忙窗承载口径。
