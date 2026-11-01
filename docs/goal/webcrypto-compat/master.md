# Web Crypto 兼容二期运行时控制面板

**状态**: DC-1~4 全部落账有据并推送（`5894374c1`，2026-10-31）；门禁
build/clippy 绿；33,109 分母零回归锚在现执行 tip `ee2ee9ec1` 三跑整
文件对锚 cmp 全等 ×3（2026-11-01，s182~s205 + justify 簇两轮 + 渲染流
R5370/R5372/R5373/R5374 + html-semantics 流 + net-api 流 S19 +
zero-web 流 change-on-blur/ClearHttpCookies + editing 流 sv-setup
style IDL + zero-web 流 t-p126 softnav 簇 + html-semantics 流 s201
SVG 命中注入 + s202 legend/option form IDL + s203 number 值消毒 +
渲染流 R5374 abspos 表 available-size 钳 CB + html-semantics 流 s204
pattern IDL/v flag + s205 style-with-value 查询面（shim `part01c/
part06.js` +438/−3）变更全吸收；
W4 案完成路径竞态低频采样（累计 5/30 轮，本轮零采样）
——均为 runner 既有代码，状态账零差异，见
[evidence 追记](evidence/2026-10-31-gates-and-closure-status.md)）；
唯一缺口 `cargo test` 残余 1F
`test_ua_display_li_list_item_canvas_inline_s24_flip` = 渲染流 D4 待裁决
冻结面（s194 账 engine lib 3642P/1F 在册同键）。收口归档待该 1F 清零
或 D2② 授权。M1/M2 里程碑已归档（`archive/2026-10-31-m1-m2.md`）。
**收口前提轮询（2026-10-31 12:55 → 2026-11-01 13:45，观测面自 12:13 起
扩为 main + D4 裁决分支 `origin/site-opt/s24-flip-red`）**：fetch/Request
簇 9F 已由 s179 `a3768d3e7` + s180 `2e260d532` 全清；唯余 ua_display_li
1F——git 追溯定性为**跨流通道交互 × 渲染流 D4 待裁决冻结面**（s161
handle gCS 通道把裸 createElement 产物路由到 host 面，host `li→Block`
系渲染流 D4 在册冻结值；修复=渲染流 D4 双面同提交翻转，用户裁决门禁，
ETA 未知，本 goal 不越界；渲染流 R5368 2026-11-01 记账仍挂对端 D4）。
**33,109 锚已在现执行 tip `ee2ee9ec1` 复验状态账逐位一致 ×3**（25,788P +
7,320F + 1T，guard 零干预；实锚链 7065ea39f→1b6b9b291→65de5f28e→
b0a6f9174→3f1399195→ba4c74fcb→edaed928a→0d4966f9d→f05b0efc1→
e30ab8494→dd0eb2ec8→578240175→e2910e933→8e84cb9a2→ebdfe3939→
a5a11cd07→66f4871c1→a88ab50c6→61720d309→ee2ee9ec1，渲染流 R5370 table OOF child layout +
s196 `_zwMEl` 安装面共享化 perf + editing 流 justify 查询面兜底
（`part01c.js` +105 行）+ html-semantics 流 s196 `:active` 伪类通道
（dom active_element 槽 + engine 查询面注入 + shim part04/part06
+112 行）+ 渲染流 R5372 abspos 嵌套 CB margin（`abspos.rs` +31 行）+
html-semantics 流 s197 fieldset disabled 焦点域（shim part03/04/05/06
+110 行）+ 渲染流 R5373 父高度回填塌缩屏障（`postprocess.rs` +36 行）+
html-semantics 流 s198 fieldset type IDL（shim `part04.js` +5 行）
+ net-api 流 S19 Headers 大小写保真（shim `part01/part03` +35 行，
`_hCase` 旁路四处）+ html-semantics 流 s199 Space 键激活抑制门
（runner `testharness.rs` + shim `part04/part06` + `script_gen.rs`）
+ zero-web 流 change-on-blur 去重闩（engine 桥 + shim）+ editing 流
sv-setup style IDL（shim `part03.js` +136 行）+ html-semantics 流
s200 键盘激活 click-only 事件序（runner `testharness.rs` +21/-4）
+ zero-web 流 t-p126 softnav 簇（PR #265，engine bridge/shim 7 文件
+1127/−77）
全吸收，见 evidence 2026-11-01 04:25/04:35/06:20/06:27/07:15/07:50/
08:35/08:50/09:09/09:20/09:33/09:43/10:05 追记）；W4 案完成路径竞态低频
采样（累计 4/20 轮，本轮零采样、三跑全 cmp 全等，状态账不受影响，
锚验收以状态账逐位/整文件 cmp 为主判据）；push 前 rebase 后重查
执行路径前移流程补强（09:20 追记）；
处置路径维持待用户
决策表 **D2**，ua_display_li 清零即当场收口。
**最后更新**: 2026-11-01 14:45（锚复跑现执行 tip `809a14134` ×3——
14:35 锚账 push 前 rebase 窗口又吸入 html-semantics 流 s207
form.action 文档 URL 反射特例（missing/空串返 document's URL，
base-url 变体 2F 收口，engine shim `part03.js` + `part09.rs`，shim 入
runner 包）：release runner 重建 27.0s（sha256 `7b7a9939…` 三跑稳定）
+ test-guard 直包三连跑（guard 三跑零击杀，墙钟 93s/93s），**RUN1/
RUN2/RUN3 对 M1 终态锚整文件逐字节一致 ×3 且三跑互等**（7,208,436 B，
sha256 同值），33,109 条状态账 0 差异（25,788P+7,320F+1T），W4 本轮
零采样累计 9/37 轮；**锚窗口推进 517ccefda → 809a14134**。本轮连续
三窗口各吸一个并行流执行路径前移（M4 注释面→R5376→s207）锚三度
复跑；起跑前 ps 二遇 editing 流 runner 兄弟进程（zz-fc-slice 迭代），
按纪律等其自行退出后起跑未硬解。轮询面：PR #267 head 仍
`2154ac296`（14:09 落定）**仍 OPEN 未合入 main，D2① 未触发**；冻结
门 main 侧末触仍 `c41b0c750`，ua_display_li 在册未清，D1 无新账面，
见 [evidence](evidence/2026-10-31-gates-and-closure-status.md)）

