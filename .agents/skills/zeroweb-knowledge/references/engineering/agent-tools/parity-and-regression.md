# Chrome 一致性、巡检与证据契约

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

定位 Chrome parity 的场景/manifest/比较器和 regression guard 的授权、缓存与内存口径。实际执行按 [parity 入口](../../../../zeroweb-browser-chrome-parity/SKILL.md) 或 [巡检入口](../../../../zeroweb-regression-guard/SKILL.md)；本轮未采集图片或运行巡检。

### Decision Entry
采集模式由 capture-chrome 的 CDP/GUI/headless 分支决定，巡检模式由手动授权或有效调度合约决定；不能凭目录或旧成功标记自动选路径。

## Branching Table
| 维度 | 条件 | 行为及可宣称边界 |
|---|---|---|
| Chrome 实例 | ORACLE_CDP_URL | 优先复用，但需读启动参数证明 GUI，否则 unverified |
| Chrome 自动启动 | gui / 缺省 | 独立 GUI / headless 诊断 |
| Chrome 沙箱 | 未显式获准关闭 | 默认保留沙箱 |
| ZeroWeb 生产 | browser-pointer + production-window-gpu | 可进入完整生产一致性门禁 |
| ZeroWeb WebDriver | live renderer | 行为诊断，不是浏览器窗口 GPU |
| 区域遮罩 | 文本控件 glyphMaskInsetPx | 可掩字形内部，仍保留区域原值与未遮罩全图 |
| macOS 内存 | ps 固定间隔采样 | sampled peak，不能当内核 RSS 高水位 |
| 巡检缓存 | 同 SHA 但策略/场景/环境变化 | 缓存未命中，不能沿旧 PASS |

表格来源：parity 的 `scripts/capture-chrome.mjs`、`references/evidence-contract.md` 与 regression guard 的 `references/guard-policy.md`。

## Affected Scope
| 路径 | 影响 |
|---|---|
| `.agents/skills/zeroweb-browser-chrome-parity/scripts/capture-chrome.mjs` | 对照模式及事件采集 |
| `.agents/skills/zeroweb-browser-chrome-parity/scripts/validate-scenario.mjs` | 可真实应用的场景环境 |
| `.agents/skills/zeroweb-browser-chrome-parity/scripts/compare-evidence.mjs` | 身份、状态、事件、几何、像素 |
| `.agents/skills/zeroweb-browser-chrome-parity/scripts/run-parity.mjs` | 有界子命令编排 |
| `.agents/skills/zeroweb-regression-guard/references/guard-policy.md` | 内存与巡检成功口径 |
| `scripts/run-oracle-capture.sh` | 旧 WPT oracle 包装，不等价现代生产 parity |

## Gotchas
- Chrome CDP 可连通不证明 GUI；检测 HeadlessChrome、headless 参数与 enable-automation，无法读取完整参数记 unverified，不能手填 manifest 绕过。 (`.agents/skills/zeroweb-browser-chrome-parity/scripts/capture-chrome.mjs`)
- validator 只检查 stateExpression 非空，语法/运行时/序列化错误只能在真实页面确认；locale 仅支持 en-US、reducedMotion 仅 no-preference，不是任意仿真环境都可验收。 (`.agents/skills/zeroweb-browser-chrome-parity/scripts/validate-scenario.mjs`)
- 比较器拒绝两边同时漏采、重复或乱序检查点，状态可显式 null 但不能缺失；只有 click offset/jitter 的 null 归一化为省略，不宽松转换其他动作字段。 (`.agents/skills/zeroweb-browser-chrome-parity/scripts/compare-evidence.mjs`)
- 全图与区域像素要求严格小于阈值，几何允许等于上限；这两种比较符不能统一为同一种门禁。 (`.agents/skills/zeroweb-browser-chrome-parity/references/evidence-contract.md`, `.agents/skills/zeroweb-browser-chrome-parity/scripts/compare-evidence.mjs`)
- macOS ps 采样最大 RSS 未超限只支持严格峰值 INCONCLUSIVE；超过预算则足以 FAIL，ru_maxrss 需目标或直接父进程取得且按 bytes 换算。 (`.agents/skills/zeroweb-regression-guard/references/guard-policy.md`)
- 老 run-oracle-capture 壳会安装 puppeteer 并以 no-sandbox 启动 X11 Chromium；它是旧 WPT oracle 工具，不继承现代 parity 默认沙箱或完整生产验收保证。 (`scripts/run-oracle-capture.sh`, `.agents/skills/zeroweb-browser-chrome-parity/SKILL.md`)

## Architecture
- run-parity 接受 JSON argv 并禁用 shell，依次 validate、Chrome、ZeroWeb、production compare；PARITY_TIMEOUT 限每个子命令而不限制内存，外层资源守卫仍需另外设置。 (`.agents/skills/zeroweb-browser-chrome-parity/scripts/run-parity.mjs`)
- success 缓存绑定 git/tree、报告摘要、环境、场景、策略，只有最终干净 HEAD 全部门禁 PASS 才更新；deferred 是专项未关闭，不得当成功。 (`.agents/skills/zeroweb-regression-guard/references/guard-policy.md`)
- 分进程绝对预算 renderer/compositor 峰值各≤192MiB，稳定空闲合计≤256MiB，与 perf-gate 相对基线独立；不能以一门通过抵消另一门失败。 (`.agents/skills/zeroweb-regression-guard/references/guard-policy.md`)

## Branching Behavior
- 手动验收不因触发 Skill 自动切 main、修复、通知或推送；调度合约的具体权限仅适用于该次巡检，读取 run-rules 不继承其他 job 的外部操作权限。 (`.agents/skills/zeroweb-regression-guard/SKILL.md`)
