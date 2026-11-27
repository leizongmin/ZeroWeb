# 刀㊽ 判读（s10k49）——module 子系统 R1-b：importmap 动态注册链（TT 语料 +8P/−8F，importmap 面八连收，真实池 21→13F）

日期：2026-11-27 · 域：trusted-types importmap 子系统（census bb1d0aea0 §1 R1 簇的注册面半区）· 树锚：三文件刀体 patch md5 `70d4b7b96424e7ef55d3074e5c6e510b`（18,449 B，发射 TREE-MD5-OK＋REAPPLY-MD5-OK 三文件断言；patch 常驻 `/tmp/s10k49/knife49.patch`）· 文件域＝`part03.js`（60+/1−）＋`webview.rs`（67+/11−）＋`import_map.rs`（56+/0−），共 171+/12−。

## 目标簇（importmap 面，8F）

| 页 | 面（×2：importmap 单 map＋two-importmaps 双 map） | 根因链 |
|---|---|---|
| script-enforcement-006 | `Untrusted HTMLScriptElement of importmap type uses the source text returned by the default policy.` ＋ two-importmaps 同面 | pre-gate classify 判 'importmap' 走 'blocked' 提前返：政策未调（got 恰 `[UNMAPPED]` 零 CREATE_SCRIPT）＋注册链三缺（宿主回调／运行时槽／决议接入） |
| script-enforcement-007 | `Untrusted SVGScriptElement of importmap type …` ＋ two-importmaps 同面 | 同上（SVG script 走同一 s109 prepare 门链） |
| script-enforcement-008.https | 同面＋inline CSP check 变体 | 同上（CSP 门以变换后文本判，允许后无臂可达） |
| script-enforcement-009.https | 同面（SVG＋inline CSP；2 importmap 变体） | 同上 |

## 根因定谳（三链）

- **门链定位**：s109 pre-gate classify 把 importmap 型归 'blocked' 提前返。该三态返回值是 6 个 trap 臂的共享契约（'blocked'→skip），不可改返回值——刀面改为 part03 局部 essence helper `_zwIsImportmapType109`（type 属性剥离 WC 白名单空白后 ASCII case-insensitive 比对 `importmap`）＋ pre-gate（~5626）/post-gate（~5755）两处旁路：`blocked && !importmap` 才返。trap 臂语义零变化，classify 契约不动。
- **动态可达性**：part01:9084-9088 childList mutation sink 对 SCRIPT 元素无条件委派 `_zwScriptMaybeStartDynamic`（appendChild 同步期，刀㊼ probe3 面包屑实证）——importmap 型动态插入可达注册臂；part04 R387 9493/9495/9499 臂对 blocked-typ 跳过（零干扰）。
- **回调名陷阱（solo 首跑假阴定谳）**：首版宿主注册 `"__zw_register_importmap"` 而 shim 检查 `typeof globalThis.__zwRegisterImportmap`——register_callback 首参＝暴露 JS 全局名，必须与 shim 逐字一致；silent no-op 使门链全通而注册静默（got 出现 CREATE_SCRIPT＋sink 名但 importmap 面仍红）。改 camelCase 后 solo 全绿。

## 修复（三文件）

1. **part03.js**：两处 gate 旁路＋注册臂（CSP 门后、s250 module 臂前）：run-once 戳（`_zwScriptStarted`/`_zwRanScripts`）→ `globalThis.__zwRegisterImportmap(String(text))`；成功回 "1"，抛错或非 "1" → setTimeout(0) 派发 error 事件（`__zwMakeTrustedEvent` 可用时，任务队列序不破当前同步链）；回调缺席＝静默 no-op（旧宿主零路径变化）。每 script 元素注册一次（two-importmaps 面＝两元素依序两次 merge）。
2. **webview.rs**：共享运行时槽 `runtime_import_map: Arc<Mutex<ImportMap>>`（静态 `page_import_map` 播种）；新宿主回调 `__zwRegisterImportmap`（`ImportMap::parse_from_page`，base＝`current_url`；Err → tracing::warn! 返空串）；三消费面接运行时槽快照——`__zw_importmap_resolve`（import.meta.resolve）、`__zw_compile_module` shim（cfg v8 / not(v8) 双臂）、V8 原生动态 import resolver（`apply_import_map` 后 parent Join 校验不变）；`registry.set_import_map`（静态模块图收集域）保持刀前快照＋FIXME（预扫域无动态 merge 语义，动态运行面已全接）。
3. **import_map.rs**：`merge()`——imports 同键覆写／新键追加，scopes 前缀合并（内层同键覆写），SpecifierMap code-unit 降序不变式两端保持；测试 `merge_disjoint_keys_and_override_s10k49`（异键双命中＋同键覆写＋降序 windows 断言）。

spec：https://html.spec.whatwg.org/multipage/webappapis.html#registering-an-import-map （multi-map：后注册覆写同键，依 script 序 merge）；旁路判定：https://html.spec.whatwg.org/multipage/scripting.html#prepare-the-script-element （type 状态机 importmap 分支）。

## solo 页级（回调名修正后，电池前先行）

