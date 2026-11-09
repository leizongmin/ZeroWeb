# M2 s329：canvas 零尺寸 toDataURL/toBlob——4F→4P

日期：2026-11-07 ｜ 承接：s328（`2026-11-07-m2s328-td-visible-target-adoption.md`）

## 目标与定谳

六域 testharness 普查（s327 survey 只覆盖 reftest 全量 + forms/text-level/links，本轮补 grouping/document-metadata/tabular/interactive/scripting-1/embedded 六域）再生施工队列，embedded 残红中 canvas 零尺寸面首选：

- `toDataURL.zero{width,size,height}` 3F + `toBlob.null` 1F：HTML spec canvas 位图零像素（width/height 内容属性任一解析为 0）时 toDataURL 恒返 `'data:,'`、toBlob `callback(null)`（spec #dom-canvas-todataurl 步骤 2 / #dom-canvas-toblob result=null，均在 origin-clean 门**之后**判定）。shim 两臂缺失，零尺寸画布把 host 兜底位图（0 宽编码 clamp 成 1×N）当真 PNG 返回。
- 普查同时收口 s327 挂账「microdata 语料缺位 fetch 侧核对」：M1 记录 + fetch script 注释已答（upstream master 目录 404，本轮 GitHub API 复证）——microdata 空域定谳维持。
- 普查归域：credentials 簇（12F+2T，需 HTTP cookie jar + check-cookie.py 端点 + iframe）移交 runner-net-infra goal（M0 未启动）；tabular/document-metadata 残红 = 布局/CSP 面；img density faces = rendering stream（既有移交不变）；dialog 族 = interactive 全 residual，已移交 R5431。

## 施工（两文件四臂）

1. `crates/engine/src/js_dom_shim/part04.js`（proxy get-trap 臂，markup canvas 走此路径）：
   - toDataURL 臂（~1041）：origin-clean SecurityError 检查后、惰性建 ctx 前，`_zwCanvasDim` 即时读 width/height（属性即时读——`c.width=0` 动态置零同臂），任一 === 0 早退 `'data:,'`。
   - toBlob 臂（~1074）：Promise.resolve().then 回调内同位置早退 `null`。
2. `crates/engine/src/js_dom_shim/part05.js`（createElement 实例包装 own 属性面）：`el.toDataURL`（~8631）/`el.toBlob`（~8658）同款早退（读 `el.width`/`el.height` getter，同 `_zwParseSpecNonneg` 口径）。

### 调试：实例面遮蔽 trap（契约测试捕获）

首轮只修 part04，契约测试 arm④（createElement canvas 50×50 fillRect 后动态 `c.width=0` → toDataURL）返真 PNG（1×50）。勘根：`_zwMakeCanvas` 实例包装给 createElement 产物挂 own `el.toDataURL`/`el.toBlob`（part05:8631/8658），own 属性遮蔽 part04 proxy get-trap——markup canvas（qSA wrap，无实例 own 属性）走 trap 臂，createElement 产物走实例臂，双路径并存。纯 part04 修复只覆盖 markup 面。补 part05 两臂后全绿。零尺寸早退不覆盖 OffscreenCanvas.convertToBlob（独立面，本切片不动）。

## 验证

| 面 | 命令/方式 | before | after | 结论 |
|---|---|---|---|---|
| 契约 | `test_canvas_zero_dim_todataurl_datablob_s329`（js_dom_bridge_tests/part06.rs） | — | 4 臂全 P | markup width=0/height=0、toBlob null（microtask drain 后回调 null）、动态置零（setter 反射写内容属性 getAttribute('width')==='0' 前置断言） |
| WPT 全域 | `make testharness-html-semantics FILTER=the-canvas-element` | 普查 112P/7F/1T | **116P/3F/1T**（r1 修复后；r2 part05 补丁后同数同键集） | (key,state) 归一 diff 位移恰 4F→4P（zero{width,size,height}+toBlob.null），零意外位移 |
| WPT solo | `FILTER=toDataURL` / `FILTER=toBlob` | — | toDataURL 21P/0F；toBlob null→Pass（jpeg 独立面仍 F） | 修复面 solo 干净 |
| engine lib | `cargo test -p zero-engine --lib`（guard） | — | 3794P/6F | 6 红全既有：stash 法归因（part04/part05/part06 三文件 stash 后基线同 6 红）——img_runtime_deferred_settle_r5275、svg_img_settle_density_r5277、test_fetch_filtered_response_type_matrix、test_iframe_service_worker_messages_dispatch_to_iframe_container、test_t8k_websocket_error_nonterminal、test_t8k_websocket_event_flow_open_message_close，与 canvas 无涉 |

日志：`logs/s329-survey-*.log`（六域普查）、`logs/s329-todataurl.log`、`logs/s329-toblob.log`、`logs/s329-canvas-all.log`（r1）、`logs/s329-canvas-r2.log`（part05 补丁后 A/B 复跑，gitignored）。

## 挂账与队列

- canvas 域残红 3F+1T：`2d-getcontext-options`（options 转换）、`historical`（CanvasRenderingContext2D 不可构造）、`toBlob.jpeg`（jpeg 编码，toBlob type 参数面）、`security.pattern.fillStyle`（文件 T，cross-origin pattern 面）。
- 六域普查再生队列（下轮候选）：dialog focus restore 2F（activeElement declarative-shadow retarget + slot-inclusive containment，s330 勘根候选，需 live probe 验证 _shadowHandles 注册）、audio_constructor throw 1F、audio loading=lazy 10F、object width/height rendered-dim IDL、dialog-popover-closedby-complex 18F（需勘根）、CSP style-element 8T、microtasks 4F+2T。
- credentials 簇 12F+2T → runner-net-infra goal（移交已记，不占本 goal 队列）。
- tabular/document-metadata/scripting-1 残红按面归域（布局/CSP/module 已分别有归属，不重复列）。
