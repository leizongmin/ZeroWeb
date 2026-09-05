# Goal 文档归档

已完成（或 scope 全落地）的顶层 goal RFC / 设计稿移入此目录，活跃 goal 仍保留在 `docs/goal/` 根目录。

| 文档 | 归档日期 | 状态 |
|---|---|---|
| [render-threading-rfc-2026-08-07.md](./render-threading-rfc-2026-08-07.md) | 2026-08-11 | 已实施（S1/S2/S3 ✅） |
| [compositor-process-rfc-2026-08-07.md](./compositor-process-rfc-2026-08-07.md) | 2026-08-11 | 已实施（P0–P3 ✅；frame_flow 17/17；Linux 默认 GPU dma-buf 链路） |
| [ai-refactor-acceptance.md](./ai-refactor-acceptance.md) | 2026-08-11 | 已落地（compositor 切片验收记录见 compositor-process-rfc 第八节） |
| [network-loading-performance-2026-08-14.md](./network-loading-performance-2026-08-14.md) | 2026-08-15 | 已实施（P0/P1/P2 ✅；仅支持 HTTP/1.1/2） |
| [canvas-2d.md](./canvas-2d.md) + [canvas-2d/](./canvas-2d/)（master/evidence/archive） | 2026-08-17 | 已完成（DC-1~4 ✅：WPT 919 文件导入、testharness 全绿、oracle-pass 100%/不一致 0、Mission 中期 80% 达成；完成日 2026-08-16） |
| [html-compat.md](./html-compat.md) + [html-compat/](./html-compat/)（master/test-matrix/completion-audit） | 2026-08-17 | 已完成（M0-M4 ✅：FR-001~012 / NFR-001~008 / IF-001~005 / 门禁全项 complete，completion-audit 2026-08-13；完成日 2026-08-13） |
| form-validation（模式 B 入口自归档——入口在 [../form-validation/archive/form-validation-goal-v1-2026-08-17.md](../form-validation/archive/form-validation-goal-v1-2026-08-17.md)，master/evidence 原地保留） | 2026-08-17 | 已完成（M1-M3 ✅：constraints 919 Pass / 0 Fail、提交阻断全链路；完成日 2026-08-17，归档由该流 commit `eb44f905e` 执行，本表补登记） |
| [storage-indexeddb.md](./storage-indexeddb.md) + [storage-indexeddb/](./storage-indexeddb/)（模式 A：master/evidence/archive） | 2026-08-19 | 已完成（DC-1~4 ✅：固定 WPT revision imported 168/210 文件、1073/1073 Pass、零 skip；Rust host 接线、Browser ownership、跨会话持久化与错误回滚完成） |
| [js-dom.md](./js-dom.md) + [js-dom/](./js-dom/)（模式 A：master/evidence/archive，108 归档切片 + 130+ evidence） | 2026-08-31 | 已完成（DC-1~8 ✅：双引擎 native default-on + kill-switch 删除、Vue/lit/WC e2e、WPT dom 上游 8 域 55808P/99.95%、roundRect panic 修复；R385 判定矩阵 DONE，完成日 2026-08-31） |
| [media-elements.md](./media-elements.md) + [media-elements/](./media-elements/)（模式 A：master/evidence/archive） | 2026-09-05 | 已完成（DC-1~4 ✅：HTMLMediaElement 非解码语义面——readyState/networkState 状态机、load 算法、事件序列、canPlayType、play/pause、track 面；WPT 终态 640P/0F/13PF = 98.01%，209 用例导入，Fail/Timeout 双清零；余 13 PF 全为选型面外编解码；完成日 2026-09-05） |
| [media-playback.md](./media-playback.md) + [media-playback/](./media-playback/)（模式 A：master/evidence/archive） | 2026-09-05 | 已完成（DC-1~5 ✅：M0 解码选型 RFC 获批路线 C（VP9 纯 Rust + AV1 dav1d）+ M1 帧上屏 + M2 连续播放（renderer 播放泵 D4）+ M3 多格式（AV1 + H.264 mp4/AAC，切片 2 AAC 音频链/伴生轨/precise-seek）；H.264 分发前置法务复核独立于 goal；完成日 2026-09-05） |
| [media-audio.md](./media-audio.md) + [media-audio/](./media-audio/)（模式 A：master/evidence/archive） | 2026-09-05 | 已完成（DC-1~5 ✅：cpal CpalSink 真设备流抽验 + NullSink headless 总线断言 + A/V 同步 audio clock 主时钟 + volume/muted 真控制 + `<audio>` 全路径 e2e + AudioContext 最小面（WPT webaudio 50 用例 1418P/0F = 100%）；余项 Mixer N→1 挂桌面可选切片非阻塞；完成日 2026-09-05） |

`rendering-compat/` 子目录的已完成设计稿见 [`../rendering-compat/archive/`](../rendering-compat/archive/)。