006 4P/0F（classic/importmap/module/two-importmaps 四面全绿）、007 4P/0F、008.https 5P/0F、009.https 5P/0F——四页 rc 全 0（日志 `/tmp/s10k49/solo-006b.log` 等）。

## 电池（2026-11-26 23:33 → 11-27 00:56 UTC；三 ABORT 皆漂移门按设计拦截，第四发 CLEAN）

四 corpus 腿（23:33:19→23:48:25；san/dp 秒回＝写者席探针 make 预编译刀体，指纹新鲜二进制即当前树字节）vs k48 腿日志：

| 腿 | 计数 | 判 |
|---|---|---|
| sanitizer-api | 713P/65F | F/T SET-IDENTICAL（零漂移） |
| domparsing | 1314P/222F | SET-IDENTICAL（零漂移） |
| web-components | 17485P/940F | F/T SET-IDENTICAL；P −172 恰 gethtml.html 单页（3396→3224，已知巨页截断抖动族，页集恒等） |
| dom | 57432P/596F(T含) | F 真实新＝0、GONE＝0；+222 分解＝114 t48-trace（写者 testharness 插桩，家族归因自移除）＋108 Timeout（并发争抢慢化族）；P −81 恰 Range-mutations 双页（918→877／895→855，已知噪声族，−41−40=−81 闭合） |

TT A/B 三 ABORT 挂账（对席 burst 窗占）：23:48 ABORT-SIB-DRIFT-05（写者 dom 腿窗中保存 part05）、00:08/00:17 两发 ABORT-MID-DRIFT-04（写者连续编辑 part04，mid-gate 拦截弃 post 腿保 A/B 纯度）。第四发窗＝写席轮界静默（runner 退出＋sibling md5 静止 60s 确认，三 sibling 锚全空 diff＝最净基线），00:38:03→00:56:26 双腿 18m23s 无 ABORT 无 NOTE：

| 腿 | 计数 | 判 |
|---|---|---|
| TT base（scoped checkout＝恰我方三文件回 HEAD） | 2245P/41FT/19T | vs k48-tt-post：GONE 恰刀㊽b 两面（setAttributeNS null＋legacy）＝上游已收编注记（2a57ea242），NEW=0 |
| TT post（reapply 三文件 md5 断言过） | **2253P/33FT/19T** | **post − base＝本刀效应 +8P/−8F**：GONE 恰 importmap 面八条（多重集各 1）；NEW=0；T 恒（19）；ΔP=−ΔF 闭合 |

判读行提取 LC_ALL=C awk＋wc 对账＋awk 多重集（/tmp/s10k45/mdiff.awk）；P 漂移页级 join 归因仅已知噪声族三页。rc=2 均为既有基线失败集（rc 只留痕）。

## 残余与边界（module 子系统 10F → 2F）

- **block-text-node-insertion-into-(svg-)script-element 2F 不在本刀**：postMessage 计数面竞速＝runner rAF-stub 架构挂账（同刀㊼ 边界：帧驱动 rAF 模式或主窗派发机制级改动，非 TT sink 门域）。
- **静态模块图收集域**：`registry.set_import_map` 保持静态快照＋FIXME——宿主桥预扫路径无动态 merge 语义；动态运行面（resolve/compile/native）已全接运行时槽。属宿主桥架构项，不涉 TT 语料。
- **注册失败传播**：spec 要求解析失败派发 error 事件且该 map 不生效（后续 script 照常）；本刀失败臂 setTimeout(0) 派发＋slot 不动已实现；`parse_from_page` 宿主侧 Err 走 warn! 返空（宿主日志域）。

## 归属与计账边界

- **本刀计账：TT 语料 +8P/−8F（importmap 面八连收），真实池 21→13F（套件 F 22→14：zz 探针 6＋散布 8 恒持），module 子系统 10→2F，累计 84,282→84,290（距 ≥99,947 差 15,657）**。基线含写者席刀㊽b（2a57ea242，+2P/−2F，84,280→84,282 已收编——本刀 TT base vs k48-tt-post 的恰 −2 为其字节效应注记）。
- part02 navigate 五面（+5P/−5F，池外计账）＝写者链字节（diff md5 `32985cd4` 全程保树不 checkout，coord 纪律），归其轮收编。
- 电池窗方法论沉淀：同 clone 双席并发下 checkout 型 A/B 需对席「轮界静默」而非 burst 间隙（三 ABORT 实证）；已录 memory `battery-ab-window-needs-seat-boundary-quiet`。

## 结论

刀㊽ 净增 **+8**（importmap 面八连收），TT 真实池 **21→13F**，Timeout 19 恒，**累计 84,282→84,290**。R1 簇（module 执行面＋注册面）16→2F，余 block-text-node 2F＝runner rAF-stub 挂账。池余：散布 8F（S1 直接 eval 间接化 1＋S3 2＋S4 5）＋navigate 5F（写者域）。下一刀位：散布簇 S1（trusted-types-reporting-for-Window-eval 直接 eval 间接化）单面；S3/S4 与写者域协商。
