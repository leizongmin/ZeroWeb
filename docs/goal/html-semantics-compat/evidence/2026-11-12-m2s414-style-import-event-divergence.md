# M2 切片 414 — the-style-element 余红勘根定谳：动态 style @import 事件臂 = 引擎分歧非缺口（勘根不施工收口）+ 语料 css-red.txt 补拉

日期：2026-11-12 夜–11-13 凌晨（承 s413 下一步：style-error-01 1T @import fetch 面 / constructed-stylesheets 1T 同族候选）
commit：fix 载体 bd41f9bd2（FIXME 注记 + goal 脚本 ③j）；本账本随 docs commit

## referent 与勘根

driving 面：`document-metadata/the-style-element/style-error-01.html` 文件级 Timeout（慢性在册，s161 起在册 T 键；s413 收口后 the-style-element 目录唯一余红）。该文件为**动态创建** style（createElement + appendChild，非静态扫描臂覆盖面）+ `@import "support/css-red.txt"` 断言 error 事件。勘根问题：动态 style 的 @import 取回/事件臂是否值得施工。

四点定谳证据：

1. **上游语义面本身分歧（wpt-metadata 实证，2026-11-13 live 复核）**：上游 `wpt-metadata/html/semantics/document-metadata/the-style-element/META.yml` 唯一条目 = product chrome + `https://bugs.chromium.org/p/chromium/issues/detail?id=1232491`，results 覆盖 `style-error-01.html`——error 语义面无全绿引擎；语料源文件 t404 子测上游亦整段注释弃用（corpus 实读确认）。同目录 `style-load-after-mutate.html` **零元数据条目**（全引擎绿；本仓 s162 driving case，在册 Pass）。
2. **两案互斥钉死**：style-load-after-mutate 的绿 = 动态 style 文本变更后 `await style.onload` 收 load（load 语义）；style-error-01 要的是动态 @import 失败收 error（error 语义）。probe3b 注入实证 error 动态臂可施工（d_err=1，`_zwSsVisitCss` 链复用）但必翻转后者 load 面——单臂承载两案语义不可兼得，弃实现。静态扫描臂 404 error 语义（style_events 2P 在册绿，s164 搭车兑现）不受影响。
3. **probe 叙事**（/tmp/s414/probe3.log；probe1/2 = /tmp/s414_probe{1,2}.log）：probe1（zz-s414-probe-style-import）page fetch asset Pass + dynamic style import events Pass + 文件级 Timeout（1 子测 pending——load await 永挂形）；probe2 收敛探针面同态；probe3（zz-s414-probe3）probe3b 注入臂 rep=`{"text":"@import \"support/css-red.txt\";","sel":null,"handle":"__n1","getText":"","imports":"no-fn","key":"@__n1","importFired":"no-table","loadFired":"no-table"}` + d_err=1/d_load=0/s_err=1/s_load=0。探针文件验后已删（sweep join new=0 交叉证实 corpus 干净）。
4. **语料资产缺口（s227 先验判据同型）**：style-error-01 引用 `support/css-red.txt`（上游 200 text/plain——error 事件 MIME 失败路径真形态），本仓 support/ 空目录 = `fetch_dir_html` .txt opt-in 固化坑（③b css-module 同坑）。缺席时 import 落 404 网络错误路径，与 MIME 失败两条 error 路径不可区分。修复：goal 脚本新增 ③j `fetch_raw` 行（pin 315976933 实拉落盘 21B `html { color: red; }`），**补语料后 style-error-01 solo 复验仍文件级 Timeout**（动态臂未建模，零行为变化与预期吻合）；未来若实现动态臂，以本资产为 MIME 真形态基准。

附随实测（probe3b rep）：`__zw_get_text_handle` 对 script-created 元素读回空串（文本未落 host doc 回放表），shim 代理 `.textContent` 才是 turn 内正确文本源——已记入 FIXME 注记备后续动态臂车道引用。

## 施工（勘根不施工）

零功能代码变更。交付物三件：

