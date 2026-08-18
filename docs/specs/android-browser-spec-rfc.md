# Spec + RFC：ZeroWeb Android 浏览器应用

**版本**：v0.1
**日期**：2026-08-19
**作者**：AI Assistant
**状态**：待确认

---

## 0. 执行摘要

- **一句话目标**：在 `apps/android-browser/` 新增应用 ID 为 `com.leizm.zeroweb` 的 Android 浏览器，以 Kotlin/Jetpack Compose 提供原生移动端 chrome，以 Rust 提供浏览器业务状态和 ZeroWeb 内核，并保持 renderer、compositor、image-decoder 物理多进程隔离。
- **本期范围**：Android 手机 arm64 可侧载 Release APK；x86_64 Debug APK 仅用于本机模拟器验证；多标签、地址栏和导航、书签、历史、下载；触摸、软键盘、系统返回、旋转和进程恢复；真实网页 GPU 呈现；Android Service 多进程；自动化测试和至少一台 arm64 真机验收。
- **明确排除**：Play 商店发布、账号同步、扩展、无痕模式、标签分组、桌面模式、广告拦截、平板专用双栏 UI、x86/32 位 ABI、x86_64 Release 分发、Android System WebView。
- **核心约束**：
  1. 页面必须使用 ZeroWeb 自研内核，不得调用 Android System WebView 渲染网页。
  2. renderer、compositor、image-decoder 必须由独立 Android Service 进程承载；不得提供进程内 renderer 回退或开关。
  3. `zero-browser-shell` 及其 Android facade 是浏览器业务状态的唯一事实源；Compose 不维护第二套标签、书签、历史或下载状态。
  4. 浏览器进程持有网络、持久化和 Android 权限；隔离 renderer 不得直接访问网络、任意文件或 Android 系统服务。
  5. 大帧像素不得经 Binder 序列化；compositor 必须直接面向 Android Surface 呈现，控制 IPC 与帧数据面分离。
- **推荐方案**：Kotlin/Compose Activity + JNI Action/Snapshot facade + Android Binder/Service 进程启动器 + Rust `cdylib` 角色入口 + compositor 持有 `ANativeWindow`/wgpu Surface。
- **首个落地步骤**：新增仅含 Android Activity、四类 native role entry（优先共用一个 `.so`）、Service manifest 和 Binder 握手测试的 M0 骨架；验证 `assembleDebug`、安装启动及 browser 与三类 helper 的至少四个 PID 均存在，不先实现浏览器 UI。

---

## 1. 背景与目标

### 1.1 背景

ZeroWeb 当前包含桌面 `apps/browser/`、独立 `apps/renderer/`、`apps/compositor/`、`apps/image-decoder/` 和嵌入式 `crates/webview/`。桌面生产浏览器固定使用 browser → renderer → compositor，以及 renderer → image-decoder 的进程边界；`ZeroWebView` 则是进程内嵌入边界，不能作为 Android 浏览器生产架构的替代品。

现有 `crates/browser-shell/` 已提供标签、书签、历史、下载与设置模型，但桌面 `apps/browser/` 同时承担窗口、UI 绘制、子进程启动和页面协调，无法原样移植到 Android。Android 应用没有桌面式单一 `main()` 启动流程，辅助进程由 Activity Manager 通过 Android Component 创建，Activity 生命周期、IME、权限、下载通知和系统返回也需要原生宿主。

因此，本项目新增与桌面应用并列的 Android 交付物，复用浏览器业务模型、页面管线和协议语义，替换宿主、UI、进程启动和 Surface 交付方式。

### 1.2 目标

- **业务目标**：提供可安装、可实际浏览网页的 ZeroWeb Android 版本，证明 ZeroWeb 自研内核和固定多进程架构可在 Android arm64 运行。
- **用户目标**：用户可以在 Android 手机上使用地址栏和触摸浏览网页，管理多个标签、书签、历史与下载；应用旋转、退后台或 renderer 被回收后不会静默丢失浏览会话。
- **工程目标**：Android 与桌面复用 `zero-browser-shell`、`zero-protocol`、renderer 页面管线及 render foundation，不复制一套浏览器内核；Android 特有代码收敛在明确 adapter 中。

### 1.3 核心用户旅程

1. **启动与恢复**：用户启动 App；系统展示上次会话或一个新标签；恢复失败时显示可操作的恢复提示。
2. **导航与阅读**：用户输入 URL/搜索词，页面加载并可触摸滚动、点击、输入；加载错误显示重试入口。
3. **多标签**：用户新建、切换、关闭标签；标签概览显示标题、URL、缩略图或占位图。
4. **保存与回访**：用户收藏页面、查看/搜索书签与历史，并重新打开条目。
5. **下载**：用户触发下载、选择或接受目标文件名、观察进度，并从系统通知或下载页打开结果。
6. **生命周期与故障恢复**：Activity 旋转/重建、退后台或 renderer 被系统终止；浏览器恢复 UI 状态，崩溃标签明确显示“页面已停止”，由用户点击重新加载。

### 1.4 范围边界

#### 在范围内

- `apps/android-browser/` Android Gradle 工程及 Rust `cdylib`。
- Android 手机布局，支持竖屏、横屏、分屏和系统字体缩放。
- Release 使用 `arm64-v8a`；Debug 额外使用 `x86_64` 供本机模拟器验证。最低 Android API 26，`compileSdk`/`targetSdk` 为 36。
- Kotlin + Jetpack Compose 浏览器 chrome。
- Rust Action/Snapshot facade，复用 `zero-browser-shell`。
- renderer、compositor、image-decoder Android Service 进程。
- 标签、书签、历史、下载的本地持久化。
- 页面导航、触摸/点击/滚动、IME 文本输入、系统返回。
- GPU 首选呈现；GPU 初始化失败时允许 compositor 内的 CPU 光栅化，但仍由独立 compositor 进程呈现，且不得切换到进程内页面内核。
- Debug APK、Release APK（本地测试签名或用户提供签名）和 ADB 安装/验收脚本。

#### 不在范围内

- Google Play 上架、AAB、正式生产签名与商店合规材料。
- Android 平板专用信息架构、折叠屏双栏界面；但布局不得锁定方向或在大窗口崩溃。
- x86/32 位 ABI 与 x86_64 Release 分发。
- 账号、云同步、跨设备发送、密码管理器、自动填充服务。
- 无痕模式、标签分组、扩展、开发者工具 UI。
- Service Worker 后台常驻、后台音频和画中画。
- 重写桌面浏览器 UI 或立即实现通用 `ui/` SDK。

---

## 2. 需求类型概览

| 类型 | 是否适用 | 来源 |
|---|---|---|
| 业务需求 | 是 | 用户要求新增 Android 浏览器交付物 |
| 用户需求 | 是 | §1.3 用户旅程 |
| 解决方案需求 | 是 | 用户确认 Kotlin/Compose + Rust + 固定多进程 |
| 功能需求 | 是 | §3 FR-001～FR-010 |
| 非功能需求 | 是 | §4 NFR-001～NFR-009 |
| 接口需求 | 是 | §5 IF-001～IF-008 |
| 过渡需求 | 是 | §7 里程碑与桌面共享代码抽取 |

---

## 3. 功能需求

### FR-001：安装、启动与会话恢复

- **描述**：安装后的 App 必须通过 launcher 启动；首次启动创建一个新标签，后续启动恢复上次正常会话中的标签顺序、活跃标签、URL 和每个标签的最近导航状态。
- **优先级**：必须
- **来源**：用户确认首期完整 MVP；§1.3 阶段 1、6

**验收场景**：

```text
场景: 首次启动
  假设 arm64 设备上未保存 ZeroWeb profile
  当 用户从 launcher 启动 App
  那么 系统显示一个可输入地址的新标签，且浏览器 chrome 在 3 秒内可交互
  验证: Android 启动测试 firstLaunchCreatesSingleTab

场景: 会话文件损坏
  假设 profile 中的会话文件截断或无法反序列化
  当 用户启动 App
  那么 系统隔离损坏文件、创建一个新标签并显示一次非阻塞恢复失败提示，不得崩溃或覆盖书签与历史
  验证: Rust 测试 corruptSessionFallsBackWithoutDeletingProfile + Android 测试 corruptSessionShowsRecoveryNotice
```

### FR-002：地址栏、导航与页面交互

- **描述**：用户必须能输入 URL 或搜索词，执行加载、后退、前进、刷新和停止；加载后的页面必须接收触摸点击、拖动滚动、长按、焦点和软键盘文本输入。
- **优先级**：必须
- **来源**：用户要求 Android 浏览器；§1.3 阶段 2

**验收场景**：

```text
场景: 导航并操作真实页面
  假设 renderer、compositor 可用且设备联网
  当 用户输入 HTTPS URL、点击页面输入框、输入文本并滚动
  那么 地址栏提交导航，页面获得焦点和文本，滚动位置改变，标题与最终 URL 回写 chrome
  验证: ADB 真机场景 android_navigation_input_scroll.json

场景: 导航失败
  假设 DNS 失败、TLS 失败或 renderer 返回 LoadFailed
  当 用户提交 URL
  那么 当前标签显示包含失败类型和“重试”的错误页，地址栏保留用户 URL，其他标签保持可用
  验证: Android 集成测试 navigationFailureIsTabScopedAndRetryable
```

### FR-003：多标签操作

- **描述**：用户必须能新建、查看、切换、关闭和恢复最近关闭的标签；标签概览必须显示标题、URL、加载/崩溃状态以及缩略图或确定性的占位图。
- **优先级**：必须
- **来源**：用户明确要求首期包含多标签

**验收场景**：

```text
场景: 操作多个标签
  假设 已打开三个不同 URL 的标签
  当 用户从标签概览切换、关闭一个标签并撤销关闭
  那么 活跃标签、标签顺序和恢复后的 URL 与操作一致，页面状态不会串到其他标签
  验证: Rust 测试 androidFacadeMultiTabRoundTrip + Compose 测试 tabGridSwitchCloseUndo

场景: 关闭最后一个标签
  假设 当前只剩一个标签
  当 用户关闭该标签
  那么 系统立即创建一个新标签，不留下无页面且不可操作的主界面
  验证: Rust 测试 closingLastTabCreatesReplacement
```

