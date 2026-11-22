# s643 收账证据——平凡窗（零位移＝0 commit）＝join 结果面全持位＝主预判逐字兑现＝stable 8517 持位＝canon f1aef7b7 链复挂第 38 连（NEW=0 GONE=0 FLIPS=0 disagree=0）＋leg 双 norm c735d371 整腿复现链第 46 连 s597→s643＋b643 第 2 轮持位（keys 1735 fail=3 三挂账 IDENT ac1d0cd9＝副形①②未现）＋runner 候选六十持位形主预判兑现（pre=mid=post a431cd3d/129,935,496 三点同值＝零换代）＋solo 发射形同值第 36 连＋**断点存活形第三演＝收账接手变体续演**（上 session 布防→限流断点→本 session 只读接管收账＝零重臂＝同窗收账闭环）

日期：2026-11-22 布防＋窗自动跑毕；2026-11-23 00:1x 本 session 接管收账（上 session 22:58 限流断点〔429 五小时限额〕，窗不受影响自动跑毕＝s641 收账接手变体第二演后续演＝**第三演**）。
环境：本机双 clone 并行；`/tmp/s643`；legs.head=**974cb73fa**；窗＝974cb73fa→974cb73fa＝**0 commit＝平凡窗（零位移）**。
**布防注记（诚实 trail）**：s643 布防在 s642 收账 commit 974cb73fa（22:42:38）内落地＝22:42:52 臂起＝收账 commit 先于布防＝钉扎纪律第 19 演零竞争。上 session 布防后限流断点＝**两臂存活跨断点**（s640 首演＋s641 变体后续演）；waiter.s643.log 空＝首臂即成（nohup 静默）＝s642 首臂 ENOENT 教训修复（复制清单含 rearm 脚本）首演兑现；rearm.s643.log 不存在＝监督者在位未触发＝零真重臂；post.s643.terminal.log＋rearm.s643.terminal.log 双在册＝**两臂齐全第 42 演零空窗**。

## 窗与 legs

- 布防（974cb73fa 内 s643 布防）：桥指针链 b642→b643 逐处核改＋轮次标签 +1（dialog r77／select-event r82／password r82／mpreload r105／waapi r164／focus-dynamic r123／onblur r75／band r76）＋参考 keys 清单照 s642 全集（含 s477 四档＋se.577＋基线 keys＋基线 canon＋**rearm 脚本**）＋bash -n＋清点＋归零审计全过。
- **窗码面＝平凡窗**：窗＝974cb73fa→legs.head 974cb73fa＝**0 commit**（s642 收账 commit 22:42:38 先于臂起 22:42:52＝布防预载落窗前＝实窗 0 commit＝无补录形）；对端 DC-2 切片 5 刀②未入窗。
- 候窗 7m15s（22:42:52→22:50:07）；window.s643.judge＝**clean-soaked procs=0 load<3 across 3×45s checks HEAD=974cb73fa**。
- leg1 22:50:07→22:52:55（2m48s）rc=2，Compiling ×6 Fresh=0＝**平凡窗重编位等形**（≡s639 先例＝源零变重编＝md5 恒＝非换代）；leg2 22:52:55→22:54:51（1m56s）rc=2，Compiling ×0。
- **runner 候选六十持位形主预判逐字兑现**：pre=mid=post=**`a431cd3d`**/129,935,496 三点同值＝**零换代**（leg1 Compiling ×6＝重编位等＝候选六十持位〔≡候选六十收锚基线〕；对端未推 runner 依赖域源刀）。
- 桥测（run-legs 内，test-guard recipe）：test **22.14s**，rcb=**101**（fail=3 所致＝预期形）。
- solo1/solo2 22:55:32→22:55:49→22:56:03（~17s/~14s）rc=2 ×2；failset（`^Fail` 原始序）md5 `329e35ed` 双 solo IDENT ≡s611–s643＝**发射形同值第 36 连**。窗起至全毕 **5m56s**（22:50:07→22:56:03）。

## join 判定（analyze.out）＝结果面全持位＝主预判逐字兑现＝canon 链复挂第 38 连

