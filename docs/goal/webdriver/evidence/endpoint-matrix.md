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
| /session/{id} | GET | ✅ | `webdriver_status_and_capabilities_readback` | capabilities 回读（browserName/browserVersion 固定值，与 New Session 一致）；未知 session → 404 |
| /session/{id}/delete | DELETE | ✅ | 同上 | 规范路径 DELETE /session/{id}（本实现同形）；删除后所有命令 → 404 no such session |
| /status | GET | ✅ | 同上 | ready/message/quit 三字段；ready 恒 true（session 按需 spawn renderer） |

## 导航（Navigation）

| Endpoint | 方法 | 状态 | 测试 | 行为注记 |
|---|---|---|---|---|
| /session/{id}/url | POST | ✅ | `webdriver_session_lifecycle` | 等待 LoadComplete（超时→408 timeout）；LoadFailed → unknown error |
| /session/{id}/url | GET | ✅ | `webdriver_navigation_family_url_back_forward_refresh` | 会话态 URL，初值 about:blank；跟随 renderer UrlChanged（重定向真值） |
| back | POST | ✅ | 同上 | RendererHandle::go_back；历史边界 no-op → 返回成功（决策见 master.md）；等待 LoadComplete |
| forward | POST | ✅ | 同上 | RendererHandle::go_forward；语义同 back |
| refresh | POST | ✅ | 同上 | renderer Reload IPC；导航语义（epoch+1）等待 LoadComplete |

## 命令执行（Command Execution）

| Endpoint | 方法 | 状态 | 测试 | 行为注记 |
|---|---|---|---|---|
| /session/{id}/execute/sync | POST | ✅ | `webdriver_drives_live_form_controls` | 脚本包 function wrapper + JSON.stringify 挣值；args 经 AutomationValue；DOM mutation 会同步回 live document |
| /session/{id}/execute/async | POST | ⬜ | — | 需 async 脚本语义 + 完成回调 |
| /session/{id}/source | GET | ✅ | `webdriver_find_elements_and_page_source` | ExecuteScript documentElement.outerHTML（live DOM 序列化，含脚本 mutation 后状态） |

## 元素定位（Element Retrieval）

| Endpoint | 方法 | 状态 | 测试 | 行为注记 |
|---|---|---|---|---|
| /session/{id}/element | POST | ✅ | `webdriver_rejects_missing_and_stale_element_references` | 仅 css selector；其他策略 → 400 invalid argument（规范还有 link text/partial link text/tag name/xpath） |
| /session/{id}/elements | POST | ✅ | `webdriver_find_elements_and_page_source` | 复数版；空匹配返回空数组（W3C 非错误）；仅 css selector |
| /session/{id}/element/active | GET | ✅ | `webdriver_drives_live_form_controls` | 无焦点 → value: null（规范如此） |
| /session/{id}/element/{ref}/text | GET | ✅ | `webdriver_element_state_family_reads_live_document` | textContent 近似（可见性过滤未实现，FIXME 记录） |
| /session/{id}/element/{ref}/rect | GET | ✅ | 同上 | shim gBCR 真值（RectBridge 注册时）；x/y/width/height |
| /session/{id}/element/{ref}/enabled | GET | ✅ | 同上 | disabled 属性/属性存在性取反；非控件恒 true |
| /session/{id}/element/{ref}/selected | GET | ✅ | 同上 | option.selected / checkbox·radio.checked |
| /session/{id}/element/{ref}/attribute/{name} | GET | ✅ | 同上 | 内容属性；缺失 → null |
| /session/{id}/element/{ref}/property/{name} | GET | ✅ | 同上 | DOM 属性直读（live value 反映输入）；undefined → null |
| /session/{id}/element/{ref}/css/{name} | GET | ✅ | 同上 | shim getComputedStyle → host 计算样式真值 |
| /session/{id}/element/{ref}/clear | POST | ✅ | 同上 | 可编辑元素置空 value + input 事件；checkbox/radio/file 跳过 |

## 元素交互（Element Interaction）

| Endpoint | 方法 | 状态 | 测试 | 行为注记 |
|---|---|---|---|---|
| /session/{id}/element/{ref}/click | POST | ✅ | `webdriver_drives_live_form_controls` | 含 label 语义、checkedness 联动（automation_click） |
| /session/{id}/element/{ref}/clear | POST | ✅ | `webdriver_element_state_family_reads_live_document` | （同元素状态族表 clear 行——可编辑元素置空 + input 事件） |
| /session/{id}/element/{ref}/value | POST | ✅ | `webdriver_drives_live_form_controls` | text 字段（兼容数组形式）；修饰键/特殊键 parse_webdriver_keys；发送前自动聚焦（规范 focus 语义） |

## 脚本超时（Script Timeouts）

| Endpoint | 方法 | 状态 | 测试 | 行为注记 |
|---|---|---|---|---|
| /session/{id}/timeouts | POST | ✅ | `webdriver_timeouts_roundtrip_and_validation` | 字段可选只更新出现的字段；非数值 → 400 invalid argument；默认值偏离 W3C（10s/15s/0s fail-fast，见 session.rs SessionTimeouts 注记） |
| /session/{id}/timeouts | GET | ✅ | 同上 | script/pageLoad/implicit 毫秒值 |

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
| stacktrace 字段 | ✅ | M1 切片 2 补齐（恒空串；renderer 无 JS stack 捕获通道） |
| 404 no such session / no such element / stale element reference | ✅ | driver_error_response |
| 400 invalid argument | ✅ | |
| 408 timeout | ✅ | M1 切片 3 修正（原 500） |
| 500 unknown error / unsupported operation | ✅ | |

## 测试执行

- 全链路：`cargo test -p zero-webdriver`（真实 TCP + 真实 zero-renderer 子进程）
- 单元：`session.rs` 内嵌 tests（webdriver keys 解析、service worker 桩响应）
- CI：接线见缺口清单 P1（M1 切片 3）
