# F6b N 族 handoff 刀收口（2026-11-09，lane8 车道）

handoff 重入（[F6b 三臂刀](2026-11-09-f6b-script-fontsize-carry-knife.md) DC-4 三分表 handoff 4 键：N1 outer font2 remap / N2 inner font3 hoist / N4 tail outer font2 remap / N5 tail inner font3 hoist）。重入条件「专项普查该四键语料期望序列化」在案（[f6b-census-keys.txt](../../../../.acceptance/dps/f6b-census-keys.txt)）；mt 残 9F（F7m 终态）降序首位。

## 定谳：非「语料形分歧」——shim appendChild 隐式 detach 缺口

handoff 立项时判「引擎产出形与语料期望形分歧」（probe 17/0 而 mt 残红）。本刀 side-band trace（[f6bn-trace-r1.log](../../../../.acceptance/dps/f6bn-trace-r1.log)，zz-probe-f6bwin2 页面侧 wrapper 五件，零 shim 改动）推翻该读法：

- N1 全链单次 MUT、carry={bucket:1}/tags=[sub] 正确、POST-in 树 `foo<font size="2"><sub>a</sub></font>baz`（重映射前中间态正确）→ POST-out 树 `foo<font size="1"><sub>a</sub></font><sub>a</sub>baz`（重复尾在 post_insert 内部产生）。N2 同构（POST-in `foo<sub>a</sub>baz` → POST-out `foo<font size="3"><sub>a</sub></font><sub>a</sub>baz`）。
- 根因＝`__zw_ce_f6b_post_insert` ② 异桶重放 `fontEl.appendChild(scriptEl)` 在 mt 顺序执行面不隐式 detach：typed 文本节点为 caret 锚节点态时 script 壳移动不脱旧亲链，元素双亲（font1 + host）被 innerHTML 序列化两次——got 形逐字复现（期望形+幽灵重复尾）。F6b 三臂（捕获/分账/路由/重映射）全正确，唯 DOM 写点非真移动。
- 孤立页同码通过＝缺口状态相关；先例注释在案（part01 F1'-B `_zwCeTypingForwardMerge`「显式摘除后头插——zz-probe-d4x X4，DocumentFragment 无移动语义；元素侧同法保守」）。修法循先例使移动无条件正确，两态语义恒等（显式 detach+append ≡ 正常 move）。

## 刀体（commit 8ac40127b，push 时重放 dcea9b117，part01 +6 + fixture +43）

part01.js `__zw_ce_f6b_post_insert` ② 异桶重放段：`sp.insertBefore(fontEl, scriptEl)` 后、`fontEl.appendChild(scriptEl)` 前插入 `if (scriptEl.parentNode) scriptEl.parentNode.removeChild(scriptEl);` + 先例注释。不新增臂、不动捕获/路由/等桶溶解/A 通道（`carry.inner` 早退与 `cb === carry.bucket` 溶解路径零扰动——W1 见证形与 A 族 17 案回归面）。

fixture：zz-probe-f6bwin.html 转正常驻回归探针（multitest 全克隆、variant ?8001-9000/?9001-last 两窗；孤立页不复现的顺序上下文缺口唯一常驻回归面）。zz-probe-f6bn（因子二分 7 案，修前后均绿无回归价值）与 zz-probe-f6bwin2（trace 载体）判读后撤除。

## 批前预验（pre-s365 组合二进制 16:52 build：part06 `e32619e9`+刀体；组合态 part06 `2fdb996f` 验证由批内承担——批 v1 make 于 17:0x 锚门后重建，全腿绑组合态字节。刀效双组合态在案：预验 N 族翻绿 + 批 f6bwin 腿 N 族红=0，判读不变）

- probe-f6bwin **2089P/3F**：N 族四键 compare 全翻绿零漂移；残 3F＝F6a×2（`#0000FF` 色值大小写）+ F1' fontsize(4)×1（占位 br 壳）——均域外白名单（lane9 后继刀素材），与 F7m 终态残键集恒一。
- probe-f6b **17/0**（oracle 直验面零回归）；zz-probe-f6bn **7/0**（因子控制组）。

## 门禁批（batch-f6bn-gate-v1，head 8ac40127b 组合态）

锚：八点 md5（part01 45f946ea 含刀体、part06 2fdb996f＝s365 新基线、余五点承 F7m）+ 特征串（knife=1/f6pi=5/f6c=1 + F7m 回归面 a1=4/a2=5/con=4）。腿：probe 五腿（f6bwin 2089/3 残红成分双门 + f6b 17/0 + f7m 149/0 + f7 9/0 + k11 224/17 judge）+ ilb/ip/ihtml/exec×3/iol/iul 零漂移（vs F7m 终态）+ mt judge（F2P 恰 4 ∧ P2F=0，vs f7m-mt-post 行级 9F→5F）+ calibrate 翻键白名单（4 entry×双页=8 全落 delete+inserttext 链）+ enginelib 双门（failed 总数==7 + 全量名册，v1.2 门盲点硬化首用）+ battery 七腿 + 二进制特征串。

