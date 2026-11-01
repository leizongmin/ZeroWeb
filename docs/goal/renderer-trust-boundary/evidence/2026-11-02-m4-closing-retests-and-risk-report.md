# M4 收口:决策面增量复测 + 剩余风险报告

日期:2026-11-02
基线:HEAD `29c8d2641`(代码面与切片 5 交付 `d5b305045` 一致,本轮无代码变更——M4 为控制面里程碑)。
范围:master「下一步」在册的决策面增量复测(切片 8 mutator 不可见面、切片 3 解码错误面/图像加载生产链、切片 4 renderer 默认沙箱生产链与 android 编译腿残余)+ DC-4 helper 受限操作余项 + 剩余风险报告。

## 复测一:切片 8 `__zw*` 特权 mutator 不可见面(生产链)

真二进制(默认 env,无任何出口变量)+ 本地 HTTP 真页面,CDP `Runtime.evaluate` 断言页面可见性:

| mode | `__zwUserActivate` | `__zwSetPermission` | `__zwClipboardStoreWrite` | `__zwDeleteAllCookies` | `__zwModalGateBypass` |
| --- | --- | --- | --- | --- | --- |
| 普通 renderer(无 `ZW_RENDERER_AUTOMATION`) | undefined | undefined | undefined | undefined | undefined |
| 自动化(`ZW_RENDERER_AUTOMATION=1` 经进程 env 继承) | function | function | function | function | undefined |

- 普通 mode 4 枚特权 mutator 全部不可见(counter-plugin ②A-env 面),自动化 mode runner 契约保持(4 枚 function)——两腿均与切片 8 验收一致。
- `__zwModalGateBypass` 页面加载态两 mode 均 undefined,与「同步置位/清除」设计一致(仅模态流瞬态可见);切片 8 的「接受残余」记档不受影响。

驱动为一次性验证脚本(方法如上表:spawn 浏览器 → stderr 取 token → 发现端点 ws → Page.navigate 真页 → Runtime.evaluate typeof 断言,两 mode 各跑一轮),不落仓;常驻覆盖在切片 8 的 9 用例回归。

## 复测二:切片 3 + 切片 4 默认沙箱生产链(真实进程链快照)

真二进制 headless + 本地 HTTP 页(valid.png = 仓库 icon-128;corrupt.png = 同文件 IDAT 中段 64 字节破坏),默认 env(无 `ZW_IMAGE_DECODER_*`/`ZW_RENDERER_*` 出口):

**进程链快照**(图像加载中 `ps --ppid` 两级采样):

```
zero-browser (headless, CDP)
└── zero-renderer --type=renderer --instance-id=1        ← 切片 4:默认 seccomp 收窄
    └── zero-image-decoder --type=image-decoder --instance-id=0  ← 切片 3:默认 seccomp+landlock
```

**解码面**:valid 128×128 naturalWidth 正常(沙箱化解码器生产链全通);corrupt naturalWidth=0 + complete(解码错误面,无内联回落);浏览器与两子进程存活,stderr 零 sandbox 降级/fallback 信号(fail-closed 下安装失败应 exit(1),无降级日志即无安装失败)。

**链路语义**:renderer 进程带默认 seccomp 完成导航/JS/子资源 fetch/排版全链;图像解码经 renderer → image-decoder helper 完成且 helper 处于默认双沙箱。切片 3/4 的「生产链不复退」M4 判据收口。

**适用门禁(同轮)**:DC-13 `make product-smoke`——welcome 800px vs chromium oracle **diff 18.14% ≤ 20% PASS**,struct-check 各腿 PASS(进程内渲染管线回归门,与进程沙箱互补的产品面信号)。

## 复测三:DC-4 helper 受限操作余项——media-host 收口期新发现

helper 全量盘点(决策包 ③ IO 面清单外的新事实):

| helper 进程 | 沙箱 | 受限操作验收 |
| --- | --- | --- |
| compositor | seccomp+landlock(M3 切片 1) | 真实 restrict + 真实进程验收 DONE |
| image-decoder | seccomp+landlock 默认开(切片 3) | 执行型子探针 + 本轮生产链 DONE |
| renderer(准 helper,信任模型核心角色) | seccomp 默认开 + landlock opt-in(切片 4) | 执行型子探针 + 本轮生产链 DONE |
| fd 通道 | 0600+SO_PEERCRED+token(M3 切片 2) | round-trip + dma-buf e2e DONE |
| **media-host**(`apps/media-host`) | **无任何沙箱钩子**(main.rs/kernel.rs 零 seccomp/landlock/apply_early) | **无验收对象——新缺口** |