---

## 当前状态

DC-1~4 已全部达成（入口文档 [`../webcrypto-compat.md`](../webcrypto-compat.md)
v2.1 契约）：

- **DC-1（语料补全）✅**——实质结论更正：pin 语料一期已补至全量（上游 202
  blob，198 文件在库、0 失败，4 个为非测试元数据被扩展名分流正确排除）；
  「derive_bits_keys 32/38 未拉」系 coverage universe 口径假象（转正重排 +
  window/worker 变体路径），pin 目录事实是 11 案全在库全执行。证据：
  [M1 语料与基线](evidence/2026-10-31-m1-corpus-and-baseline.md)。
- **DC-2（转正甄别清单）✅**——一期 7,320F 挂账域在上游 master
  （cb6cc97a，2026-10-31 查证）**全部转正**（`.tentative.` 50→0，迁入
  modern-algos/ + secure-curves/ 两族；META.yml 无 tentative 声明）。
  逐域切片建议（域面/分母/范围/依赖候选）落
  [DC-2 甄别清单](evidence/2026-10-31-dc2-promotion-screening.md) 并登记
  下方待用户决策表 D1。
- **DC-3（扩面基线）✅**——全量复跑 123 案 / 33,109 子测试 = 25,788P
  （77.89%）+ 7,320F + 1T，双跑逐字节一致；与一期终态锚
  （m6-residual-full.json.gz）逐 (case, subtest) 对照**状态变化 0 例**，
  33,109 分母零回归底线保持。分域分类通过率 + tentative/非 tentative
  失败归因见 M1 证据文档。
