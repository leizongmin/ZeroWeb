# bg02-media 基线 v2 复评——判定与根因（2026-10-25）

复评对象：DC-2 归档缺口的 bg02-media INCONCLUSIVE 子项（2026-10-23 M4 gate pairing
ANALYSIS 列出的三缺口在 A2 修复后的重开条件）。本轮产物：4 条复评腿（显式 play ×2、
autoplay ×2）+ 三处根因修复 + 六项常驻回归测试 + 基线 v2。

## 一、A2 三缺口闭合判定（PASS）

| 2026-10-23 缺口 | v2 腿证据 | 判定 |
|---|---|---|
| 缺口 1/3：autoplay 绕桥、audio load 桥武装失败 | autoplay 腿 a=0.55 桥 0.552 isPlaying=true，audio-clock 序列 1.59→0.63→1.67 跨流末回卷 | **闭合**（A2 Fix 1 走 play 内核桥武装） |
| 缺口 2：march 死钩子 | 本轮另查明 march 泵门控缺陷（见下二.2）并修复；4/4 腿 video/audio 循环健康 | **闭合** |
| 显式 play 腿「audio 未武装」 | 临时 shim 埋点实测：两轮 load 的 `#a` 均 `hit=true`（`sync key=#a hit=true`） | **原判定系误读**（见下三） |

## 二、本轮新发现三缺陷（根因修复）

埋点链：bg02-media-debug.html 页面侧诊断 → shim 临时 console 埋点（play 内核五点 +
重试链 + setLoop 推送 + march ended 分支）→ registry 单测探针。全部诊断面已随修复
移除，回归测试落位常驻。

1. **loop 属性推送缺失**（shim，`crates/engine/src/js_dom_shim/part06.js` play 内核）：
   setLoop 推送裸读 `_pMs.loop === true`，漏解析属性形态（`<audio loop>` 静态 HTML
   ——IDL getter 走 attr 回退报 true，推送却 false）→ registry loop 关 → 音频 entry
   流末停播、视频 Ended 冻结，loop 属性整链失效。修复：推送读与 IDL getter/march
   同源（dirty 镜像优先 + attr presence 回退）。回归：part09
   `test_media_play_pushes_parsed_loop_attr_to_bridge`（解析属性/无属性/IDL 压制三面）。

2. **march 泵电平门 EOF 自断**（renderer，`apps/renderer/src/runtime.rs`）：
   `any_playing` 纯电平门在 registry 侧全部流末停播拍停交 march——恰是 shim 唯一能
   观察 isEnded 的时机（ended 派发/loop 回卷 seek(0)+play），paused 卡 false、ticks
   恒进、时钟冻结（2026-10-23「paused=false、ticks 推进而 currentTime 恒 0」形态的
   产品级根因）。修复：true→false 边沿补交一拍（`media_march_submittable` 纯函数 +
   `media_march_tests` 真值表四测试）。稳态零提交保持。

3. **音频重复登记掐死播放面**（webview，`crates/webview/src/video_registry.rs`
   `register_audio_source`）：无条件 `AudioEntry::new` 替换——load 后延迟 settle 重
   登记同 URL 时，默认新 entry（playing=false/loop_on=false/cursor=0）静默替换播放
   中的音频（显式腿武装后数百 ms 死亡实测）。video 面 `register_source` 只刷 sources
   字节、player 不经登记重建本即幂等——音频面对齐同语义（已有 entry 保留）。回归：
   `registry_audio_reregister_preserves_live_entry`（播放态/游标/loop 标志存续）。

伴生锁定：`registry_mp4_aac_audio_loop_rewrinds`（mp4/AAC entry 回卷面——此前常驻
loop 测试只覆盖 mp3）。

## 三、误读订正：「显式 play audio 从未起播」不成立

- 桥 play 首试即命中（埋点 `sync key=#a hit=true`，两轮 load 均然）。
- 终态读数 `a=0.00,paused=false,bridge 0,isPlaying false` 的真实链路：**单遍播放
  （0→2.0s）→ EOF 停播（loop 推送缺陷下 setLoop false）→ 音频 entry 游标复位 0 →
  shim playing 卡 true**（缺陷 2 的 march 自断使 paused 永不翻转）。5s 首采样落在
  EOF 之后，读到的全是终态——与「从未起播」观测等价但机理不同。
- 2026-10-23 的「缺口 3：audio load 桥武装失败」应据此订正为「武装成功但单遍停播 +
  状态卡死」；A2 Fix 1 的武装路径本身当时已生效。修复前原始腿（终态误读源头）留存于
  [2026-10-25-bg02-media-v2/](../2026-10-25-bg02-media-v2/)（bg02-media ×2 +
  autoplay ×2，修复前构建）。

## 四、测量伪象教训：终态读数对循环媒体不判定「是否播过」

`media_final` 快照与 5s 间隔采样对 2s 循环媒体是相位锁定采样：probe 时序确定 →
循环相位确定 → final 读数逐腿可复现（三腿 v=2.025/2.031/2.029 ±3ms 曾被误读为
「冻结卡死」；40s 长墙腿证实 #v 全程 7 采样连续回绕）。基线 v2 据此新增
`probe_travel:*` 回卷安全行程下界（路径长度，冻结恒 0），travel 界存在时取代
delta 界（回卷序列末减首随相位可为 0/负，autoplay-b 实测 -0.003）。

## 五、验证

- 腿目录入库裁剪：http.jsonl/http.log（fixture server 请求日志，`server_start`
  事件携带本机绝对路径字段）不入库；其余全量保留（run-config/samples/probe/
  browser.log），`background-resource-gate.py check` 可复现。
- 常驻回归 6 项全绿（part09 ×1 / webview ×2 / renderer ×4，test-guard 包裹）。
- 基线 v2：2 场景 n=2 配对，`background-resource-gate.py check` 4/4 PASS。
- 墙钟/内存：观测经 `systemd-run --user --scope -p MemoryMax=24G` 包裹。
- 工作区门禁：`make guarded-clippy` + `make test`（提交前跑，见 master 执行记录）。