media-host IO 面查实:stdio IPC(PipeTransport 双向)+ cpal 音频设备输出(`audio-cpal` feature);网络 socket 零命中;`fs::read` 仅测试 fixture helper(lib.rs:454、kernel.rs:1029)。即:被攻破的 media-host 可用系统调用面 = stdio + 音频设备,无网络外联、无任意文件读(运行期)。

**处置**:决策包 ③ 范围仅 image-decoder + renderer(拍板原文),media-host 沙箱为**范围外新切片候选**(③A-media-host 最小档案:seccomp 网络/exec 黑名单 + landlock 允许音频运行面),按授权边界**不自行开工**,转缺口表待决 + 剩余风险报告。DC-4 维持 PARTIAL(余 media-host 待拍板 + android 编译腿 + macOS 真机)。

## android 编译腿残余(复核维持)

切片 4 已记录:本环境 `v8` registry crate 无 aarch64-linux-android bindings,renderer android 构建走 v8 源码编译路径(六项前置 + 代理),全量 android check 不可行;以 libc aarch64 常量逐一静态核对替代。本轮复核无环境变化,残余维持,M4 不强行解锁。

## 剩余风险报告(M4 收口交付物)

### 需用户拍板(阻塞项)

1. **renderer landlock 默认值——file:// 取舍三选项**(缺口表在册):A browser 预解析→LoadHtml / B profile 显式扩权 / C file:// 收窄为错误面。拍板前 landlock 维持 opt-in。
2. **media-host 沙箱切片**(本轮新发现,见复测三):建议 ③A-media-host 最小档案;不拍板则维持 fail-open 记风险(攻击面限定 stdio+音频设备)。
3. **DevTools frontend page-direct 在强制 token 下 fail-closed**(切片 5 记档):后续选项 per-page ws token 注入或 frontend 场景文档化。

### 环境受限在册

4. macOS 真机验收(seccomp/landlock/fd 通道均 Linux-only 编译门控,代码级对照已完成)。
5. android renderer 编译腿(上文)+ Android service bootstrap 未接线(image-decoder + renderer 两处,切片 3/4 记档)。

### 接受残余(有据记录)

6. `__zwModalGateBypass` 模态流瞬态可见(仅自影响焦点语义;加载态不可见,本轮实测)。
7. 显式 token 纯带外交付 flag(发现端点 URL 发布语义,按需立项)。
8. JS Web Storage 每文档 stub、headless `Page.navigate` 到 `zero://settings/*` 阻塞 mux ~15s(DC-5 复测记档的既有功能缺口,非本 goal)。

### 非本 goal 发现(移交对应流)

9. cdp-e2e clean HEAD 7 个 click-stability 失败(S1114 2026-09-18 基线后现形,切片 5 已证与 token 无关)——engine 渲染/动画 idle 面另案归因。
10. browser-compositor-smoke 脚本漂移(M3 切片 1 记档)。
11. `data:` URL 导航 `net::ERR_FAILED`(本轮复测观察:headless 导航 data: 页失败落 error:// 页,错误面正常无崩溃;engine/navigation 流参考)。

## 清理证据

本轮验证进程全部显式终止:retest 浏览器(SIGTERM/SIGKILL 兜底,驱动内建)、进程树探针浏览器(kill+kill -9 兜底)、本地 fixture HTTP server(34401,evidence 落盘后终止)。无 Xvfb 操作(本轮全程未触碰显示设施)。

## 结论

- 复测一/二:切片 3/4/8 决策面生产链增量复测全部通过,DC-5 增量复测判据收口。
- 复测三:media-host 无沙箱为收口期新缺口,转待决;DC-4 维持 PARTIAL(余项全部为待拍板/环境受限,无可自行推进的修复面)。
- 适用质量门禁:DC-13 product-smoke PASS;切片 5 交付时的全量门禁基线(全量 bin 736P/2F 既有红、workspace clippy 清零、fmt clean)对当前 HEAD 代码面继续有效(本轮零代码变更)。