### 终态判读（v1 18:0x + v1.1 尾批，head 8ac40127b）

- **v1 首射（17:04）受控杀批重发**：判读域内自检发现 calibrate 白名单 grep 带 `styleWithCSS` 前缀，而 census 四键链实为裸 `[["delete",""],["inserttext","a"]]`——照原模式必 ABORT 白名单外 8。批中 bash 增量读脚本不可在线改 → 按 v2 绑定断裂先例受控杀（pgid 组杀 + runner 点杀 + LOCK-FREE 验证），修正 :169 一行后 17:11:22 重发（pid 853894，锚门全过，损失 ≈7 分钟）。
- **v1 前段腿（17:11 发射，~50 分钟）**：probe 五腿 f6bwin 2089/3（残红成分双门：N 族红=0 白名单=3）/ f6b 17/0 / f7m 149/0 / f7 9/0 / k11 224/17 judge 零漂移；ilb/ip/ihtml/exec×3/iol/iul 七腿零漂移（F2P 0 P2F 0 post-only 0 lost 0）；**mt 20176P/8F（rows 20184 恒一，judge F2P 4 P2F 0 post-only 0 lost 0——机制预期恰 4=N 族 N1/N2/N4/N5，assert 级 16→8F = 四键×双页 compare rows）**；**mt 校准 entry 翻键 8/8 全落 delete+inserttext 链白名单（F2P 8 P2F 0 postF 0 lost 0）**。
- **v1 enginelib 判读门 ABORT（脚本缺陷非腿红）**：failed 总数门正则写反（cargo 行格式 `7 failed;` 非 `failed: 7`，提取得 0）；腿本身完整（3821P/7F，13.29s；+1 测试=并行流入账）。从既有日志补判：**failed=7 ✓ + 名册 7-red 逐名恒一 ✓（双门全过）**。修正正则后 v1.1 尾批（18:20:54，pid 908449）按 v1.2b 尾批先例补跑：enginelib 补判（不重跑）+ battery 七腿 + 二进制特征串。
- **残 8F 断言级成分**（f6bn-mt-post.log）：F6a×2 键×双页 4 行（`#0000FF` 色值大小写）+ F1' fontsize(4)×1×双页 2 行 + X37 2 行（subscript+formatblock qS/qV）——与 F7m 终态残键集恒一，行级 9→5F。N 族零残。

- **v1.1 尾批终态（ALL-DONE 18:40:38）**：enginelib 补判双门过（failed=7 + 名册 7-red 逐名恒一，不重跑）；battery 七腿全绿（420/880/1104/552/3456/2607/1455，F=0 T=0，与 F7m 批逐腿同数）；二进制特征串 5/5 命中（`__zw_ce_f6b_post_insert`/`__zw_f6b_walk_px`/`POST-in 单壳树`/`双亲被 innerHTML 序列化两次`/`__zw_ce_dissolve_emptied_div_tail`）。**门禁批 v1（前段腿）+ v1.1（尾批）全链 DC-4 红线收口。**

## 门盲点补记（承 F6b 三臂刀硬化线）

v1 enginelib ABORT 根因＝failed 总数门正则写反（cargo `test result` 行格式为 `N failed;`，提取模式误写 `failed: N` 得 0）——腿完整跑完而判读层 ABORT。修正后双门（`[0-9]* failed` 提取 + 全量名册）在 v1.1 尾批验证通过。腿日志落盘先于判读，判读缺陷可从既有日志补判不重跑（判读绑定发射锚字节，v1.2b 先例同构）。

## 工程记账

- 批先入库协议第二次实践：刀体先 commit（8ac40127b）再发射，EXPECTED_HEAD 锚定；本刀 commit 由 lane8 支援位代落（本施工位验证链在飞期间），guard 追扫 PASS（2 文件/49 行/0 发现）补记合规。
- part01.js 12448 行（存量超限沿 ④/⑦ 记账）；本刀 +6 行净增。
- push 时 pull --rebase 并入 origin s366/s5477/R5477 归档三 commit → 刀体重放 8ac40127b→dcea9b117、docs 82bcb2916→6da883235（F7m d32500215 先例同款字节对账，当日补记）：刀体 hunks fix^..fix 隔离逐位恒一（f276661d），docs 重放 diff 恒一（1dc05820），八点锚七点恒一（part01 45f946ea/fixture 8a3f16e2/part01c/d/f/h/bridge f7d65f32）；漂移一点 part06 2fdb996f→247d1370＝并入 s366（869dc3f71）SVG write 注入域 +124 行 hunk，非刀体域。批判读绑定批时 head（8ac40127b，part06=s365 基线 2fdb996f）域内有效；s366 自带本域门禁（143 F→P）。