- **DC-4（coverage 复测）✅**——同工具同缓存对 M1 结果复测，与官方
  coverage-2026-10-31 快照逐位一致（分子确定性证实）；分子 ≠ 实跑的
  12,541P 差值与 265 个未覆盖文件逐桶具名归因（分片变体 / 转正族 /
  worker 腿 / 新案 / 记账排除），tentative / 非 tentative 分列归因表落
  [DC-4 对账](evidence/2026-10-31-dc4-coverage-attribution.md)。

**W4 定性**：algorithm-discards-context 1T = runner/shim DOM 域边界
（iframe 跨帧注入 + `top` 回调链），非 WebCrypto 语义域、非语料边界，
维持 DOM 域挂账。

## 与兄弟 goal 的边界

- 33 号 editing-compat（ZeroWeb-3 工作区并行在跑）——无共享面。
- 23 号 selection-compat——无共享面。
- workers 通道结构性挂账（worker 腿 window-only，coverage worker 桶
  13,812 子测试）——归 runner/workers 通道，不在本 goal。

## 待用户决策

| # | 事项 | 状态 |
|---|------|------|
| D1 | 已转正域按域立项：ML-KEM（12 案/3,480）、ML-DSA（10/1,206）、X448/Ed448（26/3,700）、AES-OCB（10/2,652）、ChaCha20-Poly1305（10/1,044）、KMAC（10/1,418）、Argon2（4/92）、XOF digest（8/838）、Hybrid-KEM（7/2,742）、门禁面（12/880），合计 109 案/18,052 子测试（universe 口径）；切片建议与依赖候选见 [DC-2 甄别清单](evidence/2026-10-31-dc2-promotion-screening.md)（依赖含新 crate：ml-kem/ml-dsa/ocb3/x448/ed448-goldilocks/kangarootwelve 等，选型须用户拍板） | 已飞书征询在案（GB-20261031，msg `om_x100b616bdb6c9ca0dde792fae199471`）；待用户拍板；不 BLOCK 本 goal 收口 |
| D2 | 引擎基线红处置路径（收口唯一缺口）：**fetch/Request 契约族 9 已由 s179/s180 全清**（engine lib 3635P/1F），唯余 ua_display_li 1F——git 追溯定性为 s161 handle gCS 通道 × 渲染流 D4 待裁决冻结面（host `li→Block` 冻结值盖过 shim 已翻的 list-item 表），修复=渲染流 D4 双面同提交翻转（用户裁决门禁，非本 goal 域）；① 等渲染流 D4 裁决落地后 make test 全绿当场收口；② 授权本 goal 按 standing exclusion 口径收口（先例 net-api request-cache 98F——归因已在 [gates evidence](evidence/2026-10-31-gates-and-closure-status.md) 落档，非本 goal 域、33,109 分母零回归不受影响，DONE 的「cargo test 全过」按归因口径豁免并行流账面）。**本 goal 不越界代修**（Mission 边界维持） | 已飞书征询在案（GB-20261031，msg `om_x100b616bdb6c9ca0dde792fae199471`）；待用户拍板；期间维持低成本轮询 + 锚复跑，ua_display_li 清零即当场收口。飞书回复无法无人值守读取（bot 无回读权限、user 身份需交互授权，2026-11-01 实证），回复核验只认 git 账面或用户主动知会。**2026-11-01 12:13 D4 双面裁决现身 git 账面**——用户提交 `bb4cba524` 推上分支 `origin/site-opt/s24-flip-red`（触及冻结门、含全量门禁走查账，未合入 main），本 goal 按碰撞纪律不抢着陆，**合入 main 即按①当场收口** |

## 碰撞管理记录（run-rules §9）

- 2026-10-31：立项时无同域在跑碰撞；33 号 editing-compat 在 ZeroWeb-3 并行
  推进，域不相交。
- 一期零回归底线：33,109 分母逐 subtest 对照，锚
  [`../archive/webcrypto-compat/evidence/2026-10-17-m1-baseline.md`](../archive/webcrypto-compat/evidence/2026-10-17-m1-baseline.md)
  （逐 subtest 实际对照用终态锚 m6-residual-full.json.gz，同目录）。
