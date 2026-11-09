# M2 s332：媒体接口 Illegal constructor + WC-M1 定制内建 super 注册门——audio_constructor 1F→P 零位移

日期：2026-11-07 ｜ 承接：s331（`2026-11-07-m2s331-closedby-complex-root-cause-transfer.md`）

## 目标与定谳

`the-audio-element/audio_constructor.html` 1F「Constructing HTMLAudioElement should throw」：直构造 `new HTMLAudioElement()` 返回元素而非抛 TypeError。勘根定谳为**三重叠缺陷**：

1. **注册门缺失**：part03 `_zwHtmlElementIfaces` Audio/Video 特化分支的包装闭包对**任意** new.target 放行 `__zwIfaceSuperCreate`——直构造的 new.target 即包装自身（未注册），也建成元素。spec 元素接口构造步只对「new.target 为**已注册**定制内建」（`customElements.getName(new.target)` 非空）开放元素创建，其余落 Illegal constructor。
   https://html.spec.whatwg.org/multipage/dom.html#html-element-constructors
2. **HTMLVideoElement 从未定义**：part01b 只定义了 HTMLAudioElement；part03 泛化分支 `if (!globalThis[...])` 为本名铸出**可构造**占位，`new HTMLVideoElement()` 返元素不抛（特化分支注释所称「已由 part01b 定义」对 video 从未成立）。spec 无 [Constructor]，实例经 `createElement('video')` 产出。
   https://html.spec.whatwg.org/multipage/media.html#htmlvideoelement
3. **`var` 循环捕获错位**：包装闭包调用期读 `var _zn`（函数级共享绑定，被 video 迭代覆盖）——audio 包装反查 tag 得 `'HTMLVideoElement'`，`new XAudio()` 建 VIDEO 元素（契约测试 tag 断言面）。`var _znCap = _zn` 同在循环体内，**同样被共享**，等价失效。

## 施工（三处）

- `part01b.js`：补 HTMLVideoElement Illegal-constructor 定义（prototype 链 Media→HTMLElement 同 Audio），落进特化分支接管。
- `part03.js`：包装闭包改 **IIFE 按定义轮捕获** `_zn/_znMediaCtor/_znMediaProto`；super() 在途判定加注册门——`customElements.getName(new.target)` 命中已 define 的定制内建才走 `__zwIfaceSuperCreate`，未注册 new.target（含直构造、未注册子类）落回原 Illegal constructor throw（Chromium 同语义）。
- `part09.rs`：契约测试 `test_media_iface_ctor_illegal_and_customized_builtin_s332` 三臂——① 直构造 audio/video/调用式全 throw TypeError；② `class XAudio extends HTMLAudioElement` + `define('x-audio', …, {extends:'audio'})` → 建 AUDIO 元素、`is='x-audio'`、instanceof XAudio；③ video 同构 → throw + 建 VIDEO 元素 + instanceof HTMLMediaElement（prototype 链面）。

### JS `var` 循环捕获陷阱（本次两踩）

`var` 绑定函数级共享：循环内闭包读 `_zn` 得末轮值；首轮 `var _znCap = _zn` 也被末轮覆盖，救不回来。正确做法只有按轮固化作用域——IIFE 传参捕获（或改 `let`，但本 shim 须兼容无 `let` 的既定风格）。

## 验证

| 面 | 方式 | 结果 |
|---|---|---|
| 契约三臂 | `cargo test -p zero-engine --lib test_media_iface_ctor_illegal_and_customized_builtin_s332` | ① TypeError×3 ② tag-audio/is-ok/instance-ok ③ TypeError/tag-video/media-proto-ok 全过 |
| 目标文件 | audio_constructor.html 单文件跑 | 10P/1F → **11P/0F**（exit 0），`logs/s332-ac-after.log` |
| video-element 域 | the-video-element 全量 | 20P/3F/1T，3F+1T 与 s329 survey 基线逐键相同（resize-during-playback ×2、video-loading-lazy、video_size_preserved Timeout）——零回归，`logs/s332-video-after.log` |
| embedded-content 域 | 全域 A/B（基线 `logs/s329-survey-embedded-content.log`） | 1267P/55F/14T → 1272P/51F/14T；键级翻转恰 5 个 = 1 目标（audio_constructor F→P）+ 4 canvas（s329 canvas 修落地对 pre-fix survey 基线，预期）；键数 1335→1336 差 1 = `zz-s332-probe.html`（探针临时文件在 after 跑次注册的 Fail 键，用后已删，comm 实证）。`logs/s332-ec-after.log` |
| WC 定制内建 | custom-elements corpus FILTER | goal script 只钉 html/semantics，corpus 0 结果——回归面由契约测试②③守（工作面规则），`logs/s332-wc-before.log` |

注：video prototype 链变化（占位→Media 链）在 video-element 域 24 键零翻转，链面无回归。

## 挂账与队列

- 下一候选：audio loading=lazy 10F、object width/height rendered-dim IDL、canvas 残红 3F+1T（承 s329）、CSP style-element 8T、microtasks 4F+2T。
- s331 移交面（shadow 布局 + D1 协议分隔符）不在本 goal 可动面，待 shadow-dom-rendering M3。