- **engine**（`js_dom_shim/part03.js`）：style load fallthrough 臂前插 FIXME(compat) 注记——分歧定谳、互斥证据链（crbug 1232491 / 零条目全绿）、弃实现理由、`_zwSsVisitCss` 复用可行性、textContent 读回空串实测、spec 链接（html.spec update-a-style-block）。
- **语料**（`tests/wpt-runner/scripts/goals/25-html-semantics-compat.sh`）：③j 块 + `fetch_raw css-red.txt`（基准变更注记：补语料后零位移预期）。
- **corpus**（gitignored 本地数据）：css-red.txt 实拉落盘。

## 过程事件（跨 session 断点 + 守卫竞态）

1. **429 断点接手**：前 session 23:27 触达 5h 配额上限中断于收尾段（全部计算腿已落 /tmp/s414，账本/commit 未写）；本 session 从工件目录重建画像（final.sh freshness 门 bin>shim + 二进制内 FIXME 标记 grep ×1 + probe3b ×2 确认 sweep 二进制含终稿注释），计算证据全部采纳重算（pairs/join 从原始日志现算，未转抄上轮 draft）。
2. **target-disk-guard × 活动构建竞态（新坑）**：本 session solo 复验触发 target/ 76GB>50GB 守卫 → 一级删 incremental（7GB）后仍超 → 二级全量 `rm -rf target` 与**仍在编译的孤儿 maketest 树**（前 session 429 后 chain.sh 存活遗留）竞态——rm 首轮报「目录非空」失败致 solo 入口退出，孤儿构建因 incremental 中途被删假红（`css_vars_render_path_parity` 编译错 = 缓存删除伪影非代码错）。处置：按 cwd 归属核实后杀孤儿树（35 进程，ZeroWeb-2/3 并行流未误伤）→ 守卫二级全量删完（target 632K）→ `make test` 全量重建重跑。教训：**多重 make 入口并行前先 `du -sh target`**——守卫触发即全量删时活动构建必被打断；守卫无写入者检测（自身文档已警示 partial 清理破坏 cargo 一致性，本例为反向形态：活动写入者使「全量删」变 partial）。

## 验证（/tmp/s414/ 全量日志，终稿代码）