### FR-004：书签

- **描述**：用户必须能收藏/取消收藏当前页面，在书签页查看、搜索、打开和删除书签；每次成功变更必须原子持久化到 Android profile。
- **优先级**：必须
- **来源**：用户明确要求首期包含书签

**验收场景**：

```text
场景: 收藏并重新打开
  假设 当前标签已成功加载一个 HTTPS 页面
  当 用户收藏页面、重启 App、从书签页打开该条目
  那么 书签标题和 URL 被恢复，并在当前标签导航到该 URL
  验证: Rust 测试 bookmarkPersistsAtExplicitProfilePath + Compose 测试 bookmarkScreenOpensEntry

场景: 书签写入失败
  假设 profile 存储写入返回失败
  当 用户收藏或删除书签
  那么 UI 显示保存失败且内存状态回滚到写入前版本，不得报告成功
  验证: Rust 测试 bookmarkMutationRollsBackOnAtomicWriteFailure
```

### FR-005：历史记录

- **描述**：成功提交的顶层页面导航必须写入历史；用户必须能查看、搜索、打开单条记录，以及清除全部历史。失败导航、内部错误页和恢复中的崩溃占位不得写入历史。
- **优先级**：必须
- **来源**：用户明确要求首期包含历史

**验收场景**：

```text
场景: 查询并打开历史
  假设 用户已成功访问两个页面
  当 用户搜索其中一个标题并点击结果
  那么 仅显示匹配记录并在当前标签打开其 URL
  验证: Rust 测试 committedNavigationPersistsAndSearchesHistory + Compose 测试 historySearchOpensResult

场景: 清除历史写入失败
  假设 持久化层拒绝清除事务
  当 用户确认清除全部历史
  那么 现有记录继续显示并提示清除失败，不得只清空内存列表
  验证: Rust 测试 clearHistoryIsAtomicOnStorageFailure
```

### FR-006：下载

- **描述**：网页触发的 HTTP(S) 下载必须由浏览器进程接管，通过 Android 存储接口写入用户可访问位置；下载页必须显示文件名、来源、进度、完成/失败状态，并提供取消、重试和打开操作。
- **优先级**：必须
- **来源**：用户明确要求首期包含下载

**验收场景**：

```text
场景: 完成下载
  假设 服务端返回可下载响应且用户授予所需文件创建访问
  当 用户确认文件名并开始下载
  那么 数据由 browser 进程写入系统授予的 URI，下载页和系统通知显示完成，点击打开使用 ACTION_VIEW
  验证: Android 集成测试 downloadViaDocumentUriCompletesAndOpens

场景: 权限拒绝或目标不可写
  假设 用户取消系统文件选择器或目标 URI 写入失败
  当 下载流程继续
  那么 下载标记为取消或失败，不创建伪完成记录，不向 renderer 暴露文件路径，并允许重试
  验证: Android 集成测试 deniedDownloadDestinationDoesNotReportSuccess
```

### FR-007：Android 生命周期、系统返回与状态保存

- **描述**：Activity 重建、旋转、分屏、退后台和恢复必须保持业务状态一致；系统返回必须依次执行“关闭临时界面 → 页面后退 → 请求退出”。
- **优先级**：必须
- **来源**：用户接受手机竖横屏与恢复建议；§1.3 阶段 6

**验收场景**：

```text
场景: 旋转并返回页面
  假设 页面已滚动且地址栏没有未提交编辑
  当 设备旋转导致 Activity 重建
  那么 活跃标签、URL、页面滚动位置和加载状态保持，Surface 使用新尺寸重新注册
  验证: Android 测试 rotationRebindsSurfaceAndPreservesActiveTab

场景: renderer 忙碌时应用退后台
  假设 页面正在加载或执行脚本
  当 App 进入后台且系统稍后回收进程
  那么 会话先持久化；恢复时不得重放未确认的下载或表单提交，相关标签进入可重载状态
  验证: ADB 场景 backgroundKillRestoreDoesNotReplaySideEffects
```

### FR-008：固定多进程与故障恢复

- **描述**：每个活动页面必须关联独立 renderer 实例；compositor 与 image-decoder 必须始终运行在独立 Service 进程。辅助进程退出时，浏览器必须识别角色和受影响标签，并提供有界重启，不得退回进程内执行。
- **优先级**：必须
- **来源**：用户明确确认必须物理多进程，并接受推荐恢复行为

**验收场景**：

```text
场景: 验证进程隔离
  假设 App 打开两个活动标签并完成首帧
  当 测试读取系统进程列表与绑定服务状态
  那么 browser、两个 renderer、compositor、image-decoder 具有不同 PID，renderer 使用 isolated UID，页面 JS 不存在于 browser 进程
  验证: ADB 安全测试 assertAndroidProcessTopology

场景: renderer 被系统终止
  假设 一个后台标签拥有独立 renderer
  当 测试终止该 renderer Service 进程
  那么 仅该标签显示“页面已停止”和重新加载按钮；点击后创建新 renderer 并恢复 URL/滚动恢复点，不启用进程内 renderer
  验证: ADB 场景 killRendererIsTabScopedAndReloadable
```

### FR-009：Android 系统集成与权限

- **描述**：App 必须通过 Android 原生能力完成网络状态、剪贴板、文件选择/打开、下载通知、IME 和应用链接输入；所有跨应用 Intent、URI 和外部文本必须在 browser 信任边界校验。
- **优先级**：必须
- **来源**：Kotlin/Compose 薄宿主方案；Android 安全边界

**验收场景**：

```text
场景: 从外部分享链接到 ZeroWeb
  假设 其他 App 发送 ACTION_VIEW HTTPS Intent
  当 ZeroWeb 接收该 Intent
  那么 browser 校验 scheme 后在新标签打开 URL，并保持来源 App 无法注入内部协议或文件路径
  验证: Android 测试 externalHttpsIntentOpensNewTab

场景: 恶意外部 Intent
  假设 Intent 包含 file URI、未知 scheme、超长 URL 或非 content URI 的下载目标
  当 ZeroWeb 接收 Intent
  那么 请求被拒绝并显示安全错误，不执行导航、不读取任意本地文件
  验证: Android 安全测试 rejectsUntrustedIntentPayloads
```

### FR-010：可重复构建与 APK 交付

- **描述**：仓库必须提供单一 Android 构建入口：Release APK 仅含 `arm64-v8a`，Debug APK 额外含 `x86_64` 以供本机模拟器验证；构建必须先生成 Rust `.so` 再由 Gradle 打包，并能在 Windows PowerShell 与 CI Linux 主机执行。
- **优先级**：必须
- **来源**：用户接受 arm64 与侧载 APK；仓库跨平台工作流

**验收场景**：

```text
场景: 构建并安装 Debug APK
  假设 Android SDK 36、NDK r30、JDK、Rust Android target 和 cargo-ndk 已安装
  当 执行项目 Android build 入口
  那么 生成可通过 adb 安装并启动的本机模拟器匹配 ABI Debug APK，Release APK 只包含 arm64-v8a，二者均包含 browser、renderer、compositor、image-decoder 所需 native library
  验证: make android-apk && make android-install-smoke

场景: 缺失 V8 Android archive 或工具链
  假设 构建环境缺少指定 Android V8 archive、SDK、NDK 或 Rust target
  当 执行 Android build 入口
  那么 构建在打包前失败并指出缺失项及修复命令，不得生成缺少 renderer 的 APK
  验证: scripts/android/preflight negative cases
```

---

## 4. 非功能需求

### NFR-001：启动与交互性能

- **描述**：在基准 arm64 真机（Android 16、8 GB RAM、发布构建）上，冷启动到 chrome 可交互的 p95 必须 ≤ 3 秒；本地确定性测试页从提交导航到首个非空页面帧的 p95 必须 ≤ 2 秒；页面稳定后触摸输入到对应呈现帧的 p95 必须 ≤ 50 ms。
- **测量标准**：各执行 20 次，使用 Android Macrobenchmark/Perfetto 和帧 ID 时间戳，报告 p50/p95；首帧测试不包含公网时间。
- **优先级**：必须

### NFR-002：内存与后台资源

- **描述**：在上述基准设备打开 5 个确定性测试标签后，App 全部进程 PSS 峰值必须 ≤ 1.2 GB；进入后台 30 秒后不得保持无业务工作的忙轮询，CPU 使用率中位数必须 < 1%。超过 renderer 驻留上限的标签必须进入可恢复挂起态。
- **测量标准**：`dumpsys meminfo`、Perfetto 60 秒 trace；固定 fixture 和相同 Release APK 重复 5 次。
- **优先级**：必须

### NFR-003：可靠性与故障隔离

- **描述**：连续执行 100 次新建/加载/关闭标签循环不得导致 browser 进程崩溃；单个 renderer 或 image-decoder 崩溃不得导致其他标签、下载或 profile 数据丢失；compositor 重启必须在 5 秒内恢复 Surface 或显示明确的全局重试界面。
- **测量标准**：ADB chaos 测试注入每种子进程终止并校验 PID、标签状态与持久化校验和。
- **优先级**：必须

### NFR-004：安全隔离

- **描述**：renderer 与 image-decoder 必须运行于 `isolatedProcess` Service；所有 Service 默认 `exported=false`；renderer 不得直接持有网络、任意文件或下载目标 FD；browser 必须验证来自 renderer、Intent、Binder 和 JNI 的长度、枚举、URL scheme、revision 与对象 ID。
- **测量标准**：manifest 静态扫描、UID/PID ADB 断言、恶意 IPC/Intent 测试、renderer 内网络和文件访问负测试。
- **优先级**：必须

### NFR-005：数据完整性与隐私

