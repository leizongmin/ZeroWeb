# M2 切片 407 — 静态 embed 链 + fallback 子内容激活（parser 扫描扩 EMBED + 祖先双门 + rendered 门）

日期：2026-11-12（承 s406 下一步：the-embed-element 静态链 6F 散单面）
commit：见 git log s407 提交

## referent 与勘根

driving 六键（the-embed-element 5 + the-object-element twin 1，s406 收口时
在册 F）：

- `embed-document.html :: Test document type embedding`
- `embed-document-under-content-visibility-{focus,gbcr}.html :: ensure onload happens`
- `embed-in-object-fallback.html`（"We have the right embed element" +
  "Check that the right things loaded" 两子测）
- `embed-in-object-fallback-2.html :: Test <embed> nesting inside <object>`
- `the-object-element/object-in-object-fallback-2.html`（object twin）

三层勘根（桥测探针逐层定位）：

1. **静态 embed 无人启航**（s405 留档 FIXME(embed-static-events)）：parser 静态
   `<embed src>` 只有动态插入钩子（`_zwStartConnectedEmbed` append 链）接线，
   静态内容零启航 → embed-iframe.html 子文档永不物化 → `parent.childLoaded`
   恒 false → embed-document 族断言永假。
2. **object 平扫误启 loaded 祖先内子内容**：s403 `__zwStartParserObjects` 对
   全部带 data 的 object 平扫——object-in-object-fallback-2 的 loaded 祖先内
   `should-not-load` 子 object 被误启 → 子文档脚本在活测试上
   `assert_unreached` → 文件 F。spec：object 渲染成功时 fallback 子内容惰性
   （#the-object-element 处理模型 fallback 内容不渲染步）。
