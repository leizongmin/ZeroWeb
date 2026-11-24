# kc5r81 prep 勘察收账（verify 位）：CT 起动探针 v2 双形态定谳＋columns 残 34F 静态闭合

日期：2026-11-24 08:4x–08:5x。r80 verdict（8844b789c）＋勘误回写（2f4a05be5）后的换岗续轮：前 verify session 08:40 rate-limit 终止，本轮接续只读勘察位；r81 实施位在对向 fork 在飞。零树编辑、零重型发射、tally 不动。

## 1. probe-r81 v2：克隆 fixture class 通道 transition 起动四臂

工件：`.acceptance/wptfyi-css/kc5r81-prep/`（probe-r81-ct.sh / probe-r81-page.html / results/probe-r81-json.json）。协议复刻 r80 probe-r80-direct.sh 直调形：r80 修复 bin（c13df7e3）直跑 css-values 单页 `zw-r81-ct-start.html`，锁 `/tmp/zw-r81-probe.lock`，页跑完即删。v2 与 r80 verdict 探针处方的差异：端点不走 `el.style` 写，全部走样式表 class 规则族（复刻真实门页 options.styles 形）。

| 臂 | 形 | 结果 |
|---|---|---|
| G | clone＋class 端点 margin-%（002 页同构：加 .how→reflow→加 .to→即读） | Fail：start=80%（瞬时跳 to），anims=0，`sel=null hdl=__n3` |
| H | clone＋class 端点 width（.r81w 规则族，同序） | Fail：start=100px（瞬时跳 to），anims=0 |
| I | how-first 顺序变体（.how 与基类同挂→gCS 读 from→加 .to→即读） | Pass（mid≠100px）——**歧义臂**：探针未记录 pass 明细，"已起动持中程值"与"卡 from 不动"不可分辨 |
| J | 50 并发 clone slice（批量 .how→reflow→批量 .to 零 reflow→采样 s0/s25） | Fail：s0=100px（瞬时跳 to），anims=0 |

定谳三点：

1. **起动钩缺口与写法通道无关**。class/样式表通道在 clone 后代上与 r80 verdict 假设的 `el.style` by-handle 写通道同形失败（anims=0＋同步跳 to）。r80 verdict「克隆 fixture by-handle style 写起动钩疑未接」假说修正为：**handle 身份节点（`__zwSelector=null`）上 class 变更驱动 recalc 后 transition 起动臂未捕获端点变更**——通道无关。
2. 失败面收敛在起动臂。G/H 的 from/to gCS 读全部返回真值（33%/80%、41px/100px）＝r80 clone-gCS bind 修复在该通道确已生效。
3. I 臂（how-first：transition 属性先于端点变更完成一次带 gCS 读的完整 recalc）不跳 to，但 pass 值未记录，"起动成功"与"卡 from"两种机制都满足断言；r81 实施若需分辨，v3 探针须记录 mid/after 具体值＋transitionend 事件计数。

## 2. columns-interpolation 残 34F：C 族静态闭合（源码复核，接 NOTES 08:4x 节）

- 根因行实证：`crates/style-system/src/property/inherit.rs:1940` css-wide keyword 表 `"columns"` 臂只拷 `column_count`＋`column_width`，漏 `column_height`（全文件 `column_height` 仅 :1961 initial 臂 1 处）；对照同表 rule-inset 8 槽全拷完整型＝columns 系 height 后扩展漏补的孤立疏漏。
- 串化锚点：`crates/engine/src/js_dom_bridge/computed_style.rs:1026` `"columns" => columns_shorthand_to_css(...)`（Auto 槽省略形）。
- 机制：expected 侧（参考元素 static gCS 折叠，interpolation-testcommon.js :379-385 通道，NOTES 08:4x 静态闭合）经该臂丢 height 槽→`50px`；CA actual 侧动画 keyframe resolve 别径（apply_to_computed_style）保全双槽→`50px / 100px`。同值异形，非插值语义缺口。
- 处方（**已由实施位 session 认领执行＝(a)-cut，08:56 NOTES**）：1 行 fix＝:1940 臂补 `style.column_height = default_style.column_height;`。预注册门＝(a)-only 电池 columns-interpolation F2P=3（恰 CA at(-0.3/0/0.3) 三键）/P2F=0 fail-stop/哨兵 css-transitions SET-IDENTICAL；b0 臂恒指最新已收账 bin（r79 b1 f45097a1，r80 零落库不换锚）。B 族 14F＋WA 7F（离散跳变/keyword 端点 apply 缺口）与 A 族 10F（CT inherit 端点插值）另刀。
- 执行窗约束（认领前记录，仍有效）：inherit.rs 属驻留 face（KC-4 cis hunks :312-345），fix 编辑须避对向 r81 构建扫入窗＝防污染其哨兵；执行器与 4 腿电池脚本由实施位预制（`.acceptance/wptfyi-css/kc5r81a/` launch-a-cut.sh／battery-a-cut.sh，锁 `/tmp/zw-kc5r81a-battery.lock`，unit `zw-kc5r81a-battery`），sibling 勿另写执行器/电池脚本防 results 双写毒化。

## 3. 窗态与远端

- 对向 fork（57ee783e 谱系）r81 WIP 在飞：js_dom_bridge.rs 08:25:31 → computed_style_cache.rs 08:25:44 → transition_registry.rs 08:26:05 → callbacks.rs 08:50:05 连续编辑；驻留 face 14 文件（KC-4 cis auto-flag＋KC-5＋r81 layer-2 transition 起动锚定）。
- origin/main E0609 已解：3fc122daa 剥离修复在 tip，`git grep 'contain_intrinsic_(width|height)_auto' origin/main -- crates/` 零命中＝main 可编译；d4bd1cec1 判读注记「43ca50c9d 起 6×E0609」系剥离前窗态，不再是挂起项。
- behind=3（ca7ea0719 kb210 部署收账／d4bd1cec1 DC-2 s10 刀② part03.js +159／3842330c7 wpt 周快照）：树驻 face＋html-semantics staged docs 批（M+D，勿碰）→ pull --rebase 阻塞＝push 同挂，docs 提交先行落本地。
- **part03.js 跨基线警示**：d4bd1cec1 改 part03.js（frag 展平补章＋SR 入口续走），驻留 face part03.js 段基于旧 main——后续任何 pull/face 剥除-复贴循环前，face patch 的 part03 段须按新基重生成（face-patch-stale-base 纪律），禁直接 apply 旧快照。
- watcher `zw-cssfollow-1105b`（08:45:52 起，BASE=ca7ea0719，v6 三事件只读观察）＝**本流 css 跟踪族工具**（08:55 假触发已定谳＝docs-only push 过滤漏洞：排除 `html-semantics-compat/` 子目录漏顶层 `.md`）；v7 修正重挂由实施位 session 负责，本 round 不另挂 watcher 防双发。
- (a)-cut 执行认领（08:56 实施位 NOTES，已读）：C 族 1 行 fix 与 4 腿电池归实施位 session；电池跑驻留树态冻结 bin（FACE 锚 provenance），pull/face 重建只阻塞提交不阻塞电池，收账按 shared-tree-battery-queue-protocol 窗口纪律；verify 位不另写执行器/电池脚本、不重复发射。
