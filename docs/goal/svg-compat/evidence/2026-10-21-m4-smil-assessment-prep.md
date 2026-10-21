# M4 SMIL 评估素材——animations/ 275 案 v2 失败分类（2026-10-21）

看护轮产出（M2 S1 由基线原 session 持有，本方不碰源码面；数据源
/tmp/svg-m1-baseline2.json v2 基线，解析器留 /tmp 不入库）。

## 分类账（svg/animations/ 275 案 303 子测试）

| 桶 | 子测试 | 案 | 说明 |
|---|---|---|---|
| 接口缺失（M2 可收） | 241 | 236 | 其中 **237 = SVGSVGElement 动画控制**（pauseAnimations/unpauseAnimations/setCurrentTime/getCurrentTime——SVGAnimationTestCase helper **setup 脚手架**调用即死）；beginElement/endElement 2（SVGAnimationElement，S1 表） |
| 断言失败（时序/语义面） | 14 | 9 | custom-events 6（SMIL 事件基触发）、getComputedStyle rgb 4（动画效果应用——呈现/时序） |
| Timeout（结构面） | 23 | 23 | runner 无 SMIL 时钟，测试等待动画帧超时 |
| NotRun（结构面） | 18 | 1 | smil-clock-value-parsing 提前退出 |
| Pass | 4 | 3 | historical 等 |
| 其他（空消息等） | 3 | 3 | |

## 关键判定：SMIL 真实时序面当前被脚手架遮蔽

237/241 接口缺失失败死在 **helper setup 层**（`rootSVGElement.pauseAnimations`
is not a function），SMIL 时序断言根本未执行。**M4 挂账定稿必须在 M2 S3
（SVGSVGElement 动画控制 stub）落地后重跑**才能量化真实时序面——当前
「animations 1.3%」不代表 SMIL 语义通过率，也不代表时序面失败下限。

嵌套 `animations/` 子目录 11 案（geometry/painting/path 下 *-composition，
CSS 插值组合面）归父域，不入 SMIL 账。

## M4 定稿预估口径（待 S3 后复核）

- **接口存在性面**（SVGAnimationElement/Set/AnimateMotion/AnimateTransform 构造器
  + beginElement/endElement）→ M2 S1 表内可收；
- **脚手架面**（pause/setCurrentTime 族）→ M2 S3 stub 可收，释放时序断言；
- **SMIL 时序面**（时钟模型/冻结/事件基）→ 需要 SMIL 时钟实现，体量显著——
  按 goal 契约「SMIL 评估后挂账」，此分类账即挂账量化底稿。
