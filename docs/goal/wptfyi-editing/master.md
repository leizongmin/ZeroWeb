# Editing、Selection 与 Input Events控制面

**2026-11-13 · Active**。DC-1 落定：weekly 快照 109,059/154,595，对
11-07 基线净增 +92,875（CI
[run 42947347500](https://github.com/leizongmin/ZeroWeb/actions/runs/42947347500)）。
**d14 strike 内部界 peel + sub/sup 残段保面批**：face 1 断言级
**F2P=15/P2F=0**（37F→22F：strikethrough 16F→7F、subscript 3F→0F、
superscript 3F→0F，bold/italic/underline 字节级恒等）。四刀：① S1 族
strike 解除——composite end 侧内部界解析（包裹整体随选区侧、逐层登记
拖出段）+ peelOne 按壳 specified 值补裹拖出尾段（S1 八变体语料转绿）；
② 三门控收面（解析限 underline/strikethrough 命令、补偿 longhand 限
`<s>` 壳 × strike × css-true、drag peel 跳 peelRestore 端点回写）——
首版无门 face 一度 62F，对账归因后归零；③ sub/sup 嵌套壳残段 retag 到
最内层壳面（vaInT）+ 贴边 sandwich 门（start/end 链逐层贴边 + end 链
见 vaInT 面）——语料十行规则「残段保最深 va 壳面」，sup 5F→0F 含 d13
既有 3F 全 F2P；④ s 壳收编序列化槽（cv 摘 `<s>` 壳设槽 → FV 尾臂 wrap
落 longhand 合并值，S2c 转绿）。face 2 mt 断言 2=2 / misc 22=22 恒等；
哨兵电池 fail 名集 34=34；钉测 7/7；selection 部分语料（selection/ 全
86 页 + editing/other 12 快页）名集 P2F=0，余 ~47 慢页未跑全（delete
mutation 面单页 ~110s，本轮改动不经 delete 管线，覆盖差非回归——全量
单次 ~15 分钟超守卫墙钟，分段跑法与坑记入证据）。方法学教训入档：
**探针期望必须逐条从 data/*.js 直读**（本轮探针 N5/N6 期望自拟互换致
首版 retag 方向写反、探针假绿掩盖语料真值；face 对账 P2F 名集暴露三处
门控缺口）。残余：strike 7F（S2a/S2b/S2e ADD 邻壳不吸收 + 方向规则、
M1 multipara、qI 族 3 条）、bold 8F / italic 3F / underline 4F 维持
d13 分类。证据见
[dc2-va-peel-interior-strike-shell](evidence/2026-11-13-dc2-va-peel-interior-strike-shell.md)。
**d13 sub/sup va 壳批**：sub/sup ADD 面对祖先 inline vertical-align
decl 壳（任意值）按 Chrome removeInlineStyle 建模先清后裹——split-
conflict 链补 va decl 祖先（peel 抽取 + sibCrit 并拢邻接 sub/sup）+
contained clearValue pass 剥 decl 摘空解包（forceValue 顶层裹 tag）。
face 1 断言级 **F2P=20/P2F=0**（subscript 13F→3F、superscript 13F→3F，
余四文件字节级恒等，57F→37F）；face 2 恒等（mt 2=2 / misc 22=22）；
selection 全语料恒等（35,668P/16,019F 零漂）；钉测 7/7；哨兵
d13vashell 入库 7 案（旧红/新绿）。同批定谳 **d9 账目②瓦解**：bold
900 corpus css=false 与 cssFlag qS 族非页内跨 case 污染，而是**语料
切片割裂伪失败**——run 页 meta variant 割断条目，注册期副作用无条件
执行而 execCommand 闭包只在窗口内执行（subsetTest 跳 test 创建 +
test() 同步），割裂条目 got=输入原样，Chrome 于 wpt.fyi 同红；
sub/sup va 族真值仲裁 = 语料期望（Chrome 录音）与 vendored spec 算法
（复现嵌套形）相悖，修复按语料建模。六文件 37F 全谱重分类（伪 ~17F /
真 ~20F）与证据见
[dc2-sub-sup-va-shell](evidence/2026-11-13-dc2-sub-sup-va-shell.md)；
经验沉淀见 learnings bugs 2026-11-13-wpt-editing-corpus-truth-sources。
上一批 **d12 桥层修复批**：bold 700/900 toggle-off 残余根因修订并落地——
d10/d11 定谳的"混合树致 contained 集空转"是表象（克隆壳 plain 化对齐
后 got 不变；手工几何实证 contained walker 双宇宙均正常收中），真毒是
**plain 树插入侧 live-range 边界补偿缺失**：R263
`__zwAdjustRangesForInsert` 只接 doc/handle 面，plain mutator 缺装，
JS 侧 shift 的 `setEnd(parent, kids+1)` 撞 part06 树长校验被静默
catch，live range 塌缩 → contained 集空 → 摘壳永不发生（tc-scope
宇宙由宿主桥 proxy insertBefore → 宿主侧选区自保补偿，探针实测
eo=2 vs doc eo=1）。修复 = plain mutator 接 R263（parentNode 落位后
调用，补偿写 `_endOffsetBase` 槽绕 setEnd 校验）+ peel 克隆壳按被切
壳形态分流保持全 plain 链。face 1 六文件断言级 **F2P=4/P2F=0**
（bold 3036P/12F→3040P/8F，余五文件恒等，61F→57F）；face 2 mt/misc
fail 名集 24=24 恒等（P 差 +56 全落慢页截断窗口）；钉测 7/7；
selection 全语料对拍断言集恒等（35,668P/16,019F，P2F=0）。哨兵
d10diag/d11tree 转期望形行为断言全绿（此后 Fail 即回退信号），d10
宇宙面定谳、d11 混合树表象与 forceValue 'normal' 负结果入档见证据
d10–d12 节，
[dc2-bold-toggleoff-universe-diag](evidence/2026-11-13-dc2-bold-toggleoff-universe-diag.md)。
DC-2 P1 读面 + P2 apply 引擎 + P3 作者载体臂累计落地后，**P2 主循环
⑥·italic ADD 壳内嵌套 + collapsed qS 记账**（2026-11-13，d9 批）：
peel 链 italic 镜像门（ADD 面冲突 decl 壳——oblique——保壳内裹非抽
取，italic.js:640-647 双 flag 录形）+ part06 collapsed typing 臂接通
state override 记账（part01d `__zw_ce_state_override_arm`，on =
!DOM-only state；gate14 双执行保真、value override 不记 qV='' 口
径；首版 on 直传 state 方向反置由 scratch 探针定谳）——face 1 断言
级 **F2P=5/P2F=0/newF=0/goneF=0**（italic 6F→3F、bold 13F→12F、
d8diag 11/11 转全绿、d9diag 新增 10 case 全绿），face 2 mt/misc
fail 名集 24=24 恒等（P 差 -240 + 名漂 4 全落 `?9001-last` 慢页窗
口承 d7/d8 口径），证据
[dc2-italic-add-shell-collapsed-qs](evidence/2026-11-13-dc2-italic-add-shell-collapsed-qs.md)。
上一批 **⑥·italic toggle-off 值面**（d8 批）：bold ⑥ peel 臂之
italic 镜像（css-false decl ≡italic 转 i、css-true 切分幸存 i span
化）+ ③ 阻断载体资格两轴化（bcEligVa/bcSkipWrapVa 拆轴，cite/dfn/var
仅关值面跳外裹下潜内裹；首版单轴跳外裹产 notbold ADD 面 6 断言
P2F 定谳两轴必要）——face 1 断言级 **F2P=12/P2F=0**（italic
18F→6F；bold 13F 恒等），证据
[dc2-italic-toggleoff-face](evidence/2026-11-13-dc2-italic-toggleoff-face.md)。
更早批次：③ bold/italic 载体吸收与边界挖入（d7 批 +55，bold
42F→13F、italic 24F→18F）、P1 读面 +114、P2 apply 面 +1,647、P3
作者载体 +26、strike 新裹标签 +37、主循环 ⑤ 元素载体折叠 +10、
① decl 面壳多值偏序 +70、② bold 数值权重边界 + cef 门 rc 面 +60
（证据见下表与 [evidence/](evidence/) 目录）。
selection 扩面已接线待下周跑生效：fetch 切 deep（184 案）、独立
selection-face 片——本地 census 51,695/35,631（
[census](evidence/2026-11-11-dc2-selection-face-census.json)）。
**卡点** 无阻塞；距 146,184 验收线差 37,125，大头在 selection deep
面下周跑生效（census 口径 +~33k）与 input-events（201/3,378）等 0 覆盖面
接线。六 format 文件残余 22F（bold 8F、italic 3F、u 4F、strike 7F、
sub/sup 0F；d14 收账后），d13 重分类 + d14 修订：**伪 ~17F**（切片割
裂 harness 固有——bold 6F = 条目 139/75/204、italic 1F = 条目 131、
u 2F = 条目 65/133、strike 1-2F，任何浏览器同红，不进修复队列）；
**真 ~13F**：strike 7F = S2a/S2b/S2e ADD 面邻壳不吸收 + range-start
face 方向规则（sibCrit 对 u/s tag 形按 tag 名判同的收窄面）、M1
multipara 裸 span 溶解、qI 族 3 条（`<strike>…` qI-after、
`foo<s>[bar]</s>` qI-after/qI-before，切片伪失败族待复核）；b-in-i/
b-in-s 块边界嵌套 bold 2F + italic 2F、multipara u 2F + u-qS 条目
134 维持；sub/sup 嵌套双 tag 6F 与 strike S1 八变体/S2c 已 d14 收账
（junk 壳缺陷随 remat skip 一并消除）。d9
账目②（cssFlag 生命周期 + override 残留边界）已瓦解并入切片伪失败，
不再列为修复项；桥层支线三 bug（tc.querySelector innerHTML 重建后空
包装、addRange 后 getRangeAt(0) 越界、B2 臂双落地）未动，主链经 R263
补偿后已与 B2 臂解耦。
**下一步** ①strike 残余 7F（S2a/S2b/S2e sibCrit tag 收窄 + 方向规则、
M1 multipara 裸 span）与 multipara/跨块族（M1/M2/B1-B4 per-p 拆壳）
评估开修，selection editing/other ~47 慢页补全；②selection bidi（0/92）、shadow-dom（27/113）、
input-events 小桶按产物归类开修；③下周快照落做 DC-3 同口径对账（净增
≥130,000 + 邻域回退核对，P2/P3 语料全量增量随周跑入账；face 哨兵
探针：d5decl/d5diag/d6cef/d7diag/d10diag/d11tree/d13vashell 全绿、
d6diag 24/25、d8diag 11/11、d9diag 10 case 全绿）。

[执行契约](../wptfyi-editing.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | 完成（11/11 片；快照 109,059/154,595） | [dc1-run-shards](evidence/2026-11-11-dc1-run-shards.md) + [dc2-selection-wiring](evidence/2026-11-11-dc2-selection-wiring.md) |
| DC-2 按簇收敛 | P1 读面 +114；P2 apply 引擎（多面 +1,647 断言级零新增失败）；P3 作者载体臂 + strike/⑤/①/② 各批 + ③ 载体吸收（+55）+ ⑥ italic toggle-off（+12）+ ⑥ ADD 壳内嵌套 + collapsed qS 记账（+5）；均零回归；selection 扩面已接线 | [triage](evidence/2026-11-11-dc2-fail-cluster-triage.md) + [dc2-p2-format-apply](evidence/2026-11-11-dc2-p2-format-apply.md) + [dc2-us-carrier-author-arm](evidence/2026-11-12-dc2-us-carrier-author-arm.md) + [dc2-strike-wrap-tag](evidence/2026-11-12-dc2-strike-wrap-tag.md) + [dc2-element-carrier-fold](evidence/2026-11-12-dc2-element-carrier-fold.md) + [dc2-decl-shell-partial-order](evidence/2026-11-12-dc2-decl-shell-partial-order.md) + [dc2-bold-value-face](evidence/2026-11-12-dc2-bold-value-face.md) + [dc2-bold-italic-carrier-merge](evidence/2026-11-13-dc2-bold-italic-carrier-merge.md) + [dc2-italic-toggleoff-face](evidence/2026-11-13-dc2-italic-toggleoff-face.md) + [dc2-italic-add-shell-collapsed-qs](evidence/2026-11-13-dc2-italic-add-shell-collapsed-qs.md) + [dc2-bold-toggleoff-universe-diag](evidence/2026-11-13-dc2-bold-toggleoff-universe-diag.md)（d10–d12：宇宙面定谳 + 桥层修复落地，bold 12F→8F 零回归）+ [dc2-sub-sup-va-shell](evidence/2026-11-13-dc2-sub-sup-va-shell.md)（d13：sub/sup va 冲突壳 + 语料红账重分类，F2P=20 零回归，六文件 57F→37F）+ [dc2-va-peel-interior-strike-shell](evidence/2026-11-13-dc2-va-peel-interior-strike-shell.md)（d14：strike S1 内部界 peel + sub/sup 残段保面 + s 壳收编槽，F2P=15 零回归，六文件 37F→22F） |
| DC-3 全量对账 | 对账口径就绪（净增 +92,875，距线 37,125） | coverage-2026-11-11 快照 + 验收线 146,184 |

碰撞记录：本批 pin 钉测与并行流 D1 的 workspace 全量 test+clippy 共
享 build 目录锁（2026-11-13），改用独立 CARGO_TARGET_DIR 完成钉测，
未触碰对方进程；后续重型编译撞锁时同法分流或等待。