- **描述**：标签会话、书签、历史和下载元数据必须使用临时文件 + flush/sync + 原子替换提交；损坏一种数据不得删除其他数据。日志、通知和崩溃信息不得包含表单值、页面正文、认证 header、Cookie 或完整本地 URI grant。
- **测量标准**：逐类 fault-injection、进程终止写入测试和日志敏感字段扫描。
- **优先级**：必须

### NFR-006：Android 兼容性与自适应

- **描述**：Release APK 的运行范围必须覆盖 API 26～36 的 arm64-v8a 设备；主 Activity 不得锁定方向或固定宽高；在 320 dp～840 dp 可用宽度、字体缩放 1.0～2.0 和竖/横屏下不得出现不可达的主导航操作。
- **测量标准**：API 26、30、36 三档设备/镜像测试；Compose screenshot/semantics 测试覆盖窗口与字体矩阵。
- **优先级**：必须

### NFR-007：无障碍

- **描述**：所有 chrome 交互控件必须具有可本地化的 content description、角色、状态与至少 48×48 dp 触摸目标；TalkBack 必须能完成地址输入、页面后退、标签切换、书签打开和下载状态读取。网页无障碍树桥接不在本期，但页面输入焦点与 IME 不得被 chrome 抢占。
- **测量标准**：Compose semantics 测试、Accessibility Scanner 和真机 TalkBack 手工场景。
- **优先级**：chrome 无障碍必须；网页无障碍树后续

### NFR-008：构建可重复性与许可证

- **描述**：Gradle、AGP、AGP 内建 Kotlin、Compose、NDK、cargo-ndk 与 Rust crate 版本必须精确固定，禁止动态版本；新增依赖只能使用项目允许的宽松许可证；同一 commit 的两次 Release native 构建必须产生相同依赖图和相同 ABI 文件清单。
- **测量标准**：dependency lock/verification、许可证扫描、APK native library 清单 diff。
- **优先级**：必须

### NFR-009：可观测性

- **描述**：每条跨进程请求必须携带 role、instance ID、tab ID 或 surface ID、navigation epoch 和 request ID 中适用的字段；辅助进程退出必须记录退出角色、原因和有界 stderr/native crash 摘要；Release 日志默认不得输出逐帧信息。
- **测量标准**：故障注入日志断言和 Release 日志级别测试。
- **优先级**：必须

---

## 5. 接口需求

### IF-001：Android 浏览器 chrome

- **类型**：UI
- **规格**：单 Activity、Compose 导航；页面包括浏览页、标签概览、书签、历史、下载。浏览页包含顶部地址栏/安全状态、返回/前进/刷新、标签计数和菜单；网页 Surface 填充剩余安全区域。
- **错误处理**：Rust facade 未连接时显示启动错误和重试；单标签 renderer 崩溃显示标签内恢复卡；compositor 不可用显示全局恢复层；列表为空显示对应空状态。
- **默认动作**：首次启动打开新标签；外部 HTTPS Intent 新建标签；下载默认弹出系统文件创建器；系统返回顺序见 FR-007。
- **交叉引用**：FR-001～FR-007、IF-002～IF-006。

```text
┌──────────────────────────────────┐
│ ‹  ›  [ 锁  地址或搜索词      ] ↻ │
├──────────────────────────────────┤
│                                  │
│       ZeroWeb 页面 Surface       │
│                                  │
├──────────────────────────────────┤
│ 主页/新标签       标签数       ⋮ │
└──────────────────────────────────┘

标签概览 / 书签 / 历史 / 下载：
┌──────────────────────────────────┐
│ ‹  标题                    搜索  │
├──────────────────────────────────┤
│ 卡片或列表；空态；错误/重试状态 │
└──────────────────────────────────┘
```

关键用户可见文案采用 Android string resource，首期至少提供中文和英文；文档中的中文错误语义不是硬编码字符串。

### IF-002：Compose ↔ Rust 业务 facade

- **类型**：JNI API
- **规格**：
  - `nativeCreate(profileDir, cacheDir, localeTag) -> BrowserHandle`
  - `nativeDispatch(handle, actionEnvelopeBytes) -> DispatchResultBytes`
  - `nativeSnapshot(handle, afterRevision) -> StateSnapshotBytes`
  - `nativeDestroy(handle)`
  - Action 和 Snapshot 使用带 `schemaVersion`、`revision` 的 UTF-8 JSON envelope；Rust `serde` 类型为权威 schema，Kotlin DTO 必须由契约测试保持一致。
  - `BrowserAction` 至少覆盖导航、标签、书签、历史、下载和恢复操作；`BrowserStateSnapshot` 只携带 chrome 所需状态，不携带页面像素或正文。
  - 每个 handle 串行处理 Action；耗时 I/O 异步执行，结果以 revision 推进通知 Compose。
- **错误处理**：无效 handle、未知版本、超长 payload、过期 revision、未知对象 ID 返回结构化错误，不 panic、不部分提交。
- **默认动作**：未知 Action 必须拒绝；不得静默忽略或猜测降级。
- **交叉引用**：FR-003～FR-007、NFR-004～NFR-005。

### IF-003：页面 Surface、触摸与 IME

- **类型**：JNI + Android Surface
- **规格**：
  - `nativeAttachSurface(handle, surface, widthPx, heightPx, density, generation)` 将 Android `Surface` 转换为受生命周期约束的 `ANativeWindow`，交给 compositor Service 注册。
  - `nativeDetachSurface(handle, generation)` 必须在 Android Surface 销毁后停止对旧 generation 呈现。
  - 高频 pointer/scroll/IME 事件使用固定字段 JNI 方法或紧凑二进制批次，不经过 IF-002 JSON facade。
  - 坐标以物理像素和显式 density 传递；Rust 在唯一边界完成 logical/physical 转换。
- **错误处理**：0 尺寸、失效 generation、已释放 Surface 或非法 pointer 序列被拒绝；丢失 Surface 时页面管线可继续但不得呈现到旧窗口。
- **默认动作**：新 Surface generation 替换旧 generation；旧 generation 的迟到帧丢弃。
- **交叉引用**：FR-002、FR-007、IF-004。

### IF-004：Android 子进程 Service 与控制 IPC

- **类型**：AIDL/Binder + ParcelFileDescriptor
- **规格**：
  - browser 进程通过非导出的绑定 Service 创建角色实例：renderer slot、compositor、image-decoder。
- API 26 兼容实现预声明有限 renderer Service slots；首期最多 8 个驻留 renderer。标签数量不设 8 个上限，超额后台标签按 LRU 挂起并释放 renderer。`isolatedProcess` 角色不声明私有 `android:process` 名称，由 Android 分配独立进程。
  - renderer 与 image-decoder Service 声明 `isolatedProcess=true`；compositor 使用独立 `:compositor` 进程并与 browser 同应用 UID，以满足 GPU/Surface 平台访问。
  - Binder 仅负责 bootstrap、生命周期、health 和传递 `ParcelFileDescriptor`；`zero-protocol` 消息经 socket pair/流式 transport 传输。
  - 页面像素、PaintSnapshot 大对象和下载正文不得作为 Binder transaction payload。
- **错误处理**：slot 耗尽触发 LRU 挂起；bind 超时或 binder death 转换为明确角色故障；同一实例最多自动重启 2 次/60 秒，继续失败则等待用户操作。
- **默认动作**：不得切换到进程内角色；compositor 不健康时停止提交新帧并显示恢复层。
- **交叉引用**：FR-008、NFR-003～NFR-004、IF-003。

### IF-005：Android profile 与持久化

- **类型**：Rust 存储接口
- **规格**：所有 browser-shell 持久化 API 必须接受宿主提供的显式 `ProfilePaths`，由 Android `filesDir`/`cacheDir` 派生；禁止在 Android 使用桌面默认目录探测。profile 分别保存 session、bookmarks、history、downloads 和 settings，文件 schema 带版本。
- **错误处理**：逐文件校验、损坏隔离、原子提交和前一版本备份；迁移失败仅禁用对应数据集，不清空整个 profile。
- **默认动作**：不存在文件创建默认状态；版本高于当前实现时只读保护并提示升级，不覆盖文件。
- **交叉引用**：FR-001、FR-004～FR-006、NFR-005。

### IF-006：下载与系统文件访问

- **类型**：Android Activity Result + content URI + browser 下载服务
- **规格**：使用系统 `ACTION_CREATE_DOCUMENT` 获取目标 content URI，browser 进程持有临时 URI grant 并流式写入；下载网络请求和校验由 browser 进程负责。通知权限被允许时显示进度通知；完成后用带 grant flags 的 `ACTION_VIEW` 打开。
- **错误处理**：用户取消、URI grant 失效、空间不足、网络断开、校验或 rename/close 失败分别进入取消/失败状态；部分文件保留或删除规则必须由下载记录明确，不报告伪完成。
- **默认动作**：首期每个新下载都请求用户确认目标；通知权限拒绝时继续下载并仅在下载页显示状态。
- **交叉引用**：FR-006、NFR-004～NFR-005。

### IF-007：外部 Intent 与系统能力

- **类型**：Android 系统集成
- **规格**：接受 launcher Intent 和显式/经系统解析的 HTTP(S) `ACTION_VIEW`；剪贴板、分享、网络状态、IME 与打开文件均通过 Kotlin platform service 执行。Rust 仅接收规范化、有限长度的结果。
- **错误处理**：拒绝 `file:`、`javascript:`、未知 scheme、非预期 extras、不可解析 URI 和超限 payload；外部 Activity 不存在时显示可恢复错误。
- **默认动作**：外部 HTTP(S) URL 在新标签打开；App 已在前台时不得覆盖地址栏未提交编辑。
- **交叉引用**：FR-009、NFR-004。

### IF-008：Android 构建与打包

- **类型**：构建接口
- **规格**：Gradle 为 APK 权威打包器；固定 AGP 9.2、Gradle 9.4.1、AGP 内建 Kotlin、Android SDK 36 和 NDK r30。Gradle 任务调用固定版本 cargo-ndk，为 `aarch64-linux-android` 构建 Rust `cdylib` 并写入生成目录，再合入 APK。V8 Android archive 必须由显式校验和锁定的构建输入提供。
- **错误处理**：preflight 在编译前检查 JDK/SDK/NDK/Rust target/cargo-ndk/V8 archive；任一不匹配即失败。Gradle 禁止从旧 `jniLibs` 静默打包残留 `.so`。
- **默认动作**：`make android-apk` 构建 Debug；Release 需要显式任务且不生成或提交密钥。
- **交叉引用**：FR-010、NFR-008。

