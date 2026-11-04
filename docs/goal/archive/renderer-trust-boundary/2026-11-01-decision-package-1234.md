# 决策包：① CDP 默认 token / ② `__zw*` 装配策略 / ③ helper OS sandbox / ④ 平台策略

**日期**：2026-11-01　**HEAD 基线**：`34d3ce88a`
**性质**：设计选项细化（无代码变更）。④ 已有选项表（[M4 前置](evidence/2026-11-01-m4-preflight-platform-policy-dc-audit.md)，推荐 C）；本文把 ②③ 细化到可拍板粒度，① 归纳产品时序取舍。拍板后按「落地顺序」执行对应切片。

## 拍板记录（2026-11-01，用户逐项确认；拍板基线 `c3a862434`）

- **① 选 a**：CDP 默认强制 token + 迁移说明（仓内消费者已核对可控，同切片更新）。
- **② 选 ②C'**：全收口——②A-env counter-plugin 删 3 枚（测试模式信号 ②A-env：renderer 进程 env）+ `__zwSetPermission` shim 内闭包化；不留 fullscreen 权限路页面自授残余。
- **③ 选 ③A**：两切片，image-decoder 先行、renderer 后行；renderer 网络面另立 fetch_proxy 出网收窄切片，不在本切片硬装。
- **④ 选 C**：分层延迟拍板——compositor 维持 fail-open + warn；image-decoder 新档案即 fail-closed；renderer fail 语义随 ③A-renderer 切片定。

执行按下方「落地顺序建议」：② → ③A-image-decoder → ③A-renderer → ① → M4 收口。

## ② `__zw*` 测试钩子族装配策略（缺口表在册，DC-1/DC-2 剩余项）

### 现状机制（代码查实；2026-11-01 二轮复核，含 5791f2579 shim 变更后行号核验）

- shim 经 `include_str!` 字节拼接注入每文档（`js_dom_bridge.rs:5371-5383`）；钩子以 `globalThis.__zwX = function` 定义、`typeof` 幂等卫，**任何页面无条件可见**。全 shim `globalThis.__zw*` 名共 177 枚，逐一过名分类：绝大多数为内部管线约定（DocWrite/Slot/Ce/Nav 等实现既有页面 API，无新增特权）。
- **特权 mutator（标准 API 门控旁路面）在册 4 枚**（二轮复核新增 1 枚）：`__zwUserActivate`（part06.js:446，直置瞬态激活；fullscreen 激活门 part04.js:4970 一次性消费 `_zwTransientActive`）、`__zwSetPermission`（part02.js:8656，直写权限注册表；fullscreen 门权限路 part04.js:4966 消费）、`__zwClipboardStoreWrite`（part02.js:8866，直写剪贴板存储）、**`__zwDeleteAllCookies`（part06.js:391，无条件定义，清空 in-JS cookie jar——跨站 cookie 销毁，DC-2 类；runner TESTDRIVER_STUB 转发、钩子缺失 stub 侧拒绝，testharness.rs cookies-compat M1 注释自证）**。
- **同型 flag 候选（低风险）**：`__zwModalGateBypass`（part01.js:1173/1178 同步窗口置位/清除、part04.js:4662 模态 inert 焦点门消费）——页面可翻 true 但仅自影响焦点语义，非跨站/存储/权限特权；置 non-writable 会破 shim 自身 fixup 写入（不采），倾向记接受残余，② 切片内定。
- **②A 关键成本（二轮复核查实）**：`__zwSetPermission` 为 load-bearing——`navigator.permissions.request()` headless 自动授予语义（'prompt'→granted，part02.js:9624-9628，注释明示「经 __zwSetPermission 落同一注册表」）在调用时经 `globalThis` 活查找。②A 直接删除则 request() 静默退化为 'prompt' 不授予（非测试页行为变化；WPT 不受影响——测试模式钩子保留）。
- **合法调用方在页面之外**：runner 经注入脚本调用（`tests/wpt-runner/src/testharness.rs` set_permission/bless/delete_all_cookies 三白名单注释，10704 邻域），非页面脚本——测试模式信号可由 runner→renderer 链路携带，页面侧无从伪造。