- `stable=8517 disagree=0 l1only=0 l2only=0`；**NEW=0 GONE=0 FLIPS=0**；分布 **8305P**/206F/5T/1U（≡基线＝Pass ±0）＝**持位形主预判结果面逐字兑现**。
- **canon/keys 双 IDENT held 基线**：s643.stable.canon md5 `f1aef7b7d7eafeeef3dd0989ba6bc3d9` ≡ s634.stable.canon＝**canon 链复挂第 38 连兑现**（s639→s640→s641→s642→s643＝五连链形）；s643.stable.keys md5 `057764d38ee311a47c34ecfc3e1e3192` ≡ held keys。
- **leg 双 norm md5 `c735d37188a0fa0ea338b47d2432a26f`＝双腿逐字节同值＝整腿复现链第 46 连 s597→s643 持位**。
- 平凡窗（触点 ×0）之下 join 面零位移＝"不动"持位平凡兑现（s634/s639 谱形续）。

## b643 判定（bridge.out）＝新基第 2 轮持位主预判兑现＝三挂账未销

- keys=**1735** fail=**3** NEW=0 GONE=0；s643.bridge.keys md5 `ac1d0cd96270e199ed9982672c02664e` ≡ b643＝**换基后第 2 轮同值持位**。
- FAIL 三键持位＝`min_max_single_arg_collapses_to_argument_terms`（对端 v3 刀）＋`test_get_computed_style_numeric_special`（对端 FAM-F 刀＝part03.rs:1297）＋`test_img_breakout_pins`（s641 flip 挂账）＝**副形①〔img 销账回 1736/fail=2〕未现＝副形②〔numeric_special 销账〕未现＝对端未修＝均归对端刀作者面，本流不代修（规则 9/10）**。
- 平凡窗＝桥触点 ×0＝"不动"持位平凡兑现（对端切片 5 刀②未入窗＝无新 flip 挂账形）。

## 哨位（analyze2.out，SENTINELS_DONE）

dialog band r77＝2P 双腿（flaky 键持位第 32 轮）；select-event r82＝279/279 keys 8F Fset ≡ se.577＝持位；password r82＝225/225 keys 0F＝持位；**mpreload r105＝15/15P 双腿＝回位后稳位第 4 轮兑现**；waapi r164＝picker P/checkmark F 双腿；focus-dynamic r123＝121 all P；**onblur r75＝双腿 2F/8P Fset 双 IDENT ≡ob.s559＝稳位第 43 轮兑现**；族带 r76＝band8 hits=8×2 全 Fail（**聚合形持位第 60 连**）；双发伪影 ×2 恒（dup ×4）；爆栈 so1/so2/sob=0/0/0；**'No such' 归零审计 0/0/0/0/0 第 38 演兑现**（预装修复形第 15 轮续）。轮次标签全对齐（r77/r82/r82/r105/r164/r123/r75/r76＝与 s643 布防计划算术一致）。

## 窗底账（s643 窗＝974cb73fa→legs.head 974cb73fa＝0 commit＝平凡窗）

- 本流 0；对端 0；布防预载（s642 收账 commit 974cb73fa）落窗前＝实窗 0 commit＝无补录形。
- 窗闭后开工捕获：c22c3c0de（wpt.fyi weekly cron＝23:49:54 +0800 提交＝窗闭 22:56:03 之后＝docs-only 对端域）＝**归 s644 开工窗面**，与本窗无关。

## 基线决策（s644）

- join：**对账基线继续 held＝s634.stable 8517**（f1aef7b7/057764d3）＝链复挂第 38 连兑现＝（s644 主预判＝持位形续〔8517 持位＋canon 链复挂第 39 连观察＋norm 链第 47 连观察〕；副形＝谱内摆动再现〔记形＝不可预报照记〕；迁移形＝双腿 Fail FLIPS=1＝基线迁移决项提请用户）。
- 全桥：s644 桥基线＝**b644＝cp s643.bridge.keys**（keys **1735** fail=**3**＝ac1d0cd9 第 3 轮持位观察；副形①②销账形在册；对端 DC-2 切片 5 刀②在册〔tt 面续推〕＝新 flip 挂账形按面归因）。
- runner：**候选六十持位续＝a431cd3d**/129,935,496（s644 持位主预判＝pre=mid=post a431cd3d 三点同值预期；对端若推 runner 依赖域源刀＝候选六十一条挂号按面归因）。