---

## 6. 约束、决策与假设

### 6.1 必须约束（Must）

- Android application ID 必须为 `com.leizm.zeroweb`。
- Android 应用目录必须为 `apps/android-browser/`，Cargo package 必须为 `zero-android-browser`。
- 网页必须由 ZeroWeb renderer 渲染；不得用 Android System WebView、Custom Tabs 或远程浏览器代替。
- renderer、compositor、image-decoder 必须保持物理独立进程；任何角色不可因启动失败回退到 browser 进程。
- renderer 必须经 browser 代理网络、存储、下载和权限能力。
- `zero-browser-shell` + Android facade 必须是 chrome 业务状态唯一事实源；Kotlin DTO 只是快照。
- Surface detach、Activity 销毁、Service death 和 native handle 销毁必须幂等。
- Android profile 必须使用宿主显式路径，持久化变更必须防止断电/进程终止导致原文件丢失。
- 所有用户可见 Kotlin 文案必须进入 Android string resources；Compose 控件必须包含无障碍语义。
- Android 构建必须走项目受控入口并通过工具链 preflight，不得依赖开发机绝对路径。

### 6.2 禁止约束（Must Not）

- 禁止复制或 fork DOM、CSS、布局、脚本和绘制管线形成 Android 专用内核。
- 禁止把 `apps/browser/` 的桌面窗口、winit 事件循环或自绘桌面 chrome 整体搬入 Android App。
- 禁止在 Compose `ViewModel`、数据库或 saved state 中建立第二套完整标签/书签/历史/下载模型。
- 禁止通过 Binder 传输完整帧、PaintSnapshot、下载正文或无界 JSON。
- 禁止导出 renderer/compositor/image-decoder Service，禁止允许外部 App 直接绑定。
- 禁止为方便下载申请 `MANAGE_EXTERNAL_STORAGE` 或传统全盘读写权限。
- 禁止把 API key、签名文件、keystore 密码、SDK/NDK 绝对路径提交到仓库。
- 禁止动态 Gradle/Maven/Cargo 依赖版本。
- 禁止修改与 Android 接入无关的 DOM/CSS/layout/WPT 行为。
- 禁止以 arm64 模拟器截图替代至少一台 arm64 真机的触摸、IME、Surface 和进程验收。

### 6.3 已定决策

| 决策 | 结果 | 理由 |
|---|---|---|
| UI 技术 | Kotlin + Jetpack Compose | 原生生命周期、IME、权限、无障碍和列表组件成熟；不阻塞于尚未落地的通用 UI SDK |
| 内核边界 | Rust `cdylib` + JNI | 复用 ZeroWeb Rust 核心，Android UI 保持薄宿主 |
| 业务状态 | Rust `zero-browser-shell` facade 为唯一事实源 | 防止 Kotlin/Rust 状态分叉 |
| 进程启动 | Android bound Service + Binder bootstrap | Android 进程必须由 Activity Manager 创建 |
| 数据通道 | Binder 控制面 + socket/FD 数据面 | 保持 `zero-protocol` 语义并避开 Binder 大对象限制 |
| renderer 策略 | 8 个预声明 isolated Service slots；进程名由 Android 分配；超额标签 LRU 挂起 | 兼容 API 26 并限制移动端内存 |
| compositor 策略 | 独立同 UID Service，持有 Android Surface | GPU/Surface 可用性优先；角色仍物理隔离 |
| image-decoder 策略 | 单独 isolated Service | 隔离不可信图片解码，输入通过受限 FD/消息 |
| 下载目标 | `ACTION_CREATE_DOCUMENT` content URI | 不申请广泛存储权限，API 26 起一致 |
| ABI | Release 仅 arm64-v8a；Debug 额外 x86_64 | 保持手机发布边界，同时满足本机 x86_64 模拟器验证 |
| API 范围 | minSdk 26，compile/target 36 | 覆盖现代 64 位设备并对齐当前稳定 Android API |
| V8 构建 | 受控脚本从 `rusty_v8` 对应源码交叉编译并缓存校验和产物 | 官方 Android 路径需要交叉编译；确保版本与仓库 crate 对齐 |

### 6.4 技术约束

- Rust edition 2024，MSRV 1.85；Android target 为 `aarch64-linux-android`。
- Android NDK r30，Android SDK 36；构建环境 JDK 版本以 AGP 9.2 官方兼容矩阵为准并在 preflight 固定。
- wgpu 30 和现有 `zero-render-foundation` 为 GPU/CPU 绘制来源；Android Surface 接入不得引入第二个渲染后端。
- `apps/renderer`、`apps/compositor`、`apps/image-decoder` 的角色主循环必须抽为可复用 library entry；桌面 binary `main` 与 Android JNI entry 调用同一主循环。
- Android 子进程 transport 必须实现 `zero-protocol` transport trait/等价共享接口，不得复制消息枚举。
- Compose 只消费 revisioned snapshot；高频触摸、帧和 IME 走专用接口。
- Android 16 target 下必须适配可变窗口，不得依赖禁止旋转或禁止 resize。

### 6.5 假设

| 假设 | 状态 | 处理 |
|---|---|---|
| rusty_v8 当前版本可从 x86_64 主机交叉编译到 aarch64 Android | 已由上游文档验证；本仓未验证 | M0 首个技术 spike 实际构建并运行最小 V8 isolate |
| wgpu 30 可从 Android Surface 创建 Vulkan/OpenGL ES 支持的 present surface | 上游平台支持已知；本仓未验证 | M1 真机 Surface spike，失败时只调整 adapter，不更换内核 |
| compositor 使用同应用 UID 独立进程可稳定访问目标设备 GPU/Surface | 待实机验证 | M1 以至少一台 arm64 真机验证；不允许因此合并进程 |
| `zero-browser-shell` 现有模型可作为 Android 业务模型基础 | 已验证具备标签/书签/历史/下载类型；持久化不完整 | M1 补显式路径、历史/下载持久化和 facade，不重写模型 |
| 用户可提供至少一台 API 26～36 范围内的 arm64 真机用于最终验收 | 用户已接受真机范围 | M5 记录机型、API 和证据，不写入产品代码 |

### 6.5A 实现来源说明

| 能力/行为 | 来源类型 | 具体来源 | 备注 |
|---|---|---|---|
| 标签、书签、历史、下载状态 | 复用现有模块 | `crates/browser-shell` | 新增显式 profile 与 Android facade，不复制模型 |
| 页面运行时 | 复用现有模块 | `apps/renderer`、`crates/page-runtime`、`crates/engine` | 抽共享角色入口 |
| 合成和光栅化 | 复用现有模块 | `apps/compositor`、`crates/render-foundation`、wgpu 30 | 新增 Android Surface present adapter |
| 图片解码隔离 | 复用现有模块 | `apps/image-decoder` | 抽共享角色入口 |
| 消息 schema | 复用现有模块 | `crates/protocol` | 新增 Android socket/FD transport 与 Service launcher |
| JNI | 新增宽松许可依赖 | Rust `jni` crate + Android JNI | 仅 Android target 启用，版本精确固定 |
| Android native window | 系统能力 + 宽松许可封装 | NDK `ANativeWindow`，必要时使用 `ndk` crate | 所有权与 generation 必须测试 |
| Activity/Compose/lifecycle | 官方 Android 依赖 | AndroidX Activity、Compose、Lifecycle、Navigation | 版本 catalog 精确固定 |
| Service IPC | 系统能力 | AIDL、Binder、`ParcelFileDescriptor` socket pair | Binder 只作控制面 |
| Rust Android 构建 | 新增工具 | cargo-ndk + NDK r30 | cargo-ndk 为 MIT/Apache-2.0，版本固定 |
| V8 Android archive | 上游源码构建 | 与 workspace `v8` crate 对应的 rusty_v8 source | CI/本地缓存产物必须记录 SHA-256 |
| 下载文件访问 | 系统能力 | Storage Access Framework content URI | 不请求全盘权限 |
| 性能与进程证据 | 系统工具 | Macrobenchmark、Perfetto、ADB、dumpsys | 固定 fixture 和报告格式 |

### 6.6 代码变更边界

#### 允许修改

- `apps/android-browser/**` — 新 Android Gradle 工程、Kotlin UI、AIDL、manifest、Rust JNI facade 和测试。
- `apps/renderer/**` — 仅抽共享角色入口和 Android role adapter。
- `apps/compositor/**` — 仅抽共享角色入口和 Android Surface/present adapter。
- `apps/image-decoder/**` — 仅抽共享角色入口和 Android role adapter。
- `apps/browser/**` — 仅为共享 browser-host/page coordinator 抽取所需的最小修改；不得改变桌面产品行为。
- `crates/browser-shell/**` — 显式 profile、原子持久化、Action/Snapshot 所需公共 API。
- `crates/protocol/**` — Android transport、角色启动抽象及共享 IPC 契约。
- `crates/render-foundation/**`、`crates/host-runtime/**` — Android Surface/字体/平台能力所需最小 adapter；桌面 API 不破坏。
- `Cargo.toml`、`Cargo.lock`、`Makefile`、`scripts/android/**`、`.github/workflows/**` — workspace、构建和 CI。
- `tests/integration/**`、`docs/specs/**`、`docs/learnings/**` — 测试、设计和实施中形成的工程经验。

#### 禁止修改

- 与 Android 接入无关的 `crates/dom/**`、`crates/css-parser/**`、`crates/style-system/**`、`crates/layout-engine/**` 行为。
- 为 Android 单独 fork 的 JS DOM shim、CSS 兼容层或页面协议消息。
- 未经单独 Spec 覆盖的桌面 UI 重构和通用 `ui/**` SDK 实现。
- 现有多进程环境开关或进程内 browser fallback 的恢复。

