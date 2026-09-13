# 控制能力、对照与证据

启动/恢复完整阅读。执行命令前检查当前源码和工具帮助，不把这里的现状当作永久能力。

## 控制面选择

| 对象 | 使用方式 | 证据边界 |
|---|---|---|
| ZeroBrowser UI | 操作专用真实窗口的指针/键盘，采集全窗口图 | 地址栏、标签、焦点和窗口反馈只能由实际壳层输入验证 |
| ZeroWeb 页面 CDP | 实测支持的方法用于导航、输入、DOM/脚本只读观察、console/network | 支持部分 CDP 不等于支持整个 Playwright；必须确认 target 和渲染路径 |
| ZeroWeb 生产页面 | 兄弟 parity skill 的生产窗口输入与 GPU readback | 能证明采集检查点的实际产品输出；固定 scenario 不能代替自由探索 |
| Chrome/Chromium | GUI CDP，复用仓库锁定 Puppeteer 与 parity 工具 | headless 可诊断，不能替代既有完整生产视觉门禁 |
| Firefox | 实际可用的 WebDriver BiDi/WebDriver 或 GUI | 单独保留证据，不把 Firefox 填成 chrome manifest 绕过比较器 |

编写时核对的仓库位置：`apps/browser/src/main.rs` 的 `run_headless` 和
`apps/browser/src/headless/` 的协议实现。当前 `--headless` 进入远程控制服务，
`--remote-debugging-port` 不能单独证明存在 GUI CDP 服务。
`docs/goal/cdp-protocol/master.md` 可查进度，实际代码与探针结果决定本次能力。
`zero-webdriver` live renderer 属行为诊断；不能与无关 GUI 初始帧拼成完整 E2E。

最小预检在本地合成页面执行：导航到带按钮/输入框的页面，订阅实际支持的日志事件，
用输入协议或 GUI 点击、键入、Tab；验证真实状态改变、URL/target 对应、截图可读，
再确认退出与超时能回收本任务进程。日志探针须产生可识别的合成消息/异常，确认采集
覆盖，不能把空数组当作零错误。每项标 `available|unavailable|unverified`。
不要为了补齐预检构建整套自动化平台；局部工具缺口先试已授权 GUI 路径，其余分流。

选择当前仓库已安装/锁定的工具，不自动安装全局最新版。GUI 缺失时报告其范围阻塞，
可继续 CDP 诊断和回归，但不能承诺 UI 或生产视觉通过。只具备截图能力还不够，
执行 Agent 必须实际查看截图才能给视觉结论。

点击前检查可见、稳定、未被遮挡和可交互；使用浏览器输入链路，不用 `element.click()`、
DOM dispatch、赋值 value 或私有网站接口替代用户操作。CDP 页面导航可以测试页面加载，
不能记为地址栏操作成功。只读 evaluate 用于观察；注入诊断探针应记录且不改变待测语义。
键盘与中文输入等未支持的能力单独记缺口，不能用剪贴板替代后声称 IME 已验证。

## 差异归因

采用三角证据：真实 ZeroWeb 行为、参考浏览器行为、相关 HTML/CSS/DOM/API 规范与 WPT。
Chrome 是主要观察参照，不是所有语义的最终裁判。Firefox 与 Chrome 分歧时先查标准、
最小复现和版本差异；规范仍不清就保留 inconclusive，不投票决定实现。

固定 OS、viewport、DPR、缩放、字体、locale、主题、reduced motion、缓存和登录状态；
记录浏览器准确版本、GPU adapter/backend、软件渲染、网络条件及代理是否等价。
已有固定旧版 oracle 可用于历史回归，不能称为当前主流浏览器表现；另采当前可用版本
时独立标明，不修改旧场景的版本/阈值限制来让新版本混入同一基线。
当前 parity 环境限制以其 validator 为准；不支持的配置不能只在 Chrome 端模拟。
ZeroWeb UI 与参考浏览器 UI 不做像素相等；页面内容区域和全窗口证据分别保存。

对线上动态页面保留访问时间、最终 URL、页面状态、相关资源版本/摘要（可取得时）、
请求失败与重定向。先排查 CDN、A/B、广告、登录、反自动化和 UA 导致的内容差异。
两端收到不同内容时不能直接判引擎渲染错误。只用 UA 变更做诊断，不隐瞒测量条件。
无关动态区域可作补充区域比较，但原始全图保留；不能扩大遮罩、停掉脚本或删除资源
掩盖目标缺陷。不稳定全图记不可比，再用最小稳定 fixture 定位。

