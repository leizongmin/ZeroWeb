# F09 播放媒体

## 任务、入口与预期

用户在 audio/video 上播放、暂停、seek、调音量/静音，等待结束；应用调用 Web Audio。
预期来自媒体播放/音频/HTMLMediaElement 的既有验收；解码成功不能代替真实声音或连续帧。
生产入口为 live 页面 HTMLMediaElement 与 AudioContext；
`crates/media` 的 player/解码 API 是底座入口，不证明网页和设备输出。
源码：`crates/media/src`、`crates/webview/src/video_registry.rs`、
`webaudio_registry.rs`、`apps/renderer/src`。

## 准备与动作

先 [Doctor](../SKILL.md)，按目标 codec 核对构建 feature、音频设备和 GPU；
使用现有媒体验收中的本地 fixture，不下载未经许可内容。
按 [播放验收](../../../../docs/goal/archive/media-playback/master.md)、
[音频验收](../../../../docs/goal/archive/media-audio/master.md)、
[元素验收](../../../../docs/goal/archive/media-elements/master.md) 选择原有工具与具体测试，
通过既有守卫执行；没有已准备 fixture/设备时 BLOCKED，不制造空启动通过。
在本次页面点击播放、观察推进，再暂停、seek 到指定时间、继续播放、静音/恢复，
最后等待 ended。Web Audio 另测 start/stop/disconnect 与实际输出。

## 判据、恢复与证据

读取 currentTime、paused、ended、音量/静音和对应事件；比较连续视频帧及 seek 后内容。
真实音频输出须听测或既有输出测量；仅 API 状态不证明可听声音。
不支持 codec/解码失败应产生可观察错误，不持续增长/挂死；停止后重载本次 fixture，
重新 Doctor 验证设备和页面状态。保留事件/时间记录、帧/输出证据、codec 与设备环境。

清理本次页面、媒体流、音频节点和服务，确认无播放/输出残留；不改用户设备音量，
不退出用户音频进程。保留证据并实际重读。
2026-10-14 F09 `NOT_RUN`；没有本轮媒体闭环，版本环境见 [初始化记录](../initialization.md)。