### 选项

| 选项 | 机制 | 优点 | 缺点/风险 |
|---|---|---|---|
| **②A 反插桩 counter-plugin（推荐，范围修订为 4 枚中删 3 枚）** | shim 三文件**零改动**（§9 与 html-semantics 流碰撞规避）；本 goal 新增一个 host 侧后置 JS 片段：非测试模式在页面脚本执行前 `delete globalThis.__zwUserActivate/__zwClipboardStoreWrite/__zwDeleteAllCookies`（`__zwSetPermission` 留守——load-bearing，见上；保留内部 reader 钩子如 `__zwPermissionState`——permissions.query 实现自身消费）；测试模式跳过删除 | 碰撞面最小（新文件归本 goal）；删除面覆盖激活/剪贴板/cookie jar 三特权路；runner 路径零回归（模式开时钩子原样） | **fullscreen 权限路残留**：`__zwSetPermission('fullscreen','granted')` 页面仍可自授（激活路已封、权限路留）——收口半量；「全收口」须选 ②C' 组合（见下） |
| **②C' 组合：②A 删 3 枚 + `__zwSetPermission` shim 内部闭包化（单点小切口）** | ②A 全套 + part02.js request() 自动授予路改内部闭包引用、`__zwSetPermission` 改测试模式条件定义（part02 单文件 ~10 行） | 特权面全收口；request() 语义不变 | part02.js 单点改动需与 html-semantics 流协调（§9 一次性感撞，非长期并行占用） |
| ②B shim 全量条件化 | 三文件 `if (测试模式)` 包裹钩子定义 | 单点、无删除时序 | 正面碰撞面最大（part02/04/06 三文件）；每文件改动都需双流协调 |
| ②C 按 origin 暴露 | 仅测试 origin 注入钩子 | — | 拒绝：runner 页与普通浏览同为 localhost 端口，不可区分，攻击页可直接命中 |

测试模式信号源（②A/②C' 内子选项）不变：**②A-env（推荐）** renderer 进程 env（`ZW_RENDERER_AUTOMATION=1`，runner/browser 派生时注入，进程级不可被页面翻转；wire 不变）；②A-ipc 动 protocol wire，成本高于收益。

### 验收形态（拍板后切片随附常驻回归）

非测试模式：被删 mutator `typeof === 'undefined'` + fullscreen 激活门页面不可自授（激活门回归）+ permissions.query 正常路径不回归 + cookie jar 页面不可清（DC-2 面）；测试模式：runner 既有套件原样绿（html-semantics 流全量由该流自证）。②C' 另验：permissions.request() headless 自动授予语义非测试模式不回归。

### 测试模式信号源（②A 内子选项）

- **②A-env（推荐）**：renderer 进程 env（如 `ZW_RENDERER_AUTOMATION=1`）——runner/browser 派生 renderer 时注入，进程级不可被页面翻转；与既有 role env 剥离顺序无冲突（flag 是加法不是剥离对象）。wire 不变。
- ②A-ipc：IPC 角色扩展携带 automation 标志——语义更显式但动 protocol wire（bincode 追加变体/字段），重启重放语义要跟着核，成本高于收益。

### 验收形态（拍板后切片随附常驻回归）

非测试模式：`typeof __zwUserActivate === 'undefined'` 等三枚 mutator 不可见 + fullscreen 门页面不可自授（激活门回归）+ permissions.query 正常路径不回归；测试模式：runner 既有套件原样绿（html-semantics 流全量由该流自证）。

## ③ helper 完整 OS sandbox（DC-4 剩余项）

### 现状与 IO 面事实