### 6.7 执行技能提示

| 范围 / 触发条件 | Skill | 模式 | 原因 |
|---|---|---|---|
| 本文档修订与实施合约 | `lei-spec-rfc` | required | 保持需求、实现与验收可追溯 |
| Rust/IPC/渲染代码修改 | `zeroweb-guidelines` | required | 遵循 ZeroWeb 跨子系统不变式 |
| Android 真机最终验收 | `lei-product-acceptance` | required | 必须用真实 App、触摸、IME 和故障场景验收 |
| 真实网站行为对比 | `zeroweb-browser-chrome-parity` | preferred | 对比 Chrome Android 的点击、状态、几何和帧 |

---

## 7. 优先级、里程碑与实施交接

### 7.1 需求优先级映射

| ID | 需求 | 优先级 | 里程碑 |
|---|---|---|---|
| FR-001 | 安装、启动与会话恢复 | 必须 | M0/M2 |
| FR-002 | 地址栏、导航与页面交互 | 必须 | M3 |
| FR-003 | 多标签操作 | 必须 | M2/M4 |
| FR-004 | 书签 | 必须 | M2/M4 |
| FR-005 | 历史记录 | 必须 | M2/M4 |
| FR-006 | 下载 | 必须 | M4 |
| FR-007 | 生命周期、系统返回与状态保存 | 必须 | M2/M3 |
| FR-008 | 固定多进程与故障恢复 | 必须 | M0/M1/M3 |
| FR-009 | Android 系统集成与权限 | 必须 | M3/M4 |
| FR-010 | 可重复构建与 APK 交付 | 必须 | M0/M5 |

### 7.2 建议里程碑

- **M0：工具链、APK 与进程骨架**
  - 建立 Gradle/Compose 工程、application ID、版本锁和 preflight。
  - 构建 arm64 与 x86_64 Debug Rust `cdylib`，加载 browser native library；Release 只打包 arm64。
  - 声明 renderer slots、compositor、image-decoder Service，完成 Binder/socket bootstrap。
  - 从源码交叉编译对应 V8，运行最小 isolate。
  - 门禁：Debug APK 可安装启动；四类角色 PID/UID 符合 IF-004；无网页功能。

- **M1：共享角色入口与 Android Surface spike**
  - 将 renderer/compositor/image-decoder 主循环抽为 library entry，桌面 binary 调用同一路径。
  - 实现 Android `zero-protocol` transport。
  - compositor 接收 Surface，完成清屏/测试图案 present、resize、detach/reattach。
  - 门禁：桌面测试保持通过；真机旋转 100 次无旧 Surface 呈现或 native crash。

- **M2：Rust facade、业务状态与持久化**
  - 定义 `BrowserAction`、`BrowserStateSnapshot`、revision 和错误 schema。
  - `BrowserShell` 接入显式 `ProfilePaths`，补齐 session/bookmarks/history/downloads 原子持久化。
  - Compose 实现浏览页 chrome、标签概览、书签/历史/下载空态与错误态。
  - 门禁：无页面内核时所有业务模型、Activity 重建和损坏数据恢复测试通过。

- **M3：真实页面闭环**
  - browser host 创建 renderer、导航并连接 compositor。
  - 接入页面 Surface、viewport/density、触摸、滚动、焦点、IME、标题/URL/加载状态回写。
  - 实现 renderer/compositor death 处理与标签级重载。
  - 门禁：真机真实 HTTPS 页面可加载、点击、输入、滚动；进程终止场景通过。

- **M4：完整首期功能**
  - 完成多标签缩略图/挂起策略、书签、历史、下载 SAF/通知、外部 Intent、双语和 chrome 无障碍。
  - 完成系统返回优先级、前后台恢复和下载不重放。
  - 门禁：FR-001～FR-009 全部场景通过。

- **M5：质量与交付**
  - 完成 API 26/30/36、竖横屏、字体缩放、性能、内存、chaos、安全负测试。
  - 生成 Debug/Release APK、依赖/许可证清单和验收报告。
  - 门禁：FR-010、NFR-001～NFR-009 的全部必须指标通过。

### 7.3 实施交接（Implementation Handoff）

#### 文件/模块清单

| 路径/模块 | 动作 | 目的 | 风险/注意事项 |
|---|---|---|---|
| `apps/android-browser/settings.gradle.kts` | 新增 | Android 工程与仓库配置 | 禁止动态仓库/版本 |
| `apps/android-browser/build.gradle.kts` | 新增 | AGP/Kotlin/Compose 顶层配置 | 与 Gradle wrapper 固定兼容 |
| `apps/android-browser/gradle/libs.versions.toml` | 新增 | Android 依赖单一版本源 | 不存本机路径 |
| `apps/android-browser/app/**` | 新增 | manifest、Activity、Compose、AIDL、资源和 Android 测试 | Kotlin 不成为业务状态源 |
| `apps/android-browser/rust/Cargo.toml` | 新增 | `zero-android-browser` cdylib | target-specific Android 依赖 |
| `apps/android-browser/rust/src/facade/**` | 新增 | Action/Snapshot、handle、profile | JNI 输入视为不可信 |
| `apps/android-browser/rust/src/platform/**` | 新增 | JNI、Surface、Service bootstrap | 所有权/线程/generation 高风险 |
| `apps/android-browser/rust/src/roles/**` | 新增 | 各 Service 加载的 native role entry | 必须调用共享角色主循环 |
| `apps/renderer/src/lib.rs`、`main.rs` | 修改 | 暴露并复用 renderer role entry | 不改变桌面协议和 V8 所有权 |
| `apps/compositor/src/lib.rs`、`main.rs` | 新增/修改 | 暴露共享 compositor role entry | 现有 main 需机械瘦身 |
| `apps/compositor/src/android_present.rs` | 新增 | `ANativeWindow`/wgpu Surface present | Surface 生命周期与 GPU 恢复 |
| `apps/image-decoder/src/lib.rs`、`main.rs` | 新增/修改 | 暴露共享 decoder role entry | decoder 仍不获得文件路径 |
| `crates/protocol/src/android/**` | 新增 | socket/FD transport、bootstrap schema | Binder payload 有界，消息 schema 单一 |
| `crates/browser-shell/src/profile.rs` | 新增 | 显式 profile 路径和原子文件策略 | 桌面默认路径继续由宿主传入 |
| `crates/browser-shell/src/android_facade.rs` 或 app-local facade | 新增 | 业务 Action/Snapshot | 若仅 Android 使用，优先 app-local |
| `crates/render-foundation/**` | 最小修改 | Android Surface/字体接入 | 不 fork renderer；按已有 backend 扩展 |
| `scripts/android/**` | 新增 | preflight、V8 build/cache、ADB 验收 | 墙钟/内存限制，校验工具版本 |
| `Makefile` | 修改 | `android-apk`、`android-test`、`android-install-smoke` | Windows/Linux 同一语义 |
| `.github/workflows/android.yml` | 新增 | arm64 Release、x86_64 Debug 与 JVM/Rust 测试 | V8 cache key 含源码和配置 SHA |

#### 职责映射

| 模块 | 职责 | 依赖/被依赖 | 验证方式 |
|---|---|---|---|
| Compose chrome | 展示 Snapshot、提交 Action、调用系统 UI | 依赖 JNI facade，不依赖 renderer internals | Compose unit/instrumentation |
| Rust Android facade | 业务状态、revision、异步结果、profile | 依赖 browser-shell/browser host | Rust unit + JNI contract |
| Android process launcher | 绑定 Service、分配 slot、death recipient | 依赖 AIDL/protocol bootstrap | Robolectric/ADB process tests |
| Renderer role | 页面运行时、脚本、输入和 PaintSnapshot | 依赖现有 renderer 共享入口 | renderer tests + live page |
| Compositor role | 合成、Surface present、resize/recovery | 依赖 render-foundation/Surface | present fixture + rotation chaos |
| Image decoder role | 隔离解码 | 依赖现有 decoder 共享入口 | malformed image corpus |
| Profile store | 原子保存、版本、损坏隔离 | 被 Android facade 和桌面宿主使用 | fault-injection tests |
| Download bridge | 网络流 → content URI、通知、打开 | browser host + Kotlin platform service | fake server + SAF instrumentation |

#### 新能力来源对照

| 能力/需求 | 实现承载位置 | 来源类型 | 验证方式 |
|---|---|---|---|
| Android 多进程启动 | Kotlin Service + `crates/protocol/src/android` | Android 系统能力 + 复用协议 | PID/UID/Binder death ADB 测试 |
| Compose/Rust 状态桥 | app Rust facade + Kotlin DTO | 仓内自实现，serde_json 复用 | schema golden/unknown-version tests |
| Android Surface present | compositor Android adapter | NDK + wgpu 复用 | 真机像素/旋转/generation tests |
| 原子 profile | browser-shell profile store | 仓内自实现 + 标准文件 API | kill/fault-injection |
| SAF 下载 | Kotlin platform service + browser download host | Android 系统能力 | instrumentation + fake HTTP server |
| Android V8 | scripts/android V8 builder | rusty_v8 上游源码 | minimal isolate + checksum manifest |

#### 推荐修改顺序

1. 建立工具链 preflight、Gradle wrapper、空 Activity 与空 Rust cdylib；先验证 Windows/Linux 构建入口。
2. 抽 renderer/compositor/image-decoder 共享 role entry，并用现有桌面测试证明行为不变。
3. 建立 AIDL/Service/socket bootstrap，验证固定多进程 topology 和 failure propagation。
4. 完成 compositor Android Surface spike；验证 attach/resize/detach 后再接页面管线。
5. 完成 profile、Action/Snapshot facade 和 Compose chrome；此时仍可用 fake renderer 验证 UI。
6. 接入真实 renderer 导航、输入、IME、页面帧与 crash recovery。
7. 完成书签、历史、下载、Intent、无障碍和双语。
8. 执行全量质量门禁、真机验收和 APK 交付。

