# W3C WebDriver endpoint 兼容性清单基线

**建立日期**: 2026-09-08（M1 切片 1）
**对照标准**: [W3C WebDriver 规范](https://w3c.github.io/webdriver/)（Living Standard，2026-09 快照）
**代码事实源**: `apps/webdriver/src/main.rs` + `apps/webdriver/src/session.rs` + `apps/renderer/src/automation.rs`
**状态定义**: ✅ 已实现（有全链路测试）｜🟡 部分（实现存在但行为与规范有偏差或无测试）｜⬜ 缺失

维护规则：每个 endpoint 落地时更新本表对应行（状态 + 测试指针 + 行为注记），持续追加不删行。

## 会话（Sessions）

| Endpoint | 方法 | 状态 | 测试 | 行为注记 |
|---|---|---|---|---|
| /session | POST | ✅ | `http_session.rs::webdriver_session_lifecycle` | capabilities 固定返回 browserName=browserVersion=zero-product-version；不接受 firstMatch/alwaysMatch（规范要求 capabilities 协商，M-偏差见下） |
| /session/{id} | GET | ⬜ | — | capabilities 回读 |
| /session/{id}/delete | DELETE | ✅ | 同上 | 规范路径 DELETE /session/{id}（本实现同形）；删除后所有命令 → 404 no such session |
| /status | GET | ⬜ | — | ready/message/quit 三字段 |

## 导航（Navigation）

| Endpoint | 方法 | 状态 | 测试 | 行为注记 |
|---|---|---|---|---|
| /session/{id}/url | POST | ✅ | `webdriver_session_lifecycle` | 等待 LoadComplete（15s 超时→timeout 错误）；LoadFailed → unknown error |
| /session/{id}/url | GET | ⬜ | — | 当前 URL；renderer UrlChanged 已有，仅 webdriver 层缺 |
| back | POST | ⬜ | — | RendererHandle::go_back 已有 |
| forward | POST | ⬜ | — | RendererHandle::go_forward 已有 |
| refresh | POST | ⬜ | — | renderer Reload IPC 已有 |

## 命令执行（Command Execution）

| Endpoint | 方法 | 状态 | 测试 | 行为注记 |
|---|---|---|---|---|
| /session/{id}/execute/sync | POST | ✅ | `webdriver_drives_live_form_controls` | 脚本包 function wrapper + JSON.stringify 挣值；args 经 AutomationValue；DOM mutation 会同步回 live document |
| /session/{id}/execute/async | POST | ⬜ | — | 需 async 脚本语义 + 完成回调 |
| /session/{id}/source | GET | ⬜ | — | cached_html 可直接回读（约等于解析后 DOM 序列化） |

## 元素定位（Element Retrieval）

| Endpoint | 方法 | 状态 | 测试 | 行为注记 |
|---|---|---|---|---|
| /session/{id}/element | POST | ✅ | `webdriver_rejects_missing_and_stale_element_references` | 仅 css selector；其他策略 → 400 invalid argument（规范还有 link text/partial link text/tag name/xpath） |
| /session/{id}/elements | POST | ⬜ | — | 复数版；dom query_selector_all 已有 |
| /session/{id}/element/active | GET | ✅ | `webdriver_drives_live_form_controls` | 无焦点 → value: null（规范如此） |
| /session/{id}/element/{ref}/text | GET | ⬜ | — | |
| /session/{id}/element/{ref}/rect | GET | ⬜ | — | shim `__zw_getBoundingClientRect` 真值可用 |
| /session/{id}/element/{ref}/enabled | GET | ⬜ | — | |
| /session/{id}/element/{ref}/selected | GET | ⬜ | — | |
| /session/{id}/element/{ref}/attribute/{name} | GET | ⬜ | — | |
| /session/{id}/element/{ref}/property/{name} | GET | ⬜ | — | |
| /session/{id}/element/{ref}/css/{name} | GET | ⬜ | — | shim getComputedStyle 真值可用（part01.js:3391） |
| /session/{id}/element/{ref}/clear | POST | ⬜ | — | |

## 元素交互（Element Interaction）

| Endpoint | 方法 | 状态 | 测试 | 行为注记 |
|---|---|---|---|---|
| /session/{id}/element/{ref}/click | POST | ✅ | `webdriver_drives_live_form_controls` | 含 label 语义、checkedness 联动（automation_click） |
| /session/{id}/element/{ref}/clear | POST | ⬜ | — | （同上表 clear，归入交互族） |
| /session/{id}/element/{ref}/value | POST | ✅ | 同上 | text 字段（兼容数组形式）；修饰键/特殊键 parse_webdriver_keys；发送前自动聚焦（规范 focus 语义） |

## 脚本超时（Script Timeouts）

| Endpoint | 方法 | 状态 | 测试 | 行为注记 |
|---|---|---|---|---|
| /session/{id}/timeouts | POST | ⬜ | — | script/pageLoad/implicit；当前硬编码 15s nav / 10s automation |
| /session/{id}/timeouts | GET | ⬜ | — | |

## 窗口（Contexts/Window）

| Endpoint | 方法 | 状态 | 测试 | 行为注记 |
|---|---|---|---|---|
| /session/{id}/window | GET/DELETE | ⬜ | — | 单窗口架构，GET 可返当前 handle；DELETE(close) 语义待定 |
| /session/{id}/window/handles | GET | ⬜ | — | 单窗口 → 单元素 |
| /session/{id}/window/new | POST | ⬜ | — | 多 tab 不在范围 |
| /session/{id}/window/handle | GET | ⬜ | — | |
| /session/{id}/window/rect | GET/POST | ⬜ | — | SetViewport 已有（800x600 固定） |
| /session/{id}/window/maximize | POST | ⬜ | — | |
| /session/{id}/window/minimize | POST | ⬜ | — | |
| /session/{id}/window/fullscreen | POST | ⬜ | — | |
| /session/{id}/frame(s) | POST | ⬜ | — | iframe 深化依赖 renderer |
| /session/{id}/frame/parent | POST | ⬜ | — | |

## 截图（Screen Capture）

| Endpoint | 方法 | 状态 | 测试 | 行为注记 |
|---|---|---|---|---|
| /session/{id}/screenshot | GET | ⬜ | — | 待评估 renderer 截图能力（M3，待用户决策清单） |
| /session/{id}/element/{ref}/screenshot | GET | ⬜ | — | 同上 |

## 用户提示（User Prompts）— 明确排除

| Endpoint | 方法 | 状态 | 行为注记 |
|---|---|---|---|
| /alert/*（text/text/dismiss/accept） | — | ⬜ 排除 | 依赖 host 对话框能力；已记「待用户决策」 |

## 动作（Actions）— 明确排除

| Endpoint | 方法 | 状态 | 行为注记 |
|---|---|---|---|
| /session/{id}/actions | POST | ⬜ 排除 | 完整指针/触摸语义依赖输入管线深化；keyboard 够用子集已由 send keys 覆盖 |

## Wire Format 对照（DC-1 第三项）

W3C 响应包络：`{"value": <result>}`；错误 `{"value": {"error": <code>, "message": <msg>, "stacktrace": ""}}`。

| 项 | 状态 | 注记 |
|---|---|---|
| 成功包络 {"value": ...} | ✅ | 全部已实现 endpoint 遵循 |
| 错误包络 error+message | ✅ | error_response() |
| stacktrace 字段 | 🟡 | 缺失（规范要求必带，可空串）——M1 切片 2 顺手补 |
| 404 no such session / no such element / stale element reference | ✅ | driver_error_response |
| 400 invalid argument | ✅ | |
| 408 timeout | 🟡 | 现映射 500 unknown error——M1 切片 3 修正为 408 |
| 500 unknown error / unsupported operation | ✅ | |
| 200 无 value 字段的 unknown command | 🟡 | 未知路由返 404 + `unknown command`（规范该场景应为 404 unknown command，✅ 一致）；但 DELETE 后 GET title 已验证 404 no such session ✅ |

## 测试执行

- 全链路：`cargo test -p zero-webdriver`（真实 TCP + 真实 zero-renderer 子进程）
- 单元：`session.rs` 内嵌 tests（webdriver keys 解析、service worker 桩响应）
- CI：接线见缺口清单 P1（M1 切片 3）