- renderer：env 剥离 only（RFC §六 P3 占位，`apps/renderer/src/sandbox.rs:19` 自述）；启动时序契约已审计成立（M3 切片 2）。
- image-decoder：**无任何沙箱钩子**；IO 面查实为**纯 stdio 循环**（`run_role` stdin/stdout 收发字节→内存 decode→rgba，`fs::`/`TcpStream`/`UnixStream` 命中 0）——天然最紧档案候选。
- compositor 为现成模板：seccomp+landlock、线程时序、真实 restrict 回归（M3 切片 1）。
- renderer 与 compositor 的本质差异：**renderer 直连网络**（fetch 在 renderer 进程，生产日志 `[renderer-1] renderer fetch start` 证）——compositor 的 seccomp 网络过滤不能照搬，renderer 档案须诚实缩限为 exec 过滤 + landlock 文件面（网络面收口需先有 fetch_proxy 出网收窄设计，另立切片不捆绑）。

### 选项

| 选项 | 内容 | 评价 |
|---|---|---|
| **③A 两 helper 分两切片补齐（推荐）** | 先 image-decoder（landlock 空 allowed-set（restrict 后全拒 fs）+ seccomp exec 过滤；无 fs/socket 依赖故风险最低、验收最简单），后 renderer（landlock 允许清单：字体 ro、profile/缓存 rw、/dev/shm fd socket rw；seccomp exec 过滤不动网络） | 由易到难；image-decoder 切片可先行独立验收（DC-4 部分） |
| ③B 仅 image-decoder | renderer 维持 P3 占位记风险 | renderer 是威胁模型核心角色，只做一半价值有限 |
| ③C 维持现状 | P3 占位 + image-decoder 裸奔记剩余风险 | DC-4 长期 PARTIAL |

### 验收形态

镜像 M3 切片 1：真实 restrict 单元回归 + 真实进程安装日志（成功无降级）+ 受限操作验收（image-decoder 打开任意文件 EACCES；renderer 保护树外读 EACCES）+ 功能不回退（解码/页面管线全通）。

## ① CDP 默认强制 token（DC-3 剩余项，产品时序决策）

opt-in `--remote-debugging-token` 已交付（切片 4）。默认强制的取舍：破坏既有自动化客户端（本仓生产脚本消费者已核对全为 127.0.0.1 Host 形态，外部客户端未知）；候选：a) 直接默认强制 + 迁移说明；b) 宽限期（默认放行但每次启动 warn 明示倒计时）；c) 维持 opt-in 记剩余风险。决策性质为发布时序，无设计阻塞，拍板即一行级变更+回归调整。

## ④ 平台缺沙箱能力产品策略

选项 A/B/C 表见 [M4 前置](evidence/2026-11-01-m4-preflight-platform-policy-dc-audit.md)（推荐 C：随 ③ 统一拍板，每个进程档案落地时定该进程 fail 语义；compositor 维持 fail-open + warn）。与 ③ 联动：③A image-decoder 切片落地时同时定 image-decoder 的 fail 语义（推荐 fail-closed——新档案新语义，老内核降级明确），renderer 切片落地时同理。

## 落地顺序建议（拍板后）

1. ②（②A-env）：小切片，触点为本 goal 新 counter-plugin 文件 + runner 派生链 env 注入点；不碰 shim 三文件（若拍板 ②C' 全收口，另含 part02.js 单点闭包化，需与 html-semantics 流一次协调）。
2. ③A-image-decoder：最小档案，连同 ④ 的该进程 fail 语义一起落。
3. ③A-renderer：档案调参面大（网络/字体/缓存/fd socket），独立切片。
4. ①：随时可做，与 ②③④ 无依赖。
5. 全部落定后 M4 收口：DC-5 增量复测 + 剩余风险报告。

每切片独立可撤销、不删证据（goal 文档执行交接约束）；② 的 counter-plugin 若与 html-semantics 流新增的 shim 出口钩子相遇，按 §9 碰头信号记入 master.md 暂停协调，不硬解。
