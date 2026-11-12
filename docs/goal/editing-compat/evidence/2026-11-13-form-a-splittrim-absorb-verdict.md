# Form A splitText 欠发射面定谳：吸收除名（2026-11-13）

## 结论

Form A 立项症状（splitText 头截断重放树未裁，D2 钉测 v1 形 `fooobaz<i>ob</i>az`）**不复现**——D2 v1 同形全流程重放树与 JS 视图均 `foo<i>ob</i>az` Chrome canonical 恒等。Form A 从「嵌套簇余量」除名；钉测 `test_form_a_splittext_trim_emission_r3254_m3`（crates/engine/src/js_dom_bridge_tests/part111_form_a_splittrim_probe.rs）转常驻正期望。

## 探针设计与读数（双向双子探针，一次全绿）

探针文件 part111，handle 世界（V8Sandbox + generate_js_dom_shim + register_dom_callbacks + mutations 流 + apply_dom_mutations 重放）：

**① 直接 splitText(3) 子探针**（handle 章完整文本节点 "fooobaz"，不涉编辑命令）：
- JS 视图：head `t1.data=="foo"`、tail `tail1.data=="obaz"`、`h1.innerHTML=="fooobaz"` 全过；
- 发射流：`SetTextOnHandle{text:"foo"}`（R196 handle 门 trim 臂，part04.js）+ `CreateTextNode{text:"obaz"}` + 插入变异齐全；
- 重放树无孤儿无重复（apply 零错）。

**② execCommand("italic") 全流程子探针**（D2 钉测 v1 同形：contenteditable 单文本节点，range offsets [3,5) 选 "ob"）：
- JS 视图 `h2.innerHTML=="foo<i>ob</i>az"`；
- 重放树 `doc.inner_html(#fah2)=="foo<i>ob</i>az"`（D2 时代本形曾实证 `fooobaz<i>ob</i>az` 头截断未达重放）。

## 归因

D2 时代 v1 形红样本的症状链落在嵌套簇 apply 面（重放坐标/父链/发射到达），非 splitText 发射缺失：E1 收口注记的 apply 面三缺陷收敛已覆盖——① `__zwChildIndex` 契约错位（E1 融合视图身份终验 + B1 ④ 现算）、② contained 集截断（D1 父链重挂 + B1 insertBefore 分发臂）、③ override 泄漏（B1 r7 状态表象定谳）。splitText 发射本体在 handle 章完整路径恒正确，与 E1 门禁批 mutation 流实证「splitText 双发射正确」一致。

R196 裸臂静态读法（`handle` 为假时 `cur=''` 连 split 失败）属物化未登记子，JS proxy 世界不可达（createTextNode 即登记 handle），不构成 JS 可观测缺陷面；重放侧 parsed Text 域归 R335（sel 域 `_write` 发射完整），残余面已由 ip/ihtml 道承接口径覆盖，不另立账。

## 验证（绿分支树面=纯测试新增，产品码零字节变化）

- 目标钉测：test-guard 包裹 1 passed（3918 filtered，/tmp/form-a-probe-run1.log 当轮快照）；
- 全 lib 名册腿：**3912P/7F/0I**（vs E1 r2 post 3911P/7F/0I：+1P=本钉测参与跑，L6 锚「passed 允许 +1=解钉测参与跑」明示通道）；失败名册逐名与 [E1 r2 档](../../../acceptance/dps/e1-enginelib-failures-now.txt) diff 恒一；
- `cargo fmt --all -- --check` 零 diff；`cargo clippy -p zero-engine --all-targets -- -D warnings` 干净；
- 语料腿不重跑：partNNN.js/js_dom_bridge.rs 零字节变化 ⟹ 语料读出与 E1 r2 post（batch-nested-e1-gate-v1-r2 01:35:41 ALL-DONE 八腿全绿）同域字节恒一，按质量门禁「按实际影响选择检查」收缩。

## 账面

- 账本 2026-11-13 01:50 Form A 认领条就此关闭（双向预案绿分支）；
- 控制面状态行轮换记本定谳；DC-4 下一步不变：ip 206 div/p 容器子族 116 起刀（ihtml 119 随道；exec1 474 域决策门）。

## 换基传递（push 前 rebase 复验）

提交未推时 pull --rebase 拉入他轮 d7 刀 dd0e52c5d（part01c +24 / part01d +382 bold/italic 载体吸收五臂 + wpt 探针页）与 a217fc7f4（wptfyi docs）：入站与本刀域（part04.js/part05.js/js_dom_bridge.rs/控制面）零文件相交，刀体跨 rebase 字节恒一（`git diff 6163a3d14 fe9744290 -- 本刀四文件` 为空）。行为面上 part01d 改 italic 命令流（探针②执行路径），按字节绑定纪律新基复验：探针钉测复跑 ok（1 passed）；全 lib 3912P/7F/0I 总数与失败名册逐名仍与 E1 r2 档恒一（d7 新增断言不落 engine-lib 测试函数）。VERDICT 随附传递成立（knife11 先例三要件：刀体字节恒一 + 入站零相交 + 新基复跑绿）。