- driving solo（leg1-solo.log，23:08，the-style-element 全目录）：**49 子测 Pass + 1 Timeout**——tentative/ 11 文件 37 子测全 Pass 保持、style-load-after-mutate/style_events/style_load_event 等 s162 面全保持；余 1T = style-error-01（本切片勘根对象，定谳在册）。补语料后 solo 复验（11-13）同态 1T。
- sweep ×2（RAF_FRAME_DRIVEN，test-guard 包裹，23:12–23:19）：**双跑全等 8274P/214F/16T**（pairs 逐键现算），join-cross stable=8504 state-diff=0 new=0；NotRun 5 不变（css-module/content-type 4 + json-module 1，s413 同清单）。
- join-b1/join-b2（对 s413 终稿批 pairs）：**恰 2 state-diff、new=0/gone=0**——`the-select-element/inserted-or-removed.html :: The last selected OPTION should win; Inserted by jQuery append()` **Pass→Fail** + `customizable-select/select-home-end-pagedown-pageup-detailed.optional.html` 文件键 **Timeout→Fail**（红集不变形迁移，见口径）。
- **ior 静机定谳（flake4，负载 10 安静窗，重建 release runner〔freshness bin>shim + FIXME 标记 grep ×1，rebase 后 bd41f9bd2 树〕）**：jQuery append() **3/3 Fail**——非负载形。全时序：s413 批（20:04）Pass ×2 → s414 批（23:12，0079443c6+注释同码二进制）Fail ×2 → flake3（负载窗）Fail ×3 → flake4（静机 + 重建二进制）Fail ×3。**11 天稳定 Pass（s236 修复起数十轮 sweep）→ 确定性 Fail，零功能 delta 跨两代二进制持续**——非本切片可归因（本切片施工面=style 域注释+语料资产，leg1 style 目录零位移实证），勘根挂账下轮首位（s235 三跑归因：s413-exact 树重建二进制 → ior ×N 复现 P 则 bisect 上游五 commit；候选根因假设 = `_zwOptStateKeyOf` plain stamp 分配序敏感〔s236 状态键机制，二进制布局变化翻转碰撞行为〕）。
- **home-end-detailed 静机定谳（f4-she ×2 静机）**：run1 8 子测全 F 文件跑完（展开形）↔ run2 2F+2 unreported / 4 pending 双文件 file-T（塌缩形）——**连跑双向翻转**，s238/s239「塌缩↔展开形」漂移族实证；子测全在册红（customizable-select 177F 挂起域），T→F 为文件键形迁移红集不变。
- `make test` 全量（maketest2.log，rebase 后 bd41f9bd2 终稿树全量重建——target-disk-guard 竞态全清后冷启动）：**154 腿 153 绿 + 仅 engine lib 在册红腿**（3910P/**7F**，失败清单与 s413 maketest 逐名 diff **IDENTICAL** 零位移〔img×2/fetch+stream×2/websocket×2/iframe-service-worker×1〕；+1P = 3910 vs 3909 归并行流合入域 `part11-color.js` +13 编辑流 B1 终绑批；exit 101 → make exit 2 同 s412/s413 收口形态）。compile-first 全量冷编译 ~70m（load 200+ 与 ZeroWeb-2/3 并行流互相拖长，单列非本流负载）。
- fmt/clippy：make test fmt 门禁前置过 + clippy 腿（13 crate `-D warnings` 全 strict，零 `-A`）零 error——css-parser random.rs 跨流级红已被渲染流上游清偿，本切片无 Rust 变更。
- webview battery（webview-lib.log）：**853P/1F**——1F = csp_connect_gate_blocks_fetch_sh1_m2s5 在册；image_decoder 本批绿 = s413 已账 favorable flake 形（guard 批回在册对语义，本批独立单跑绿不改变在册对口径）；852→853 与 engine +1P 同源归并行流合入域。
- driving solo re-pin（leg1-final.log，rebase 后终稿树 + css-red.txt 补后）：the-style-element **62P+1T 与补语料前 leg1 逐行 diff 空**（Pass 集全等）——补语料零位移预期实证；余 1T = style-error-01 在册。

## 口径

- 本流域在册红：s413 收口 **212F+17T → 213F+16T**（+1F 真新红 = ior jQuery append()〔勘根挂账下轮首位〕；余 +1F−1T = home-end-detailed 文件键 T→F 形迁移〔展开形回摆，红集不变，子测全在册 177F 挂起域，flake4 静机连跑双向翻转实证〕）；NotRun 5 不变。measured sweep 总量 = 8274P/214F/16T（两跑全等）。
- tentative/ 口径不变：11 文件 37 子测全 Pass、0F/0T/0NotRun。
- the-style-element 目录口径：62P + 1T（style-error-01，勘根定谳在册非缺口；补语料前后逐行全等）。

## 残差与边界

- **动态 style @import 事件臂**：定谳不施工（引擎分歧非缺口，互斥钉死）。FIXME 注记已锚定未来若上游语义收敛（crbug 1232491 关闭）或语料出现单案面时的再勘根入口。
- **`__zw_get_text_handle` script-created 空串**：已知读回空串（回放表未落 host doc），shim 代理 `.textContent` 可用——零案面挂账不施工。
- **ior jQuery append() 新红（下轮首位勘根候选）**：见验证节静机定谳——跨两代二进制（0079443c6+注释 与 bd41f9bd2 含上游五 commit）确定性 Fail，s413 批前 11 天稳定 Pass。按 s235 三跑归因协议勘根：s413-exact 树重建 → ior ×N → P 复现则 bisect；F 持续则勘根 `_zwOptStateKeyOf`/`_mo_notify` 时序域（s236 钉面潜在分配序敏感）。

## 下一步

**ior jQuery append() 新红勘根（三跑归因协议，升首位）**；constructed-stylesheets 1T 勘根候选（s412/s413 连提）；挂账池承 s404/s405/s411/s412/s413（外链模块基座 map 键错配/`.sheet` kind 固定面不全/标记属性外泄/多进程 parity/forward map 空返勘根/渲染流 5 面/form-submission-target 6T/sizes-auto 半边/ilf+modulepreload 观察池/适用态打满 reject 面/maxLength-minLength IDL 不适用态反射/unhandled-rejection 换算）；checkable 父 label 2F 渲染域移交承 s410；select/combobox（簇 D）与 A2+D 簇立项征询仍待用户决策。