#### 首批提交建议

| 提交/批次 | 范围 | 预期结果 | 验证 |
|---|---|---|---|
| Commit 1 | Spec/RFC | 架构、边界和验收获确认 | Spec Lint、`git diff --check` |
| Commit 2 | Android build skeleton | 空 APK 加载 Rust browser library | `make android-apk`、ADB launch |
| Commit 3 | Shared role entries | 三个桌面 helper 可由 lib entry 启动 | fmt/clippy/workspace tests |
| Commit 4 | Service topology | 四类 Android 进程和 socket handshake | `assertAndroidProcessTopology` |
| Commit 5 | Surface spike | compositor 真机 present/resize/rebind | rotation/generation smoke |

首个编码步骤是 Commit 2 的空 APK + Rust library；不得把 Compose 全功能页面或真实 renderer 提前混入首批提交。

---

## 8. 技术设计（RFC）

### 8.1 现状分析

#### 当前架构

- `apps/browser` 同时包含桌面 CLI、winit/softbuffer/wgpu 窗口、chrome 绘制、`ProcessTabBackend`、网络/存储代理和 helper 定位/启动。
- `crates/protocol::ProcessManager` 使用 `std::process::Command`、stdio transport 和桌面子进程路径，Android Activity Manager 无法直接采用。
- `apps/renderer` 的生产入口集中在 binary `main.rs`；`lib.rs` 仅暴露少量测试能力。
- `apps/compositor`、`apps/image-decoder` 只有 binary entry，Android Service 无法复用其主循环而不先抽取 library entry。
- `crates/protocol::frame_shm` 的高性能路径主要针对 Linux POSIX shm/dma-buf；非 Linux 路径不等于 Android Surface 直呈现。
- `crates/browser-shell` 已有核心业务对象，但 bookmarks/settings 使用默认桌面路径，history/download/session 的 Android 持久化契约尚未闭合。
- `crates/webview` 是进程内嵌入边界；把它直接放进 Android Activity 会违反用户确认的固定多进程要求。

#### 当前痛点

1. 桌面 App 层混合了可复用 browser-host 逻辑与桌面 UI/进程启动逻辑。
2. Android 角色进程需要 Component/Service 入口，而不是 sibling executable。
3. Android 页面像素必须直接交给 Surface；跨 Binder/Java 复制 RGBA 帧不可接受。
4. Rust 与 Kotlin 之间若暴露可变对象图，会产生线程、生命周期和双状态源问题。
5. 现有默认路径与写入方式不足以保证 Android 进程回收下的数据完整性。

### 8.2 目标架构

```text
┌──────────────────── Android browser process ─────────────────────┐
│ MainActivity / Compose                                           │
│  ├─ Browser chrome + ZeroPageView(SurfaceView/InputConnection)   │
│  ├─ Activity Result / Intent / Notification / Clipboard          │
│  └─ Kotlin DTO ← revisioned snapshot → JNI action                │
│                         │                                        │
│ Rust Android facade     │                                        │
│  ├─ zero-browser-shell (single source of truth)                  │
│  ├─ browser host: network/storage/download/permission broker     │
│  ├─ ProfileStore                                              │
│  └─ AndroidProcessLauncher ── Binder bootstrap + socket FD ──────┼──┐
└──────────────────────────────────────────────────────────────────┘  │
                                                                       │
     ┌──────── isolated renderer slot N ────────┐                      │
     │ shared zero-renderer role entry           │◄─────────────────────┤
     │ DOM/CSS/layout/JS; no direct net/files    │                      │
     └───────────┬───────────────────────────────┘                      │
                 │ PaintSnapshot/control socket                        │
                 ▼                                                     │
     ┌──────── :compositor process ─────────────┐                      │
     │ shared compositor role entry              │◄── Surface parcel ───┤
     │ wgpu/CPU raster → ANativeWindow present   │                      │
     └───────────────────────────────────────────┘                      │
                                                                       │
     ┌──────── isolated image-decoder ──────────┐                      │
     │ shared decoder role entry                 │◄── bounded FD/socket ┘
     └───────────────────────────────────────────┘
```

浏览器进程是唯一可信 broker。Compose 不直接连接 renderer；renderer 不直接请求 Android 权限；compositor 不解释 URL 或业务 Action。进程拓扑与桌面语义一致，但进程创建和 Surface 数据面使用 Android 原生机制。

### 8.3 影响范围分析

| 影响项 | 程度 | 说明 |
|---|---|---|
| Android 新应用与构建系统 | 高 | 全新 APK、Kotlin/Compose、JNI、Service/AIDL |
| 进程启动与 transport | 高 | 新 Android launcher/transport，协议消息语义复用 |
| compositor present | 高 | 新 Surface 生命周期与 Android GPU 路径 |
| renderer/image-decoder entry | 中 | 主循环抽 lib，桌面 main 变薄 |
| browser-shell 持久化 | 中 | 显式 profile、原子写、history/download/session |
| 桌面浏览器 | 中风险/低预期行为影响 | 共享抽取可能触及入口，但产品行为必须保持不变 |
| DOM/CSS/layout/JS 行为 | 低 | 原则上不改；仅由 Android 构建暴露平台编译问题 |
| CI/发布缓存 | 中 | Android SDK/NDK/V8 构建成本高 |

### 8.4 详细设计

#### 8.4.1 Android 工程与 native library 布局

`apps/android-browser/` 是独立 Gradle root，其 `app` module 生成 APK。Rust crate 生成四个逻辑角色可加载的 native library；优先采用单个包含共享代码的 `libzero_android.so` 并由不同 Service 调用不同 JNI entry，避免在 APK 中静态重复 V8/renderer 代码。若 Android linker/进程初始化证明单库角色入口不可行，允许拆为 browser/renderer/compositor/decoder `.so`，但必须通过 APK size 和符号清单验证，且不得改变进程边界。

生成物只能写入 `build/generated/jniLibs/<abi>/` 一类 Gradle build 目录；Release 只打包 arm64-v8a，Debug 可额外打包 x86_64。源目录不保存编译产物。Gradle task 明确依赖 cargo-ndk task，并在打包前清理/重建生成目录。

#### 8.4.2 Rust 角色入口

三个现有 helper 采用相同模式：

```text
desktop main(args, stdio)
  └─ parse desktop bootstrap
     └─ run_<role>(RoleBootstrap { transport, instance, platform })

Android JNI service entry(fd, bootstrap bytes)
  └─ validate Android bootstrap
     └─ run_<role>(RoleBootstrap { transport, instance, platform })
```

`run_<role>` 不读取 Android Activity，不自行定位二进制，不根据环境变量切换进程模式。桌面专属 sandbox/路径逻辑保留在 desktop bootstrap；Android sandbox 由 manifest isolated process 加角色内可行的 syscall hardening 提供。

#### 8.4.3 Service slot 与 tab 生命周期

- manifest 预声明 `RendererService0`～`RendererService7`，每个使用 `isolatedProcess=true`；进程名由 Android 分配，避免与显式 `android:process` 组合触发平台包解析缺陷。
- browser 维护 `RendererSlot { slot, pid, binder, state, tab_id, last_used }`。
- 前台标签必须驻留；后台标签按最近使用时间保留，slot 用尽时挂起最久未使用的后台标签。
- 挂起前保存 URL、navigation stack、scroll restoration point、title/favicon/thumbnail；不承诺恢复 JS heap、未提交表单或媒体状态。
- 恢复挂起/崩溃标签时创建新 renderer 并重新导航；只有用户明确重试时才重放可能具有副作用的导航。
- Service death recipient 必须先使 slot generation 失效，再更新标签状态，防止旧进程迟到消息污染新实例。

状态机：

```text
Unbound → Binding → Running → Suspending → Suspended
              │         │          │
              └─Failed◄─┴─Died─────┘

Failed/Suspended --用户切换或重试--> Binding（新 generation）
```

#### 8.4.4 IPC 分层

Binder/AIDL 只定义：

```text
start(bootstrap, controlFd, dataFd?) -> instanceToken
attachSurface(surface, width, height, density, generation)
detachSurface(generation)
health(instanceToken) -> RoleHealth
shutdown(instanceToken)
```

`bootstrap` 必须 ≤ 64 KiB，包含 schema version、role、instance/slot、随机启动 nonce 和 FD 描述。普通 `zero-protocol` 消息继续走 socket transport；消息长度使用现有上限或更严格 Android 上限。Surface parcel 只传 Android buffer queue handle，不传页面像素。

#### 8.4.5 Action/Snapshot 模型

```text
ActionEnvelope {
  schema_version: u32,
  expected_revision: u64,
  request_id: u64,
  action: BrowserAction
}

BrowserStateSnapshot {
  schema_version: u32,
  revision: u64,
  active_tab_id: TabId?,
  tabs: [TabSummary],
  current_page: PageChromeState?,
  bookmark_state: BookmarkUiState,
  history_state: HistoryUiState,
  download_state: DownloadUiState,
  global_error: UiError?
}
```

Action payload 最大 64 KiB，Snapshot 最大 1 MiB，单 URL 最大 16 KiB，列表查询分页最大 200 项。大列表通过 query/page Action 获取，不把完整历史长期塞入每个 Snapshot。Rust mutation 成功且所需持久化提交成功后才递增 revision；过期 revision 的破坏性 Action 返回 conflict，幂等查询允许读取最新状态。

#### 8.4.6 Compose 与 `ZeroPageView`

- Compose 负责 chrome 和页面容器布局。
- `ZeroPageView` 是嵌入 Compose 的自定义 Android View，拥有 Surface 生命周期、pointer batching、焦点和 `InputConnection`。
- 页面请求文本输入时，Rust 回传 input type、selection 和 surrounding text 的有界快照；View 更新 IME，不把完整 DOM 暴露给 Kotlin。
- chrome 地址栏与网页输入使用不同 focus owner；切换时显式完成/取消 composition，避免字符发往错误目标。
- WindowInsets、状态栏、导航栏和显示 cutout 由 Compose 消费，传给 Rust 的 viewport 仅为真实页面矩形。

