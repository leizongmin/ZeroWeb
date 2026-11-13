# M2 s417 收口账——video_size_preserved_after_ended 1T 勘根修复（settle duration 真值链 + headless EOS ended 派发）+ 16T 清单分型

- 日期: 2026-11-13；基线: HEAD db6019177（s416 收口），s416 账本所记「form-submission-target 6T 勘根首位」系**过期指针**（s392 已归因 navigation M3 门控，账本 line 7618 在案）——本轮改以 sweep T 清单证据分型开题。
- 驱动: 在册 16T 清分（16T 逐案归因/施工/门控收口），video_size_preserved_after_ended 为唯一 engine/runner 施工候选。

## T 清单分型（/tmp/s415/fix4sweep-2.log 17 文件级 T → 在册口径 16T = 17 − button-click-submits s392 flake）

| 案 | 数 | 归因 |
|---|---|---|
| forms/form-submission-target/*（rel-input/rel-form/rel-button/rel-base/request-header/blank-useractivation） | 6 | **navigation M3 门控**（真实提交导航/浏览上下文创建；s392 在册归因 line 7618 + 门控范围 7345/7366——iframe 树/深结构用户门控 rule 11） |
| links/links-created-by-a-and-area-elements/target_blank_*（useractivation/implicit_noopener/implicit_noopener_base）+ rel-attribute-clearing | 4 | **同 M3 门控**（target=_blank 新浏览上下文创建；勘根注记：`_zwAnchorActivate` part03.js:30774 无 target 属性处理——current-context 导航面，不属本流施工位） |
| forms/the-select-element/customizable-select/select-home-end-pagedown-pageup(.detailed).optional | 2 | **簇 D**（select/combobox 立项征询待用户决策；s415 已记族内 T↔F 换位漂移红集不变） |
| forms/the-button-element/button-click-submits | 1 | s392 在册 flake（solo 全绿 = sweep 负载 flake，不入在册） |
| embedded-content/the-video-element/video_size_preserved_after_ended | 1 | **本轮施工收口**（见下） |
| embedded-content/the-img-element/sizes/sizes-auto | 1 | F5 用户门（68 案独立面挂账） |
| embedded-content/the-canvas-element/security.pattern.fillStyle.sub | 1 | **13-hang 单点勘根**：forEachCanvasSource 工厂 #2 cross-origin SVGImageElement 载入链（createElementNS SVG + xlink:href + externalResourcesRequired，onload 永不派）阻塞 13 串行 promise_test；test #1（cross-origin HTMLImageElement）已 Pass——SECURITY_ERR 面半存在。SVG `<image>` 载入链 = engine 未来候选（挂账池新记） |
| document-metadata/the-style-element/style-error-01 | 1 | **同名异案辨析**：s413/s414 收口的是 the-link-element/link-style-error-01（本轮 sweep 全 Pass 复核在案）；the-style-element/style-error-01.html 为独立慢性文件级 T（两基线轮与本轮均 T，不除账） |

## solo 复现与探针定谳（zzz-s417-probe.html，已删）

- 基线文件级 Timeout；manifest 全域唯一 ended 等待案（wait_for("ended") 挂死）。
- 探针 DUMP（8s 后）：`dur:600, ended:false, cur:7.987, rw:320, rh:240`，事件链 loadstart→durationchange→loadedmetadata→canplay→canplaythrough→play→playing 全派。
- 判读：**载入链健康**（mp4 404→error→webm test-1s.webm loaded→parent available→加载序列派 loadedmetadata；dims 真值 320×240）；march 墙钟推进正常；唯一断点 = **duration 真值缺失（600 fallback 烘焙）+ ended 永不派**。

## 根因三层

1. **runner settle durationMs 恒 None**（tests/wpt-runner/src/testharness.rs:9786 原形态 `reg.duration()`）：runner 无 `media-process` feature（Cargo default=["v8"]），registry 走 in-process 面——`duration()` 读 `players`（play 时才建，settle 恒空）→ commit durationMs=null。dims 之所以有真值（320×240）是 `probe_dimensions` 现场开解码器探针——两条读取路径不同构，duration 缺同款探针。
2. **shim 600 fallback 烘焙**（part06.js:13353 load 序列 `_settled.durationMs` null → duration getter 兜底 600）：ms.duration=600 恒大于 runner 超时窗，EOS 判定永假。
3. **无桥 march 无 EOS/ended 派发**：源子元素 video 无 src 属性 → play 桥命中判定读 **src 属性**（part06.js:13099 `_pSrcRaw`）→ `_pAbs` 空 → `bridgeOn` 恒 false → march 走 headless 墙钟臂（cur 7.987 实证），桥 ended 臂永不入门。

## 修复（两件套）

- **runner settle 单探针真值**（testharness.rs，static + dynamic 两处）：Video/Source loaded 臂改 `reg.source_media_meta(&resolved)` 单探针 (w,h,durMs)——in-process 现场开 `VideoTrackDecoder::open_media` 读容器声明时长（webm Info/Duration raw 1008.0 tick × scale 1e6/1e6 = **1008ms**；既有 fixture 单测 sample-webm-vp9 Some(2000) 佐证公式正确）。audio 臂保持 mirror 读（open_media 不适用音频字节）。media-process 开态读宿主 mirror（settle 时点同 None，零回归）。
- **shim headless EOS ended 派发臂**（part06.js，`_zwMediaTimeMarchesOn` 内、桥 ended 臂后）：`!ms.bridgeOn && ms.playing && duration 真值 && nowMs >= duration*1000` 门控——镜像桥 ended 臂语义（loop 分叉回卷 vs 尾段 played/ended IDL 翻转 + cue exit + timeupdate/ended 事件对 + `_zwEndedDispatched` 单次门）；600 fallback 恒惰性不误触（600s ≫ runner 超时窗）。spec https://html.spec.whatwg.org/multipage/media.html#ended-playback 。

## 附带考证

- matroska-demuxer 0.8.1 `duration()` 文档自述 ns，实证返回 raw tick（fixture DurationRaw 2000.0 × 公式 → 2000ms 与 ffprobe/单测一致）——decode.rs 公式无需改，文档误导记此。
- test-1s.webm 容器：TimecodeScale 1000000、Duration float64 raw 1008.0（EBML 手解析在案）；test-1s.mp4 缺席（上游同缺，source 候选语义 error 落下一候选）。

## 验证

- solo：video_size_preserved_after_ended **Pass**（修复前 Timeout）。
- sweep ×2（8801/8800 行）pairs join 对 s415 双基线（fix4pairs-1/2，(key,state) 语义）：
  - **my2 vs b2 = 0 state-diff + 恰 1 new**（driving `Timeout→Pass`，file 键 gone + 子测键 new 同源）——配对轮全等，唯 driving diff。
  - my1 vs my1 自身双跑仅 1 state-diff：checkable following-label 子测（ck F→P）——同二进制双向翻转 = flake（b1 ck-F/b2 ck-P/m1 ck-F+radio-F/m2 双 P；该文件无 media 元素，settle 零行为变化机制排除）；慢性 parent-label 2F 四轮全 F 不变（在册挂账承 s410）。
  - select-home-end ×2 F→T 族漂移仅对 b1 显（vs b2 零差；b1=F/b2=T/m1=T/m2=T）——s415 在册族内漂移记录复现，非本轮。
- 数字闭合：在册红 **212F+16T → 212F+15T**（−1T 除账 video）；NotRun 5 不变；sweep 口径 17 文件级 T − flake − driving = 15。
- make test 全量：155 腿 154 绿 + 仅 engine lib 在册红腿 3911P/**7F** 失败清单
  逐名 IDENTICAL 零位移（img_runtime_deferred_settle_r5275 /
  svg_img_settle_density_r5277 / fetch_filtered_response_type_matrix /
  iframe_service_worker_messages / response_body_readable_stream_r2967 /
  t8k_websocket_error_nonterminal / t8k_websocket_event_flow——对 s415 收口
  轮 make test 同名单 diff 空）；clippy -p zero-wpt-runner -D warnings 干净 +
  rustfmt --check 干净（make test 内建 quickjs clippy 腿同绿）。

## 下一步

- NotRun 5 = css-module/json-module content-type 子测（外链模块基座 map 键错配）为剩余池首位候选；挂账池承 s416（渲染流 6 面/sizes-auto 半边 F5/簇 D 立项征询/A2+D 簇/checkable parent-label 2F 渲染域）+ 新记 SVG `<image>` 载入链（canvas 13-hang 面）。