3. **data: 不可渲染 essence 误映射**：s404 把非 text/*、非 image/* essence 落
   `image` 态 → 可渲染面假真，祖先门读不到「渲染失败」信号 → 不可渲染祖先的
   子 embed 该激活而不激活（fallback-2 驱动案第二臂）。spec：资源类型无法渲染
   → fallback 内容步（= 渲染失败，非 image 成功面）。

时序前提（本切片免改 load 门的关键）：`__zw_fetch('objnav:…')` host 桥同步
→ 物化 + 子脚本执行在一个 JS turn 内完成 → 静态扫描置于 part06 load 分支
（监听器前）即可让子旗标先于 onload 断言落地，R5275 门零改动。

## 施工

- **part04 `__zwStartParserObjects` 重写**（part04.js:1305 位）：
  - `_zwStaticAncGate407(sel)` 祖先链静态门（`_realTag` + `__zw_parent` 上行，
    64 步防环）：返 `'media'`（video/audio 祖先）/`'loaded'`（object 祖先
    entry ∈ {done,image}）/`''` 放行。键形态双查（QSA handle proxy 经
    `__zw_selector_for_handle` 反解，R4400 先例；sel proxy 直接走 sel 键）。
  - object 循环：带 data 即扫，门 truthy（media + loaded 双臂）跳过；同步直调
    `_zwLoadObjectResource(..., fireEvents=false, isEmbed=false)`（原
    `_zwStartConnectedObject` defer 异步会把启动序打散——祖先门须读先行元素
    终态；load 派发位无 mutation 风暴，同步安全）。
  - 新 EMBED 循环：带 src 即扫 + 同一祖先门 + **being rendered 门**（复用
    R3074 `_zwCheckVisibility`——display:none 于自身或任一祖先不启航；
    content-visibility 不在判面，cv 双 driving 案不受影响）；同步直调
    `(..., false, true)`。
- **part01 data: 臂**：`_objDpEss` essence `/^(text\/|image\/)/i` → `image`，
  其余 → 新 `'fallback'` 态（text/plain 与 SVG data: 口径保持——allowed-schemas
  + document-getters 在册 Pass 零位移）。
- **part01 `_zwFireObjectResultEvent`**：gate 加 `fallback`；事件映射
  `error|fallback → object? error : load`（embed 恒 load——失败终态映射语义，
  s405 同款）。
- **语料资产**：`embed-in-object-fallback-subdocument.html`（75B，md5
  63ccc3fdd…，pin 315976933870b34d6ea30e3f6643403edae678ba）——子文档按 query
  参数置父旗标；缺口机制 = 语料 the-embed-element/ 目录缺该文件 404 → load 派
  但标志恒 false（s227/s404 语料缺口同族）。挂账
  scripts/update-wpt-data.sh + goal 25 脚本 `fetch_raw`。
- **常驻钉**：`js_dom_bridge_tests/part107_s407_embed_static_fallback.rs` 三钉
  ——①静态 embed 启航链 + media/loaded 门 + 静默面；②fallback 激活矩阵
  （无 data / 不可渲染 essence / 渲染成功祖先、嵌套两层）+ 不可渲染 essence
  动态派 error + text/plain parity；③双红回归面（rendered 门 display:none +
  object 版 media 门，见过程陷阱）。

## 过程陷阱

1. **首轮终局 join 抓到双红**（join 双跑一致 P→F，非 flake）：
   - `embed-represent-nothing-04`：display:none 内 `<embed src=embed-iframe.html>`
     ——spec never-rendered embed 不加载；首版无 rendered 门无条件启航 → 子
     文档 `parent.childLoaded = true` → `assert_false` 爆红。修：embed 循环加
     `_zwCheckVisibility` 门（钉③）。
   - `object-ignored-in-media-element`：旧 defer 启航时子文档
     `nestingTest.step(assert_unreached)` 在 onload 完成后跑、被 harness
     step-after-done 吞掉假绿；改同步启航后子脚本落活测试真爆。修：object 门
     加 media 臂（spec 本义即不加载，media 内 object 被忽略）——比时序回退更
     spec 忠实，且消灭该文件同步物化 hazards。两案 A/B 均为门语义缺口而非
     时序本质（时序只揭示了旧假绿）。
2. **预修 sweep-1 select-event.html 82 键行静默丢失**（password 段 45 + url 段
   37，同文件后续段与后继文件照跑）——s406 在案 select-event T 塌缩同文件
   家族的新形态（整段行丢失 vs 整文件 T），单列非回归。
3. **`LC_ALL=C` 只给了管道首命令**（s287/s292 在案坑重踩）：`LC_ALL=C grep …
   | sort` 中 sort 没吃到 → comm 假报「文件未排序」。一刀切 export 后复跑。
4. **webview lib 复跑 848P/3F**：第 3 红
   `service_worker_runtime::navigator_update_activates_replacement_…`——全量
   63s 档并发时序翻转；单测隔离复跑 Pass + 全量复跑 849P/2F 在册对逐名相等
   → flake 单列。
5. **`cargo clippy … -D warnings` 直挂报 Usage: cargo check**——本机工具链须
   `-- -D warnings` 分离形。

## 验证（/tmp/s407/ 全量日志）

- 桥测 part107 三钉全绿（exit 0；3894 filtered 面 0F）。
- 单元：engine lib **3884P/10F**（预修 3883P/10F +1 钉；失败清单逐名 = s406
  在册 9 + r307 HEAD 既有红单列——stash A/B 承 s406 定谳）；webview lib
  **849P/2F**（image_decoder proxy / csp_gate 在册对）。
- driving solo（门修复后重编两轮）：the-embed-element 全目录 8 关键面全 Pass
  （6 driving + represent-nothing-04 复绿 + ignored-in-media 保护）；
  the-object-element 目录 object-ignored 复绿 + in-object-fallback-2 保持 +
  constraint-validation 在册同红；`object-in-display-none-load-event` Pass
  （佐证 object 不挂 rendered 门——spec object 不渲染仍加载）。
- 全树 sweep ×2（runner 重编含 WIP，二进制 token `_zwStaticAncGate407` 命中 3
  处钉证新鲜度）：run1 = run2 = **8484 keys 8242P/216F/26T** 完全一致；
  **join-cross state-diff=0**（同二进制零翻转，s292 协议）。
- join（对 s406-run1/run2）：**恰 6 STATE-DIFF 全为 driving F→Pass，双跑一致；
  真 new=0**（join-2 的 16 NEW 全为 select-event flake 家族复现键——s406-run2
  塌缩丢键在我跑全数复得）。
- 另 2 键 F→Timeout（customizable-select select-home-end-pagedown-pageup{,-detailed}
  .optional.html）：文件零 embed/object 引用（码路径零交集）+ stash A/B HEAD
  二进制 solo 双跑亦 Timeout + s406-run2 同 T——既有 watchdog 边界翻转族，
  单列非回归（s406 select-event/focus 塌缩族同款）。
- fmt：本切片文件 `rustfmt --check` 干净；`js_dom_bridge/callbacks.rs` HEAD
  既有 fmt 脏（stash A/B 钉证，并行 CSSOM 流 0ff1bc92c 落树负债，单列不代修）。
  clippy `-p zero-engine --all-targets -- -D warnings` 干净。
- smoke：`make product-smoke` + `make product-smoke-legacy` 双 PASS
  （struct-check 0 issues）。

## 口径

- 本流域在册红：**221F+24T → 215F+24T**（−6F = 六 driving 键 F→P 双跑一致）。
- sweep 总数：8484 keys **8242P/216F/26T**（双跑全等）。单列池：+2
  customizable-select F↔T 翻转（不进在册两池）、select-event 塌缩族跨跑复现、
  callbacks.rs fmt HEAD 脏、engine r307 HEAD 红。
- engine lib 单测在册红 **9F + r307 HEAD 单列**（承 s406 口径不变）。

## 残差与边界

- 静态 embed/object 仍不派终态元素事件（`fireEvents=false`——FIXME(object-
  static-events) 族 spec 偏差记档，钉①静默面保持）；静态 embed 失败终态
  映射 load 派语义静默面不变。
- frames 索引面（`frames[i]`/`frames.length`）iframe-only 不对称承 s406 记档。
- rendered 门只判 display（R3074 判面）——content-visibility 祖先不在判面
  （cv 双 driving 案依赖此豁免；cv:hidden 内 embed spec 面记 FIXME 观察）。
- object 循环无 rendered 门（spec：object 不渲染仍加载，
  object-in-display-none-load-event 在册 Pass 佐证）。
- 自建 probe 恒红案族（s343 口径）继续单列，不入在册两池。

## 下一步

散单续勘（下轮候选，按池体检视）：the-style-element 10F 未归因面 /
links 族 4F / the-input-element 散单；select/combobox（簇 D）与
form-submission-target（M3 门控）仍待用户决策，不动。