#### 8.4.7 Surface 与 compositor

Surface 每次创建递增 generation。compositor 持有当前 generation 的 native window，并在 resize 后重新配置 wgpu Surface。呈现流程验证 `(surface_id, generation, navigation_epoch, frame_id)`；任一旧值都丢弃。detach 先停止 present，再释放 wgpu Surface/native window；API 必须允许 detach 与 binder death 任意顺序发生。

GPU adapter 初始化失败时 compositor 可使用现有 CPU raster 输出到 Android native buffer 的 adapter；该降级只发生在独立 compositor 内，不改变 renderer 或进程拓扑。M1 spike 必须先证明 Vulkan/Surface 路径，再接真实页面。

#### 8.4.8 Profile 与恢复事务

`ProfilePaths` 由 browser native create 参数构造并规范化，所有文件必须留在授予的 app-private 根目录。每类数据独立版本和独立事务：

```text
serialize → write <name>.tmp → flush/sync → validate tmp
          → replace <name>.json atomically → best-effort directory sync
```

会话保存采用 debounce + lifecycle 强制 checkpoint。下载开始/完成与书签变更立即提交；历史允许短 debounce。恢复绝不重放 POST、下载确认或权限对话框，只恢复为需用户确认/重载的状态。

#### 8.4.9 下载状态机

```text
Requested → AwaitingDestination → Running → Completed
     │              │                ├→ PausedByNetwork → Running
     └→ Cancelled ◄─┴→ Failed ◄──────┘
```

browser 保存 response 元数据并请求 Kotlin 启动 `ACTION_CREATE_DOCUMENT`。取得 content URI 后，Kotlin 只把受限 FD/结果交给 browser；renderer 永远不获得 URI 或 FD。应用被杀时 `Running` 记录恢复为 `Failed/Interrupted`，首期不自动续传；用户可重试并重新选择目标。

### 8.5 安全考虑

- **进程权限**：browser 具有 `INTERNET`；isolated renderer/image-decoder 无应用 UID 权限；compositor 同 UID 但接口仅接受有界绘制/Surface 数据。
- **组件暴露**：所有 Service、provider 和内部 receiver `exported=false`；launcher Activity 只接受已声明的 launcher 与 HTTP(S) VIEW intent。
- **启动认证**：每次 bind 生成不可预测 nonce，经 binder bootstrap 和 socket 首消息双向校验；旧 generation/nonce 连接拒绝。
- **输入验证**：所有 JNI/Binder/IPC 长度先验证再分配；enum/version/ID/URL 均 fail closed；未知消息不得触发默认行为。
- **网络边界**：renderer 发送结构化 fetch 请求；browser 复用 CORS/CSP/同源与下载判断，不提供任意 socket 代理。
- **文件边界**：外部文件只通过 content URI/FD；拒绝外部 `file:`，不申请全盘存储；profile path 必须位于 app-private root。
- **日志隐私**：URL 日志默认仅保留 scheme/host 或散列 request ID；认证、Cookie、正文、输入和 content URI 不落日志。
- **依赖供应链**：Gradle dependency verification、Cargo.lock、V8 source revision/SHA-256 和 APK SBOM/许可证清单进入 Release 证据。

### 8.6 替代方案

| 维度 | A：Kotlin/Compose + Rust（选定） | B：纯 Rust NativeActivity/winit | C：Android WebView 壳 | D：等待通用 UI SDK |
|---|---|---|---|---|
| 移动系统集成 | 🟢 原生 | 🔴 需自建大量桥接 | 🟢 原生 | 🟡 未实现 |
| ZeroWeb 内核复用 | 🟢 完整 | 🟢 完整 | 🔴 不使用 ZeroWeb | 🟢 完整 |
| 固定多进程可实现性 | 🟢 Service/AIDL | 🟡 仍需 Java Service | 🔴 不受本项目控制 | 🟡 延迟 |
| 多标签/书签/历史/下载 UI | 🟢 Compose 成熟 | 🔴 需先造移动组件 | 🟢 容易 | 🟡 需先完成 SDK |
| 长期状态一致性 | 🟢 Rust 单一源 | 🟢 Rust 单一源 | 🔴 两套内核语义 | 🟢 可设计 |
| 首期风险 | 🟡 JNI/双语言 | 🔴 IME/a11y/权限 | 🟢 低但违背目标 | 🔴 前置工程过大 |
| 推荐度 | ⭐⭐⭐ | ⭐ | 不可接受 | ⭐ |

**选择 A 的理由**：

1. 它与 Chrome/Edge Android 的“原生移动 UI + 自带内核 + Android Service 多进程”架构原则一致。
2. 用户要求首期即包含完整浏览器 chrome 功能，Compose 显著降低移动 UI、IME、无障碍和系统能力风险。
3. Rust facade 保留 ZeroWeb 业务和内核的单一事实源，Kotlin 不演变成第二个浏览器核心。

方案 B 仅在未来通用 UI SDK 已具备移动组件、IME、无障碍和平台服务后重新评估；方案 C 违反核心目标；方案 D 会让 Android 交付被未落地的大型前置项目阻塞。

---

### 8.7 实施计划

实施顺序以 §7.2 M0～M5 和 §7.3 推荐修改顺序为权威来源。各阶段必须满足以下切点：

1. **先证明工具链和 V8**：没有可重复的 arm64 V8/native build，不进入 UI 或进程大改。
2. **再抽共享角色入口**：每次抽取后先证明桌面 binary 行为不变，不把 Android `cfg` 散入页面逻辑。
3. **再证明 Android 物理进程**：先用 ping/health 和 PID/UID 验收 Binder/socket 拓扑，再发送页面消息。
4. **再证明 Surface**：先呈现确定性测试图案并覆盖 rotate/detach/death，再接 PaintSnapshot。
5. **业务 facade 与 UI 可并行于 fake renderer 开发**，但只有 Surface 和进程门禁通过后才接真实页面。
6. **功能闭合后再做性能优化**：优化必须建立在 frame/request ID trace 和 PSS 数据上，不以去掉隔离换性能。

任何阶段发现必须恢复进程内 renderer、使用 System WebView 或让 Kotlin 持有第二套业务数据库，均属于范围严重偏离，按紧急停止条款处理。

### 8.8 测试策略

#### 单元与契约测试

- `zero-browser-shell`：显式 profile、原子写 fault injection、session/history/download schema、Action revision/conflict。
- `zero-protocol`：Android bootstrap 上限、未知版本、socket close、Binder death 映射、slot generation。
- JNI facade：无效 handle、重复 destroy、超长 JSON、并发 dispatch、Kotlin/Rust golden schema。
- role entry：desktop 与 Android bootstrap 调用同一核心主循环；错误只停角色实例。
- compositor：Surface generation、resize、旧帧丢弃、GPU→CPU compositor 内降级。
- Compose：每个页面正常/空/加载/错误状态、系统返回优先级、双语和 semantics。

#### 集成测试

- fake renderer 驱动 Compose/Rust 全业务流程，避免 UI 测试依赖公网。
- 本地 HTTP/HTTPS fixture 覆盖导航、重定向、输入、下载、断网与大响应。
- Service 集成覆盖 8 slots、LRU 挂起、renderer/image-decoder/compositor death。
- Activity lifecycle 覆盖 rotate、recreate、background kill、Surface rebind 和未提交地址栏编辑。
- SAF fake/provider 覆盖取消、空间不足、FD close、URI grant 失效和打开 Activity 缺失。

#### 真机端到端验收

- 至少一台 arm64 真机执行启动、真实 HTTPS 页面、触摸滚动、点击、IME、多标签、书签、历史和下载。
- 使用 ADB 记录 browser/renderer/compositor/image-decoder PID 与 UID，主动 kill 各角色并验证恢复。
- 对选定真实站点执行 Chrome Android/ZeroWeb 同步骤比较，保存截图、页面状态、输入结果、几何和帧 ID 证据；截图本身不作为唯一通过依据。
- API 26/30/36 至少覆盖三档中的真机一档与受支持 arm64 设备/镜像其余两档。

#### 性能与资源测试

- Macrobenchmark：冷启动、恢复启动、标签切换。
- Perfetto：触摸→renderer→compositor→present 时序，后台 CPU 和 binder/socket 活动。
- `dumpsys meminfo`：1/5/8 驻留 renderer 及挂起后的 PSS。
- 100 次 tab churn、100 次 rotate/rebind 和重复下载取消；测试由 `make android-test` 的墙钟/内存 guard 包裹。

#### 构建与回归门禁

```text
make android-preflight
make android-apk
make android-test
make android-install-smoke
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
make test
```

若默认 V8 feature 因 Android archive 只在专用构建机可用，本地至少执行 quickjs/可编译 feature 的严格 clippy，并由 Android CI 执行 V8 arm64 构建；覆盖差异必须写入验收报告，不能把未运行标为通过。

### 8.9 回滚计划

- **M0 回滚**：Android 工程是新增目录，可整体撤销；不修改现有运行时行为。
- **M1 回滚**：共享 role entry 抽取按 renderer/compositor/image-decoder 分提交；任一抽取导致桌面回归时回滚对应提交，不保留 Android 专用复制主循环。
- **M2 回滚**：profile schema 每类独立版本并保留前一文件；旧 APK 遇到未来 schema 必须只读保护，禁止覆盖。
- **M3 回滚**：Android Surface adapter 与桌面 present 以 target module 隔离；回滚 adapter 不恢复进程内 renderer。
- **M4/M5 回滚**：功能按 Action/Compose screen 分批提交；可回滚单功能 UI，但首期 Release 门禁仍要求 FR-001～FR-010 全部满足，不得以隐藏未完成功能冒充完成。
- **架构不可回滚项**：固定物理多进程、ZeroWeb 内核、Rust 单一状态源、无广泛存储权限属于用户已定约束；若实施失败必须返回 RFC 重新决策，而不是添加兼容开关。

---

## 9. Spec Lint 报告

### 9.1 结构完整性

