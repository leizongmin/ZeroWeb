# 刀㊺ 后定位评估——TT 散布余 12F 逐页 census 归簇＋module 子系统切片 16F 立项评估

日期：2026-11-27 · 权威日志：`/tmp/s10k45/k45-tt-post.log`（刀㊺ 电池 post 腿，2233P/34F/19T，树态＝HEAD＋刀㊺ 两文件＋兄弟席 part02 在飞字节；判读行提取 LC_ALL=C awk＋uniq -c 对账）· 本文档为分析产物（非判读），刀位选择与排期归后续轮。

## §1 散布 12F census（真实池口径，逐面定谳）

套件 F 34＝**module 16**＋**散布 12**＋**旧探针页 6**（单列）。T 19 另列。兄弟席 navigate 五面（001/002/003/004/008）在其落库前仍占池 5F（33F 口径）；其落库后池 28F＝module 16＋散布 12。T 19 构成：navigate 域 3（005/006/csp-headers＝真导航面，兄弟席后继位）＋trusted-types-navigation 八切片 8＋creation-001/002/003 3＋sink-mismatch-001/source-file-path/set-event-handlers/HTMLScriptElement-internal-slot/getAttributeType-event-handler 各 1。

### 归簇（四簇）

| 簇 | 面 | 机制 |
|---|---|---|
| **S1 eval 违规报告边（3F）** | reporting-check-report-Window-sink-mismatch「Passing a plain string to eval throws」／reporting-for-Window-eval「No violation reported for eval with TrustedScript」／eval-reporting-report-only「evaluating a string」 | 刀㊶ eval 包装的**报告边**精化：enforce 派违规＋RO 面派 report＋**合法 TrustedScript eval 不派**（负向边）。三面同管线（eval 门违规派发/样本串），宜同刀 |
| **S2 execCommand sink 门（2F）** | block-Document-execCommand「execCommand(\"insertHTML\") throws」／reporting-for-Document-execCommand「plain string (insertHTML command)」 | execCommand 作为 TT sink **整体缺门**（spec TT sink 列表 insertHTML 等）：enforce throw＋violation 派发两面一机制，宜同刀 |
| **S3 CSP 违规报告 location（2F）** | should-…-creation-be-blocked-by-csp-005「Location of trusted-types violations」／should-sink-type-mismatch-…-csp-003「Location of required-trusted-types-for violations」 | violation report 的 line/column/ url 定位字段保真；同页 001/002（creation）与 001（sink-mismatch）现 Timeout——**先解页级 T 再磨 location 面**（T/F 同页耦合） |
| **S4 杂项单面（5F）** | legacy-trusted-scripts「Legacy sinks accept arbitrary strings」／setAttributeNS「accepts null for non-specced accessor」／getAttributeType-namespace（页级）／SVGScriptElement-internal-slot「manipulating script during loading」／DOMParser-parseFromString-regression | 五面互不同根因，逐面一行为补；SVGScript-internal-slot 与 HTMLScriptElement-internal-slot（T，已挂 solo 悬挂族）同族——**开工前先解 T 耦合** |

### 刀位提示（非指令）

- S1/S2 的 sink 面（eval/execCommand）开工前先 grep 定位现门文件域——与兄弟席在飞 part02 刀面（javascript: 预导航）相邻的执行链文件可能重叠，开工前对表 [s10k45-coord.md](../../../.acceptance/dps/s10k45-coord.md) 席位。
- S3/S4 各面文件域分散（part12 factory/getAttributeType、part04 SVGScript/script 载入、DOMParser sink），适合作「杂项拼盘刀」一轮多面。
- 面字符串全表见本节上表＋权威日志 grep（`^Fail trusted-types/` 非 zz 非 module 页）。

## §2 module 子系统切片 16F 立项评估

**构成**（master 卡点行逐面对账一致）：script-enforcement-006/007/008/009 各 3F（module 执行/importmap/动态 import() 三缺切片位）＋block-text-node-insertion-into-(svg-)?script-element module 臂 2F＋script-enforcement-010/011 module 臂 2F。

**机制盘点（代码事实，2026-11-27 现行树）**：

- webview 宿主侧桥**已存在**：`crates/webview/src/webview.rs:5088`（s250 `__zw_compile_inline_module` 动态内联 module prepare，五态契约：取回失败/specifier 解析/属性校验/链接失败/成功 IIFE）＋`__zw_compile_module` 运行时取回链（part01.js:16834 shim 消费面、16816-16870 module 记录/depsPromise/导出 getter 面完整）。
- runner 侧有**转换式** module 执行：`compile_module_script` import/export 转换（runner/mod.rs:346 R3083）＋内联投递体静态 import spec 重写（testharness.rs:11751 s101，动态 import() 明注「不在本重写面（module 尾片挂账）」）＋module 取回闭包含 credentials 语义（testharness.rs:6595 s338）。
- importmap：runner 仅 pinned 上游子集独立跑（testharness.rs:4094/4169 经 script-sandbox `import_map.rs`）——**未接入页脚本 prepare 管线**。
- script-sandbox 的 es_module/import_map 服务独立沙箱（worker/esm 用例），非 DOM 页管线（master 定谳不变）。

**缺口清单（runner 页脚本管线四线）**：

1. **module 执行路径未过 TT script 门**——classic 内联/外链面刀㊷ 已落 TT 门，转换式 module 路径绕过门（006-009 的 require-trusted-types-for enforcement 在 module 内失灵）。**语义要点**：门必须作用于**原始脚本体**还是转换后体须先定——default policy 入参与 violation 样本串对文本敏感，转换改文本身（gate-before-transpile vs gate-after 决定 R1 形状，开工前先做该定谳）。
2. **`__zw_compile_inline_module` runner 挂钩域未注册**（仅 webview.rs）——shim 动态插入 type=module 内联脚本在 runner 死链。
3. **动态 import() 运行时链 `__zw_compile_module` runner 未注册**（11755 既有挂账）。
4. **importmap registry 未入页脚本 prepare**（解析语义在 sandbox 侧已有，缺接线＋页级 fallback 失败语义）。

**切片建议（2-3 刀位，各随电池 A/B 锚定）**：

- **R1（~8-10F）**：module 执行路径 TT 门（1＋block-text-insertion module 臂；前置完成 gate 位置定谳小演）。006/007 的 enforcement 臂主菜。
- **R2（~4F）**：runner 注册 `__zw_compile_module`＋`__zw_compile_inline_module`（2＋3；fetcher 管线 s338 闭包已有可复用）。008/009 动态 import/内联臂。
- **R3（~2-4F）**：importmap registry 入 prepare（4）＋010/011 module 臂（default policy 于 module/类型切换面）。若 R1 定谳显示门位需迁移，R3 顺带收口。

**风险**：V8/QuickJS 双通道 feature gate（runner 主回归走 V8Sandbox——挂钩注册须双后端落点或显式划界，见 [[engine-default-v8-runner-channel]] 同款纪律）；转换式执行与 TT 门的文本保真冲突（上述语义要点）；internal-slot 族页级 T 耦合（solo 悬挂族在案，module 臂开工前先中性化验证）。

**结论**：维持 master 定谳——**宜单独立项排期非续刀位**（四线跨 runner 宿主/引擎门/importmap 三域，非单文件续磨形态）。R1 前置的 gate 位置定谳演（~半小时级）可作独立小轮先行。
