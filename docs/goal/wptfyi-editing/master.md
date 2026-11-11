# Editing、Selection 与 Input Events控制面

**2026-11-11 · Active**。DC-1 落定：weekly 快照 109,059/154,595，对
11-07 基线净增 +92,875（CI
[run 42947347500](https://github.com/leizongmin/ZeroWeb/actions/runs/42947347500)）。
DC-2 P1 读面落地（六 format 命令 qS/qI spec 化 +114，证据
[dc2-p1-format-state](evidence/2026-11-11-dc2-p1-format-state.md)）；
**P2 apply 面 vendored 引擎落地**：六 inline format 命令非 collapsed
execCommand 切 `__zw_ce_format_apply`（spec action 直译：跨段落 wrap /
跨边界 merge / push-down / clearValue 摘壳 + state override 写面 +
sibCrit css 门 + cef 端点门 + peel 残料溶解 + styleWithCSS 出口补拍）。
三面同树前后二进制对跑断言级 **净增 +1,647 零新增失败**：六 format
文件 10,499/1,430→11,657/272（F2P 1,174）、font 族 +95、multitest
+380（F 恒 2）+misc 6F 转绿——证据
[dc2-p2-format-apply](evidence/2026-11-11-dc2-p2-format-apply.md)。
selection 扩面已接线待下周跑生效：fetch 切 deep（184 案）、独立
selection-face 片——本地 census 51,695/35,631（
[census](evidence/2026-11-11-dc2-selection-face-census.json)）。
**卡点** 无阻塞；距 146,184 验收线差 37,125，大头在 selection deep
面下周跑生效（census 口径 +~33k）与 input-events（201/3,378）等 0 覆盖面
接线。已知 P2F 8（strike/underline class-span 作者样式载体——Chrome 对
作者表携带 decoration 的 span 原位并 style/识别已激活，我方 gCS 无作者
规则臂，P1 FIXME 挂账升级 P3 待办）。
**下一步** ①gCS 作者样式表规则臂 + 载体识别（8 P2F 清账，state 读 +
apply 原位并 style 双面）；②六文件残余 272F 长尾按簇开修（bold 数值
权重边界 merge、u/s 多值 decoration 偏序删除、italic var/dfn 归一、
sub/sup vertical-align 邻 span）；③selection bidi（0/92）、shadow-dom
（27/113）、input-events 小桶按产物归类开修；④下周快照落做 DC-3 同口径
对账（净增 ≥130,000 + 邻域回退核对，P2 语料全量增量随周跑入账）。

[执行契约](../wptfyi-editing.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | 完成（11/11 片；快照 109,059/154,595） | [dc1-run-shards](evidence/2026-11-11-dc1-run-shards.md) + [dc2-selection-wiring](evidence/2026-11-11-dc2-selection-wiring.md) |
| DC-2 按簇收敛 | P1 读面 +114；**P2 apply 引擎落地（三面 +1,647 断言级零新增失败）**；selection 扩面已接线 | [triage](evidence/2026-11-11-dc2-fail-cluster-triage.md) + [dc2-p2-format-apply](evidence/2026-11-11-dc2-p2-format-apply.md)：残余 P2F 8 = 作者样式载体（gCS 作者臂 P3） |
| DC-3 全量对账 | 对账口径就绪（净增 +92,875，距线 37,125） | coverage-2026-11-11 快照 + 验收线 146,184 |

碰撞记录暂无。遇到共享文件或语料 pin 冲突时，记下对应 goal、范围和可并行工作。