| 规则 | 裁决 | 说明 |
|---|---|---|
| 执行摘要存在性 | ✅ Pass | §0 位于正文首部，包含目标、范围、排除、约束、推荐方案和首步。 |
| 场景存在性 | ✅ Pass | 机械统计 FR-001～FR-010 均有 2 个验收场景。 |
| 异常路径覆盖 | ✅ Pass | 每个 FR 均为 1 个正常场景 + 1 个失败/边缘场景，异常数等于正常数。 |
| 测试绑定 | ✅ Pass | 20 个 FR 场景均含 `验证:`；NFR-001～NFR-009 均含测量标准，见 §3～§4。 |
| UI 对齐 | ✅ Pass | IF-001 定义 5 个页面、正常/空/错误状态、默认动作、双语资源和 ASCII 线框。 |
| TBD 清零 | ✅ Pass | §10 明确无阻塞 TBD；TBD-1～TBD-4 均绑定后续里程碑或排除项。 |
| 约束覆盖 | ✅ Pass | §6.1 的内核/进程/权限由 FR-008/NFR-004 覆盖，状态与 profile 由 FR-001/003～006 覆盖，Surface 由 FR-007/NFR-003 覆盖，构建由 FR-010 覆盖，无障碍由 NFR-007 覆盖。 |
| 实施交接完备 | ✅ Pass | §7.3 包含文件/模块、职责、能力来源、修改顺序和首批提交。 |
| 首步可执行性 | ✅ Pass | §0 与 §7.3 均指定空 APK + Rust library，验证为 Android build/ADB launch。 |

### 9.2 语言精确性

| 规则 | 裁决 | 说明 |
|---|---|---|
| 模糊动词 | ✅ Pass | 对 FR/NFR 正文机械扫描“处理/管理/优化/支持”等词为 0；行为均使用创建、拒绝、保存、恢复、呈现等可观察动词。 |
| 无量化描述 | ✅ Pass | 性能、内存、CPU、API、窗口、payload、slot、重启和测试次数均在 §4、IF-002/004、§8.8 量化。 |
| 非确定性措辞 | ✅ Pass | FR/NFR 机械扫描“应该/可能/大概/尽量”为 0；未验证事实统一放入 §6.5 假设或 §10。 |

### 9.3 一致性

| 规则 | 裁决 | 说明 |
|---|---|---|
| 范围冲突 | ✅ Pass | §1.4 将手机 MVP 与 Play、账号、无痕、平板专用 UI 等排除项分开；FR-001～FR-010 未引入排除能力。 |
| 约束冲突 | ✅ Pass | §6.1/§6.2 均坚持 ZeroWeb 内核、Rust 单一状态源和物理多进程；CPU 降级仍留在独立 compositor，见 §1.4。 |
| 方案漂移 | ✅ Pass | §0 推荐方案、§6.3 决策、§7 交接和 §8 目标架构一致采用 Compose/JNI/Service/Surface。 |
| CLI 语义一致 | ⏭️ Skip | 本任务不新增用户 CLI；Make/Gradle targets 是构建入口，统一定义于 FR-010、IF-008 和 §8.8，不存在参数/退出码产品契约。 |
| 默认动作闭合 | ✅ Pass | IF-001～IF-008 均显式定义默认动作；slot、Surface、下载、Intent 和构建失败均无静默回退。 |
| 章节引用正确 | ✅ Pass | 机械提取 FR/NFR/IF 引用均能找到定义；权威来源分别为 §3、§4、§5、§6、§7 和 §8。 |
| 外部事实保守化 | ✅ Pass | 固定工具版本是 IF-008/§6.3 的实现决策；rusty_v8/wgpu/真机等未由本仓验证的事实均在 §6.5 标注状态。 |
| 未验证细节泄漏 | ✅ Pass | §6.5 的 Surface backend、真机和本仓 V8 状态没有提升为 FR 成功断言；均由 M0/M1/M5 spike 闭合。 |
| 场景预期泄漏 | ✅ Pass | FR 场景验证进程、行为和失败恢复，不硬编码 TBD-1～TBD-3 的机型、依赖补丁号或 wgpu backend。 |
| 实现来源闭合 | ✅ Pass | §6.5A 与 §7.3 对状态、renderer、compositor、decoder、协议、JNI、Surface、V8、SAF 和测试逐项给出来源。 |
| 来源-测试联动 | ✅ Pass | §7.3 能力来源均绑定验证；§8.8 按 browser-shell/protocol/JNI/Surface/SAF/V8 分层测试。 |
| 脆弱选择逻辑覆盖 | ✅ Pass | renderer slot/LRU、Surface generation、schema version、V8 校验和和下载 URI 分支均在 FR-007/008/010、IF-003/004/006/008 与 §8.8 覆盖。 |
| 类型分层清晰 | ✅ Pass | Requirement 位于 §3～§5，Decision/Constraint/Assumption 位于 §6，Implementation 位于 §7～§8，TBD 位于 §10。 |
| 优先级完备 | ✅ Pass | FR-001～FR-010 和 NFR-001～NFR-009 均显式标注优先级。 |
| 代码边界完备 | ✅ Pass | §6.6 分别列出允许和禁止路径，并禁止 Android 内核 fork、桌面无关重构及进程内回退。 |
| 清单数量一致 | ✅ Pass | 四类 Android 角色 = browser + renderer + compositor + image-decoder；三个现有 helper = renderer + compositor + image-decoder；8 个 renderer slots 在 IF-004、§6.3、§8.4.3 一致。 |
| 依赖清单一致 | ✅ Pass | AndroidX/Compose、JNI/NDK、cargo-ndk、V8 均以 §6.5A 为主定义；IF-008 与 §7 仅引用其构建用途，未出现冲突计数。 |
| 重复失控 | ✅ Pass | FR 定义行为、IF 定义接口、§8 定义实现；摘要和交接使用交叉引用/提炼，没有重复参数表作为第二权威源。 |

**汇总**：29 Pass / 0 Warning / 0 Fail / 1 Skip
**门禁判定**：Fail = 0，文档允许提交用户确认；Skip 为不适用的用户 CLI，不代表 Android 构建或运行验证已通过。

---

## 10. 待定列表

| ID | 项目 | 优先级 | 缺失信息 | 下一步 |
|---|---|---|---|---|
| TBD-1 | 基准真机型号 | 重要 | 用户最终可提供的 arm64 真机具体型号/API/RAM | M5 验收前记录设备；不改变 NFR 测量方法 |
| TBD-2 | Compose BOM、`jni`、`ndk`、cargo-ndk 精确补丁版本 | 重要 | 实施日与 AGP 9.2 兼容的稳定版本 | M0 查询官方/上游发布并写入 version catalog/Cargo.lock；禁止动态版本 |
| TBD-3 | wgpu Android Surface 首选 backend 和 swapchain 配置 | 重要 | 目标真机 Vulkan/GL 能力与 wgpu 30 实测结果 | M1 spike 记录 adapter、format、present mode；不得改变物理多进程 |
| TBD-4 | Release 正式签名 | 可选 | 用户尚未提供 keystore/发布主体 | 本期只生成未签名或本地测试签名 Release APK；密钥不进仓库 |

本文档不存在阻塞级 TBD。TBD-1～TBD-3 是对应里程碑内必须通过实测闭合的实现参数，不能被提升为未经验证的产品断言。

---

## 11. 修订历史

| 版本 | 日期 | 变更内容 |
|---|---|---|
| v0.1 | 2026-08-19 | 初始 Spec/RFC：确认 Compose + Rust、固定 Android 多进程、完整首期 MVP、arm64 和 application ID |
| v0.2 | 2026-08-19 | 本机仅有 API 36 x86_64 模拟器；Debug 增加 x86_64 验证 ABI，Release 仍仅分发 arm64-v8a |
| v0.3 | 2026-08-19 | AGP 9.2 启用内建 Kotlin；移除已被 AGP 拒绝的 `org.jetbrains.kotlin.android` 插件，保留 Kotlin/Compose UI 方案 |
| v0.4 | 2026-08-19 | API 36 模拟器拒绝 `isolatedProcess` 与显式私有进程名组合；isolated renderer/decoder 改由 Android 分配进程名，隔离 UID 与 Service slot 语义不变 |
| v0.5 | 2026-08-19 | 本机可用 NDK 为 r30；构建基线从 r29 更新为已验证的 r30，Release ABI/进程边界不变 |
| v0.6 | 2026-08-19 | AGP 9 需显式启用 AIDL；render foundation 经 winit 在 Android 编译时需启用 native-activity glue feature，均为构建适配，不改变 Kotlin Activity 宿主或多进程边界 |
| v0.7 | 2026-08-19 | winit native-activity glue 需要 `android_main` 链接符号；JNI cdylib 提供未被 manifest 调用的锚点，Kotlin Activity 仍是唯一宿主入口 |
| v0.8 | 2026-08-19 | renderer Android 依赖图暴露 reqwest native-tls 的 OpenSSL 交叉编译缺口；workspace 网络栈改用 rustls TLS，保持同一 `zero-net` API 和平台行为 |
| v0.9 | 2026-08-19 | 当前 rusty_v8 crate 不含 Android target binding；Android Gradle 按 variant 以 `V8_FROM_SOURCE=1` 构建单 ABI V8，避免 debug 构建同时编译 arm64 与 x86_64 |
| v1.0 | 2026-08-19 | M1 构建探针证实 rusty_v8 的 Android source-build 在 Windows 主机解包 Linux sysroot 时因符号链接不受支持而失败；真实 renderer APK 构建须迁至 Linux/WSL CI 或获得 Android 预编译 V8，不能以进程内或 QuickJS 替代绕过 |
| v1.1 | 2026-08-19 | Linux/WSL 复验表明迁移宿主仍不足：rusty_v8 150.2.0 source-build 缺少 Android GN 所需 Python 依赖文件，且构建脚本隐含 NDK/工具下载。真实 Android renderer 的前置条件调整为升级到具备完整 Android source build 的 V8 发行版，或引入经校验的官方 Android V8 archive；此前不接入 renderer 到 APK |