网站快照/HAR 仅在允许保留且已脱敏时采集；回放会改变缓存、时序、来源和 Service Worker
行为，需标 diagnostic。最小本地用例用于确定性回归，原网站用于任务复测，两者都保留。
不复制整站或带凭据资源进入仓库；常驻测试用自有最小资源并按仓库 WPT 流程登记。

## 等待与失败

等待有上限的任务状态成立、导航进度、字体/图片就绪及新帧到达；稳定场景再核对连续
两次帧。固定 sleep 或 network idle 不能单独代表完成，长连接、动画和懒加载需按任务
定义关键区域/里程碑。超时记录最后状态、日志、截图与进程状态，不无限重试。
重试保留第一次失败；同条件只重试一次基础设施探针，无新信息则分流工具/环境问题。
renderer 崩溃、断连和导航超时可能是产品缺陷，不能一律当作测试环境故障排除。
每条长命令的超时不得超过本次剩余时间；启动新候选前预留完整验证成本。收尾时发现
无法在截止前完成验证，保存为 inconclusive，不把外层 120 分钟限制交给无限子进程。

## 性能口径

优化前冻结指标、测量起止点、冷/热缓存方案、样本数和不可退化范围。
主指标优先“导航开始到关键内容可用”和“输入发出到正确可见反馈”，分别保留协议/
主机时钟来源；不混用不同进程的未校准时钟。Agent 思考和工具排队耗时单独记录。
不要在重构建或多个浏览器争抢 CPU/GPU 时测性能；保存同机负载和每次原始结果。

默认对前后版本交错做至少 5 组同条件测量，冷缓存与热缓存分开。报告中位数、范围、
全部失败和样本数；这是项目起始策略，不是统计显著性保证，少量样本不报告可信 p95。
差异小于环境波动则 inconclusive。参考浏览器给出体验参照，修复收益以 ZeroWeb 自身
前后配对为主，不拿 Chrome 的速度变化当作 ZeroWeb 优化收益。
RSS/CPU 覆盖 browser、renderer、compositor 等本任务进程树；不将单进程数值称作总量。

LCP、CLS、Event Timing 等只在双方真实支持且语义可比时采集；不支持记 unavailable。
合成操作反馈延迟不能称作真实用户 INP，少量实验室样本不能声称达到现场 Web Vitals
通过率。需要分析启动/布局/脚本时再定向 trace/profile，不默认记录全站全部敏感负载。
项目长期性能回归复用 bench-gate 和已提交基线，不为本次通过改基线或测量配置。

## 依据与设计取舍

以下一手资料查阅于 2026-09-13；运行时核对版本。预算、样本数与饱和窗口均为本
skill 的工程默认值，不是来源规定或已证明最优的参数。

- [Playwright CDP 连接](https://playwright.dev/docs/api/class-browsertype#browser-type-connect-over-cdp)：其公开支持范围是 Chromium，且 CDP 连接能力弱于原生 Playwright 协议。ZeroWeb 兼容性要实测。
- [Firefox 远程协议设置](https://firefox-source-docs.mozilla.org/remote/Prefs.html)：CDP 支持已结束，Firefox 141 移除了相应协议选择设置；不能假设 Chrome/Firefox 共用 CDP。
- [Playwright actionability](https://playwright.dev/docs/actionability)：可见、稳定、接收事件等输入前提用于避免操作假成功；ZeroWeb 不支持高层工具时仍需核对这些条件。
- [WPT 测试类型](https://web-platform-tests.org/writing-tests/)：API 语义优先 testharness，视觉行为适用 reftest。支持将站点问题抽为可复用规范回归。
- [Web Vitals](https://web.dev/articles/vitals)：区分实验室和现场度量，不能用一次自动化导航声称真实用户性能达标。

参考 ZeroSeed 的流程思想在本仓自包含落地；未复制其模型计费、REPL 或专属 Eval
协议。复用 ZeroWeb parity 而不新建适配器框架；未来 GUI CDP 完善后，通过能力预检
接入，无需把探索工作流绑定到某个 Agent 产品。
